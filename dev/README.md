# Local development configuration

This directory holds the default config for running VNOX on your machine.

## File: `config.toml`

```toml
[node]
name    = "dev-node"
address = "127.0.0.1"

[gateway]
bind            = "127.0.0.1:7600"
max_connections = 100
session_timeout = 600

[voice]
bind = "127.0.0.1:7700"

[storage]
data_dir    = "./dev/data"
backend     = "sqlite"
sqlite_path = "./dev/data/vnox.db"
```

### Sections

**`[node]`**

- `name` - shown to clients after connect (HELLO / session UI).
- `address` - public node address for future federation; local dev uses loopback.

**`[gateway]`**

- `bind` - TCP listen address for client connections. Default dev port: `7600`.
- `max_connections` - connection cap (not enforced in all code paths yet).
- `session_timeout` - session lifetime in seconds; sent to clients as `expires_at`.

**`[voice]`**

- `bind` - UDP listen address for voice relay. Default dev port: `7700`.
- Sent to clients as `voice_endpoint` when they join a channel.

**`[storage]`**

- `data_dir` - gateway data directory; stores SQLite DB and `server_identity.json`.
- `sqlite_path` - SQLite database file for chat history and user records.

## First run

```sh
mkdir -p dev/data

cargo run -p vnox-gateway -- --config dev/config.toml
cargo run -p vnox-voice-node -- --config dev/config.toml
cargo run -p vnox-client
```

On first gateway start, `dev/data/server_identity.json` is created automatically.
Back it up if you care about a stable server identity across reinstalls.

## E2E voice test

With gateway and voice-node running:

```sh
cargo run -p vnox-client --bin vnox-e2e-voice
```

This headless test connects two TCP clients, joins the `voice` channel, sends UDP packets, and verifies the voice-node relays between them. Exit code 0 means pass.

## Logs

Set log level via environment variable:

```sh
VNOX_LOG=debug cargo run -p vnox-gateway -- --config dev/config.toml
```

## Full reference

Production-oriented options: [docs/03-server/configuration.md](../docs/03-server/configuration.md).
