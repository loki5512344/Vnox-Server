use anyhow::Result;
use prost::Message;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    domain::session,
    net::{
        io,
        state::{BroadcastMsg, State},
    },
    proto::{
        PacketId, PresenceEventPayload, PresenceInfo, PresenceSyncPayload, PresenceUpdatePayload,
        SessionCrypto, encode_packet, to_payload,
    },
};

pub async fn handle_presence_update(
    _stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    session_id: &str,
    payload: &[u8],
    _crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let req = PresenceUpdatePayload::decode(payload)?;
    let sess = session::get(&state.sessions, session_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("session not found"))?;

    let info = PresenceInfo {
        user_id: sess.user_id.clone(),
        nickname: sess.nickname.clone(),
        status: req.status.clone(),
        activity_type: req.activity_type,
        activity_text: req.activity_text,
    };

    state
        .presences
        .write()
        .await
        .insert(sess.user_id.clone(), info.clone());

    // Broadcast to everyone (friends/guild-mates will filter client-side for now)
    let event = PresenceEventPayload {
        user_id: info.user_id,
        nickname: info.nickname,
        status: info.status,
        activity_type: info.activity_type,
        activity_text: info.activity_text,
    };

    let _ = state.broadcast.send(BroadcastMsg {
        channel_id: None,
        exclude_session: Some(session_id.into()),
        target_session_id: None,
        data: encode_packet(PacketId::PresenceEvent, *seq, &to_payload(&event)),
    });

    Ok(())
}

pub async fn handle_presence_sync(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    session_id: &str,
    crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let _sess = session::get(&state.sessions, session_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("session not found"))?;

    let presences: Vec<PresenceInfo> = state.presences.read().await.values().cloned().collect();

    io::send_encrypted(
        stream,
        PacketId::PresenceSync,
        seq,
        &to_payload(&PresenceSyncPayload { presences }),
        crypto,
    )
    .await?;
    Ok(())
}
