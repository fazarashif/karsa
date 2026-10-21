//! Reminders: storage, the link to todo deadlines, and the scheduler that fires alerts.
//!
//! One thread sleeps until the next alert is due. It wakes when reminders change, when
//! the PC resumes or the clock changes, and at least every 15 minutes as a safety net.
//! Alerts that fire more than a minute late (Karsa was closed or the PC was asleep) are
//! gathered into one "missed reminders" alert.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Duration;

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::db::{now_ms, Db};
use crate::localtime::{add_local_days, format_time, is_weekend, local_to_ms, to_local};
use crate::ISLAND;

const MISSED_AFTER_MS: i64 = 60_000;
const SAFETY_WAKE_MS: i64 = 15 * 60_000;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    General,
    Meeting,
    Deadline,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Repeat {
    None,
    Daily,
    Weekdays,
    Weekly,
    /// Every `every_min` minutes within a daily window (see `Window`).
    Interval,
}

impl Kind {
    fn as_str(self) -> &'static str {
        match self {
            Kind::General => "general",
            Kind::Meeting => "meeting",
            Kind::Deadline => "deadline",
        }
    }

    fn parse(value: &str) -> Self {
        match value {
            "meeting" => Kind::Meeting,
            "deadline" => Kind::Deadline,
            _ => Kind::General,
        }
    }
}

impl Repeat {
    fn as_str(self) -> &'static str {
        match self {
            Repeat::None => "none",
            Repeat::Daily => "daily",
            Repeat::Weekdays => "weekdays",
            Repeat::Weekly => "weekly",
            Repeat::Interval => "interval",
        }
    }

    fn parse(value: &str) -> Self {
        match value {
            "daily" => Repeat::Daily,
            "weekdays" => Repeat::Weekdays,
            "weekly" => Repeat::Weekly,
            "interval" => Repeat::Interval,
            _ => Repeat::None,
        }
    }
}

/// When an "every N minutes" reminder goes off: every `every` minutes from `from` until
/// `until` (minutes after local midnight, both included), on weekdays or every day.
#[derive(Clone, Copy)]
struct Window {
    every: i64,
    from: i64,
    until: i64,
    weekdays_only: bool,
}

impl Window {
    fn of(r: &Reminder) -> Option<Window> {
        Some(Window {
            every: r.every_min?,
            from: r.window_from?,
            until: r.window_until?,
            weekdays_only: r.window_days.as_deref() == Some("weekdays"),
        })
    }

