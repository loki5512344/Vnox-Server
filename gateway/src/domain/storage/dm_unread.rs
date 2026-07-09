use anyhow::Result;

impl super::Storage {
    pub async fn increment_dm_unread(&self, dm_id: &str, recipient_id: &str) -> Result<()> {
        let (u1, _u2) = sqlx::query_as::<_, (String, String)>(
            "SELECT user1_id,user2_id FROM direct_messages WHERE id=?",
        )
        .bind(dm_id)
        .fetch_one(&self.pool)
        .await?;
        if recipient_id == u1 {
            sqlx::query("UPDATE direct_messages SET unread_count_2=unread_count_2+1 WHERE id=?")
                .bind(dm_id)
                .execute(&self.pool)
                .await?;
        } else {
            sqlx::query("UPDATE direct_messages SET unread_count_1=unread_count_1+1 WHERE id=?")
                .bind(dm_id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    pub async fn reset_dm_unread(&self, dm_id: &str, user_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE direct_messages SET \
             unread_count_1 = CASE WHEN user1_id=? THEN 0 ELSE unread_count_1 END, \
             unread_count_2 = CASE WHEN user2_id=? THEN 0 ELSE unread_count_2 END WHERE id=?",
        )
        .bind(user_id)
        .bind(user_id)
        .bind(dm_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
