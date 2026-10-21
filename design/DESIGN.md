# Karsa design system v2

v2 builds on v1 and M7: the black island, the kuningan accents, Segoe UI Variable, the kawung and the mripat. It adds a character, a shape for each function, Now playing with lyrics, and a calendar. Everything v1 specified still applies unless this file changes it.

All sizes are logical px at 100% scaling, and colours are hex. `tokens.css` keeps every v1 name and marks new ones `v2`. The screens in `screens/27-53` are 2x renders of `prototype/v2.html`. They were captured without Segoe UI, so the type there uses a fallback.

## 0. Changes from v1

| # | Change | Why |
|---|---|---|
| C1 | **Window envelope 560 × 640** (v1: 400 × 540). Each function gets its own shape inside it (section 3). | Audit A3/A4: one 380 × 520 panel left Todo, Capture and Rampung half empty and squeezed the meeting note. |
| C2 | **Karsa's character:** the mripat wear a Yogyakarta blangkon (with mondolan) in the notch, Peek, Focus, empty states and the app icon. Eyebrows appear at 48 px and above. The corner pill and the panel header keep eyes only. | User choice. The blangkon is a separate layer, so 12 expressions × 2 hats do not multiply into files. |
| C3 | **Four new expressions:** nyanyi, kaget, seneng and gumun, each with a trigger, a priority and a duration (section 2). | They respond to events only. Nothing loops. |
| C4 | **Cheeky but quiet.** With overdue tasks, hovering the pill makes the eyes blink, then glance mrengut for 1.6 s. There is no text and no nagging. | User choice: "usil" limited to a glance. |
| C5 | **One selector system (A5).** Segmented for switching views, chips for filters, Select for a choice from a list, Switch for on/off. The loose "Into Inbox ⌄" text becomes a Select. | Three styles did the same job. |
| C6 | **All text buttons are 28 high (A6).** The large "New reminder" (36) and "New" (32) buttons become 28. Only the Start focus button stays 36. | Consistency. |
| C7 | **Media shows its source and progress (A8).** The pill and the Peek Playing row show a 2 px progress line. Peek shows the source app's icon. | Audit A8. |
| C8 | **Even kain (A9).** All cells are the same size (12 px, gap 4). The level shows in colour: sogan-deep, sogan, kuningan. An empty day is a 3 px dot. | Cells of different sizes looked random. |
| C9 | **Calendar is a new icon tab** next to Shelf and Settings. Tab gap is 16 (v1: 20). | User choice. The four text tabs and three icon tabs fit in 348 px. |
| C10 | **Quick capture is a 360 × 140 card**, not the full panel. | Audit A10. |
| C11 | **Rampung ends with a closing moment** (section 4.8). | User choice. |
| C12 | **App icon v2:** Karsa's head with the kawung blangkon on the black squircle. The tray icons are unchanged. | User choice: the character on the icon. |
| C13 | **Focus has quick lengths** (15 / 25 / 50), and its shape is 380 × 500. | User choice. |
| Kept | Palette, type scale, spacing, radii, v1 eyes geometry, the kawung, the aksara watermark (A1 was not selected), the greeting and pasaran in the header, the alerts. | |

Retired: nothing. `assets/eyes/` (v1, plus `eyes-bungah` and `eyes-mrengut` as built) stays for the code that uses it. `assets/mascot/` holds the v2 set; switch `Eyes.svelte` to it, then `eyes/` can go.

## 1. Principles (v2 additions)

1. **A shape per job.** The island is as big as its content and no bigger. Every shape morphs to and from the pill.
2. **Character reacts, never performs.** Every expression change has an event behind it, a fixed duration, and a reduced-motion version.
3. **Known patterns first.** Each flow below names the tool or guideline it follows.
4. **Light stays light.** No new continuous work while idle. The equalizer is opt-in and is measured before it ships.

## 2. Character sheet (screen 27)

### 2.1 Anatomy and layers

All mascot files live in `assets/mascot/` and are drawn for `--island-bg`.

| Layer | File(s) | viewBox | Colours |
|---|---|---|---|
| Eyes | `eyes-<expression>.svg` × 12 | 28 × 14 | sclera `--eye-sclera`, iris `--eye-iris`, glint `#FFF`, blush `--eye-blush`, sparkle and note `--mascot-accent` |
| Blangkon, small | `blangkon-plain.svg` | 28 × 11 | dome `--blangkon` #C08552, band, mondolan and pleat `--blangkon-deep` #8A5A2E |
| Blangkon, large | `blangkon-kawung.svg` | 28 × 11 | as above, plus a kawung pattern in `--blangkon-deep` at 75%, clipped to the dome (`kb-dome`) |
| Brows | `brows-neutral`, `-raised`, `-soft`, `-frown`, `-focus` | 28 × 4 | stroke `--brow` 1.1 |

The blangkon is the Yogyakarta style: a rounded dome, an iket band across the forehead, a front pleat (*wiru*), and the **mondolan** knot behind at the right.

**Composition** (units are the 28-wide grid; scale the whole group):

