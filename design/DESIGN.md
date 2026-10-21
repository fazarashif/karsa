# Karsa design system v1

This is the spec for the Karsa island: direction, tokens, components, motion, brand and copy. All measurements are logical px at 100% scaling. All colors are hex, with alpha where used. `tokens.css` replaces the `:root` block in `src/app.css`, and `tokens.json` holds the same values. The screens in `screens/` are 2x renders of `prototype/index.html`. They were captured without Segoe UI, so the type there uses a fallback. On Windows it renders in Segoe UI Variable.

## 1. Direction

Karsa is a quiet black island with Javanese accents, at the intensity the brief calls *terasa* (noticeable). The island stays solid `#000`. The Javanese character comes through four channels only:

1. **Color.** Accents come from batik sogan and brass: kuningan gold, sogan brown, bata brick, gading ivory and daun leaf.
2. **Kawung.** A 10 px four-petal mark sets the type color in the pill, on alerts and in separators.
3. **Aksara.** ꦏꦂꦱ appears in the wordmark lockup and as a 6% watermark in the panel header.
4. **Words and character.** The UI is in English. The exceptions are the time-of-day greeting, one proverb, and a pair of doll eyes (*mripat*) that replace the clock when nothing is due today.

Motion follows the existing spring, plus a gamelan-like beat when content enters.

### Principles

- **Black first.** The island is a hole in the screen. Accents mark meaning, never decoration.
- **One accent per moment.** Each state shows a single semantic color: focus kuningan, meeting sogan, deadline bata, general gading, done daun.
- **Still when idle.** Nothing moves unless an event happens: hover, a timer tick, an alert, or a change of expression.
- **Small and exact.** Ornaments are 8–10 px, the watermark is 6%, and only one proverb ships.
- **Legible everywhere.** All text meets WCAG AA on black, every text size is ≥ 11 px, and nothing relies on blur or shadow.

### Javanese nuance: do / don't

| Do | Don't |
|---|---|
| Use kawung only at 8–10 px as a type marker or separator, and 24 px in the Todo empty state | Tile batik across surfaces or behind text |
| Keep the aksara watermark at 6% opacity, in the panel header only | Use aksara as UI text, or set it in a font |
| Show the greeting (Sugeng enjing / siang / sonten / dalu) above the date in the panel | Translate UI labels into Javanese |
| Use the proverb "Alon-alon waton kelakon" only in the Todo empty state, with its English meaning | Rotate proverbs, or put them in alerts |
| Let the eyes react once per event | Loop, blink on a timer, or follow the cursor |
| Use sogan-deep `#8A5A2E` for ornaments | Use sogan-deep for text (3.6:1) |

## 2. Tokens

### 2.1 Color

| Token | Value | On black | Use |
|---|---|---|---|
| `--island-bg` | `#000000` | | Island, all states |
| `--fg` | `#F5F5F7` | 19.3:1 | Primary text |
| `--muted` | `#EBEBF58F` | 5.6:1 | Secondary text, inactive tabs, time |
| `--faint` | `#EBEBF580` | 4.6:1 | Captions, meta, placeholders. **M0 was 0.32 alpha (2.3:1) and failed AA.** |
| `--disabled` | `#EBEBF552` | 2.4:1 | Non-text and disabled only |
| `--hairline` | `#FFFFFF17` | | Dividers, input borders |
| `--hairline-strong` | `#FFFFFF29` | | Hovered borders |
| `--surface` | `#FFFFFF12` | | Inputs, secondary buttons |
| `--surface-hover` | `#FFFFFF1F` | | Hover, selected chip or segment |
| `--surface-pressed` | `#FFFFFF29` | | Pressed |
| `--track` | `#FFFFFF1F` | | Progress track, empty session dot |
| `--popover-bg` | `#141414` | | Menus (solid, no shadow) |
| `--checkbox-border` | `#EBEBF580` | 4.6:1 | Unchecked checkbox |
| `--focus-ring` | `#EFE6D2B3` | | 2 px outline on `:focus-visible` |
| `--accent` | `#D4A84B` kuningan | 9.5:1 | Primary buttons, active tab, greeting. **M0 was `#ff9f0a`.** |
| `--accent-hover` / `--accent-pressed` | `#DDB662` / `#C29640` | | |
| `--on-accent` | `#000000` | 9.5:1 on kuningan | Text on primary |
| `--focus` | `#D4A84B` kuningan | 9.5:1 | Focus timer. **M0 was `#30d158` (green).** |
| `--meeting` | `#C08552` sogan | 6.7:1 | Meeting type |
| `--deadline` | `#D9644A` bata | 5.9:1 | Deadline type, due today or overdue |
| `--general` | `#EFE6D2` gading | 16.9:1 | General type |
| `--done` | `#A8B46A` daun | 9.4:1 | Checked checkbox, "Added", "All done" |
| `--danger` | `#F07A6A` abang | 7.7:1 | Delete |
| `--ornament` | `#8A5A2E` sogan-deep | 3.6:1 | Kawung separators and the empty-state mark only |
| `--*-tint` | accent + `29` (16%); general `1F` | | Badge and mini-button backgrounds |
| `--eye-sclera` / `--eye-iris` / `--eye-glint` / `--eye-blush` | `#EFE6D2` / `#3B2414` / `#FFFFFF` / `#D9644A8C` | | Eyes |
| `--edge` | `inset 0 0 0 1px #FFFFFF12` | | The island's only elevation |

