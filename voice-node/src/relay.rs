use crate::jitter::{BufferedPacket, JitterBuffer};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tracing::debug;

/// Per-channel state: members, jitter buffer, and sender tracking.
pub struct ChannelState {
    /// member address → last packet time
    pub members: HashMap<SocketAddr, Instant>,
    /// reorders and smooths voice packets
    pub jitter: JitterBuffer,
    /// voice_seq → original sender address (for relay after pop)
    pub senders: HashMap<u32, SocketAddr>,
}

/// Maps channel_id → per-channel state.
pub type ChannelMap = Arc<RwLock<HashMap<u64, ChannelState>>>;

pub fn new_channel_map() -> ChannelMap {
    Arc::new(RwLock::new(HashMap::new()))
}

/// Drop members with no packets for longer than `max_idle`.
pub async fn cleanup_stale(channels: &ChannelMap, max_idle: Duration) {
    let cutoff = Instant::now() - max_idle;
    let mut lock = channels.write().await;
    lock.retain(|_, state| {
        state.members.retain(|_, last_seen| *last_seen >= cutoff);
        !state.members.is_empty()
    });
}

/// Push a raw voice packet into the jitter buffer for `channel_id`.
pub async fn push_packet(
    channels: &ChannelMap,
    channel_id: u64,
    sender: SocketAddr,
    voice_seq: u32,
    timestamp: u32,
    raw_data: &[u8],
    arrived_at: u64,
) {
    let pkt = BufferedPacket {
        voice_seq,
        timestamp,
        channel_id,
        opus_data: raw_data.to_vec(),
        arrived_at,
    };
    let mut lock = channels.write().await;
    let state = lock.entry(channel_id).or_insert_with(|| ChannelState {
        members: HashMap::new(),
        jitter: JitterBuffer::new(40, true),
        senders: HashMap::new(),
    });
    state.jitter.push(pkt);
    state.senders.insert(voice_seq, sender);
}

/// Pop ready packets from every channel's jitter buffer and relay them.
pub async fn pop_and_relay(socket: &UdpSocket, channels: &ChannelMap, now_ms: u64) {
    let mut lock = channels.write().await;
    for (_channel_id, state) in lock.iter_mut() {
        while let Some(pkt) = state.jitter.pop_ready(now_ms) {
            let sender = state.senders.remove(&pkt.voice_seq).unwrap_or_else(|| {
                tracing::warn!("sender not found for seq={}", pkt.voice_seq);
                SocketAddr::from(([0, 0, 0, 0], 0))
            });
            for &addr in state.members.keys() {
                if addr != sender
                    && let Err(e) = socket.send_to(&pkt.opus_data, addr).await
                {
                    debug!("relay send error to {addr}: {e}");
                }
            }
        }
    }
}

/// Register (or refresh) a sender in the channel member set.
pub async fn touch_member(channels: &ChannelMap, channel_id: u64, addr: SocketAddr) {
    channels
        .write()
        .await
        .entry(channel_id)
        .or_insert_with(|| ChannelState {
            members: HashMap::new(),
            jitter: JitterBuffer::new(40, true),
            senders: HashMap::new(),
        })
        .members
        .insert(addr, Instant::now());
}
