pub mod create;
pub mod edit;
pub mod join;
pub mod leave;

pub use create::{handle_channel_create, handle_channel_delete, handle_channel_list};
pub use edit::handle_channel_edit;
pub use join::join;
pub use leave::leave;

use crate::{
    net::state::{BroadcastMsg, State},
    proto::{PacketId, UserLeavePayload, encode_packet, to_payload},
};

pub async fn broadcast_leave(state: &State, channel_id: &str, session_id: &str) {
    if let Some(sess) = crate::domain::session::get(&state.sessions, session_id).await {
        let p = UserLeavePayload {
            channel_id: channel_id.into(),
            user_id: sess.user_id.clone(),
        };
        let _ = state.broadcast.send(BroadcastMsg {
            channel_id: Some(channel_id.into()),
            exclude_session: Some(session_id.into()),
            target_session_id: None,
            data: encode_packet(PacketId::UserLeave, 0, &to_payload(&p)),
        });
    }
}

async fn set_channel(state: &State, session_id: &str, ch: Option<String>) {
    if let Some(s) = state.sessions.write().await.get_mut(session_id) {
        s.channel_id = ch;
    }
}
