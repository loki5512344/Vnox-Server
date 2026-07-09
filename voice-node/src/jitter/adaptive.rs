use tracing::info;

use super::JitterBuffer;

pub const MIN_TARGET_MS: u32 = 20;
pub const MAX_TARGET_MS: u32 = 150;
pub const JITTER_WINDOW: usize = 64;
const LOSS_INCREASE_THRESH: u32 = 5;
const LOSS_DECREASE_THRESH: u32 = 1;
const STABLE_WINDOWS_BEFORE_DECREASE: u32 = 10;
pub const LOSS_WINDOW_PACKETS: u32 = 50;
const ADJUST_STEP_MS: u32 = 5;

impl JitterBuffer {
    pub(super) fn observed_jitter_ms(&self) -> u32 {
        if self.arrival_count < 2 {
            return 0;
        }
        let mut deltas = Vec::with_capacity(self.arrival_count - 1);
        for i in 0..self.arrival_count - 1 {
            let idx0 = (self.arrival_idx + JITTER_WINDOW - self.arrival_count + i) % JITTER_WINDOW;
            let idx1 =
                (self.arrival_idx + JITTER_WINDOW - self.arrival_count + i + 1) % JITTER_WINDOW;
            let delta = self.arrival_times[idx1].saturating_sub(self.arrival_times[idx0]);
            deltas.push(delta);
        }
        if deltas.is_empty() {
            return 0;
        }
        deltas.sort_unstable();
        let p90_idx = ((deltas.len() as f64) * 0.90).ceil() as usize - 1;
        let p90_idx = p90_idx.min(deltas.len() - 1);
        deltas[p90_idx] as u32
    }

    pub(super) fn adapt_target(&mut self) {
        let loss_pct = if self.total_expected > 0 {
            (self.loss_count as f64 / self.total_expected as f64 * 100.0) as u32
        } else {
            0
        };
        let jitter_ms = self.observed_jitter_ms();
        let jitter_based_target = (jitter_ms * 2).clamp(MIN_TARGET_MS, MAX_TARGET_MS);

        if loss_pct >= LOSS_INCREASE_THRESH {
            let new_target = (self.target_ms + ADJUST_STEP_MS).min(MAX_TARGET_MS);
            if new_target != self.target_ms {
                info!(
                    target_ms = new_target,
                    loss_pct, jitter_ms, "adaptive: increasing target due to packet loss"
                );
                self.target_ms = new_target;
            }
            self.stable_windows = 0;
        } else if loss_pct <= LOSS_DECREASE_THRESH {
            self.stable_windows += 1;
            if self.stable_windows >= STABLE_WINDOWS_BEFORE_DECREASE {
                let candidate = self.target_ms.saturating_sub(ADJUST_STEP_MS);
                let new_target = candidate.max(jitter_based_target).max(MIN_TARGET_MS);
                if new_target != self.target_ms {
                    info!(
                        target_ms = new_target,
                        loss_pct, jitter_ms, "adaptive: decreasing target, link stable"
                    );
                    self.target_ms = new_target;
                }
                self.stable_windows = 0;
            }
        } else {
            self.stable_windows = 0;
        }
        self.loss_count = 0;
        self.total_expected = 0;
    }
}
