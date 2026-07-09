# Direct Messages — Phase 1.1

## Goal

Allow users to send private messages to each other outside of channels.
DM conversations appear in a dedicated section of the sidebar (like Discord).

## Wire protocol

New packet type in LNEx:

```
DM_START        → client→gateway: "open DM with user X"
DM_MESSAGE      → client→gateway: "send message to DM"
                 gateway→client: "new message in DM"
DM_HISTORY      → client→gateway: "get DM history"
                 gateway→client: DM message list
```

### DM_START payload

```json
{
    "packet_id": "DM_START",
    "target_user_id": "<pubkey of recipient>"
}
```

Gateway response:

```json
{
    "packet_id": "DM_START",
    "dm_id": "dm_<uid1>_<uid2>",
    "other_user": {
        "user_id": "<pubkey>",
        "nickname": "alice"
    },
    "messages": []
}
```

### DM_MESSAGE payload

```json
{
    "packet_id": "DM_MESSAGE",
    "dm_id": "dm_<uid1>_<uid2>",
    "sender_id": "<pubkey>",
    "content": "hello!",
    "timestamp": 1715000000
}
```

## DM ID format

`dm_<user1_hex>_<user2_hex>` where user1 < user2 lexicographically.
This ensures the same DM has the same ID on both sides.

## Database schema

```sql
CREATE TABLE direct_messages (
    id TEXT PRIMARY KEY,                          -- "dm_<uid1>_<uid2>"
    user1_id TEXT NOT NULL,                       -- lexicographically smaller
    user2_id TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    last_message_at INTEGER,
    UNIQUE(user1_id, user2_id)
);

CREATE TABLE dm_messages (
    id TEXT PRIMARY KEY,
    dm_id TEXT NOT NULL REFERENCES direct_messages(id),
    sender_id TEXT NOT NULL,
    body TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_dm_messages_dm_id ON dm_messages(dm_id, created_at);
CREATE INDEX idx_dm_participant ON direct_messages(user1_id, user2_id);
```

## Gateway handler

`gateway/src/handler/direct_message.rs` (new file):

1. `handle_dm_start(session, target_user)`:
   - Find or create DM record (lexicographic user ID ordering)
   - Return DM history

2. `handle_dm_send(session, dm_id, body)`:
   - Validate session is participant of this DM
   - Save to `dm_messages` table
   - If recipient is connected → forward `DM_MESSAGE` to their session
   - If recipient is offline → stored for delivery on reconnect

3. `handle_dm_history(session, dm_id)`:
   - Return last N messages (N = 50 default)

## Client UI

- New section in sidebar: "Direct Messages" with user list
- Click a user → opens DM chat panel (same layout as channel chat)
- Button on user context menu: "Message"
- Unread count badge on DM entries
- New messages trigger notification dot

## Client state

Add to `client/src/ui/state/types.rs`:

```rust
pub struct DmConversation {
    pub dm_id: String,
    pub other_user_id: String,
    pub other_nickname: String,
    pub messages: Vec<ChatMessage>,
    pub unread_count: u32,
}

// Add to UiState:
pub dms: Vec<DmConversation>,
pub active_dm: Option<String>,
```

## Open questions

- Should DMs support group conversations (3+ users)? → Phase 2
- Should DMs be encrypted (E2EE)? → Phase 2
- Should users be able to block DMs from specific users? → Phase 2
