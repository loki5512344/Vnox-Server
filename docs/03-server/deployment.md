# Deployment

> **Phase 1 note:** published container images and HTTP health checks are not available yet.
> For local development, build from source and use [dev/README.md](../../dev/README.md).

## Option A - Docker (when images are published)

The fastest way to get a node running once official images exist.

### Prerequisites

- Docker 24+
- Docker Compose v2

### docker-compose.yml

```yaml
version: "3.9"

services:
  gateway:
    # Build locally until ghcr.io/vnox images are published:
    # build: { context: ../.., dockerfile: docker/gateway.Dockerfile }
    image: ghcr.io/vnox/gateway:latest
    restart: unless-stopped
    ports:
      - "7600:7600"         # TCP — client connections
    volumes:
      - ./config.toml:/etc/vnox/config.toml:ro
      - vnox-data:/var/lib/vnox
    environment:
      - VNOX_LOG=info

  voice-node:
    image: ghcr.io/vnox/voice-node:latest
    restart: unless-stopped
    ports:
      - "7700:7700/udp"     # UDP — voice packets
    volumes:
      - ./config.toml:/etc/vnox/config.toml:ro
    environment:
      - VNOX_LOG=info

volumes:
  vnox-data:
```

### Start

```bash
docker compose up -d
docker compose logs -f
```

### Verify

```bash
# Phase 1: no /health endpoint yet. Check TCP instead:
nc -zv localhost 7600

# Or watch gateway logs after start:
# VNOX_LOG=info cargo run -p vnox-gateway -- --config dev/config.toml
```

When an HTTP health endpoint ships (Phase 2), it may look like:

```bash
curl http://localhost:7600/health
# {"status":"ok","version":"LNEx v1","node":"your-node-name"}
```

---

## Option B - systemd (bare binary)

For servers where Docker is not available or not desired.

### Download

```bash
# Replace VERSION with the latest release tag
VERSION=0.1.0
curl -L https://github.com/vnox/vnox/releases/download/v${VERSION}/vnox-linux-x86_64.tar.gz \
  | tar xz -C /usr/local/bin/
```

Binaries installed:
- `/usr/local/bin/vnox-gateway`
- `/usr/local/bin/vnox-voice-node`

### Config

```bash
mkdir -p /etc/vnox /var/lib/vnox
cp config.example.toml /etc/vnox/config.toml
$EDITOR /etc/vnox/config.toml
```

### systemd units

`/etc/systemd/system/vnox-gateway.service`:

```ini
[Unit]
Description=VNOX Gateway
After=network.target

[Service]
ExecStart=/usr/local/bin/vnox-gateway --config /etc/vnox/config.toml
Restart=on-failure
RestartSec=5s
User=vnox
Group=vnox
StateDirectory=vnox
RuntimeDirectory=vnox

[Install]
WantedBy=multi-user.target
```

`/etc/systemd/system/vnox-voice-node.service`:

```ini
[Unit]
Description=VNOX Voice Node
After=network.target vnox-gateway.service

[Service]
ExecStart=/usr/local/bin/vnox-voice-node --config /etc/vnox/config.toml
Restart=on-failure
RestartSec=5s
User=vnox
Group=vnox

[Install]
WantedBy=multi-user.target
```

### Enable and start

```bash
# Create service user
useradd -r -s /sbin/nologin vnox

# Enable services
systemctl daemon-reload
systemctl enable --now vnox-gateway vnox-voice-node

# Check status
systemctl status vnox-gateway
systemctl status vnox-voice-node
```

---

## Option C - bare binary (dev / test)

For local development and testing without systemd, see [dev/README.md](../../dev/README.md).

```bash
# Terminal 1 - gateway
cargo run -p vnox-gateway -- --config dev/config.toml

# Terminal 2 - voice node
cargo run -p vnox-voice-node -- --config dev/config.toml
```

---

## Minimal config.toml

The minimum required configuration to get a node running:

```toml
[node]
name    = "my-node"           # displayed to clients
address = "my-node.example.com"  # public address (domain or IP)

[gateway]
bind = "0.0.0.0:7600"

[voice]
bind = "0.0.0.0:7700"

[identity]
# Generated on first start if not present
# keypair_path = "/var/lib/vnox/node.key"
```

See [configuration.md](configuration.md) for the full reference.

---

## Firewall

Open the following ports:

```bash
# TCP — client connections
ufw allow 7600/tcp

# UDP — voice packets
ufw allow 7700/udp
```

If running behind a reverse proxy (nginx / caddy), only expose port 443 externally
and proxy to 7600 internally. UDP 7700 must be directly accessible.

---

## First connection test

1. Download the VNOX client
2. Open VNOX → Add node → enter your server address
3. The client will generate an identity on first launch
4. Connect — you should see your node's channels

If connection fails, check:
- gateway is running: `systemctl status vnox-gateway`
- port 7600 is open: `nc -zv your-server 7600`
- logs: `journalctl -u vnox-gateway -f`
