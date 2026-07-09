# Vnox Repository Metadata

## Description
Self-hosted realtime voice and chat platform. Decentralized, lightweight, and moddable. Built on LNEx, a custom protocol for low-latency federated communication. Not Discord. Not TeamSpeak. Not cloud.

## Topics / Tags
- communication
- voice-chat
- rust
- self-hosted
- decentralized
- real-time
- low-latency
- protocol
- federated
- lightweight
- moddable
- open-source
- p2p
- networking
- gateway
- voice-relay
- chat
- lnex

## Short Descriptions

### Primary (for GitHub repo description)
Self-hosted realtime voice and chat on a custom low-latency protocol. Decentralized, lightweight, and extensible.

### Secondary Options
- Decentralized alternative to Discord for self-hosted deployment
- Low-latency federated communication platform with custom LNEx protocol
- Open-source voice and chat server written in Rust

## Key Features
- Self-hosted deployment
- Decentralized architecture
- Real-time voice and text communication
- Low-latency protocol (LNEx)
- Lightweight footprint
- Extensible/moddable design
- Federation support (Phase 3 roadmap)
- Desktop client with egui UI
- SQLite/PostgreSQL backend support

## Technology Stack
- **Language**: Rust (99.6%)
- **Protocol**: LNEx (custom)
- **Audio**: Opus codec
- **Database**: SQLite (with PostgreSQL support planned)
- **UI**: egui (desktop client)
- **Transport**: TCP (gateway), UDP (voice relay)

## Current Status
- Phase 1: Implemented, not production-ready
- Gateway: ✅ TCP listener, LNEx handshake, channels, chat, SQLite
- Voice node: ✅ UDP relay, voice packet routing
- Desktop client: ✅ egui UI, net layer, audio pipeline (partial)
- LNEx protocol: ✅ Specified and implemented
- Federation: 🔄 Planned (Phase 3)
- Mobile client: 🔄 Planned (Phase 3)

⚠️ **Security Warning**: Traffic in v0.1.x is unencrypted plaintext. Do not use in production.

## License
- **Code**: GPL-3.0
- **Protocol**: CC0 (Public Domain)
