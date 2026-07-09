# Security

> **Phase 1.1 status.** TCP traffic is encrypted with ChaCha20-Poly1305 AEAD.
> UDP voice path is still plaintext (Phase 2). See
> [features/encryption.md](../05-features/encryption.md) for details.

## Threat model

VNOX is designed for self-hosted deployments where the node operator is trusted.
The threat model covers:

| Threat | Mitigated by |
|--------|--------------|
| MITM on client-server connection | LNEx packet encryption (ChaCha20-Poly1305) active on TCP |
| MITM on UDP voice path | LNEx packet encryption (Phase 2; not active in v0.1.x) |
| Rogue node in federation | Mutual keypair authentication on federation handshake (Phase 3) |
| Identity spoofing | Ed25519 signature on every auth (challenge-response) |
| Metadata leakage (who talks to whom) | Phase 2 |
| Passive voice interception | Packet encryption (Phase 2; not active in v0.1.x) |
| Replay attacks | Sequence number + nonce per packet (Phase 2 wire encryption) |
| Brute-force on session | Sessions are short-lived, token is not a password |

### Out of scope

- Physical access to the server
- Compromised node operator (they own the node, this is by design)
- E2EE between clients (planned Phase 2, not in v1)
- Anonymity / traffic analysis resistance

---

## Encryption

> **Phase 1.1 target.** See [features/encryption.md](../05-features/encryption.md) for the implementation plan.
> Phase 1 uses JSON over plain TCP and raw Opus over UDP.

### LNEx packet encryption

All LNEx packets (both TCP and UDP paths) are encrypted at the LNEx layer.

Algorithm: **ChaCha20-Poly1305** (AEAD)

- ChaCha20 for stream cipher
- Poly1305 for authentication tag (16 bytes)
- 96-bit nonce, derived from: `session_id || sequence`
- Key derived from ECDH exchange during handshake (X25519)

ChaCha20-Poly1305 is chosen over AES-GCM because:
- constant-time on all platforms (no hardware AES requirement)
- faster in software on hardware without AES-NI (common on ARM)
- simpler nonce management

### Key exchange

During the LNEx handshake:

1. Server sends its ephemeral X25519 public key in HELLO
2. Client generates its own ephemeral X25519 keypair
3. Both compute the shared secret via X25519 ECDH
4. Shared secret is passed through HKDF-SHA256 to derive:
   - client → server encryption key
   - server → client encryption key

Ephemeral keys are discarded after the session. This provides
**forward secrecy** — compromising the long-term identity keypair
does not expose past sessions.

### TCP transport

TCP connections additionally use TLS 1.3.
LNEx packet encryption runs inside TLS — defense in depth.

TLS certificate: self-signed by default, pinned on first connect (TOFU).
Node operators may configure a proper CA-signed certificate.

### UDP transport

UDP has no TLS. LNEx packet encryption (ChaCha20-Poly1305) is the only
protection layer on the voice path. This is standard practice for
real-time voice protocols (SRTP, DTLS-SRTP follow the same model).

---

## E2EE (Phase 2)

In v1, encryption is between client and server (node). The node operator
can theoretically decrypt voice and text in transit.

Phase 2 will introduce optional end-to-end encryption for:
- direct messages
- private channels (opt-in)

E2EE for voice is significantly harder (requires key distribution to all
channel members in real time) and is a Phase 4 design question.

---

## Identity verification

When user A sees a message from `raven@nightcore.lnex`, how do they know
it's the same raven they spoke to yesterday?

In v1: the gateway enforces that a connected user's pubkey matches their
asserted identity. The client can verify the server's claim by checking
the pubkey shown in the UI against a known value.

Future: out-of-band key verification (QR code, safety number, similar to Signal).

---

## Rate limiting and anti-flood

Applied at the gateway level:

| Limit | Default | Configurable |
|-------|---------|-------------|
| Auth attempts per IP | 5 / minute | yes |
| Messages per user per second | 10 | yes |
| Voice packet rate per user | ~50/s (20ms frames) | no (codec-determined) |
| Federation connection attempts | 3 / minute per remote | yes |
| Max concurrent connections per IP | 4 | yes |

Exceeding a rate limit returns `ERR_RATE_LIMITED` and may trigger a
temporary ban depending on node configuration.

---

## Node operator notes

### What the operator can see

- IP addresses of connected clients
- usernames (nicknames) and pubkeys
- channel activity (who joined when)
- message content (in v1, no E2EE)

### What the operator cannot do (by design)

- impersonate a user's identity (requires their private key)
- forge signatures on behalf of a user

### Recommended hardening

- run gateway behind a reverse proxy (nginx / caddy) for TLS termination
- restrict UDP port to known IP ranges if possible
- enable fail2ban or equivalent on auth failure logs
- back up the node keypair (used for federation identity)
- rotate node keypair on suspected compromise

See `03-server/operations.md` for hardening checklist.
