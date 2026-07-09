# Database Schema

Version: 1.0  
Status: Specification and implementation guide

---

## Overview

Vnox uses SQLite for local deployment and supports PostgreSQL for larger deployments.

Database schema defines storage for:
- Users and identity
- Guilds, channels, categories
- Roles and permissions
- Messages and history
- Direct messages
- Presence and online status
- Audit logs

All tables use UUID for primary keys (no auto-increment) for federation compatibility.

---

## Core Tables

### users

Global user identity across all Vnox instances.

```sql
CREATE TABLE users (
  id TEXT PRIMARY KEY,
  username TEXT NOT NULL,
  identity_key TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL
);

CREATE INDEX idx_users_username ON users(username);
```

| Column       | Type      | Constraints    | Description          |
| ------------ | --------- | -------------- | -------------------- |
| id           | TEXT      | PRIMARY KEY    | UUID                 |
| username     | TEXT      | NOT NULL       | Display name         |
| identity_key | TEXT      | NOT NULL       | Ed25519 public key   |
| created_at   | TIMESTAMP | NOT NULL       | Creation time        |
| updated_at   | TIMESTAMP | NOT NULL       | Last modification    |

---

### sessions

Active user sessions on a gateway instance.

```sql
CREATE TABLE sessions (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  session_token TEXT NOT NULL UNIQUE,
  expires_at TIMESTAMP NOT NULL,
  created_at TIMESTAMP NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(id)
);

CREATE INDEX idx_sessions_user ON sessions(user_id);
CREATE INDEX idx_sessions_expires ON sessions(expires_at);
```

| Column        | Type      | Constraints    | Description          |
| ------------- | --------- | -------------- | -------------------- |
| id            | TEXT      | PRIMARY KEY    | Session UUID         |
| user_id       | TEXT      | FK users       | User owning session  |
| session_token | TEXT      | UNIQUE         | Auth token           |
| expires_at    | TIMESTAMP | NOT NULL       | Expiration time      |
| created_at    | TIMESTAMP | NOT NULL       | Creation time        |

---

## Guild Tables

### guilds

Guild entities (communities/servers).

```sql
CREATE TABLE guilds (
  id TEXT PRIMARY KEY,
  owner_id TEXT NOT NULL,
  name TEXT NOT NULL,
  description TEXT,
  icon TEXT,
  banner TEXT,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  FOREIGN KEY(owner_id) REFERENCES users(id)
);

CREATE INDEX idx_guilds_owner ON guilds(owner_id);
```

| Column      | Type      | Constraints    | Description          |
| ----------- | --------- | -------------- | -------------------- |
| id          | TEXT      | PRIMARY KEY    | Guild UUID           |
| owner_id    | TEXT      | FK users       | Guild owner          |
| name        | TEXT      | NOT NULL       | Guild name           |
| description | TEXT      |                | Guild description    |
| icon        | TEXT      |                | Asset ID             |
| banner      | TEXT      |                | Asset ID             |
| created_at  | TIMESTAMP | NOT NULL       | Creation time        |
| updated_at  | TIMESTAMP | NOT NULL       | Last modification    |

---

### guild_members

Guild membership tracking.

```sql
CREATE TABLE guild_members (
  guild_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  nickname TEXT,
  joined_at TIMESTAMP NOT NULL,
  timeout_until TIMESTAMP,
  PRIMARY KEY (guild_id, user_id),
  FOREIGN KEY(guild_id) REFERENCES guilds(id),
  FOREIGN KEY(user_id) REFERENCES users(id)
);

CREATE INDEX idx_guild_members_user ON guild_members(user_id);
CREATE INDEX idx_guild_members_joined ON guild_members(joined_at);
```

| Column        | Type      | Constraints    | Description          |
| ------------- | --------- | -------------- | -------------------- |
| guild_id      | TEXT      | PRIMARY KEY    | Guild UUID           |
| user_id       | TEXT      | PRIMARY KEY    | User UUID            |
| nickname      | TEXT      |                | Guild-specific nick  |
| joined_at     | TIMESTAMP | NOT NULL       | Join time            |
| timeout_until | TIMESTAMP |                | Moderation timeout   |

---

### categories

Channel categories for organization.

