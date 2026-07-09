# Slint UI Migration — Design Spec

**Date:** 2026-05-31
**Status:** Draft
**Target:** Phase 1.1 — Big-bang rewrite of egui → Slint

## Overview

Replace the existing egui (eframe + wgpu) desktop client with a Slint-based UI.
The HTML design at `vnox-ui.html` serves as the pixel-accurate visual reference:
Discord-like dark theme, orange accent (#ff6b35), bubble-style messages,
voice overlay, settings modal, profile card, connect screen.

**Migration approach:** Big-bang. One focused branch, no side-by-side egui/Slint hybrid.
All Slint components are built from scratch following the HTML reference.

## Architecture

### Render stack

```
Slint (winit + FemtoVG backend)
├── .slint files (declarative UI)
└── Rust glue (app.rs, state.rs, callbacks)
```

- Backend: `winit` + `FemtoVG` (default, no extra deps)
- No Qt, no external rendering libs
- Slint version: latest stable (2.x)

### Component tree

```
Window
├── if (connection-state == "disconnected"): ConnectScreen
├── if (connection-state == "connected"):   MainScreen
│   ├── Rail                            (левая панель иконок серверов)
│   ├── ChannelsPanel                   (поиск + список каналов)
│   │   ├── ServerHeader
│   │   ├── SearchInput
│   │   ├── ChannelSection[]            (text / voice)
│   │   │   └── ChannelRow[]
│   │   └── VoiceUserList[]
│   ├── UserBar                         (аватар + имя + кнопки)
│   ├── ChatArea                        (основная область)
│   │   ├── ChatHeader
│   │   ├── MessageList
│   │   │   ├── DaySeparator
│   │   │   ├── SystemMessage
│   │   │   └── MessageGroup[]
│   │   │       └── MessageBubble[]
│   │   └── InputArea
│   └── MembersPanel                    (участники)
│       └── MemberSection[]
│           └── MemberRow[]
├── if (show-voice):   VoiceOverlay
├── if (show-settings): SettingsOverlay
│   ├── SettingsHeader
│   ├── TabNav
│   ├── TabAudio
│   ├── TabIdentity (inline)
│   ├── TabAppearance (inline)
│   ├── TabNetwork (inline)
│   └── TabAdvanced (inline)
└── if (show-profile): ProfileCard
```

### State management

**Rust owns the state.** `UiState` struct stays in Rust. Slint reads it
through `in-out property` bindings updated on a 16ms timer (60 fps),
and writes back through `callback → Rust handler → NetCommand` channel.

```
Rust (UiState) ──[16ms timer sync]──▶ Slint properties
Slint (callback) ──[on_* handlers]──▶ Rust (event → net layer)
```

Key callbacks:
- `send-message(string)` → net:SendText
- `join-voice(string)` → net:JoinVoice
- `toggle-mute()` → UiState.mic_enabled
- `open-settings()`, `close-settings()`
- `show-profile(string)`, `hide-profile()`

## File structure

Every `.slint` / `.rs` file stays ≤ 200 lines.

```
client/src/
├── main.rs                          // entry point, slint Window::run()
├── ui/
│   ├── app.rs                       // setup, sync timer, callback wiring
│   ├── main.slint                   // root Window component
│   ├── state.rs                     // UiState (unchanged core)
│   │
│   ├── components/                  // reusable UI primitives
│   │   ├── avatar.slint
│   │   ├── toggle.slint
│   │   ├── badge.slint
│   │   ├── icon_button.slint
│   │   ├── slider.slint
│   │   └── level_bar.slint
│   │
│   ├── theme/
│   │   ├── palette.slint            // colors from HTML :root
│   │   ├── typography.slint         // IBM Plex Sans/Mono
│   │   └── spacing.slint            // radii, paddings
│   │
│   ├── connect/
│   │   ├── connect.slint            // connect screen
│   │   └── connect.rs               // connect/disconnect logic
│   │
│   ├── main_area/
│   │   ├── main_area.slint          // grid layout of main window
│   │   ├── rail.slint               // server icon rail
│   │   ├── sidebar.slint            // channel list + search
│   │   ├── userbar.slint            // bottom user bar
│   │   ├── members.slint            // right members panel
│   │   └── chat/
│   │       ├── chat_header.slint
│   │       ├── message_list.slint
│   │       ├── message_group.slint
│   │       ├── message_bubble.slint
│   │       └── input_area.slint
│   │
│   ├── voice_overlay/
│   │   ├── voice_overlay.slint
│   │   └── voice_overlay.rs
│   │
│   ├── settings/
│   │   ├── settings.slint           // modal + all tabs except audio
│   │   └── settings_audio.slint     // audio tab only
│   │
│   └── profile_card/
│       ├── profile_card.slint
│       └── profile_card.rs
```

Total: ~30 .slint files, ~10 .rs files.

## Screen specifications

### 1. ConnectScreen (`connect/`)
- Centered card, max 340px wide
- Logo block (VNOX icon + text), tagline "secure voice & text · quic/v1"
- Three inputs: Address, Username, Password
- Two buttons: Connect (primary, accent), Keypair (ghost)
- Recent servers section below with ping
- Background: dark with radial gradient glow

### 2. MainScreen — Rail (`main_area/rail.slint`)
- Fixed 56px wide, full height
- Logo icon (36px, rounded, accent bg, tooltip "VNOX")
- Thin separator
- Server icons (36px circles, 2-letter initials, active state with left highlight bar)
- Dashed "+" button (add server, fixed to bottom)
- Monospace "VNOX" label at very bottom
- Tooltips on hover via [data-tip] pattern

### 3. MainScreen — ChannelsSidebar (`main_area/sidebar.slint`)
- ServerHeader: name + lnex:// address in monospace
- SearchInput: icon + field, dark bg, thin border
- Collapsible sections: TEXT, VOICE
- ChannelRow: icon (# / ▶) + name + optional badge (ping/green, unread/orange, count)
- VoiceUserRow inside voice channels: avatar, name, speaking dot
- Active channel: left accent bar + highlighted bg

### 4. MainScreen — UserBar (`main_area/userbar.slint`)
- 52px height, border-top + border-right
- Avatar 30px + status dot
- Username + RTT/loss stats in monospace
- Three icon buttons: Mute (toggle red), Deafen (toggle), Settings

### 5. MainScreen — ChatArea (`main_area/chat/`)
- ChatHeader: icon + name + separator + description + action buttons (pin, search, toggle-members)
- MessageList (scrollable, flex)
  - DaySeparator: "── today ──" style
  - SystemMessage: "connected · server" centered
  - MessageGroup: avatar 34px + bubble (bg2, border, radius 4/12/12/12)
    - Header: author name (colored, clickable for profile) + time
    - Content: text, word-break
    - Continuation messages: same group, inline style (margin-left 46px)
  - Hover actions bar above group: reply, react, copy
- InputArea: bg2 box with focus accent border
  - Top row: #channel-name prefix + text input
  - Bottom row: Attach, Emoji buttons, then right-aligned E2E badge + Send button

### 6. MainScreen — MembersPanel (`main_area/members.slint`)
- 252px wide, border-left
- Header: "УЧАСТНИКИ — N"
- Sections by role: Admin, Moderator, Online, Offline
- MemberRow: avatar 30px + status dot + name + role tag
- Offline members at 35% opacity

### 7. VoiceOverlay (`voice_overlay/`)
- Full-screen overlay, dark backdrop with blur
- Header: channel name, quality indicator, RTT, codec, loss
- Main speaker: large card (500px), avatar 88px, name, status badges
- Active speaker: green border glow on card + avatar border
- Secondary speakers: small cards row, avatar 48px, name, mute/deafen icons
- Control bar: mic toggle, headphones, screen share, camera, disconnect (accent)

### 8. SettingsOverlay (`settings/`)
- Modal 700px, centered, dark backdrop with blur
- Header: logo, node name + status dot, badge, close button
- Tab navigation bar: Audio, Identity, Appearance, Network, Advanced
- Audio tab: input device, gain slider + level bar, noise gate toggle, RNNoise toggle, codec select, bitrate select, FEC toggle, DTX toggle, output device, volume slider + level bar
- Identity tab: username input, status select, public key, fingerprint
- Appearance tab: theme select, accent color picker, compact toggle
- Network tab: protocol select, RTT, packet loss, jitter metrics
- Advanced tab: packet stats toggle, verbose logging toggle, version badge, runtime info

### 9. ProfileCard (`profile_card/`)
- 300px card, centered overlay
- Banner 72px gradient, avatar 56px overlapping
- Name + tag (#0001 · server)
- Role with colored badge
- Info rows: status, RTT, joined date, encryption
- Action buttons: Message (primary), Voice Invite, Mute

## Theme

All CSS variables from `vnox-ui.html` converted to Slint `property`:

```slint
export global Palette {
    in-out property <color> bg0: #0c0c0c;
    in-out property <color> bg1: #111111;
    in-out property <color> bg2: #181818;
    in-out property <color> bg3: #202020;
    in-out property <color> bg4: #2a2a2a;
    in-out property <color> accent: #ff6b35;
    in-out property <color> accent-hover: #e85d28;
    in-out property <color> border: #222222;
    in-out property <color> border2: #2e2e2e;
    in-out property <color> text1: #f0f2f5;
    in-out property <color> text2: #9ca3af;
    in-out property <color> text3: #4a5568;
    in-out property <color> green: #4ade80;
    in-out property <color> red: #f87171;
    in-out property <color> yellow: #fbbf24;
    in-out property <color> teal: #5bbf9f;
}
```

Accent color is user-configurable via the Appearance tab (color picker input).

Avatar color palette (8 colours, indexed by hash of user ID):
`#c0522a`, `#7a52c4`, `#3a9e5f`, `#c43a7a`, `#3a7ac4`, `#9e7a3a`, `#3a9e9e`, `#8a3ac4`

## Dependencies

**Removed:**
- `eframe = "0.33"` (and transitive egui/wgpu deps)

**Added:**
- `slint = "2.x"` (winit + FemtoVG backend)

Everything else (tokio, serde, crypto, opus, cpal, rodio, nnnoiseless) stays.

## Migration steps

1. Create new `client/src/ui/` structure with Slint files
2. Wire `app.rs` with `slint::Window::run()` replacing `eframe::run_native()`
3. Implement `theme/` — palette, typography, spacing
4. Implement `components/` — avatar, toggle, badge, icon_button, slider, level_bar
5. Implement `connect/` — connect screen (independent, good first block)
6. Implement `main_area/` — rail, sidebar, userbar, chat, members
7. Implement `voice_overlay/` — full voice UI
8. Implement `settings/` — modal with all tabs
9. Implement `profile_card/`
10. Wire all callbacks → existing net/audio layers
11. Remove old egui code, clean up `Cargo.toml`
12. Test build, fix clippy, cargo fmt
