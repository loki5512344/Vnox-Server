# Community Model

Version: 1.0  
Status: Specification for Phase 1.2

---

## Overview

The Community Model defines the core entities of the Vnox social platform. Communities (called "Guilds" in the spec, analogous to Discord servers or Slack workspaces) are the primary organizational unit where users can communicate through channels, voice calls, and direct messages.

### Entity Hierarchy

```
Guild
├── Categories
│   └── Channels (TEXT, VOICE, ANNOUNCEMENT, STAGE)
├── Members
├── Roles
├── Invites
└── Settings
```

Each entity has:
- Unique identifier (UUID)
- Lifecycle events (CREATE, UPDATE, DELETE)
- Associated Gateway events
- Audit trail in moderator actions

---

## Guild

A Guild represents an independent community of users.

### Analogues
- Discord Server
- Slack Workspace
- Matrix Space
- TeamSpeak Server

### Responsibilities
- Store and manage members
- Store and manage roles
- Store and manage channels and categories
- Manage invitations
- Enforce community settings

### Fields

| Field       | Type      | Description              |
| ----------- | --------- | ------------------------ |
| id          | UUID      | Unique identifier        |
| owner_id    | UUID      | Guild owner (immutable)  |
| name        | String    | Display name             |
| description | String    | Guild description        |
| icon        | AssetId   | Guild icon (optional)    |
| banner      | AssetId   | Guild banner (optional)  |
| created_at  | Timestamp | Creation time            |
| updated_at  | Timestamp | Last modification time   |

### Limits

Recommended thresholds:
- 500 Categories per guild
- 5,000 Channels per guild
- 250 Roles per guild
- 100,000+ Members per guild

---

## Category

Categories group channels for organization and permission inheritance.

### Responsibilities
- Organize channels visually
- Inherit and override permissions
- Provide structural organization for the guild

### Fields

| Field      | Type      | Description              |
| ---------- | --------- | ------------------------ |
| id         | UUID      | Unique identifier        |
| guild_id   | UUID      | Parent guild             |
| name       | String    | Display name             |
| position   | Integer   | Sort order (ascending)   |
| created_at | Timestamp | Creation time            |

### Example Structure

```
Development
├── backend
├── frontend
└── infrastructure

Community
├── general
├── memes
└── voice
```

---

## Channel

Channels are the primary communication unit within a guild.

### Channel Types

| Type         | Purpose                                          |
| ------------ | ------------------------------------------------ |
| TEXT         | Standard text messages with history             |
| VOICE        | Voice communication with connected members      |
| ANNOUNCEMENT | Read-only broadcasts from moderators            |
| STAGE        | One-way speaker podiums with audience           |
| DM           | Private 1:1 conversation (no guild)             |
| GROUP_DM     | Private group conversation (no guild)           |

### Fields

| Field       | Type      | Description              |
| ----------- | --------- | ------------------------ |
| id          | UUID      | Unique identifier        |
| guild_id    | UUID      | Parent guild             |
| category_id | UUID      | Parent category (nullable) |
| type        | Enum      | Channel type             |
| name        | String    | Display name             |
| topic       | String    | Channel description      |
| position    | Integer   | Sort order (ascending)   |
| created_at  | Timestamp | Creation time            |

### Permissions

Each channel can define permission overrides for roles and users.

Example:

```
Role: Member

VIEW_CHANNEL: ALLOW
SEND_MESSAGES: DENY
```

---

## Role

Roles represent groups of permissions that can be assigned to members.

### Characteristics
- Members can have multiple roles
- Roles inherit permissions additively
- Roles have a display color
- Roles have a hierarchical position

### Fields

| Field       | Type    | Description              |
| ----------- | ------- | ------------------------ |
| id          | UUID    | Unique identifier        |
| guild_id    | UUID    | Parent guild             |
| name        | String  | Display name             |
| color       | Integer | RGB color (0x000000-0xFFFFFF) |
| permissions | u128    | Permission bitmask       |
| position    | Integer | Role hierarchy (higher = more power) |
| hoist       | Boolean | Display separately in member list |
| mentionable | Boolean | Can be mentioned with @role |

