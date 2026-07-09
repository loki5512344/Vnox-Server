pub mod flags;
mod id;

pub use id::{ErrorCode, PacketId};

#[derive(Debug, Clone)]
pub struct PacketHeader {
    pub packet_id: u16,
    pub flags: u16,
    pub sequence: u32,
    pub payload_length: u32,
}

impl PacketHeader {
    pub const SIZE: usize = 12;

    pub fn new(packet_id: PacketId, sequence: u32, payload_length: u32) -> Self {
        Self {
            packet_id: packet_id as u16,
            flags: 0,
            sequence,
            payload_length,
        }
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        buf[0..2].copy_from_slice(&self.packet_id.to_be_bytes());
        buf[2..4].copy_from_slice(&self.flags.to_be_bytes());
        buf[4..8].copy_from_slice(&self.sequence.to_be_bytes());
        buf[8..12].copy_from_slice(&self.payload_length.to_be_bytes());
        buf
    }

    pub fn from_bytes(buf: &[u8; Self::SIZE]) -> Self {
        Self {
            packet_id: u16::from_be_bytes([buf[0], buf[1]]),
            flags: u16::from_be_bytes([buf[2], buf[3]]),
            sequence: u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]),
            payload_length: u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
        }
    }
}
