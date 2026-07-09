use anyhow::Result;

use super::InviteRow;

#[derive(sqlx::FromRow, Debug)]
pub struct RoleRow {
    pub color: String,
    pub position: i32,
}

impl super::super::Storage {
    pub async fn create_role(
        &self,
        guild_id: &str,
        name: &str,
        color: &str,
        permissions: u64,
        position: i32,
    ) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO roles (id,guild_id,name,color,permissions,position,created_at) VALUES (?,?,?,?,?,?,?)")
            .bind(&id).bind(guild_id).bind(name).bind(color).bind(permissions as i64).bind(position).bind(super::super::now_ms())
            .execute(&self.pool).await?;
        Ok(id)
    }

    pub async fn delete_role(&self, role_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM roles WHERE id=?")
            .bind(role_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM member_roles WHERE role_id=?")
            .bind(role_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_user_roles(&self, guild_id: &str, user_id: &str) -> Result<Vec<RoleRow>> {
        Ok(sqlx::query_as::<_, RoleRow>(
            "SELECT r.id,r.guild_id,r.name,r.color,r.permissions,r.position \
             FROM roles r JOIN member_roles mr ON r.id=mr.role_id \
             WHERE mr.guild_id=? AND mr.user_id=? ORDER BY r.position DESC",
        )
        .bind(guild_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn get_user_role_perms(&self, guild_id: &str, user_id: &str) -> Result<Vec<u64>> {
        #[derive(sqlx::FromRow)]
        struct P {
            permissions: i64,
        }
        let rows: Vec<P> = sqlx::query_as::<_, P>(
            "SELECT r.permissions FROM roles r \
             JOIN member_roles mr ON r.id=mr.role_id \
             WHERE mr.guild_id=? AND mr.user_id=?",
        )
        .bind(guild_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.permissions as u64).collect())
    }

    pub async fn create_invite(
        &self,
        guild_id: &str,
        creator_id: &str,
        max_uses: Option<i64>,
        expires_in_s: Option<i64>,
    ) -> Result<InviteRow> {
        let id = uuid::Uuid::new_v4().to_string();
        let code = super::super::generate_invite_code();
        let now = super::super::now_ms();
        let expires_at = expires_in_s.map(|s| now + s * 1000);
        sqlx::query("INSERT INTO invites (id,guild_id,creator_id,code,max_uses,expires_at,created_at) VALUES (?,?,?,?,?,?,?)")
            .bind(&id).bind(guild_id).bind(creator_id).bind(&code).bind(max_uses).bind(expires_at).bind(now)
            .execute(&self.pool).await?;
        Ok(InviteRow {
            id,
            guild_id: guild_id.into(),
            guild_name: String::new(),
            creator_id: creator_id.into(),
            code,
            max_uses,
            uses: 0,
            expires_at,
            created_at: now,
        })
    }

    pub async fn get_invite_by_code(&self, code: &str) -> Result<Option<InviteRow>> {
        Ok(sqlx::query_as::<_, InviteRow>(
            "SELECT i.id,i.guild_id,i.creator_id,i.code,i.max_uses,i.uses,i.expires_at,i.created_at, \
             g.name as guild_name FROM invites i JOIN guilds g ON i.guild_id=g.id WHERE i.code=?"
        ).bind(code).fetch_optional(&self.pool).await?)
    }

    pub async fn use_invite(&self, invite_id: &str) -> Result<()> {
        sqlx::query("UPDATE invites SET uses=uses+1 WHERE id=?")
            .bind(invite_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_invite(&self, invite_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM invites WHERE id=?")
            .bind(invite_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