### Default Roles

Every guild starts with these built-in roles:

| Role          | Permissions          |
| ------------- | -------------------- |
| Owner         | All permissions      |
| Administrator | All except ownership transfer |
| Moderator     | Message management, member moderation |
| Member        | Standard member permissions |
| Guest         | Limited permissions  |
| Bot           | API permissions for bots |

---

## Permission System

Permissions control which actions a user can perform. They are stored as a 128-bit bitmask.

### Guild Permissions

| Permission           | Bit | Description                    |
| -------------------- | --- | ------------------------------ |
| VIEW_CHANNEL         | 0   | See channels and categories    |
| SEND_MESSAGES        | 1   | Send messages in text channels |
| EMBED_LINKS          | 2   | Send URL embeds                |
| ATTACH_FILES         | 3   | Upload files                   |
| MENTION_EVERYONE     | 4   | Use @everyone and @here        |
| MANAGE_MESSAGES      | 5   | Delete/edit others' messages   |
| MANAGE_CHANNELS      | 6   | Create/delete channels         |
| MANAGE_ROLES         | 7   | Manage roles                   |
| MANAGE_GUILD         | 8   | Edit guild settings            |
| CREATE_INVITE        | 9   | Create invitations             |
| VIEW_AUDIT_LOG       | 10  | View audit log                 |

### Voice Permissions

| Permission           | Bit | Description                    |
| -------------------- | --- | ------------------------------ |
| CONNECT              | 16  | Join voice channels            |
| SPEAK                | 17  | Transmit audio                 |
| STREAM               | 18  | Share screen/camera            |
| PRIORITY_SPEAKER     | 19  | Auto unmute when speaking      |
| MUTE_MEMBERS         | 20  | Mute other users               |
| DEAFEN_MEMBERS       | 21  | Deafen other users             |
| MOVE_MEMBERS         | 22  | Move users between channels    |

### Administrative Permissions

| Permission           | Bit | Description                    |
| -------------------- | --- | ------------------------------ |
| ADMINISTRATOR        | 30  | All permissions                |
| OWNER                | 31  | Guild ownership (unique)       |
| BYPASS_CHECKS        | 32  | Bypass permission checks       |

### Permission Resolution Order

When determining if a user can perform an action:

1. **Guild Owner** → Always allowed
2. **ADMINISTRATOR flag** → All permissions granted
3. **Role Permissions** → Sum all roles' permissions
4. **Channel Overrides** → Role-specific channel overrides
5. **User Overrides** → User-specific channel overrides (highest priority)

Last rule wins (most specific override takes precedence).

---

## Invite

Invites allow users to join guilds.

### Types

| Type        | Description                        |
| ----------- | ---------------------------------- |
| Permanent   | Never expires, unlimited uses      |
| Temporary   | Expires after time or usage limit |

### Fields

| Field      | Type      | Description              |
| ---------- | --------- | ------------------------ |
| id         | UUID      | Unique identifier        |
| guild_id   | UUID      | Target guild             |
| creator_id | UUID      | User who created invite  |
| code       | String    | Invite code (slug)       |
| expires_at | Timestamp | Expiration (nullable)    |
| max_uses   | Integer   | Use limit (nullable)     |
| uses       | Integer   | Current use count        |

### Example Codes

```
vnox.gg/dev
vnox.gg/community
vnox.gg/events-2026
```

---

## Member

A Member represents a user within a specific guild. Important: User and Member are separate entities.

- **User**: Global identity across all nodes
- **Member**: User's role and status within a specific guild

### Fields

| Field         | Type          | Description              |
| ------------- | ------------- | ------------------------ |
| guild_id      | UUID          | Parent guild             |
| user_id       | UUID          | User identity            |
| nickname      | String        | Guild-specific nickname  |
| joined_at     | Timestamp     | Join time                |
| roles         | Array<UUID>   | Assigned role IDs        |
| timeout_until | Timestamp     | Moderation timeout (nullable) |

