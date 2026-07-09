use anyhow::Result;

impl super::Storage {
    pub async fn create_friend_request(&self, from_id: &str, to_id: &str) -> Result<bool> {
        let (u1, u2) = if from_id < to_id {
            (from_id, to_id)
        } else {
            (to_id, from_id)
        };
        let exists = sqlx::query_as::<_, (String,)>(
            "SELECT user_id_1 FROM friendships WHERE user_id_1=? AND user_id_2=?",
        )
        .bind(u1)
        .bind(u2)
        .fetch_optional(&self.pool)
        .await?;
        if exists.is_some() {
            return Ok(false);
        }
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT OR IGNORE INTO friend_requests (id,from_user_id,to_user_id,status,created_at) VALUES (?,?,?,?,?)"
        ).bind(&id).bind(from_id).bind(to_id).bind("PENDING").bind(super::now_ms())
         .execute(&self.pool).await?;
        Ok(true)
    }

    pub async fn accept_friend_request(&self, from_id: &str, to_id: &str) -> Result<bool> {
        let updated = sqlx::query(
            "UPDATE friend_requests SET status='ACCEPTED' WHERE from_user_id=? AND to_user_id=? AND status='PENDING'"
        ).bind(from_id).bind(to_id).execute(&self.pool).await?;
        if updated.rows_affected() == 0 {
            return Ok(false);
        }
        let (u1, u2) = if from_id < to_id {
            (from_id, to_id)
        } else {
            (to_id, from_id)
        };
        sqlx::query(
            "INSERT OR IGNORE INTO friendships (user_id_1,user_id_2,created_at) VALUES (?,?,?)",
        )
        .bind(u1)
        .bind(u2)
        .bind(super::now_ms())
        .execute(&self.pool)
        .await?;
        Ok(true)
    }

    pub async fn decline_friend_request(&self, from_id: &str, to_id: &str) -> Result<()> {
        sqlx::query("UPDATE friend_requests SET status='DECLINED' WHERE from_user_id=? AND to_user_id=? AND status='PENDING'")
            .bind(from_id).bind(to_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn remove_friend(&self, user_a: &str, user_b: &str) -> Result<()> {
        let (u1, u2) = if user_a < user_b {
            (user_a, user_b)
        } else {
            (user_b, user_a)
        };
        sqlx::query("DELETE FROM friendships WHERE user_id_1=? AND user_id_2=?")
            .bind(u1)
            .bind(u2)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_friends(&self, user_id: &str) -> Result<Vec<String>> {
        let rows1 =
            sqlx::query_as::<_, (String,)>("SELECT user_id_2 FROM friendships WHERE user_id_1=?")
                .bind(user_id)
                .fetch_all(&self.pool)
                .await?;
        let rows2 =
            sqlx::query_as::<_, (String,)>("SELECT user_id_1 FROM friendships WHERE user_id_2=?")
                .bind(user_id)
                .fetch_all(&self.pool)
                .await?;
        Ok(rows1.into_iter().chain(rows2).map(|(id,)| id).collect())
    }

    pub async fn is_friend(&self, user_a: &str, user_b: &str) -> Result<bool> {
        let (u1, u2) = if user_a < user_b {
            (user_a, user_b)
        } else {
            (user_b, user_a)
        };
        Ok(sqlx::query_as::<_, (String,)>(
            "SELECT user_id_1 FROM friendships WHERE user_id_1=? AND user_id_2=?",
        )
        .bind(u1)
        .bind(u2)
        .fetch_optional(&self.pool)
        .await?
        .is_some())
    }

    pub async fn block_user(&self, blocker: &str, blocked: &str) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO blocks (blocker_id,blocked_id,created_at) VALUES (?,?,?)",
        )
        .bind(blocker)
        .bind(blocked)
        .bind(super::now_ms())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn unblock_user(&self, blocker: &str, blocked: &str) -> Result<()> {
        sqlx::query("DELETE FROM blocks WHERE blocker_id=? AND blocked_id=?")
            .bind(blocker)
            .bind(blocked)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn is_blocked(&self, blocker: &str, blocked: &str) -> Result<bool> {
        Ok(sqlx::query_as::<_, (String,)>(
            "SELECT blocker_id FROM blocks WHERE blocker_id=? AND blocked_id=?",
        )
        .bind(blocker)
        .bind(blocked)
        .fetch_optional(&self.pool)
        .await?
        .is_some())
    }

    pub async fn list_blocks(&self, blocker: &str) -> Result<Vec<String>> {
        Ok(
            sqlx::query_as::<_, (String,)>("SELECT blocked_id FROM blocks WHERE blocker_id=?")
                .bind(blocker)
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|(id,)| id)
                .collect(),
        )
    }
}
