# Custom Emoji, Stickers & GIF

> Target phase: 1.3 (emoji picker) / 2 (custom emoji, stickers, GIF)
> Depends on: Phase 1.2 (guild system, permissions)

---

## Overview

Three related but distinct features:

| Feature | Size | Format | Scope | Storage |
|---------|------|--------|-------|---------|
| **Custom Emoji** | 128×128 max | PNG, GIF, WebP | Per-guild | Server filesystem |
| **Stickers** | 512×512 max | PNG, GIF, WebP, Lottie† | Per-guild + per-user | Server filesystem |
| **GIF Picker** | External | GIF (Tenor/Giphy API) | Per-server config | Proxy via server |

† Lottie — future, not Phase 2.

### Difference from unicode reactions

Unicode emoji reactions (`message_reactions` table, Phase 1.3) use single-codepoint emoji (👍, 🔥, 😂).
Custom emoji extend this with server-hosted images referenced as `:emoji_name:` in messages.

---

## Storage & Schema

### Filesystem layout

```
data/
├── guilds/
│   └── {guild_id}/
│       ├── emojis/
│       │   ├── kappa.png
│       │   ├── pogchamp.gif
│       │   └── ...
│       └── stickers/
│           ├── wave.png
│           └── ...
├── users/
│   └── {user_id}/
│       └── stickers/       # personal stickers
│           └── ...
└── emoji_cache/            # client-side cache (per client, not server)
```

### Database tables

#### guild_emojis

```sql
CREATE TABLE guild_emojis (
  id TEXT PRIMARY KEY,              -- UUID
  guild_id TEXT NOT NULL,           -- FK guilds
  name TEXT NOT NULL,               -- :name: reference (lowercase, alphanumeric + underscore)
  filename TEXT NOT NULL,           -- on-disk filename (id.png)
  content_type TEXT NOT NULL,       -- image/png, image/gif, image/webp
  file_size INTEGER NOT NULL,       -- bytes
  width INTEGER NOT NULL,           -- px
  height INTEGER NOT NULL,          -- px
  is_animated BOOLEAN NOT NULL,     -- true for GIF
  uploaded_by TEXT NOT NULL,        -- FK users
  created_at TIMESTAMP NOT NULL,
  UNIQUE(guild_id, name)
);

CREATE INDEX idx_guild_emojis_guild ON guild_emojis(guild_id);
```

#### guild_stickers

```sql
CREATE TABLE guild_stickers (
  id TEXT PRIMARY KEY,
  guild_id TEXT NOT NULL,
  name TEXT NOT NULL,
  description TEXT,
  filename TEXT NOT NULL,
  content_type TEXT NOT NULL,
  file_size INTEGER NOT NULL,
  width INTEGER NOT NULL,
  height INTEGER NOT NULL,
  uploaded_by TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  UNIQUE(guild_id, name)
);

CREATE INDEX idx_guild_stickers_guild ON guild_stickers(guild_id);
```

#### user_stickers

```sql
CREATE TABLE user_stickers (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  name TEXT NOT NULL,
  filename TEXT NOT NULL,
  content_type TEXT NOT NULL,
  file_size INTEGER NOT NULL,
  width INTEGER NOT NULL,
  height INTEGER NOT NULL,
  created_at TIMESTAMP NOT NULL,
  UNIQUE(user_id, name)
);

CREATE INDEX idx_user_stickers_user ON user_stickers(user_id);
```

### Limits (configurable in server config)

```toml
[limits.emojis]
max_per_guild = 150          # default: 150 static + animated combined
max_static_per_guild = 100
max_animated_per_guild = 50
max_file_size_kb = 256       # per emoji
max_dimension = 128          # px, both width and height

[limits.stickers]
max_per_guild = 50
max_per_user = 25
max_file_size_kb = 512       # per sticker
max_dimension = 512
```

---

## Protocol

### New packet types