### 2.2 Typography

Segoe UI Variable is the only typeface (no extra font file). Use the Text optical size everywhere and the Display optical size for the timer. Numbers that change use `tabular-nums`.

| Style | Size / line | Weight | Extra | Where |
|---|---|---|---|---|
| caption | 11 / 14 (18 in rows) | 600 | uppercase, 0.06em | Peek keys, section headers, field labels |
| meta | 11 / 14 | 400 | | Due dates, counts, "Saved" |
| greeting | 11 / 14 | 600 | 0.02em, `--accent` | Panel greeting |
| body | 13 / 18 | 400 | | Default |
| body-strong | 13 / 18 | 600 | pill: 0.01em | Pill, tabs, buttons, list titles |
| editor | 13 / 20 | 400 | | Note textarea |
| peek-date | 14 / 18 | 600 | | Peek header |
| title | 15 / 20 | 600 | | Alert title, meeting title, proverb |
| title-lg | 17 / 22 | 600 | | Panel date |
| timer | 56 / 60 | 600 | Display, −0.01em, tabular | Focus tab |

### 2.3 Spacing, radii, sizes

The spacing grid is 4 px: `--space-1` 4, `-2` 8, `-3` 12, `-4` 16, `-5` 20, `-6` 24, `-8` 32, plus `-0-5` 2 for optical fixes.

| Radius | px | | Size | px |
|---|---|---|---|---|
| `--radius-checkbox` | 4 | | `--pill-h` | 32 |
| `--radius-icon-button` | 8 | | `--pill-min-w` | **72** (M0: 120, see 4.1) |
| `--radius-badge` | 9 | | `--pill-max-w` | 220 |
| `--radius-control` | 10 | | `--peek-w/h` | 320 × 172 |
| `--radius-menu` | 12 | | `--panel-w/h` | 380 × 520 |
| `--radius-button` | 14 | | `--alert-w/h` | 320 × 112 |
| `--radius-pill` | 16 | | `--control-h` | 36 |
| `--radius-card` | 22 (peek, alert) | | `--button-h`, `--icon-button` | 28 |
| `--radius-panel` | 26 | | `--tab-h` | 32 |

The window stays 400 × 540, and every state fits inside it.

### 2.4 Elevation and opacity

There are no drop shadows. The only elevation is `--edge`, which keeps black readable on dark wallpapers. Menus use the solid `--popover-bg` plus `--edge`. Opacity tokens: `--o-disabled` 0.32 (applies to the whole control) and `--o-watermark` 0.06.

### 2.5 Motion tokens

| Token | Value | Use |
|---|---|---|
| `--spring` | existing `linear()` (ζ 0.78, ~2% overshoot) | Box width and height |
| `--morph-duration` | 700ms | Box morph |
| `--radius-duration` | 350ms, ease-out | Border-radius |
| `--delay-content-in` | 90ms | Content waits for the box to start growing |
| `--dur-content-in` | 220ms, `--ease-out` | opacity 0→1, scale 0.94→1 |
| `--dur-content-out` | 100ms | opacity 1→0, scale 1→0.98 |
| `--beat` | 60ms (max 5 beats) | Gamelan stagger between rows |
| `--dur-blink` | 140ms | Kedhep |
| `--dur-gong` | 240ms | Kawung on alert arrival |
| `--dur-tab` | 160ms | Tab underline and content |
| `--dur-fast` / `--dur-base` | 120 / 160ms | Hover and press |

