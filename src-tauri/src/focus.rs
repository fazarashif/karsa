//! The Pomodoro timer. The core owns it: a running phase ends at an absolute wall-clock
//! time, one thread sleeps until then, and the UI only renders the state it is sent.
//! Nothing ticks while the timer is idle.

use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Duration;

use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::db::{now_ms, Db};
use crate::ISLAND;

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Focus,
    ShortBreak,
    LongBreak,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Nothing started (or stopped): the next phase is focus session 1.
    Ready,
    Running,
    Paused,
    /// A phase ended; the next one (in `phase`) waits for the user to start it.
    Waiting,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    focus_min: i64,
    short_break_min: i64,
    long_break_min: i64,
    sessions_per_cycle: u32,
    auto_start: bool,
}

impl Settings {
    /// Values stored in the settings table, or the defaults (25 / 5 / 15, 4 per cycle).
    pub fn load(db: &Db) -> Self {
        let minutes = |key: &str, default: i64| {
            db.setting(key).and_then(|v| v.parse().ok()).unwrap_or(default).clamp(1, 180)
        };
        Self {
            focus_min: minutes("focus.focus_min", 25),
            short_break_min: minutes("focus.short_break_min", 5),
            long_break_min: minutes("focus.long_break_min", 15),
            sessions_per_cycle: db
                .setting("focus.sessions_per_cycle")
                .and_then(|v| v.parse().ok())
                .unwrap_or(4u32)
                .clamp(1, 12),
            auto_start: db.setting("focus.auto_start").as_deref() == Some("true"),
        }
    }

