# Video, Camera & Screen Share — Design Audit

> Audits `docs/05-features/video.md` against the current codebase and roadmap.
> Each gap has: problem, recommendation, priority, target file.

---

## Summary

`video.md` provides a solid high-level sketch (video-node, H.264 UDP relay, nokhwa capture, grid UI).
But critical details are missing for actual implementation. This audit fills those gaps.

---

## 1. Video-node — Integration with existing architecture

### Gap 1.1 — Video-node lifecycle

**Problem:** `video.md` says "separate binary or embedded in voice-node" but doesn't decide. Voice-node has just been refactored with jitter buffer — video needs the same relay pattern.

**Recommendation:** Separate binary (`vnox-video-node`). Video frames are fundamentally different from voice packets (keyframes, fragmentation, higher bandwidth). Keeping them separate avoids coupling and allows independent scaling.

**Priority:** P0 — must decide before writing a line of code.

**Target:** `video.md` — add "Deployment" section.

### Gap 1.2 — Channel membership signalling

**Problem:** Video-node needs to know which users are in which channel. Currently this is managed by gateway. How does video-node learn membership?

**Recommendation:** Gateway sends `VIDEO_SESSION_JOIN` / `VIDEO_SESSION_LEAVE` events to video-node over TCP (internal control channel), same pattern as planned for gateway→voice-node membership signalling (Phase 2 in roadmap).

```json
// Gateway → Video-node (internal TCP)
{
  "event": "VIDEO_SESSION_JOIN",
  "channel_id": 12345,
  "user_id": "<pubkey>",
  "socket_addr": "192.168.1.5:7701"
}
```

**Priority:** P0 — video relay can't work without knowing recipients.

**Target:** `video.md` — add "Gateway integration" section.

### Gap 1.3 — No encryption spec

**Problem:** Voice packets are encrypted with ChaCha20-Poly1305. Video packets are not addressed — are they encrypted? With what keys?

**Recommendation:** Same ChaCha20-Poly1305 AEAD as voice. Each video session gets ephemeral keys derived via the same X25519 ECDH + HKDF-SHA256 path used for voice. Video-node does NOT decrypt (transparent relay, same as voice-node).

The session key exchange happens during `HELLO` handshake — add a `video_key` field alongside the existing `voice_key`.

**Priority:** P0 — security regression if video is plaintext.

**Target:** `video.md` — add "Encryption" section; `docs/02-protocol/security.md` — add video encryption note.

---

## 2. Camera Capture — Missing details

### Gap 2.1 — nokhwa API surface and fallback

**Problem:** `video.md` mentions `nokhwa` but not the concrete API or what happens on platforms where it doesn't work (Linux v4l2 permissions, Wayland).

**Recommendation:** Abstract behind a `VideoSource` trait:

```rust
// client/src/video/capture.rs
pub trait VideoSource {
    fn devices() -> Vec<DeviceInfo>;
    fn start(config: StreamConfig) -> Result<FrameStream>;
}

struct NokhwaSource { ... }
struct StubSource { ... }  // fallback

pub struct StreamConfig {
    pub device_id: String,
    pub resolution: Resolution,   // 640x480, 1280x720, 1920x1080
    pub fps: u32,                 // 15, 30
    pub format: PixelFormat,      // NV12, I420, BGRA
}
```

Make `nokhwa` a feature gate (same pattern as `rnnoise` for voice).

**Priority:** P0 — blocks camera implementation.

**Target:** New file: `client/src/video/capture.rs` (spec in `video.md`).

### Gap 2.2 — Device hotplug / switch

**Problem:** No mention of what happens when camera is unplugged/replugged or user switches camera mid-call.

**Recommendation:** `VideoSource` emits events:
```rust
enum CaptureEvent {
    Frame(VideoFrame),
    DeviceLost(String),
    DeviceReconnected(String),
    Error(String),
}
```

Client handles `DeviceLost` by showing placeholder tile. User can select new device in settings without leaving call.

**Priority:** P1 — quality of life, not MVP blocker.

**Target:** `video.md` — add "Device lifecycle" to capture section.

### Gap 2.3 — Resolution / FPS negotiation

**Problem:** `video.md` says "720p default, 30 fps" but doesn't say how clients agree on parameters. Different clients have different cameras.

**Recommendation:** Server advertises max resolution per channel. Client sends its capability in `VIDEO_SESSION_JOIN`. Server picks min(max_server, max_client) per sender.

