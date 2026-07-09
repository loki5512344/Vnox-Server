use anyhow::Result;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    domain::session,
    net::{io, state::BroadcastMsg, state::State},
    proto::SessionCrypto,
};

pub async fn deliver_encrypted(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    session_id: &str,
    msg: &BroadcastMsg,
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    if msg.exclude_session.as_deref() == Some(session_id) {
        return Ok(());
    }
    if let Some(ref target) = msg.target_session_id {
        if target != session_id {
            return Ok(());
        }
        io::deliver_encrypted(stream, crypto, seq, &msg.data).await?;
        return Ok(());
    }
    if let Some(ref ch) = msg.channel_id {
        let in_ch = session::get(&state.sessions, session_id)
            .await
            .and_then(|s| s.channel_id)
            .as_deref()
            == Some(ch.as_str());
        if !in_ch {
            return Ok(());
        }
    }
    io::deliver_encrypted(stream, crypto, seq, &msg.data).await
}
