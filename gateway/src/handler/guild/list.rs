use anyhow::Result;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    domain::session,
    net::{io, state::State},
    proto::{GuildInfo, GuildListPayload, PacketId, SessionCrypto, UserRoleUpdatePayload, to_payload},
};

pub async fn handle_guild_list(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    session_id: &str,
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let sess = session::get(&state.sessions, session_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("session not found"))?;

    let rows = state.storage.list_user_guilds(&sess.user_id).await?;
    let guilds: Vec<GuildInfo> = rows
        .iter()
        .map(|g| GuildInfo {
            id: g.id.clone(),
            owner_id: g.owner_id.clone(),
            name: g.name.clone(),
            member_count: g.member_count,
            created_at: g.created_at,
        })
        .collect();

    for g in &rows {
        let roles = state.storage.get_user_roles(&g.id, &sess.user_id).await?;
        let color = roles
            .iter()
            .max_by_key(|r| r.position)
            .map(|r| r.color.clone())
            .unwrap_or_else(|| "#ffffff".into());
        io::send_encrypted(
            stream,
            PacketId::UserRoleUpdate,
            seq,
            &to_payload(&UserRoleUpdatePayload {
                user_id: sess.user_id.clone(),
                guild_id: g.id.clone(),
                color,
            }),
            crypto,
        )
        .await?;
    }

    io::send_encrypted(
        stream,
        PacketId::GuildList,
        seq,
        &to_payload(&GuildListPayload { guilds }),
        crypto,
    )
    .await?;
    Ok(())
}
