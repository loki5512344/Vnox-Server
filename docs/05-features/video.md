# Video Chat — Architecture

## Goal

Add real-time video to voice channels.
Users can share their webcam feed alongside voice.

## Architecture

```
Current (voice only):
├─ Gateway: TCP (signaling, chat)
├─ Voice-node: UDP (Opus voice relay)
└─ Client: mic → Opus → UDP → voice-node → UDP → client → playback

With video:
├─ Gateway: TCP (signaling) — UNCHANGED
├─ Voice-node: UDP (Opus voice) — UNCHANGED
├─ Video-node: UDP (H.264/VP9 relay) — NEW
│   └─ Receives encoded frames, relays to channel members
└─ Client: mic → Opus → UDP → voice-node
           webcam → H.264 → UDP → video-node ← UDP → decode → display grid
```

## Video-node

New optional component, separate binary or embedded in voice-node.

### Responsibilities
- Receive encoded video frames over UDP
- Relay to all other clients in the same voice channel
- No transcoding (relay only — CPU efficient)
- Max resolution / bitrate per channel configurable

### Packet format

```json
{
    "packet_id": "VIDEO_FRAME",
    "channel_id": 12345,
    "sender_id": "<pubkey>",
    "frame_seq": 42,
    "codec": "h264",           // or "vp9"
    "keyframe": false,
    "data": "<base64 encoded frame>"
}
```

Initially JSON (matching Phase 1 convention), binary framing in Phase 2.

## Client capture pipeline

New file: `client/src/video/capture.rs`

1. Enumerate webcam devices via `nokhwa` or `video4linux`
2. Capture frames at configurable resolution (720p default)
3. Encode to H.264 via `ffmpeg-next` or hardware encoder
4. Packetize into MTU-friendly chunks
5. Send over UDP to video-node

### Dependencies

- `nokhwa` — cross-platform camera capture (Rust)
- `ffmpeg-next` or `rav1e` — H.264/VP9 encoding

## Client UI — Video grid

New component: `client/src/ui/video.rs`

- Grid layout (max 4×4 = 16 participants visible)
- Active speaker highlight (green border)
- Self-view (small, picture-in-picture corner)
- Mute video button per participant
- Resolution/quality indicator per stream

Layout modes:
- 1 participant → full width
- 2-4 → 2×2 grid
- 5-9 → 3×3 grid
- 10-16 → 4×4 grid with scrolling

## Implementation phases

### Phase 1 (MVP)
- H.264 encoding with `nokhwa` + `ffmpeg-next`
- Single video-node binary
- 2×2 grid in UI
- 720p max resolution, 30 fps

### Phase 2
- VP9 support (better quality/bitrate)
- Adaptive resolution (auto downscale on packet loss)
- Picture-in-picture self-view
- Screen sharing (desktop capture)

### Phase 3
- Hardware encoding (NVENC/VAAPI)
- Simulcast (different resolution per stream)
- Recording support
