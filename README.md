# Karsa

A lightweight "dynamic island" companion for Windows 11: a small black pill in the bottom-right corner that grows into todos, a Pomodoro timer, notes, meeting notes and reminders.

<img src="design/screens/10-expanded-todo.png" alt="The Karsa panel open on the Todo tab" width="400">

## Download

1. Go to [Releases](https://github.com/fazarashif/karsa/releases/latest) and download `Karsa_0.1.0_x64-setup.exe`.
2. Run it. It installs for your Windows account only, so it needs no administrator rights.
3. The installer is not code-signed, so Windows SmartScreen may say "Windows protected your PC". Click **More info**, then **Run anyway**.

Karsa needs Windows 11 (64-bit). It appears as a small pill above the taskbar on the right; hover it for a summary, click it to open the panel. Right-click the tray icon for Settings and Quit. It starts with Windows; turn that off in Settings.

Everything stays on your PC: Karsa makes no network requests. Tasks, reminders and settings are in `%APPDATA%\app.karsa.desktop`, and notes are plain Markdown files in `Documents\Karsa\Notes` (or the folder you choose in Settings).

To uninstall, use Windows Settings → Apps → Installed apps → Karsa. That removes the app, its shortcut and its Start with Windows entry. Your data stays unless you tick "Delete the application data" in the uninstaller; notes are never deleted.

## About this project

See [docs/requirements.md](docs/requirements.md) for the full v1 requirements and milestones. The visual design system is [design/DESIGN.md](design/DESIGN.md) (tokens in `design/tokens.css`, assets in `design/assets`, mockups in `design/screens`); it was made from the brief in [docs/claude-design-brief.md](docs/claude-design-brief.md).

## Status

**v0.1.0 — all v1 milestones (M0–M5) done.**

- **Island (M0):** Idle, Peek, Expanded and Alert states, click-through, focus handling, fullscreen hiding, tray icon, autostart.
- **Design system v1:** black island with Javanese accents (kawung, aksara watermark, *mripat* eyes, Javanese greetings).
- **Todo (M1):** lists, tasks, subtasks and due dates, saved in SQLite. The next task due today shows in the pill and in Peek.
- **Focus (M2):** Pomodoro timer run by the core, with a live countdown in the pill and Peek, the Focus tab, and silent phase-end alerts.
- **Reminders (M3):** one-off and repeating reminders with lead time and snooze, linked todo deadlines, a missed-reminders summary, and the next reminder in the pill and Peek.
- **Notes (M4):** Markdown notes and structured meeting notes (when, topic, attendees, notes, action items) in a folder of `.md` files, with search, "Add to todo" for action items, conflict detection for edits made elsewhere, and an "Open full" window.
- **Settings and polish (M5):** a Settings tab (Start with Windows, hide during fullscreen, Pomodoro lengths, default reminder lead time, notes folder), a per-user installer, and a performance pass that cut a running countdown from 4.4% to 0.8% of one core.

Release build, measured: ~0% CPU and ~75 MB RAM while idle (including WebView2), 0.8% of one core while a focus countdown runs, 60 fps morphs, a 1.8 MB installer, and the pill visible 1–2.5 s after launch depending on CPU speed. Details in [docs/requirements.md](docs/requirements.md#2-performance-budget).

## Stack

- [Tauri 2](https://tauri.app) with a Rust core (`src-tauri/`)
- Svelte 5 + TypeScript UI (`src/`), built with Vite
- SQLite through `rusqlite` (bundled), stored in `%APPDATA%\app.karsa.desktop\karsa.db`
- Notes as plain Markdown files, in `Documents\Karsa\Notes` by default (Obsidian-compatible)
- Win32 APIs through `windows-sys`

## Development

Prerequisites: Node.js 20+, Rust (stable, MSVC toolchain), and the MSVC C++ build tools with the Windows SDK.

```powershell
npm install
npm run tauri dev
```

On a machine whose Visual Studio is missing the standard x64 C runtime libraries (see the comment in the script), load the build environment first, in the same PowerShell session:

```powershell
. .\scripts\msvc-env.ps1
npm run tauri dev
```

Other scripts:

| Command | What it does |
|---|---|
| `npm run dev` | UI only, in a browser at http://localhost:1420, with an in-memory stand-in for the todo storage. Use `karsa.alert()` and `karsa.fullscreen(true)` in the console to simulate core events. |
| `node scripts/import-design-assets.mjs` | Copies the SVGs the app uses from `design/assets` to `src/assets`, stripping the design tool's metadata. Run it after updating `design/`. |
| `npm run check` | Type-checks the Svelte/TypeScript code |
| `npm run tauri build` | Builds the release app and the installer, `src-tauri\target\release\bundle\nsis\Karsa_<version>_x64-setup.exe` (the first run downloads NSIS, ~2 MB). It installs per user into `%LOCALAPPDATA%\Karsa`. |
| `npm run tauri build -- --no-bundle` | Builds only `src-tauri/target/release/karsa.exe` |

Set `KARSA_DATA_DIR` to a folder to give a test or development instance its own database instead of `%APPDATA%\app.karsa.desktop`, and its own notes in `<folder>\Notes` (unless a notes folder is set in Settings).

Start with Windows is switched on automatically on first run only for installed copies; builds run from `src-tauri/target/` never register themselves. Toggle it in Settings or the tray menu.

## How the island works

- The window never resizes: it is a fixed 400 × 540 transparent, always-on-top window anchored to the bottom-right of the primary monitor's work area. Only the web content animates, which keeps the morph smooth.
- Everything outside the visible box is click-through: the UI reports the box size and the core clips the window to it with a window region (`SetWindowRgn`). The region grows before a morph and shrinks once it settles.
- The window is `WS_EX_NOACTIVATE` in every state except Expanded, so hovering or clicking the island never steals keyboard focus from the app you are using.
- Fullscreen apps are detected from shell notifications and foreground-window changes (`src-tauri/src/hooks.rs`), not by polling; the island hides until they exit.
- Visibility, focus and click-through are set with Win32 calls in `src-tauri/src/island.rs`, never through Tauri's window setters, which re-apply every style and call `ShowWindow` (hiding the island or stealing focus).
- WebView2 runs with software compositing (`additionalBrowserArgs` in `tauri.conf.json`). For this small surface it is as smooth as GPU compositing, uses less CPU while animating, and saves ~70 MB of memory.

## License

Copyright © 2026 Faza Rashif. All rights reserved. You are welcome to download and use Karsa; the source code is published for reference and is not licensed for reuse.
