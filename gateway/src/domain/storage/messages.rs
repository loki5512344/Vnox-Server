use anyhow::Result;

use crate::proto::ChatMessagePayload;

use super::Pool;

#[derive(sqlx::FromRow)]
struct MsgRow {
    id: String,
    channel_id: String,
    sender_id: String,
    content: String,
    timestamp: i64,
    #[sqlx(default)]
    reply_to: Option<String>,
}

impl super::Storage {
    pub async fn save_message(&self, msg: &ChatMessagePayload) -> Result<()> {
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query("INSERT OR IGNORE INTO messages (id,channel_id,sender_id,content,timestamp,reply_to) VALUES (?,?,?,?,?,?)")
                    .bind(&msg.message_id).bind(&msg.channel_id).bind(&msg.sender_id)
                    .bind(&msg.content).bind(msg.timestamp).bind(&msg.reply_to)
                    .execute(p).await?;
            }
            Pool::Postgres(p) => {
                sqlx::query("INSERT INTO messages (id,channel_id,sender_id,content,timestamp,reply_to) VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT (id) DO NOTHING")
                    .bind(&msg.message_id).bind(&msg.channel_id).bind(&msg.sender_id)
                    .bind(&msg.content).bind(msg.timestamp).bind(&msg.reply_to)
                    .execute(p).await?;
            }
        }
        Ok(())
    }

    pub async fn get_history(
        &self,
        channel_id: &str,
        limit: i64,
    ) -> Result<Vec<ChatMessagePayload>> {
        let rows = match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query_as::<_, MsgRow>(
                    "SELECT id,channel_id,sender_id,content,timestamp,reply_to FROM \
                     (SELECT * FROM messages WHERE channel_id=? ORDER BY timestamp DESC LIMIT ?) \
                     ORDER BY timestamp ASC",
                )
                .bind(channel_id)
                .bind(limit)
                .fetch_all(p)
                .await?
            }
            Pool::Postgres(p) => {
                sqlx::query_as::<_, MsgRow>(
                    "SELECT id,channel_id,sender_id,content,timestamp,reply_to FROM \
                     (SELECT * FROM messages WHERE channel_id=$1 ORDER BY timestamp DESC LIMIT $2) \
                     ORDER BY timestamp ASC",
                )
                .bind(channel_id)
                .bind(limit)
                .fetch_all(p)
                .await?
            }
        };
        Ok(rows
            .into_iter()
            .map(|r| ChatMessagePayload {
                message_id: r.id,
                channel_id: r.channel_id,
                sender_id: r.sender_id,
                content: r.content,
                timestamp: r.timestamp,
                edited: false,
                reply_to: r.reply_to,
            })
            .collect())
    }

    pub async fn update_read_receipt(
        &self,
        channel_id: &str,
        user_id: &str,
        message_id: &str,
    ) -> Result<()> {
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query(
                    "INSERT OR REPLACE INTO read_receipts (channel_id,user_id,last_read_message_id,updated_at) VALUES (?,?,?,?)",
                )
                .bind(channel_id).bind(user_id).bind(message_id).bind(super::now_ms())
                .execute(p).await?;
            }
            Pool::Postgres(p) => {
                sqlx::query(
                    "INSERT INTO read_receipts (channel_id,user_id,last_read_message_id,updated_at) VALUES ($1,$2,$3,$4) ON CONFLICT (channel_id,user_id) DO UPDATE SET last_read_message_id=$3, updated_at=$4",
                )
                .bind(channel_id).bind(user_id).bind(message_id).bind(super::now_ms())
                .execute(p).await?;
            }
        }
        Ok(())
    }

    pub async fn add_reaction(&self, message_id: &str, user_id: &str, emoji: &str) -> Result<()> {
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query("INSERT OR IGNORE INTO reactions (message_id,user_id,emoji,created_at) VALUES (?,?,?,?)")
                    .bind(message_id).bind(user_id).bind(emoji).bind(super::now_ms())
                    .execute(p).await?;
            }
            Pool::Postgres(p) => {
                sqlx::query("INSERT INTO reactions (message_id,user_id,emoji,created_at) VALUES ($1,$2,$3,$4) ON CONFLICT (message_id,user_id,emoji) DO NOTHING")
                    .bind(message_id).bind(user_id).bind(emoji).bind(super::now_ms())
                    .execute(p).await?;
            }
        }
        Ok(())
    }

    pub async fn remove_reaction(
        &self,
        message_id: &str,
        user_id: &str,
        emoji: &str,
    ) -> Result<()> {
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query("DELETE FROM reactions WHERE message_id=? AND user_id=? AND emoji=?")
                    .bind(message_id)
                    .bind(user_id)
                    .bind(emoji)
                    .execute(p)
                    .await?;
            }
            Pool::Postgres(p) => {
                sqlx::query(
                    "DELETE FROM reactions WHERE message_id=$1 AND user_id=$2 AND emoji=$3",
                )
                .bind(message_id)
                .bind(user_id)
                .bind(emoji)
                .execute(p)
                .await?;
            }
        }
        Ok(())
    }

    pub async fn has_user_reacted(
        &self,
        message_id: &str,
        user_id: &str,
        emoji: &str,
    ) -> Result<bool> {
        match &self.pool {
            Pool::Sqlite(p) => Ok(sqlx::query_as::<_, (String,)>(
                "SELECT user_id FROM reactions WHERE message_id=? AND user_id=? AND emoji=?",
            )
            .bind(message_id)
            .bind(user_id)
            .bind(emoji)
            .fetch_optional(p)
            .await?
            .is_some()),
            Pool::Postgres(p) => Ok(sqlx::query_as::<_, (String,)>(
                "SELECT user_id FROM reactions WHERE message_id=$1 AND user_id=$2 AND emoji=$3",
            )
            .bind(message_id)
            .bind(user_id)
            .bind(emoji)
            .fetch_optional(p)
            .await?
            .is_some()),
        }
    }

    pub async fn edit_message(&self, message_id: &str, new_content: &str) -> Result<()> {
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query("UPDATE messages SET content=? WHERE id=?")
                    .bind(new_content)
                    .bind(message_id)
                    .execute(p)
                    .await?;
            }
            Pool::Postgres(p) => {
                sqlx::query("UPDATE messages SET content=$1 WHERE id=$2")
                    .bind(new_content)
                    .bind(message_id)
                    .execute(p)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn delete_message(&self, message_id: &str) -> Result<()> {
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query("DELETE FROM messages WHERE id=?")
                    .bind(message_id)
                    .execute(p)
                    .await?;
            }
            Pool::Postgres(p) => {
                sqlx::query("DELETE FROM messages WHERE id=$1")
                    .bind(message_id)
                    .execute(p)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn get_message_sender(&self, message_id: &str) -> Result<Option<String>> {
        match &self.pool {
            Pool::Sqlite(p) => Ok(sqlx::query_as::<_, (String,)>(
                "SELECT sender_id FROM messages WHERE id=?",
            )
            .bind(message_id)
            .fetch_optional(p)
            .await?
            .map(|(id,)| id)),
            Pool::Postgres(p) => Ok(sqlx::query_as::<_, (String,)>(
                "SELECT sender_id FROM messages WHERE id=$1",
            )
            .bind(message_id)
            .fetch_optional(p)
            .await?
            .map(|(id,)| id)),
        }
    }

    pub async fn get_message(&self, message_id: &str) -> Result<Option<ChatMessagePayload>> {
        match &self.pool {
            Pool::Sqlite(p) => Ok(sqlx::query_as::<_, MsgRow>(
                "SELECT id,channel_id,sender_id,content,timestamp,reply_to FROM messages WHERE id=?",
            )
            .bind(message_id)
            .fetch_optional(p)
            .await?
            .map(|r| ChatMessagePayload {
                message_id: r.id,
                channel_id: r.channel_id,
                sender_id: r.sender_id,
                content: r.content,
                timestamp: r.timestamp,
                edited: false,
                reply_to: r.reply_to,
            })),
            Pool::Postgres(p) => Ok(sqlx::query_as::<_, MsgRow>(
                "SELECT id,channel_id,sender_id,content,timestamp,reply_to FROM messages WHERE id=$1",
            )
            .bind(message_id)
            .fetch_optional(p)
            .await?
            .map(|r| ChatMessagePayload {
                message_id: r.id,
                channel_id: r.channel_id,
                sender_id: r.sender_id,
                content: r.content,
                timestamp: r.timestamp,
                edited: false,
                reply_to: r.reply_to,
            })),
        }
    }
}
