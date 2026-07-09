# Admin Panel — Design

> Target: Phase 1.1 (core) → Phase 1.2 (guilds, roles) → Phase 2 (plugins, bot permissions)

## Separation of concerns

| Layer | Config file (TOML) | Admin panel (GUI) |
|-------|-------------------|-------------------|
| **Infrastructure** | `bind`, `ports`, `data_dir`, TLS certs | — |
| **Node identity** | `name`, `address` | Display only |
| **Storage** | `backend`, `sqlite_path`, `retention` | — |
| **Security** | federation gate, rate limits, session timeout | Ban list, mute, kick |
| **Runtime state** | — | Channels, roles, members, invites, audit log |
| **Plugins** | Plugin enable/disable | Permission keys, commands |

**Rule of thumb:** If it needs a server restart → TOML. If it can apply live → GUI.

---

## Permission System

Designed for both built-in actions and plugin/bot commands (inspired by LuckPerms).

### Built-in permission keys

```
vnox.admin.manage_channels      — create, edit, delete channels
vnox.admin.manage_roles         — create, edit, delete roles
vnox.admin.manage_members       — kick, ban, mute, timeout
vnox.admin.manage_invites       — create, delete invites
vnox.admin.manage_guild         — edit guild name, icon, settings
vnox.admin.view_audit_log       — read audit log

vnox.channel.read               — view channel
vnox.channel.send               — send messages
vnox.channel.voice_connect      — join voice channel
vnox.channel.voice_speak        — speak in voice (push-to-talk)
vnox.channel.voice_mute_members — server-mute others

vnox.dm.send                    — send direct messages
vnox.friend.request             — send friend requests
vnox.presence.view              — see online status
```

### Custom permission keys (plugin/bot commands)

Plugins register their permissions at load time via the RPC API:

```
mybot.command.greet            — /greet command
mybot.command.play             — /play music
mybot.command.ban_override     — override sub-command
```

**Wildcard support:**
```
vnox.admin.*                   — all admin permissions
vnox.channel.*                 — all channel permissions
mybot.*                        — all commands from mybot
*                              — everything (owner only)
```

### How it works

```
┌──────────────────────────────────────────────────┐
│  Role Editor — "Moderator"                        │
├──────────────────────────────────────────────────┤
│  Permission keys:                                │
│                                                   │
│  ┌─ Built-in ──────────────────────────────┐     │
│  │ ☑ vnox.admin.manage_channels            │     │
│  │ ☑ vnox.admin.manage_members             │     │
│  │ ☐ vnox.admin.manage_roles               │     │
│  │ ☑ vnox.admin.view_audit_log             │     │
│  │ ☑ vnox.channel.*                        │     │
│  └──────────────────────────────────────────┘     │
│                                                   │
│  ┌─ Plugin permissions ────────────────────┐     │
│  │ ☑ mybot.command.greet                   │     │
│  │ ☐ mybot.command.play                    │     │
│  │ ☑ mybot.command.*                       │     │
│  └──────────────────────────────────────────┘     │
│                                                   │
│  ┌─ Custom key ────────────────────────────┐     │
│  │ [ Type a permission key... ]  [ +Add ]  │     │
│  └──────────────────────────────────────────┘     │
│                                                   │
│        [Cancel]  [Save]                           │
└──────────────────────────────────────────────────┘
```

**Resolution order (highest priority wins):**
1. Owner — overrides everything
2. Channel override (user-specific) — allow/deny
3. Channel override (role-specific) — allow/deny
4. Role hierarchy (top role has higher priority)
5. @everyone default
6. Server-wide default

Each override is `(allow_mask, deny_mask)` — deny always beats allow at the same level.

### Database

Already specified in `docs/10-database.md`:

```
roles(id, guild_id, name, color, permissions BITMASK, position)
channel_overrides(id, channel_id, target_id, target_type, allow_mask, deny_mask)
member_roles(member_guild_id, member_user_id, role_id)
```

**Custom permission keys** are stored separately:

```sql
CREATE TABLE permission_keys (
  id TEXT PRIMARY KEY,
  guild_id TEXT NOT NULL,
  name TEXT NOT NULL,           -- e.g. "mybot.command.greet"
  description TEXT,             -- e.g. "Allow using /greet command"
  plugin_id TEXT,               -- which plugin registered it, null = built-in
  created_at TIMESTAMP NOT NULL
);

CREATE UNIQUE INDEX idx_perm_keys_guild_name ON permission_keys(guild_id, name);
```

