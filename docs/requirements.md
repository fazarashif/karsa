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

- **Hover:** shows Peek after a 300 ms delay and closes it 400 ms after the cursor leaves. The delay prevents accidental opens when the cursor passes by.
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
- **Morph strategy:** the window never resizes. It is a fixed 400 × 540 window covering the largest state; only the web content animates. A window region (`SetWindowRgn`) clips the window to the visible box, so the window neither draws nor takes clicks anywhere else. The region grows immediately when a state gets bigger and shrinks ~450 ms after it gets smaller, once the spring has mostly settled.
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

- **Fields:** title, date & time, type (General / Meeting / Deadline), repeat (none, daily, weekdays, weekly), lead time (at time, 5, 10, 15 or 30 min before).
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
- **General:** Start with Windows (the Windows Run key, kept in step with the tray's check mark either way) and Hide during fullscreen. With hiding off, the island stays on top of fullscreen apps and alerts show there too.
- **Focus:** steppers for Focus (5–120 min, steps of 5), Short break (1–30 min), Long break (5–60 min, steps of 5) and Long break after (2–8 sessions), plus "Start the next phase by itself". A running phase keeps its length; a ready or waiting one changes right away.
- **Reminders:** "Remind me" (At time, 5, 10, 15 or 30 min before): the starting lead time of new reminders and of new todo deadline reminders. Existing reminders keep their own.
- **Notes:** the folder path, **Change…** (the Windows folder picker), **Open** (Explorer) and **Use default**. Karsa then reads and saves notes in the new folder; notes already saved are not moved.
- **Storage:** keys `island.hide_in_fullscreen`, `reminders.default_lead_min`, `notes.folder` and the `focus.*` keys in the `settings` table.

## 7. Data and storage

| Data | Location |
|---|---|
| Lists, tasks, subtasks, reminders, Pomodoro sessions, settings | `%APPDATA%\app.karsa.desktop\karsa.db` (SQLite, WAL mode, schema version in `PRAGMA user_version`) |
| Notes | `<notes folder>\*.md` |
| Meeting notes | `<notes folder>\Meetings\YYYY-MM-DD Title.md` |

The notes folder can be pointed at OneDrive or Google Drive for backup and sync. There is no built-in cloud sync.

## 8. Language and locale

- UI language: English.
- Dates and times follow the Windows regional settings.

## 9. Out of scope for v1

- Water and stand-up reminders
- Clipboard history (24 hours)
- Calendar sync (ICS, Google Calendar, Outlook)
- Global hotkey and quick capture
- Media controls
- Linking Pomodoro sessions to tasks; daily summary
- Meeting recording, transcription and AI summaries
- Drag-to-reposition, choosing a monitor
- File shelf

## 10. Milestones

| # | Milestone | Done when | Status |
|---|---|---|---|
| M0 | Island shell prototype | Pill sits bottom-right; Idle / Peek / Expanded states with smooth morph; hover and click behave as specified; no focus stealing; hides during fullscreen; tray icon; autostart; idle CPU ~0% and RAM within budget | Done 2026-10-19 |
| M1 | Todo | Lists, tasks, subtasks, due dates; Peek shows top tasks | Done 2026-10-19, with design system v1 |
| M2 | Focus | Pomodoro with live activity in the pill and silent phase-end alerts | Done 2026-10-19 |
| M3 | Reminders | Manual reminders with repeat, lead time, snooze, missed-reminder handling; todo deadlines linked | Done 2026-10-19 |
| M4 | Notes & meeting notes | Markdown notes with search, the meeting template, and the "Open full" window | Done 2026-10-19 |
| M5 | Polish | Settings screen, installer, performance pass against the budget | Done 2026-10-19 (cold start above target while the CPU is throttled; see section 2) |

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
