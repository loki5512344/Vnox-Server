# Clients

VNOX is a native-only platform. There is no web client and none is planned.

## Why no web client

- WebRTC adds latency overhead incompatible with VNOX's voice latency targets
- WASM + wgpu in browser is not a viable egui deployment target today
- Electron is explicitly rejected (it's what we're building against)
- A browser tab is not the right environment for a persistent voice client

The desktop client is a real native binary. It starts fast, uses minimal memory,
and has direct access to audio hardware without browser sandboxing.

## Clients

- [desktop.md](desktop.md) — primary client, Windows / macOS / Linux
- [mobile.md](mobile.md) — Phase 3, stack TBD
- [overlay.md](overlay.md) — in-game HUD, Phase 2
