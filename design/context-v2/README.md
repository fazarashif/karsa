# Karsa as built (context for design v2)

Screenshots of the app after M7 (2026-10-21), for Claude Design. They were taken with sample data on a test instance, at 2x, on a neutral backdrop, in both positions: `corner-*` (bottom right, above the taskbar) and `notch-*` (top centre). The island is 400 × 540 logical px; small states are cropped to the strip they occupy.

The three `preview-*` shots come from the UI preview (`npm run dev`, with its built-in sample data) at 1x. They show what the test instance could not: a woven kain, a clipboard history, and the Rampung offer.

| File | What it shows |
|---|---|
| `*-01-idle-eyes` | The idle pill / notch with the mripat (nothing due) |
| `*-02-idle-media` | The pill while music plays: album art and title (sample art) |
| `*-03-peek-media` | Peek with the Playing row (previous, play/pause, next) |
| `*-04-idle-next-up` | Corner: the next reminder today in the pill. Notch: still the eyes, because the notch leaves later reminders to Peek |
| `*-05-peek` | Peek: date and pasaran, focus, next up, the top three tasks |
| `*-06-live-focus` | A running focus countdown in the pill |
| `*-07-peek-focus` | Peek with the timer, its progress and controls |
| `*-08-alert-meeting` | A meeting reminder alert (+ Meeting note, Snooze 5m, Dismiss) |
| `*-09-alert-waiting` | The alert waiting in the pill after 8 s |
| `*-10-drop-target` | Files dragged over the island |
| `*-11-panel-todo` | Todo: Inbox with a subtask count, deadlines, completed section |
| `*-12-panel-focus` | Focus, ready, with an empty kain (no sessions yet on the test instance) |
| `*-13-panel-notes` | Notes list: a note and a meeting note, with filters and search |
| `*-14-panel-meeting-note` | The meeting note editor: when, topic, attendees, notes, action items |
| `*-15-panel-reminders` | Reminders by day, including an "every 30 min" reminder and a todo deadline |
| `*-17-panel-shelf-files` | Shelf › Files with the real Windows file icons and extension badges |
| `*-18-panel-settings` | Settings: You (name, birthday, weton) and General (Position) |
| `*-19-rampung` | The Rampung end-of-day view |
| `*-20-quick-capture` | Quick capture (Win+Shift+K) |
| `preview-alert-rampung-offer` | The Rampung offer alert at the end of a weekday |
| `preview-panel-focus-kain` | Focus with four weeks of kain and the streak |
| `preview-panel-shelf-clipboard` | Shelf › Clipboard with history |

Number 16 (the clipboard on the test instance) is left out on purpose: the preview shot shows it with sample text instead.
