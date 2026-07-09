use anyhow::Result;
use prost::Message;

use crate::{
    domain::session,
    net::state::{BroadcastMsg, State},
    proto::{PacketId, TypingBroadcastPayload, TypingStartPayload, encode_packet, to_payload},
};

pub async fn handle_typing_start(session_id: &str, payload: &[u8], state: &State) -> Result<()> {
    let req = TypingStartPayload::decode(payload)?;
    let sess = match session::get(&state.sessions, session_id).await {
        Some(s) => s,
        None => return Ok(()),
    };

    let data = TypingBroadcastPayload {
        user_id: sess.user_id.clone(),
        nickname: sess.nickname.clone(),
        channel_id: req.channel_id.clone(),
    };

    let _ = state.broadcast.send(BroadcastMsg {
        channel_id: Some(req.channel_id),
        exclude_session: Some(session_id.into()),
        target_session_id: None,
        data: encode_packet(PacketId::TypingStart, 0, &to_payload(&data)),
    });

    Ok(())
}
