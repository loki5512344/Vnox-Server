use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tracing::{debug, error, info, warn};

use crate::relay;
use crate::{
    Config, MAX_UDP_PACKET, PLAYOUT_INTERVAL_MS, VOICE_HDR_SIZE, VOICE_PACKET_ID, VoiceHeader,
};

pub async fn run(config: Config) -> anyhow::Result<()> {
    run_bind(&config.node.name, &config.voice.bind).await
}

pub async fn run_bind(node_name: &str, bind: &str) -> anyhow::Result<()> {
    info!("VNOX Voice Node starting — node: {node_name}");
    info!("UDP bind: {bind}");

    let socket = UdpSocket::bind(bind).await?;
    info!("voice node listening on {bind}");

    let channels = relay::new_channel_map();

    let channels_cleanup = channels.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15));
        loop {
            interval.tick().await;
            relay::cleanup_stale(&channels_cleanup, Duration::from_secs(30)).await;
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
