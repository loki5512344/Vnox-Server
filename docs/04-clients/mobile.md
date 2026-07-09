# Mobile Client

> Status: Phase 3 — not yet started.
> This document tracks intentions and open questions.

---

## Goal

A native mobile client for iOS and Android.
Not a PWA. Not a wrapper around the desktop client.

## Open questions

### UI framework

egui on mobile is not practical today — touch input support is limited,
and the immediate mode model doesn't map well to mobile interaction patterns.

Candidates under consideration:

| Option | Notes |
|--------|-------|
| Rust + custom egui mobile backend | Most consistent with desktop codebase, significant work |
| Rust + Makepad | Rust-native UI designed for mobile, less mature |
| Rust core + Flutter UI | Dart for UI, Rust for audio/networking via FFI |
| Rust core + Swift/Kotlin UI | Platform-native UI, Rust for the important parts |

Decision: deferred to Phase 3.

### Audio

Mobile audio APIs are significantly more constrained than desktop:

- iOS: AVAudioSession, strict background audio rules
- Android: AAudio / OpenSL ES, varying latency by device

opus encoding is the same. cpal has partial mobile support.
The audio pipeline will need platform-specific tuning.

### Background operation

Voice calls in the background require OS-level permission and
platform-specific handling (CallKit on iOS, ConnectionService on Android).
This is non-trivial and will be a significant portion of mobile dev effort.

---

## What mobile must support (MVP)

- connect to a VNOX node
- join voice channels
- push-to-talk
- text chat
- identity (same keypair as desktop, importable via QR or keyfile)

## What mobile explicitly will not do

- host a node (gateway or voice-node)
- run plugins
- game overlay

---

## Timeline

Mobile client is Phase 3, after:
- Phase 1: desktop client + server MVP
- Phase 2: overlay, permissions, friend system

Estimated start: after Phase 2 is stable.
