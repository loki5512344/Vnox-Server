# Overlay

> Status: Phase 2 — design draft.

The VNOX overlay renders a HUD on top of running games and applications,
showing voice channel state without alt-tabbing.

---

## What it shows

- who is currently speaking (avatar / nickname + audio indicator)
- your own mic state (active / muted / push-to-talk held)
- current channel name
- latency (RTT to node)
- hotkey state hints

---

## Architecture

The overlay is a separate process that communicates with the main VNOX client
via a local IPC socket (Unix socket / named pipe).

The client pushes state updates to the overlay:
- user speaking events (`VOICE_STATE`)
- channel changes
- mute state changes

The overlay renders on top of other applications.

### Rendering approach

| Platform | Method |
|----------|--------|
| Windows | DirectX overlay injection or transparent top-level window |
| Linux (X11) | Shaped transparent window, always-on-top |
| Linux (Wayland) | Layer shell protocol (wlr-layer-shell) |
| macOS | CGWindow overlay |

Implementation complexity varies significantly by platform.
Windows DX injection is the most reliable for full-screen games.

---

## Game integrations

For games that support it, the overlay can receive additional data:

### Positional voice (Phase 4)

Games that expose player position data can send it to VNOX,
enabling positional audio — players hear each other based on
in-game distance and direction.

Integration methods:

| Game / Engine | Method |
|---------------|--------|
| Minecraft | Fabric/Forge mod that sends position via local socket |
| Source Engine | Client plugin / VScript |
| Unreal Engine | Plugin exposing position to named pipe |
| Unity | SDK that writes position to shared memory |

Protocol for positional data is TBD (Phase 4 design).

### Speaking indicators in-game

Some games support custom HUD elements. Where possible, the overlay
will render directly within the game's UI rather than as an external window.

---

## Configuration

```toml
[overlay]
enabled = true

# Overlay position on screen
position = "top-right"   # top-left | top-right | bottom-left | bottom-right

# Opacity (0.0–1.0)
opacity = 0.85

# Show latency
show_latency = false

# Hotkey to toggle overlay visibility
toggle_hotkey = "ctrl+shift+o"
```

---

## Hotkeys

All hotkeys are global (work even when VNOX window is not focused).

| Action | Default |
|--------|---------|
| Push-to-talk | `mouse4` |
| Mute toggle | `ctrl+m` |
| Deafen toggle | `ctrl+d` |
| Toggle overlay | `ctrl+shift+o` |

Configurable in Settings → Keybinds.
