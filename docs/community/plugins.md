# Plugins

VNOX has a plugin system for extending server-side behavior.
Plugins run on the node, not on the client.

---

## Supported languages

- TypeScript
- JavaScript

---

## Runtime

**Deno** — chosen as the plugin runtime.

Reasons:
- TypeScript native, no transpile step for plugin authors
- built-in permissions model — plugins explicitly declare what they need
- active ecosystem, good Rust embedding via `deno_core`

---

## API

Plugins communicate with the gateway via **WebSocket RPC**.
The gateway exposes a local WebSocket endpoint that plugins connect to.

```
Plugin process
    │ WebSocket (localhost)
    ▼
Gateway plugin API (localhost:7800)
```

### RPC format

```json
{
  "id": "req-1",
  "method": "channel.send_message",
  "params": {
    "channel_id": "general",
    "content": "Hello from plugin"
  }
}
```

Response:

```json
{
  "id": "req-1",
  "result": { "message_id": "abc123" }
}
```

Errors:

```json
{
  "id": "req-1",
  "error": { "code": 403, "message": "permission denied" }
}
```

---

## Available methods

### Events (subscribe)

```typescript
// Subscribe to all events
ws.on('message', (event) => {
  const e = JSON.parse(event);
  // e.event, e.data
});
```

| Event | Payload |
|-------|---------|
| `user.join` | `{ user_id, channel_id }` |
| `user.leave` | `{ user_id, channel_id }` |
| `message.created` | `{ message_id, channel_id, sender_id, content }` |
| `voice.speaking` | `{ user_id, channel_id, state }` |
| `user.muted` | `{ user_id }` |

### Commands

| Method | Description |
|--------|-------------|
| `channel.send_message` | Send a message to a channel |
| `channel.list` | List all channels |
| `user.kick` | Kick a user from a channel |
| `user.mute` | Mute a user (server-side) |
| `user.ban` | Ban a user by pubkey |
| `user.get` | Get user info by pubkey |
| `node.get_stats` | Get node statistics |

---

## Plugin manifest

Each plugin is a directory with a `plugin.json`:

```json
{
  "name": "relay-switcher",
  "version": "0.1.2",
  "author": "raven",
  "description": "Auto-selects the nearest relay node",
  "main": "index.ts",
  "permissions": [
    "node.stats",
    "channel.read"
  ]
}
```

Permissions are declared in the manifest and granted by the node admin.
A plugin that requests permissions not granted to it will receive `403` on those methods.

---

## Example plugin

A simple moderation bot that deletes messages containing a banned word:

```typescript
// index.ts

const ws = new WebSocket("ws://localhost:7800/plugins");
const BANNED = ["badword"];

ws.addEventListener("message", async (event) => {
  const e = JSON.parse(event.data);

  if (e.event === "message.created") {
    const { message_id, channel_id, content } = e.data;
    const lower = content.toLowerCase();

    if (BANNED.some(word => lower.includes(word))) {
      await rpc("message.delete", { message_id, channel_id });
      console.log(`Deleted message ${message_id}`);
    }
  }
});

function rpc(method: string, params: object): Promise<any> {
  return new Promise((resolve) => {
    const id = crypto.randomUUID();
    ws.send(JSON.stringify({ id, method, params }));
    ws.addEventListener("message", function handler(e) {
      const res = JSON.parse(e.data);
      if (res.id === id) {
        ws.removeEventListener("message", handler);
        resolve(res.result);
      }
    });
  });
}
```

---

## Installing a plugin

```bash
# Copy plugin directory to node's plugin folder
cp -r my-plugin/ /var/lib/vnox/plugins/

# Restart gateway (or use hot-reload if supported)
systemctl restart vnox-gateway
```

Or via the client: Settings → Plugins → Install → select directory.

---

## Plugin marketplace

A community plugin registry is planned for Phase 3.
Plugins will be installable directly from the client.

Until then, plugins are distributed as source code repositories.