| Head | Frame | Blangkon | Brows | Eyes | Used at |
|---|---|---|---|---|---|
| Eyes only | 28 × 14 | none | none | y 0 | corner pill (28), panel header (24) |
| Small head | 28 × 21 | plain, y 0, drawn **above** the eyes | none | y 7 (the band overlaps the top of the eyes by 2.3) | notch (24 × 18), Peek (22 × 16.5), lyric-focus notch, loading lyrics (28) |
| Large head | 28 × 26 | kawung, y 0 | y 9 | y 12 | Focus (56 × 52), empty states (48 × 44.6), Rampung (84 × 78), app icon |

Each head is a `<svg viewBox>` holding three nested `<svg>` layers at those offsets (see `prototype/v2.html`). The clipPath ids are unique per file (`kf-*`, `kl-*`, `km-c`, `kb-dome`). When the same file is inlined twice, the duplicate ids are identical definitions and resolve the same way.

### 2.2 Expressions

Priority runs top to bottom. A higher state replaces a lower one, and a timed state returns to whatever the lower states say once it ends.

| Expression | Eyes | Brows | Trigger | Duration | Where |
|---|---|---|---|---|---|
| **kaget** (surprised) v2 | round, small iris | raised | An alert arrives | 600 ms. In the pill, 240 ms before the alert morph starts (section 5). | pill, notch, open panel header |
| **seneng** (pleased) v2 | ^ ^ with stronger blush | soft | A task is checked anywhere | 1200 ms | panel header, Peek, pill if visible |
| **gumun** (amazed) v2 | big, kuningan star glints | raised | A finished session extends the streak, or sets a new best week in the kain | 1600 ms | Focus tab, Peek, pill |
| **mrengut** (sulking) | half lid, iris down-left | frown | 2 min after a focus phase is stopped early (M6). **v2:** a 1.6 s glance after hover-blink while tasks are overdue, at most once per 10 min. | 2 min / 1.6 s | everywhere |
| **nyanyi** (singing) v2 | happy arcs, kuningan note | raised | A track starts playing (not on resume). Also static beside the line during lyric focus. | 1600 ms / while lyric focus shows | pill, notch, Peek |
| **merem** (asleep) | closed arcs, no blush | soft | 22:00–05:00 and nothing due within 60 min | state | everywhere |
| **bungah** (proud) | arcs, sparkle | raised | Birthday, weton day, or Rampung done today | state | everywhere |
| **fokus** | kuningan lids, iris down | focus | Focus phase running | state | everywhere |
| **leren** (resting) | flat lids | soft | Break running | state | everywhere |
| **mesem** (smiling) | soft arcs | soft | All of today's tasks done | state | everywhere |
| **kedhep** (blink) | lines | neutral | `mouseenter` on the pill or notch | 140 ms, once | pill, notch |
| **melek** (awake) | default | neutral | none of the above | state | everywhere |

**Rules**

- **Blink:** only on hover enter, and once after any timed expression ends. Never on a timer.
- **Glance (M6, kept):** the pupils shift up to 1.3 px toward the cursor on `mousemove` over the island, at most once per frame.
- **Change:** crossfade the two SVGs for `--dur-expression` (160 ms). Kedhep scales the eye layer on Y, 1 → 0.18 → 1. The blangkon and brows layers do not move, and brows swap with the eyes.
- **Reduced motion:** expressions swap instantly and holds keep their length. There is no blink and no glance motion.
- **Corner vs notch:** the corner pill shows eyes only (28 × 14). The notch shows the small head (24 × 18) in every eyes state. Peek shows the small head in both positions. The panel header shows eyes only (24 × 12).

## 3. Shape language (screen 52)

**Window envelope:** 560 × 640, fixed. In the corner it is anchored bottom-right of the work area; as a notch, top-centre. Each shape keeps `--overshoot-pad` (10 px) from the envelope edges, so the largest content is 540 × 620. The window region still clips to the current shape plus the overshoot.

| State / function | Corner w × h | Radius (corner) | Notch | Notes |
|---|---|---|---|---|
| Idle pill (eyes) | 72 × 32 | 16 | 80 × 26, 0 0 12 12 | small head in the notch |
| Live activity (focus, reminder, alert waiting, media) | 72–220 × 32 | 16 | 80–200 × 26 | media: art 20 (notch 16) and a 2 px progress line |
| Lyric focus line | up to 220 × 32 | 16 | up to 200 × 26 | nyanyi eyes + one line, ellipsis |
| Peek | 320 × 172 | 22 | same, top 0 | |
| Peek + Playing row | 320 × 210 | 22 | same | |
| Alert, drop target | 320 × 112 | 22 | same | |
| Quick capture | 360 × 140 | 22 | same | |
| Now playing | 520 × 212 | 26 | same | wide and short |
| Now playing + lyrics | 520 × 420 | 26 | same | |
| Todo, Notes list | 380 × 520 | 26 | same | v1 size |
| Focus | 380 × 500 | 26 | same | |
| Note editor, meeting note | 420 × 620 | 26 | same | tall |
| Calendar | 400 × 600 | 26 | same | tall |
| Shelf | 400 × 520 | 26 | same | |
| Settings | 400 × 560 | 26 | same | |
| Rampung | 380 × 480 | 26 | same | |