    /// The first slot strictly after `after`, looking up to two weeks ahead.
    fn next_slot(self, after: i64) -> Option<i64> {
        let at_minutes = |day: i64, minutes: i64| {
            let t = to_local(day);
            local_to_ms(t.wYear, t.wMonth, t.wDay, (minutes / 60) as u16, (minutes % 60) as u16)
        };
        let every = self.every.max(1) * 60_000;
        let mut day = after;
        for _ in 0..14 {
            let start = at_minutes(day, self.from);
            let end = at_minutes(day, self.until);
            if !(self.weekdays_only && is_weekend(start)) {
                let steps = if after < start { 0 } else { (after - start) / every + 1 };
                let slot = start + steps * every;
                if slot <= end {
                    return Some(slot);
                }
            }
            day = add_local_days(start, 1);
        }
        None
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    id: i64,
    title: String,
    kind: Kind,
    /// Unix ms of the event itself (a meeting's start, a deadline).
    at: i64,
    repeat: Repeat,
    lead_min: i64,
    snoozed_until: Option<i64>,
    fired: bool,
    /// Set for a todo task's linked deadline.
    task_id: Option<i64>,
    /// "Every N minutes" reminders only.
    every_min: Option<i64>,
    window_from: Option<i64>,
    window_until: Option<i64>,
    /// "weekdays" or "daily".
    window_days: Option<String>,
}

const COLUMNS: &str =
    "id, title, kind, at, repeat, lead_min, snoozed_until, fired, task_id, every_min, window_from, window_until, window_days";

fn read(r: &Row) -> rusqlite::Result<Reminder> {
    Ok(Reminder {
        id: r.get(0)?,
        title: r.get(1)?,
        kind: Kind::parse(&r.get::<_, String>(2)?),
        at: r.get(3)?,
        repeat: Repeat::parse(&r.get::<_, String>(4)?),
        lead_min: r.get(5)?,
        snoozed_until: r.get(6)?,
        fired: r.get(7)?,
        task_id: r.get(8)?,
        every_min: r.get(9)?,
        window_from: r.get(10)?,
        window_until: r.get(11)?,
        window_days: r.get(12)?,
    })
}

/// When the alert for a reminder goes off.
fn fire_time(r: &Reminder) -> i64 {
    r.snoozed_until.unwrap_or(r.at - r.lead_min * 60_000)
}

/// Reminders still to come (repeating ones never finish).
pub fn list(conn: &Connection) -> rusqlite::Result<Vec<Reminder>> {
    conn.prepare(&format!("SELECT {COLUMNS} FROM reminders WHERE fired = 0 ORDER BY at, id"))?
        .query_map([], read)?
        .collect()
}

/// The first occurrence after `at` whose alert is still ahead of `after`.
fn next_occurrence(at: i64, repeat: Repeat, lead_ms: i64, after: i64) -> Option<i64> {
    let mut next = at;
    // Ten years of daily occurrences is plenty for catching up after a long absence.
    for _ in 0..3700 {
        next = match repeat {
            // Interval reminders are scheduled from their window (Window::next_slot).
            Repeat::None | Repeat::Interval => return None,
            Repeat::Daily => add_local_days(next, 1),
            Repeat::Weekly => add_local_days(next, 7),
            Repeat::Weekdays => {
                let mut day = add_local_days(next, 1);
                while is_weekend(day) {
                    day = add_local_days(day, 1);
                }
                day
            }
        };
        if next - lead_ms > after {
            return Some(next);
        }
    }
    None
}

/// Settings → "Remind me", the lead time new reminders and todo deadlines start with.
pub const DEFAULT_LEAD_KEY: &str = "reminders.default_lead_min";

pub fn default_lead_min(conn: &Connection) -> i64 {
    conn.query_row("SELECT value FROM settings WHERE key = ?1", [DEFAULT_LEAD_KEY], |r| r.get::<_, String>(0))
        .optional()
        .ok()
        .flatten()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0i64)
        .clamp(0, 24 * 60)
}

/// Keeps a task's linked deadline reminder in step with the task: it exists while the
/// task is open and has a due time, and follows the task's title and due time. Called
/// inside the transaction that changed the task. A new link starts with the default lead
/// time; after that the lead time is the reminder's own.
pub fn sync_task(conn: &Connection, task_id: i64) -> rusqlite::Result<()> {
    let task = conn
        .query_row(
            "SELECT title, done, due_at, due_has_time FROM tasks WHERE id = ?1",
            [task_id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, bool>(1)?, r.get::<_, Option<i64>>(2)?, r.get::<_, bool>(3)?)),
        )
        .optional()?;
    match task {
        Some((title, false, Some(due_at), true)) => {
            // A due time already in the past does not alert (it is just overdue). One still
            // ahead whose lead time has already begun alerts right away, as in the form.
            conn.execute(
                "INSERT INTO reminders (title, kind, at, task_id, created_at, fired, lead_min, snoozed_until)
                 VALUES (?1, 'deadline', ?2, ?3, ?4, ?2 <= ?4, ?5,
                         CASE WHEN ?2 > ?4 AND ?2 - ?5 * 60000 <= ?4 THEN ?4 END)
                 ON CONFLICT(task_id) DO UPDATE SET
                   title = excluded.title,
                   fired = CASE WHEN reminders.at = excluded.at THEN reminders.fired ELSE excluded.at <= ?4 END,
                   snoozed_until = CASE
                     WHEN reminders.at = excluded.at THEN reminders.snoozed_until
                     WHEN excluded.at > ?4 AND excluded.at - reminders.lead_min * 60000 <= ?4 THEN ?4 END,
                   at = excluded.at",
                params![title, due_at, task_id, now_ms(), default_lead_min(conn)],
            )?;
        }
        _ => {
            conn.execute("DELETE FROM reminders WHERE task_id = ?1", [task_id])?;
        }
    }
    Ok(())
}

