use anyhow::Result;

use super::{Pool, now_ms};

pub struct PresenceRow {
    pub user_id: String,
    pub nickname: String,
    pub status: String,
    pub activity_type: Option<String>,
    pub activity_text: Option<String>,
    pub last_seen: i64,
}

impl super::Storage {
    pub async fn save_presence(
        &self,
        user_id: &str,
        nickname: &str,
        status: &str,
        activity_type: Option<&str>,
        activity_text: Option<&str>,
    ) -> Result<()> {
        let now = now_ms();
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query(
                    "INSERT OR REPLACE INTO presences (user_id,nickname,status,activity_type,activity_text,last_seen) \
                     VALUES (?,?,?,?,?,?)",
                )
                .bind(user_id)
                .bind(nickname)
                .bind(status)
                .bind(activity_type)
                .bind(activity_text)
                .bind(now)
                .execute(p)
                .await?;
            }
            Pool::Postgres(p) => {
                sqlx::query(
                    "INSERT INTO presences (user_id,nickname,status,activity_type,activity_text,last_seen) \
                     VALUES ($1,$2,$3,$4,$5,$6) \
                     ON CONFLICT (user_id) DO UPDATE SET \
                     nickname=EXCLUDED.nickname, status=EXCLUDED.status, \
                     activity_type=EXCLUDED.activity_type, activity_text=EXCLUDED.activity_text, \
                     last_seen=EXCLUDED.last_seen",
                )
                .bind(user_id)
                .bind(nickname)
                .bind(status)
                .bind(activity_type)
                .bind(activity_text)
                .bind(now)
                .execute(p)
                .await?;
            }
        }
        Ok(())
    }

    pub async fn load_all_presences(&self) -> Result<Vec<PresenceRow>> {
        match &self.pool {
            Pool::Sqlite(p) => {
                let rows = sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>, i64)>(
                    "SELECT user_id,nickname,status,activity_type,activity_text,last_seen FROM presences",
                )
                .fetch_all(p)
                .await?;
                Ok(rows
                    .into_iter()
                    .map(
                        |(user_id, nickname, status, activity_type, activity_text, last_seen)| {
                            PresenceRow {
                                user_id,
                                nickname,
                                status,
                                activity_type,
                                activity_text,
                                last_seen,
                            }
                        },
                    )
                    .collect())
            }
            Pool::Postgres(p) => {
                let rows = sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>, i64)>(
                    "SELECT user_id,nickname,status,activity_type,activity_text,last_seen FROM presences",
                )
                .fetch_all(p)
                .await?;
                Ok(rows
                    .into_iter()
                    .map(
                        |(user_id, nickname, status, activity_type, activity_text, last_seen)| {
                            PresenceRow {
                                user_id,
                                nickname,
                                status,
                                activity_type,
                                activity_text,
                                last_seen,
                            }
                        },
                    )
                    .collect())
            }
        }
    }

    pub async fn remove_presence(&self, user_id: &str) -> Result<()> {
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query("DELETE FROM presences WHERE user_id=?")
                    .bind(user_id)
                    .execute(p)
                    .await?;
            }
            Pool::Postgres(p) => {
                sqlx::query("DELETE FROM presences WHERE user_id=$1")
                    .bind(user_id)
                    .execute(p)
                    .await?;
            }
        }
        Ok(())
    }
}