In the notch, every shape keeps square top corners and the bottom radius from this table, and uses `--edge-notch`.

**A new module** picks the nearest shape: card (320–360 wide, ≤ 140 high), wide (520 wide, 212–404 high), panel (380–400 wide, 480–560 high), or tall (400–420 wide, 600–620 high). It composes from the parts in section 6. Nothing else needs a redesign.

### 3.1 Shape-to-shape morph (tab switch, opening a view)

| Step | Time | What |
|---|---|---|
| 1 | 0–100 ms | Old body content fades out (`--dur-content-out`, opacity 1→0, scale 1→0.98). Header and tab bar stay visible. |
| 2 | 0–700 ms | The box morphs width and height on `--spring` (700 ms) and border-radius on `--ease-out` (350 ms). **Corner:** the bottom-right corner stays put, so it grows up and left. **Notch:** the top edge stays centred, so it grows down and to both sides. Header and tabs move with the top edge (corner) or stay (notch), with no fade. |
| 3 | from 90 ms | New body enters on the gamelan beat: rows at 90, 150, 210, 270, 330 ms (`--beat` 60, max 5), each 220 ms with opacity 0→1 and scale 0.94→1. |
| Region | | Grows to the union of old and new shapes before step 2, and shrinks 450 ms after the morph settles (v1 rule). |
| Same size | | Tabs with equal shapes (Todo ↔ Notes list) skip step 2 and crossfade the body (80 ms out, 160 ms in, translateY 4→0). |
| Reduced motion | | Instant swap. |

## 4. UX playbook

Each function below gives the main flow, the fastest path, defaults, its states, and its fun moment. Every pattern is checked against Step 3: no idle work, only event-driven updates, motion on transform and opacity only.

### 4.1 Notes (screens 42, 43)

- **Start in one step:** **Ctrl+N** anywhere in the panel, or "New" in Notes, opens an empty editor with the caret on line 1. The first line becomes the title; there is no title field. (Pattern: Apple Notes, Bear.)
- **Autosave:** 600 ms after the last keystroke, plus on blur. The status line shows "Saved" or "Saving…". There is no Save button. If the file changed elsewhere, the v1 conflict prompt appears.
- **Live Markdown:** it renders as you type.
  - `# ` → H1 (17/24 600) and `## ` → H2 (15/22 600).
  - `- ` → bullet, `[ ] ` → checkbox, `[x]` → checked.
  - The Markdown mark stays visible in `--disabled` while the caret is on that line, and hides otherwise. (Pattern: Bear, Typora.)
  - The file stays plain `.md`.
- **Link a task:** "Link a task" chip under the bar, or type `[[` to pick one.
  - The linked task shows as a chip: link icon, title, due date. The task row shows a "1 note" meta item (screen 44).
  - Clicking either side opens the other.
- **Checkbox to Todo:** hovering a checkbox line shows "To Todo". After sending, the label changes to "In Todo" in `--done`.
- **Search:** full text over titles and bodies as you type, debounced 120 ms. Matches are highlighted with `--search-hit`, and the snippet is centred on the first hit. "3 notes" sits on the right. Esc clears.
- **Empty:** "No notes yet", with "Ctrl+N starts one" in faint below it. **No results:** "No notes match "{query}"".
- **Undo:** Ctrl+Z in the editor. Deleting a note shows "Note deleted · Undo" for 5 s (pattern: Gmail undo send).
- **Fun:** seneng on the first note of the day.

### 4.2 Meeting notes (screen 41)

- **Before:**
  - The meeting alert's "+ Meeting note" opens the editor with the title, When, and attendees from the reminder.
  - For a **recurring** meeting (the same title on a repeating reminder), "Last time · {date}" shows the previous note's open action items. Each one can be ticked or left to carry over. (Pattern: Fellow, Notion recurring docs.)
  - The card collapses with the chevron and stays collapsed per series.
- **During:** free typing in Notes, in the tall editor (420 × 620).
  - A line that starts with `[ ] ` becomes an action item in place. A date word at its end ("fri") is parsed, shown in faint, and becomes the task's due date.
  - Attendees are chips; "+" adds one, with autocomplete from earlier meetings.
- **After:** the footer reads "Action items · N", followed by a **Select** (list, default Inbox) and the primary "Send N to Todo".
  - Sent items show "In Todo".
  - Ticking an item in Todo ticks it in the note too (they are linked).
- **Find:** the Notes "Meetings" chip, search by title, topic or attendee name, and the meeting dots in the Calendar day agenda.
- **Fun:** gumun when every action item from last time is done.

### 4.3 Todo (screen 44)

- **Quick add with natural dates:** "Send the deck to Rina fri 5pm".
  - The date phrase is highlighted in the input (`.parse`: `--accent` on `--focus-tint`), and a hint chip shows the result ("Fri 23 Oct · 17:00").
  - **Enter** adds the task.
  - **Esc** once removes the highlight and keeps the words as plain text.
  - Understood phrases: today, tonight, tomorrow, weekday names, "next week", "in 3 days", 5pm, 17:00, "23 oct". (Pattern: Todoist, Things.)