```toml
# server config
[video.channel_defaults]
max_resolution = "1280x720"
max_fps = 30
max_bitrate_kbps = 2500
```

**Priority:** P1 — needed before multi-user video testing.

**Target:** `video.md` — add "Capability negotiation" section.

---

## 3. Screen Share — Almost entirely missing

### Gap 3.1 — Screen capture crate

**Problem:** `video.md` mentions "Screen sharing (desktop capture)" as one bullet in Phase 2, no detail.

**Recommendation:**

| Platform | Crate | Notes |
|----------|-------|-------|
| Windows | `windows-capture` or DXGI via `winapi` | DXGI Desktop Duplication API — best perf |
| Linux X11 | `x11cap` or raw XSHM | X11 only, no Wayland |
| Linux Wayland | `pipewire` via `pw-video` or xdg-desktop-portal | Portal is the standard path |
| macOS | `screencapturekit` via objc bindings or `CGDisplay` | ScreenCaptureKit requires macOS 13+ |

Abstract behind same `VideoSource` trait as camera. Two implementations: `CameraSource`, `ScreenSource`.

**Priority:** P1 — needed for Phase 2 screen share.

**Target:** New doc: `docs/05-features/screen-share.md` or extend `video.md` Phase 2.

### Gap 3.2 — Window/display selection UI

**Problem:** User needs to pick which screen or window to share. No UI spec.

**Recommendation:** Modal dialog when user clicks "Screen Share":

```
┌─────────────────────────────────────────┐
│ Share Your Screen                        │
│─────────────────────────────────────────│
│ [Screens]  [Windows]                     │  ← tabs
│─────────────────────────────────────────│
│ ┌──────────┐ ┌──────────┐               │
│ │ Display 1│ │ Display 2│               │
│ │1920×1080 │ │2560×1440 │               │
│ │   [✓]    │ │          │               │
│ └──────────┘ └──────────┘               │
│─────────────────────────────────────────│
│ ☐ Share system audio                    │
│ ☐ Optimize for video (60fps)            │
│─────────────────────────────────────────│
│               [Cancel]  [Share]          │
└─────────────────────────────────────────┘
```

**Priority:** P1 — UX blocker for screen share.

**Target:** `video.md` — add "Screen share UI" section.

### Gap 3.3 — System audio capture with screen

**Problem:** "Share system audio" checkbox in the mockup — how? No mention in video.md.

**Recommendation:**
- Windows: WASAPI loopback (`cpal` supports it with `loopback` config)
- Linux: PulseAudio monitor or PipeWire
- macOS: BlackHole or ScreenCaptureKit (built-in on 13+)

Feature-gate it: `--features screen-audio`. Not all platforms support it cleanly.

**Priority:** P2 — nice to have, not MVP.

**Target:** `video.md` Phase 2 — add "System audio" bullet.

### Gap 3.4 — Screen share FPS strategy

**Problem:** Screen share has different FPS needs than camera. Coding: 15fps is fine. Gaming: 60fps needed.

**Recommendation:** Client detects content type (static → low FPS, motion → high FPS) and adapts. User can override with "Optimize for video" checkbox.

Default: 15fps for screen share, 30fps for camera.

**Priority:** P2 — optimization, not MVP.

**Target:** `video.md` — add to Phase 2.

---

## 4. Interaction Model — Video + Voice

### Gap 4.1 — Video tied to voice channel

**Problem:** `video.md` implies video is in the same channel as voice. What if users want video-only (no voice) or voice-only (no video)?

**Recommendation:** Video is a capability toggle within a voice channel, not a separate channel type:

1. User joins voice channel (as today)
2. User clicks "Enable Camera" → client starts sending video frames to video-node
3. User clicks "Disable Camera" → stops sending, voice continues
4. Same for "Share Screen"

Channel membership = voice channel membership. Video is additive.

New voice state: `VOICE_STATE` gets a `video: bool` field.

**Priority:** P0 — architectural decision, affects protocol design.

**Target:** `video.md` — add "Interaction model" section; `docs/02-protocol/packets.md` — extend VOICE_STATE.

### Gap 4.2 — Video without voice

**Problem:** Some users may want to watch a stream without transmitting audio. Current model requires voice channel join.

**Recommendation:** Phase 1: allow joining voice channel muted+deafened to watch video. Phase 3: separate "watch-only" mode (VIEW permission).