pub struct ReminderEngine {
    dirty: Mutex<bool>,
    wake: Condvar,
    ready: AtomicBool,
}

impl ReminderEngine {
    pub fn new() -> Self {
        Self { dirty: Mutex::new(false), wake: Condvar::new(), ready: AtomicBool::new(false) }
    }

    fn lock(&self) -> MutexGuard<'_, bool> {
        self.dirty.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Re-evaluates the schedule now: reminders changed, the PC resumed, or the clock moved.
    pub fn poke(&self) {
        *self.lock() = true;
        self.wake.notify_all();
    }

    /// Starts firing once the UI can show alerts, so reminders missed while Karsa was
    /// closed are not lost to a window that has not loaded yet.
    pub fn start(&self) {
        self.ready.store(true, Ordering::Relaxed);
        self.poke();
    }
}

fn next_fire(app: &AppHandle) -> Option<i64> {
    app.state::<Db>()
        .conn()
        .query_row(
            "SELECT MIN(COALESCE(snoozed_until, at - lead_min * 60000)) FROM reminders WHERE fired = 0",
            [],
            |r| r.get::<_, Option<i64>>(0),
        )
        .ok()
        .flatten()
}

/// Fires every reminder that is due: one-offs are marked done, repeating ones move to
/// their next occurrence, snoozes are cleared. Returns false if the change could not be
/// saved (the caller backs off instead of firing in a loop).
fn fire_due(app: &AppHandle, now: i64) -> bool {
    let db = app.state::<Db>();
    let due = {
        let mut conn = db.conn();
        let Ok(tx) = conn.transaction() else { return false };
        let due: Vec<Reminder> = match tx
            .prepare(&format!(
                "SELECT {COLUMNS} FROM reminders
                 WHERE fired = 0 AND COALESCE(snoozed_until, at - lead_min * 60000) <= ?1 ORDER BY at, id"
            ))
            .and_then(|mut s| s.query_map([now], read)?.collect())
        {
            Ok(due) => due,
            Err(_) => return false,
        };
        for r in &due {
            let result = if r.snoozed_until.is_some() || r.repeat == Repeat::None {
                // A snooze only delayed this alert; a one-off is finished once it fires.
                tx.execute(
                    "UPDATE reminders SET snoozed_until = NULL, fired = ?1 WHERE id = ?2",
                    params![r.repeat == Repeat::None, r.id],
                )
            } else {
                let next = match r.repeat {
                    Repeat::Interval => Window::of(r).and_then(|w| w.next_slot(now)),
                    _ => next_occurrence(r.at, r.repeat, r.lead_min * 60_000, now),
                };
                match next {
                    Some(next) => tx.execute("UPDATE reminders SET at = ?1 WHERE id = ?2", params![next, r.id]),
                    None => tx.execute("UPDATE reminders SET fired = 1 WHERE id = ?1", [r.id]),
                }
            };
            if result.is_err() {
                return false;
            }
        }
        if tx.commit().is_err() {
            return false;
        }
        due
    };
    if !due.is_empty() {
        // An "every N minutes" nudge is never "missed": shortly late it alerts as usual,
        // and once its next slot has come it is skipped (the PC was asleep or off).
        let fired: Vec<(Reminder, bool)> = due
            .into_iter()
            .filter_map(|r| {
                let late = now - fire_time(&r);
                if r.repeat == Repeat::Interval {
                    let every = r.every_min.unwrap_or(30) * 60_000;
                    return (late <= every).then_some((r, false));
                }
                Some((r, late > MISSED_AFTER_MS))
            })
            .collect();
        if !fired.is_empty() {
            let _ = app.emit_to(ISLAND, "island://alert", alert_for(&fired, now));
        }
        emit_list(app);
    }
    true
}