- **Priority:** a star on the right of each row, toggled by click or **S**. Starred tasks sort above the others within their due group and show the `star-filled` icon in `--priority`. One level only.
- **Reorder:** drag by the handle that appears on hover at the left (`drag-handle`, 16, `--faint`), or **Alt+↑/↓**.
  - While dragging, the row lifts with scale 1.02, and a 2 px kuningan line marks the drop point.
  - Order is kept per list.
- **Keyboard:** ↑/↓ select, Space checks, Enter edits inline, Delete deletes (with undo), S stars.
- **Finish:** the checkbox fills over 180 ms (`--dur-check`), the row stays for 1.5 s with "Undo", then moves to Completed. The eyes go seneng.
- **Overdue:** the due label shows "Yesterday" or a weekday in `--deadline`. Cheeky glance only (C4).
- **Empty:** the v1 proverb state.

### 4.4 Focus (screen 45; running states from v1 13)

- **One click:** "Start focus" in the tab, the play button in Peek, or **Ctrl+Shift+F** (new hotkey, off by default).
- **Length:** a segmented control (15 / 25 / 50) under the timer, which remembers the last choice. Settings sets the default and the break lengths.
- **Running:** fokus head, timer, bar, then stop / pause / skip (v1). The pill shows fokus and the countdown.
- **End:** the phase-end alert is silent (v1) and uses mesem eyes. The kain cell for today fades to its new level (400 ms). If the streak grew, gumun plays.
- **Kain:** 4 rows × 7 cells (Mon first).
  - Cell size `--kain-cell` 12, gap 4.
  - Each cell is a kawung mark at 10 px, coloured by level: `--kain-1` (1–24 min), `--kain-2` (25–74), `--kain-3` (75+). Empty days are a 3 px `--kain-0` dot.
  - Today has a 1 px `--fg` ring.
- **Streak:** "N days in a row". A missing day resets it with no message and no red. Weekends without focus do not break it.
- **Pause:** has no penalty. A session counts once its focus phase is complete.

### 4.5 Reminders and the calendar (screens 37–40)

- **Quick add with plain words:** "+ Reminder" in the Calendar agenda or the Reminders tab takes one line: "standup weekdays 9am", "water every 30m 7am-5pm", "pay rent 25th". The parsed result shows as a hint chip, and the form (v1 19) stays one click away for details. (Pattern: Todoist recurring dates, Fantastical.)
- **Repeats:** daily, weekdays, weekly, monthly, every N minutes. In the calendar, every-N reminders show once per day as one line ("Drink water · every 30 min, 07:00–17:00").
- **Snooze:** "Snooze 5m" in the alert (v1). Right-click Snooze for 15m, 1h, Tomorrow 09:00.
- **Missed:** one summary alert (v1 24).
- **Deadlines from Todo** appear as Deadline items in the agenda, tickable there.
- **Calendar:** month view, details in section 6.3.
  - Moving around: ‹ › buttons, **PgUp/PgDn**, and "Today" (**T**). Arrow keys move the selected day and Enter opens quick add.
  - **Empty month:** the large merem head, the paribasan of the day and "Nothing planned this month."
  - **Empty day:** "Nothing on {day}" with the two add buttons.

### 4.6 Shelf (screen 48)

- **Type to search:** opening Shelf puts focus in the search field, so typing filters at once and highlights matches. **Enter** copies the top match and closes the panel; ↑/↓ choose another. (Pattern: Raycast clipboard history, Ditto.)
- **Pin:** the pin icon on hover, or **P**. Pinned items sit under "Pinned", are kept past 24 hours until unpinned, and are still encrypted.
- **Privacy:** Pause stops recording (v1). While paused, the clipboard segment shows "Paused" in `--faint`. Clear asks once: "Clear 24 hours of copied text? Pinned items stay."
- **Files:** v1 behaviour: drop, paste, Add files…, drag out.
- **Empty:** "Copied text from the last 24 hours shows here."

### 4.7 Quick capture (screens 46, 47)

- **Win+Shift+K** opens a 360 × 140 card straight from the pill, morphing from the pill's anchor. Focus goes into the input.
- **Layout:**
  - Segmented Task / Note (24 high) and the shortcut (faint) on the right.
  - The input (36), with live date parsing for tasks.
  - A hint chip plus "Enter saves to Inbox · Tab: note · Esc".
- **Enter** saves and closes. A seneng blink shows in the pill as it returns, and focus goes back to the previous app (v1).
- **Tab** switches to Note. A note uses the first line as its title, and Shift+Enter adds lines; the card grows to 360 × 220 once there are 3+ lines.
- **Esc** discards with no prompt when empty, or keeps a draft for 10 min when text was typed.

### 4.8 Rampung (screen 49; the list step stays v1 19)

- The offer alert, the list step and "Tomorrow" are as built. Pressing **Rampung** shows the closing moment in the same 380 × 480 shape:
  - The large bungah head (84).
  - "Rampung. Matur nuwun." (title-lg).
  - "N tasks done · M min focus".
  - "This week" with seven kain cells. Today's cell weaves in: opacity 0→1 and scale 0.6→1 over 400 ms, at 300 ms.
  - The paribasan of the day with its meaning, in a `--surface` card.
  - "Close" (secondary, 28).
