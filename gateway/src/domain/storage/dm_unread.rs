use anyhow::Result;

use super::Pool;

impl super::Storage {
    pub async fn increment_dm_unread(&self, dm_id: &str, recipient_id: &str) -> Result<()> {
        match &self.pool {
            Pool::Sqlite(p) => {
                let (u1, _u2) = sqlx::query_as::<_, (String, String)>(
                    "SELECT user1_id,user2_id FROM direct_messages WHERE id=?",
                )
                .bind(dm_id)
                .fetch_one(p)
                .await?;
                if recipient_id == u1 {
                    sqlx::query(
                        "UPDATE direct_messages SET unread_count_2=unread_count_2+1 WHERE id=?",
                    )
                    .bind(dm_id)
                    .execute(p)
                    .await?;
                } else {
                    sqlx::query(
                        "UPDATE direct_messages SET unread_count_1=unread_count_1+1 WHERE id=?",
                    )
                    .bind(dm_id)
                    .execute(p)
                    .await?;
                }
            }
            Pool::Postgres(p) => {
                let (u1, _u2) = sqlx::query_as::<_, (String, String)>(
                    "SELECT user1_id,user2_id FROM direct_messages WHERE id=$1",
                )
                .bind(dm_id)
                .fetch_one(p)
                .await?;
                if recipient_id == u1 {
                    sqlx::query(
                        "UPDATE direct_messages SET unread_count_2=unread_count_2+1 WHERE id=$1",
                    )
                    .bind(dm_id)
                    .execute(p)
                    .await?;
                } else {
                    sqlx::query(
                        "UPDATE direct_messages SET unread_count_1=unread_count_1+1 WHERE id=$1",
                    )
                    .bind(dm_id)
                    .execute(p)
                    .await?;
                }
            }
        }
        Ok(())
    }

    pub async fn reset_dm_unread(&self, dm_id: &str, user_id: &str) -> Result<()> {
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query(
                    "UPDATE direct_messages SET \
                     unread_count_1 = CASE WHEN user1_id=? THEN 0 ELSE unread_count_1 END, \
                     unread_count_2 = CASE WHEN user2_id=? THEN 0 ELSE unread_count_2 END WHERE id=?",
                )
                .bind(user_id)
                .bind(user_id)
                .bind(dm_id)
                .execute(p)
                .await?;
            }
            Pool::Postgres(p) => {
                sqlx::query(
                    "UPDATE direct_messages SET \
                     unread_count_1 = CASE WHEN user1_id=$1 THEN 0 ELSE unread_count_1 END, \
                     unread_count_2 = CASE WHEN user2_id=$2 THEN 0 ELSE unread_count_2 END WHERE id=$3",
                )
                .bind(user_id)
                .bind(user_id)
                .bind(dm_id)
                .execute(p)
                .await?;
            }
        }
        Ok(())
    }
}