fn alert_for(fired: &[(Reminder, bool)], now: i64) -> Value {
    if let [(r, false)] = fired {
        let minutes = (r.at - now + 30_000).div_euclid(60_000);
        let subtitle = match r.kind {
            Kind::General if minutes >= 1 => format!("Reminder · in {minutes} min"),
            Kind::General => "Reminder · now".to_string(),
            Kind::Meeting if minutes >= 1 => format!("Meeting · starts in {minutes} min"),
            Kind::Meeting => "Meeting · now".to_string(),
            Kind::Deadline => format!("Deadline · due {}", format_time(r.at)),
        };
        return json!({ "title": r.title, "subtitle": subtitle, "kind": r.kind, "reminder": r.id, "at": r.at });
    }
    let missed = fired.iter().any(|(_, missed)| *missed);
    let title = match (missed, fired.len()) {
        (true, 1) => "1 missed reminder".to_string(),
        (true, n) => format!("{n} missed reminders"),
        (false, n) => format!("{n} reminders"),
    };
    let subtitle = fired
        .iter()
        .map(|(r, _)| format!("{} {}", r.title, format_time(r.at)))
        .collect::<Vec<_>>()
        .join(" · ");
    json!({ "title": title, "subtitle": subtitle, "kind": "general", "summary": true })
}

fn emit_list(app: &AppHandle) {
    if let Ok(reminders) = list(&app.state::<Db>().conn()) {
        let _ = app.emit_to(ISLAND, "reminders://changed", reminders);
    }
}

/// Call after reminders change outside the scheduler (edits, todo deadlines).
pub fn changed(app: &AppHandle) {
    if let Some(engine) = app.try_state::<Arc<ReminderEngine>>() {
        engine.poke();
    }
    emit_list(app);
}

pub fn spawn(app: AppHandle, engine: Arc<ReminderEngine>) -> std::io::Result<()> {
    std::thread::Builder::new().name("karsa-reminders".into()).spawn(move || loop {
        let next = if engine.ready.load(Ordering::Relaxed) { next_fire(&app) } else { None };
        let now = now_ms();
        if let Some(at) = next {
            if at <= now {
                if !fire_due(&app, now) {
                    std::thread::sleep(Duration::from_secs(60));
                }
                continue;
            }
        }
        let mut dirty = engine.lock();
        if !*dirty {
            dirty = match next {
                Some(at) => {
                    let wait = Duration::from_millis((at - now).min(SAFETY_WAKE_MS) as u64);
                    match engine.wake.wait_timeout(dirty, wait) {
                        Ok((guard, _)) => guard,
                        Err(poisoned) => poisoned.into_inner().0,
                    }
                }
                None => engine.wake.wait(dirty).unwrap_or_else(|poisoned| poisoned.into_inner()),
            };
        }
        *dirty = false;
    })?;
    Ok(())
}

type ListResult = Result<Vec<Reminder>, String>;