- Esc or Close closes the panel, and the eyes stay bungah until midnight (M6).

### 4.9 Now playing and lyrics (screens 28–36)

- **Glance:** while a song plays and nothing else needs the pill, the pill shows art 20, the title and the 2 px progress line. Peek's Playing row shows the source icon (12), the title, previous / play-pause / next, and the progress line.
- **Open:** click the media pill or the Playing row. This opens Now playing (520 × 212).
- **Source row:**
  - The real app icon (16) and its name, with the site for a browser: "YouTube Music · Chrome".
  - A chevron. With 2+ sessions, a count badge appears and the row becomes a Select (screen 36) that lists every session with its state.
  - Lyrics toggle and Close on the right.
- **Main row:** art 56 (radius 8), title (title 15/600), then "Artist · Album" (muted).
- **Seek:** a full-width 4 px bar with a 16 px hit area, showing elapsed time and "−remaining" below.
  - On hover it grows to 6 px and shows a 12 px knob.
  - Dragging scrubs and seeks on release.
  - Hidden when the app does not report a timeline.
- **Controls:** shuffle, previous, play/pause (44 round gading on black), next, repeat. **Shuffle and repeat are hidden when the app does not support them** (YouTube Music via Chrome, screen 32).
  - Active shuffle or repeat shows in `--accent`. Repeat cycles off → all → one (`repeat-one`).
- **Volume** is a button on the right: click mutes, the wheel changes volume by 5%, and hover shows the level as a badge ("62%").
- **Keyboard:** Space plays or pauses, ←/→ seek 5 s, Shift+←/→ change track, M mutes.
- **Lyrics (opt-in):** "Turn on lyrics" in the lyrics area or in Settings. The area adds 208 px (520 × 420).
  - **Timed:** 6 lines, 24 px rows. The current line is 15/600 gading, the next ones 13 `--muted`, the sung ones `--faint`. On a line change the list slides up by one row (220 ms).
  - **Nudge:** − / + in 0.5 s steps, remembered per song.
  - **Plain:** scrollable text, nothing highlighted.
  - **Loading:** small head melek with "Looking for lyrics…" and three skeleton bars. The bars are static, not shimmering.
  - **Not found:** kawung, "No lyrics found for this song", and "Search again".
  - **Off:** the privacy note and "Turn on lyrics".
- **Lyric focus:** a switch in the lyrics bar and in Settings. While on, and a timed line exists, the idle pill shows nyanyi eyes and the line.
  - Max width 220 in the corner and 200 in the notch, with an ellipsis. It never scrolls.
  - A new line slides up 6 px and fades in over 220 ms.
  - Alerts, reminders due within 15 min and a running focus still take the pill first.
- **Fun:** nyanyi for 1.6 s when a track starts. The equalizer (3 bars) is optional and off by default. It uses transform: scaleY, restarts only on play, and Settings explains its cost.

### 4.10 Settings (screen 50)

- Every row has a title and a one-line helper in `--faint` 11/15 that says what happens.
  - Toggles are Switches (36 × 20).
  - Multi-choice settings use Segmented (Position, default focus length).
- New rows under **Now playing:**
  - Show media in the pill
  - Equalizer in the pill
  - Lyrics, with the privacy helper
  - Lyric focus, disabled until Lyrics is on
- Changes apply at once, with no Save button.

### 4.11 Alerts (v1 20–24, unchanged layout)

- **Arrival:** if the pill shows eyes, they go kaget for 240 ms, then the morph starts. The kawung gong (v1) follows.
- Alerts never take keyboard focus (WS_EX_NOACTIVATE). After 8 s they wait in the pill. **Esc** while hovering dismisses, and **S** snoozes.
- During lyric focus or media, the alert still takes the pill first.

## 5. Motion additions

| Moment | Trigger | Spec | Reduced motion |
|---|---|---|---|
| Shape morph | tab or view change | section 3.1 | instant |
| Expression change | event | crossfade 160 ms; holds per 2.2 | instant swap |
| Kedhep | hover enter | scaleY 1→0.18→1, 140 ms | none |
| Kaget before alert | alert arrives | kaget eyes 240 ms, then the morph | swap, no delay |
| Lyric line change (pill/notch) | LRC timestamp | old line translateY 0→−6, opacity 1→0, 120 ms; new line 6→0, 0→1, 220 ms `--ease-out` | swap |
| Lyric line change (Now playing) | LRC timestamp | the list translates up one row (24 px), 220 ms; the current line's weight and colour swap at the end | swap |
| Track change | new media properties | art and title crossfade 240 ms; the progress line resets without animating | swap |
| Month change | ‹ › PgUp PgDn | the grid slides 12 px in the direction of travel and fades, 200 ms; the agenda crossfades | swap |
| Day select | click / arrows | `--selected-bg` fades in over 120 ms; the agenda crossfades 160 ms | swap |
| Check-off | checkbox | fill 180 ms; seneng 1.2 s; the row moves to Completed after 1.5 s (opacity 1→0 over 160 ms, then the list closes up) | no fade |
| Kain weave | Rampung or a session ends | the cell's opacity 0→1 and scale 0.6→1, 400 ms | instant |
| Gumun | streak grows | expression 1.6 s | swap |
| Equalizer (opt-in) | playing | 3 bars, scaleY between 0.3 and 1 at 3 fixed phases, 900 ms per cycle, **runs only while playing and visible**; paused stops it | off |
| Quick capture open | hotkey | pill → 360 × 140 morph; content on beats | instant |

