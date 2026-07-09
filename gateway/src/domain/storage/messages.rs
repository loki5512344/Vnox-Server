use anyhow::Result;

use crate::proto::ChatMessagePayload;

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
        sqlx::query("INSERT OR IGNORE INTO messages (id,channel_id,sender_id,content,timestamp,reply_to) VALUES (?,?,?,?,?,?)")
            .bind(&msg.message_id).bind(&msg.channel_id).bind(&msg.sender_id)
            .bind(&msg.content).bind(msg.timestamp).bind(&msg.reply_to)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_history(
        &self,
        channel_id: &str,
        limit: i64,
    ) -> Result<Vec<ChatMessagePayload>> {
        let rows = sqlx::query_as::<_, MsgRow>(
            "SELECT id,channel_id,sender_id,content,timestamp,reply_to FROM \
             (SELECT * FROM messages WHERE channel_id=? ORDER BY timestamp DESC LIMIT ?) \
             ORDER BY timestamp ASC",
        )
        .bind(channel_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
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
        sqlx::query(
            "INSERT OR REPLACE INTO read_receipts (channel_id,user_id,last_read_message_id,updated_at) VALUES (?,?,?,?)",
        ).bind(channel_id).bind(user_id).bind(message_id).bind(super::now_ms())
         .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn add_reaction(&self, message_id: &str, user_id: &str, emoji: &str) -> Result<()> {
        sqlx::query("INSERT OR IGNORE INTO reactions (message_id,user_id,emoji,created_at) VALUES (?,?,?,?)")
            .bind(message_id).bind(user_id).bind(emoji).bind(super::now_ms())
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn remove_reaction(
        &self,
        message_id: &str,
        user_id: &str,
        emoji: &str,
    ) -> Result<()> {
        sqlx::query("DELETE FROM reactions WHERE message_id=? AND user_id=? AND emoji=?")
            .bind(message_id)
            .bind(user_id)
            .bind(emoji)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn has_user_reacted(
        &self,
        message_id: &str,
        user_id: &str,
        emoji: &str,
    ) -> Result<bool> {
        Ok(sqlx::query_as::<_, (String,)>(
            "SELECT user_id FROM reactions WHERE message_id=? AND user_id=? AND emoji=?",
        )
        .bind(message_id)
        .bind(user_id)
        .bind(emoji)
        .fetch_optional(&self.pool)
        .await?
        .is_some())
    }

    pub async fn edit_message(&self, message_id: &str, new_content: &str) -> Result<()> {
        sqlx::query("UPDATE messages SET content=? WHERE id=?")
            .bind(new_content)
            .bind(message_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_message(&self, message_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM messages WHERE id=?")
            .bind(message_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_message_sender(&self, message_id: &str) -> Result<Option<String>> {
        Ok(
            sqlx::query_as::<_, (String,)>("SELECT sender_id FROM messages WHERE id=?")
                .bind(message_id)
                .fetch_optional(&self.pool)
                .await?
                .map(|(id,)| id),
        )
    }

    pub async fn get_message(&self, message_id: &str) -> Result<Option<ChatMessagePayload>> {
        Ok(sqlx::query_as::<_, MsgRow>(
            "SELECT id,channel_id,sender_id,content,timestamp,reply_to FROM messages WHERE id=?",
        )
        .bind(message_id)
        .fetch_optional(&self.pool)
        .await?
        .map(|r| ChatMessagePayload {
            message_id: r.id,
            channel_id: r.channel_id,
            sender_id: r.sender_id,
            content: r.content,
            timestamp: r.timestamp,
            edited: false,
            reply_to: r.reply_to,
        }))
    }
}
