use anyhow::Result;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    domain::{permissions::Permissions, session},
    net::{io, state::State},
    proto::{
        GuildMemberJoinPayload, GuildMemberKickPayload, GuildMemberLeavePayload, PacketId,
        SessionCrypto, to_payload,
    },
};

use super::send_err;

pub async fn handle_guild_member_join(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    session_id: &str,
    payload: &[u8],
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let req: GuildMemberJoinPayload = serde_json::from_slice(payload)?;
    let sess = session::get(&state.sessions, session_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("session not found"))?;

    state
        .storage
        .add_guild_member(&req.guild_id, &sess.user_id)
        .await?;

    let roles = state
        .storage
        .get_user_roles(&req.guild_id, &sess.user_id)
        .await?;
    let color = roles
        .iter()
        .max_by_key(|r| r.position)
        .map(|r| r.color.clone())
        .unwrap_or_else(|| "#ffffff".into());

    io::send_encrypted(
        stream,
        PacketId::GuildMemberJoin,
        seq,
        &to_payload(&serde_json::json!({"guild_id": req.guild_id, "user_id": sess.user_id})),
        crypto,
    )
    .await?;

    io::send_encrypted(
        stream,
        PacketId::UserRoleUpdate,
        seq,
        &to_payload(
            &serde_json::json!({"user_id": sess.user_id, "guild_id": req.guild_id, "color": color}),
        ),
        crypto,
    )
    .await?;
    Ok(())
}

pub async fn handle_guild_member_leave(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    session_id: &str,
    payload: &[u8],
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let req: GuildMemberLeavePayload = serde_json::from_slice(payload)?;
    let sess = session::get(&state.sessions, session_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("session not found"))?;

    let target = if req.user_id.is_empty() {
        &sess.user_id
    } else {
        &req.user_id
    };
    state
        .storage
        .remove_guild_member(&req.guild_id, target)
        .await?;
    io::send_encrypted(
        stream,
        PacketId::GuildMemberLeave,
        seq,
        &to_payload(&serde_json::json!({"guild_id": req.guild_id, "user_id": target})),
        crypto,
    )
    .await?;
    Ok(())
}

pub async fn handle_guild_member_kick(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    session_id: &str,
    payload: &[u8],
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let req: GuildMemberKickPayload = serde_json::from_slice(payload)?;
    let sess = session::get(&state.sessions, session_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("session not found"))?;

    if !super::require_perm(
        state,
        &req.guild_id,
        &sess.user_id,
        Permissions::KICK_MEMBERS,
    )
    .await?
    {
        send_err(
            stream,
            seq,
            crate::proto::ErrorCode::PermissionDenied,
            "missing KICK_MEMBERS permission",
            crypto,
        )
        .await?;
        return Ok(());
    }

    state
        .storage
        .remove_guild_member(&req.guild_id, &req.user_id)
        .await?;
    state
        .storage
        .append_audit_log(
            &req.guild_id,
            &sess.user_id,
            "MEMBER_KICK",
            Some(&req.user_id),
            Some("member"),
            None,
        )
        .await?;
    io::send_encrypted(
        stream,
        PacketId::GuildMemberKick,
        seq,
        &to_payload(&serde_json::json!({"guild_id": req.guild_id, "user_id": req.user_id})),
        crypto,
    )
    .await?;
    Ok(())
}
