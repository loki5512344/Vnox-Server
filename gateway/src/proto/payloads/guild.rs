use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GuildCreatePayload {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GuildInfo {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    pub member_count: i64,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuildListPayload {
    pub guilds: Vec<GuildInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuildMemberJoinPayload {
    pub guild_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuildMemberLeavePayload {
    pub guild_id: String,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuildMemberKickPayload {
    pub guild_id: String,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleCreatePayload {
    pub guild_id: String,
    pub name: String,
    pub color: Option<String>,
    pub permissions: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleDeletePayload {
    pub guild_id: String,
    pub role_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InviteCreatePayload {
    pub guild_id: String,
    pub max_uses: Option<i64>,
    pub expires_in_seconds: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InviteInfo {
    pub id: String,
    pub guild_id: String,
    pub guild_name: String,
    pub code: String,
    pub creator_id: String,
    pub max_uses: Option<i64>,
    pub uses: i64,
    pub expires_at: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InviteAcceptPayload {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InviteDeletePayload {
    pub guild_id: String,
    pub invite_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuildAuditLogFetchPayload {
    pub guild_id: String,
    #[serde(default = "default_audit_limit")]
    pub limit: i64,
}

fn default_audit_limit() -> i64 {
    50
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditLogEntryPayload {
    pub id: String,
    pub guild_id: String,
    pub actor_id: String,
    pub action: String,
    pub target_id: Option<String>,
    pub target_type: Option<String>,
    pub reason: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuildAuditLogPayload {
    pub guild_id: String,
    pub entries: Vec<AuditLogEntryPayload>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuildMemberListFetchPayload {
    pub guild_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GuildMemberInfoPayload {
    pub user_id: String,
    pub nickname: String,
    pub joined_at: i64,
    pub role_color: String,
    pub role_name: String,
    /// True if this user is the guild owner.
    pub is_owner: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuildMemberListPayload {
    pub guild_id: String,
    pub members: Vec<GuildMemberInfoPayload>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleAssignPayload {
    pub guild_id: String,
    pub user_id: String,
    pub role_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuildRoleListFetchPayload {
    pub guild_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GuildRoleInfoPayload {
    pub id: String,
    pub guild_id: String,
    pub name: String,
    pub color: String,
    pub permissions: u64,
    pub position: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuildRoleListPayload {
    pub guild_id: String,
    pub roles: Vec<GuildRoleInfoPayload>,
}