Under `prefers-reduced-motion: reduce`, every duration and beat is 0. **Remove the `filter: blur()` from `contentIn` in `src/lib/motion.ts`.** The brief disallows blur, and opacity plus scale give the same feel.

## 3. Ornament system

| Asset | Size | Color | Where | Rule |
|---|---|---|---|---|
| `ornaments/kawung-mark.svg` | 10 px (8 px in separators) | currentColor | Pill (type color), alert mark, Peek separator, panel greeting | Max one per surface row. Never alone on the idle pill. |
| `ornaments/separator-kawung.svg` | 64 × 8 | currentColor | Optional fixed-width separator | Prefer the CSS `.sep` (two hairlines plus the mark) |
| `ornaments/kawung-tile.svg` | 24 × 24 tile | currentColor | Reserved for the "Open full" note window header at ≤ 6% | Never inside the island |
| Kawung at 24 px | 24 px | `--ornament` | Todo empty state | |
| `ornaments/aksara-karsa.svg` | 147 × 60 in the panel | `--gading` at 0.06 | Panel header, `right: 76px; top: 6px` | One per panel, behind the header, `pointer-events: none` |

Restraint rules: a surface has at most one ornament type. Ornaments are never interactive, never animated except the gong, and never carry information that the text does not also carry.

## 4. Components and screens

Each screen lists its tokens and assets. Screens are numbered as in `screens/`.

### 4.1 Pill (`Pill.svelte`)

- **Container:** height 32, `--radius-pill`, padding `0 16 0 14`, gap 7. Text is body-strong with 0.01em tracking and tabular nums. Width hugs content, clamped to 72–220. The label ellipsizes.
- **Size change:** min width drops from 120 to 72 because the eyes pill has no text. Text pills hug their content, and the shortest real label still exceeds 120. In `config.ts`, change the CSS `min-width` to 72.
- **Notch (M7):** with Settings › Position on Top, the pill hangs from the top edge of the screen: height 26 (`--notch-h`), width 80–200 (`--notch-min-w`, `--notch-max-w`), padding `0 14 0 12`, square top corners and `--radius-notch` (12) at the bottom, eyes 24 × 12, and `--edge-notch` (the edge without its top side). Every larger state keeps its size, centred under the notch with square top corners. Peek opens after 450 ms there.