#[tauri::command]
pub fn reminders_list(db: State<'_, Db>) -> ListResult {
    list(&db.conn()).map_err(|e| e.to_string())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderInput {
    id: Option<i64>,
    title: String,
    kind: Kind,
    at: i64,
    repeat: Repeat,
    lead_min: i64,
    /// For `Repeat::Interval`.
    every_min: Option<i64>,
    window_from: Option<i64>,
    window_until: Option<i64>,
    window_days: Option<String>,
}

/// Checks an "every N minutes" window from the form.
fn input_window(input: &ReminderInput) -> Result<Window, String> {
    let every = input.every_min.unwrap_or(30);
    let from = input.window_from.unwrap_or(7 * 60);
    let until = input.window_until.unwrap_or(17 * 60);
    if !(5..=240).contains(&every) {
        return Err("Pick an interval between 5 minutes and 4 hours".into());
    }
    if !(0..24 * 60).contains(&from) || !(0..24 * 60).contains(&until) || from >= until {
        return Err("The end time must be after the start time".into());
    }
    Ok(Window { every, from, until, weekdays_only: input.window_days.as_deref() != Some("daily") })
}

/// Creates or updates a reminder. A todo task's linked deadline only takes the lead time
/// here: its title and time belong to the task.
#[tauri::command]
pub fn reminders_save(app: AppHandle, db: State<'_, Db>, input: ReminderInput) -> ListResult {
    let lead = input.lead_min.clamp(0, 24 * 60);
    let lead_ms = lead * 60_000;
    let now = now_ms();
    {
        let conn = db.conn();
        let existing: Option<(Option<i64>, i64)> = match input.id {
            Some(id) => conn
                .query_row("SELECT task_id, at FROM reminders WHERE id = ?1", [id], |r| Ok((r.get(0)?, r.get(1)?)))
                .optional()
                .map_err(|e| e.to_string())?,
            None => None,
        };
        if input.id.is_some() && existing.is_none() {
            return Err("Reminder not found".into());
        }
        // If the lead time puts the alert in the past while the event is still ahead,
        // alert right away instead of counting it as missed.
        let alert_now = |at: i64| (at - lead_ms <= now && at > now).then_some(now);
        let result = match (input.id, existing) {
            (Some(id), Some((Some(_task), at))) => conn.execute(
                "UPDATE reminders SET lead_min = ?1, snoozed_until = ?2,
                   fired = CASE WHEN ?2 IS NULL THEN fired ELSE 0 END
                 WHERE id = ?3",
                params![lead, alert_now(at), id],
            ),
            (id, _) => {
                let title = input.title.trim();
                if title.is_empty() {
                    return Err("Title cannot be empty".into());
                }
                let title: String = title.chars().take(500).collect();
                let window = if input.repeat == Repeat::Interval { Some(input_window(&input)?) } else { None };
                let (at, lead, snoozed) = match window {
                    // An interval reminder starts at its next slot; it has no lead time.
                    Some(w) => (w.next_slot(now).ok_or("This window has no time left in the next two weeks")?, 0, None),
                    None => {
                        let mut at = input.at;
                        if input.repeat == Repeat::None && at <= now {
                            return Err("Pick a time in the future".into());
                        }
                        if at - lead_ms <= now {
                            at = next_occurrence(at, input.repeat, lead_ms, now).unwrap_or(at);
                        }
                        (at, lead, alert_now(at))
                    }
                };
                let (every, from, until, days) = match window {
                    Some(w) => (Some(w.every), Some(w.from), Some(w.until), Some(if w.weekdays_only { "weekdays" } else { "daily" })),
                    None => (None, None, None, None),
                };
                match id {
                    Some(id) => conn.execute(
                        "UPDATE reminders SET title = ?1, kind = ?2, at = ?3, repeat = ?4, lead_min = ?5,
                           snoozed_until = ?6, fired = 0,
                           every_min = ?8, window_from = ?9, window_until = ?10, window_days = ?11
                         WHERE id = ?7",
                        params![title, input.kind.as_str(), at, input.repeat.as_str(), lead, snoozed, id, every, from, until, days],
                    ),
                    None => conn.execute(
                        "INSERT INTO reminders (title, kind, at, repeat, lead_min, snoozed_until, created_at,
                           every_min, window_from, window_until, window_days)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                        params![title, input.kind.as_str(), at, input.repeat.as_str(), lead, snoozed, now, every, from, until, days],
                    ),
                }
            }
        };
        result.map_err(|e| e.to_string())?;
    }
    changed(&app);
    list(&db.conn()).map_err(|e| e.to_string())
}

/// Deletes a reminder. A linked deadline goes away by removing the task's due time.
#[tauri::command]
pub fn reminders_delete(app: AppHandle, db: State<'_, Db>, id: i64) -> ListResult {
    {
        let conn = db.conn();
        let linked: Option<Option<i64>> = conn
            .query_row("SELECT task_id FROM reminders WHERE id = ?1", [id], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        if matches!(linked, Some(Some(_))) {
            return Err("Remove the due time in Todo to delete this reminder".into());
        }
        conn.execute("DELETE FROM reminders WHERE id = ?1", [id]).map_err(|e| e.to_string())?;
    }
    changed(&app);
    list(&db.conn()).map_err(|e| e.to_string())
}

/// Fires the reminder's alert again `minutes` from now.
#[tauri::command]
pub fn reminders_snooze(app: AppHandle, db: State<'_, Db>, id: i64, minutes: i64) -> ListResult {
    let until = now_ms() + minutes.clamp(1, 24 * 60) * 60_000;
    db.conn()
        .execute("UPDATE reminders SET snoozed_until = ?1, fired = 0 WHERE id = ?2", params![until, id])
        .map_err(|e| e.to_string())?;
    changed(&app);
    list(&db.conn()).map_err(|e| e.to_string())
}
