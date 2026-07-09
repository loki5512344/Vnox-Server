use anyhow::Result;
use tokio::net::TcpStream;
use tracing::debug;

use crate::{
    domain::session,
    net::{io, state::State},
    proto::{GuildCreatePayload, GuildInfo, PacketId, SessionCrypto, to_payload},
};

use super::send_err;

pub async fn handle_guild_create(
    stream: &mut TcpStream,
    seq: &mut u32,
    session_id: &str,
    payload: &[u8],
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let req: GuildCreatePayload = serde_json::from_slice(payload)?;
    let sess = session::get(&state.sessions, session_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("session not found"))?;

    if req.name.len() < 2 || req.name.len() > 100 {
        send_err(
            stream,
            seq,
            crate::proto::ErrorCode::InvalidPacket,
            "guild name must be 2-100 chars",
            crypto,
        )
        .await?;
        return Ok(());
    }

    let guild_id = state.storage.create_guild(&sess.user_id, &req.name).await?;
    let guild = state
        .storage
        .get_guild(&guild_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("guild not found after create"))?;

    let info = GuildInfo {
        id: guild.id.clone(),
        owner_id: guild.owner_id,
        name: guild.name,
        member_count: guild.member_count,
        created_at: guild.created_at,
    };

    io::send_encrypted(
        stream,
        PacketId::GuildCreate,
        seq,
        &to_payload(&info),
        crypto,
    )
    .await?;

    debug!(
        "guild: {} created {} (owner={})",
        sess.nickname, guild_id, sess.user_id
    );
    Ok(())
}

pub async fn handle_guild_delete(
    stream: &mut TcpStream,
    seq: &mut u32,
    session_id: &str,
    payload: &[u8],
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    #[derive(serde::Deserialize)]
    struct Req {
        guild_id: String,
    }
    let req: Req = serde_json::from_slice(payload)?;
    let sess = session::get(&state.sessions, session_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("session not found"))?;

    let guild = state
        .storage
        .get_guild(&req.guild_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("guild not found"))?;
    if guild.owner_id != sess.user_id {
        send_err(
            stream,
            seq,
            crate::proto::ErrorCode::PermissionDenied,
            "only owner can delete guild",
            crypto,
        )
        .await?;
        return Ok(());
    }

    state.storage.delete_guild(&req.guild_id).await?;
    state
        .storage
        .append_audit_log(
            &req.guild_id,
            &sess.user_id,
            "GUILD_DELETE",
            None,
            None,
            None,
        )
        .await?;
    io::send_encrypted(
        stream,
        PacketId::GuildDelete,
        seq,
        &to_payload(&serde_json::json!({"guild_id": req.guild_id})),
        crypto,
    )
    .await?;
    Ok(())
}