```
0x0070  EMOJI_SYNC         server → client, full emoji list for guild
0x0071  EMOJI_ADD           client → server, admin upload
0x0072  EMOJI_DELETE        client → server, admin delete
0x0073  EMOJI_UPDATE        server → client, delta update (single emoji added/removed)

0x0074  STICKER_SYNC        server → client, full sticker list
0x0075  STICKER_SEND        client → server, send sticker in channel
0x0076  STICKER_UPLOAD      client → server, upload sticker
0x0077  STICKER_DELETE      client → server, delete sticker

0x0078  EMOJI_DATA          client → server, request emoji image bytes
0x0079  EMOJI_DATA_RESP     server → client, image bytes (lazy-load, not in sync)

0x0080  GIF_SEARCH          client → server, proxy search to Tenor/Giphy
0x0081  GIF_TRENDING        client → server, trending GIFs
```

### EMOJI_SYNC (0x0070)

Server sends on guild join. Client caches locally.

```json
{
  "guild_id": "uuid",
  "emojis": [
    {
      "id": "uuid",
      "name": "kappa",
      "content_type": "image/png",
      "is_animated": false,
      "width": 128,
      "height": 128,
      "file_size": 4096
    }
  ]
}
```

Image bytes are NOT included — client lazy-loads via `EMOJI_DATA` + `EMOJI_DATA_RESP`.

### EMOJI_ADD (0x0071)

Admin uploads new emoji. Requires `MANAGE_EMOJIS` permission.

```json
{
  "guild_id": "uuid",
  "name": "pogchamp",
  "data": "<base64>"
}
```

Server validates:
- Name: lowercase alphanumeric + underscore, 2-32 chars
- Image: PNG/GIF/WebP, max 256KB, max 128×128
- Count: under guild limit

Response: new emoji object (same shape as in EMOJI_SYNC), server broadcasts `EMOJI_UPDATE` to guild.

### EMOJI_DELETE (0x0072)

```json
{
  "guild_id": "uuid",
  "emoji_id": "uuid"
}
```

Server removes file + DB row, broadcasts `EMOJI_UPDATE` with `deleted: true`.

### EMOJI_DATA / EMOJI_DATA_RESP (0x0078/0x0079)

Lazy-load pattern — client requests image bytes when first rendering an emoji.

Request:
```json
{
  "emoji_id": "uuid"
}
```

Response:
```json
{
  "emoji_id": "uuid",
  "content_type": "image/png",
  "data": "<base64>"
}
```

Client caches image bytes on disk in `emoji_cache/{emoji_id}.{ext}`.

### STICKER_SEND (0x0075)

Stickers are sent as a separate message type (not inline). The sticker reference is embedded in a chat message.

```json
{
  "channel_id": "uuid",
  "sticker_id": "uuid",
  "sticker_type": "guild"  // "guild" | "user"
}
```

Gateway creates a message with `content_type: "sticker"` and `content: {sticker_id, sticker_type}`.

### GIF_SEARCH (0x0080)

Client requests GIF search via server (server proxies to Tenor/Giphy so API key stays server-side).

```json
{
  "query": "cat dancing",
  "limit": 20,
  "offset": 0
}
```

Response:
```json
{
  "results": [
    {
      "id": "tenor_12345",
      "url": "https://media.tenor.com/...",
      "preview_url": "https://media.tenor.com/.../tiny.gif",
      "width": 498,
      "height": 280,
      "title": "Cat Dancing"
    }
  ]
}
```

GIF config in server `config.toml`:

```toml
[gif]
enabled = true
provider = "tenor"           # "tenor" | "giphy"
api_key = "TENOR_API_KEY"
max_results = 50
```

When `enabled = false` the GIF picker is hidden in all clients.

---

## Client Integration

### Message format with custom emoji

Messages reference custom emoji as `<:emoji_name:emoji_id>` inline in text content:

```
User: check out this <:kappa:abc123> and <:pog:def456>
```

Client renders by looking up emoji_id in local cache, falling back to lazy-load via `EMOJI_DATA_RESP`.

Unicode emoji (`👍`) render as-is via system font.

### Emoji picker

New widget: `client/src/ui/chat/emoji_picker.rs`

```
┌─────────────────────────────┐
│ [Search emoji...        🔍] │
│─────────────────────────────│
│ Favorites  │ Custom  │ GIF  │  ← tabs
│────────────┴─────────┴──────│
│ 👍 😂 🔥 💯 🙏 😎          │
│ 🎉 ✨ 😭 💀 👀 🫡          │
│ 😈 🐱 💪 🚀 🫶 🔒          │
│─────────────────────────────│
│ ┌───┐ ┌───┐ ┌───┐ ┌───┐   │
│ │Kappa│ │Pog │ │Hmm │ │Clap│  │  ← custom (rendered as images)
│ └───┘ └───┘ └───┘ └───┘   │
└─────────────────────────────┘
```

