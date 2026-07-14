use anyhow::Result;
use sqlx::AssertSqlSafe;

use crate::proto::DmMessagePayload;

use super::Pool;

#[derive(sqlx::FromRow)]
struct DmMsgRow {
    dm_id: String,
    sender_id: String,
    body: String,
    created_at: i64,
}

impl super::Storage {
    pub async fn find_or_create_dm(&self, user1: &str, user2: &str) -> Result<(String, i64)> {
        let (u1, u2) = if user1 < user2 {
            (user1, user2)
        } else {
            (user2, user1)
        };
        match &self.pool {
            Pool::Sqlite(p) => {
                let existing = sqlx::query_as::<_, (String, i64, i64)>(
                    "SELECT id,unread_count_1,unread_count_2 FROM direct_messages WHERE user1_id=? AND user2_id=?",
                ).bind(u1).bind(u2).fetch_optional(p).await?;
                if let Some((id, uc1, uc2)) = existing {
                    return Ok((id, if u1 == user1 { uc1 } else { uc2 }));
                }
                let dm_id = format!("dm_{}_{}", u1, u2);
                let now = super::now_ms();
                sqlx::query(
                    "INSERT INTO direct_messages (id,user1_id,user2_id,created_at,unread_count_1,unread_count_2) VALUES (?,?,?,?,0,0)",
                ).bind(&dm_id).bind(u1).bind(u2).bind(now).execute(p).await?;
                Ok((dm_id, 0))
            }
            Pool::Postgres(p) => {
                let existing = sqlx::query_as::<_, (String, i64, i64)>(
                    "SELECT id,unread_count_1,unread_count_2 FROM direct_messages WHERE user1_id=$1 AND user2_id=$2",
                ).bind(u1).bind(u2).fetch_optional(p).await?;
                if let Some((id, uc1, uc2)) = existing {
                    return Ok((id, if u1 == user1 { uc1 } else { uc2 }));
                }
                let dm_id = format!("dm_{}_{}", u1, u2);
                let now = super::now_ms();
                sqlx::query(
                    "INSERT INTO direct_messages (id,user1_id,user2_id,created_at,unread_count_1,unread_count_2) VALUES ($1,$2,$3,$4,0,0)",
                ).bind(&dm_id).bind(u1).bind(u2).bind(now).execute(p).await?;
                Ok((dm_id, 0))
            }
        }
    }

