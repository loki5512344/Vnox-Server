use anyhow::Result;

use crate::domain::storage::Pool;

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
    pub async fn get_audit_log(&self, guild_id: &str, limit: i64) -> Result<Vec<AuditLogRow>> {
        match &self.pool {
            Pool::Sqlite(p) => Ok(sqlx::query_as::<_, AuditLogRow>(
                "SELECT id, guild_id, actor_id, action, target_id, target_type, reason, created_at \
                 FROM audit_logs WHERE guild_id=? ORDER BY created_at DESC LIMIT ?",
            )
            .bind(guild_id)
            .bind(limit)
            .fetch_all(p)
            .await?),
            Pool::Postgres(p) => Ok(sqlx::query_as::<_, AuditLogRow>(
                "SELECT id, guild_id, actor_id, action, target_id, target_type, reason, created_at \
                 FROM audit_logs WHERE guild_id=$1 ORDER BY created_at DESC LIMIT $2",
            )
            .bind(guild_id)
            .bind(limit)
            .fetch_all(p)
            .await?),
        }
    }
}
