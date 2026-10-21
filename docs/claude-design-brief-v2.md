# Brief for Claude Design: Karsa design system v2

This continues the Karsa design project in which you made **design system v1** (`design/DESIGN.md`, `tokens.css`, the mripat eyes, the kawung ornaments, screens 01–26). v1 has been built and is in daily use. v2 does five things:

1. **Refine** what exists, after an honest audit of v1 against the app as built.
2. Design **every function to best practice**: how taking a note, running a meeting note, adding a task, capturing a thought and the rest should flow, so each one is quick, clear and a little fun to use, while the app stays light.
3. Give Karsa a **character**: the mripat eyes with a **blangkon**, and more expressions.
4. Introduce a **shape language**: each function gets the island shape its content needs.
5. Design **M8**: Now playing (with lyrics and a lyric focus mode) and a Karsa **calendar**.
6. Leave room for **future modules** without redesigning everything.

Your output will again be implemented by Claude Code in the existing Tauri 2 + Svelte 5 codebase, so every deliverable must be precise: exact sizes, hex colours, named tokens, SVG assets.

**Language:** talk with me in **Bahasa Indonesia**. Everything you produce (UI copy, token names, file names, DESIGN.md) is in **English**. Javanese words appear only as deliberate accents, each with its meaning in the copy deck.

**If this session still has the v1 work,** continue from it. Otherwise read the attached files first (Step 2). Keep what works: change v1 only where the audit finds a reason and I agree.

## Step 1 — Interview me before designing

Ask in Bahasa Indonesia, **at most 4 questions per round**, each with concrete options and small sketches where they help. Never fill in my answers. Cover at least:

1. **The audit.** Show me what you see in the current screenshots: what works, what feels off, what is inconsistent between v1's mockups and the app as built. Ask which points I want fixed.
2. **How I use each function.** Ask, a few functions per round, how I take notes and meeting notes, add tasks, use focus, reminders, the shelf and quick capture: what I do most, what annoys me, and what would make it feel fun. Show me what best practice looks like for that kind of tool (good note apps, meeting tools, task managers, timers, clipboard managers), with options I can pick from.
3. **Karsa's character.**
   - Eyes only, or eyes with a **blangkon**? If a blangkon, which style: Yogyakarta (with the *mondolan* bump at the back), Surakarta (flat, *trepes*), or a simplified one of your own? Batik on it (kawung, parang) or plain?
   - Where it appears: in the 26–32 px pill, or only in bigger places (panel header, empty states, the app icon, special days such as the birthday and weton day)?
   - Personality: calm and dignified, cheeky, cute? How "face-like" (eyebrows, cheeks, a mouth) versus abstract?
   - Which new expressions, and when each appears. Candidates: *nyanyi* (singing, while music plays), *kaget* (surprised, an alert arrives), *ngantuk* (sleepy, late at night), *seneng* (pleased, a task is done), *mikir* (thinking, while something loads). Existing: melek, kedhep, fokus, leren, mesem, merem, bungah, mrengut.
