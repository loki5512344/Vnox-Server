use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DmStartPayload {
    pub target_user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DmStartResponsePayload {
    pub dm_id: String,
    pub other_user_id: String,
    pub other_nickname: String,
    pub messages: Vec<DmMessagePayload>,
    #[serde(default)]
    pub unread_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DmMessagePayload {
    pub dm_id: String,
    pub sender_id: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DmHistoryPayload {
    pub dm_id: String,
    #[serde(default)]
    pub messages: Vec<DmMessagePayload>,
    #[serde(default)]
    pub search_query: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FriendRequestPayload {
    pub to_user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FriendAcceptPayload {
    pub from_user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FriendDeclinePayload {
    pub from_user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FriendRemovePayload {
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FriendInfo {
    pub user_id: String,
    pub nickname: String,
    pub status: String,
    pub since: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FriendListPayload {
    pub friends: Vec<FriendInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockUserPayload {
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnblockUserPayload {
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockListPayload {
    pub blocked: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FriendEventPayload {
    pub event: String,
    pub user_id: String,
    pub nickname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PresenceUpdatePayload {
    pub status: String,
    pub activity_type: Option<String>,
    pub activity_text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PresenceInfo {
    pub user_id: String,
    pub nickname: String,
    pub status: String,
    pub activity_type: Option<String>,
    pub activity_text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PresenceSyncPayload {
    pub presences: Vec<PresenceInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PresenceEventPayload {
    pub user_id: String,
    pub nickname: String,
    pub status: String,
    pub activity_type: Option<String>,
    pub activity_text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadReceiptPayload {
    pub channel_id: String,
    pub last_read_message_id: String,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypingStartPayload {
    pub channel_id: String,
}
