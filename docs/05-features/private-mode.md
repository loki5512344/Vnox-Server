# Private Mode — Phase 1.1

## Goal

Allow running a VNOX server in fully isolated mode.
When private, the server does not discover, connect to, or federate with any other node.

This makes VNOX safe for personal/family use (OwnCord-style),
while keeping the option to go federated later.

## Config

In `dev/config.toml`:

```toml
[federation]
enabled = false              # ← PRIVATE: server is isolated
# enabled = true             # ← FEDERATED: can peer with other nodes
```

Or with explicit mode:

```toml
[server]
mode = "private"             # isolated, no federation
# mode = "federated"         # can discover and peer
```

## Implementation

### Gateway config

```rust
// gateway/src/domain/config.rs
pub struct Config {
    pub server: ServerConfig,
    pub federation: FederationConfig,
}

pub struct ServerConfig {
    pub mode: ServerMode,  // Private | Federated
}

pub enum ServerMode {
    Private,
    Federated,
}

pub struct FederationConfig {
    pub enabled: bool,
    pub known_nodes: Vec<String>,
}
```

### Behavior when private

- Gateway does not send any federation packets
- Gateway does not accept inbound federation connections
- No node discovery (DNS SRV queries skipped)
- No federation port needs to be open
- All features (channels, voice, DMs) work normally, just local-only

### Gate check

```rust
// gateway/src/handler/federation.rs
pub async fn should_sync_to_remotes(config: &Config) -> bool {
    config.federation.enabled
}

// All federation entry points start with:
if !config.federation.enabled {
    return Ok(()); // silently skip
}
```

## Result

| Mode | Behaviour |
|------|-----------|
| `private` | Fully isolated. Like OwnCord. Safe for personal LAN/ home server. |
| `federated` | Can discover, peer, and sync with other VNOX nodes. Like Matrix. |

Default is `private` — opt-in to federation.