```sql
CREATE TABLE categories (
  id TEXT PRIMARY KEY,
  guild_id TEXT NOT NULL,
  name TEXT NOT NULL,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  FOREIGN KEY(guild_id) REFERENCES guilds(id)
);

CREATE INDEX idx_categories_guild ON categories(guild_id);
CREATE INDEX idx_categories_position ON categories(guild_id, position);
```

| Column     | Type      | Constraints    | Description          |
| ---------- | --------- | -------------- | -------------------- |
| id         | TEXT      | PRIMARY KEY    | Category UUID        |
| guild_id   | TEXT      | FK guilds      | Parent guild         |
| name       | TEXT      | NOT NULL       | Display name         |
| position   | INTEGER   | NOT NULL       | Sort order           |
| created_at | TIMESTAMP | NOT NULL       | Creation time        |
| updated_at | TIMESTAMP | NOT NULL       | Last modification    |

---

### channels

Communication channels (text, voice, announcements).

```sql
CREATE TABLE channels (
  id TEXT PRIMARY KEY,
  guild_id TEXT NOT NULL,
  category_id TEXT,
  type TEXT NOT NULL,
  name TEXT NOT NULL,
  topic TEXT,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  FOREIGN KEY(guild_id) REFERENCES guilds(id),
  FOREIGN KEY(category_id) REFERENCES categories(id)
);

CREATE INDEX idx_channels_guild ON channels(guild_id);
CREATE INDEX idx_channels_category ON channels(category_id);
CREATE INDEX idx_channels_position ON channels(guild_id, position);
```

| Column      | Type      | Constraints    | Description          |
| ----------- | --------- | -------------- | -------------------- |
| id          | TEXT      | PRIMARY KEY    | Channel UUID         |
| guild_id    | TEXT      | FK guilds      | Parent guild         |
| category_id | TEXT      | FK categories  | Parent category      |
| type        | TEXT      | NOT NULL       | TEXT/VOICE/etc       |
| name        | TEXT      | NOT NULL       | Display name         |
| topic       | TEXT      |                | Channel description  |
| position    | INTEGER   | NOT NULL       | Sort order           |
| created_at  | TIMESTAMP | NOT NULL       | Creation time        |
| updated_at  | TIMESTAMP | NOT NULL       | Last modification    |

---

## Role & Permission Tables

### roles

Guild roles with permission bitmasks.

```sql
CREATE TABLE roles (
  id TEXT PRIMARY KEY,
  guild_id TEXT NOT NULL,
  name TEXT NOT NULL,
  color INTEGER NOT NULL DEFAULT 0,
  permissions INTEGER NOT NULL DEFAULT 0,
  position INTEGER NOT NULL DEFAULT 0,
  hoist INTEGER NOT NULL DEFAULT 0,
  mentionable INTEGER NOT NULL DEFAULT 1,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  FOREIGN KEY(guild_id) REFERENCES guilds(id)
);

CREATE INDEX idx_roles_guild ON roles(guild_id);
CREATE INDEX idx_roles_position ON roles(guild_id, position);
```

| Column       | Type      | Constraints    | Description          |
| ------------ | --------- | -------------- | -------------------- |
| id           | TEXT      | PRIMARY KEY    | Role UUID            |
| guild_id     | TEXT      | FK guilds      | Parent guild         |
| name         | TEXT      | NOT NULL       | Display name         |
| color        | INTEGER   | NOT NULL       | RGB color value      |
| permissions  | INTEGER   | NOT NULL       | Bitmask (u128)       |
| position     | INTEGER   | NOT NULL       | Hierarchy position   |
| hoist        | INTEGER   | NOT NULL       | Display separately   |
| mentionable  | INTEGER   | NOT NULL       | Can be @mentioned    |
| created_at   | TIMESTAMP | NOT NULL       | Creation time        |
| updated_at   | TIMESTAMP | NOT NULL       | Last modification    |

---

### member_roles

Many-to-many mapping of members to roles.

```sql
CREATE TABLE member_roles (
  member_guild_id TEXT NOT NULL,
  member_user_id TEXT NOT NULL,
  role_id TEXT NOT NULL,
  PRIMARY KEY (member_guild_id, member_user_id, role_id),
  FOREIGN KEY(member_guild_id, member_user_id) 
    REFERENCES guild_members(guild_id, user_id),
  FOREIGN KEY(role_id) REFERENCES roles(id)
);

CREATE INDEX idx_member_roles_role ON member_roles(role_id);
```

