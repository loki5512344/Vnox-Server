use anyhow::Result;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct AuditLogRow {
    pub id: String,
    pub guild_id: String,
    pub actor_id: String,
    pub action: String,
    #[sqlx(default)]
    pub target_id: Option<String>,
    #[sqlx(default)]
    pub target_type: Option<String>,
    #[sqlx(default)]
    pub reason: Option<String>,
    pub created_at: i64,
}

impl super::super::Storage {
    /// Fetch the last `limit` audit log entries for a guild, newest first.
    pub async fn get_audit_log(&self, guild_id: &str, limit: i64) -> Result<Vec<AuditLogRow>> {
        Ok(sqlx::query_as::<_, AuditLogRow>(
            "SELECT id, guild_id, actor_id, action, target_id, target_type, reason, created_at \
             FROM audit_logs WHERE guild_id=? ORDER BY created_at DESC LIMIT ?",
        )
        .bind(guild_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?)
    }
}