Role grants use the same `roles.permissions` bitmask for built-in keys (first 128 bits).
For custom/plugin keys beyond 128, they're stored in a separate table:

```sql
CREATE TABLE role_perm_grants (
  role_id TEXT NOT NULL,
  permission_key TEXT NOT NULL,  -- "mybot.command.greet"
  granted INTEGER NOT NULL,      -- 1 = allow, 0 = deny
  PRIMARY KEY (role_id, permission_key),
  FOREIGN KEY(role_id) REFERENCES roles(id)
);
```

---

## Channel Management

### Create Channel

```
Right-click channel list → "Create Channel"
  or
Server Settings → Channels → [+ Add]
```

**Modal:**

```
┌──────────────────────────────────────┐
│  Create Channel                      │
├──────────────────────────────────────┤
│                                      │
│  Name        [_________________]     │
│                                      │
│  Type        ○ Text  ● Voice         │
│                                      │
│  Category    [General ▼]  [None]     │
│                                      │
│  Topic       [_________________]     │
│                                      │
│  ┌─ Permissions ▾ ─────────────┐     │
│  │ @everyone                    │     │
│  │   ☑ View Channel             │     │
│  │   ☑ Send Messages            │     │
│  │   ☐ Voice Connect            │     │
│  │                              │     │
│  │ @admin                       │     │
│  │   ☑ Everything               │     │
│  └──────────────────────────────┘     │
│                                      │
│           [Cancel]     [Create]       │
└──────────────────────────────────────┘
```

### Edit Channel

Double-click channel or right-click → "Edit Channel".

Same modal pre-filled, plus:
- **Name change** — updates slug
- **Category change** — moves channel
- **Drag position** — reorder inside category
- **Delete** — with confirmation (soft delete in DB)

### Drag & Drop

```
Channels                    Server: My Server
─────────────────────────────────────────────
▶ TEXT CHANNELS                           [+]
  ○ general
  ○ announcements                       
▸ VOICE CHANNELS                         [+]
  ○ lobby         ← drag handles ≡
  ○ afk           ← can drag to reorder inside category
                  ← can drag to another category
```

**Behavior:**
- Drag handle `≡` on hover
- Drop zone highlight between channels and around categories
- Hold `Ctrl` to copy permission overrides when moving to another category
- Position stored in `channels.position` column (integer, step by 10 for gaps)
- Changes broadcast via `CHANNEL_UPDATED` packet to all guild members in real-time

---

## Server Settings UI Layout

```
Server Settings: "My Server"
─────────────────────────────────────────────
│  Navigation           │  Content Panel     │
│                       │                    │
│  Overview             │  ┌──────────────┐  │
│  Channels             │  │              │  │
│  Roles                │  │  (dynamic)   │  │
│  Members              │  │              │  │
│  Invites              │  └──────────────┘  │
│  Bans                 │                    │
│  Audit Log            │                    │
│  Plugins              │                    │
│  ───────────────────  │                    │
│  Danger Zone          │                    │
│    Delete Server      │                    │
│    Transfer Owner     │                    │
│                       │                    │
└───────────────────────────────────────────┘
```

### Overview tab

```
┌──────────────────────────────────────────┐
│  Server Overview                          │
│                                          │
│  Name     [My Server________________]    │
│  Description [_______________________]   │
│                                          │
│  Icon     [🔵 Upload image]  drag & drop │
│                                          │
│  Owner:   loki5512344                    │
│  Region:  Warsaw (default)               │
│  Created: 2026-06-06                     │
│                                          │
│  ████████████████████░░  Manage Channels │
│  ████████████░░░░░░░░░░  Manage Roles    │
│                                          │
│              [Save Changes]              │
└──────────────────────────────────────────┘
```

### Plugins tab