**Priority:** P2 — Phase 1 workaround is acceptable.

**Target:** `video.md` — add "Watch-only mode" to Phase 3.

---

## 5. Performance & Quality

### Gap 5.1 — Bitrate adaptation

**Problem:** No mechanism to adjust video bitrate based on network conditions. Voice has jitter buffer adaptation, video has nothing.

**Recommendation:** Client-side adaptive bitrate (ABR):
- Measure packet loss and RTT from video-node ACKs
- Adjust encoder bitrate up/down based on available bandwidth
- 3 tiers: low (500kbps), medium (1500kbps), high (4000kbps)

Video-node sends periodic `VIDEO_STATS` with per-receiver loss rates:

```json
{
  "channel_id": 12345,
  "receivers": {
    "user_a": { "loss_pct": 0.5, "rtt_ms": 12 },
    "user_b": { "loss_pct": 8.0, "rtt_ms": 150 }
  }
}
```

**Priority:** P1 — needed for real-world use.

**Target:** `video.md` — add "Adaptive bitrate" section.

### Gap 5.2 — Simulcast design

**Problem:** "Simulcast" is listed as Phase 3 with no design. Different receivers have different bandwidth.

**Recommendation:** Sender encodes 2-3 quality layers. Video-node forwards appropriate layer to each receiver based on their reported bandwidth.

```
Sender → encode:  1080p (4Mbps) + 720p (1.5Mbps) + 360p (500kbps)
                         │
                   Video-node
                    ╱    │    ╲
            1080p→A  720p→B  360p→C  (based on per-receiver stats)
```

Requires VP9 SVC or H.264 simulcast. Complex — Phase 3 is correct.

**Priority:** P2 — Phase 3, but design it now to avoid architecture lock-in.

**Target:** `video.md` Phase 3 — expand "Simulcast" bullet.

### Gap 5.3 — Keyframe interval

**Problem:** New viewers joining mid-stream need a keyframe to start decoding. Video.md doesn't mention keyframe strategy.

**Recommendation:**
- Sender inserts keyframe every 2 seconds (configurable)
- Video-node caches last keyframe per sender
- On new viewer join: video-node sends cached keyframe immediately, then regular frames
- `VIDEO_SESSION_JOIN` triggers keyframe push

Add `REQUEST_KEYFRAME` packet for explicit PLI (Picture Loss Indication).

**Priority:** P1 — new viewers can't see video without keyframe.

**Target:** `video.md` — add "Keyframe handling" section.

---

## 6. Missing Protocol Details

### Gap 6.1 — No packet fragmentation spec

**Problem:** Video frames can be 50-100KB. UDP MTU is ~1400 bytes. Video.md says "Packetize into MTU-friendly chunks" but doesn't define the fragmentation protocol.

**Recommendation:** Reuse the LNEx fragmentation flags from the base packet header (bits 2-3):

```
FRAGMENTED (bit 2) + LAST_FRAG (bit 3)
```

Each video frame:
1. Split into 1400-byte chunks
2. Each chunk gets same `frame_seq`, incrementing `fragment_seq`
3. Last chunk sets `LAST_FRAG` flag
4. Receiver reassembles before decode

New packet type: `VIDEO_FRAME_FRAGMENT (0x0012)` — separate from unfragmented `VIDEO_FRAME (0x0011)`.

Wait — `VIDEO_FRAME` ID isn't assigned in the packet registry yet. Register:

```
0x0011  VIDEO_FRAME        single (small) video frame, no fragmentation
0x0012  VIDEO_FRAME_FRAG   fragment of a larger video frame
```

**Priority:** P0 — video can't work over UDP without fragmentation.

**Target:** `video.md` — replace packet format section; `docs/02-protocol/packets.md` — add 0x0011/0x0012 to registry.

### Gap 6.2 — No RTCP-like receiver reports

**Problem:** Sender has no feedback about what receivers are experiencing. Voice has implicit feedback (jitter buffer stats), video needs explicit.

**Recommendation:** Minimal receiver report packet:

```json
// 0x0013 VIDEO_RECEIVER_REPORT — client → video-node → sender
{
  "frame_seq": 1042,
  "loss_cumulative": 15,
  "loss_fraction": 2,       // percent of last N packets
  "jitter_ms": 8,
  "rtt_ms": 35
}
```

Client sends every 1 second. Video-node aggregates and forwards to sender.