    pub async fn save_dm_message(
        &self,
        dm_id: &str,
        sender_id: &str,
        body: &str,
    ) -> Result<DmMessagePayload> {
        let msg_id = uuid::Uuid::new_v4().to_string();
        let ts = super::now_ms();
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query(
                    "INSERT INTO dm_messages (id,dm_id,sender_id,body,created_at) VALUES (?,?,?,?,?)",
                )
                .bind(&msg_id)
                .bind(dm_id)
                .bind(sender_id)
                .bind(body)
                .bind(ts)
                .execute(p)
                .await?;
                sqlx::query("UPDATE direct_messages SET last_message_at=? WHERE id=?")
                    .bind(ts)
                    .bind(dm_id)
                    .execute(p)
                    .await?;
            }
            Pool::Postgres(p) => {
                sqlx::query(
                    "INSERT INTO dm_messages (id,dm_id,sender_id,body,created_at) VALUES ($1,$2,$3,$4,$5)",
                )
                .bind(&msg_id)
                .bind(dm_id)
                .bind(sender_id)
                .bind(body)
                .bind(ts)
                .execute(p)
                .await?;
                sqlx::query("UPDATE direct_messages SET last_message_at=$1 WHERE id=$2")
                    .bind(ts)
                    .bind(dm_id)
                    .execute(p)
                    .await?;
            }
        }
        Ok(DmMessagePayload {
            dm_id: dm_id.to_string(),
            sender_id: sender_id.to_string(),
            content: body.to_string(),
            timestamp: ts,
        })
    }

    pub async fn get_dm_messages(
        &self,
        dm_id: &str,
        limit: i64,
        search_query: Option<&str>,
        before_timestamp: Option<i64>,
    ) -> Result<Vec<DmMessagePayload>> {
        let rows = match &self.pool {
            Pool::Sqlite(p) => {
                let mut sql = String::from(
                    "SELECT dm_id,sender_id,body,created_at FROM \
                     (SELECT * FROM dm_messages WHERE dm_id=? ",
                );
                if search_query.is_some() {
                    sql.push_str("AND body LIKE '%' || ? || '%' ");
                }
                if before_timestamp.is_some() {
                    sql.push_str("AND created_at < ? ");
                }
                sql.push_str("ORDER BY created_at DESC LIMIT ?) ORDER BY created_at ASC");
                let mut q = sqlx::query_as::<_, DmMsgRow>(AssertSqlSafe(sql.as_str()));
                q = q.bind(dm_id);
                if let Some(sq) = search_query {
                    q = q.bind(sq);
                }
                if let Some(bt) = before_timestamp {
                    q = q.bind(bt);
                }
                q = q.bind(limit);
                q.fetch_all(p).await?
            }
            Pool::Postgres(p) => {
                let mut sql = String::from(
                    "SELECT dm_id,sender_id,body,created_at FROM \
                     (SELECT * FROM dm_messages WHERE dm_id=$1 ",
                );
                let mut n = 2u32;
                if search_query.is_some() {
                    sql.push_str(&format!("AND body LIKE '%' || ${n} || '%' "));
                    n += 1;
                }
                if before_timestamp.is_some() {
                    sql.push_str(&format!("AND created_at < ${n} "));
                    n += 1;
                }
                sql.push_str(&format!(
                    "ORDER BY created_at DESC LIMIT ${n}) ORDER BY created_at ASC"
                ));
                let mut q = sqlx::query_as::<_, DmMsgRow>(AssertSqlSafe(sql.as_str()));
                q = q.bind(dm_id);
                if let Some(sq) = search_query {
                    q = q.bind(sq);
                }
                if let Some(bt) = before_timestamp {
                    q = q.bind(bt);
                }
                q = q.bind(limit);
                q.fetch_all(p).await?
            }
        };
        Ok(rows
            .into_iter()
            .map(|r| DmMessagePayload {
                dm_id: r.dm_id,
                sender_id: r.sender_id,
                content: r.body,
                timestamp: r.created_at,
            })
            .collect())
    }

    pub async fn get_dm_user_id(&self, dm_id: &str, my_id: &str) -> Result<Option<String>> {
        match &self.pool {
            Pool::Sqlite(p) => Ok(sqlx::query_as::<_, (String, String)>(
                "SELECT user1_id,user2_id FROM direct_messages WHERE id=?",
            )
            .bind(dm_id)
            .fetch_optional(p)
            .await?
            .map(|(u1, u2)| if u1 == my_id { u2 } else { u1 })),
            Pool::Postgres(p) => Ok(sqlx::query_as::<_, (String, String)>(
                "SELECT user1_id,user2_id FROM direct_messages WHERE id=$1",
            )
            .bind(dm_id)
            .fetch_optional(p)
            .await?
            .map(|(u1, u2)| if u1 == my_id { u2 } else { u1 })),
        }
    }

    pub async fn get_dm_nickname(&self, user_id: &str) -> Result<Option<String>> {
        match &self.pool {
            Pool::Sqlite(p) => Ok(sqlx::query_as::<_, (String,)>(
                "SELECT nickname FROM users WHERE pubkey=?",
            )
            .bind(user_id)
            .fetch_optional(p)
            .await?
            .map(|(n,)| n)),
            Pool::Postgres(p) => Ok(sqlx::query_as::<_, (String,)>(
                "SELECT nickname FROM users WHERE pubkey=$1",
            )
            .bind(user_id)
            .fetch_optional(p)
            .await?
            .map(|(n,)| n)),
        }
    }

    pub async fn get_nickname(&self, user_id: &str) -> Result<Option<String>> {
        self.get_dm_nickname(user_id).await
    }
}
