use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HelloPayload {
    pub lnex_version: String,
    pub server_pubkey: String,
    pub challenge_nonce: String,
    pub node_name: String,
    pub server_eph_pubkey: String,
    pub private_mode: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthPayload {
    pub client_pubkey: String,
    pub nickname: String,
    pub lnex_version: String,
    pub signature: String,
    pub client_eph_pubkey: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionPayload {
    pub session_id: String,
    pub token: String,
    pub expires_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PingPayload {
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PongPayload {
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinChannelPayload {
    pub channel_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaveChannelPayload {
    pub channel_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelStatePayload {
    pub channel_id: String,
    pub channel_name: String,
    pub kind: String,
    pub members: Vec<MemberInfo>,
    pub voice_endpoint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelCreatePayload {
    pub channel_id: String,
    pub channel_name: String,
    /// "text" or "voice".
    pub kind: String,
    /// Optional guild_id this channel belongs to (Phase 1.x — unused, future).
    #[serde(default)]
    pub guild_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelDeletePayload {
    pub channel_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelListPayload {
    pub channels: Vec<ChannelListItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelListItem {
    pub channel_id: String,
    pub channel_name: String,
    pub kind: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemberInfo {
    pub user_id: String,
    pub nickname: String,
    pub in_voice: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserJoinPayload {
    pub channel_id: String,
    pub user_id: String,
    pub nickname: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLeavePayload {
    pub channel_id: String,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessagePayload {
    pub message_id: String,
    pub channel_id: String,
    pub sender_id: String,
    pub content: String,
    pub timestamp: i64,
    #[serde(default)]
    pub edited: bool,
    /// Optional message_id this message is replying to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReactionPayload {
    pub message_id: String,
    pub channel_id: String,
    pub emoji: String,
    #[serde(default)]
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageEditPayload {
    pub message_id: String,
    pub channel_id: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageDeletePayload {
    pub message_id: String,
    pub channel_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatHistoryPayload {
    pub channel_id: String,
    pub messages: Vec<ChatMessagePayload>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub code: u32,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisconnectPayload {
    pub reason: String,
}