**Priority:** P1 — needed for ABR and quality adaptation.

**Target:** `video.md` — add "Receiver reports" section.

---

## 7. Guild Integration (Phase 1.2)

### Gap 7.1 — Video permissions already defined

**Status:** ✅ `STREAM` (bit 18) permission is already in the community model (`docs/07-community-model.md`). Covers both camera and screen share.

No gap here — just implement the check in gateway when processing `VIDEO_SESSION_JOIN`.

### Gap 7.2 — Channel-level video settings

**Problem:** Guild admins may want to disable video in certain channels (text-only channels, AFK channel).

**Recommendation:** Add `video_allowed: bool` to channel config. Default: `true` for voice channels, `false` for text channels.

```sql
ALTER TABLE channels ADD COLUMN video_allowed BOOLEAN NOT NULL DEFAULT 1;
```

Gateway rejects `VIDEO_SESSION_JOIN` if `video_allowed = false`.

**Priority:** P1 — admin control expected by guild owners.

**Target:** `docs/10-database.md` — add column to channels table; `docs/07-community-model.md` — add `video_allowed` to channel settings.

---

## 8. Mobile Considerations (Phase 3)

### Gap 8.1 — Front/back camera switch

**Problem:** Mobile video.md doesn't mention camera switching at all.

**Recommendation:** Mobile client sends `CAMERA_SWITCH` event to video-node — no protocol change needed, just local capture change. New frame stream with `camera: front|back` metadata.

**Priority:** P2 — Phase 3.

**Target:** `docs/04-clients/mobile.md` (doesn't exist yet — create in Phase 3).

### Gap 8.2 — Self preview

**Problem:** Desktop video grid has "Self-view (small, picture-in-picture corner)". On mobile the self-view is more important (front camera framing).

**Recommendation:** Mobile layout: self-view full-width at top, other participants in scrollable grid below. Toggle button to swap.

**Priority:** P2 — Phase 3.

**Target:** `docs/04-clients/mobile.md`.

---

## Gap Summary — What to Add Before Implementation

| # | Gap | Priority | Target file |
|---|-----|----------|-------------|
| 1 | Video-node binary decision + deployment | P0 | `video.md` |
| 2 | Gateway membership signalling to video-node | P0 | `video.md` |
| 3 | Video encryption (ChaCha20-Poly1305) | P0 | `video.md`, `security.md` |
| 4 | Video frame fragmentation protocol | P0 | `video.md`, `packets.md` |
| 5 | Video ↔ voice interaction model | P0 | `video.md`, `packets.md` |
| 6 | VideoSource trait + feature-gated nokhwa | P0 | `video.md` |
| 7 | Keyframe caching + PLI request | P1 | `video.md` |
| 8 | Receiver reports (RTCP-like) | P1 | `video.md` |
| 9 | Adaptive bitrate design | P1 | `video.md` |
| 10 | Screen share crate selection + ScreenSource | P1 | `video.md` |
| 11 | Window/display selection UI | P1 | `video.md` |
| 12 | Resolution/FPS negotiation | P1 | `video.md` |
| 13 | Channel-level video_allowed setting | P1 | `database.md`, `community-model.md` |
| 14 | Camera hotplug/switch handling | P1 | `video.md` |
| 15 | Video-only (watch) mode | P2 | `video.md` |
| 16 | System audio with screen share | P2 | `video.md` |
| 17 | Screen share FPS strategy | P2 | `video.md` |
| 18 | Simulcast design | P2 | `video.md` |
| 19 | Mobile front/back camera + self preview | P2 | `mobile.md` |

---

## Recommended Implementation Order

1. **Update `video.md`** — incorporate P0 and P1 gaps from this audit into the design doc
2. **Register packet types** — 0x0011–0x0013 in `packets.md`
3. **Video-node prototype** — separate binary, UDP relay, no encoding/decoding (transparent relay like voice)
4. **Gateway ↔ video-node signalling** — internal TCP control channel for membership
5. **Client camera capture** — `VideoSource` trait + `NokhwaSource` behind feature gate
6. **Fragmentation** — implement FRAGMENTED/LAST_FRAG for video frames
7. **Grid UI** — basic 2×2 grid in client
8. **Encryption** — ChaCha20-Poly1305 for video frames
9. **ABR + receiver reports** — iterate toward production quality
10. **Screen share** — `ScreenSource` implementation + picker UI