```
┌──────────────────────────────────────────┐
│  Plugins                                  │
│                                          │
│  ┌─ Installed ─────────────────────┐     │
│  │ 🎵 MusicBot     v1.2  [Config]  │     │
│  │    [Disable]  [Permissions ▸]   │     │
│  │                                  │     │
│  │ 🤖 ModBot       v0.8  [Config]  │     │
│  │    [Disable]  [Permissions ▸]   │     │
│  └──────────────────────────────────┘     │
│                                          │
│  [Install from URL...]                    │
│                                          │
│  ┌─ Permission Keys ──────────────────┐  │
│  │ Search: [___________________]  🔍  │  │
│  │                                    │  │
│  │ Key                     │ Granted  │  │
│  │ ────────────────────────────────   │  │
│  │ musicbot.command.play  │ admin    │  │
│  │ musicbot.command.skip  │ @everyone│  │
│  │ modbot.warn            │ mod      │  │
│  │ modbot.ban             │ admin    │  │
│  └──────────────────────────────────────┘
└──────────────────────────────────────────┘
```

---

## LNEx Protocol Packets

New packets for admin operations:

```
PID 0x0070  ADMIN_LIST_CHANNELS     client → gateway
PID 0x0071  ADMIN_CREATE_CHANNEL    client → gateway
PID 0x0072  ADMIN_UPDATE_CHANNEL    client → gateway
PID 0x0073  ADMIN_DELETE_CHANNEL    client → gateway
PID 0x0074  CHANNEL_CREATED         gateway → broadcast
PID 0x0075  CHANNEL_UPDATED         gateway → broadcast
PID 0x0076  CHANNEL_DELETED         gateway → broadcast

PID 0x0080  ADMIN_LIST_ROLES        client → gateway
PID 0x0081  ADMIN_CREATE_ROLE       client → gateway
PID 0x0082  ADMIN_UPDATE_ROLE       client → gateway
PID 0x0083  ADMIN_DELETE_ROLE       client → gateway
PID 0x0084  ADMIN_ADD_ROLE_MEMBER   client → gateway
PID 0x0085  ADMIN_REMOVE_ROLE_MEMBER client → gateway

PID 0x0090  ADMIN_KICK_MEMBER       client → gateway
PID 0x0091  ADMIN_BAN_MEMBER        client → gateway
PID 0x0092  ADMIN_UNBAN_MEMBER      client → gateway
PID 0x0093  ADMIN_MUTE_MEMBER       client → gateway

PID 0x00A0  GUILD_UPDATE            client → gateway
PID 0x00A1  GUILD_DELETE            client → gateway (owner only)
PID 0x00A2  GUILD_TRANSFER          client → gateway (owner only)
```

**Examples:**

**ADMIN_CREATE_CHANNEL (0x0071):**
```json
{
  "guild_id": "uuid",
  "category_id": null,
  "type": "voice",
  "name": "lobby",
  "topic": "General voice"
}
```

**CHANNEL_CREATED broadcast (0x0074):**
```json
{
  "channel": {
    "id": "uuid",
    "guild_id": "uuid",
    "category_id": null,
    "type": "voice",
    "name": "lobby",
    "topic": "General voice",
    "position": 3
  }
}
```

**ADMIN_UPDATE_ROLE (0x0082):**
```json
{
  "role_id": "uuid",
  "name": "Moderator",
  "color": 16744960,
  "permissions": {
    "vnox.admin.manage_channels": true,
    "vnox.admin.manage_members": true,
    "mybot.command.greet": true,
    "mybot.command.warn": true
  },
  "hoist": true,
  "mentionable": true
}
```

**ADMIN_UPDATE_CHANNEL (0x0072) — includes position changes via drag & drop:**
```json
{
  "channel_id": "uuid",
  "name": "general",
  "category_id": "uuid",
  "position": 5,
  "topic": "General discussion"
}
```

---

## Permission checks flow

```
Client click "Create Channel"
  │
  ▼
Client sends ADMIN_CREATE_CHANNEL → Gateway
  │
  ▼
Gateway checks:
  1. Is user authenticated?                → 403 if no
  2. Is user in guild?                     → 403 if no
  3. Does user have vnox.admin.manage_channels?
     - Resolve role hierarchy
     - Check channel_overrides (none — new channel)
     - Check wildcards (vnox.admin.*)
     - Owner override                     → 403 if no
  │
  ▼
Gateway: INSERT into channels table
Gateway: INSERT into channel_overrides if specified
Gateway: Broadcast CHANNEL_CREATED to all guild members
Gateway: Log to audit_logs
  │
  ▼
All clients in guild see new channel in channel list
```
