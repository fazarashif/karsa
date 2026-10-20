# Brief for Claude Design: the Karsa design system

You are designing the visual design system and screens for **Karsa**, a lightweight "dynamic island" productivity companion for Windows 11. Your output will be implemented by Claude Code (an AI coding agent) in an existing Tauri 2 + Svelte 5 codebase, so every deliverable must be precise and ready to implement: exact sizes, hex colors, named tokens, and SVG assets.

**Language:** talk with me in **Bahasa Indonesia**. Everything you produce (UI copy, token names, file names, DESIGN.md) is in **English**, because the app's UI is English. Javanese words appear only as deliberate accents.

## Step 1 — Interview me before designing

Do not start designing until you have asked me about my preferences. Ask in Bahasa Indonesia, at most 4 questions per round, each with concrete options (and small visual sketches where it helps). Do not fill in answers yourself. Cover at least:

1. **Intensity of the Javanese nuance:** a subtle accent, clearly noticeable, or a bold signature?
2. **Which Javanese references:** batik motifs (kawung, parang, truntum, sido mukti), lurik stripes, the wayang gunungan (kayon), aksara Jawa (e.g. a wordmark ꦏꦂꦱ, to be checked by a native reader), brass (kuningan) or carved-wood details, gamelan-like rhythm in motion. Show me options.
3. **Color direction:** the island stays pure black; which accent palette? For example sogan brown, kuningan gold, nila/indigo, gading ivory, merah bata, or a mix. Accents also need to carry meaning (focus timer, meeting, deadline, general alert, done).
4. **Javanese language touches in an English UI:** time-of-day greetings (Sugeng enjing / siang / sonten / dalu), proverbs on empty states (e.g. "Alon-alon waton kelakon"), a tagline from "ing madya mangun karsa", or English only.
5. **Typography:** Segoe UI Variable only (lightest), or one display face for dates and headings?
6. **Icon style:** line or filled, stroke weight, rounded or sharp.
7. **Motion personality:** calm spring or snappy; any signature motion (e.g. a gunungan-like reveal when the panel opens)?
8. Things I love or dislike, and reference apps or images.

Then summarize my answers as a short **design direction** and wait for my OK. Next, show **2–3 quick explorations** (idle pill + peek card + panel header) at different Javanese intensities and let me pick one before building the full system.

## Step 2 — Understand the product (context files)

I will attach these files from the project. Read them before designing:

| File | What it tells you |
|---|---|
| `docs/requirements.md` | Full v1 spec: features, island states, sizes, behaviors, decisions |
| `README.md` | How the island works technically |
| `design/context/*.png` | Screenshots of the current prototype (M0), rendered at 2x on a neutral backdrop: idle, peek, expanded todo, a placeholder tab, alert, alert waiting in the pill |
| `src/app.css` | Current color/motion tokens (names to keep or map) |
| `src/lib/config.ts` | Current state sizes and timings |
| `src-tauri/icons/128x128@2x.png` | Current app icon (placeholder) |

### Product summary

- **Karsa** means "will, intention" in Javanese and Indonesian ("ing madya mangun karsa"). It lives as a small black pill in the **bottom-right corner** of the screen, just above the taskbar, and grows up and to the left.
- **v1 features:** Todo (multiple lists, subtasks, due dates), Focus (Pomodoro), Notes, Meeting notes (structured: title, date/time, topic, attendees, notes, action items with "Add to todo"), Reminders (manual, with type General/Meeting/Deadline, repeat, lead time, snooze).
- **States and exact sizes** (logical px at 100% scaling; all states anchor to the bottom-right corner):

| State | Trigger | Size | Content |
|---|---|---|---|
| Idle pill | nothing active | 120–220 × 32, radius 16 | Next reminder/deadline today (e.g. "Weekly sync · 14:00"), else the time |
| Live activity | timer running or reminder within 15 min | up to 220 × 32 | One activity, e.g. focus countdown |
| Peek | hover 300 ms | 320 × 172, radius 22 | Date, active timer, next up, top 3 tasks |
| Expanded panel | click | 380 × 520, radius 26 | Tabs: Todo · Focus · Notes · Reminders |
| Alert | reminder fires / focus phase ends | 320 × 112, radius 22 | Title, subtitle, actions (Snooze 5m / Dismiss, Start break / Skip, New meeting note) |
| Alert waiting | alert not acknowledged after 8 s | pill | Accent dot + title |

The window is a fixed **400 × 540** area, so nothing may exceed it. If you need different state sizes, propose them explicitly with a reason.

## Step 3 — Hard constraints (the app must stay lightweight)

The app currently uses ~0% CPU and ~73 MB RAM while idle. The design must keep it that way.

