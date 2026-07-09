use super::packet::{PacketHeader, PacketId};
use serde::Serialize;

/// Encode a packet: header + JSON payload bytes.
pub fn encode_packet(id: PacketId, seq: u32, payload: &[u8]) -> Vec<u8> {
    let header = PacketHeader::new(id, seq, payload.len() as u32);
    let mut out = Vec::with_capacity(PacketHeader::SIZE + payload.len());
    out.extend_from_slice(&header.to_bytes());
    out.extend_from_slice(payload);
    out
}

/// Serialize a payload struct to JSON bytes.
pub fn to_payload<T: Serialize>(v: &T) -> Vec<u8> {
    serde_json::to_vec(v).expect("payload serialization is infallible")
}
