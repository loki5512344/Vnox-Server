use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub enum VoiceMembershipEvent {
    Joined {
        channel_id: String,
        session_id: String,
        user_id: String,
    },
    Left {
        channel_id: String,
        session_id: String,
        user_id: String,
    },
}

pub fn voice_membership_channel() -> (
    broadcast::Sender<VoiceMembershipEvent>,
    broadcast::Receiver<VoiceMembershipEvent>,
) {
    broadcast::channel(256)
}