### Responsibilities

- Track guild membership
- Store assigned roles
- Maintain guild-specific nickname
- Enforce moderation state (timeouts)

---

## Presence

Presence describes a user's current online status and activity. Presence is **global** (not per-guild).

### Status Types

| Status             | Description                          |
| ------------------ | ------------------------------------ |
| ONLINE             | Active and available                 |
| IDLE               | Away but still connected             |
| DO_NOT_DISTURB     | Online but do not send notifications |
| OFFLINE            | Not connected                        |
| INVISIBLE          | Appears offline to others            |

### Activity Types

| Activity | Example                  |
| -------- | ------------------------ |
| PLAYING  | "Rust" or "Minecraft"    |
| LISTENING | "Spotify" or "Podcast"  |
| WATCHING | "Live Stream" or "Movie" |
| STREAMING | "Twitch" or "YouTube"   |
| CUSTOM   | User-defined text        |

### Fields

| Field         | Type      | Description              |
| ------------- | --------- | ------------------------ |
| user_id       | UUID      | User identity            |
| status        | Enum      | Online status            |
| activity_type | Enum      | Current activity type    |
| activity_text | String    | Activity description     |
| last_seen     | Timestamp | Last activity time       |

### Examples

```
Status: ONLINE
Activity: Playing Rust

Status: IDLE
Activity: Listening to Spotify

Status: DO_NOT_DISTURB
Custom: Building Vnox
```

---

## Entity Relationships

```
Guild (1) ──── (N) Category
Guild (1) ──── (N) Channel
Guild (1) ──── (N) Role
Guild (1) ──── (N) Invite
Guild (1) ──── (N) Member

User (1) ──── (N) Presence
User (1) ──── (N) Member
```

---

## Gateway Events

The Gateway broadcasts events when community entities change.

### Guild Events

| Event         | Trigger                  |
| ------------- | ------------------------ |
| GUILD_CREATE  | Guild created            |
| GUILD_UPDATE  | Guild settings changed   |
| GUILD_DELETE  | Guild deleted            |

### Category Events

| Event             | Trigger                  |
| ----------------- | ------------------------ |
| CATEGORY_CREATE   | Category created         |
| CATEGORY_UPDATE   | Category settings changed |
| CATEGORY_DELETE   | Category deleted         |

### Channel Events

| Event           | Trigger                  |
| --------------- | ------------------------ |
| CHANNEL_CREATE  | Channel created          |
| CHANNEL_UPDATE  | Channel settings changed |
| CHANNEL_DELETE  | Channel deleted          |

### Role Events

| Event         | Trigger                  |
| ------------- | ------------------------ |
| ROLE_CREATE   | Role created             |
| ROLE_UPDATE   | Role modified            |
| ROLE_DELETE   | Role deleted             |

### Invite Events

| Event          | Trigger                  |
| -------------- | ------------------------ |
| INVITE_CREATE  | Invite generated         |
| INVITE_DELETE  | Invite revoked           |

### Member Events

| Event          | Trigger                  |
| -------------- | ------------------------ |
| MEMBER_JOIN    | User joined guild        |
| MEMBER_UPDATE  | Member data changed      |
| MEMBER_LEAVE   | User left guild          |

### Presence Events

| Event             | Trigger                  |
| ----------------- | ------------------------ |
| PRESENCE_UPDATE   | Status or activity changed |

---

## Design Goals

✓ Discord-like usability and familiarity  
✓ Scalable architecture (thousands of members)  
✓ Federation compatibility (Phase 3+)  
✓ Plugin compatibility (Phase 2+)  
✓ Self-hosting support  
✓ Efficient permission resolution (cached, bitwise)  
✓ Low memory overhead  
✓ Gateway-first synchronization (eventual consistency)

---

## Future Extensions

Planned for later phases:

- Forum Channels
- Threads (message branching)
- Scheduled Events
- Voice Regions (automatic routing)
- Guild Templates (one-click setup)
- Role Icons
- Verification Levels
- Community Discovery
- Guild Analytics
