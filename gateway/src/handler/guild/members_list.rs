use anyhow::Result;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    domain::{permissions::Permissions, session},
    net::{io, state::State},
    proto::{
        GuildMemberInfoPayload, GuildMemberListFetchPayload, GuildMemberListPayload,
        GuildRoleInfoPayload, GuildRoleListFetchPayload, GuildRoleListPayload, PacketId,
        RoleAssignPayload, SessionCrypto, to_payload,
    },
};

use super::{require_perm, send_err};

/// Fetch the member list for a guild (visible to all members).
pub async fn handle_member_list_fetch(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    session_id: &str,
    payload: &[u8],
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let req: GuildMemberListFetchPayload = serde_json::from_slice(payload)?;
    let sess = match session::get(&state.sessions, session_id).await {
        Some(s) => s,
        None => return Ok(()),
    };

    // Optional: verify the requester is a member of the guild.
    let owner_id = state
        .storage
        .get_guild(&req.guild_id)
        .await?
        .map(|g| g.owner_id);

    let rows = state.storage.list_guild_members(&req.guild_id).await?;
    let members: Vec<GuildMemberInfoPayload> = rows
        .into_iter()
        .map(|r| GuildMemberInfoPayload {
            user_id: r.user_id.clone(),
            nickname: r.nickname,
            joined_at: r.joined_at,
            role_color: r.role_color,
            role_name: r.role_name,
            is_owner: owner_id.as_deref() == Some(&r.user_id),
        })
        .collect();
    let _ = sess;
    let resp = GuildMemberListPayload {
        guild_id: req.guild_id,
        members,
    };
    io::send_encrypted(
        stream,
        PacketId::GuildMemberList,
        seq,
        &to_payload(&resp),
        crypto,
    )
    .await?;
    Ok(())
}

/// Assign a role to a user (admin only: requires MANAGE_ROLES).
pub async fn handle_role_assign(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    session_id: &str,
    payload: &[u8],
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let req: RoleAssignPayload = serde_json::from_slice(payload)?;
    let sess = match session::get(&state.sessions, session_id).await {
        Some(s) => s,
        None => return Ok(()),
    };

    if !require_perm(
        state,
        &req.guild_id,
        &sess.user_id,
        Permissions::MANAGE_ROLES,
    )
    .await?
    {
        send_err(
            stream,
            seq,
            crate::proto::ErrorCode::PermissionDenied,
            "missing MANAGE_ROLES permission",
            crypto,
        )
        .await?;
        return Ok(());
    }

    state
        .storage
        .assign_role(&req.guild_id, &req.user_id, &req.role_id)
        .await?;
    state
        .storage
        .append_audit_log(
            &req.guild_id,
            &sess.user_id,
            "ROLE_ASSIGN",
            Some(&req.user_id),
            Some("user"),
            Some(&req.role_id),
        )
        .await?;

    // Confirm to caller.
    io::send_encrypted(
        stream,
        PacketId::GuildRoleAssign,
        seq,
        &to_payload(
            &serde_json::json!({"ok": true, "user_id": req.user_id, "role_id": req.role_id}),
        ),
        crypto,
    )
    .await?;
    Ok(())
}

/// Remove a role from a user (admin only: requires MANAGE_ROLES).
pub async fn handle_role_unassign(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    session_id: &str,
    payload: &[u8],
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let req: RoleAssignPayload = serde_json::from_slice(payload)?;
    let sess = match session::get(&state.sessions, session_id).await {
        Some(s) => s,
        None => return Ok(()),
    };

    if !require_perm(
        state,
        &req.guild_id,
        &sess.user_id,
        Permissions::MANAGE_ROLES,
    )
    .await?
    {
        send_err(
            stream,
            seq,
            crate::proto::ErrorCode::PermissionDenied,
            "missing MANAGE_ROLES permission",
            crypto,
        )
        .await?;
        return Ok(());
    }

    state
        .storage
        .remove_role_from_user(&req.guild_id, &req.user_id, &req.role_id)
        .await?;
    state
        .storage
        .append_audit_log(
            &req.guild_id,
            &sess.user_id,
            "ROLE_UNASSIGN",
            Some(&req.user_id),
            Some("user"),
            Some(&req.role_id),
        )
        .await?;

    io::send_encrypted(
        stream,
        PacketId::GuildRoleUnassign,
        seq,
        &to_payload(
            &serde_json::json!({"ok": true, "user_id": req.user_id, "role_id": req.role_id}),
        ),
        crypto,
    )
    .await?;
    Ok(())
}

/// Fetch all roles defined in a guild (visible to all members).
pub async fn handle_role_list_fetch(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    session_id: &str,
    payload: &[u8],
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let req: GuildRoleListFetchPayload = serde_json::from_slice(payload)?;
    let _sess = session::get(&state.sessions, session_id).await;

    let rows = state.storage.list_guild_roles(&req.guild_id).await?;
    let roles: Vec<GuildRoleInfoPayload> = rows
        .into_iter()
        .map(|r| GuildRoleInfoPayload {
            id: r.id,
            guild_id: r.guild_id,
            name: r.name,
            color: r.color,
            permissions: r.permissions as u64,
            position: r.position,
        })
        .collect();
    let resp = GuildRoleListPayload {
        guild_id: req.guild_id,
        roles,
    };
    io::send_encrypted(
        stream,
        PacketId::GuildRoleList,
        seq,
        &to_payload(&resp),
        crypto,
    )
    .await?;
    Ok(())
}
