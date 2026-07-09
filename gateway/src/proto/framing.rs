use super::packet::{PacketHeader, PacketId};
use prost::Message;

/// Encode a packet: header + protobuf payload bytes.
pub fn encode_packet(id: PacketId, seq: u32, payload: &[u8]) -> Vec<u8> {
    let header = PacketHeader::new(id, seq, payload.len() as u32);
    let mut out = Vec::with_capacity(PacketHeader::SIZE + payload.len());
    out.extend_from_slice(&header.to_bytes());
    out.extend_from_slice(payload);
    out
}

/// Encode a protobuf payload struct to bytes.
pub fn to_payload(msg: &impl Message) -> Vec<u8> {
    msg.encode_to_vec()
}
