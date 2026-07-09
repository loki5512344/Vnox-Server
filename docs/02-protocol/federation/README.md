# Federation

> Status: Phase 3 — design draft, not yet implemented.
> This document describes the intended design. Details may change.

---

## Goal

Federation allows independent VNOX nodes to communicate with each other.
Users on one node can join channels, send messages, and voice-chat with
users on a different node — without either node being subordinate to the other.

There is no central federation server. Every node is equal.

---

## Identity in federation

Federated identity format:

```
user@node
```

Examples:
```
raven@nightcore.lnex
0xmist@dev.vnox.io
you@192.168.1.10
```

The `node` part is a resolvable address: domain, IP, or LNEx node ID.
The `user` part is the short form of the user's pubkey (or their chosen nickname,
disambiguated by pubkey if collision).

---

## What can be federated

| Feature | Status |
|---------|--------|
| Text chat bridging | Phase 3 |
| Voice relay across nodes | Phase 3 |
| Channel sharing | Phase 3 |
| Identity sync | Phase 3 |
| Permissions across nodes | Phase 4 |
| Encrypted federation | Phase 4 |

---

## Node discovery

Three strategies are planned. A node may use any combination.

### 1. Bootstrap list

A static list of known nodes in `config.toml`:

```toml
[federation]
bootstrap = [
  "relay.nightcore.lnex",
  "192.168.1.20:7700",
]
```

Simple and predictable. Suitable for private networks.

### 2. DNS SRV records

A node publishes its LNEx endpoint via DNS:

```
_lnex._udp.nightcore.lnex.  SRV  0 0 7700 nightcore.lnex.
```

Allows discovery via domain name without hardcoded IPs.

### 3. DHT (future)

For fully decentralized discovery without any bootstrap list or DNS.
Not planned until Phase 4.

---

## Federation routing

When a client on node A wants to reach node B:

```
Client (node A)
    │
    ▼
Gateway A  ──── resolves node B address (DNS / bootstrap)
    │
    ▼
LNEx Federation handshake (node A → node B)
    │  mutual auth: exchange pubkeys
    │  negotiate: shared channels, relay policy
    ▼
Established federation link
    │
    ├── text: messages forwarded via TCP federation channel
    └── voice: packets relayed via UDP federation relay path
```

### Federation link lifecycle

1. Node A initiates connection to node B
2. Both nodes authenticate using their node keypairs
3. Nodes exchange a list of channels available for bridging
4. A federation session is established with a session token
5. Session is kept alive with periodic PING/PONG
6. On disconnect, pending messages are queued for retry

---

## Voice relay across nodes

```
Client A (node A)
    │ UDP
    ▼
Voice Node A
    │ UDP (federation relay path)
    ▼
Voice Node B
    │ UDP
    ▼
Client B (node B)
```

Voice packets are forwarded between voice nodes using the same
LNEx UDP packet format. The federation relay path adds one hop
and ~1–10ms additional latency depending on network distance.

---

## Channel bridging

A channel can be bridged between two nodes. Bridged channels appear
in both node's channel lists. Messages and voice from either side
flow through the federation link.

```toml
# On node A config
[federation.bridges]
"#general" = "nightcore.lnex/#general"
```

---

## Anti-spam and rate limiting

Federation links are subject to rate limiting to prevent a rogue node
from flooding a target node.

Limits (configurable):
- max federation connections per node: 16
- max bridged channels: 64
- message rate per federation link: 100/s
- voice packet rate per relayed user: standard voice rate

A node can block or denylist other nodes in `config.toml`.

---

## Open questions (TODO)

- How to handle split-brain: node B is temporarily unreachable during a conversation
- Message history sync across federated nodes
- Permission model for federated users (what can a `user@external-node` do on your node)
- Trust levels: `trusted` / `untrusted` / `blocked` per federation link
