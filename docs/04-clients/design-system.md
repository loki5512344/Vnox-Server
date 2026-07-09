# VNOX Client — Design System

> Based on Hi-Fi Minimalism v2.0
> Adapted for: native desktop voice/chat client (Rust + egui + wgpu)

---

## Philosophy

VNOX UI должен ощущаться как инструмент, а не социальная сеть.

```
calm · focused · fast · warm · precise
```

Не:
```
gamer RGB · Discord clone · neon cyberpunk · glassmorphism · corporate SaaS
```

> The client is infrastructure. The UI is just the control surface.

---

## Color Tokens

```css
/* Backgrounds — layered, never pure black */
--bg-base:        #0d0d0d;   /* root, titlebar */
--bg-surface:     #151515;   /* panels, sidebars  (was #111111 — lifted for panel contrast) */
--bg-elevated:    #1d1d1d;   /* inputs, cards     (was #141414 — lifted for depth) */
--bg-interactive: #2a2a2a;   /* hover targets, dropdowns */

/* Borders — lifted significantly for visual hierarchy */
--border-subtle:  #1c1c1c;   /* was #171717 */
--border-default: #262626;   /* was #1e1e1e — now actually visible */
--border-strong:  #2e2e2e;
--border-accent:  rgba(255, 107, 53, 0.20);

/* Accent — warm orange */
--accent:         #ff6b35;
--accent-hover:   #ff844f;
--accent-active:  #e85d04;
--accent-10:      rgba(255, 107, 53, 0.08);
--accent-20:      rgba(255, 107, 53, 0.15);

/* Text — warm, never pure white */
--text-primary:   #f4c89a;   /* main content (added) */
--text-secondary: #c88b5a;   /* names, labels */
--text-muted:     #8a6a52;   /* secondary info */
--text-dim:       #6a4a2a;   /* hints, timestamps */
--text-ghost:     #5a422c;   /* section labels, separators  (was #3d2e20 — lifted) */
--text-invisible: #443222;   /* near-invisible, decorative  (was #2d1e12 — lifted) */

/* Semantic */
--success:        #7cb87a;   /* online, connected, low latency */
--success-muted:  rgba(124, 184, 122, 0.10);
--warning:        #e6a230;   /* ping indicator, unread */
--warning-muted:  rgba(230, 162, 48, 0.10);
--error:          #d9604a;   /* muted mic (when needed), errors */
--error-muted:    rgba(217, 96, 74, 0.10);
--info:           #6a9ecf;   /* second user accent color */
--info-muted:     rgba(106, 158, 207, 0.10);
```

---

## Typography

Клиент использует **только monospace**. Это намеренно — усиливает ощущение инструмента.

```
Primary font: IBM Plex Mono
Fallback:     Fira Code, Geist Mono, monospace
```

### Scale (клиент-специфичный, компактный)

| Role | Size | Weight | Color |
|------|------|--------|-------|
| Section label | 10px | 400 | `--text-ghost` |
| Timestamp, ID | 9px | 400 | `--text-invisible` |
| Status, badge | 9px | 500 | semantic |
| Channel name | 12px | 400 | `--text-ghost` → `--text-secondary` |
| Message text | 11px | 400 | `--text-muted` |
| Username | 11px | 600 | varies per user |
| Node name | 13px | 600 | `--text-secondary` |
| Settings title | 13px | 600 | `--text-secondary` |
| Wordmark VNOX | 11px | 600 | letter-spacing: 0.2em |
| Panel title | 11px | 400 | `--text-secondary` |
| Dashboard heading | 14px | 600 | `--text-secondary` |

Letter spacing для section labels: `0.10–0.12em`, text-transform: uppercase.

---

## Layout

### Title bar (34px)

- Background: `--bg-strip` (`theme::BG_STRIP`); **без отдельной painter-linии** под всю ширину — переход «titlebar ↔ контент» только за счёт контраста `--bg-strip` vs `--bg-base`. Раньше 1px `hline` воспринимался как случайная полоска «под логотипом» из-за состыковки с левым rail.
- Сетка из **трёх равных колонок**:
  - **левая** — **`[ prefs ]`** + **`VNOX`** (wordmark `--text-primary`, слева направо после prefs);
  - **центр** — статус ноды: **`●`** + **`NODE: …`** (`OFFLINE` / `CONNECTING` / имя ноды), **по центру средней колонки** (= визуальный центр окна);
  - **правая** — **transport‑подсказка** справа: `transport: idle` · `transport: connecting…` · `transport: quic/v1`.
- Вход в настройки: **`[ prefs ]`** — явная консольная кнопка (без нестабильных Unicode‑иконок); при открытых настройках тот же текст, цвет **accent**. Дубль: кнопка **open prefs** в нижней левой колонке, **только пока offline**.
- Нативный заголовок ОС (**taskbar / список окон**) — короткий **`Vnox`**, чтобы не дублировать тот же wordmark **`VNOX`** внутри клиента.

### Структура окна

