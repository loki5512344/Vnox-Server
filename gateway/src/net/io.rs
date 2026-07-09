use anyhow::Result;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::proto::{
    ErrorCode, ErrorPayload, PacketHeader, PacketId, SessionCrypto, encode_packet, flags,
    to_payload,
};

const MAX_PAYLOAD: u32 = 4 * 1024 * 1024;

pub async fn send_packet(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    id: PacketId,
    seq: &mut u32,
    payload: &[u8],
) -> Result<()> {
    let data = encode_packet(id, *seq, payload);
    *seq = seq.wrapping_add(1);
    stream.write_all(&data).await?;
    Ok(())
}

pub async fn send_error(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    code: ErrorCode,
    msg: &str,
) -> Result<()> {
    let p = ErrorPayload {
        code: code as u32,
        message: msg.into(),
    };
    debug_assert!(ErrorCode::from_u32(p.code).is_some());
    send_packet(stream, PacketId::Error, seq, &to_payload(&p)).await
}

pub async fn read_packet(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
) -> Result<(PacketHeader, Vec<u8>)> {
    let mut buf = [0u8; PacketHeader::SIZE];
    stream.read_exact(&mut buf).await?;
    let hdr = PacketHeader::from_bytes(&buf);
    if hdr.flags & !flags::KNOWN_MASK != 0 {
        return Err(anyhow::anyhow!("unknown packet flags: 0x{:04X}", hdr.flags));
    }
    if hdr.payload_length > MAX_PAYLOAD {
        return Err(anyhow::anyhow!("payload too large: {}", hdr.payload_length));
    }
    let mut payload = vec![0u8; hdr.payload_length as usize];
    if !payload.is_empty() {
        stream.read_exact(&mut payload).await?;
    }
    Ok((hdr, payload))
}

pub async fn send_encrypted(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    id: PacketId,
    seq: &mut u32,
    payload: &[u8],
    crypto: &SessionCrypto,
) -> Result<()> {
    let encrypted = crypto.encrypt_s2c(*seq as u64, payload);
    let flags_val = flags::ENCRYPTED;
    let mut hdr_buf = [0u8; PacketHeader::SIZE];
    hdr_buf[0..2].copy_from_slice(&(id as u16).to_be_bytes());
    hdr_buf[2..4].copy_from_slice(&flags_val.to_be_bytes());
    hdr_buf[4..8].copy_from_slice(&seq.to_be_bytes());
    hdr_buf[8..12].copy_from_slice(&(encrypted.len() as u32).to_be_bytes());
    *seq = seq.wrapping_add(1);

    let mut out = Vec::with_capacity(PacketHeader::SIZE + encrypted.len());
    out.extend_from_slice(&hdr_buf);
    out.extend_from_slice(&encrypted);
    stream.write_all(&out).await?;
    Ok(())
}

pub async fn read_encrypted(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    crypto: &SessionCrypto,
) -> Result<(PacketHeader, Vec<u8>)> {
    let mut buf = [0u8; PacketHeader::SIZE];
    stream.read_exact(&mut buf).await?;
    let hdr = PacketHeader::from_bytes(&buf);
    if hdr.flags & !flags::KNOWN_MASK != 0 {
        return Err(anyhow::anyhow!("unknown packet flags: 0x{:04X}", hdr.flags));
    }
    if hdr.flags & flags::ENCRYPTED == 0 {
        return Err(anyhow::anyhow!("packet not encrypted"));
    }
    if hdr.payload_length > MAX_PAYLOAD {
        return Err(anyhow::anyhow!("payload too large: {}", hdr.payload_length));
    }
    let mut encrypted = vec![0u8; hdr.payload_length as usize];
    if !encrypted.is_empty() {
        stream.read_exact(&mut encrypted).await?;
    }
    let payload = crypto.decrypt_c2s(hdr.sequence as u64, &encrypted)?;
    Ok((hdr, payload))
}

pub async fn deliver_encrypted(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    crypto: &SessionCrypto,
    seq: &mut u32,
    raw_data: &[u8],
) -> Result<()> {
    if raw_data.len() < PacketHeader::SIZE {
        return Err(anyhow::anyhow!("deliver: packet too short"));
    }
    let raw_hdr = PacketHeader::from_bytes(raw_data[..PacketHeader::SIZE].try_into().unwrap());
    let pid = match PacketId::from_u16(raw_hdr.packet_id) {
        Some(p) => p,
        None => {
            return Err(anyhow::anyhow!(
                "deliver: unknown packet id 0x{:04X}",
                raw_hdr.packet_id
            ));
        }
    };
    let payload = &raw_data[PacketHeader::SIZE..];
    send_encrypted(stream, pid, seq, payload, crypto).await
}
