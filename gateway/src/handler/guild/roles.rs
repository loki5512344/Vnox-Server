use anyhow::Result;
use tokio::net::TcpStream;

use crate::{
    domain::{permissions::Permissions, session},
    net::{io, state::State},
    proto::{PacketId, RoleCreatePayload, RoleDeletePayload, SessionCrypto, to_payload},
};

use super::send_err;

pub async fn handle_role_create(
    stream: &mut TcpStream,
    seq: &mut u32,
    session_id: &str,
    payload: &[u8],
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let req: RoleCreatePayload = serde_json::from_slice(payload)?;
    let sess = session::get(&state.sessions, session_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("session not found"))?;

    if !super::require_perm(
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

    let color = req.color.as_deref().unwrap_or("#ffffff");
    let permissions = req.permissions.unwrap_or(0);
    let role_id = state
        .storage
        .create_role(&req.guild_id, &req.name, color, permissions, 1)
        .await?;

    state
        .storage
        .append_audit_log(
            &req.guild_id,
            &sess.user_id,
            "ROLE_CREATE",
            Some(&role_id),
            Some("role"),
            None,
        )
        .await?;
    io::send_encrypted(
        stream,
        PacketId::RoleCreate,
        seq,
        &to_payload(
            &serde_json::json!({"id": role_id, "guild_id": req.guild_id, "name": req.name}),
        ),
        crypto,
    )
    .await?;
    Ok(())
}

pub async fn handle_role_delete(
    stream: &mut TcpStream,
    seq: &mut u32,
    session_id: &str,
    payload: &[u8],
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let req: RoleDeletePayload = serde_json::from_slice(payload)?;
    let sess = session::get(&state.sessions, session_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("session not found"))?;

    if !super::require_perm(
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

    state.storage.delete_role(&req.role_id).await?;
    state
        .storage
        .append_audit_log(
            &req.guild_id,
            &sess.user_id,
            "ROLE_DELETE",
            Some(&req.role_id),
            Some("role"),
            None,
        )
        .await?;
    io::send_encrypted(
        stream,
        PacketId::RoleDelete,
        seq,
        &to_payload(&serde_json::json!({"role_id": req.role_id})),
        crypto,
    )
    .await?;
    Ok(())
}
