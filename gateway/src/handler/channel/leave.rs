use anyhow::Result;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::info;

use crate::{
    domain::{channels, session},
    net::state::State,
    proto::SessionCrypto,
};

use super::{broadcast_leave, set_channel};

pub async fn leave(
    _stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    _seq: &mut u32,
    session_id: &str,
    channel_id: &str,
    _crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    let user_id = session::get(&state.sessions, session_id)
        .await
        .map(|s| s.user_id);

    let is_voice = channels::get_channel(&state.channels, channel_id)
        .await
        .is_some_and(|c| c.kind == channels::ChannelKind::Voice);

    channels::leave(&state.channels, channel_id, session_id).await;
    set_channel(state, session_id, None).await;
    broadcast_leave(state, channel_id, session_id).await;

    if is_voice
        && let Some(tx) = &state.voice_member_tx
        && let Some(ref uid) = user_id
    {
        let event = serde_json::json!({
            "type": "leave",
            "channel_id": channel_id,
            "user_id": uid,
        });
        let _ = tx.send(event.to_string());
    }

    info!("session {} left {channel_id}", &session_id[..8]);
    Ok(())
}
