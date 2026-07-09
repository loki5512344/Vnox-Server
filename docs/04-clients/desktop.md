# Desktop Client

The primary VNOX client. Native, fast, lightweight.

## Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust |
| UI framework | egui |
| Renderer | wgpu |
| Audio capture/playback | cpal |
| Audio processing | rodio |
| Voice codec | opus (libopus bindings) |
| Networking | quinn (QUIC/UDP), tokio |
| Serialization | prost (Protobuf) |

### Why egui

- immediate mode UI: simple to reason about, no complex state trees
- runs on wgpu: same renderer as the rest of the GPU pipeline
- truly cross-platform: one codebase, same behavior on Windows/macOS/Linux
- no runtime dependencies: ships as a single binary

### Why wgpu

- modern GPU API (Vulkan / Metal / DX12 / WebGPU backend)
- future-proof for overlay rendering and spatial audio visualizations
- native on all tier-1 platforms

---

## Platforms

| Platform | Status |
|----------|--------|
| Linux x86_64 | Phase 1 |
| Windows x86_64 | Phase 1 |
| macOS (Apple Silicon) | Phase 1 |
| macOS (Intel) | Phase 1 |
| Linux ARM64 | Phase 2 |

---

## Audio pipeline

```
cpal (capture)
    │ PCM f32 48000Hz
    ▼
RNNoise (noise suppression)
    ▼
AEC (echo cancellation)
    ▼
VAD (voice activity detection)
    ▼
opus encode
    ▼
LNEx UDP packet → voice-node
```

Playback:

```
LNEx UDP packet ← voice-node
    ▼
jitter buffer
    ▼
opus decode
    ▼
rodio (playback)
    ▼
cpal (output device)
```

Audio device selection is configurable in Settings → Voice and Settings → Audio Output.

---

## Design system

The client follows the VNOX Hi-Fi Minimalism design system.
Full specification: `docs/04-clients/design-system.md`

Key decisions:
- monospace font throughout (IBM Plex Mono)
- warm dark palette (`#0d0d0d` base, `#ff6b35` accent)
- no round server icons — square with abbreviation, 40px sidebar
- latency indicator bottom-left, next to identity
- no separate member list panel — voice users shown inline in channel list

---

## Layout

```
┌──────────────────────────────────────────────────────┐
│ titlebar: dots · VNOX wordmark · connected node      │
├────────┬──────────────────┬───────────────────────── │
│ nodes  │ channels         │ main (chat / voice)      │
│ 40px   │ 190px            │ flex                     │
│        │                  │                          │
│  NC ◄  │ nightcore.lnex   │ #general                 │
│  DV ·  │ ─────────────    │ ─────────────────────    │
│  GG    │ # general ◄      │ messages                 │
│  VD    │ # dev-talk       │                          │
│  +     │ # plugins        │                          │
│        │                  │                          │
│        │ ~ lobby          │                          │
│        │   · raven        │ ─────────────────────    │
│        │   · 0xmist       │ #general › [input]       │
│        │   🔇 lurker_7    │                          │
│        │ ~ gaming         │                          │
│        │ ─────────────    │                          │
│        │ ● 12ms lnex v1   │                          │
│        │ [YU] you 🎤 🎧 ⚙ │                          │
└────────┴──────────────────┴──────────────────────────┘
```

---

## Settings

Settings are stored locally. No settings are synced to the server.

### Voice
- input device
- input volume
- noise suppression (RNNoise on/off)
- echo cancellation (on/off)
- activation mode: push-to-talk / voice activity / always on
- VAD threshold
- Opus bitrate (8–128k)
- Opus frame interval (10 / 20 / 40ms)

### Audio output
- output device
- output volume
- jitter buffer size
- adaptive jitter buffer (on/off)

### Network
- relay address
- auto relay selection
- UDP port
- force relay only (disable direct)

### Identity
- view pubkey
- export keypair
- seed phrase backup
- rotate keypair

### Appearance
- color scheme
- UI scale
- font

### Keybinds
- push-to-talk key
- mute toggle
- deafen toggle
- overlay toggle

### Plugins
- installed plugin list
- enable / disable per plugin

### Advanced (debug)
- log level
- log voice packets
- show packet stats
- disable encryption (dev only)

---

## Building from source

```bash
# Prerequisites: Rust stable, system audio libs

# Linux (Ubuntu/Debian)
apt install libasound2-dev libopus-dev

# macOS
brew install opus

# Build
git clone https://github.com/vnox/vnox
cd vnox/client
cargo build --release

# Binary
./target/release/vnox-client
```