| Column           | Type | Constraints    | Description          |
| ---------------- | ---- | -------------- | -------------------- |
| member_guild_id  | TEXT | PRIMARY KEY    | Guild UUID           |
| member_user_id   | TEXT | PRIMARY KEY    | User UUID            |
| role_id          | TEXT | PRIMARY KEY    | Role UUID            |

---

### channel_overrides

Role and user permission overrides per channel.

```sql
CREATE TABLE channel_overrides (
  id TEXT PRIMARY KEY,
  channel_id TEXT NOT NULL,
  target_id TEXT NOT NULL,
  target_type TEXT NOT NULL,
  allow_mask INTEGER NOT NULL DEFAULT 0,
  deny_mask INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  FOREIGN KEY(channel_id) REFERENCES channels(id)
);

CREATE INDEX idx_channel_overrides_channel 
  ON channel_overrides(channel_id);
CREATE INDEX idx_channel_overrides_target 
  ON channel_overrides(target_id, target_type);
```

| Column      | Type      | Constraints    | Description              |
| ----------- | --------- | -------------- | ------------------------ |
| id          | TEXT      | PRIMARY KEY    | Override UUID            |
| channel_id  | TEXT      | FK channels    | Channel                  |
| target_id   | TEXT      | NOT NULL       | Role or User UUID        |
| target_type | TEXT      | NOT NULL       | "ROLE" or "USER"         |
| allow_mask  | INTEGER   | NOT NULL       | Allowed permissions      |
| deny_mask   | INTEGER   | NOT NULL       | Denied permissions       |
| created_at  | TIMESTAMP | NOT NULL       | Creation time            |
| updated_at  | TIMESTAMP | NOT NULL       | Last modification        |

---

## Message Tables

### messages

Text messages in channels.

```sql
CREATE TABLE messages (
  id TEXT PRIMARY KEY,
  channel_id TEXT NOT NULL,
  sender_id TEXT NOT NULL,
  content TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP,
  deleted_at TIMESTAMP,
  FOREIGN KEY(channel_id) REFERENCES channels(id),
  FOREIGN KEY(sender_id) REFERENCES users(id)
);

CREATE INDEX idx_messages_channel ON messages(channel_id, created_at DESC);
CREATE INDEX idx_messages_sender ON messages(sender_id);
```

| Column     | Type      | Constraints    | Description          |
| ---------- | --------- | -------------- | -------------------- |
| id         | TEXT      | PRIMARY KEY    | Message UUID         |
| channel_id | TEXT      | FK channels    | Parent channel       |
| sender_id  | TEXT      | FK users       | Message author       |
| content    | TEXT      | NOT NULL       | Encrypted content    |
| created_at | TIMESTAMP | NOT NULL       | Creation time        |
| updated_at | TIMESTAMP |                | Edit time            |
| deleted_at | TIMESTAMP |                | Deletion time (soft) |

---

### message_reactions

Emoji reactions on messages.

```sql
CREATE TABLE message_reactions (
  message_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  emoji TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  PRIMARY KEY (message_id, user_id, emoji),
  FOREIGN KEY(message_id) REFERENCES messages(id),
  FOREIGN KEY(user_id) REFERENCES users(id)
);

CREATE INDEX idx_reactions_message ON message_reactions(message_id);
```

| Column     | Type      | Constraints    | Description          |
| ---------- | --------- | -------------- | -------------------- |
| message_id | TEXT      | PRIMARY KEY    | Message UUID         |
| user_id    | TEXT      | PRIMARY KEY    | User UUID            |
| emoji      | TEXT      | PRIMARY KEY    | Emoji character      |
| created_at | TIMESTAMP | NOT NULL       | Reaction time        |

---

## Direct Message Tables

### direct_messages

DM channel tracking (not per-guild).

