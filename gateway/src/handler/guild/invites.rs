use anyhow::Result;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    domain::{permissions::Permissions, session},
    net::{io, state::State},
    proto::{
        InviteAcceptPayload, InviteCreatePayload, InviteDeletePayload, InviteInfo, PacketId,
        SessionCrypto, to_payload,
    },
};

use super::{now_ms, send_err};

pub async fn handle_invite_create(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    session_id: &str,
    payload: &[u8],
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let req: InviteCreatePayload = serde_json::from_slice(payload)?;
    let sess = session::get(&state.sessions, session_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("session not found"))?;

    if !super::require_perm(
        state,
        &req.guild_id,
        &sess.user_id,
        Permissions::CREATE_INVITE,
    )
    .await?
    {
        send_err(
            stream,
            seq,
            crate::proto::ErrorCode::PermissionDenied,
            "missing CREATE_INVITE permission",
            crypto,
        )
        .await?;
        return Ok(());
    }

    let inv = state
        .storage
        .create_invite(
            &req.guild_id,
            &sess.user_id,
            req.max_uses,
            req.expires_in_seconds,
        )
        .await?;

    state
        .storage
        .append_audit_log(
            &req.guild_id,
            &sess.user_id,
            "INVITE_CREATE",
            Some(&inv.id),
            Some("invite"),
            None,
        )
        .await?;

    let info = InviteInfo {
        id: inv.id,
        guild_id: inv.guild_id,
        guild_name: inv.guild_name,
        code: inv.code,
        creator_id: inv.creator_id,
        max_uses: inv.max_uses,
        uses: inv.uses,
        expires_at: inv.expires_at,
        created_at: inv.created_at,
    };
    io::send_encrypted(
        stream,
        PacketId::InviteCreate,
        seq,
        &to_payload(&info),
        crypto,
    )
    .await?;
    Ok(())
}

pub async fn handle_invite_accept(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    session_id: &str,
    payload: &[u8],
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let req: InviteAcceptPayload = serde_json::from_slice(payload)?;
    let sess = session::get(&state.sessions, session_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("session not found"))?;

    let inv = state
        .storage
        .get_invite_by_code(&req.code)
        .await?
        .ok_or_else(|| anyhow::anyhow!("invite not found"))?;

    if let Some(exp) = inv.expires_at {
        let now = now_ms();
        if now > exp {
            send_err(
                stream,
                seq,
                crate::proto::ErrorCode::InvalidPacket,
                "invite expired",
                crypto,
            )
            .await?;
            return Ok(());
        }
    }
    if let Some(max) = inv.max_uses
        && inv.uses >= max
    {
        send_err(
            stream,
            seq,
            crate::proto::ErrorCode::InvalidPacket,
            "invite max uses reached",
            crypto,
        )
        .await?;
        return Ok(());
    }

    state
        .storage
        .add_guild_member(&inv.guild_id, &sess.user_id)
        .await?;
    state.storage.use_invite(&inv.id).await?;
    state
        .storage
        .append_audit_log(
            &inv.guild_id,
            &sess.user_id,
            "MEMBER_JOIN_INVITE",
            Some(&sess.user_id),
            Some("member"),
            None,
        )
        .await?;

    io::send_encrypted(
        stream,
        PacketId::InviteAccept,
        seq,
        &to_payload(&serde_json::json!({"guild_id": inv.guild_id, "guild_name": inv.guild_name})),
        crypto,
    )
    .await?;
    Ok(())
}

pub async fn handle_invite_delete(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    session_id: &str,
    payload: &[u8],
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let req: InviteDeletePayload = serde_json::from_slice(payload)?;
    let sess = session::get(&state.sessions, session_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("session not found"))?;

    if !super::require_perm(
        state,
        &req.guild_id,
        &sess.user_id,
        Permissions::MANAGE_GUILD,
    )
    .await?
    {
        send_err(
            stream,
            seq,
            crate::proto::ErrorCode::PermissionDenied,
            "missing MANAGE_GUILD permission",
            crypto,
        )
        .await?;
        return Ok(());
    }

    state.storage.delete_invite(&req.invite_id).await?;
    state
        .storage
        .append_audit_log(
            &req.guild_id,
            &sess.user_id,
            "INVITE_DELETE",
            Some(&req.invite_id),
            Some("invite"),
            None,
        )
        .await?;
    io::send_encrypted(
        stream,
        PacketId::InviteDelete,
        seq,
        &to_payload(&serde_json::json!({"invite_id": req.invite_id})),
        crypto,
    )
    .await?;
    Ok(())
}