4. **Shape language.** Confirm that Now playing is **wide and short**, the calendar and note editing are **tall**, and alerts stay compact. How dramatic should the difference be? How should a tab switch morph from one shape to another?
5. **Lyric focus mode.** How one lyric line looks in the idle pill and in the notch: a maximum width, cut off with an ellipsis or a smaller font, eyes beside the line or not, a karaoke-style fill or a plain line, how a new line arrives (cross-fade, slide).
6. **Now playing.** Art size, how the source shows (app icon plus "YouTube Music · Chrome"), how many controls are visible, whether volume is a slider or a button, and how to switch between two apps that both have media.
7. **Calendar.** Which Javanese layers: pasaran under every date, the wuku, a mark on my weton day and birthday, Javanese day names (Senin Legi…) in the agenda. How busy a day may look (dots, a count, a tint), how focus minutes show (the kain's kawung, a bar, nothing), and whether **Reminders becomes Calendar** (one tab with Month and Reminders views) or the calendar is a separate tab.
8. **Corner versus notch.** Anything that should differ between the two positions.
9. Things I love or dislike since v1, and any references (apps, images, Dynamic Island moments).

Then summarise my answers as a short **v2 direction** and wait for my OK. Next, show **2–3 explorations** for each of:

- Karsa with a blangkon: idle pill (corner and notch) and panel header.
- Now playing: the wide shape, with and without lyrics, and lyric focus in the pill.
- Calendar: the month view with a day agenda.

Let me pick, then build the full spec. Show the per-function UX proposals (Step 4.2) as short flows I can approve one function at a time.

## Step 2 — Context (attached files)

I attach one zip, `karsa-design-v2-context.zip`, with these files at their project paths:

| File | What it tells you |
|---|---|
| `design/DESIGN.md`, `design/tokens.css`, `design/tokens.json` | v1 as delivered. `tokens.css` already has the notch tokens added in M7 (`--notch-*`, `--radius-notch`, `--edge-notch`) |
| `design/screens/*.png` | v1 mockups, 01–26 |
| `design/assets/eyes/*.svg`, `src/assets/eyes/eyes-bungah.svg`, `eyes-mrengut.svg` | The current expressions (bungah and mrengut were drawn by Claude Code in v1's style) |
| `design/context-v2/*.png`, `design/context-v2/README.md` | Screenshots of the app as built today, at 2x, in both positions (`corner-*`, `notch-*`), plus three from the UI preview; the README says what each one shows |
| `docs/requirements.md` | The full spec. Read 4 (the island), 6b (M6), 6c (M7, the notch) and **6d (M8, the plan for this work)** |
| `README.md` | How the island works technically |

### What has been built since v1

- **M6, personal companion and tools:** the greeting with my name ("Sugeng siang, Faza"), today's pasaran and a paribasan of the day; mripat that glance at the cursor; bungah and mrengut; the focus **kain** (four weeks of focus as kawung cells) and streak; the **Rampung** end-of-day ritual; **quick capture** (Win+Shift+K); a **Shelf** tab with a 24-hour clipboard history and a **file shelf** that shows each file's real Windows icon (colourful, e.g. PowerPoint's); "every N minutes" reminders; media in Peek (a Playing row) and in the pill (album art and title).
- **M7, the notch:** Settings › Position puts Karsa at the **top centre** as a notch: 80 × 26 idle, flush with the top edge (square top corners, 12 px bottom radius), eyes 24 × 12, text up to 200 px, an edge without its top side. Every bigger state hangs from it and grows downwards. Peek opens after 450 ms there (300 ms in the corner).
- **Drop target:** files dragged over the pill grow it into a 320 × 112 "Drop to keep on the shelf" card.

### Current shapes (logical px)

| State | Corner | Notch |
|---|---|---|
| Idle pill / live activity | 72–220 × 32, radius 16 | 80–200 × 26, radius 0 0 12 12 |
| Peek | 320 × 172 (+26 with media), radius 22 | same, square top corners |
| Alert, drop target | 320 × 112, radius 22 | same, square top corners |
| Panel (every tab) | 380 × 520, radius 26 | same, square top corners |

### M8 in short (details in requirements 6d)

- **Now playing:** every app that has media in Windows (Spotify, browsers, Media Player…), with the source app's real icon and, for a browser, the site when Karsa can tell ("YouTube Music · Chrome"). Play/pause, previous, next, a seek bar with times, shuffle and repeat **only when the app supports them**, the app's volume, a switcher when several apps have media. Opened by clicking the media pill or Peek's Playing row.
- **Lyrics:** timed lyrics (LRC) from LRCLIB, **opt-in** because it is Karsa's first use of the internet. The current line is highlighted; untimed lyrics are plain text; ±0.5 s timing nudge. States: loading, found, plain only, not found, off (with a short privacy note and a button to turn it on).
- **Lyric focus mode:** while on, the idle pill or notch shows the current lyric line, one line only, so it covers little of my work.
- **Calendar:** a month view (Monday first) built from Karsa's own data: task deadlines, reminders and meetings, meeting notes, focus minutes, plus pasaran per day and the wuku. Selecting a day shows its agenda, with quick add. No outside calendars in M8.
- **A shape per function:** see Step 4.3.

### Later (design so these fit without a redesign)

Outlook meetings (read-only), the Javanese month and year (sasi, taun) and Indonesian public holidays in the calendar, and other small modules. A clear shape language and component kit should make a new module a matter of picking its shape and parts.

## Step 3 — Hard constraints (Karsa must stay light)

Idle today: about 0% CPU and 70 MB of memory. Keep it that way.

- **The island stays solid black (#000).** Character comes from accents, the mripat, small ornaments, copy and motion, never from textures, photos or gradients.
- **Rendering is software-composited WebView2.** No `backdrop-filter`, blur, glassmorphism, big soft shadows, animated gradients, video, Lottie, canvas or WebGL.
- **Motion:** animate only `transform` and `opacity`, plus the box morph (width, height, border-radius on the spring in `tokens.css`). **Nothing loops while idle.** A lyric line change, a track change or an expression change is an event, so a short transition then is fine. Any "playing" animation (an equaliser) must be optional and cheap enough to be measured and dropped if it costs CPU. Give a reduced-motion variant of everything.
- **The window never resizes at runtime.** It is a fixed transparent area, and only the box inside morphs. Today it is 400 × 540; v2 may enlarge it to cover the largest shape, **at most about 560 × 640**. Propose the exact envelope. In the corner it is anchored bottom-right, so shapes grow up and to the left. As a notch it is anchored top-centre, so shapes grow down and to both sides. Every shape must morph to and from the pill.
- **Legibility:** text ≥ 11 px, WCAG AA on black, crisp at 100 / 125 / 150% scaling, icons readable at 16 px, the tray icon at 16 px on light and dark taskbars.
- **The mripat must read at pill size** (24–28 px wide). If the blangkon cannot read there, show where it appears instead. Draw the blangkon as a **separate layer** that fits every expression, so expressions × headwear do not multiply into many files.
- **Assets:** SVG only, optimised, ideally under 4 KB each, `currentColor` where it makes sense, no embedded rasters. Total budget for new assets: under 100 KB. Album art and the shelf's file icons are rasters supplied by Windows; design around them, do not restyle them.
- **Fonts:** the system font (Segoe UI Variable) unless v1's audit argues otherwise; any aksara Jawa is an SVG path.
- **No real song lyrics** in mockups (they are copyrighted): use placeholder lines such as "The line being sung right now".
- **Fun, but light.** Delight comes from things that cost nothing while idle: the mripat reacting to an event, a short spring or fade when something happens, a well-chosen Javanese phrase, a satisfying check-off, a small reward in the kain or Rampung. No confetti engines, particle effects, sound, or animation that keeps running. Every delightful moment is listed in the motion spec with its trigger and duration.
- **Best practice, not novelty.** Prefer patterns people already know from good tools (keyboard shortcuts, autosave, undo, inline editing, sensible defaults) over clever new ones, and say which tool or guideline a pattern follows.

## Step 4 — What to design

1. **v2 direction and the audit:** what changes from v1 and why, as a short list I can approve.
2. **A UX playbook for every function.** For each one, how it should work at its best: the main flow step by step, the fastest path (keyboard shortcuts included), defaults, empty / loading / error / undo states, and its "fun moment", each checked against the constraints in Step 3. Cover:
   - **Notes:** starting a note in one step, autosave, the title from the first line, light Markdown (lists, checkboxes, headings) as you type, search, finding a note again, linking a note to a task.
   - **Meeting notes:** before, during and after a meeting: opening one from a meeting reminder, attendees as chips, the date and topic, notes while people talk, turning lines into action items, sending action items to Todo, and finding past meetings.
   - **Todo:** quick add (and whether to understand dates typed in plain words, such as "tomorrow 5pm"), subtasks, due dates, reordering, finishing, overdue tasks, lists.
   - **Focus:** starting in one click, what you are focusing on, breaks, pausing, the end of a session, the kain and streak as motivation without guilt.
   - **Reminders and the calendar:** adding one fast, repeats and "every N minutes", snoozing, missed reminders, how deadlines from Todo appear.
   - **Shelf:** finding and reusing copied text, privacy (pause, clear), getting files in and out.
   - **Quick capture:** from any app, with zero friction, and back to work.
   - **Rampung:** the end-of-day ritual as something to look forward to.
   - **Now playing and lyrics:** glanceable, controllable without opening the panel.
   - **Settings:** clear, with each setting explaining itself.
   - **Alerts:** never in the way, easy to act on or dismiss.
3. **Karsa's character sheet:**
   - Eyes and blangkon at every size used (pill, Peek, panel header, empty states, app icon, special days).
   - The full expression set (existing plus new), each with its trigger, priority and duration, plus blink and glance rules.
   - How the character behaves in the corner and in the notch.
4. **Shape language:** one table of every state and function with its size, radius, and corner/notch anchoring. Cover:
   - The idle pill, live activity, and the lyric focus line.
   - Peek, alert, the drop target.
   - Now playing (compact and with lyrics).
   - Calendar, Todo, Focus, Notes list, the note and meeting-note editor, Shelf, Settings, Rampung, quick capture.

   Then specify how a tab switch morphs between shapes (timing, which edge stays put, how content fades).
5. **M8 screens**, every state:
   - **Now playing:**
     - The pill/notch compact view (art, title, optional thin progress line) and lyric focus (one line).
     - Peek's Playing row with progress.
     - The wide Now playing view: art, title, artist, album, source, seek bar with times, controls (hidden when unsupported), volume, the session switcher.
     - The lyrics area: timed, plain, loading, not found, off with the privacy note.
     - The Settings rows: Lyrics (opt-in) and Lyric focus.
   - **Calendar:**
     - The month grid: date and pasaran, dots for deadline / meeting / reminder, today, selected, weekend, weton and birthday marks, focus days, the wuku.
     - Month navigation and "Today".
     - The day agenda: overdue, deadlines with a checkbox, reminders and meetings by time, meeting notes, focus minutes, tasks done, "+ Task" and "+ Reminder".
     - The Reminders view and the switch between views.
     - Empty days and empty months (with a paribasan).
   - Hover, pressed, focus-visible and disabled states for every new control.
6. **Refreshed screens** for every function whose flow changes in the playbook, and wherever the audit changes something.
7. **Motion additions:** the shape-to-shape morph, a lyric line change, a track change, a month change, and expression changes.
8. **Copy deck additions:** every new English string, and every Javanese phrase with its meaning and where it appears.

## Step 5 — Deliverables

One zip that I unpack into the project's `design/` folder, replacing v1 (unchanged files keep their names):

```
design/
  DESIGN.md        v2 spec, starting with "Changes from v1"; the UX playbook per
                   function; shape table, character sheet, component specs with
                   exact px, motion, copy deck
  tokens.css       v2 tokens, drop-in (keep v1 names; mark any renamed or retired ones)
  tokens.json      the same, machine-readable
  assets/
    mascot/        eyes per expression + blangkon layer(s), currentColor where possible
    icons/*.svg    new 16 px icons: shuffle, repeat, repeat-one, volume, volume-mute,
                   lyrics, calendar, chevrons, source/app fallback, …
    ornaments/     any new ornament
    icon-app.svg, icon-app-1024.png, icon-tray-16.svg, icon-tray-32.svg  (if they change)
  screens/*.png    2x mockups, numbered from 27 onwards, each state above, in the corner
                   and as a notch where they differ
  prototype/       optional static HTML/CSS using tokens.css, no frameworks, no CDNs
```

Rules:

- Measurements in logical px at 100% scaling; colours in hex (with alpha where used).
- Every screen in DESIGN.md lists the tokens and assets it uses.
- No external URLs, CDNs or remote fonts; everything ships in the zip.
- SVGs optimised: no editor metadata, no embedded rasters, unique ids per file (clipPaths and gradients would clash when inlined).