```
┌─────────────────────────────────────────────────────┐
│ [prefs] VNOX  │   ● NODE: OFFLINE    │ transport: idle │
│        (нет линии 1px на всю ширину под панелью)    │
├──────────────────────────────────────────────────────┤
│ rail    │ channels (208px)    │ main content        │
│ (52px)  │                     │                     │
│ SERVERS │ node / status       │ connect or chat      │
│  [AB]   │ channel sections    │                     │
│  [+]    │ ───────────────     │                     │
│         │ identity + voice UI │                     │
└─────────┴─────────────────────┴─────────────────────┘
```

### Node rail (bookmark strip, 52px)

Узкая колонка **закладок серверов** (не «Discord‑кругляши»):

- Ширина **52px**, фон `--bg-strip`, как верхний titlebar для визуальной связности.
- Заголовок колонки: **SERVERS** (8px, `--text-ghost`, центрируется по ширине rail).
- Плитки **30×30px**, скругление 5px, **строго центрируются** по горизонтали (без случайной короткой линии‑разделителя посередине — она давала эффект «криво» в узком rail).
- **Idle** (нет активной закладки): рамка 1px `--border-faint`, подпись `···` во внутреннем поле, tooltip объясняет «подключайся через Connect».
- **Активная нода**: 2 символа аббревиатуры, `--accent` левый акцент 2px внутри плитки, `--accent-10` фон.
- **«+»**: только одна вторичная плита внизу; tooltip — «bookmark позже».

### Node switcher (legacy note)

Исторически описывался как 40px rail с линией перед `+`; в текущем клиенте заменено на блок **Node rail** выше.

### Channel list (208px)

- Node info (верх): имя ноды 12px semibold + `lnex://nc.<short_id>` 9px `--text-ghost` при коннекте; офлайн — одна строка **not connected**
- Section labels: 9px uppercase, `--text-ghost`, letter-spacing 0.12em
- Channel item: 4px 12px padding, gap 6px (icon + name)
  - Default: `--text-ghost`
  - Hover: `--text-dim` (без background)
  - Active: `--text-secondary` + левая полоска 2px `--accent` + `--accent-10` bg
- Voice users под каналом: indent 26px, 10px, `--success` для говорящих, `--text-ghost` для muted

### Bottom-left identity bar

```
[ open prefs ]     (только пока offline)

— voice ~ channel    00:42
  [x] microphone     [x] hear others

[YU] you
     a1b2…9f0e   (pubkey hex, middle-truncated)
```

- **Не показывать** фиктивный RTT (например «12ms»), если нет реального измерения.
- **Не дублировать** протокольную строку вида `LNEx v1 · udp` под ником — перегружает и налезает на аватар/текст; протокол раскрывается в Connect / docs.
- Настройки **не** прячем в ряд мелких иконок у ника: основной вход — **`[ prefs ]`** в title bar; офлайн — компактная кнопка **open prefs**.
- В голосе: чекбоксы **microphone** / **hear others** (без эмодзи/символов, которые на части шрифтов дают «квадратики»).
- Avatar: 32×32px, radius 6px, disabled button (только визуал), self — `--accent-10` + border.

### Main area

- Channel header: 36px, border-bottom `--border-subtle`
- Messages: padding 12px 14px, gap между группами 8px
- Input: `#general ›` prefix + cursor blink + hint text

---

## Components

### Toggle

```
Off: width 30px, height 16px, bg --bg-elevated, border --border-default
     thumb: 10px circle, bg #2d2d2d, left 2px

On:  bg --accent-10, border --border-accent
     thumb: bg --accent, left 16px, glow rgba(255,107,53,0.4)

Transition: 150ms ease
```

### Badge

```css
/* Protocol / transport */
.badge-lnex    { color: #ff844f; border: 1px solid rgba(255,107,53,0.2); bg: rgba(255,107,53,0.08) }
.badge-udp     { color: #6a9ecf; border: 1px solid rgba(106,158,207,0.2); bg: rgba(106,158,207,0.10) }
.badge-tcp     { color: #e6a230; border: 1px solid rgba(230,162,48,0.2); bg: rgba(230,162,48,0.10) }
.badge-enc     { color: #7cb87a; border: 1px solid rgba(124,184,122,0.2); bg: rgba(124,184,122,0.10) }

/* Размер: 9px, padding 1px 6px, border-radius 3px */
```

### Select / Input

```
bg: --bg-elevated (#111)
border: 1px solid --border-default
border-radius: 5px
font: 10px IBM Plex Mono
color: --text-muted

focus:
  border-color: --border-accent
```

### Slider

```
track: 2px height, --border-default
thumb: 10px circle, --accent, subtle glow

::-webkit-slider-thumb {
  background: var(--accent);
  box-shadow: 0 0 6px rgba(255, 107, 53, 0.3);
}
```

### User avatar

```
Size variants:
  sm  — 24×24px, border-radius 5px   (identity bar)
  md  — 26×26px, border-radius 6px   (member list)
  lg  — 36×36px, border-radius 8px   (identity card in settings)

Default:  bg --bg-elevated, border --border-default, color --text-muted
Self:     bg --accent-10, border --border-accent, color --accent
Other:    custom per-user, based on their seed color (--info, --text-secondary, etc.)
```

