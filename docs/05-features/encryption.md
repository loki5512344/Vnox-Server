# Encryption — Phase 1.1

> **Phase 1.1 status:** Fully implemented. ChaCha20-Poly1305 AEAD with X25519 ECDH + HKDF-SHA256 key exchange. Both TCP (control) and UDP (voice) paths are now encrypted. Forward secrecy via ephemeral session keys.

## Goal

Replace all plaintext traffic (TCP + UDP) with ChaCha20-Poly1305 AEAD encryption.
X25519 ECDH ephemeral key exchange during handshake for forward secrecy.

✅ **COMPLETED in Phase 1.1**

## Key exchange (during HELLO handshake)

1. Server generates ephemeral X25519 keypair per session
2. Server sends public key in `HELLO` packet
3. Client generates own ephemeral X25519 keypair
4. Both compute shared secret via X25519 ECDH
5. Shared secret → HKDF-SHA256 → two keys:
   - `client→server` encryption key
   - `server→client` encryption key
6. Ephemeral keys discarded after session (forward secrecy)

## Packet encryption

- Algorithm: **ChaCha20-Poly1305** (AEAD)
- 96-bit nonce: `session_id || packet_sequence`
- 16-byte Poly1305 authentication tag per packet
- Applied to ALL packets on both TCP and UDP

## Why ChaCha20-Poly1305 over AES-GCM

- Constant-time on all platforms (no hardware AES requirement)
- Faster in software on ARM (common for mobile — Phase 3)
- Simpler nonce management (no IV collision risk)

## Implementation plan

### ✅ Client (DONE)
- `client/src/net/crypto.rs` — SessionCrypto with ChaCha20-Poly1305
- `client/src/net/voice.rs` — `build_packet()` encrypts voice payloads with c2s_key
- `client/src/net/voice.rs` — `spawn_recv()` decrypts incoming voice with s2c_key
- `client/src/net/session/session_loop.rs` — voice_seq counter for encryption nonces

### ✅ Gateway (DONE)
- `gateway/src/proto/crypto.rs` — X25519 ECDH, HKDF-SHA256, ChaCha20-Poly1305
- `gateway/src/net/handshake.rs` — ephemeral key exchange in HELLO/AUTH
- `gateway/src/net/io.rs` — TCP framing with encryption

### ✅ Voice
- `client/src/net/voice.rs` — encrypted voice packets on UDP (plaintext at rest in voice-node, encrypted on wire)
- Voice-node treats packets as opaque bytes (transparent relay)
- End-to-end encryption: client A → encrypted → voice-node → encrypted → client B

## Nonce management

Each session has a monotonic sequence counter:
- Start at 0 on session establishment
- Increment per packet (both directions independently)
- Nonce = `session_id (8 bytes) || sequence (4 bytes)`
- 12-byte nonce fits ChaCha20-Poly1305 standard

## Key dependencies

Already in `Cargo.toml` (workspace):
- `chacha20poly1305 = "0.10"`
- `x25519-dalek = { version = "2", features = ["static_secrets"] }`

Need to add:
- `hkdf = "0.12"` for key derivation
- `sha2 = "0.10"` (HKDF dependency, likely already transitive)

## Testing

- ✅ Unit test: encrypt → decrypt round-trip with known keys (`client/src/net/crypto.rs`)
- ✅ Unit test: tampered ciphertext fails authentication 
- ✅ Unit test: build_packet creates valid encrypted packets
- Manual test: Run client + gateway + voice-node, join voice channel, verify packets encrypted
- Wireshark: Capture UDP traffic, confirm it's not readable plaintext
