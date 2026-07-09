# Slint UI Migration — Phase 1.1

## Current state

Desktop client uses **egui** (immediate-mode GUI). Functional but:
- Grey, utilitarian aesthetic
- Limited design system
- Text-heavy, no modern chat app feel
- Difficult to style consistently

## Why Slint

| Criteria | egui | Slint | Iced |
|----------|------|-------|------|
| UI language | Rust code | Declarative DSL (`.slint`) | Elm-style |
| Design quality | Utilitarian | Design-oriented | Good |
| Performance | Fast | Fast | Moderate |
| Chat app fit | ❌ feel | ✅ modern | ✅ modern |
| Learning curve | Low | Medium | Medium |

**Recommendation:** Slint — declarative DSL makes UI easier to iterate,
built-in design system produces professional look out of the box.

## Migration strategy

### Stage 1 — Side-by-side (week 1)
- Embed Slint canvas alongside existing egui panels
- Rewrite sidebar (server list + channel list) in Slint first
- Keep chat area in egui
- Verify data flow works between both frameworks

### Stage 2 — Chat panel (week 2)
- Rewrite channel chat (text messages + input bar) in Slint
- Rewrite voice panel in Slint
- Remove egui chat dependency

### Stage 3 — Settings & polish (week 3)
- Rewrite settings window in Slint
- Add dark/light theme toggle
- Add accent color picker
- Apply consistent spacing/typography

## Example Slint structure

```slint
// client/src/ui/main.slint
export component MainWindow inherits Window {
    in-out property <[Channel]> channels;
    in-out property <[Message]> messages;
    in-out property <string> active-channel;
    
    HorizontalLayout {
        ServerSidebar {}
        ChatPanel {}
        VoicePanel {}
    }
}

component ServerSidebar {
    VerticalLayout {
        Text { text: "VNOX"; }
        ListView {
            for ch in channels: ChannelRow {
                text: ch.name;
                icon: ch.kind == "voice" ? "~" : "#";
            }
        }
        UserBar {}
    }
}
```

## Data binding

Use Slint's `in-out property` bindings to sync with Rust state:

```rust
// client/src/ui/app.rs
use slint::ComponentHandle;

let ui = MainWindow::new()?;
ui.set_channels(slint::ModelRc::from(models));
ui.on_send_message(|content| {
    // forward to net layer
});
ui.run()?;
```

## File structure after migration

```
client/src/ui/
├── app.rs              # entry, wiring
├── main.slint          # main layout
├── sidebar.slint       # server sidebar + channel list
├── chat.slint          # message list + input bar
├── voice.slint         # voice panel
├── settings.slint      # settings window
├── theme.slint         # color/font definitions
└── state/              # UiState (stays in Rust)
```

## Open questions

- Slint's rendering backend (gl/winit/software) — test on all target platforms
- Slint's text input / rich text support for chat messages (emoji, markdown?)
- Whether to remove egui entirely or keep for specific panels (e.g. overlay)
- Licensing: Slint is LGPL (compatible with GPL-3.0 for now, but check for future)