| # | Variant | Content | Tokens / assets |
|---|---|---|---|
| 01 | Idle, nothing due today | Eyes only, width 72, centered | `eyes/eyes-melek.svg` 28 × 14 |
| 02 | Idle, next up | kawung `--meeting` (or the item's type color) · title `--fg` · time in type color | kawung-mark |
| 03 | Live: focus | padding-left 12, gap 8 · `eyes-fokus` · `18:42` in `--focus` | eyes-fokus |
| 04 | Live: break | `eyes-leren` · "Break" `--muted` · `04:12` `--fg` | eyes-leren |
| 05 | Live: reminder ≤ 15 min | kawung type color · title · "in 4m" type color | kawung-mark |
| 06 | Alert waiting | kawung in alert type color · title `--fg` | kawung-mark |
| 07 | Eye expressions | kedhep (hover), mesem (all of today's tasks done), merem (22:00–05:00 with nothing due) | eyes-* |

Priority follows requirements 4.2: alert waiting, then reminder ≤ 15 min, then focus or break, then next up, then eyes. The time is never shown, because the taskbar already has the clock.

### 4.2 Eyes (*mripat*)

The eyes form a 28 × 14 frame, two eyes 14 px apart. Each sclera is an ellipse 7.2 × 9.6 in gading. The iris is 5.2 × 7.2 in `#3B2414`, offset +0.3, +0.6. Glints: Ø2 at the top-left and Ø1 at the bottom-right, in white. Blush: two 4 × 2 ellipses of `--eye-blush` below the outer corners. The SVGs are drawn for `--island-bg` and use clipPaths with file-unique ids (`kf-*`, `kl-*`).

| Expression | File | Trigger | Notes |
|---|---|---|---|
| Melek (awake) | eyes-melek.svg | Idle, nothing due | Default |
| Kedhep (blink) | eyes-kedhep.svg | `mouseenter` on the pill | Once: scaleY 1 → 0.18 → 1 over 140ms, origin center. Peek still opens at 300 ms. |
| Fokus | eyes-fokus.svg | Focus phase running | Kuningan lids tilt inward, iris looks down, no blush |
| Leren (rest) | eyes-leren.svg | Break running | Flat gading lids, blush |
| Mesem (smile) | eyes-mesem.svg | All of today's tasks done (until the next due item) · focus-end alert | Arcs and blush |
| Merem (asleep) | eyes-merem.svg | 22:00–05:00 and nothing due | Closed arcs, no blush |

When the expression changes, crossfade the two SVGs over 160ms. Nothing loops.

### 4.3 Peek (`Peek.svelte`): 08, 09

The card is 320 × 172, `--radius-card`, padding `14 18 12`.

- **Head:** date (peek-date) on the left, time `--muted` on the right, padding-bottom 9.
- **Separator:** `.sep`, two 1 px `--hairline` lines with an 8 px kawung in `--ornament` between them, gap 8, margin-bottom 4.
- **Rows:** padding `4 0`, gap 12, key column 58 px (caption, line-height 18, `--faint`).
  - **Focus:** time in `--focus` 600, a 96 × 3 bar (`--track` with `--focus` fill), then 20 × 20 icon buttons for pause and stop (12 px icons) on the right.
  - **Next up:** title, with the time in the type color on the right.
  - **Tasks:** up to 3 lines, each title plus a due label on the right (bata if due today or overdue, `--muted` for a weekday).
- **The M0 "Click to open" hint is removed** to fit three tasks. The whole card stays clickable.
- **Quiet day (09):** "No session running", "Nothing else today" and "No open tasks", all in `--muted`.

Assets: kawung-mark, icons/pause, icons/stop.

### 4.4 Panel shell (`Panel.svelte`): 10–19

The panel is 380 × 520, `--radius-panel`, padding `16 16 12`.

- **Watermark:** `aksara-karsa.svg`, 60 px tall.
- **Header:** padding `0 4 12`, gap 2.
  - Line 1: the greeting (kawung 8 px in `--accent`, gap 6).
  - Line 2: date (title-lg) on the left, time `--muted` on the right.
- **Tabs:** gap 20, padding `0 4`, 1 px `--hairline` bottom border.
  - Each tab is 32 high, body-strong, `--muted`. Hover is `--fg`.
  - The active tab is `--fg` with a 2 px `--accent` underline at bottom −1, full label width, radius 1.
  - The tab bar is text only. The tab icons ship for other uses (empty states, tray menu, the full window).
- **Body:** fills the remaining height, padding-top 12, and scrolls with an overlay scrollbar.
- **The M0 footer ("Prototype…", "Esc to close") is removed.** Esc still closes the panel. Put `aria-keyshortcuts="Escape"` on the panel.

Greeting by local time: 05:00–10:59 "Sugeng enjing", 11:00–14:59 "Sugeng siang", 15:00–17:59 "Sugeng sonten", 18:00–04:59 "Sugeng dalu".

#### Todo: 10, 11, 12

- **Toolbar** (28 high, margin-bottom 8): list switcher, then "N open" (meta) on the right.
  - The switcher is 28 high, padding `0 6 0 8`, radius 8, body-strong, with a 12 px chevron-down. It shows `--surface` while open.
- **Switcher menu (11):** 200 wide, `--popover-bg` + `--edge`, radius 12, padding 4, 8 px below the switcher.
  - Items are 32 high, padding `0 10`, radius 8: name, count (meta) and a check in `--focus` on the current list.
  - Then a divider, "New list" (add icon) and "Manage lists…" (edit icon).
- **Quick add:** 36 high, radius 10, `--surface`, 1 px `--hairline`, a 16 px add icon in `--faint`, and the placeholder "Add a task…". On focus the border is `#D4A84B80`.
- **Task row:**
  - Layout: padding `8 4`, gap 10, 1 px `--hairline` bottom border. Checkbox 16, radius 4, 1.5 px `--checkbox-border`. Checked: `--done` fill with a 10 px check in black.
  - Title is body. The meta line (11/14 `--faint`, gap 10) holds the due date (calendar 12, bata when due today or overdue) and subtask progress ("2/5", subtask icon 12).
  - A 20 × 20 chevron toggles the subtasks.
- **Subtask row:** padding `6 4 6 30`, checkbox 14.
- **Completed section:** a 32-high caption header ("Completed · N") with a 12 px chevron. It starts collapsed. Done rows have `--faint` text with line-through.
- **Empty state (12):**
  - Kawung 24 px `--ornament`.
  - "Alon-alon waton kelakon" (title, `--gading`).
  - "Slowly, as long as it gets done." (`--muted`).
  - "Type above and press Enter." (meta `--faint`).
  - Gap 6, centered, padding-bottom 32.

Assets: icons/add, chevron-down, calendar, subtask, check, edit; kawung-mark.

#### Focus: 13, 14

- Column centered, gap 12.
  - Phase label (caption): "Focus · session 3 of 4" in `--focus`, or "Short break" / "Long break" in `--muted`, or "Ready" in `--faint`.
  - Eyes at 56 × 28 (fokus / leren / melek).
  - Timer (56/60 Display).
  - Bar 240 × 4 (`--track` with `--focus` fill, radius 2).
  - "Ends at 14:35" (`--muted`).
- **Controls row (gap 16, margin-top 8):**
  - 40 × 40 round `--surface` buttons for stop and skip.
  - A 56 × 56 round `--accent` button with pause or play in black (20 px).
  - Ready state (14): a single 36-high primary button, "Start focus", with the play icon.
- **Stats footer:** 1 px `--hairline` top border, padding `12 4 0`.
  - "Today" caption, then 4 session dots (8 px, gap 6). Done dots are `--focus`, the current one is half-filled, the rest are `--track`.
  - "2 sessions · 50 min" (`--muted`) on the right.

Assets: icons/play, pause, stop, skip; eyes-fokus, eyes-leren, eyes-melek.

#### Notes: 15, 16

- **Toolbar:**
  - Search, 32 high, radius 10, search icon, placeholder "Search notes".
  - Gap 8, then a "New" button (32 high, radius 16, add icon and chevron). It opens the menu pattern from 11 with "New note" and "New meeting note".
- **Filter:** chips "All / Notes / Meetings", 28 high, radius 14, gap 6, margin `10 0 4`. Inactive chips have a 1 px `--hairline` border and `--muted` text. The active chip is `--surface-hover` with `--fg`.
- **List item:** padding `10 4`, 1 px `--hairline` bottom border.
  - Line 1: body-strong title and a date (meta) on the right. Meetings get a 12 px meeting icon in `--meeting`.
  - Line 2 (meta): a snippet, or for meetings the topic plus "N open" in `--focus` ("All done" in `--done`).
- **Editor (16):**
  - Bar (28 high): "‹ Notes" back button (text, `--muted`), then on the right the open-full and more icon buttons.
  - Textarea: `--surface`, 1 px `--hairline`, radius 12, padding 12, editor type.
  - Status line: "Saved" and "Markdown" (meta `--faint`).

#### Meeting note: 17

- **Bar:** "‹ Notes", the "Meeting" badge (18 high, radius 9, `--meeting` on `--meeting-tint`, 12 px meeting icon) and the open-full button.
- **Title:** title style, padding `0 4 8`.
- **Fields:** a 76 / 1fr grid, row gap 6. Keys are captions: When, Topic, Attendees.
  - Attendees are 22-high chips, radius 11, `--surface`, 12 px text. A "+" chip adds one.
- **Notes:** "Notes" caption, then bullets (a 3 px `--muted` dot, indent 16).
- **Action items:**
  - Header: "Action items · N open" caption, with "Add all" (text button, 24 high, 11 px, add-to-todo icon) on the right.
  - Rows: padding `6 4`, checkbox, title, then on the right either "Add to todo" (24 high, radius 12, `--accent` on `--focus-tint`, 11/600) or "Added" (`--done`, check-circle icon, no background). Done items are `--faint` with line-through and have no button.

#### Reminders: 18, 19

- **Header:** "Upcoming" caption, with a "New reminder" button (28, add icon) on the right.
- **Day groups:** caption headers, 28 high, margin-top 6: "Today", "Tomorrow", then "Wed 21 Oct" style.
- **Row:** padding `8 4`, gap 12.
  - Time column 40 px, body-strong, tabular.
  - Title, with a meta line containing:
    - the type badge ("Meeting" `--meeting`, "Deadline" `--deadline`, "General" `--general` on its tint)
    - and one of: the repeat icon plus "Daily / Weekdays / Weekly", the clock icon plus "N min before", or the link icon plus "From Todo" for task deadlines.
- **Form (19):**
  - Bar: "‹ Reminders", plus "New reminder" or "Edit reminder" (meta).
  - Fields stack with gap 12, each a caption label over a control, gap 4:
    - Title (input)
    - Date and Time (two inputs with calendar and clock icons)
    - Type (segmented: 2 px inset, 28-high segments, the selected one `--surface-hover` with the type icon)
    - Repeat (select styled as an input with chevron-down)
    - Remind me (chips: At time, 5m, 10m, 15m, 30m)
  - Footer pinned to the bottom: "Delete" (danger text button with the delete icon, edit only) on the left, "Cancel" and "Save" (primary) on the right.

### 4.5 Alert (`Alert.svelte`): 20–24

The card is 320 × 112, `--radius-card`, padding `16 16 14 18`, space-between layout.

- **Top:** a 10 px kawung in the type color (margin-top 5), then the title (title, ellipsis) and the subtitle (`--muted`).
- **Actions:** a row with gap 8. Buttons are 28 high, padding `0 12`, radius 14, and never wrap. Secondary buttons are `--surface`, primary is `--accent` on `--on-accent`. A left-side text action can sit before a spacer.

| # | Type | Mark | Title / subtitle | Actions |
|---|---|---|---|---|
| 20 | General | kawung `--general` | "Call the bank" / "Reminder · now" | Snooze 5m (snooze icon), **Dismiss** |
| 21 | Meeting | kawung `--meeting` | "Weekly sync" / "Meeting · starts in 5 min" | "+ Meeting note" (text, `--meeting`, left), Snooze 5m, **Dismiss** |
| 22 | Deadline | kawung `--deadline` | "Submit Q3 report" / "Deadline · due 15:00" | Snooze 5m, **Dismiss** |
| 23 | Focus phase end | eyes-mesem 28 × 14 | "Focus session done" / "Session 3 of 4 · short break next" | Skip, **Start break** (break end: "Break's over" / "Start focus") |
| 24 | Missed | kawung `--general` | "2 missed reminders" / "Standup 09:00 · Call the bank 11:00" | Review (opens Reminders), **Dismiss** |

The meeting alert's label is shortened to "+ Meeting note" so that three actions fit in 286 px.

### 4.6 Interaction states: 25

| Control | Hover | Pressed | Focus-visible | Disabled |
|---|---|---|---|---|
| Button (secondary) | bg `--surface-hover` | bg `--surface-pressed`, scale 0.97 | 2 px `--focus-ring`, offset 2 | opacity 0.32, no pointer |
| Primary | `--accent-hover` | `--accent-pressed`, scale 0.97 | same ring | 0.32 |
| Text button | bg `--surface` | bg `--surface-hover` | ring | 0.32 |
| Icon button 28 | `--surface` + `--fg` | `--surface-hover` | ring | 0.32 |
| Round 40 / 56 | `--surface-hover` / `--accent-hover` | `--surface-pressed`, scale 0.95 | ring | 0.32 |
| Input | border `--hairline-strong` | | border `#D4A84B80`, caret `--accent` (no outer ring) | 0.32 |
| Checkbox | border `--fg`; checked `#B5C07A` | bg `--surface`, scale 0.9 | ring | 0.32 |
| Tab | `--fg` | → active | ring, offset −2, radius 6 | 0.32 |
| Chip | border `--hairline-strong`, `--fg` | → on | ring | 0.32 |
| Menu item | bg `--surface-hover` | bg `--surface-pressed` | ring, offset −2 | 0.32 |

Hover and press use `--dur-fast` with `--ease-standard`, on background-color and transform only.

## 5. Motion spec

The box morph is the existing one: width and height on `--spring` for 700ms, border-radius ease-out for 350ms. The window region grows immediately and shrinks after 450ms. Content uses `transform` and `opacity` only.

| Transition | Spec |
|---|---|
| Idle → Peek | Kedhep fires on `mouseenter` (eyes pill only). After 300 ms the box morphs. Old content leaves in 100ms. New content enters on beats: the head at 90ms, then the separator and each row 60ms apart (90, 150, 210, 270…), each fading 220ms with scale 0.94 → 1. |
| Peek → Expanded | Morph. The header enters at 90ms, the tabs at 150ms, the body at 210ms. Tab-body rows do not stagger individually. |
| Collapse (any → pill) | Content out in 100ms. Morph to pill. The pill content enters at 90ms with no stagger. |
| Alert arrival | Morph from the pill. Content enters on beats (top at 90ms, actions at 150ms). The **gong**: the kawung scales 0.6 → 1.08 → 1 over 240ms at 150ms, once. No sound. |
| Alert → waiting pill | After 8 s without hover: content out in 100ms, morph to pill. The pill's kawung enters with the gong at 90ms. The title enters with it. |
| Tab switch | The underline slides between tabs (transform translateX + scaleX, 160ms `--ease-standard`). The body crossfades: out 80ms, in 160ms with translateY 4 → 0. No morph, since the panel size is fixed. |
| Eye expression change | Crossfade 160ms. Kedhep: scaleY 1 → 0.18 → 1, 140ms total. |
| Reduced motion | Morph and content changes are instant. No beats, gong, blink or crossfade. Expressions swap instantly. |

There is no continuous animation. The only per-second work is the visible countdown text.

## 6. Brand

| Asset | Spec |
|---|---|
| `assets/icon-app.svg` / `icon-app-1024.png` | 1024 squircle (rx 232) `#000` with a 12 px `#FFFFFF14` inner edge. Gading pill 672 × 224 (rx 112) at y 400, with a sogan-deep kawung (scale 11) centered at x 300. This evolves the M0 icon: the green dot becomes kawung. Export the Tauri sizes from the 1024 PNG (`npx tauri icon design/assets/icon-app-1024.png`). |
| `assets/icon-tray-16.svg` | Black pill 15 × 7 with a 1 px gading stroke and a 4-dot kawung in kuningan. Reads on both the dark (#202020) and light (#F3F3F3) taskbar. |
| `assets/icon-tray-32.svg` | The same at 30 × 14 with a 1.5 px stroke, for 200% scaling. |
| `assets/logo-mark.svg` | 64 × 28 gading pill with a sogan-deep kawung. |
| `assets/logo-wordmark.svg` | "karsa" in lowercase, outlines from Plus Jakarta Sans SemiBold (Tokotype, OFL 1.1), currentColor. |
| `assets/logo-wordmark-aksara.svg` | Lockup: karsa · kawung · ꦏꦂꦱ, outlines from Noto Sans Javanese (OFL 1.1), currentColor. **To be checked by a native reader:** the layar (ꦂ) is placed by the font's default mark position, above the end of ꦏ. |

Fonts were used only to draw the outlines. No font files ship.

## 7. Copy deck

### English UI strings

| Area | Strings |
|---|---|
| Pill | `{title}` · `{HH:MM}` · `in {n}m` · `Break` · `{mm:ss}` · aria: `Karsa, {label}`, `Alert: {title}`, `Karsa, nothing due today` |
| Peek | `Focus` `Next up` `Tasks` · `No session running` `Nothing else today` `No open tasks` · `Today` `Tue`… |
| Tabs | `Todo` `Focus` `Notes` `Reminders` |
| Todo | `Inbox` `{n} open` `Add a task…` `Today {HH:MM}` `{done}/{total}` `Completed · {n}` `New list` `Manage lists…` `Slowly, as long as it gets done.` `Type above and press Enter.` · aria: `New task`, `Delete task`, `Show subtasks` |
| Focus | `Focus · session {n} of {m}` `Short break` `Long break` `Ready` `Ends at {HH:MM}` `Start focus` `{n} min focus · {n} min break` `Today` `{n} sessions · {h}h {m} min` · tooltips: `Pause` `Resume` `Stop` `Skip` |
| Notes | `Search notes` `New` `New note` `New meeting note` `All` `Notes` `Meetings` `{n} open` `All done` `Saved` `Markdown` `Notes` (back) · tooltips: `Open full`, `More` |
| Meeting note | `Meeting` `When` `Topic` `Attendees` `Notes` `Action items · {n} open` `Add all` `Add to todo` `Added` |
| Reminders | `Upcoming` `New reminder` `Edit reminder` `Today` `Tomorrow` `{Ddd D Mmm}` `General` `Meeting` `Deadline` `Daily` `Weekdays` `Weekly` `{n} min before` `From Todo` `Title` `Date` `Time` `Type` `Repeat` `None` `Remind me` `At time` `5m` `10m` `15m` `30m` `Delete` `Cancel` `Save` |
| Alerts | `Reminder · now` `Meeting · starts in {n} min` `Deadline · due {HH:MM}` `Focus session done` `Session {n} of {m} · short break next` `Break's over` `{n} missed reminders` · `Snooze 5m` `Dismiss` `+ Meeting note` `Start break` `Start focus` `Skip` `Review` |
| Tray | `Open` `Start with Windows` `Quit` |

### Javanese phrases

| Phrase | Meaning | Where |
|---|---|---|
| Sugeng enjing | Good morning (05:00–10:59) | Panel greeting |
| Sugeng siang | Good day (11:00–14:59) | Panel greeting |
| Sugeng sonten | Good afternoon (15:00–17:59) | Panel greeting |
| Sugeng dalu | Good night (18:00–04:59) | Panel greeting |
| Alon-alon waton kelakon | Slowly, as long as it gets done | Todo empty state, with the English line below |
| ꦏꦂꦱ (karsa) | Will, intention | Wordmark lockup, panel watermark |
| Ing madya mangun karsa | In the middle, build the will | Origin of the name. Not shown in the UI. |

Expression names (melek, kedhep, fokus, leren, mesem, merem) are internal names and never shown. The aria label for the eyes pill is "Karsa, nothing due today".

## 8. Assets and budget

`assets/icons/` contains 36 SVGs on a 16 px grid. Line icons use a 1.5 px stroke with round caps and joins in currentColor. Transport glyphs and `more` are solid. The `-filled` variants are the "active" versions.

The icon names are: todo, todo-filled, focus, focus-filled, notes, notes-filled, reminders, reminders-filled, meeting, deadline, general, missed, play, pause, stop, skip, add, check, check-circle, close, delete, calendar, clock, repeat, search, snooze, subtask, add-to-todo, list, link, edit, open-full, more, chevron-down, chevron-right, chevron-left. The brief's "note" icon is `notes.svg`.

Type icons map as follows: general → general (bell), meeting → meeting, deadline → deadline (flag), focus → focus, missed → missed.

| Group | Files | Clean size |
|---|---|---|
| icons/ | 36 | ~11 KB |
| eyes/ | 6 | ~3.5 KB |
| ornaments/ | 4 | ~4.6 KB |
| logos and icons (SVG) | 6 | ~9.7 KB |
| icon-app-1024.png | 1 | ~50 KB |
| **Total** | | **~79 KB** (budget 150 KB) |

Every SVG is under 4 KB except `logo-wordmark-aksara.svg` (5.7 KB of glyph outlines). The design tool adds a provenance `<metadata>` block to each saved SVG (~8 KB each). Run `node design/tools/strip-svg-metadata.mjs` once after unpacking to get the clean sizes above.

## 9. Implementation checklist

1. Replace `:root` in `src/app.css` with `design/tokens.css`. Keep the existing `button` and focus rules, and switch their outline to `var(--focus-ring)`.
2. In `config.ts`, keep the `BOX` sizes. Change the Pill CSS `min-width` from 120 to 72.
3. In `motion.ts`, drop the blur from `contentIn` and add stagger via `delay: 90 + i * 60` (cap at i = 4).
4. In `Pill.svelte`, use eyes when nothing is due, kawung in type color for items and alerts, and never the clock.
5. In `Peek.svelte`, add the kawung separator, the focus bar and controls, and 3 tasks. Remove the hint.
6. In `Panel.svelte`, add the greeting, the watermark and the text tabs with underline. Remove the footer.
7. In `Alert.svelte`, add the kawung mark per type, the gong, and the per-type actions.
8. Generate the Tauri icons from `assets/icon-app-1024.png` and swap in the tray icon.
