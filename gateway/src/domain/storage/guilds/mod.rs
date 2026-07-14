pub mod audit;
pub mod roles;

use anyhow::Result;

#[allow(unused_imports)]
pub use audit::AuditLogRow;

use super::Pool;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct GuildRow {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    pub created_at: i64,
    pub member_count: i64,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct GuildMemberRow {
    pub user_id: String,
    pub nickname: String,
    pub joined_at: i64,
    #[sqlx(default)]
    pub role_color: String,
    #[sqlx(default)]
    pub role_name: String,
}

#[derive(sqlx::FromRow, Debug)]
pub struct InviteRow {
    pub id: String,
    pub guild_id: String,
    #[sqlx(default)]
    pub guild_name: String,
    pub creator_id: String,
    pub code: String,
    pub max_uses: Option<i64>,
    pub uses: i64,
    pub expires_at: Option<i64>,
    pub created_at: i64,
}

impl super::Storage {
    pub async fn create_guild(&self, owner_id: &str, name: &str) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = super::now_ms();
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query("INSERT INTO guilds (id,owner_id,name,created_at) VALUES (?,?,?,?)")
                    .bind(&id)
                    .bind(owner_id)
                    .bind(name)
                    .bind(now)
                    .execute(p)
                    .await?;
                sqlx::query(
                    "INSERT OR IGNORE INTO guild_members (guild_id,user_id,joined_at) VALUES (?,?,?)",
                )
                .bind(&id)
                .bind(owner_id)
                .bind(now)
                .execute(p)
                .await?;
                let role_id = uuid::Uuid::new_v4().to_string();
                sqlx::query("INSERT INTO roles (id,guild_id,name,permissions,position,created_at) VALUES (?,?,?,?,0,?)")
                    .bind(&role_id).bind(&id).bind("@everyone").bind(u64::MAX as i64).bind(now)
                    .execute(p).await?;
                sqlx::query(
                    "INSERT OR IGNORE INTO member_roles (guild_id,user_id,role_id) VALUES (?,?,?)",
                )
                .bind(&id)
                .bind(owner_id)
                .bind(&role_id)
                .execute(p)
                .await?;
            }
            Pool::Postgres(p) => {
                sqlx::query(
                    "INSERT INTO guilds (id,owner_id,name,created_at) VALUES ($1,$2,$3,$4)",
                )
                .bind(&id)
                .bind(owner_id)
                .bind(name)
                .bind(now)
                .execute(p)
                .await?;
                sqlx::query(
                    "INSERT INTO guild_members (guild_id,user_id,joined_at) VALUES ($1,$2,$3) ON CONFLICT (guild_id,user_id) DO NOTHING",
                )
                .bind(&id)
                .bind(owner_id)
                .bind(now)
                .execute(p)
                .await?;
                let role_id = uuid::Uuid::new_v4().to_string();
                sqlx::query("INSERT INTO roles (id,guild_id,name,permissions,position,created_at) VALUES ($1,$2,$3,$4,0,$5)")
                    .bind(&role_id).bind(&id).bind("@everyone").bind(u64::MAX as i64).bind(now)
                    .execute(p).await?;
                sqlx::query("INSERT INTO member_roles (guild_id,user_id,role_id) VALUES ($1,$2,$3) ON CONFLICT (guild_id,user_id,role_id) DO NOTHING")
                    .bind(&id)
                    .bind(owner_id)
                    .bind(&role_id)
                    .execute(p)
                    .await?;
            }
        }
        Ok(id)
    }

    pub async fn get_guild(&self, guild_id: &str) -> Result<Option<GuildRow>> {
        match &self.pool {
            Pool::Sqlite(p) => Ok(sqlx::query_as::<_, GuildRow>(
                "SELECT g.id,g.owner_id,g.name,g.created_at, \
                 (SELECT COUNT(*) FROM guild_members WHERE guild_id=g.id) as member_count \
                 FROM guilds g WHERE g.id=?",
            )
            .bind(guild_id)
            .fetch_optional(p)
            .await?),
            Pool::Postgres(p) => Ok(sqlx::query_as::<_, GuildRow>(
                "SELECT g.id,g.owner_id,g.name,g.created_at, \
                 (SELECT COUNT(*) FROM guild_members WHERE guild_id=g.id) as member_count \
                 FROM guilds g WHERE g.id=$1",
            )
            .bind(guild_id)
            .fetch_optional(p)
            .await?),
        }
    }

    pub async fn list_user_guilds(&self, user_id: &str) -> Result<Vec<GuildRow>> {
        match &self.pool {
            Pool::Sqlite(p) => Ok(sqlx::query_as::<_, GuildRow>(
                "SELECT g.id,g.owner_id,g.name,g.created_at, \
                 (SELECT COUNT(*) FROM guild_members WHERE guild_id=g.id) as member_count \
                 FROM guilds g JOIN guild_members gm ON g.id=gm.guild_id \
                 WHERE gm.user_id=? ORDER BY g.name",
            )
            .bind(user_id)
            .fetch_all(p)
            .await?),
            Pool::Postgres(p) => Ok(sqlx::query_as::<_, GuildRow>(
                "SELECT g.id,g.owner_id,g.name,g.created_at, \
                 (SELECT COUNT(*) FROM guild_members WHERE guild_id=g.id) as member_count \
                 FROM guilds g JOIN guild_members gm ON g.id=gm.guild_id \
                 WHERE gm.user_id=$1 ORDER BY g.name",
            )
            .bind(user_id)
            .fetch_all(p)
            .await?),
        }
    }

    pub async fn delete_guild(&self, guild_id: &str) -> Result<()> {
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query("DELETE FROM guild_members WHERE guild_id=?")
                    .bind(guild_id)
                    .execute(p)
                    .await?;
                sqlx::query("DELETE FROM roles WHERE guild_id=?")
                    .bind(guild_id)
                    .execute(p)
                    .await?;
                sqlx::query("DELETE FROM invites WHERE guild_id=?")
                    .bind(guild_id)
                    .execute(p)
                    .await?;
                sqlx::query("DELETE FROM guilds WHERE id=?")
                    .bind(guild_id)
                    .execute(p)
                    .await?;
            }
            Pool::Postgres(p) => {
                sqlx::query("DELETE FROM guild_members WHERE guild_id=$1")
                    .bind(guild_id)
                    .execute(p)
                    .await?;
                sqlx::query("DELETE FROM roles WHERE guild_id=$1")
                    .bind(guild_id)
                    .execute(p)
                    .await?;
                sqlx::query("DELETE FROM invites WHERE guild_id=$1")
                    .bind(guild_id)
                    .execute(p)
                    .await?;
                sqlx::query("DELETE FROM guilds WHERE id=$1")
                    .bind(guild_id)
                    .execute(p)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn add_guild_member(&self, guild_id: &str, user_id: &str) -> Result<()> {
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query(
                    "INSERT OR IGNORE INTO guild_members (guild_id,user_id,joined_at) VALUES (?,?,?)",
                )
                .bind(guild_id)
                .bind(user_id)
                .bind(super::now_ms())
                .execute(p)
                .await?;
            }
            Pool::Postgres(p) => {
                sqlx::query(
                    "INSERT INTO guild_members (guild_id,user_id,joined_at) VALUES ($1,$2,$3) ON CONFLICT (guild_id,user_id) DO NOTHING",
                )
                .bind(guild_id)
                .bind(user_id)
                .bind(super::now_ms())
                .execute(p)
                .await?;
            }
        }
        Ok(())
    }

    pub async fn remove_guild_member(&self, guild_id: &str, user_id: &str) -> Result<()> {
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query("DELETE FROM guild_members WHERE guild_id=? AND user_id=?")
                    .bind(guild_id)
                    .bind(user_id)
                    .execute(p)
                    .await?;
            }
            Pool::Postgres(p) => {
                sqlx::query("DELETE FROM guild_members WHERE guild_id=$1 AND user_id=$2")
                    .bind(guild_id)
                    .bind(user_id)
                    .execute(p)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn list_guild_members(&self, guild_id: &str) -> Result<Vec<GuildMemberRow>> {
        match &self.pool {
            Pool::Sqlite(p) => Ok(sqlx::query_as::<_, GuildMemberRow>(
                "SELECT gm.user_id, COALESCE(u.nickname, gm.user_id) as nickname, gm.joined_at, \
                 COALESCE((SELECT r.color FROM roles r \
                           JOIN member_roles mr ON r.id=mr.role_id \
                           WHERE mr.guild_id=gm.guild_id AND mr.user_id=gm.user_id \
                           ORDER BY r.position DESC LIMIT 1), '#ffffff') as role_color, \
                 COALESCE((SELECT r.name FROM roles r \
                           JOIN member_roles mr ON r.id=mr.role_id \
                           WHERE mr.guild_id=gm.guild_id AND mr.user_id=gm.user_id \
                           ORDER BY r.position DESC LIMIT 1), 'member') as role_name \
                 FROM guild_members gm LEFT JOIN users u ON gm.user_id=u.pubkey \
                 WHERE gm.guild_id=? ORDER BY gm.joined_at ASC",
            )
            .bind(guild_id)
            .fetch_all(p)
            .await?),
            Pool::Postgres(p) => Ok(sqlx::query_as::<_, GuildMemberRow>(
                "SELECT gm.user_id, COALESCE(u.nickname, gm.user_id) as nickname, gm.joined_at, \
                 COALESCE((SELECT r.color FROM roles r \
                           JOIN member_roles mr ON r.id=mr.role_id \
                           WHERE mr.guild_id=gm.guild_id AND mr.user_id=gm.user_id \
                           ORDER BY r.position DESC LIMIT 1), '#ffffff') as role_color, \
                 COALESCE((SELECT r.name FROM roles r \
                           JOIN member_roles mr ON r.id=mr.role_id \
                           WHERE mr.guild_id=gm.guild_id AND mr.user_id=gm.user_id \
                           ORDER BY r.position DESC LIMIT 1), 'member') as role_name \
                 FROM guild_members gm LEFT JOIN users u ON gm.user_id=u.pubkey \
                 WHERE gm.guild_id=$1 ORDER BY gm.joined_at ASC",
            )
            .bind(guild_id)
            .fetch_all(p)
            .await?),
        }
    }

    pub async fn assign_role(&self, guild_id: &str, user_id: &str, role_id: &str) -> Result<()> {
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query(
                    "INSERT OR IGNORE INTO member_roles (guild_id,user_id,role_id) VALUES (?,?,?)",
                )
                .bind(guild_id)
                .bind(user_id)
                .bind(role_id)
                .execute(p)
                .await?;
            }
            Pool::Postgres(p) => {
                sqlx::query("INSERT INTO member_roles (guild_id,user_id,role_id) VALUES ($1,$2,$3) ON CONFLICT (guild_id,user_id,role_id) DO NOTHING")
                    .bind(guild_id)
                    .bind(user_id)
                    .bind(role_id)
                    .execute(p)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn remove_role_from_user(
        &self,
        guild_id: &str,
        user_id: &str,
        role_id: &str,
    ) -> Result<()> {
        match &self.pool {
            Pool::Sqlite(p) => {
                sqlx::query(
                    "DELETE FROM member_roles WHERE guild_id=? AND user_id=? AND role_id=?",
                )
                .bind(guild_id)
                .bind(user_id)
                .bind(role_id)
                .execute(p)
                .await?;
            }
            Pool::Postgres(p) => {
                sqlx::query(
                    "DELETE FROM member_roles WHERE guild_id=$1 AND user_id=$2 AND role_id=$3",
                )
                .bind(guild_id)
                .bind(user_id)
                .bind(role_id)
                .execute(p)
                .await?;
            }
        }
        Ok(())
    }

    pub async fn list_guild_roles(&self, guild_id: &str) -> Result<Vec<RoleFullRow>> {
        match &self.pool {
            Pool::Sqlite(p) => Ok(sqlx::query_as::<_, RoleFullRow>(
                "SELECT id, guild_id, name, color, permissions, position, created_at \
                 FROM roles WHERE guild_id=? ORDER BY position DESC",
            )
            .bind(guild_id)
            .fetch_all(p)
            .await?),
            Pool::Postgres(p) => Ok(sqlx::query_as::<_, RoleFullRow>(
                "SELECT id, guild_id, name, color, permissions, position, created_at \
                 FROM roles WHERE guild_id=$1 ORDER BY position DESC",
            )
            .bind(guild_id)
            .fetch_all(p)
            .await?),
        }
    }
}

#[allow(dead_code)]
#[derive(sqlx::FromRow, Debug, Clone)]
pub struct RoleFullRow {
    pub id: String,
    pub guild_id: String,
    pub name: String,
    pub color: String,
    pub permissions: i64,
    pub position: i32,
    pub created_at: i64,
}
