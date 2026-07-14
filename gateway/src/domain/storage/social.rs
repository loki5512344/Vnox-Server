use anyhow::Result;

use super::Pool;

impl super::Storage {
    pub async fn create_friend_request(&self, from_id: &str, to_id: &str) -> Result<bool> {
        let (u1, u2) = if from_id < to_id {
            (from_id, to_id)
        } else {
            (to_id, from_id)
        };
        match &self.pool {
            Pool::Sqlite(p) => {
                let exists = sqlx::query_as::<_, (String,)>(
                    "SELECT user_id_1 FROM friendships WHERE user_id_1=? AND user_id_2=?",
                )
                .bind(u1)
                .bind(u2)
                .fetch_optional(p)
                .await?;
                if exists.is_some() {
                    return Ok(false);
                }
                let id = uuid::Uuid::new_v4().to_string();
                sqlx::query(
                    "INSERT OR IGNORE INTO friend_requests (id,from_user_id,to_user_id,status,created_at) VALUES (?,?,?,?,?)"
                ).bind(&id).bind(from_id).bind(to_id).bind("PENDING").bind(super::now_ms())
                 .execute(p).await?;
                Ok(true)
            }
            Pool::Postgres(p) => {
                let exists = sqlx::query_as::<_, (String,)>(
                    "SELECT user_id_1 FROM friendships WHERE user_id_1=$1 AND user_id_2=$2",
                )
                .bind(u1)
                .bind(u2)
                .fetch_optional(p)
                .await?;
                if exists.is_some() {
                    return Ok(false);
                }
                let id = uuid::Uuid::new_v4().to_string();
                sqlx::query(
                    "INSERT INTO friend_requests (id,from_user_id,to_user_id,status,created_at) VALUES ($1,$2,$3,$4,$5) ON CONFLICT (id) DO NOTHING"
                ).bind(&id).bind(from_id).bind(to_id).bind("PENDING").bind(super::now_ms())
                 .execute(p).await?;
                Ok(true)
            }
        }
    }

    pub async fn accept_friend_request(&self, from_id: &str, to_id: &str) -> Result<bool> {
        match &self.pool {
            Pool::Sqlite(p) => {
                let updated = sqlx::query(
                    "UPDATE friend_requests SET status='ACCEPTED' WHERE from_user_id=? AND to_user_id=? AND status='PENDING'"
                ).bind(from_id).bind(to_id).execute(p).await?;
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
                .execute(p)
                .await?;
                Ok(true)
            }
            Pool::Postgres(p) => {
                let updated = sqlx::query(
                    "UPDATE friend_requests SET status='ACCEPTED' WHERE from_user_id=$1 AND to_user_id=$2 AND status='PENDING'"
                ).bind(from_id).bind(to_id).execute(p).await?;
                if updated.rows_affected() == 0 {
                    return Ok(false);
                }
                let (u1, u2) = if from_id < to_id {
                    (from_id, to_id)
                } else {
                    (to_id, from_id)
                };
                sqlx::query(
                    "INSERT INTO friendships (user_id_1,user_id_2,created_at) VALUES ($1,$2,$3) ON CONFLICT (user_id_1,user_id_2) DO NOTHING",
                )
                .bind(u1)
                .bind(u2)
                .bind(super::now_ms())
                .execute(p)
                .await?;
                Ok(true)
            }
        }
    }

