use std::collections::BTreeMap;

pub mod adaptive;
pub mod relay;

pub use adaptive::{JITTER_WINDOW, LOSS_WINDOW_PACKETS, MAX_TARGET_MS, MIN_TARGET_MS};

pub struct JitterBuffer {
    target_ms: u32,
    adaptive: bool,
    packets: BTreeMap<u32, BufferedPacket>,
    last_played: Option<u32>,
    arrival_times: Vec<u64>,
    arrival_idx: usize,
    arrival_count: usize,
    loss_count: u32,
    total_expected: u32,
    stable_windows: u32,
}

impl JitterBuffer {
    pub fn new(target_ms: u32, adaptive: bool) -> Self {
        Self {
            target_ms,
            adaptive,
            packets: BTreeMap::new(),
            last_played: None,
            arrival_times: vec![0u64; JITTER_WINDOW],
            arrival_idx: 0,
            arrival_count: 0,
            loss_count: 0,
            total_expected: 0,
            stable_windows: 0,
        }
    }

    pub fn push(&mut self, pkt: BufferedPacket) {
        let arrived = pkt.arrived_at;
        self.arrival_times[self.arrival_idx] = arrived;
        self.arrival_idx = (self.arrival_idx + 1) % JITTER_WINDOW;
        if self.arrival_count < JITTER_WINDOW {
            self.arrival_count += 1;
        }

        let gap = self
            .packets
            .keys()
            .next_back()
            .map(|&highest| {
                if pkt.voice_seq > highest {
                    pkt.voice_seq.wrapping_sub(highest).saturating_sub(1)
                } else {
                    0
                }
            })
            .unwrap_or(0);

        if gap > 0 {
            self.loss_count += gap;
        }
        self.total_expected += 1 + gap;
        self.packets.insert(pkt.voice_seq, pkt);

        if self.adaptive && self.total_expected >= LOSS_WINDOW_PACKETS {
            self.adapt_target();
        }
    }

    pub fn len(&self) -> usize {
        self.packets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }

    pub fn set_target_ms(&mut self, target_ms: u32) {
        self.target_ms = target_ms.clamp(MIN_TARGET_MS, MAX_TARGET_MS);
    }

    pub fn target_ms(&self) -> u32 {
        self.target_ms
    }

    pub fn is_adaptive(&self) -> bool {
        self.adaptive
    }
}

#[derive(Debug)]
pub struct BufferedPacket {
    pub voice_seq: u32,
    pub timestamp: u32,
    pub channel_id: u64,
    pub opus_data: Vec<u8>,
    pub arrived_at: u64,
}

#[cfg(test)]
mod tests;