    fn phase_ms(&self, phase: Phase) -> i64 {
        60_000
            * match phase {
                Phase::Focus => self.focus_min,
                Phase::ShortBreak => self.short_break_min,
                Phase::LongBreak => self.long_break_min,
            }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusState {
    phase: Phase,
    status: Status,
    /// Wall-clock ms at which the running phase ends.
    ends_at: Option<i64>,
    /// Time left in the phase while it is paused, ready or waiting.
    remaining_ms: i64,
    /// Full length of the phase, for progress bars.
    duration_ms: i64,
    /// The focus session this phase belongs to, counted from 1 within the cycle.
    session: u32,
    settings: Settings,
}

impl FocusState {
    fn ready(settings: Settings) -> Self {
        let duration_ms = settings.phase_ms(Phase::Focus);
        Self {
            phase: Phase::Focus,
            status: Status::Ready,
            ends_at: None,
            remaining_ms: duration_ms,
            duration_ms,
            session: 1,
            settings,
        }
    }

    fn enter(&mut self, phase: Phase, session: u32, status: Status) {
        self.phase = phase;
        self.session = session;
        self.status = status;
        self.duration_ms = self.settings.phase_ms(phase);
        self.remaining_ms = self.duration_ms;
        self.ends_at = (status == Status::Running).then(|| now_ms() + self.duration_ms);
    }

    /// The phase that follows this one, and the session it belongs to.
    fn following(&self) -> (Phase, u32) {
        match self.phase {
            Phase::Focus if self.session >= self.settings.sessions_per_cycle => (Phase::LongBreak, self.session),
            Phase::Focus => (Phase::ShortBreak, self.session),
            Phase::ShortBreak => (Phase::Focus, self.session + 1),
            Phase::LongBreak => (Phase::Focus, 1),
        }
    }
}

pub struct Focus {
    state: Mutex<FocusState>,
    wake: Condvar,
}

impl Focus {
    pub fn new(settings: Settings) -> Self {
        Self { state: Mutex::new(FocusState::ready(settings)), wake: Condvar::new() }
    }

    fn lock(&self) -> MutexGuard<'_, FocusState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Wakes the timer thread, e.g. after the PC resumes from sleep.
    pub fn notify(&self) {
        self.wake.notify_all();
    }

    /// Applies a change, wakes the timer thread, and tells the UI.
    fn update(&self, app: &AppHandle, change: impl FnOnce(&mut FocusState)) -> FocusState {
        let state = {
            let mut s = self.lock();
            change(&mut s);
            s.clone()
        };
        self.wake.notify_all();
        let _ = app.emit_to(ISLAND, "focus://state", &state);
        state
    }

    /// Ends the running phase: records a finished focus session, moves to the next phase
    /// (running if auto-start is on, otherwise waiting), and returns the alert to show.
    fn complete(&self, app: &AppHandle, s: &mut FocusState) -> Value {
        let finished = s.phase;
        let ended_at = s.ends_at.unwrap_or_else(now_ms);
        if finished == Phase::Focus {
            if let Some(db) = app.try_state::<Db>() {
                let _ = db.conn().execute(
                    "INSERT INTO focus_sessions (started_at, ended_at, duration_ms) VALUES (?1, ?2, ?3)",
                    params![ended_at - s.duration_ms, ended_at, s.duration_ms],
                );
            }
        }
        let (next, session) = s.following();
        let auto = s.settings.auto_start;
        s.enter(next, session, if auto { Status::Running } else { Status::Waiting });

        let of = s.settings.sessions_per_cycle;
        let when = if auto { "started" } else { "next" };
        let action = if auto { "auto" } else if finished == Phase::Focus { "break-next" } else { "focus-next" };
        let (title, subtitle) = match finished {
            Phase::Focus => {
                let kind = if next == Phase::LongBreak { "long break" } else { "short break" };
                ("Focus session done", format!("Session {session} of {of} · {kind} {when}"))
            }
            _ => ("Break's over", format!("Session {session} of {of} {when}")),
        };
        json!({ "title": title, "subtitle": subtitle, "kind": "focus", "focus": action })
    }
}

/// Sleeps until the running phase ends. Waits on wall-clock time in steps of at most a
/// minute, so a phase that ends while the PC sleeps completes right after it wakes.
pub fn spawn_timer(app: AppHandle, focus: Arc<Focus>) -> std::io::Result<()> {
    std::thread::Builder::new().name("karsa-focus".into()).spawn(move || loop {
        let mut s = focus.lock();
        match (s.status, s.ends_at) {
            (Status::Running, Some(ends_at)) => {
                let left = ends_at - now_ms();
                if left <= 0 {
                    let alert = focus.complete(&app, &mut s);
                    let state = s.clone();
                    drop(s);
                    let _ = app.emit_to(ISLAND, "focus://state", &state);
                    let _ = app.emit_to(ISLAND, "island://alert", alert);
                } else {
                    let step = Duration::from_millis(left.min(60_000) as u64);
                    drop(focus.wake.wait_timeout(s, step));
                }
            }
            _ => drop(focus.wake.wait(s)),
        }
    })?;
    Ok(())
}

type FocusRef<'a> = State<'a, Arc<Focus>>;

#[tauri::command]
pub fn focus_state(focus: FocusRef<'_>) -> FocusState {
    focus.lock().clone()
}

/// Starts focus session 1 from Ready, or the phase that is waiting.
#[tauri::command]
pub fn focus_start(app: AppHandle, focus: FocusRef<'_>) -> FocusState {
    focus.update(&app, |s| match s.status {
        Status::Ready => s.enter(Phase::Focus, 1, Status::Running),
        Status::Waiting => s.enter(s.phase, s.session, Status::Running),
        Status::Running | Status::Paused => {}
    })
}

#[tauri::command]
pub fn focus_pause(app: AppHandle, focus: FocusRef<'_>) -> FocusState {
    focus.update(&app, |s| {
        if let (Status::Running, Some(ends_at)) = (s.status, s.ends_at) {
            s.status = Status::Paused;
            s.remaining_ms = (ends_at - now_ms()).max(0);
            s.ends_at = None;
        }
    })
}

#[tauri::command]
pub fn focus_resume(app: AppHandle, focus: FocusRef<'_>) -> FocusState {
    focus.update(&app, |s| {
        if s.status == Status::Paused {
            s.status = Status::Running;
            s.ends_at = Some(now_ms() + s.remaining_ms);
        }
    })
}

/// Leaves the current (or waiting) phase without recording it and starts the next one.
#[tauri::command]
pub fn focus_skip(app: AppHandle, focus: FocusRef<'_>) -> FocusState {
    focus.update(&app, |s| {
        if s.status != Status::Ready {
            let (next, session) = s.following();
            s.enter(next, session, Status::Running);
        }
    })
}

#[tauri::command]
pub fn focus_stop(app: AppHandle, focus: FocusRef<'_>) -> FocusState {
    focus.update(&app, |s| *s = FocusState::ready(s.settings))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatch {
    focus_min: Option<i64>,
    short_break_min: Option<i64>,
    long_break_min: Option<i64>,
    sessions_per_cycle: Option<u32>,
    auto_start: Option<bool>,
}

/// Saves timer settings from the Settings screen. A running phase keeps
/// its length; a ready or waiting one picks up the new length right away.
#[tauri::command]
pub fn focus_update_settings(
    app: AppHandle,
    db: State<'_, Db>,
    focus: FocusRef<'_>,
    patch: SettingsPatch,
) -> Result<FocusState, String> {
    let values = [
        ("focus.focus_min", patch.focus_min.map(|v| v.to_string())),
        ("focus.short_break_min", patch.short_break_min.map(|v| v.to_string())),
        ("focus.long_break_min", patch.long_break_min.map(|v| v.to_string())),
        ("focus.sessions_per_cycle", patch.sessions_per_cycle.map(|v| v.to_string())),
        ("focus.auto_start", patch.auto_start.map(|v| v.to_string())),
    ];
    {
        let conn = db.conn();
        for (key, value) in values {
            let Some(value) = value else { continue };
            conn.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![key, value],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    let settings = Settings::load(&db);
    Ok(focus.update(&app, |s| {
        s.settings = settings;
        if matches!(s.status, Status::Ready | Status::Waiting) {
            s.enter(s.phase, s.session, s.status);
        }
    }))
}

#[derive(Serialize)]
pub struct FocusStats {
    sessions: i64,
    minutes: i64,
}

/// Finished focus sessions since `since` (the UI passes local midnight).
#[tauri::command]
pub fn focus_stats(db: State<'_, Db>, since: i64) -> Result<FocusStats, String> {
    db.conn()
        .query_row(
            "SELECT COUNT(*), IFNULL(SUM(duration_ms), 0) / 60000 FROM focus_sessions WHERE ended_at >= ?1",
            [since],
            |r| Ok(FocusStats { sessions: r.get(0)?, minutes: r.get(1)? }),
        )
        .map_err(|e| e.to_string())
}

/// End time and length (ms) of each focus session finished since `since`, oldest first.
/// The UI groups them by local day for the kain and the streak.
#[tauri::command]
pub fn focus_history(db: State<'_, Db>, since: i64) -> Result<Vec<(i64, i64)>, String> {
    let conn = db.conn();
    let mut stmt = conn
        .prepare("SELECT ended_at, duration_ms FROM focus_sessions WHERE ended_at >= ?1 ORDER BY ended_at")
        .map_err(|e| e.to_string())?;
    let rows = stmt.query_map([since], |r| Ok((r.get(0)?, r.get(1)?))).map_err(|e| e.to_string())?;
    rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
}
