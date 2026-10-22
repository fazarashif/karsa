# Karsa — Requirements (v1)

Status: v1 built (v0.1.0) · Last updated: 2026-10-19

Karsa is a lightweight Windows desktop companion that lives in a small floating "island" pill, inspired by the iPhone Dynamic Island. Version 1 focuses on the island itself plus a handful of core productivity features: todos, a Pomodoro timer, notes, meeting notes and manual reminders.

## 1. Goals

- **Lightweight.** The app should feel instant and never be noticeable in Task Manager.
- **Unobtrusive.** The island never steals focus, never blocks important UI, and stays quiet unless something needs attention.
- **Organized.** Tasks, reminders and notes are in one place and reachable in one hover or click.
- **Local-first.** All data stays on the machine. v1 makes no network requests.

## 2. Performance budget

| Metric | Target | M0 measured | v0.1.0 measured (M5) |
|---|---|---|---|
| CPU while idle | ~0% (no polling loops, no idle animations) | 0.05% of one core, once WebView2 settles (~2 min after start) | 0.08% of one core by cycle count, once settled |
| CPU with a focus timer running | (only the visible countdown) | — | 0.82% of one core |
| RAM while idle | ≤ 100 MB total, including WebView2 processes | ~73 MB private working set, 6 processes | ~74 MB, 6 processes |
| Expand / collapse animation | 60 fps, ~250 ms | 60 fps, no frame over 25 ms; most motion within ~300 ms of a 700 ms spring | unchanged |
| Cold start to visible pill | < 1.5 s | 1.0–1.3 s | 1.6–2.4 s with the CPU held at 1.38 GHz (see below) |
| Installer size | < 15 MB | not built yet; `karsa.exe` is 3.1 MB | 1.79 MB NSIS installer; `karsa.exe` is 4.5 MB, UI bundle 193 KB |

Measured on an i5-1135G7 laptop with 16 GB RAM running Windows 11 25H2. During the M5 measurements the CPU was held at 1.38 GHz (`MaxClockSpeed` and a calibrated cycle counter both reported it), so start-up there is slower than in M0, when the CPU could boost. Almost all of the start-up time is WebView2 creating its browser and renderer processes; Karsa shows the window on the first painted frame, without waiting for data.

After M1 (design system v1 + Todo on SQLite), the release build measured the same: 0.10% of one core and ~73 MB private working set while idle. `karsa.exe` grew to 4.3 MB (bundled SQLite and PNG decoding for the tray icon).

After M2 (Focus), idle is unchanged (0.05% of one core, ~73 MB). While a focus timer runs, the once-per-second countdown cost 4.7–5.9% of one core by process CPU time, almost all of it in WebView2 rendering. The M5 performance pass found why (see below).

After M3 (Reminders), idle is still 0.00–0.05% of one core and ~76 MB; with six reminders scheduled the scheduler thread just sleeps (0.31% of one core measured shortly after start).

After M4 (Notes), idle is 0.05% of one core and ~75 MB; nothing watches the notes folder, so notes cost nothing until the Notes tab is opened. `karsa.exe` is 4.5 MB. A note opened with "Open full" adds one WebView2 renderer while the window is open; it is destroyed on close.

**M5 performance pass (running focus timer).** Measured with CPU cycle counters (`QueryProcessCycleTime`), which unlike process times are not rounded to 15.6 ms ticks. This laptop ran at 1.38 GHz.

| | Before | After |
|---|---|---|
| WebView2 renderer | 2.85% of a core | 0.40% |
| WebView2 GPU process (software compositor) | 1.32% | 0.27% |
| Karsa core | 0.12% | 0.00% |
| **Total** | **4.39% of one core** | **0.82% of one core** |

A Chromium trace showed most seconds cost 3 frames, but some digits ("20:37") are 0.24 px narrower than others. The pill's rounded width then flipped between 96 and 97 px, which restarted the island's 700 ms width spring at 60 fps and resent the window region. The pill now grows at once but ignores 1 px shrinks (`setPillWidth` in `App.svelte`), so a countdown costs one small repaint per second. The pill's accessible label also counts whole minutes, so the accessibility tree changes once a minute instead of every second.

After M6 (v0.2.0: clipboard listener, media thread, hotkey, 15-minute housekeeping tick), idle measured 0.04% of one core by cycle count and ~68 MB across the six processes, so the new listeners cost nothing measurable while nothing happens. The installer is 1.83 MB.

All background work is event-driven. Timers are scheduled for the next due moment instead of ticking every second; the only per-second work is updating a visible countdown.

## 3. Platform and stack

- **OS:** Windows 11. Windows 10 reached end of support in October 2025 and is not targeted.
- **Shell:** Tauri 2 (Rust core + WebView2, which ships with Windows 11).
- **UI:** Svelte 5 + TypeScript, plain CSS for animations.
- **Core:** Rust, using the `windows` crate for Win32 APIs.
- **Storage:** SQLite via `rusqlite` for structured data; Markdown files for notes.
- **Tauri plugins:** `autostart`, `single-instance`.

The Rust core is the single source of truth for timers, scheduling and system state. The UI only renders state and sends user actions.

## 4. The island

### 4.1 Placement

- Bottom-right corner of the primary monitor, just above the taskbar, inside the monitor's work area with a 12 px margin.
- Anchored at its bottom-right corner, so it grows up and to the left when it expands.
- Since M7, Settings can move it to the top centre of the screen as a notch, which grows downwards (see 6c).
- Repositions automatically when the resolution, DPI scale, taskbar size or work area changes.
- Per-monitor DPI aware.
- Multi-monitor selection and drag-to-reposition are out of scope for v1.

### 4.2 States

| State | Trigger | Content | Approx. size (logical px) |
|---|---|---|---|
| **Idle** | Nothing active | Solid black pill with the next reminder or todo deadline due today (e.g. `Weekly sync · 14:00`, or `in 4m` within 15 minutes), or the *mripat* eyes when nothing else is due today (see `design/DESIGN.md` 4.1–4.2) | 72–220 × 32 |
| **Live activity** | Running timer or reminder due soon | One activity, e.g. `◔ Focus 18:42` or `Weekly sync · in 4m` | up to 220 × 32 |
| **Peek** | Cursor hovers on the pill for ~300 ms | Summary: date, active timer with pause/stop, next reminder today, top 3 open tasks by due date | ~320 × 170 |
| **Expanded** | Click on the pill or on the peek card | Full panel with tabs: **Todo · Focus · Notes · Reminders** | ~380 × 520 |
| **Alert** | Reminder fires or a Pomodoro phase ends | Expands silently with the title and actions (e.g. Snooze 5m / Dismiss) | ~320 × 120 |
| **Hidden** | A fullscreen app or presentation is in the foreground | Not shown; all alerts, including Meeting reminders, are held and shown once fullscreen ends | — |

When several things compete for the collapsed pill, only the one with the highest priority is shown:

1. Unacknowledged alert
2. Reminder due within 15 minutes
3. Running or paused Pomodoro
4. Idle content

### 4.3 Interaction

- **Hover:** shows Peek after a 300 ms delay (450 ms on the notch) and closes it 400 ms after the cursor leaves. The delay prevents accidental opens when the cursor passes by.
- **Click:** opens Expanded. It stays open until the user clicks outside, presses `Esc` or clicks the pill again.
- **Alerts:** silent, with no sound. The alert stays expanded for 8 seconds, then collapses into the pill with an accent color until the user acknowledges it.
- **Focus:** Idle, Live activity, Peek and Alert never take keyboard focus away from the current app. Only Expanded activates the window, because it needs keyboard input.
- **Tray icon:** a right-click menu with Open, Settings, Start with Windows and Quit (development builds also have "Show test alert"). Left-click opens the panel. The app has no taskbar button.
- **Start with Windows:** on by default and can be turned off in Settings.
- **Single instance:** launching the app again focuses the existing instance.

### 4.4 Visual style

The visual design is specified in [`design/DESIGN.md`](../design/DESIGN.md) (design system v1) with tokens in `design/tokens.css`. In short:

- Solid black (`#000`) island with Javanese accents: kuningan, sogan, bata, gading and daun colors, the kawung mark, an aksara ꦏꦂꦱ watermark in the panel, the *mripat* eyes, and Javanese greetings. No blur or transparency effects.
- System font only: Segoe UI Variable.
- The box morphs on a spring; content enters on staggered "gamelan" beats and only animates `transform` and `opacity`.
- When Windows "Animation effects" is off (`prefers-reduced-motion`), transitions are instant.
- No animation runs while the island is idle.
- SVG assets are copied from `design/assets` into `src/assets` by `scripts/import-design-assets.mjs`, which also strips the design tool's metadata (350 KB → 16 KB).