```sql
CREATE TABLE direct_messages (
  dm_id TEXT PRIMARY KEY,
  user_id_1 TEXT NOT NULL,
  user_id_2 TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  FOREIGN KEY(user_id_1) REFERENCES users(id),
  FOREIGN KEY(user_id_2) REFERENCES users(id)
);

CREATE UNIQUE INDEX idx_dm_users 
  ON direct_messages(
    MIN(user_id_1, user_id_2),
    MAX(user_id_1, user_id_2)
  );
```

| Column     | Type      | Constraints    | Description          |
| ---------- | --------- | -------------- | -------------------- |
| dm_id      | TEXT      | PRIMARY KEY    | DM UUID              |
| user_id_1  | TEXT      | FK users       | First user (lexicog) |
| user_id_2  | TEXT      | FK users       | Second user          |
| created_at | TIMESTAMP | NOT NULL       | Creation time        |
| updated_at | TIMESTAMP | NOT NULL       | Last message time    |

---

### dm_messages

Direct messages (archived).

```sql
CREATE TABLE dm_messages (
  id TEXT PRIMARY KEY,
  dm_id TEXT NOT NULL,
  sender_id TEXT NOT NULL,
  content TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  FOREIGN KEY(dm_id) REFERENCES direct_messages(dm_id),
  FOREIGN KEY(sender_id) REFERENCES users(id)
);

CREATE INDEX idx_dm_messages_dm ON dm_messages(dm_id, created_at DESC);
CREATE INDEX idx_dm_messages_sender ON dm_messages(sender_id);
```

| Column     | Type      | Constraints    | Description          |
| ---------- | --------- | -------------- | -------------------- |
| id         | TEXT      | PRIMARY KEY    | Message UUID         |
| dm_id      | TEXT      | FK dm tables   | Parent DM            |
| sender_id  | TEXT      | FK users       | Message author       |
| content    | TEXT      | NOT NULL       | Encrypted content    |
| created_at | TIMESTAMP | NOT NULL       | Creation time        |

---

## Invitation Tables

### invites

Guild invitations.

```sql
CREATE TABLE invites (
  id TEXT PRIMARY KEY,
  guild_id TEXT NOT NULL,
  creator_id TEXT NOT NULL,
  code TEXT NOT NULL UNIQUE,
  expires_at TIMESTAMP,
  max_uses INTEGER,
  uses INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMP NOT NULL,
  FOREIGN KEY(guild_id) REFERENCES guilds(id),
  FOREIGN KEY(creator_id) REFERENCES users(id)
);

CREATE INDEX idx_invites_guild ON invites(guild_id);
CREATE INDEX idx_invites_code ON invites(code);
CREATE INDEX idx_invites_expires ON invites(expires_at);
```

| Column     | Type      | Constraints    | Description          |
| ---------- | --------- | -------------- | -------------------- |
| id         | TEXT      | PRIMARY KEY    | Invite UUID          |
| guild_id   | TEXT      | FK guilds      | Target guild         |
| creator_id | TEXT      | FK users       | Creator              |
| code       | TEXT      | UNIQUE         | Invite slug          |
| expires_at | TIMESTAMP |                | Expiration time      |
| max_uses   | INTEGER   |                | Use limit            |
| uses       | INTEGER   | NOT NULL       | Current uses         |
| created_at | TIMESTAMP | NOT NULL       | Creation time        |

---

## Presence Tables

### presence

User online status and activity.

```sql
CREATE TABLE presence (
  user_id TEXT PRIMARY KEY,
  status TEXT NOT NULL,
  activity_type TEXT,
  activity_text TEXT,
  last_seen TIMESTAMP NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(id)
);

CREATE INDEX idx_presence_status ON presence(status);
CREATE INDEX idx_presence_last_seen ON presence(last_seen);
```

| Column        | Type      | Constraints    | Description          |
| ------------- | --------- | -------------- | -------------------- |
| user_id       | TEXT      | PRIMARY KEY    | User UUID            |
| status        | TEXT      | NOT NULL       | ONLINE/IDLE/etc      |
| activity_type | TEXT      |                | PLAYING/LISTENING    |
| activity_text | TEXT      |                | Activity description |
| last_seen     | TIMESTAMP | NOT NULL       | Last activity time   |

---

## Friendship Tables

### friends

User friendship relationships.

