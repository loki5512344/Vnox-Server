use anyhow::Result;
use sqlx::AssertSqlSafe;

use crate::proto::E2eeDmMessagePayload;

use super::Pool;

#[derive(sqlx::FromRow)]
struct E2eeDmMsgRow {
    dm_id: String,
    sender_id: String,
    ciphertext: Vec<u8>,
    created_at: i64,
}

impl super::Storage {
    pub async fn save_e2ee_dm_message(
        &self,
        dm_id: &str,
        sender_id: &str,
        ciphertext: &[u8],
    ) -> Result<E2eeDmMessagePayload> {
        let msg_id = uuid::Uuid::new_v4().to_string();
        let ts = super::now_ms();
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query(
                    "INSERT INTO e2ee_dm_messages (id,dm_id,sender_id,ciphertext,created_at) VALUES (?,?,?,?,?)",
                )
                .bind(&msg_id)
                .bind(dm_id)
                .bind(sender_id)
                .bind(ciphertext)
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
                    "INSERT INTO e2ee_dm_messages (id,dm_id,sender_id,ciphertext,created_at) VALUES ($1,$2,$3,$4,$5)",
                )
                .bind(&msg_id)
                .bind(dm_id)
                .bind(sender_id)
                .bind(ciphertext)
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
        Ok(E2eeDmMessagePayload {
            dm_id: dm_id.to_string(),
            sender_id: sender_id.to_string(),
            ciphertext: ciphertext.to_vec(),
            timestamp: ts,
        })
    }

    pub async fn get_e2ee_dm_messages(
        &self,
        dm_id: &str,
        limit: i64,
    ) -> Result<Vec<E2eeDmMessagePayload>> {
        let rows = match &self.pool {
            Pool::Sqlite(p) => {
                let sql = AssertSqlSafe(
                    "SELECT dm_id,sender_id,ciphertext,created_at FROM \
                     (SELECT * FROM e2ee_dm_messages WHERE dm_id=? \
                     ORDER BY created_at DESC LIMIT ?) ORDER BY created_at ASC",
                );
                sqlx::query_as::<_, E2eeDmMsgRow>(sql)
                    .bind(dm_id)
                    .bind(limit)
                    .fetch_all(p)
                    .await?
            }
            Pool::Postgres(p) => {
                let sql = AssertSqlSafe(
                    "SELECT dm_id,sender_id,ciphertext,created_at FROM \
                     (SELECT * FROM e2ee_dm_messages WHERE dm_id=$1 \
                     ORDER BY created_at DESC LIMIT $2) ORDER BY created_at ASC",
                );
                sqlx::query_as::<_, E2eeDmMsgRow>(sql)
                    .bind(dm_id)
                    .bind(limit)
                    .fetch_all(p)
                    .await?
            }
        };
        Ok(rows
            .into_iter()
            .map(|r| E2eeDmMessagePayload {
                dm_id: r.dm_id,
                sender_id: r.sender_id,
                ciphertext: r.ciphertext,
                timestamp: r.created_at,
            })
            .collect())
    }
}
