use anyhow::Result;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    domain::session,
    net::io,
    proto::{FriendDeclinePayload, PacketId, SessionCrypto, to_payload},
};

pub async fn handle_friend_decline(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    session_id: &str,
    payload: &[u8],
    crypto: &SessionCrypto,
    state: &crate::net::state::State,
) -> Result<()> {
    let req: FriendDeclinePayload = serde_json::from_slice(payload)?;
    let sess = session::get(&state.sessions, session_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("session not found"))?;

    state
        .storage
        .decline_friend_request(&req.from_user_id, &sess.user_id)
        .await?;

    io::send_encrypted(
        stream,
        PacketId::FriendDecline,
        seq,
        &to_payload(&serde_json::json!({"status": "DECLINED"})),
        crypto,
    )
    .await?;
    Ok(())
}