```sql
CREATE TABLE friends (
  requester_id TEXT NOT NULL,
  target_id TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  PRIMARY KEY (requester_id, target_id),
  FOREIGN KEY(requester_id) REFERENCES users(id),
  FOREIGN KEY(target_id) REFERENCES users(id)
);

CREATE INDEX idx_friends_target ON friends(target_id, status);
```

| Column       | Type      | Constraints    | Description              |
| ------------ | --------- | -------------- | ------------------------ |
| requester_id | TEXT      | PRIMARY KEY    | Request originator       |
| target_id    | TEXT      | PRIMARY KEY    | Request recipient        |
| status       | TEXT      | NOT NULL       | pending/accepted/blocked |
| created_at   | TIMESTAMP | NOT NULL       | Request time             |

---

## Audit Log Table

### audit_logs

Moderation and administrative actions.

```sql
CREATE TABLE audit_logs (
  id TEXT PRIMARY KEY,
  guild_id TEXT NOT NULL,
  actor_id TEXT NOT NULL,
  action TEXT NOT NULL,
  target_id TEXT,
  target_type TEXT,
  reason TEXT,
  changes TEXT,
  created_at TIMESTAMP NOT NULL,
  FOREIGN KEY(guild_id) REFERENCES guilds(id),
  FOREIGN KEY(actor_id) REFERENCES users(id)
);

CREATE INDEX idx_audit_logs_guild ON audit_logs(guild_id, created_at DESC);
CREATE INDEX idx_audit_logs_actor ON audit_logs(actor_id);
CREATE INDEX idx_audit_logs_target 
  ON audit_logs(target_id, target_type);
```

| Column      | Type      | Constraints    | Description          |
| ----------- | --------- | -------------- | -------------------- |
| id          | TEXT      | PRIMARY KEY    | Log UUID             |
| guild_id    | TEXT      | FK guilds      | Guild context        |
| actor_id    | TEXT      | FK users       | Admin/moderator      |
| action      | TEXT      | NOT NULL       | ACTION_TYPE          |
| target_id   | TEXT      |                | Affected entity      |
| target_type | TEXT      |                | Entity type          |
| reason      | TEXT      |                | Reason for action    |
| changes     | TEXT      |                | JSON of old/new      |
| created_at  | TIMESTAMP | NOT NULL       | Action time          |

---

## Migration Strategy

### Phase 1.1 (Current)

```sql
-- Already exists
CREATE TABLE users
CREATE TABLE sessions
CREATE TABLE channels
CREATE TABLE messages
CREATE TABLE direct_messages
CREATE TABLE dm_messages
```

### Phase 1.2

```sql
-- Add guild system
CREATE TABLE guilds
CREATE TABLE guild_members
CREATE TABLE categories
CREATE TABLE roles
CREATE TABLE member_roles
CREATE TABLE channel_overrides
CREATE TABLE invites
CREATE TABLE presence
CREATE TABLE friends
CREATE TABLE audit_logs
```

### Backward Compatibility

During transition to guilds:
- Create a default guild per deployment
- Migrate existing channels to default guild
- Add all existing users as guild members

---

## Performance Considerations

### Indexes

Key composite indexes:
- `messages(channel_id, created_at DESC)` — range queries
- `dm_messages(dm_id, created_at DESC)` — pagination
- `presence(status, last_seen)` — online user queries
- `guild_members(guild_id, joined_at)` — membership lists

### Query Patterns

Optimize for:
- Message history fetch (paginated, DESC)
- Channel member list
- Permission resolution (cached)
- Online presence broadcast
- Recent DM threads

### Caching

Recommended client-side caches:
- Guild data (invalidated on GUILD_UPDATE)
- Channel list (invalidated on CHANNEL_CREATE/DELETE)
- User presence (invalidated on PRESENCE_UPDATE)
- Role permissions (cached by guild)

---

## PostgreSQL vs SQLite

### SQLite (Development, Single-Server)

```toml
[database]
type = "sqlite"
path = "data/vnox.db"
```

### PostgreSQL (Production, Distributed)

```toml
[database]
type = "postgres"
url = "postgresql://user:pass@localhost/vnox"
pool_size = 32
```

Both use identical schema; driver handles translation of:
- `INTEGER` ↔ `BIGINT`
- `TEXT` ↔ `VARCHAR`
- UUID handling