    pub async fn decline_friend_request(&self, from_id: &str, to_id: &str) -> Result<()> {
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query("UPDATE friend_requests SET status='DECLINED' WHERE from_user_id=? AND to_user_id=? AND status='PENDING'")
                    .bind(from_id).bind(to_id).execute(p).await?;
            }
            Pool::Postgres(p) => {
                sqlx::query("UPDATE friend_requests SET status='DECLINED' WHERE from_user_id=$1 AND to_user_id=$2 AND status='PENDING'")
                    .bind(from_id).bind(to_id).execute(p).await?;
            }
        }
        Ok(())
    }

    pub async fn remove_friend(&self, user_a: &str, user_b: &str) -> Result<()> {
        let (u1, u2) = if user_a < user_b {
            (user_a, user_b)
        } else {
            (user_b, user_a)
        };
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query("DELETE FROM friendships WHERE user_id_1=? AND user_id_2=?")
                    .bind(u1)
                    .bind(u2)
                    .execute(p)
                    .await?;
            }
            Pool::Postgres(p) => {
                sqlx::query("DELETE FROM friendships WHERE user_id_1=$1 AND user_id_2=$2")
                    .bind(u1)
                    .bind(u2)
                    .execute(p)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn list_friends(&self, user_id: &str) -> Result<Vec<String>> {
        match &self.pool {
            Pool::Sqlite(p) => {
                let rows1 = sqlx::query_as::<_, (String,)>(
                    "SELECT user_id_2 FROM friendships WHERE user_id_1=?",
                )
                .bind(user_id)
                .fetch_all(p)
                .await?;
                let rows2 = sqlx::query_as::<_, (String,)>(
                    "SELECT user_id_1 FROM friendships WHERE user_id_2=?",
                )
                .bind(user_id)
                .fetch_all(p)
                .await?;
                Ok(rows1.into_iter().chain(rows2).map(|(id,)| id).collect())
            }
            Pool::Postgres(p) => {
                let rows1 = sqlx::query_as::<_, (String,)>(
                    "SELECT user_id_2 FROM friendships WHERE user_id_1=$1",
                )
                .bind(user_id)
                .fetch_all(p)
                .await?;
                let rows2 = sqlx::query_as::<_, (String,)>(
                    "SELECT user_id_1 FROM friendships WHERE user_id_2=$1",
                )
                .bind(user_id)
                .fetch_all(p)
                .await?;
                Ok(rows1.into_iter().chain(rows2).map(|(id,)| id).collect())
            }
        }
    }

    pub async fn is_friend(&self, user_a: &str, user_b: &str) -> Result<bool> {
        let (u1, u2) = if user_a < user_b {
            (user_a, user_b)
        } else {
            (user_b, user_a)
        };
        match &self.pool {
            Pool::Sqlite(p) => Ok(sqlx::query_as::<_, (String,)>(
                "SELECT user_id_1 FROM friendships WHERE user_id_1=? AND user_id_2=?",
            )
            .bind(u1)
            .bind(u2)
            .fetch_optional(p)
            .await?
            .is_some()),
            Pool::Postgres(p) => Ok(sqlx::query_as::<_, (String,)>(
                "SELECT user_id_1 FROM friendships WHERE user_id_1=$1 AND user_id_2=$2",
            )
            .bind(u1)
            .bind(u2)
            .fetch_optional(p)
            .await?
            .is_some()),
        }
    }

    pub async fn block_user(&self, blocker: &str, blocked: &str) -> Result<()> {
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query(
                    "INSERT OR IGNORE INTO blocks (blocker_id,blocked_id,created_at) VALUES (?,?,?)",
                )
                .bind(blocker)
                .bind(blocked)
                .bind(super::now_ms())
                .execute(p)
                .await?;
            }
            Pool::Postgres(p) => {
                sqlx::query(
                    "INSERT INTO blocks (blocker_id,blocked_id,created_at) VALUES ($1,$2,$3) ON CONFLICT (blocker_id,blocked_id) DO NOTHING",
                )
                .bind(blocker)
                .bind(blocked)
                .bind(super::now_ms())
                .execute(p)
                .await?;
            }
        }
        Ok(())
    }

    pub async fn unblock_user(&self, blocker: &str, blocked: &str) -> Result<()> {
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query("DELETE FROM blocks WHERE blocker_id=? AND blocked_id=?")
                    .bind(blocker)
                    .bind(blocked)
                    .execute(p)
                    .await?;
            }
            Pool::Postgres(p) => {
                sqlx::query("DELETE FROM blocks WHERE blocker_id=$1 AND blocked_id=$2")
                    .bind(blocker)
                    .bind(blocked)
                    .execute(p)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn is_blocked(&self, blocker: &str, blocked: &str) -> Result<bool> {
        match &self.pool {
            Pool::Sqlite(p) => Ok(sqlx::query_as::<_, (String,)>(
                "SELECT blocker_id FROM blocks WHERE blocker_id=? AND blocked_id=?",
            )
            .bind(blocker)
            .bind(blocked)
            .fetch_optional(p)
            .await?
            .is_some()),
            Pool::Postgres(p) => Ok(sqlx::query_as::<_, (String,)>(
                "SELECT blocker_id FROM blocks WHERE blocker_id=$1 AND blocked_id=$2",
            )
            .bind(blocker)
            .bind(blocked)
            .fetch_optional(p)
            .await?
            .is_some()),
        }
    }

    pub async fn list_blocks(&self, blocker: &str) -> Result<Vec<String>> {
        match &self.pool {
            Pool::Sqlite(p) => Ok(sqlx::query_as::<_, (String,)>(
                "SELECT blocked_id FROM blocks WHERE blocker_id=?",
            )
            .bind(blocker)
            .fetch_all(p)
            .await?
            .into_iter()
            .map(|(id,)| id)
            .collect()),
            Pool::Postgres(p) => Ok(sqlx::query_as::<_, (String,)>(
                "SELECT blocked_id FROM blocks WHERE blocker_id=$1",
            )
            .bind(blocker)
            .fetch_all(p)
            .await?
            .into_iter()
            .map(|(id,)| id)
            .collect()),
        }
    }
}
