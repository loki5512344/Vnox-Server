use super::{BufferedPacket, JitterBuffer};

impl JitterBuffer {
    pub fn pop_ready(&mut self, now_ms: u64) -> Option<BufferedPacket> {
        if self.adaptive {
            return self.pop_ready_adaptive(now_ms);
        }
        self.pop_ready_fixed(now_ms)
    }

    pub fn has_gap(&self) -> bool {
        match self.last_played {
            None => false,
            Some(last) => {
                let expected = last.wrapping_add(1);
                !self.packets.contains_key(&expected) && !self.packets.is_empty()
            }
        }
    }

    fn pop_ready_fixed(&mut self, now_ms: u64) -> Option<BufferedPacket> {
        let seq = *self.packets.keys().next()?;
        let pkt = self.packets.get(&seq)?;
        let buffered_for = now_ms.saturating_sub(pkt.arrived_at);
        if buffered_for >= self.target_ms as u64 {
            let pkt = self.packets.remove(&seq)?;
            self.last_played = Some(seq);
            return Some(pkt);
        }
        None
    }

    fn pop_ready_adaptive(&mut self, now_ms: u64) -> Option<BufferedPacket> {
        let seq = *self.packets.keys().next()?;
        let pkt = self.packets.get(&seq)?;
        let buffered_for = now_ms.saturating_sub(pkt.arrived_at);
        if buffered_for >= self.target_ms as u64 {
            let pkt = self.packets.remove(&seq)?;
            self.last_played = Some(seq);
            return Some(pkt);
        }
        None
    }
}