### Sticker panel

New widget: `client/src/ui/chat/sticker_panel.rs`

- Grid of sticker thumbnails
- Tabs: Guild stickers / My stickers
- Click to send as sticker message (not inline)
- Send button shows sticker preview before sending

### GIF picker

New widget: `client/src/ui/chat/gif_picker.rs`

- Search bar at top
- Trending grid when empty query
- Results grid with animated previews
- Click to send GIF as embedded image in chat message

### Local cache

```
%APPDATA%/vnox/cache/
├── emojis/{emoji_id}.{ext}     # custom emoji images
├── stickers/{sticker_id}.{ext}  # sticker images
└── gifs/{gif_id}.gif           # recently used GIFs (LRU, max 50)
```

Cache invalidation: client re-requests on version mismatch (server increments `emoji_version` per guild).

---

## Admin UX

### Permissions

| Permission        | Bit | Description                 |
| ----------------- | --- | --------------------------- |
| MANAGE_EMOJIS     | 12  | Upload/delete guild emojis  |
| MANAGE_STICKERS   | 13  | Upload/delete guild stickers|

These fit into the existing u128 permission bitmask (bits 12-13 are available).

### Emoji management panel

Accessible from guild settings (admin only):

```
┌──────────────────────────────────────────────────┐
│ Guild Emojis                          [Upload]   │
│──────────────────────────────────────────────────│
│ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌────────┐│
│ │Kappa │ │ Pog  │ │ Hmm  │ │ Clap │ │ (free) ││
│ │ 4KB  │ │ 12KB │ │ 8KB  │ │ 16KB │ │        ││
│ │ [✕]  │ │ [✕]  │ │ [✕]  │ │ [✕]  │ │        ││
│ └──────┘ └──────┘ └──────┘ └──────┘ └────────┘│
│──────────────────────────────────────────────────│
│ 4/150 emojis used                    [Save order]│
└──────────────────────────────────────────────────┘
```

Upload flow:
1. Admin clicks Upload → file dialog (PNG/GIF/WebP)
2. Client validates: ≤256KB, ≤128×128, valid format
3. Client sends `EMOJI_ADD` with base64 data
4. Server validates + stores + broadcasts `EMOJI_UPDATE`

### Sticker management

Same pattern as emoji management, separate tab in guild settings.

Personal stickers managed in user settings (no admin required).

---

## Federation Considerations (Phase 3)

### Emoji visibility across nodes

When two nodes federate:
1. Each node maintains its own emoji set
2. Guild emoji metadata synced via federation protocol
3. Image bytes fetched on-demand (lazy, same as client)
4. Reference format includes origin node: `<:emoji_name:emoji_id@node>`

### Cross-node sticker send

1. Sticker reference sent as `{sticker_id, origin_node}`
2. Receiving node fetches sticker metadata + image from origin node
3. Caches locally for subsequent renders

### GIF proxy

GIF search always goes through the user's home node.
The server proxies to Tenor/Giphy — no federation needed for GIF results (they're just URLs).

---

## Migration Path

### Phase 1.3 — Unicode reactions only
- `message_reactions` table (already designed)
- Unicode emoji picker in client
- No custom emoji, no stickers, no GIF

### Phase 2 — Custom emoji
- Add `guild_emojis` table
- Add packet types 0x0070–0x0073, 0x0078–0x0079
- Client emoji picker with custom tab
- Admin upload/delete UI
- No federation support yet

### Phase 2 — Stickers
- Add `guild_stickers` + `user_stickers` tables
- Add packet types 0x0074–0x0077
- Sticker panel in client
- New message content_type: "sticker"

### Phase 2 — GIF picker
- Add Tenor/Giphy config to gateway
- Add packet types 0x0080–0x0081
- GIF picker widget in client
- Server-side proxy (API key protection)

### Phase 3 — Federation
- Cross-node emoji/sticker sync
- `@node` suffix in references