### System message / separator

```
font-size: 9–10px
color: --text-ghost
display: flex + ::before/::after lines in --border-subtle

Examples:
  ● connected · nightcore.lnex · LNEx v1
  — today —
```

---

## Active / Indicator Language

Везде используется одна и та же визуальная метафора для "активно":

```
Левая вертикальная полоска 2px --accent
+ subtle --accent-10 background
```

Это применяется к:
- активной ноде в switcher
- активному каналу в списке
- активному разделу в настройках

Не используется background без полоски, и не используется полоска без подсветки.

---

## Voice Indicators

```
Говорит:   dot 5px #7cb87a, box-shadow 0 0 6px rgba(124,184,122,0.45)
Muted:     icon ti-microphone-off, color --text-ghost
В канале:  dot или icon перед именем, indent 26px от иконки канала
```

В voice badge (активный call):
```
dot 5px --accent, box-shadow 0 0 6px rgba(255,107,53,0.5)
```

---

## Settings Layout

```
┌─────────────────────────────────────────────────────┐
│ titlebar — [prefs]* │ VNOX │ …                      │
│ открыть настройки: клик [prefs]; обратный маршрут   │
│ тот же, пока приложение показывает settings screen │
├──────────────────────────────────────────────────────┤
│ nav (160px)     │ content                            │
│                 │                                    │
│ account         │ [page title]                       │
│   identity      │ [subtitle]                         │
│                 │                                    │
│ audio           │ [group label] ──────────────────   │
│   voice ◄       │   row: label + control             │
│   output        │   row: label + control             │
│                 │                                    │
│ network         │ [group label] ──────────────────   │
│   network       │   ...                              │
│   overlay       │                                    │
│                 │                                    │
│ app             │                                    │
│   appearance    │                                    │
│   keybinds      │                                    │
│   plugins       │                                    │
│                 │                                    │
│ debug           │                                    │
│   advanced      │                                    │
└─────────────────┴────────────────────────────────────┘
```

- Nav width: 160px, bg `--bg-base`
- Section labels в nav: 9px uppercase, `--text-ghost`
- Nav item: 11px, default `--text-ghost`, hover `--text-dim`, active `--text-secondary` + левая полоска
- Content padding: 16px 20px
- Group label: 9px uppercase, `--text-ghost`, border-bottom `--border-subtle`, margin-bottom 8px
- Row: `display:flex; justify-content:space-between; align-items:center; padding:6px 0`
- Row separator: `border-top: 1px solid #0f0f0f` (почти невидимый, только ритм)

---

## Motion

```
duration-instant: 80ms   — toggle, dot
duration-fast:    120ms  — hover color change
duration-base:    150ms  — toggle thumb, panel transitions
duration-slow:    250ms  — page switch in settings

ease: cubic-bezier(0.0, 0.0, 0.2, 1)  — ease-out для всего
```

Никаких scale transforms на hover. Только:
- `color` transition
- `background` transition
- `border-color` transition
- `box-shadow` для glow (subtle)
- `left` для toggle thumb

---

## Anti-patterns (клиент-специфично)

| ❌ Не делать | ✅ Делать |
|-------------|---------|
| Отдельная 1px линия на всю ширину только под titlebar (стек с rail = «случайная полоска») | Отделение только контрастом `--bg-strip` / `--bg-base`; линии локально там, где группируешь контент |
| Круглые аватары | Квадратные с border-radius 5–8px |
| Sidebar с серверами как у Discord (72px, круглые) | Узкий rail **52px**, квадратные плитки **30×30**, аббревиатура, заголовок **SERVERS** |
| Панель участников справа как у Discord | Нет отдельной панели, участники — в боковом списке |
| Neon glow на элементах | Subtle glow max 0.18 opacity |
| Pure black backgrounds | Минимум #0a0a0a |
| Pure white text | Максимум #f4c89a |
| RGB accent | Один тёплый accent #ff6b35 |
| Rounded pill buttons everywhere | border-radius 4–6px, pill только для badge |
| Жирные разделители | 1px --border-subtle, почти невидимые |

---

## Noise Texture

Лёгкий шум поверх всего интерфейса — добавляет аналоговое ощущение.

```css
.root::after {
  content: '';
  position: absolute;
  inset: 0;
  background-image: url("data:image/svg+xml,..."); /* SVG feTurbulence */
  opacity: 0.025;  /* максимум 0.03, иначе грязно */
  pointer-events: none;
  z-index: 999;
}
```

В egui/wgpu: реализуется как overlay texture на финальном render pass.

---

## Seed Colors для пользователей

У каждого пользователя свой цвет ника, детерминированный от pubkey.

Палитра допустимых цветов (тёплая, совместимая с системой):

```
#c88b5a  — warm amber   (default / self)
#6a9ecf  — steel blue
#7cb87a  — warm green
#c49a6c  — sand
#b07cc6  — muted purple
#d4876a  — terracotta
```

Не используются: яркие/neon цвета, холодные синие, чистый белый/красный.
