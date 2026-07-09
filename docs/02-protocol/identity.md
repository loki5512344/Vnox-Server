# Identity

> **Phase 1 note:** the client stores the keypair as plain JSON in the data directory.
> Passphrase encryption (Argon2id), seed phrase UI, and keyfile export are Phase 2.

## Model

VNOX has no accounts. No email. No phone number. No central registry.

On first launch, the client generates a keypair locally:

```json
{
  "id": "4f3a8b2c9d1e7a0f3b5c8d2e1a9f4b7c2d8e3f1a",
  "nickname": "user",
  "pubkey": "ed25519:4f3a8b2c...",
  "created_at": 1716000000
}
```

The private key never leaves the device (unless explicitly exported by the user).
The public key is the identity. Everything — permissions, history, session tokens —
is tied to the pubkey.

---

## Keypair

Algorithm: Ed25519

- fast signature verification
- small key size (32 bytes public, 64 bytes private)
- widely supported in Rust ecosystem (dalek-cryptography)

The keypair is stored locally in the client's data directory.

**Phase 1 (current):** plain JSON file (`identity.json`).

**Phase 2 (planned):** encrypted at rest with a user-chosen passphrase (Argon2id KDF).

---

## Auth flow

```
Client                              Gateway
  │                                    │
  │── TCP connect ────────────────────▶│
  │                                    │
  │◀── HELLO { server_pubkey,          │
  │            challenge_nonce } ──────│
  │                                    │
  │── AUTH {                           │
  │     client_pubkey,                 │
  │     nickname,                      │
  │     lnex_version,                  │
  │     sig: sign(challenge_nonce,     │
  │               client_privkey)      │
  │   } ──────────────────────────────▶│
  │                                    │  verify signature
  │                                    │  check if banned
  │◀── SESSION {                       │
  │     session_id,                    │
  │     token,                         │
  │     expires_at                     │
  │   } ──────────────────────────────│
  │                                    │
  │   [authenticated]                  │
```

The gateway verifies the signature against the challenge nonce.
If valid, a session token is issued. The token is used for subsequent
requests within the TCP session.

The gateway does not store private keys. It only stores public keys
of users who have connected (for banning and permission assignment).

---

## Session

Sessions are in-memory on the gateway. They expire when the TCP connection closes
or after a configurable idle timeout.

There is no persistent login. Each connection requires a fresh auth exchange.
The session token is not a password — it proves nothing without the underlying
keypair.

---

## Nickname

Nickname is a human-readable label, not unique. Two users may have the same nickname.
The pubkey (or its short form) is the unique identifier.

Nickname is self-asserted and can be changed at any time.
Servers may impose length or character limits.

---

## Permissions

Permissions are assigned to pubkeys on each node independently.
There is no global permission registry.

Permission levels (example model — configurable per node):

```
guest       — read channels, no voice
member      — read + write + voice
moderator   — member + kick + mute others + manage channels
admin       — full control of the node
owner       — same as admin, cannot be demoted by admins
```

A user's permission level on node A has no bearing on their level on node B.
Federated permission model is a Phase 4 design question.

---

## Keypair backup

> **Phase 2.** UI and export flows below are not implemented in v0.1.x.

If the keypair is lost, the identity is lost. There is no recovery
without a backup. This is intentional. There is no central authority
to reset your account.

Two backup methods are planned:

### Seed phrase

A 24-word BIP39-compatible mnemonic derived from the private key entropy.

```
word1 word2 word3 ... word24
```

The seed phrase can regenerate the keypair deterministically.
Store it offline. Never share it.

To enable: Settings → Identity → Seed phrase backup → Show phrase.

### Encrypted keyfile

Export the keypair as a `.vnox` file, encrypted with a passphrase
(AES-256-GCM, passphrase stretched with Argon2id).

```
Settings → Identity → Export keypair → Save as .vnox
```

To restore: launch client → Import keypair → select .vnox file → enter passphrase.

---

## Rotating the keypair

If a keypair is compromised, the user can generate a new one.
The old keypair becomes inactive immediately.

Effects:
- new identity on all nodes (the old pubkey is a different person)
- permissions tied to old pubkey remain on the server (admins can clean up)
- history attributed to old pubkey is not migrated

Rotation is permanent and cannot be undone.

`Settings → Identity → Rotate keypair → Confirm`
