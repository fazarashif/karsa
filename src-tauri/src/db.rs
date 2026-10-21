//! The SQLite database shared by every feature, and its schema migrations.

use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension};

/// The todo list that always exists and cannot be deleted.
pub const INBOX_ID: i64 = 1;

pub struct Db {
    conn: Mutex<Connection>,
}

pub fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

impl Db {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA secure_delete = ON;",
        )?;
        migrate(&conn)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn setting(&self, key: &str) -> Option<String> {
        self.conn()
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0))
            .optional()
            .ok()
            .flatten()
    }
}

/// Each step runs once, in order, tracked by `PRAGMA user_version`.
fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if version < 1 {
        conn.execute_batch(&format!(
            "BEGIN;
             CREATE TABLE lists (
               id INTEGER PRIMARY KEY,
               name TEXT NOT NULL,
               position INTEGER NOT NULL,
               created_at INTEGER NOT NULL
             );
             CREATE TABLE tasks (
               id INTEGER PRIMARY KEY,
               list_id INTEGER NOT NULL REFERENCES lists(id) ON DELETE CASCADE,
               parent_id INTEGER REFERENCES tasks(id) ON DELETE CASCADE,
               title TEXT NOT NULL,
               done INTEGER NOT NULL DEFAULT 0,
               due_at INTEGER,
               due_has_time INTEGER NOT NULL DEFAULT 0,
               position INTEGER NOT NULL,
               created_at INTEGER NOT NULL,
               completed_at INTEGER
             );
             CREATE INDEX tasks_by_list ON tasks(list_id, parent_id, position);
             INSERT INTO lists (id, name, position, created_at) VALUES ({INBOX_ID}, 'Inbox', 0, {now});
             PRAGMA user_version = 1;
             COMMIT;",
            now = now_ms()
        ))?;
    }
    if version < 2 {
        conn.execute_batch(
            "BEGIN;
             CREATE TABLE focus_sessions (
               id INTEGER PRIMARY KEY,
               started_at INTEGER NOT NULL,
               ended_at INTEGER NOT NULL,
               duration_ms INTEGER NOT NULL
             );
             CREATE INDEX focus_sessions_by_end ON focus_sessions(ended_at);
             CREATE TABLE settings (
               key TEXT PRIMARY KEY,
               value TEXT NOT NULL
             );
             PRAGMA user_version = 2;
             COMMIT;",
        )?;
    }
    if version < 3 {
        // `at` is the moment of the event (a meeting's start, a deadline); the alert fires
        // `lead_min` earlier, or at `snoozed_until` when set. A reminder with `task_id` is
        // the linked deadline of a todo task and is kept in sync with it.
        conn.execute_batch(
            "BEGIN;
             CREATE TABLE reminders (
               id INTEGER PRIMARY KEY,
               title TEXT NOT NULL,
               kind TEXT NOT NULL CHECK (kind IN ('general', 'meeting', 'deadline')),
               at INTEGER NOT NULL,
               repeat TEXT NOT NULL DEFAULT 'none' CHECK (repeat IN ('none', 'daily', 'weekdays', 'weekly')),
               lead_min INTEGER NOT NULL DEFAULT 0,
               snoozed_until INTEGER,
               fired INTEGER NOT NULL DEFAULT 0,
               task_id INTEGER UNIQUE REFERENCES tasks(id) ON DELETE CASCADE,
               created_at INTEGER NOT NULL
             );
             CREATE INDEX reminders_pending ON reminders(fired, at);
             PRAGMA user_version = 3;
             COMMIT;",
        )?;
    }
    if version < 4 {
        // A task added from a meeting note's action items links back to that note
        // (its path relative to the notes folder).
        conn.execute_batch(
            "BEGIN;
             ALTER TABLE tasks ADD COLUMN note_path TEXT;
             PRAGMA user_version = 4;
             COMMIT;",
        )?;
    }
    if version < 5 {
        // Clipboard history: the text is encrypted with the user's Windows account (DPAPI)
        // and kept for 24 hours. The shelf keeps links to files, not copies.
        conn.execute_batch(
            "BEGIN;
             CREATE TABLE clipboard (
               id INTEGER PRIMARY KEY,
               data BLOB NOT NULL,
               copied_at INTEGER NOT NULL
             );
             CREATE INDEX clipboard_by_time ON clipboard(copied_at);
             CREATE TABLE shelf (
               id INTEGER PRIMARY KEY,
               path TEXT NOT NULL UNIQUE,
               added_at INTEGER NOT NULL
             );
             PRAGMA user_version = 5;
             COMMIT;",
        )?;
    }
    if version < 6 {
        // "Every N minutes" reminders (drink water, stretch): the interval, and the daily
        // window in minutes after local midnight, on weekdays or every day. SQLite cannot
        // change a CHECK constraint in place, so the table is rebuilt with the new repeat.
        conn.execute_batch(
            "BEGIN;
             CREATE TABLE reminders_v6 (
               id INTEGER PRIMARY KEY,
               title TEXT NOT NULL,
               kind TEXT NOT NULL CHECK (kind IN ('general', 'meeting', 'deadline')),
               at INTEGER NOT NULL,
               repeat TEXT NOT NULL DEFAULT 'none'
                 CHECK (repeat IN ('none', 'daily', 'weekdays', 'weekly', 'interval')),
               lead_min INTEGER NOT NULL DEFAULT 0,
               snoozed_until INTEGER,
               fired INTEGER NOT NULL DEFAULT 0,
               task_id INTEGER UNIQUE REFERENCES tasks(id) ON DELETE CASCADE,
               created_at INTEGER NOT NULL,
               every_min INTEGER,
               window_from INTEGER,
               window_until INTEGER,
               window_days TEXT CHECK (window_days IN ('weekdays', 'daily'))
             );
             INSERT INTO reminders_v6 (id, title, kind, at, repeat, lead_min, snoozed_until, fired, task_id, created_at)
               SELECT id, title, kind, at, repeat, lead_min, snoozed_until, fired, task_id, created_at FROM reminders;
             DROP TABLE reminders;
             ALTER TABLE reminders_v6 RENAME TO reminders;
             CREATE INDEX reminders_pending ON reminders(fired, at);
             PRAGMA user_version = 6;
             COMMIT;",
        )?;
    }
    Ok(())
}