### 4.5 Window implementation notes

- Tauri window settings: transparent, undecorated, always on top, skip taskbar, not resizable, no native shadow, created hidden.
- Win32 extended styles: `WS_EX_TOOLWINDOW` and `WS_EX_NOACTIVATE`. Remove `WS_EX_NOACTIVATE` only in the Expanded state.
- **Morph strategy:** the window never resizes. It is a fixed window covering the largest shape (400 × 540 until M8, 560 × 640 since, for the per-function shapes of design v2); only the web content animates. A window region (`SetWindowRgn`) clips the window to the visible box, so the window neither draws nor takes clicks anywhere else. The region grows immediately when a state gets bigger and shrinks ~450 ms after it gets smaller, once the spring has mostly settled.
- **Hover:** detected by the web content itself (`mouseenter`/`mouseleave`), because the region limits the window to the box. No global mouse hook.
- **Closing the panel on click-away:** decided from foreground changes (another process's window became foreground), not from Tauri's `Focused(false)` event. WebView2 moves keyboard focus into its child window, which fires `Focused(false)` while the island is still active and would close the panel immediately.
- **Keeping focus outside the panel:** clicks in non-panel states call `preventDefault()` on `mousedown` so the page never takes focus; as a safety net, if the window is activated anyway while it should not be, it hands activation straight back to the previous app.
- **Do not use Tauri's window setters** for visibility, focusability or click-through (`show`, `hide`, `set_focusable`, `set_ignore_cursor_events`, …). They re-apply every window style and call `ShowWindow`, which hides the island (when created hidden) or activates it, stealing focus. Use Win32 calls in `island.rs` instead.
- **Fullscreen detection:** listen for foreground-window changes (`SetWinEventHook` with `EVENT_SYSTEM_FOREGROUND` and `EVENT_SYSTEM_MOVESIZEEND`), register a hidden app bar to receive `ABN_FULLSCREENAPP`, then compare the foreground window with the island's monitor and check `SHQueryUserNotificationState`. Do not poll.
- **WebView2 rendering:** software compositing (`--disable-gpu --disable-gpu-compositing`, network service in-process). For this small surface it keeps 60 fps, uses less CPU during morphs than GPU compositing (14.8% vs 19.7% of a core over four hover cycles) and ~70 MB less memory.
- **Sleep/resume:** listen for `WM_POWERBROADCAST` and recompute all timers on resume (M3).
- **No title bar flash:** the undecorated window still has `WS_CAPTION` (tao keeps it and hides the frame in `WM_NCCALCSIZE`), so the default `WM_NCACTIVATE` handling painted a classic "Karsa ✕" title bar over the top of the panel whenever its activation changed, visible for ~0.5 s after closing the panel. The island's subclass passes `WM_NCACTIVATE` on with `lParam = -1`, which tells Windows not to repaint the non-client area.

## 5. Features

### 5.1 Todo

- **Lists:** multiple lists (e.g. Work, Personal) that can be created, renamed, reordered and deleted. A default `Inbox` list always exists.
- **Task fields:** title, optional due date/time, completed flag, list, sort order.
- **Subtasks:** one level deep. The parent task shows progress (e.g. `2/5`).
- **Deadlines:** a task with a due date automatically gets a linked reminder (default: at the due time; lead time is configurable). Completing the task cancels the reminder. *(M3. Since M1, the next task due later today already shows in the pill and in Peek's "Next up".)*
- **Quick add:** an input field at the top of each list. Pressing `Enter` adds the task.
- **Completed tasks:** collapsed under a "Completed" section in each list.
- **Task detail:** clicking a task opens its detail view: title, list, due date (Today / Tomorrow / Next week / None, or a date and optional time), subtasks (add, tick, delete), and delete with a confirmation step. Changes save immediately; Esc goes back to the list.
- **Managing lists:** the list switcher creates lists and switches between them; "Manage lists…" renames, reorders and deletes them (with a confirmation that names how many tasks go with the list).
- **Peek:** shows the top 3 open tasks across all lists, sorted by due date (tasks without a due date come last), excluding the one already shown as "Next up".

### 5.2 Focus (Pomodoro)

- **Defaults:** 25 min focus, 5 min short break, 15 min long break after every 4 focus sessions. All values are configurable.
- **Controls:** start, pause, resume, skip phase, stop.
- **Collapsed pill:** shows the phase and remaining time, updated once per second.
- **Phase end:** silent alert with "Start break" / "Skip" (or "Start focus" / "Skip"). Auto-starting the next phase is off by default and can be turned on in Settings.
- **Stats:** the Focus tab shows how many focus sessions were completed today.
- **States:** Ready → Running ⇄ Paused → (phase ends) → Waiting for the next phase, or Running again when auto-start is on. Stop returns to Ready and resets the cycle to session 1. Skip leaves the current or waiting phase without recording it and starts the following one (so "Skip" on a focus-end alert skips the break).
- **Alerts:** focus end → "Focus session done" with Skip / Start break; break end → "Break's over" with Stop / Start focus. If the waiting phase is started from the Focus tab instead, the alert clears itself.
- **Surfaces:** the pill shows the *fokus* eyes with the countdown in kuningan (break: *leren* eyes, "Break" and the countdown); Peek's Focus row shows the countdown, a progress bar, and pause/stop buttons that work without opening the panel; clicking a timer pill opens the panel on the Focus tab.
- **Settings:** stored in the `settings` table (`focus.focus_min`, `focus.short_break_min`, `focus.long_break_min`, `focus.sessions_per_cycle`, `focus.auto_start`), defaults 25 / 5 / 15 / 4 / off, edited in Settings (`focus_update_settings`).
- **Engine:** `src-tauri/src/focus.rs`. One thread sleeps until the running phase's wall-clock end, waking at least once a minute and immediately on resume from sleep (`WM_POWERBROADCAST`). Finished focus sessions go to the `focus_sessions` table.
- **Timing:** the Rust core stores absolute end timestamps rather than counting down. If the PC sleeps through a phase end, the phase is marked as ended when the PC resumes.

### 5.3 Notes

- Plain Markdown notes. The title is taken from the first line.
- **Expanded panel:** note list with search (title and content), a filter (All / Notes / Meetings) and a lightweight editor (plain textarea, no live preview in v1).
- **Open full:** opens the note in a larger separate window. The window is created on demand and fully destroyed on close, so its memory is released.
- **Autosave:** debounced at about 500 ms.
- **External edits:** the file is re-read when a note is opened. If the file changed on disk since it was loaded, the user is asked before Karsa overwrites it.

### 5.4 Meeting notes

A meeting note is a regular note with a structured header, so every meeting is easy to find and review. It records what the meeting was, what it was about, when it happened, what was discussed and what the action items are. The user writes the content manually: there is no timer, no automatic timestamps and no microphone use.

- **Fields:** title, date & time, topic/agenda, attendees (optional), notes, action items.
- **Create:** "New meeting note" in the Notes tab, with the date and time prefilled with the current time. It can also be created from a Meeting reminder's alert, which prefills the title and date/time from the reminder.
- **Action items:** a Markdown checklist that can be ticked off inside the note.
- **Add to todo:** each open action item has an "Add to todo" button, and there is also an "Add all" button. Items are copied into a chosen list (default `Inbox`) with a link back to the meeting note. Items that were already added are marked so they are not added twice.
- **List view:** meeting notes are sorted by date and show the title, date and number of open action items.

File format (Obsidian-compatible):

```markdown
---
type: meeting
title: Weekly sync
date: 2026-10-19T14:00
topic: Q3 roadmap review
attendees: [Alex, Sam]
---

## Notes

- Budget for Q3 approved
- Launch moved to November

## Action items

- [ ] Send updated deck to the team
- [x] Book room for next review
```

As built in M4 (`src-tauri/src/notes.rs`, `src/lib/notes/`, `src/lib/meeting.ts`):

- **Folder:** the `notes.folder` setting, else `Documents\Karsa\Notes` (created on first save). The Rust core only reads and writes `.md` files inside it; paths from the UI are relative and may not leave the folder.
- **File names follow the title:** `Title.md` for notes, `Meetings\YYYY-MM-DD Title.md` for meeting notes. Renaming the title renames the file on the next save (" 2", " 3", … when the name is taken), and linked todo tasks follow the new path. Characters Windows does not allow and reserved names (`CON`, `NUL`, …) are replaced.
- **Saving:** 500 ms after the last keystroke, when leaving the note, and when its window closes. Writes go to a temporary file that is then renamed, so a crash never leaves half a note. Each save sends the file's modification time from when it was read; if the file changed since, the save is refused and a banner offers **Reload** or **Keep mine**.
- **List and search:** files up to three folders deep (hidden folders skipped, files over 2 MB ignored), sorted by date: a meeting's date, otherwise the file's modification time. Search matches the title and the whole text, case-insensitively, while typing (debounced). Meeting rows show the topic and "N open" action items.
- **Meeting editor:** a form over the same Markdown file: title, When, Topic, attendee chips, a growing Notes area, and the action-item checklist. Unknown sections and front-matter keys are kept as they are, so notes edited in Obsidian or Notepad survive a round trip.
- **Add to todo:** creates the task in the chosen list with `tasks.note_path` set and appends `<!-- karsa-task:ID -->` to the checklist line, which is how the item shows as "Added". The task's detail view shows "From a meeting note · Open", which opens the note in the panel.
- **From a Meeting alert:** "Meeting note" creates and opens `Meetings\<date> <reminder title>.md` with the reminder's time.
- **Open full:** a 720 × 640 window (minimum 420 × 360) with the kawung header, the same editor, and the window title set to the note title. The panel collapses when it opens. Several notes can be open at once; opening the same note again focuses its window.
- **Delete:** "Delete note" (with a confirm step) moves the file to the Recycle Bin. "Show in folder" opens Explorer with the file selected.

### 5.5 Reminders (manual)

- **Fields:** title, date & time, type (General / Meeting / Deadline), repeat (none, daily, weekdays, weekly, or every few minutes), lead time (at time, 5, 10, 15 or 30 min before).
- **Every few minutes** (added after M6, e.g. "drink water"): every 15, 20, 30, 45, 60, 90 or 120 minutes, from a start time until an end time, on weekdays or every day. These nudges have no date or lead time. They never take the pill or Peek's "Next up" (they would always be next), and a slot missed while the PC slept is skipped quietly instead of counted as missed. Stored in `every_min`, `window_from`, `window_until` (minutes after local midnight) and `window_days` (schema v6, which rebuilds the `reminders` table to allow the new repeat).
- **Alert:** silent expand with Snooze (5 min) and Dismiss. Meeting reminders also offer "New meeting note".
- **Collapsed pill:** shows a countdown when a reminder is due within 15 minutes.
- **Reminders tab:** upcoming reminders grouped by day, with deadlines from todos shown alongside (linked to the task).
- **Missed reminders:** when the PC was asleep or off, a single alert lists what was missed (e.g. "2 missed reminders").
- **Scheduler:** the Rust core keeps one timer for the next due item and reschedules whenever data changes or the PC resumes from sleep.

As built in M3 (`src-tauri/src/reminders.rs`):

- **Storage:** `reminders` table. `at` is the event time (a meeting's start, a deadline); the alert fires `lead_min` earlier, or at `snoozed_until` when snoozed. One-off reminders are marked fired; repeating ones move to their next occurrence, computed in local wall-clock time with Windows' time zone rules (`src-tauri/src/localtime.rs`).
- **Scheduler:** one thread sleeps until the next alert. It wakes on any change, on resume from sleep or a system clock change (`WM_POWERBROADCAST`, `WM_TIMECHANGE`), and at least every 15 minutes. It starts only after the UI has loaded, so alerts due while Karsa was closed are not lost.
- **Missed:** an alert firing more than a minute late counts as missed. All reminders due in one pass become a single alert ("2 missed reminders" with titles and times, or "2 reminders"), with Review (opens the Reminders tab) and Dismiss.
- **Alert text:** "Reminder · now" / "Reminder · in 10 min", "Meeting · starts in 5 min" / "Meeting · now", "Deadline · due 15:00". Alerts that arrive while one is showing wait in a queue; several held during fullscreen are merged into one summary.
- **Todo deadlines:** a task with a due *time* has a linked reminder (type Deadline, "From Todo"), created, updated or deleted in the same transaction as the task. Completing or deleting the task, or removing its time, cancels it. Only its lead time is edited in the reminder form; the title and time follow the task ("Open task" jumps there). A new link starts with the default lead time from Settings. A due time already in the past does not alert; one still ahead whose lead time has already begun alerts right away. Date-only deadlines never alert but are listed as "All day".
- **Form:** title, date, time, type, repeat, "Remind me" lead time; Save, Cancel, Delete (with a confirm step). A one-off in the past is refused ("Pick a time in the future"); a repeating one starting in the past moves to its next occurrence. If the lead time puts the alert in the past while the event is still ahead, it alerts right away.
- **Pill and Peek:** the next reminder still ahead today drives the pill (`Title · in 4m` within 15 minutes, in its type color; otherwise `Title · 14:00` below the focus timer) and Peek's "Next up".

## 6. Settings

- Start with Windows (default: on)
- Notes folder location (default: `Documents\Karsa\Notes`)
- Pomodoro durations, long-break interval, auto-start next phase
- Default reminder lead time
- Hide during fullscreen (default: on)

As built in M5 (`src-tauri/src/settings.rs`, `src/lib/settings/`):

- **Where:** a Settings tab, shown as an icon at the end of the panel's tab row, and the tray's "Settings" item, which opens the panel on it. Changes apply at once; there is no Save button.
- **General:** Position (Bottom right or Top, since M7), Start with Windows (the Windows Run key, kept in step with the tray's check mark either way) and Hide during fullscreen. With hiding off, the island stays on top of fullscreen apps and alerts show there too.
- **Focus:** steppers for Focus (5–120 min, steps of 5), Short break (1–30 min), Long break (5–60 min, steps of 5) and Long break after (2–8 sessions), plus "Start the next phase by itself". A running phase keeps its length; a ready or waiting one changes right away.
- **Reminders:** "Remind me" (At time, 5, 10, 15 or 30 min before): the starting lead time of new reminders and of new todo deadline reminders. Existing reminders keep their own.
- **Notes:** the folder path, **Change…** (the Windows folder picker), **Open** (Explorer) and **Use default**. Karsa then reads and saves notes in the new folder; notes already saved are not moved.
- **Storage:** keys `island.hide_in_fullscreen`, `island.position` (`corner` or `top`, M7), `reminders.default_lead_min`, `notes.folder` and the `focus.*` keys in the `settings` table.

## 6b. M6 — Personal companion and tools

Everything in M6 stays local and event-driven: no polling, no network, nothing that runs while the island is idle except what Windows tells Karsa (a hotkey, a clipboard change, a media change).

### 6b.1 Personal

- **Name:** optional, in Settings. The panel greeting becomes "Sugeng enjing, Faza".
- **Weton:** today's pasaran (Legi, Pahing, Pon, Wage, Kliwon) is shown next to the date, e.g. "Tue 20 Oct · Kliwon". It is computed from the date alone (anchor: 17 August 1945 was Jumat Legi). With an optional birthday in Settings, the greeting becomes "Sugeng ambal warsa" on the birthday and "Sugeng wetonan" on the user's weton day (every 35 days).
- **Paribasan:** a Javanese proverb of the day, with its meaning in English, in the panel's empty states and at the end of the Rampung ritual. The same proverb all day.

### 6b.2 Mripat

- Small mripat eyes sit in Peek and in the panel header. While the cursor moves over the island, the pupils glance toward it. Nothing moves when the cursor is elsewhere.
- Two new expressions: **bungah** (proud: closed happy eyes and a kuningan sparkle) on the birthday, the weton day and after a finished Rampung; **mrengut** (sulking) for two minutes after a focus session is stopped early.

### 6b.3 Kain (focus record)

- The Focus tab shows the last four weeks as a small woven cloth: one cell per day, a kawung mark whose strength grows with the focus minutes that day, and the current streak of days with at least one finished session.

### 6b.4 Rampung (end of the day)

- At 17:00 on weekdays (Settings: on/off and the time) Karsa offers to wrap up the day, as a silent alert with **Start** and **Later** (30 min). It waits while a focus phase runs and is offered at most once a day. The tray's "Wrap up the day" starts it at any time.
- The ritual is a panel view: today's finished tasks and focus minutes; open tasks due today or overdue, each with "Tomorrow" (or "Move all to tomorrow"); tomorrow's reminders and deadlines; then **Rampung**, which closes the panel with the proverb of the day and makes the mripat bungah until midnight.

### 6b.5 Quick capture

- A global hotkey, **Win+Shift+K** by default (changeable in Settings, or off), opens the panel straight into a capture box from any app. Enter saves a task to Inbox; Tab switches to Note, which saves a new note. Esc closes and returns focus to the app the user was in.

### 6b.6 Shelf (clipboard and files)

A "Shelf" icon tab next to Settings, with two lists:

- **Clipboard:** text copied in the last 24 hours, newest first, up to 200 items, with search. Click copies an item back. Items are stored in the database encrypted with the user's Windows account (DPAPI) and deleted after 24 hours. Copies marked as private by the copying app (password managers set `ExcludeClipboardContentFromMonitorProcessing` or `CanIncludeInClipboardHistory = 0`) are never recorded. **Pause** stops recording until resumed; **Clear** deletes everything. Settings can turn the feature off.
- **Files:** drop files on the island, paste files copied in Explorer (Ctrl+V or **Paste** in Shelf › Files), or pick them with **Add files…**. The shelf keeps links to the originals (no copies). Each item shows the icon Explorer shows for it (PowerPoint for a `.pptx`, the folder icon for a folder), or its extension on a small page for a type Windows does not know; hovering it shows the type ("Microsoft PowerPoint Presentation") and the path. Items can be dragged back out to any app, copied (then pasted as files), opened, shown in Explorer or removed. A missing original is shown as missing.

### 6b.7 Media

- When something plays through Windows' media controls (Spotify, a browser tab, Media Player…), Peek shows a "Playing" row with the title, artist and previous / play-pause / next buttons, and grows by that one row.
- The pill shows the track (album art and title) only while it plays and nothing else needs the pill: no alert, no reminder or deadline today, no focus timer. A track left paused does not take the pill. Settings can turn this off.

### 6b.8 As built (M6)

- **Personal** (`src/lib/jawa.ts`): pasaran from the day number since the anchor, checked against published calendars (Selasa Kliwon on 20 October 2026, Sabtu Legi on 1 January 2000). 28 proverbs rotate by day.
- **Mripat** (`Eyes.svelte`, `mood.svelte.ts`, `pointer.svelte.ts`): the pupils and highlights shift up to ~1.3 px toward the cursor, updated at most once per frame and only on `mousemove` over the island. Mood order: mrengut (2 min after an early stop) › merem (22:00–05:00) › bungah (birthday, weton day, Rampung done today) › mesem (today's tasks all done) › melek. The two new expressions are app-made SVGs in `src/assets/eyes`, drawn in the design system's style.
- **Kain** (`focus/Kain.svelte`, `focus_history`): 28 cells, Monday first; levels at 1–24, 25–74 and 75+ minutes, coloured sogan-deep, sogan and kuningan.
- **Rampung** (`rampung/Rampung.svelte`): the offer is checked on the minute clock (no timer of its own); "Tomorrow" keeps a timed deadline's time of day. The tray has "Wrap up the day".
- **Quick capture** (`capture/Capture.svelte`): the hotkey is registered with `RegisterHotKey` on the hook thread's hidden window, so Windows posts `WM_HOTKEY` only when it is pressed. Settings offers a few shortcuts (Win+Shift+K, Ctrl+Alt+K, Alt+Shift+K, Ctrl+Shift+Space, Ctrl+Alt+Space, Off) and says when another app already holds one. It waits while a fullscreen app hides the island.
- **Clipboard** (`clipboard.rs`, `hooks.rs`): `AddClipboardFormatListener` on the same window; only `CF_UNICODETEXT` up to 20,000 characters; `ExcludeClipboardContentFromMonitorProcessing`, `Clipboard Viewer Ignore` and `CanIncludeInClipboardHistory = 0` are skipped. Text is stored with `CryptProtectData` (current user, plus a fixed entropy). Copying the newest item again moves it instead of adding it; copying back from the history is not recorded again. A 15-minute tick on the hook thread deletes items older than 24 hours; the database runs with `secure_delete`, and Clear also checkpoints the WAL.
- **Files** (`shelf.rs`, `App.svelte`): WebView2 itself takes the drop, the way Edge does (the island window has `dragDropEnabled: false`, so external drops stay allowed). The page accepts file drags (`dragover` with `Files`), grows the island into the drop target, and on `drop` posts the dropped `File`s with `chrome.webview.postMessageWithAdditionalObjects`; the core's `WebMessageReceived` handler (`accept_drops`) reads each `ICoreWebView2File` path. Links and text dropped outside a text field are refused, so the webview never navigates away. A drag counts as gone on a `dragleave` without `relatedTarget` (moving between elements, as the morph does, has one), or after 10 s without drag events. Two earlier designs failed with a real drag from Explorer (red "no" cursor): Tauri's drop handling and Karsa's own OLE drop target both sit on windows in Karsa's process, but the window under the cursor belongs to the WebView2 process and Windows did not reach them. Dropped, pasted (`CF_HDROP` from the clipboard) and picked files all land in the same table. Icons come from `SHGetFileInfoW` (the large icon and the type name) and go to the UI as PNG data URLs, 1–3 KB each, looked up once per extension (per path for programs, shortcuts and folders, which carry their own) on a background thread with COM; lookups take turns, since the shell's first ones failed when two threads raced. An icon identical to the one for an unknown extension becomes the extension badge. Over the pill the island grows into a 320 × 112 drop target; over the open panel it switches to Shelf › Files. Dragging out uses `SHDoDragDrop` with the shell's own data object (from `SHCreateShellItemArrayFromIDLists`), and Copy puts the same object on the clipboard (`OleSetClipboard`), so Explorer, Outlook and Teams accept the files.
- **Media** (`media.rs`): `GlobalSystemMediaTransportControlsSessionManager` on its own thread, which waits for the current-session, media-properties and playback-info events. Album art is read once per track and sent as a data URL; app ids become friendly names ("Chrome", "Spotify").

## 6c. M7 — The notch

Karsa can sit at the top of the screen instead of the bottom-right corner, like the Dynamic Island: a small black notch hanging from the top edge that grows down into Peek, alerts and the panel. Settings › General › Position switches between **Bottom right** (the default) and **Top**, at once and without a restart.

- **Proportions:** the notch is kept small because it sits over title bars, tabs and the search boxes some apps centre there. Idle it is 80 × 26 (the corner pill is 72 × 32), with 24 px eyes, square top corners flush with the top edge and 12 px bottom corners. Text in it (an alert, a reminder due within 15 minutes, the focus countdown, what is playing) is capped at 200 px wide instead of 220.
- **Idle shows the eyes:** a later reminder or deadline today stays in Peek instead of widening the notch all day, so the idle notch is the mripat unless something needs attention now.
- **Bigger states:** Peek, alerts, the drop target and the panel keep their corner sizes, centred under the notch, with square top corners so they read as the notch growing.
- **Hover:** Peek opens after 450 ms instead of 300 ms, because the pointer often crosses the top centre on its way to tabs and title bars.

### 6c.1 As built (M7)

- **Placement** (`island.rs`): `set_position` stores the choice and calls `place()` again. On top, the 400 × 540 window is centred on the primary monitor horizontally and sits at the top of the work area (below anything docked there), with no margin. The window region is anchored to the middle of the window's top edge, with 18 px of padding below and on both sides for the spring overshoot and the shadow (10 px before the shadow was strengthened). The window still never resizes; a display, DPI or work-area change re-places it in whichever position is chosen.
- **Layout** (`App.svelte`, `Pill.svelte`): the island and its content layer are anchored at `left: 50%` with `translate: -50% 0` and grow from the top centre. The notch's width is rounded up to an even number so its edges stay on whole pixels; the pill-width hysteresis allows 2 px for that. Its edge (`--edge-notch`) leaves out the top side, which meets the screen edge, and since M8 it floats over what is under it (`--shadow-notch`): a 1 px hairline just outside it at 12% white, which is what shows over dark windows (where no shadow can), a contact shadow (0 1 3 at 60% black) and a soft one (0 6 14 −3 at 70%) for light windows. It reaches 17 px, so the notch's region padding grew from 10 to 18 px (`NOTCH_PADDING`); that ring is still part of the window and takes clicks. The corner keeps the edge only.
- **Tokens:** `--notch-h`, `--notch-min-w`, `--notch-max-w`, `--radius-notch` and `--edge-notch` in `design/tokens.css` (and `tokens.json`); `NOTCH` and `TIMING.hoverOpenNotch` in `config.ts`.

## 6d. M8 — Now playing and calendar (planned)

Planned on 2026-10-21 from an exploration of the code and the Windows APIs; nothing is built yet. Decisions still open are listed in 6d.4.

### 6d.1 Now playing

**What Windows offers.** Every app that plays media through Windows' media controls (the overlay behind the volume keys) has a session in `GlobalSystemMediaTransportControlsSessionManager`: Spotify (desktop and Store), Chrome, Edge and Firefox tabs (through the web Media Session API), Media Player, Apple Music, TIDAL, Deezer, Amazon Music and most others. M6 reads only the *current* session's title, artist, art and play state. A session can also report:

- the album, and a timeline (position, length, last update), with seeking (`TryChangePlaybackPositionAsync`);
- shuffle and repeat state, and setting them;
- which of these it supports (`IsPlaybackPositionEnabled`, `IsShuffleEnabled`, `IsRepeatEnabled`). Support differs per app: browsers pass on what the site sets (YouTube and YouTube Music set the position; the web API has no shuffle or repeat), Spotify reports more. This is checked at runtime, so a control shows only when the app supports it. The first M8 step is a probe with Spotify and YouTube Music playing on this PC.

**Where it plays.** The session names the app (`SourceAppUserModelId`: "Spotify.exe", "Chrome", a Store app id), never the web page. Karsa shows:

- the app's real name and icon, looked up through the shell (`shell:AppsFolder\<app id>`), the same way the shelf gets file icons;
- for a browser, the site when it can tell, without any network: the browser window whose title holds the track's title ("… - YouTube Music", "… - YouTube", "… | SoundCloud"); failing that, a hint from the metadata (YouTube Music sends an album, YouTube does not). Shown as "YouTube Music · Chrome", or just "Chrome" when unsure. A background tab cannot be identified, and Chrome exposes one session per browser.
- every session at once: with Spotify paused and a YouTube tab playing, both are listed and either can be controlled.

**Controls.** Play/pause, previous, next, a seek bar with elapsed and remaining time, shuffle and repeat (when supported), and the app's volume and mute through Windows' per-app audio sessions (`IAudioSessionManager2`, matched by process). "Like" and playlists are app features that Windows does not expose.

**Where it shows.**

- Pill and notch (M6): album art and title while playing. New: a thin progress line under it, and optionally the current lyric line instead of the title.
- Peek: the Playing row (M6) gains the progress line.
- **Now playing** (new): a panel view, like Rampung and Capture, opened by clicking the media pill or Peek's Playing row. Large art, title, artist, album, source, seek bar, controls, volume, a switcher when several apps have media, and lyrics below.

**Cost.** Event-driven as in M6: the media thread waits for Windows' change events, adding the timeline event. Between updates the UI works out the position from the last one and the playback rate; it redraws the seek bar about once a second, and only while it is visible. No audio is analysed, so there is no real visualiser; an optional animated equaliser in the pill must stay within the CPU budget (measured, otherwise dropped).

### 6d.2 Lyrics

**Source.** Windows does not provide lyrics, and Spotify's (from Musixmatch) are not available to other apps. The practical source is **LRCLIB** (lrclib.net): free and open, no API key, community lyrics with timestamps (LRC) for many songs, looked up by title, artist, album and length (`GET /api/get`, with `/api/search` as a fallback). Genius and Musixmatch need keys, limit what may be shown, or forbid scraping.

**Privacy.** This is Karsa's first network request. Lyrics are therefore **off by default**: Settings › Lyrics explains that turning them on sends the title, artist, album and length of what is playing to lrclib.net, and nothing else. The request goes through Windows' own HTTP stack (WinRT `HttpClient`), so a company proxy works, and it adds no new library. Each answer, including "not found", is cached in the database (schema v7), so a song is asked for once.

**Showing them.** In Now playing: the timed lines scroll, with the current one in gading and the rest muted. Untimed lyrics are shown as plain text. A ±0.5 s offset adjusts the timing of each song, and is remembered. Optionally the current line also shows in the pill. The UI schedules a timer for each line change rather than polling.

### 6d.3 Calendar

A calendar in Karsa's own style, built from what Karsa already knows, with no outside calendar:

- **Month view** (design v2, 6.3). Weeks start on Monday, as in the kain. Each day cell shows its date. Up to three dots in the type colours mark deadlines (bata), meetings (sogan) and other reminders (gading). A 2 px kuningan bar shows the day's focus minutes. Today has a kuningan ring, and your weton day (every 35 days) a small kawung. By the user's choice there is no pasaran or wuku in the calendar; the panel header keeps today's pasaran.
- **Day agenda.** Selecting a day lists overdue tasks (today only), then that day's task deadlines (tick to finish, "Move to…" to reschedule), reminders and meetings in time order (with the meeting note, if any), focus minutes, and tasks finished that day. "+ Task" and "+ Reminder" add one on that day.
- **Repeats.** Daily, weekday and weekly reminders appear on every matching day. "Every N minutes" reminders show once per day as a single line ("Drink water · every 30 min, 07:00–17:00"), not as dozens of dots.
- **Data.** Tasks and reminders are already in the UI. One new command, `calendar_month`, returns focus minutes per day and the dated meeting notes. Nothing runs in the background.

### 6d.4 Decisions (settled with design v2)

1. **Lyrics:** opt-in, off by default, with the privacy note (design v2 4.9).
2. **Calendar:** a new **icon tab** (calendar-month) beside Shelf and Settings; Reminders stays a tab. The tab gap shrinks to 16 so four text tabs and three icon tabs fit.
3. **Javanese layers in the calendar:** none per day (no pasaran, no wuku), only the weton-day kawung. The Javanese month and year and Indonesian public holidays stay possible later.
4. **Outlook meetings:** not in M8; the classic Outlook could be read offline through COM, but the new Outlook has no such interface.

Design v2 itself (`design/DESIGN.md`, with v1 kept in `design/DESIGN-v1.md`) is the visual and interaction spec for M8: the 560 × 640 envelope and shape table (3), the character sheet with the Yogyakarta blangkon and four new expressions (2), Now playing and lyrics (4.9, 6.2), and the calendar (4.5, 6.3).

### 6d.5 Shapes, lyric focus and the design refresh

Added on 2026-10-21, after the plan above:

- **A shape for each function.** The open island takes the shape its content needs, instead of one panel size for everything: Now playing is wide and short, the calendar and the note editor are tall, alerts stay compact. Switching tabs morphs between shapes with the same spring as today. The window still never resizes at runtime: its fixed size grows to cover the largest shape (the design proposes the envelope, at most about 560 × 640), and the window region keeps everything outside the shape click-through. In the corner the shapes grow up and to the left; as a notch they grow down and to both sides.
- **Lyric focus.** A switch, in Settings and in Now playing, that makes the idle pill or notch show the current lyric line while a song with timed lyrics plays. It shows one line only, so it covers little; a long line is cut off, never scrolled. Lines change with a short cross-fade when the song reaches them, and nothing moves in between. Alerts, reminders due soon and a running focus timer still take the pill first.
- **Design refresh first.** Before M8 is built, Claude Design updates the design system to v2 (brief: [docs/claude-design-brief-v2.md](claude-design-brief-v2.md)): an audit of v1 against the app as built, Karsa's character (the mripat with a blangkon, and more expressions), the shape language above, and the M8 screens. M8 then follows design v2.

### 6d.6 Order of work

0. **Design v2** in Claude Design (6d.5), unpacked into `design/`.
1. **Probe** (with the user playing Spotify, then YouTube Music): what each app reports. This decides which controls appear for which app.
2. **Now playing:** all sessions, timeline and seek, shuffle/repeat, volume, the source and its icon, the Now playing view, the progress line in Peek and the pill.
3. **Lyrics:** setting and privacy note, LRCLIB lookup and cache, the timed view, the lyric line in the pill.
4. **Calendar:** the tab and its views, month grid, pasaran and wuku, day agenda, quick add and reschedule.
4b. **Shapes and character:** the larger window envelope, a shape per function, lyric focus, and design v2's character and expressions.
5. **Checks:** idle CPU and memory unchanged, the cost of a playing track with lyrics, docs, the installer.

### 6d.7 As built so far (2026-10-21)

- **Design v2 imported:** `design/` holds v2 (`DESIGN.md`, tokens, assets, screens 27–53, prototype); v1's spec stays as `design/DESIGN-v1.md`. `scripts/import-design-assets.mjs` also brings `mascot/` into `src/assets` (19 files, metadata stripped).
- **Envelope and shapes:** the window is 560 × 640 (`WINDOW_SIZE`, `tauri.conf.json`, `WINDOW`). `SHAPE` in `config.ts` gives each view its size and `panel.view` says which view is showing (the open note editor counts as its own view), so the island box and the panel both take that shape: Todo, Notes list and Reminders 380 × 520, Focus 380 × 500, the note and meeting-note editor 420 × 620, Calendar 400 × 600, Shelf 400 × 520, Settings 400 × 560, Rampung 380 × 480. Quick capture keeps the panel until it becomes a card in M10 (the UX playbook).
- **Character:** `Eyes.svelte` composes the v2 mascot layers with a `head` prop: eyes only (corner pill, panel header), the small head with the plain blangkon (notch 24 px, Peek 22 px), and the large head with the kawung blangkon and brows (Focus 56 px, Rampung, the empty month). `reactions.svelte.ts` holds the timed expressions by rank: kaget (600 ms) when an alert arrives, which also waits 240 ms in the pill before the alert opens; seneng (1.2 s) when a task is ticked; gumun (1.6 s) when a finished session grows the focus streak (the streak moved into the focus store); mrengut's 1.6 s glance on hover while tasks are overdue, at most every 10 minutes; nyanyi (1.6 s) when a new track starts. The eyes blink once when a reaction ends. Each head has `data-expression` for tests.
- **Calendar:** an icon tab (calendar-month) beside Shelf and Settings, tab gap 16. `calendar.ts` works out each day's dots (deadlines from tasks and deadline reminders, meetings from meeting reminders and meeting notes, other reminders), the focus bar from the focus history, and the agenda in v2 order; repeats show on every matching day of the month, and "every N minutes" reminders show as one line, without dots. Quick add: "+ Task" adds an all-day task on the selected day, "+ Reminder" a one-off reminder at a chosen time; late tasks have "Move to…" (the native date picker). Keyboard: PgUp/PgDn, T, the arrows and Enter. The weton day carries the kawung; the empty month shows the large merem head and the paribasan.
- **Now playing** (`media.rs`, `nowplaying/NowPlaying.svelte`): the core lists every Windows media session and shows Windows' current one unless the user picked another in the switcher. Per session it reads the album, the timeline (position, length, the time of the last report, rate), shuffle and repeat, and which controls the app supports; a timeline event updates only the position. The UI moves the position on from the last report while playing, redrawing once a second only while a progress line is visible. Actions: play/pause, previous, next, seek, shuffle, repeat (off → all → one), select, volume and mute; volume goes through the app's Windows audio sessions (`IAudioSessionManager2`, matched by process name; a browser's several sessions move together). The source shows the app's icon from the Start menu (`shell:AppsFolder`), or its exe's, and for a browser the site from the window title whose page holds the track title (YouTube Music, YouTube, Spotify's "Song • Artist", SoundCloud and a few more). The view is 520 × 212 (520 × 420 with lyrics): source row with the session switcher, art, title, "Artist · Album", seek bar (drag to scrub), controls without the ones the app lacks, volume (click mutes, wheel ±5%). Keyboard: Space, ←/→ 5 s, Shift+←/→ track, M. Opened from the media pill or Peek's Playing row; Peek with media is 320 × 210 with the app icon and a progress line, and the pill has a 2 px progress line.
- **Probe** (2026-10-21, YouTube in Chrome): title, artist and art, no album; play/pause, previous, next and seeking; no shuffle or repeat; the position is reported only on changes (one report was 223 s old and the interpolated position was right).
- **Lyrics** (`lyrics.rs`, `lyrics.svelte.ts`): off by default (`lyrics.enabled`, `lyrics.focus`). `lyrics_get` cleans YouTube-style titles ("Artist - Song (Official Video)", "ArtistVEVO", "- Topic"), asks LRCLIB's `/api/get` and then `/api/search` (closest length within 4 s, timed first) through WinRT `HttpClient` (the system proxy), parses LRC, and caches every answer in the `lyrics` table (schema v7), "not found" and instrumental included, with the per-song timing nudge (`lyrics_offset`). Tested from the company network: 0.97 s for a first lookup, 3 ms from the cache. The lyrics area shows timed lines (the current one in gading, the list sliding one row on each line), plain lyrics, loading, not found with "Search again", or the privacy note with "Turn on lyrics". The current line moves on a timer set for the next timestamp.
- **Lyrics fix (after first use):** every lookup came back "not found". YouTube reports the video's title ("Guy Sebastian - Battle Scars (Official Video) ft. Lupe Fiasco") and the channel as the artist ("guysebastianVEVO"), the first cleanup only split "Artist - Song" when the channel matched the left part letter for letter, the length check allowed 4 s while a video often runs longer than the song, and LRCLIB answers bursts with 503. Now names compare without spaces, punctuation or case; a video's "A - B" title is split (the channel decides the order when it matches either side, else "Artist - Song", and the other order is tried too); bracketed extras, "| …" tails and "ft./feat." are dropped; results must match title and first artist, within 30 s of a video's length or 6 s of a music app's; requests are spaced 250 ms apart and a 503 or 429 is retried twice after a pause; a cached "not found" is asked again after three days, and migration v8 forgot the misses cached by the first version. Seven test titles (English and Indonesian, both orders, "| Official Video", YouTube Music) all found timed lyrics.
- **Lyric focus:** with Lyrics and Lyric focus on, the idle pill shows nyanyi eyes and the line being sung at a fixed size, each new line rising 6 px into place. On the user's feedback it is larger than design v2's one line of 220 (corner) and 200 (notch): the corner pill is 320 × 52 with 20 px corners, and the notch is 440 wide with one row.
  - **Two lines in the corner (2026-10-22):** the line being sung in kuningan (`--accent`, also in the notch), and the next line under it, 12 px and muted. A line that does not fit steps down from 13 to 12 and then 11 px; one still too long takes both rows (the next line makes way), and only then ends in an ellipsis. The notch has the current line alone.
  - **Between lines:** while the music plays before the first line, between lines or after the last, the pill keeps lyric focus with a kuningan music note (and, in the corner, the next line under it) instead of falling back to the song's title.
  - **Priority:** while lyric focus sings, only a waiting alert and a running focus timer go before it. Reminders and meetings, even within 15 minutes, wait (their alerts still come at their time). Paused, or with no timed lyrics, the pill is as before.
- **Peek while music plays (2026-10-22):** hovering the island while something plays shows Now playing (520 × 212, or 520 × 420 with lyrics) instead of the Peek summary, with a "Panel" button (Karsa's small head) at the start of its source row that opens the panel. The choice is made when Peek opens, so pausing from it keeps Now playing until the pointer leaves. Paused or nothing playing: the usual Peek. The current line in Now playing's lyrics is kuningan too.
- **Now playing from the panel:** while something has media, the panel header shows a chip (the app icon and title) between the date and the time; it opens Now playing with a back button, and Esc returns to the tabs. From the pill or Peek, Now playing closes the island as before.
- **Cost:** 0.58% of one core while a song plays and the media pill shows its progress line (cycle count, 30 s); nothing more while idle.
- **Not built:** the optional equalizer in the pill (design v2 marks it optional, off by default).

## 6e. M9 — Outlook meetings

The user's work calendar is in Microsoft 365, used through classic Outlook on this PC. The user wants those meetings in Karsa's calendar, an optional meeting note per meeting that starts filled in, and a reminder a chosen time before each meeting. Read only: Karsa never changes Outlook.

### 6e.1 How Karsa reads Outlook

- **The choice (explored 2026-10-22):** Karsa reads classic Outlook through its COM object model, on this PC, while Outlook runs. It needs no network, no sign-in, no app registration and no administrator, and nothing leaves the PC. The alternatives: Microsoft Graph works with any Outlook (new Outlook and the web included) but needs an Entra app registration, likely IT's consent, a browser sign-in and network calls; it is the path if the company moves to the new Outlook, which has no COM. A published ICS link is often disabled and makes the calendar readable to anyone with the link; EWS is being switched off for Exchange Online from October 2026; the Windows calendar API needs a packaged app.
- **The probe:** reading appointments, attendees and bodies raised no Outlook security prompt on this PC (Outlook asks only when its programmatic-access setting or the antivirus status calls for it). Two meetings this week were read in 0.37 s from PowerShell.
- **The reader** (`outlook.rs`): a thread in the multithreaded COM apartment attaches to a running Outlook with `GetActiveObject` (it never starts one), then by late binding (`IDispatch`, DISPIDs cached per name): `GetNamespace("MAPI")` → `GetDefaultFolder(olFolderCalendar)` → `Items`, sorted by `[Start]` with `IncludeRecurrences`, then `Restrict` to the 1st of last month up to the end of next month. Filter dates are written in the user's short date and time format (`GetDateFormatEx`/`GetTimeFormatEx`), as Outlook expects. Per item: subject, start, end, all day, location, organizer, required and optional attendees (without the user and the organizer, at most 30 kept), canceled (`MeetingStatus` 5 or 7) and declined (`ResponseStatus` 4). Bodies are not read.
- **Keys:** a meeting's key is a 64-bit FNV-1a hash of its `GlobalAppointmentID` (or `EntryID`), plus the local date for an occurrence of a recurring meeting.
- **When:** at once when turned on, every 5 minutes after that, when the Calendar tab opens, and a minute after a failed read. Outlook closed: the last read stays (status "closed") and its alerts still fire. Nothing is written to Karsa's database: the meetings live in memory, and turning the feature off forgets them.

### 6e.2 In Karsa

- **Calendar:** meeting dots on their days (not for canceled or declined ones) and agenda rows at their start time (all-day ones with the untimed items). The tooltip gives the time range, place and organizer. Canceled meetings are struck through, declined ones faint. A meeting with a note shows "Note · N open"; one without shows "+ Note". If Outlook cannot be read, a line under the agenda says why.
- **Meeting notes:** "+ Note" (or "+ Meeting note" on the alert) creates the note with the title, the date and time, and the organizer plus up to 11 attendees (a town hall's fifty names would bury the note); the header keeps `outlook: <key>`. Notes are optional: a meeting without one simply has none. Asking again opens the same note.
- **Reminders:** Settings → Outlook → "Remind me before a meeting": Off, 5 min, 10 min (the default), 15 min, 30 min, 1 h or 2 h. The alert is the meeting alert ("Meeting · starts in 10 min", + Meeting note, Snooze 5m, Dismiss). It fires once per meeting and start time, only before the meeting starts, never for all-day, canceled or declined ones, and waits for the UI like other alerts. Changing the lead time reschedules at once. A snooze lasts until Karsa restarts.
- **Pill and Peek:** the next meeting today counts as "next up", beside reminders.
- **Settings:** "Outlook meetings" is off until turned on, with a status line: connected (with the count and when it was read), Outlook closed, or the error.

### 6e.3 Tanggal merah

The calendar shows Indonesia's red dates, as a printed Indonesian calendar does.

- **Look:** Sundays and public holidays (libur nasional) have their date in abang (`--holiday`, the palette's red; "tanggal abang" in Javanese), and so does the Sunday letter above the grid. Cuti bersama is a softer abang (`--holiday-leave`, 65%). Saturdays stay `--muted`, and days of other months stay `--disabled`. Hovering a day gives the holiday's name.
- **Agenda:** a holiday's day starts with a row in abang: a kawung mark, the name (in Indonesian, as the decree has it, e.g. "Idulfitri 1447 H") and "Public holiday" or "Cuti bersama".
- **Data** (`src/lib/holidays.ts`, nothing fetched): the fixed holidays (1 Jan, 1 May, 1 Jun, 17 Aug, 25 Dec) and the Easter ones (Wafat Yesus Kristus, Paskah, Kenaikan Yesus Kristus, from Easter by Meeus's algorithm) for any year; the holidays the government sets each year (Isra Mikraj, Imlek, Nyepi, Idulfitri, Iduladha, Waisak, Tahun Baru Islam, Maulid) and all cuti bersama only for the years listed, from the SKB 3 Menteri: 2025 (17 holidays, 10 cuti bersama and the 18 August 2025 one added that August), 2026 (17 and 8) and 2027 (18 and 8; signed 15 September 2026, with Isra Mikraj twice). A new year is added by hand when its decree comes out, usually in September; until then that year shows only the fixed and Easter holidays.

### 6e.4 Note projects

Notes can be sorted into projects, as tasks are into lists, and each project is a folder in the notes folder, so File Explorer shows the same grouping.

- **On disk** (`notes.rs`): a project is a folder directly inside the notes folder, laid out like the notes folder itself: `Project/Title.md` and `Project/Meetings/YYYY-MM-DD Title.md`. Notes at the top (and in the top `Meetings`) have no project. Every folder there except `Meetings` and hidden ones is a project, so one made in File Explorer shows up too. Names follow the file name rules (no `\ / : * ? " < > |`, not `Meetings`), and two projects cannot differ only in case.
- **The Notes list:** a switcher above the search, as Todo's list switcher: All notes, No project, then each project with its count; New project; Manage projects. With All notes, each note shows its project as a small tag. New note and New meeting note go into the project shown (none from All notes). Meeting notes started from Outlook or a reminder go to no project.
- **A note's project:** the editor's bar shows it (a folder mark and the name), and More → Move to project moves the file (saving what was typed first). A title change renames the file inside its project.
- **Manage projects:** rename (the folder is renamed), delete (only an empty project; its folder goes to the Recycle Bin) and add. Tasks added from a meeting note keep their link through a move or a rename.

## 7. Data and storage

| Data | Location |
|---|---|
| Lists, tasks, subtasks, reminders, Pomodoro sessions, settings | `%APPDATA%\app.karsa.desktop\karsa.db` (SQLite, WAL mode, schema version in `PRAGMA user_version`) |
| Clipboard history (24 h) and the file shelf (links) | Same database, tables `clipboard` (text encrypted with DPAPI) and `shelf` (schema v5) |
| Notes | `<notes folder>\*.md` |
| Meeting notes | `<notes folder>\Meetings\YYYY-MM-DD Title.md` |

The notes folder can be pointed at OneDrive or Google Drive for backup and sync. There is no built-in cloud sync.

## 8. Language and locale

- UI language: English.
- Dates and times follow the Windows regional settings.

## 9. Out of scope for v1

- Dedicated water and stand-up trackers (an "every few minutes" reminder covers the nudge)
- Calendar sync (ICS, Google Calendar, Outlook)
- Linking Pomodoro sessions to tasks
- Meeting recording, transcription and AI summaries
- Drag-to-reposition, choosing a monitor

(Clipboard history, quick capture, media controls and the file shelf moved into M6.)

## 10. Milestones

| # | Milestone | Done when | Status |
|---|---|---|---|
| M0 | Island shell prototype | Pill sits bottom-right; Idle / Peek / Expanded states with smooth morph; hover and click behave as specified; no focus stealing; hides during fullscreen; tray icon; autostart; idle CPU ~0% and RAM within budget | Done 2026-10-19 |
| M1 | Todo | Lists, tasks, subtasks, due dates; Peek shows top tasks | Done 2026-10-19, with design system v1 |
| M2 | Focus | Pomodoro with live activity in the pill and silent phase-end alerts | Done 2026-10-19 |
| M3 | Reminders | Manual reminders with repeat, lead time, snooze, missed-reminder handling; todo deadlines linked | Done 2026-10-19 |
| M4 | Notes & meeting notes | Markdown notes with search, the meeting template, and the "Open full" window | Done 2026-10-19 |
| M5 | Polish | Settings screen, installer, performance pass against the budget | Done 2026-10-19 (cold start above target while the CPU is throttled; see section 2) |
| M6 | Personal companion and tools | Name, weton and paribasan; expressive mripat; focus kain and streak; Rampung ritual; quick capture hotkey; 24 h clipboard history; file shelf; media controls; idle cost unchanged | Done 2026-10-20 (v0.2.0) |
| M7 | The notch | Settings choose Bottom right or Top; on top, a slim notch with the eyes when idle that grows downwards into every state; switching needs no restart | Done 2026-10-21, open to revision after use |
| M8 | Now playing and calendar | All media sessions with seek, shuffle/repeat and volume where the app supports them; the source app (and site, when it can be told); opt-in timed lyrics from LRCLIB; a Karsa calendar with deadlines, reminders, meetings and focus per day; design v2's shapes and character (6d) | Done 2026-10-21 except the optional equalizer (6d.7) |
| M9 | Outlook meetings, tanggal merah and note projects | Meetings from classic Outlook on this PC (read only, COM) in the calendar, Peek and the pill; a meeting note per meeting that starts filled in; a meeting alert a chosen time before; Sundays, public holidays and cuti bersama marked in the calendar; notes sorted into projects, each a folder (6e) | Built 2026-10-22 |
| M10 | UX playbook | Design v2's section 4 for the features that already exist: live Markdown, autosave and task links in notes; "Last time" and inline action items in meeting notes; natural dates, stars, drag and undo in Todo; quick focus lengths and the even kain; type-to-search and pins in the shelf; quick capture as a card; the Rampung closing moment; the v2 selector system and 28 px buttons | Planned (was M9) |

## 11. Decision log

| Date | Question | Decision |
|---|---|---|
| 2026-10-19 | Idle pill content, given the taskbar already shows the clock | Show the next reminder or deadline due today; fall back to the time |
| 2026-10-19 | "Add to todo" for meeting action items | Yes |
| 2026-10-19 | "New meeting note" from a Meeting reminder alert | Yes |
| 2026-10-19 | Should Meeting reminders break through fullscreen? | No; all alerts are held until fullscreen ends |
| 2026-10-19 | Click-through: mouse hook + `set_ignore_cursor_events`, or a window region? | Window region; the setter approach would hide the island and steal focus, and the region needs no global hook |
| 2026-10-19 | WebView2 GPU or software compositing? | Software; measured equal smoothness, lower CPU and ~70 MB less memory |
| 2026-10-19 | Idle pill when nothing is due | The *mripat* eyes instead of the time (design system v1); the taskbar already shows the clock |
| 2026-10-19 | Second action on the break-end alert | "Stop" (end the Pomodoro run); "Skip" there would skip the focus session, which nobody wants from an alert |
| 2026-10-19 | Do date-only task deadlines alert? | No; only deadlines with a time get a reminder. Date-only ones are listed as "All day" in Reminders |
| 2026-10-19 | Several reminders due at once | One alert listing them (with Review), instead of alerts replacing each other |
| 2026-10-19 | Where the timer runs | In the Rust core on absolute wall-clock end times, so it survives the panel closing, the webview throttling timers, and sleep |
| 2026-10-19 | SQLite: bundled or Windows' built-in `winsqlite3`? | Bundled; current `rusqlite` no longer links `winsqlite3`, and bundling costs ~1 MB of binary with no measurable memory cost |
| 2026-10-19 | Note file names | Follow the title and rename with it, so the folder stays readable in Explorer and Obsidian |
| 2026-10-19 | Watch the notes folder for changes? | No; the list refreshes when shown, and a changed file is caught at save time by its modification time. A watcher would cost a thread and wake-ups for no visible gain |
| 2026-10-19 | Deleting a note | To the Recycle Bin, never a permanent delete |
| 2026-10-19 | Link from an action item to its task | An HTML comment on the checklist line (`<!-- karsa-task:ID -->`); invisible in Markdown viewers and survives edits elsewhere |
| 2026-10-19 | Where Settings lives | A fifth panel tab shown as an icon, plus the tray item; the panel's size fits every setting without a separate window |
| 2026-10-19 | Folder picker | The Windows `IFileOpenDialog` through the `windows` crate, which WebView2 already builds; a dialog plugin would add a download and more code for one dialog |
| 2026-10-19 | Changing the notes folder | Point Karsa at the new folder without moving files; moving notes is left to Explorer, where the user can see what happens |
| 2026-10-19 | "Show test alert" in the tray | Development builds only |
| 2026-10-19 | Countdown cost | Keep seconds in the pill (as designed) but stop 1 px width flips from restarting the width spring; 4.4% → 0.8% of a core |
| 2026-10-20 | Clipboard history storage | Text only, on disk for 24 hours, encrypted with the user's Windows account; private copies skipped; Pause and Clear on the Shelf |
| 2026-10-20 | File shelf | Links to the originals, not copies; drag out and Copy use the shell's own data object |
| 2026-10-20 | Media in the pill | Only while playing and nothing else needs the pill; Peek always has the controls |
| 2026-10-20 | Rampung timing | 17:00 on weekdays by default, waits for a running focus phase, "Later" = 30 minutes, at most once a day |
| 2026-10-20 | Where Clipboard and Files live | One "Shelf" icon tab with two halves, so the tab row still fits |
| 2026-10-20 | Drops from Explorer refused | Karsa registers its own drop target on its windows after they show, instead of relying on Tauri's; paste and "Add files…" added as other ways in |
| 2026-10-21 | File-type icons on the shelf | The icons Explorer shows (`SHGetFileInfoW`), so a `.pptx` shows PowerPoint's; the extension on a small page for types Windows does not know. Real icons over Karsa-drawn ones: they are what the user recognises, and they cover every installed app |
| 2026-10-21 | Which app's media to show | Windows' current session, unless the user picks another in Now playing's switcher; every session is listed |
| 2026-10-21 | Telling the site in a browser | From the browser's window titles, locally, once per track; a background tab shows just the browser |
| 2026-10-21 | Lyrics transport | WinRT `HttpClient` (Windows' own stack and proxy), no new library; every answer cached so a song is asked for once |
| 2026-10-22 | How to read the work calendar | Classic Outlook's COM object model, read only, while Outlook runs: no network, sign-in, app registration or administrator. Microsoft Graph if the company moves to the new Outlook |
| 2026-10-22 | Where Outlook meetings live in Karsa | In memory only, read again every 5 minutes; not copied into the database. Their alerts are scheduled by the Outlook thread, not stored as reminders |
| 2026-10-22 | People in a meeting note from Outlook | The organizer and up to 11 attendees, without the user |
| 2026-10-22 | Indonesian holidays in the calendar | A table in the app from the SKB 3 Menteri (2025–2027), added to by hand each year, rather than an online holiday API: no network, and the government's dates for Islamic holidays can differ from computed ones. Holidays in abang, cuti bersama softer, Saturdays unchanged |
| 2026-10-22 | Lyric focus and reminders | While lyric focus sings it keeps the pill from reminders and meetings (alerts still come); a running focus timer still goes first |
| 2026-10-22 | Lyric focus between lines | A music note in the pill, not the song's title |
| 2026-10-22 | Peek while music plays | Now playing, with a "Panel" button; the usual Peek when paused or silent |
| 2026-10-22 | What a note project is | A folder in the notes folder, read from disk each time (no database table), so File Explorer and Karsa always agree; the top `Meetings` folder and hidden folders are not projects |
| 2026-10-22 | Deleting a note project | Only when empty, to the Recycle Bin; a project with notes asks for them to be moved or deleted first |
| 2026-10-22 | Milestone order | Outlook meetings became M9; the UX playbook moved to M10 |
| 2026-10-21 | Lyric focus size | On the user's feedback: the notch 440 px wide (design v2 200, then 360), the corner 320 × 52 with two rows (design v2 220 × 32, one row); text steps down to 11 px before an ellipsis |
| 2026-10-21 | A shadow under the notch | On the user's request, so the notch floats; first a thin one (9 px), which hardly showed. The user's screen is dark under the notch, where no shadow shows, so it became a light 1 px hairline plus a denser shadow reaching 17 px, with the notch's region padding raised to 18 px. The corner pill keeps only its edge |
| 2026-10-21 | Reaching Now playing from the open panel | A chip in the panel header while something plays; no fourth icon tab |
| 2026-10-21 | Drops still refused with a real drag | WebView2 takes the drop itself and the page hands the files to the core with their paths (`postMessageWithAdditionalObjects`); OLE targets in Karsa's process are never reached from the WebView2 window under the cursor |
| 2026-10-20 | Interval reminders ("drink water every 30 min, 07:00–17:00, weekdays") | A fifth repeat with a daily window; nudges stay out of the pill and skip missed slots |
| 2026-10-20 | Unreadable dropdown lists | Every `<option>` gets the popover colours; the native list opens in its own popup and ignored the page's dark scheme |
| 2026-10-20 | Default quick-capture shortcut | Win+Shift+K, chosen from a short list in Settings instead of a key recorder (Windows keeps many Win combinations for itself) |
| 2026-10-21 | Notch shape | Flush with the top edge (square top corners), like a camera notch, rather than a pill floating below it; the top few pixels of the screen are not left as a sliver above it |
| 2026-10-21 | Notch size | 80 × 26 idle, text capped at 200 px; Peek, alerts and the panel keep their sizes so their content needs no second layout |
| 2026-10-21 | Later reminders in the notch | Left to Peek; the notch widens only for what needs attention now (alerts, reminders within 15 minutes, focus, playing media) |
