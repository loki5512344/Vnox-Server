# VNOX

Self-hosted realtime voice and chat. Decentralized. Lightweight. Moddable.
Built on LNEx, a custom protocol for low-latency federated communication.

Not Discord. Not TeamSpeak. Not cloud.

```
vnox://server/channel
```

## Quick links

- [Architecture](docs/01-architecture.md)
- [Protocol](docs/02-protocol/README.md)
- [Current status and limitations](docs/00-status.md)
- [Server setup](docs/03-server/deployment.md)
- [Local dev config](dev/README.md)
- [Contributing](docs/community/contributing.md)
- [Changelog](CHANGELOG.md)

## Status

Phase 1: implemented, not production ready.

See [docs/00-status.md](docs/00-status.md) for an honest list of what works,
what is only specified on paper, and known gaps.

| Component      | Status |
|----------------|--------|
| Gateway        | TCP listener, LNEx handshake, channels, chat, SQLite |
| Voice node     | UDP relay, voice packet routing |
| Desktop client | egui UI, net layer, audio pipeline (partial) |
| LNEx protocol  | Specified and implemented (JSON in Phase 1) |
| Federation     | Planned (Phase 3) |
| Mobile client  | Planned (Phase 3) |

Traffic in v0.1.x is **unencrypted plaintext**. Do not use in production.

## Running locally

Requires Rust 1.85+.

```sh
# terminal 1 - gateway
cargo run -p vnox-gateway -- --config dev/config.toml

# terminal 2 - voice node
cargo run -p vnox-voice-node -- --config dev/config.toml

# terminal 3 - client
cargo run -p vnox-client
```

The client connects to `127.0.0.1:7600` by default (editable in the UI).
Config details: [dev/README.md](dev/README.md).

### Opus on Windows

`audiopus_sys` builds libopus from source via CMake.
CMake 4.x requires a policy flag, already set in `.cargo/config.toml`:

```toml
[env]
CMAKE_POLICY_VERSION_MINIMUM = "3.5"
```

No manual steps needed.

## License

GPL-3.0. See [docs/LICENSE.md](docs/LICENSE.md).

The LNEx protocol specification is CC0 (public domain).
