use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::relay;
use crate::{
    Config, MAX_UDP_PACKET, PLAYOUT_INTERVAL_MS, VOICE_HDR_SIZE, VOICE_PACKET_ID, VoiceHeader,
};

pub async fn run(config: Config) -> anyhow::Result<()> {
    let (_, rx) = broadcast::channel(1);
    run_bind(&config.node.name, &config.voice.bind, rx).await
}

pub async fn run_bind(
    node_name: &str,
    bind: &str,
    mut voice_member_rx: broadcast::Receiver<String>,
) -> anyhow::Result<()> {
    info!("VNOX Voice Node starting — node: {node_name}");
    info!("UDP bind: {bind}");

    let socket = UdpSocket::bind(bind).await?;
    info!("voice node listening on {bind}");

    let channels = relay::new_channel_map();
    let user_map = relay::new_user_map();

    let channels_cleanup = channels.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15));
        loop {
            interval.tick().await;
            relay::cleanup_stale(&channels_cleanup, Duration::from_secs(30)).await;
        }
    });

    let channels_events = channels.clone();
    let user_map_events = user_map.clone();
    tokio::spawn(async move {
        while let Ok(event_str) = voice_member_rx.recv().await {
            if let Ok(event) = serde_json::from_str::<serde_json::Value>(&event_str) {
                let event_type = event["type"].as_str().unwrap_or("");
                let channel_id = event["channel_id"].as_str().unwrap_or("");
                let user_id = event["user_id"].as_str().unwrap_or("");
                match event_type {
                    "join" => {
                        let endpoint_str = match event["endpoint"].as_str() {
                            Some(s) => s,
                            None => continue,
                        };
                        let addr = match endpoint_str.parse::<SocketAddr>() {
                            Ok(a) => a,
                            Err(_) => continue,
                        };
                        relay::add_user_mapping(&user_map_events, user_id.to_string(), addr).await;
                        if let Ok(ch) = channel_id.parse::<u64>() {
                            relay::touch_member(&channels_events, ch, addr).await;
                        }
                    }
                    "leave" => {
                        if let Some(addr) =
                            relay::remove_user_mapping(&user_map_events, user_id).await
                        {
                            relay::remove_member_from_all(&channels_events, &addr).await;
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    let socket = Arc::new(socket);
    let socket_relay = socket.clone();
    let channels_playout = channels.clone();
    let epoch = Instant::now();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(PLAYOUT_INTERVAL_MS));
        loop {
            interval.tick().await;
            let now_ms = epoch.elapsed().as_millis() as u64;
            relay::pop_and_relay(socket_relay.as_ref(), &channels_playout, now_ms).await;
        }
    });

    let mut buf = vec![0u8; MAX_UDP_PACKET];

    loop {
        let (len, src) = match socket.recv_from(&mut buf).await {
            Ok(r) => r,
            Err(e) => {
                error!("UDP recv error: {e}");
                continue;
            }
        };

        let data = &buf[..len];

        let hdr = match VoiceHeader::parse(data) {
            Some(h) => h,
            None => {
                warn!("short packet from {src} ({len} bytes), dropping");
                continue;
            }
        };

        if hdr.packet_id != VOICE_PACKET_ID {
            debug!(
                "non-voice packet 0x{:04X} from {src}, dropping",
                hdr.packet_id
            );
            continue;
        }

        relay::touch_member(&channels, hdr.channel_id, src).await;

        let arrived_at = epoch.elapsed().as_millis() as u64;

        debug!(
            "voice seq={} ch={} from {src} ({} bytes opus)",
            hdr.voice_seq,
            hdr.channel_id,
            len - VOICE_HDR_SIZE,
        );

        relay::push_packet(
            &channels,
            hdr.channel_id,
            src,
            hdr.voice_seq,
            hdr.timestamp,
            data,
            arrived_at,
        )
        .await;
    }
}