- **The island stays solid black (#000)**, the Dynamic Island DNA. The Javanese nuance comes from accents, small ornaments, typography details, copy and motion, never from heavy textures or photos.
- **Rendering is software-composited WebView2.** No `backdrop-filter`, blur, glassmorphism, large soft shadows, animated gradients, video, Lottie or canvas effects.
- **Motion:** animate only `transform` and `opacity`, plus the existing box width/height morph (spring easing in `src/app.css`). No continuous or looping animation while idle. Provide a reduced-motion variant.
- **Assets:** SVG only, optimized, ideally under 4 KB each, using `currentColor` where possible. Patterns are small SVG tiles. The only raster exports are app icon PNGs. Total asset budget: under 150 KB.
- **Fonts:** prefer the system font (Segoe UI Variable). At most one extra font, as subset `.woff2` (≤ 40 KB) with an OFL license. Any aksara Jawa in the logo must be an SVG path, not a font.
- **Legibility:** dark UI over any wallpaper or app. Text ≥ 11 px, WCAG AA contrast on black, crisp at 100% / 125% / 150% Windows scaling. Icons must read at 16 px (and the tray icon at 16 px on light and dark taskbars).
- **Implementable in plain CSS + inline SVG:** no design that needs a canvas, WebGL or a UI library.

## Step 4 — What to design

1. **Design direction and principles** (short), plus a do/don't list for the Javanese nuance.
2. **Tokens:** colors (background, text levels, surfaces, hairlines, semantic accents for focus / meeting / deadline / general / done / danger), typography scale, spacing (4 px grid), radii, elevation (minimal), motion (durations, easings, including the spring), opacity levels. Keep or map the current names in `src/app.css` (`--island-bg`, `--fg`, `--muted`, `--faint`, `--hairline`, `--surface`, `--surface-hover`, `--accent`, `--focus`, `--spring`, `--morph-duration`).
3. **Ornament system:** which motif, where it appears (e.g. a tiny separator, an empty-state mark, a header watermark at ≤ 6% opacity) and the rules of restraint.
4. **Components and screens**, every state listed above plus:
   - Idle pill variants: time, next-up item, running focus timer, alert waiting.
   - Peek card.
   - Expanded panel: header, tab bar, and each tab:
     - **Todo:** list switcher, task with subtasks (progress like 2/5) and due date, quick add, completed section, empty state.
     - **Focus:** timer display (ring or bar), controls, phase label, today's count.
     - **Notes:** list with All / Notes / Meetings filter, search, editor.
     - **Meeting note:** structured header fields, notes, action items with "Add to todo" / "Add all".
     - **Reminders:** grouped by day, type badge, repeat indicator, add/edit form.
   - Alerts: general, meeting (with "New meeting note"), deadline, focus phase end, missed-reminders summary.
   - Interaction states for every control: hover, pressed, focus-visible, disabled.
5. **Motion spec** for idle→peek, peek→expanded, collapse, alert arrival, alert collapsing into the pill, tab switch.
6. **App icon** (1024 px master) and a **tray icon** optimized for 16 and 32 px, plus a wordmark.

## Step 5 — Deliverables

Package everything as one zip that I will unpack into the project's `design/` folder:

```
design/
  DESIGN.md            the spec: direction, principles, token tables, component specs
                       with exact px, motion spec, copy deck, Javanese do/don't
  tokens.css           :root custom properties, drop-in for src/app.css
  tokens.json          the same tokens, machine-readable
  assets/
    logo-mark.svg
    logo-wordmark.svg          (Latin, plus aksara Jawa variant if chosen)
    icon-app.svg               master
    icon-app-1024.png
    icon-tray-16.svg
    icon-tray-32.svg
    icons/*.svg                16 px grid, currentColor: tab icons, alert types,
                               play, pause, skip, stop, add, check, delete,
                               calendar, repeat, search, note, meeting…
    ornaments/*.svg            motif tiles and separators
  fonts/                       only if a font is chosen: *.woff2 + LICENSE.txt
  screens/*.png                2x mockups of every component and state above,
                               numbered like the files in design/context/
  prototype/                   optional: static HTML/CSS of the screens using
                               tokens.css, no frameworks, no CDNs
```

Rules for the deliverables:

- All measurements in logical px at 100% scaling; all colors in hex (with alpha where used).
- Every screen in DESIGN.md lists the tokens and assets it uses.
- The **copy deck** lists every UI string in English, plus any Javanese phrase with its meaning and where it appears.
- No external URLs, CDNs or remote fonts. Everything ships inside the zip.
- SVGs are optimized (no editor metadata, no embedded rasters).
