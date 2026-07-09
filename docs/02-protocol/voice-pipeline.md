# Voice Pipeline

> **Phase 1 note:** capture, Opus encode/decode, and UDP relay exist in the repo.
> RNNoise, echo cancellation, VAD, and jitter buffer integration are Phase 2 or not wired yet.
> See [00-status.md](../00-status.md).

## Overview

```
Microphone
    │
    ▼
PCM Capture (cpal)
    │  48000 Hz, mono, f32
    ▼
Pre-processing
    │  noise suppression (RNNoise) - Phase 2
    │  echo cancellation - Phase 2
    │  VAD (voice activity detection) - Phase 2
    ▼
Opus Encode
    │  frame: 10 / 20 / 40ms
    │  bitrate: 8–128 kbps (default 64k)
    │  mode: VOIP (optimized for speech)
    ▼
LNEx Voice Packet
    │  header + opus_data
    │  encrypted + compressed (Phase 2; plaintext in v0.1.x)
    ▼
UDP → Voice Node
    │
    ▼
Jitter Buffer
    │  reorder by voice_seq (code exists; not used in relay yet)
    │  schedule by timestamp
    │  adaptive size: 20–80ms
    ▼
Opus Decode
    │  packet loss concealment if gap in sequence
    ▼
PCM Output
    │
    ▼
Playback (cpal / rodio)
```

---

## Codec

### Opus

VNOX uses Opus exclusively for voice encoding.

Parameters:

| Setting | Value | Notes |
|---------|-------|-------|
| Sample rate | 48000 Hz | Opus native rate |
| Channels | 1 (mono) | stereo optional in Phase 2 |
| Application | VOIP | optimized for speech, lower complexity |
| Bitrate | 8–128 kbps | default 64k, user-configurable |
| Frame size | 20ms default | configurable: 10 / 20 / 40ms |
| FEC | enabled | forward error correction for packet loss |
| DTX | enabled | discontinuous transmission, silence suppression |

Lower frame size = lower latency, higher CPU and packet rate.
Recommended: 20ms for balance, 10ms for ultra-low latency setups.

### Why Opus

- royalty-free
- outperforms MP3/AAC at low bitrates for speech
- built-in packet loss concealment
- adaptive bitrate
- widely supported (libopus, bindings for every language)

---

## Pre-processing

> **Phase 2.** Not implemented in the current client (`client/src/audio/`).

Applied before Opus encoding on the capture path (target design).

### Noise suppression

Implementation: RNNoise (ML-based, ~2% CPU)
Applied to raw PCM before encoding.
Configurable: on / off.

### Echo cancellation

Removes microphone pickup of speaker output.
Implementation: platform AEC or software fallback.
Configurable: on / off.

### Voice activity detection (VAD)

Detects when the user is speaking to avoid sending silence packets.
Used in "voice activity" mode (alternative to push-to-talk).

Threshold: configurable 0–100%, default 40%.

In push-to-talk mode, VAD is bypassed — packets are sent only while
the hotkey is held.

DTX in Opus also provides a secondary layer of silence suppression
at the encoder level.

---

## UDP relay

### Path

```
Client A  ──UDP──▶  Voice Node  ──UDP──▶  Client B
                         │
                    ──UDP──▶  Client C
                         │
                    ──UDP──▶  Client D
```

The voice node receives packets from each speaker and relays them
to all other clients in the same channel.

No mixing is done on the server. Clients receive separate streams
per speaker and mix locally. This allows per-speaker volume control
on the client side.

### Direct P2P (future)

In Phase 4, direct P2P paths may be established between clients
to skip the relay hop. The relay remains as fallback.

---

## Jitter buffer

The jitter buffer absorbs network jitter and reorders out-of-order packets
before passing them to the decoder.

### Operation

1. Packets arrive with `voice_seq` and `timestamp`
2. Buffer holds packets for a configurable window
3. Packets are released in sequence order at scheduled playout time
4. If a packet is missing when due: Opus PLC generates a concealment frame
5. If a late packet arrives after playout: discarded

### Configuration

| Setting | Default | Range | Notes |
|---------|---------|-------|-------|
| Buffer size | 40ms | 20–80ms | lower = less latency, more glitches |
| Adaptive mode | on | on/off | auto-adjusts based on observed jitter |
| Max late tolerance | 80ms | — | packets older than this are discarded |

Adaptive mode measures jitter over a rolling window and expands/shrinks
the buffer target accordingly. On a stable LAN, buffer converges to ~20ms.
On a lossy WAN, it may expand to 60–80ms.

---

## Packet loss concealment

When a voice_seq gap is detected, Opus generates a concealment frame
using the previous frame's data. This produces a short fade or
interpolated audio rather than a click or silence.

FEC (Forward Error Correction) in Opus encodes redundant data from
the previous frame into the current packet. If the previous packet was
lost but the current one arrives, the previous frame can be recovered.

---

## Latency budget (target)

```
Microphone capture latency       ~5ms
Pre-processing (RNNoise, AEC)    ~2ms
Opus encode (20ms frame)         ~20ms
UDP tx                           ~1–5ms (local)
Voice node relay                 ~0.5ms
UDP rx                           ~1–5ms (local)
Jitter buffer (adaptive)         ~20–40ms
Opus decode                      ~1ms
Playback buffer                  ~5ms
──────────────────────────────────────
Total (local network)            ~55–80ms
Target (good conditions)         < 60ms
```

For ultra-low latency setups (LAN gaming): use 10ms frame size,
reduce jitter buffer to 20ms, disable adaptive mode.
Expected total: ~35–45ms.