Nothing animates while idle. During playback the only continuous work is the seek bar or pill progress redrawing once per second while visible, plus the optional equalizer.

## 6. Components (new or changed)

### 6.1 Selector system (screen 51)

| Component | Size | Default | Hover | Selected / on | Focus-visible | Disabled |
|---|---|---|---|---|---|---|
| **Segmented** (view switch) | 28 segments in a 2 px inset track, radius 10 / 8 (small: 24) | `--muted` on `--surface` track | segment `--surface` | `--selected-bg` + `--fg` | 2 px `--focus-ring`, offset −2 | 0.32 |
| **Chip** (filter) | 28, radius 14, padding 0 12 | 1 px `--hairline`, `--muted` | `--hairline-strong`, `--fg` | `--surface-hover` + `--fg`, no border | ring offset 2 | 0.32 |
| **Select** (choose from list) | 28, radius 14, 1 px `--hairline`, chevron 12 | `--fg` | `--hairline-strong` | open: `--surface`, chevron-up, menu (v1 11) | ring | 0.32 |
| **Switch** | 36 × 20, knob 14 | track `--surface-hover` + 1 px `--hairline-strong`, knob `--muted` | inner border `--fg` | `--accent` track, black knob right | ring | 0.32 |

Use **segmented** for Task/Note, Clipboard/Files, Position and the focus length. Use **chips** for Notes filters and "Remind me". Use **select** for the action-item list, Now playing sessions and Repeat.

### 6.2 Now playing parts

| Part | Spec |
|---|---|
| Container | 520 × 212, padding 16, `--radius-panel` |
| Source row | 24 high. App icon 16 (radius 4, from Windows). Name 13/600 `--muted`, gap 6. Count badge 18, radius 9, `--surface-hover`. Icon buttons 28 on the right. |
| Art | 56 × 56, `--radius-art` 8, gap 14 to the text. Placeholder when missing: `#3A2A1E` with the `music` icon 24 in `#C0855299`. |
| Seek | 4 high (hover 6), `--track` with `--gading` fill, knob 12 on hover or drag, hit area 16, margin-top 14. Times 11/14 `--muted` tabular, margin-top 6. |
| Controls | Row gap 20, margin-top 8. Icon buttons 32 in `--fg`, active `--accent`. Play 44 round, `--gading` with black icon 18 (hover `#F7F0E1`, pressed `#DCD2BC` at scale 0.95). Volume pinned right. |
| Lyrics area | Margin-top 14, 1 px `--hairline` top border, padding-top 10. Bar 24: caption "Lyrics · lrclib.net", nudge buttons 24 round `--surface`, offset 11 faint, Lyric focus switch. Lines 24 high. |

### 6.3 Calendar parts

| Part | Spec |
|---|---|
| Nav | 32 high: "October 2026" title 15/600, "Today" button 28, ‹ › icon buttons 28 |
| Weekday header | M T W T F S S, 18 high, caption style, `--faint` |
| Day cell | 52 × 44, radius 10. Date 13/600 at top 6. Weekend date `--muted`. Other month 400 `--disabled`. |
| Type dots | 4 px, gap 3, centred, bottom 9. Max 3, in the order deadline `--deadline`, meeting `--meeting`, reminder `--general`. |
| Focus bar | 2 px `--focus`, bottom 4, centred. Width = minutes ÷ 120 × 32, clamped to 6–32 px. |
| Today | 1.5 px inset ring `--accent` |
| Selected | `--selected-bg` |
| Weton day | kawung 7 px in `--accent`, top 4, right 5. No pasaran or wuku anywhere in the calendar (user choice); the header keeps today's pasaran. |
| Agenda | 1 px `--hairline` top border, padding-top 10. Header 28: "{Ddd D Mmm}" (+ " · today"), then text buttons "+ Task" and "+ Reminder". Rows padding 6 4, time column 40. |
| Agenda order | Late (today only, label `--deadline`, "Move to…"), then timed items by time (meetings with badge and note link, deadlines with checkbox), then untimed deadlines, then every-N reminders as one line, then focus minutes + "N done". |

### 6.4 Other new parts

