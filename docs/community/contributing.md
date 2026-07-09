# Contributing

---

## License

VNOX is licensed under **GPL-3.0**. See `../LICENSE.md` for the full text and
what this means for plugins and protocol implementations.

The LNEx protocol specification is additionally licensed under **CC0** (public domain) —
anyone can implement it under any license.

---

## Before you start

1. Check existing issues — your idea or bug may already be tracked.
2. For significant changes, open an issue first to discuss approach.
3. For small fixes (typos, docs, obvious bugs), PRs are welcome directly.

---

## Setting up the dev environment

### Prerequisites

- Rust stable (latest) — `rustup update stable`
- System audio libraries:
  - Linux: `apt install libasound2-dev libopus-dev pkg-config`
  - macOS: `brew install opus`
  - Windows: opus ships with the crate, no extra step
- Docker (optional, for running a local node)

### Build

```bash
git clone https://github.com/vnox/vnox
cd vnox

# Build everything
cargo build

# Build release
cargo build --release

# Run gateway (dev mode)
cargo run -p vnox-gateway -- --config dev/config.toml

# Run voice node
cargo run -p vnox-voice-node -- --config dev/config.toml

# Run client
cargo run -p vnox-client
```

### Running tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p vnox-gateway

# With logs
RUST_LOG=debug cargo test -p vnox-gateway -- --nocapture
```

---

## Repository structure

```
vnox/
├── client/         # Desktop client (egui + wgpu)
│   └── src/
│       ├── ui/     # egui panels and widgets
│       ├── audio/  # cpal capture/playback, Opus encode/decode
│       ├── net/    # LNEx client, quinn UDP
│       └── identity/ # keypair, auth
│
├── gateway/        # TCP gateway
│   └── src/
│       ├── auth/   # identity verification, sessions
│       ├── channels/ # channel management
│       ├── proto/  # LNEx packet handling
│       └── storage/ # SQLite / Postgres
│
├── voice-node/     # UDP voice relay
│   └── src/
│       ├── relay/  # packet routing per channel
│       └── jitter/ # jitter buffer
│
├── protocol/       # LNEx .proto schemas
│   └── *.proto
│
├── plugins/        # Plugin runtime + example plugins
│   └── examples/
│
├── sdk/            # Client SDK (future)
├── docs/           # This documentation
└── dev/            # Dev config, test fixtures
    └── config.toml
```

---

## Code style

- `cargo fmt` before committing — enforced in CI
- `cargo clippy -- -D warnings` must pass — enforced in CI
- No `unwrap()` in library code — use proper error propagation
- No `unsafe` without a comment explaining why it's safe
- Public API items must have doc comments

---

## Commit messages

Follow conventional commits:

```
feat(gateway): add rate limiting per IP
fix(client): handle reconnect on TCP drop
docs(protocol): clarify voice packet sequence field
chore(deps): update tokio to 1.37
```

Types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `perf`

Scope is the crate or subsystem: `gateway`, `client`, `voice-node`, `protocol`, `docs`.

---

## Pull requests

- Keep PRs focused — one concern per PR
- Include tests for non-trivial changes
- Update docs if the change affects user-facing behavior
- CI must pass before merge

PR title follows the same format as commit messages.

---

## Governance

VNOX uses the **BDFL model** — the maintainer makes final decisions on all matters.

For large or breaking changes (especially LNEx protocol changes), open an issue
and discuss before implementing. Protocol PRs without prior discussion will not be merged.

---

## Protocol changes

Changes to the LNEx protocol are treated differently from implementation changes.

- Any change to packet format, auth flow, or behavior must be documented
  in `docs/02-protocol/` before implementation
- Breaking changes require a LNEx version bump
- Non-breaking additions are allowed within the same version with a changelog entry
- Protocol PRs require more review time than implementation PRs

---

## Where to start

Good first issues are tagged `good first issue` on GitHub.

If you want to contribute but don't know where:

1. Run a node locally and report anything confusing about the setup process
2. Improve documentation — anything unclear in `docs/` is a valid fix
3. Write tests for existing gateway or voice-node code
4. Implement a feature from the Phase 1 checklist in `../06-roadmap.md`
