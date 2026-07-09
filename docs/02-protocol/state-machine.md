# State Machine

> **Phase 1 note:** the gateway and voice-node do not share membership state.
> The voice node registers a client on the first UDP packet and drops idle addresses
> after 30 seconds. Diagrams that show `gateway notify voice node` describe the Phase 2 target.

Client and server maintain synchronized state machines over the LNEx connection.
This document describes the states, transitions, and what triggers them.

---

## Client states

```
                    ┌─────────────────┐
                    │   DISCONNECTED  │ ◄─── initial state / after disconnect
                    └────────┬────────┘
                             │ connect(address)
                             ▼
                    ┌─────────────────┐
                    │   CONNECTING    │ TCP handshake in progress
                    └────────┬────────┘
                             │ TCP established
                             ▼
                    ┌─────────────────┐
                    │   HANDSHAKING   │ HELLO received, sending AUTH
                    └────────┬────────┘
                             │ SESSION received
                             ▼
                    ┌─────────────────┐
                    │   CONNECTED     │ ◄─── idle, no channel joined
                    └────────┬────────┘
                             │ join_channel()
                             ▼
                    ┌─────────────────┐
              ┌────►│  IN_CHANNEL     │ text + voice available
              │     └────────┬────────┘
              │              │ leave_channel() / switch channel
              └──────────────┘
                             │ disconnect() / TCP drop
                             ▼
                    ┌─────────────────┐
                    │  RECONNECTING   │ exponential backoff (Phase 2)
                    └────────┬────────┘
                             │ max retries exceeded
                             ▼
                    ┌─────────────────┐
                    │  DISCONNECTED   │
                    └─────────────────┘
```

Any state can transition to `DISCONNECTED` on:
- TCP connection drop
- `DISCONNECT` packet received
- auth failure (`ERR_AUTH_FAILED`)
- version mismatch (`ERR_VERSION_MISMATCH`)

---

## Server session states

Per-client session on the gateway:

```
                    ┌─────────────────┐
                    │    ACCEPTING    │ TCP connection received, sending HELLO
                    └────────┬────────┘
                             │ AUTH received
                             ▼
                    ┌─────────────────┐
                    │  AUTHENTICATING │ verifying Ed25519 signature
                    └────────┬────────┘
                             │ signature valid + not banned
                             ▼
                    ┌─────────────────┐
                    │   ESTABLISHED   │ SESSION sent, client is authenticated
                    └────────┬────────┘
                             │ JOIN_CHANNEL received
                             ▼
                    ┌─────────────────┐
              ┌────►│  IN_CHANNEL     │ forwarding messages + voice state
              │     └────────┬────────┘
              │              │ LEAVE_CHANNEL / JOIN different channel
              └──────────────┘
                             │ TCP drop / DISCONNECT / idle timeout
                             ▼
                    ┌─────────────────┐
                    │   TERMINATED    │ session cleaned up, resources freed
                    └─────────────────┘
```

### Auth failure paths

```
AUTHENTICATING
    │ invalid signature      → send ERR_AUTH_FAILED → TERMINATED
    │ banned pubkey          → send ERR_AUTH_FAILED → TERMINATED
    │ unsupported version    → send ERR_VERSION_MISMATCH → TERMINATED
    │ rate limit on auth     → send ERR_RATE_LIMITED → TERMINATED
```

---

## Voice session states (per user, on voice-node)

> **Phase 1:** transition from `INACTIVE` to `JOINING` happens when the first UDP
> packet arrives from a client address, not when the gateway sends a signal.

```
                    ┌─────────────────┐
                    │    INACTIVE     │ user not in a voice channel
                    └────────┬────────┘
                             │ first UDP packet from client (Phase 1)
                             │ gateway signals join (Phase 2, not implemented)
                             ▼
                    ┌─────────────────┐
                    │    JOINING      │ UDP path being established
                    └────────┬────────┘
                             │ first UDP packet received from client
                             ▼
                    ┌─────────────────┐
                    │    SILENT       │ in channel, not transmitting
                    └────────┬────────┘
                             │ voice packets arriving
                             ▼
                    ┌─────────────────┐
              ┌────►│    SPEAKING     │ relaying to other channel members
              │     └────────┬────────┘
              │              │ DTX / push-to-talk released / silence
              └──────────────┘
                    ┌─────────────────┐
                    │     MUTED       │ server-side mute (moderator action)
                    └────────┬────────┘
                             │ unmuted by moderator
                             ▼
                    ┌─────────────────┐
                    │    SILENT       │
                    └─────────────────┘
```

Voice state is broadcast to all channel members as `VOICE_STATE` packets
on every transition: `SILENT → SPEAKING`, `SPEAKING → SILENT`, `MUTED`, `DEAFENED`.

---

## Channel join sequence (detailed)

Full flow from `join_channel()` call to receiving audio:

```
Client                    Gateway                   Voice Node
  │                          │                          │
  │── JOIN_CHANNEL ─────────►│                          │
  │                          │ check permissions         │
  │                          │ check channel exists      │
  │◄─ CHANNEL_STATE ─────────│                          │
  │   { members,             │                          │
  │     voice_endpoint,      │   (Phase 2: gateway      │
  │     channel_id }         │    notifies voice node)  │
  │                          │                          │
  │   [start UDP]            │                          │
  │── UDP VOICE_PACKET ──────────────────────────────── ►│
  │                          │                          │ relay to others
  │◄─ UDP VOICE_PACKET ───────────────────────────────── │
  │                          │                          │
  │◄─ USER_JOIN broadcast ───│                          │
  │   (sent to all members)  │                          │
```

If `ERR_PERMISSION_DENIED` is returned on `JOIN_CHANNEL`, the client
stays in `CONNECTED` state and does not attempt UDP.

---

## Reconnect logic (Phase 2)

On unexpected TCP drop from `IN_CHANNEL` or `CONNECTED`:

```
disconnect detected
    │
    ▼
RECONNECTING state
    │
    ├── attempt 1: wait 1s
    ├── attempt 2: wait 2s
    ├── attempt 3: wait 4s
    ├── attempt 4: wait 8s
    ├── attempt 5: wait 16s
    └── attempt 6+: wait 30s (cap)
```

On successful reconnect:
- full auth exchange (new session token)
- if user was `IN_CHANNEL` before disconnect: auto-rejoin same channel

On reconnect failure after N attempts (configurable, default 10):
- transition to `DISCONNECTED`
- notify user in UI

---

## Packet validity by state

Packets received in an unexpected state are dropped with `ERR_INVALID_PACKET`.

| Packet | Valid in states |
|--------|----------------|
| `HELLO` | server sends in `ACCEPTING` |
| `AUTH` | client sends in `HANDSHAKING` |
| `SESSION` | server sends in `AUTHENTICATING` |
| `JOIN_CHANNEL` | `CONNECTED`, `IN_CHANNEL` |
| `LEAVE_CHANNEL` | `IN_CHANNEL` |
| `CHAT_MESSAGE` | `IN_CHANNEL` |
| `VOICE_PACKET` | UDP, user in voice channel |
| `PING` / `PONG` | any authenticated state |
| `DISCONNECT` | any state |
