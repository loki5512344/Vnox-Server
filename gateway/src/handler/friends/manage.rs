use anyhow::Result;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    domain::session,
    net::{io, state::State},
    proto::{
        BlockListPayload, BlockUserPayload, FriendRemovePayload, PacketId, SessionCrypto,
        UnblockUserPayload, to_payload,
    },
};

pub async fn handle_friend_remove(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    session_id: &str,
    payload: &[u8],
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let req: FriendRemovePayload = serde_json::from_slice(payload)?;
    let sess = session::get(&state.sessions, session_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("session not found"))?;

    state
        .storage
        .remove_friend(&sess.user_id, &req.user_id)
        .await?;

    io::send_encrypted(
        stream,
        PacketId::FriendRemove,
        seq,
        &to_payload(&serde_json::json!({"removed": true})),
        crypto,
    )
    .await?;
    Ok(())
}

pub async fn handle_block_user(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    session_id: &str,
    payload: &[u8],
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let req: BlockUserPayload = serde_json::from_slice(payload)?;
    let sess = session::get(&state.sessions, session_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("session not found"))?;

    state
        .storage
        .block_user(&sess.user_id, &req.user_id)
        .await?;

    io::send_encrypted(
        stream,
        PacketId::BlockUser,
        seq,
        &to_payload(&serde_json::json!({"blocked": true})),
        crypto,
    )
    .await?;
    Ok(())
}

pub async fn handle_unblock_user(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    session_id: &str,
    payload: &[u8],
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let req: UnblockUserPayload = serde_json::from_slice(payload)?;
    let sess = session::get(&state.sessions, session_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("session not found"))?;

    state
        .storage
        .unblock_user(&sess.user_id, &req.user_id)
        .await?;

    io::send_encrypted(
        stream,
        PacketId::UnblockUser,
        seq,
        &to_payload(&serde_json::json!({"unblocked": true})),
        crypto,
    )
    .await?;
    Ok(())
}

pub async fn handle_block_list(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    session_id: &str,
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let sess = session::get(&state.sessions, session_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("session not found"))?;

    let blocked = state.storage.list_blocks(&sess.user_id).await?;

    io::send_encrypted(
        stream,
        PacketId::BlockList,
        seq,
        &to_payload(&BlockListPayload { blocked }),
        crypto,
    )
    .await?;
    Ok(())
}
