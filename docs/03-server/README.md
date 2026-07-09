# Server / Gateway

A VNOX node consists of two processes:

```
gateway      — TCP, handles auth / chat / channels / permissions
voice-node   — UDP, handles voice relay
```

Both are written in Rust. They can run on the same machine or separately.
For most self-hosted setups, running both on one machine is fine.

---

## Minimum requirements

| | Minimum | Recommended |
|---|---|---|
| CPU | 1 core | 2+ cores |
| RAM | 256 MB | 512 MB |
| Bandwidth | 1 Mbps up | 10 Mbps up |
| OS | Linux x86_64 | Linux x86_64 |
| Ports | TCP 7600, UDP 7700 | configurable |

Voice bandwidth scales with concurrent speakers:
~64 kbps per active speaker (default 64k bitrate, 20ms frames).

---

## Sections

- [deployment.md](deployment.md) — how to install and run a node
- [configuration.md](configuration.md) — full config.toml reference
- [operations.md](operations.md) — metrics, logs, backup, upgrades
