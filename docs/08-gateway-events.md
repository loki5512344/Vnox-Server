# Gateway Events System

Version: 1.0  
Status: Specification for Phase 1.2+

---

## Overview

The Gateway Events system is the real-time synchronization layer for Vnox. All state changes (messages, user status, channel creation, etc.) are broadcast as events to interested clients.

Events follow a standard structure and allow clients to:
- Stay synchronized without polling
- React to user actions in real-time
- Build responsive UIs
- Implement typing indicators and read receipts

---

## Event Structure

### Standard Packet Format

```json
{
  "op": 0,
  "event": "MESSAGE_CREATE",
  "data": {
    "message_id": "...",
    "channel_id": "...",
    "sender_id": "...",
    "content": "...",
    "timestamp": "..."
  }
}
```

### Fields

| Field | Type   | Description                          |
| ----- | ------ | ------------------------------------ |
| op    | u32    | Opcode (0=event, 1=command response) |
| event | String | Event type name                      |
| data  | Object | Event-specific payload               |

---

## Message Events

Events related to text messages in channels.

### MESSAGE_CREATE

Sent when a message is posted.

**Payload:**
```json
{
  "message_id": "uuid",
  "channel_id": "uuid",
  "sender_id": "uuid",
  "sender_name": "string",
  "content": "string",
  "timestamp": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** Everyone with `VIEW_CHANNEL` permission

---

### MESSAGE_UPDATE

Sent when a message is edited.

**Payload:**
```json
{
  "message_id": "uuid",
  "channel_id": "uuid",
  "content": "string (new content)",
  "edited_at": "2026-06-05T12:35:00Z"
}
```

**Broadcast:** Everyone with `VIEW_CHANNEL` permission

---

### MESSAGE_DELETE

Sent when a message is deleted.

**Payload:**
```json
{
  "message_id": "uuid",
  "channel_id": "uuid",
  "deleted_at": "2026-06-05T12:36:00Z"
}
```

**Broadcast:** Everyone with `VIEW_CHANNEL` permission

---

### MESSAGE_REACTION_ADD

Sent when a user adds an emoji reaction.

**Payload:**
```json
{
  "message_id": "uuid",
  "channel_id": "uuid",
  "reactor_id": "uuid",
  "emoji": "👍"
}
```

**Broadcast:** Everyone with `VIEW_CHANNEL` permission

---

### MESSAGE_REACTION_REMOVE

Sent when a user removes an emoji reaction.

**Payload:**
```json
{
  "message_id": "uuid",
  "channel_id": "uuid",
  "reactor_id": "uuid",
  "emoji": "👍"
}
```

**Broadcast:** Everyone with `VIEW_CHANNEL` permission

---

## Typing Events

### TYPING_START

Sent when a user begins typing.

**Payload:**
```json
{
  "channel_id": "uuid",
  "user_id": "uuid",
  "user_name": "string",
  "timestamp": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** Everyone with `VIEW_CHANNEL` permission

**Note:** Sender must stop sending after 5-10 seconds of inactivity.

---

### TYPING_STOP

Sent when a user stops typing.

**Payload:**
```json
{
  "channel_id": "uuid",
  "user_id": "uuid"
}
```

**Broadcast:** Everyone with `VIEW_CHANNEL` permission

---

## Read Receipt Events

### MESSAGE_ACK

Sent when a user reads up to a certain message.

**Payload:**
```json
{
  "channel_id": "uuid",
  "user_id": "uuid",
  "message_id": "uuid (last read)",
  "timestamp": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** Sender and channel members

---

## Guild Events

### GUILD_CREATE

Sent when a new guild is created.

**Payload:**
```json
{
  "guild_id": "uuid",
  "owner_id": "uuid",
  "name": "string",
  "description": "string",
  "icon": "asset_id (nullable)",
  "created_at": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** All guild members on login

---

### GUILD_UPDATE

Sent when guild settings change.

**Payload:**
```json
{
  "guild_id": "uuid",
  "changes": {
    "name": "new name",
    "description": "new description",
    "icon": "asset_id (nullable)"
  },
  "updated_at": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** All guild members

---

### GUILD_DELETE

Sent when a guild is deleted.

**Payload:**
```json
{
  "guild_id": "uuid",
  "deleted_at": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** All former members

---

## Category Events

### CATEGORY_CREATE

Sent when a category is created.

**Payload:**
```json
{
  "category_id": "uuid",
  "guild_id": "uuid",
  "name": "string",
  "position": 0,
  "created_at": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** All members with `VIEW_CHANNEL` permission

---

### CATEGORY_UPDATE

Sent when category properties change.

**Payload:**
```json
{
  "category_id": "uuid",
  "guild_id": "uuid",
  "changes": {
    "name": "new name",
    "position": 1
  },
  "updated_at": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** All members with `VIEW_CHANNEL` permission

---

### CATEGORY_DELETE

Sent when a category is deleted.

**Payload:**
```json
{
  "category_id": "uuid",
  "guild_id": "uuid",
  "deleted_at": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** All members with `VIEW_CHANNEL` permission

---

## Channel Events

### CHANNEL_CREATE

Sent when a channel is created.

**Payload:**
```json
{
  "channel_id": "uuid",
  "guild_id": "uuid",
  "category_id": "uuid (nullable)",
  "type": "TEXT | VOICE | ANNOUNCEMENT | STAGE",
  "name": "string",
  "topic": "string (nullable)",
  "position": 0,
  "created_at": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** All members with `VIEW_CHANNEL` permission

---

### CHANNEL_UPDATE

Sent when channel properties change.

**Payload:**
```json
{
  "channel_id": "uuid",
  "guild_id": "uuid",
  "changes": {
    "name": "new name",
    "topic": "new topic",
    "position": 2
  },
  "updated_at": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** All members with `VIEW_CHANNEL` permission

---

### CHANNEL_DELETE

Sent when a channel is deleted.

**Payload:**
```json
{
  "channel_id": "uuid",
  "guild_id": "uuid",
  "deleted_at": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** All members with `VIEW_CHANNEL` permission

---

## Role Events

### ROLE_CREATE

Sent when a new role is created.

**Payload:**
```json
{
  "role_id": "uuid",
  "guild_id": "uuid",
  "name": "string",
  "color": 16711680,
  "permissions": 0,
  "position": 5,
  "hoist": false,
  "mentionable": true,
  "created_at": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** All members with `MANAGE_ROLES` permission

---

### ROLE_UPDATE

Sent when role settings change.

**Payload:**
```json
{
  "role_id": "uuid",
  "guild_id": "uuid",
  "changes": {
    "name": "new name",
    "color": 16711680,
    "permissions": 1024,
    "position": 6
  },
  "updated_at": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** All members with `MANAGE_ROLES` permission

---

### ROLE_DELETE

Sent when a role is deleted.

**Payload:**
```json
{
  "role_id": "uuid",
  "guild_id": "uuid",
  "deleted_at": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** All members with `MANAGE_ROLES` permission

---

## Invite Events

### INVITE_CREATE

Sent when a new invite is generated.

**Payload:**
```json
{
  "invite_id": "uuid",
  "guild_id": "uuid",
  "code": "string",
  "creator_id": "uuid",
  "expires_at": "2026-06-10T12:34:56Z (nullable)",
  "max_uses": 10,
  "uses": 0,
  "created_at": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** All members with `MANAGE_INVITES` permission

---

### INVITE_DELETE

Sent when an invite is revoked.

**Payload:**
```json
{
  "invite_id": "uuid",
  "guild_id": "uuid",
  "code": "string",
  "deleted_at": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** All members with `MANAGE_INVITES` permission

---

## Member Events

### MEMBER_JOIN

Sent when a user joins a guild.

**Payload:**
```json
{
  "guild_id": "uuid",
  "user_id": "uuid",
  "user_name": "string",
  "nickname": "string (nullable)",
  "joined_at": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** All guild members

---

### MEMBER_UPDATE

Sent when member data changes (roles, nickname, timeout).

**Payload:**
```json
{
  "guild_id": "uuid",
  "user_id": "uuid",
  "changes": {
    "nickname": "new nickname",
    "roles": ["role_id_1", "role_id_2"],
    "timeout_until": "2026-06-05T13:34:56Z (nullable)"
  },
  "updated_at": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** All guild members with `MANAGE_MEMBERS` permission

---

### MEMBER_LEAVE

Sent when a user leaves a guild.

**Payload:**
```json
{
  "guild_id": "uuid",
  "user_id": "uuid",
  "left_at": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** All remaining guild members

---

## Voice Events

### VOICE_STATE_UPDATE

Sent when a user's voice state changes (connect, disconnect, mute, deafen).

**Payload:**
```json
{
  "user_id": "uuid",
  "user_name": "string",
  "guild_id": "uuid",
  "channel_id": "uuid (nullable, null = disconnect)",
  "muted": false,
  "deafened": false,
  "self_muted": false,
  "self_deafened": false,
  "speaking": false,
  "volume_level": 0.8,
  "timestamp": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** All members of the voice channel and moderators

---

## Presence Events

### PRESENCE_UPDATE

Sent when a user's presence (online status, activity) changes.

**Payload:**
```json
{
  "user_id": "uuid",
  "status": "ONLINE | IDLE | DO_NOT_DISTURB | OFFLINE | INVISIBLE",
  "activity_type": "PLAYING | LISTENING | WATCHING | STREAMING | CUSTOM (nullable)",
  "activity_text": "string (nullable)",
  "last_seen": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** All connected clients of mutual friends/guild members

---

## Direct Message Events

### DM_CREATE

Sent when a new DM channel is opened.

**Payload:**
```json
{
  "dm_id": "uuid",
  "user_id": "uuid",
  "recipient_id": "uuid",
  "recipient_name": "string",
  "created_at": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** Both participants

---

### DM_MESSAGE

Sent when a direct message is received.

**Payload:**
```json
{
  "dm_id": "uuid",
  "message_id": "uuid",
  "sender_id": "uuid",
  "content": "string",
  "timestamp": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** Both participants

---

### DM_TYPING

Sent when a user types in a DM.

**Payload:**
```json
{
  "dm_id": "uuid",
  "user_id": "uuid",
  "timestamp": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** Recipient only

---

## Friend Events

### FRIEND_REQUEST

Sent when a friend request is received.

**Payload:**
```json
{
  "requester_id": "uuid",
  "requester_name": "string",
  "timestamp": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** Recipient only

---

### FRIEND_ACCEPT

Sent when a friend request is accepted.

**Payload:**
```json
{
  "friend_id": "uuid",
  "friend_name": "string",
  "timestamp": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** Both parties

---

### FRIEND_REMOVE

Sent when a friendship is ended.

**Payload:**
```json
{
  "friend_id": "uuid",
  "timestamp": "2026-06-05T12:34:56Z"
}
```

**Broadcast:** Both parties

---

## Implementation Notes

### Permission Checks

The gateway should verify permissions before broadcasting:

```
if event_requires_permission:
  for client in subscribers:
    if client.has_permission(event.required_permission):
      send(client, event)
```

### Ordering Guarantees

Events within a single session are in causal order:
- Messages from the same user appear in send order
- Role changes before member updates using that role

### Event Deduplication

Clients should handle duplicate events gracefully. Use `message_id` or unique event identifiers to deduplicate.

### Backpressure

If a client falls behind, the gateway should:
1. Queue up to N events in memory
2. If queue exceeds N, force reconnect (replay full state)
3. Implement exponential backoff for slow clients

---

## Design Goals

✓ Real-time synchronization without polling  
✓ Bandwidth efficient (only changed state)  
✓ Permissible (respect channel and role visibility)  
✓ Ordered causally (maintain consistency)  
✓ Extensible (easy to add new event types)  
✓ Compatible with federation (events can be bridged)