| Part | Spec |
|---|---|
| Pill progress line | 2 px `--track`, fill `--gading`, inset 14 left and right, bottom 3 (notch: bottom 2) |
| Equalizer | 3 bars 2 × (4–10) px, gap 2, `--gading`, after the title |
| Parse highlight | `--accent` on `--focus-tint`, radius 4, padding 0 2 |
| Hint chip | 22 high, radius 11, `--focus-tint` / `--accent`, 11/600, calendar icon 12 |
| Link chip | 24 high, radius 12, `--surface`, 12 px text, link icon 12 |
| "Last time" card | `--surface`, radius 12, padding 8 10, caption header with chevron-up, 14 px checkboxes |
| To Todo / In Todo | 20 high, radius 10, 11/600; `--accent` on `--focus-tint` / `--done` with check-circle |
| Search hit | `<mark>` `--search-hit` background, `--fg`, radius 3 |
| Star | 16, `--faint`; hover `--fg`; on `star-filled` `--priority` |
| Drag handle | 16, `--faint`, shown on row hover, cursor grab |
| Tab bar | 4 text tabs + 3 icon tabs (calendar-month, shelf, settings), 28 × 32 each, gap 4; active icon tab `--fg` with a 2 px underline inset 4 |

## 7. Screens

| # | Screen | Tokens | Assets |
|---|---|---|---|
| 27 | Character sheet | eye-*, blangkon*, brow, mascot-accent | mascot/* |
| 28 | Corner pill: media (eq on/off), lyric focus, overdue glance, kaget | pill-*, pill-progress-h, pill-art, lyric-max-w | eyes-nyanyi, -mrengut, -kaget, icons/music |
| 29 | Notch: idle head, lyric focus, media, focus | notch-*, notch-art, lyric-max-w-notch, edge-notch | blangkon-plain, eyes-melek, -nyanyi, -fokus |
| 30, 31 | Peek with Playing row and progress (corner, notch) | peek-media-h, pill-progress-h | blangkon-plain, eyes-melek, previous, pause, next, kawung-mark |
| 32 | Now playing (YouTube Music: no shuffle/repeat) | nowplaying-*, radius-art, seek-*, round-button-lg | lyrics, close, previous, pause, next, volume, music |
| 33, 34 | Now playing with timed lyrics (corner, notch) | nowplaying-lyrics-h, fs-lyric* | + shuffle, repeat |
| 35 | Lyrics states: plain, loading, not found, off | fs-lyric-other, surface | blangkon-plain, eyes-melek, kawung-mark, lyrics |
| 36 | Session switcher open | popover-bg, radius-menu | chevron-up, check |
| 37, 38 | Calendar month + agenda (corner, notch) | calendar-*, day-cell-*, dot, focus-bar-h, selected-bg | calendar-month, chevron-left/right, add, repeat, notes, check, kawung-mark |
| 39 | Empty month | | blangkon-kawung, brows-soft, eyes-merem |
| 40 | Quick add on a day | parse, hint chip | calendar |
| 41 | Meeting note, tall editor | editor-* | meeting, open-full, more, chevron-up, add-to-todo, check-circle |
| 42 | Note editor, live Markdown, linked task | fs-h1-note, fs-h2-note | link, add, add-to-todo |
| 43 | Notes search highlight | search-hit | search, meeting |
| 44 | Todo: natural dates, stars, drag, undo | priority | star, star-filled, drag-handle, notes, subtask, calendar |
| 45 | Focus: quick lengths, even kain | focus-*, kain-* | blangkon-kawung, eyes-melek, kawung-mark, play |
| 46, 47 | Quick capture card (corner, notch) | capture-* | todo, notes, calendar |
| 48 | Shelf: type to search, pinned | search-hit | pin, pin-filled, copy, file, pause, more |
| 49 | Rampung closing moment | rampung-*, kain-*, dur-weave | eyes-bungah, blangkon-kawung, brows-raised |
| 50 | Settings, Now playing rows | | |
| 51 | Selector system and new control states | segment-h, chip-h, focus-ring | chevron-*, volume, volume-mute, star |
| 52 | Shape language | all shape tokens | |
| 53 | Brand: app icon v2, tray | | icon-app.svg, icon-tray-* |

## 8. Copy deck additions

### English strings

| Area | Strings |
|---|---|
| Pill / notch | aria: `Karsa, playing {title}`, `Karsa, lyric: {line}`, `Karsa, {n} tasks overdue` |
| Peek | `Playing` (key), tooltips `Previous` `Pause` `Play` `Next` |
| Now playing | `{App}` · `{Site} · {Browser}`, `{n}` (sessions), `Playing · {title}`, `Paused · {title}`, `{artist} · {album}`, `{m:ss}`, `−{m:ss}`, tooltips `Shuffle` `Repeat` `Repeat one` `Repeat off` `Volume` `Mute` `Unmute` `Lyrics` `Close`, `{n}%` |
| Lyrics | `Lyrics · lrclib.net`, `Lyrics · not timed`, `Looking for lyrics…`, `No lyrics found for this song`, `Karsa asked once and will not ask again for this track.`, `Search again`, `Lyrics are off`, `Turning them on sends the title, artist, album and length of what is playing to lrclib.net. Nothing else leaves your PC.`, `Turn on lyrics`, `Lyric focus`, `Timing {+0.5 s}` |
| Calendar | `Calendar` (tab tooltip), `{Month YYYY}`, `Today`, `M T W T F S S`, `{Ddd D Mmm} · today`, `Task`, `Reminder`, `Late`, `Move to…`, `Note · {n} open`, `every {n} min, {HH:MM}–{HH:MM}`, `Focus · {n} sessions`, `{n} done`, `Nothing on {Ddd D Mmm}`, `Nothing planned this month.`, `Enter to add · Esc to cancel`, aria `{D Mmmm}, {n} items, weton` |
| Notes | `Link a task`, `Saved · title from the first line`, `Ctrl+N new note`, `Saving…`, `No notes yet`, `Ctrl+N starts one`, `No notes match "{q}"`, `{n} notes`, `To Todo`, `In Todo`, `Note deleted`, `Undo` |
| Meeting | `Meeting · weekly`, `Last time · {Ddd D Mmm}`, `carried`, `Action items · {n}`, `Send {n} to Todo` |
| Todo | `Enter to add · Esc keeps it as text`, `Yesterday`, `{n} note(s)`, `Undo`, tooltips `Star`, `Unstar`, `Drag to reorder` |
| Focus | `15` `25` `50`, `{n} days in a row`, `Today {n} min` |
| Capture | `Enter saves to Inbox · Tab: note · Esc`, `Draft kept for 10 minutes` |
| Shelf | `Pinned`, `Pinned · kept until you unpin it`, `Last 24 hours`, `Enter copies the top match and closes · Esc clears`, `Paused`, `Clear 24 hours of copied text? Pinned items stay.`, `Copied text from the last 24 hours shows here.` |
| Rampung | `Rampung. Matur nuwun.`, `{n} tasks done · {m} min focus`, `This week`, `Close` |
| Settings | `Now playing`, `Show media in the pill`, `Equalizer in the pill`, `Three small bars that move with playback. Costs a little CPU while music plays.`, `Lyrics`, `Lyric focus`, `Shows the current lyric line in the pill instead of the title. Needs Lyrics.`, `Default length`, `15 min` `25 min` `50 min` |

### Javanese phrases (v2 additions)

| Phrase | Meaning | Where |
|---|---|---|
| Rampung. Matur nuwun. | Done. Thank you. | Rampung closing moment |
| Ana dina ana upa | Each day brings its own bread (each day provides) | Calendar empty month (or the paribasan of the day) |
| Sapa temen bakal tinemu | Whoever is earnest will find their way | Sample paribasan of the day in Rampung (from the M6 rotation) |
| Blangkon | Javanese headcloth; Yogyakarta style with the *mondolan* knot | Character (not shown as text) |
| Nyanyi, kaget, seneng, gumun | singing, surprised, pleased, amazed | Internal expression names, never shown |

The v1 and M6 phrases are unchanged.

## 9. Assets and budget

| Group | Files | Clean size |
|---|---|---|
| `mascot/` | 12 eyes, 2 blangkon, 5 brows | 9.5 KB (largest 2.1 KB) |
| `icons/` new | shuffle, repeat-one, volume, volume-low, volume-mute, lyrics, lyric-focus, calendar-month, chevron-up, previous, next, source-app, star, star-filled, pin, pin-filled, drag-handle | 5.4 KB |
| `icons/` from the app | shelf, settings, copy, music, minus, file, folder (drawn by Claude Code in v1 style, now in `design/`) | as built |
| `icon-app.svg` v2 | | 3.6 KB |
| **New SVG total** | | **18.5 KB** (budget 100 KB) |
| `icon-app-1024.png` | Tauri icon source, not loaded by the UI | 89 KB |

`next.svg` is the same glyph as v1 `skip.svg`, under the media name. `previous.svg` replaces the app-made one with the same geometry style. `source-app.svg` is the fallback when Windows gives no app icon.

The design tool adds a `<metadata>` block to every saved SVG. `scripts/import-design-assets.mjs` (or `design/tools/strip-svg-metadata.mjs`) removes it.

## 10. Implementation checklist

1. Replace `:root` with `tokens.css` v2. Set the window to `--window-w/h` (560 × 640) in `island.rs` / `tauri.conf.json`. In the corner the anchor stays bottom-right; in the notch the window is centred, so the 80 px notch sits at x 240 of 560.
2. `config.ts`: add a `SHAPE` map keyed by view (section 3), and morph to `SHAPE[view]` on every tab or view change, following 3.1.
3. `Eyes.svelte`: load `mascot/eyes-*`, add the `head` prop (`eyes | small | large`), and compose the layers per 2.1. Add the four expressions to `mood.svelte.ts`, using the priority in 2.2.
4. Pill: add the lyric focus variant, the progress line and the optional equalizer. In the notch, use the small head.
5. Build the Now playing view and the lyrics area (M8 steps 2–3). Add the Calendar tab and view (step 4).
6. Replace selectors per 6.1 and set every text button to 28.
7. Build the Todo parser, star and drag; the Notes live Markdown, links and search highlight; the meeting "Last time" card and inline action items; quick capture as a card; the Rampung closing; the Shelf pin and type-to-search.
8. Generate the Tauri icons from `assets/icon-app-1024.png` (`npx tauri icon`).
9. Check that idle CPU and memory are unchanged, and measure the equalizer; drop it if it costs more than 0.3% of one core.
