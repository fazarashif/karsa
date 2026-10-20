//! Todo storage: lists, tasks and one level of subtasks, in SQLite.
//!
//! Every command returns the whole snapshot. The data is small (hundreds of rows at
//! most), and replacing the UI's copy wholesale keeps the two sides from drifting. Every
//! change is also broadcast to all windows (the island and any open note windows).

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::db::{now_ms, Db, INBOX_ID};
use crate::reminders;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct List {
    id: i64,
    name: String,
    position: i64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    id: i64,
    list_id: i64,
    parent_id: Option<i64>,
    title: String,
    done: bool,
    /// Unix ms. For a date without a time, local midnight of that day.
    due_at: Option<i64>,
    due_has_time: bool,
    position: i64,
    completed_at: Option<i64>,
    /// The meeting note this task came from, relative to the notes folder.
    note_path: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct Snapshot {
    lists: Vec<List>,
    tasks: Vec<Task>,
}

type CmdResult = Result<Snapshot, String>;

fn clean_title(title: &str) -> Result<String, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("Title cannot be empty".into());
    }
    Ok(title.chars().take(500).collect())
}

fn snapshot(conn: &Connection) -> rusqlite::Result<Snapshot> {
    let lists = conn
        .prepare("SELECT id, name, position FROM lists ORDER BY position, id")?
        .query_map([], |r| Ok(List { id: r.get(0)?, name: r.get(1)?, position: r.get(2)? }))?
        .collect::<rusqlite::Result<_>>()?;
    let tasks = conn
        .prepare(
            "SELECT id, list_id, parent_id, title, done, due_at, due_has_time, position, completed_at, note_path
             FROM tasks ORDER BY position, id",
        )?
        .query_map([], |r| {
            Ok(Task {
                id: r.get(0)?,
                list_id: r.get(1)?,
                parent_id: r.get(2)?,
                title: r.get(3)?,
                done: r.get(4)?,
                due_at: r.get(5)?,
                due_has_time: r.get(6)?,
                position: r.get(7)?,
                completed_at: r.get(8)?,
                note_path: r.get(9)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(Snapshot { lists, tasks })
}

/// Runs a change in a transaction, broadcasts the new snapshot, and returns it.
fn change(app: &AppHandle, db: &Db, f: impl FnOnce(&Connection) -> Result<(), String>) -> CmdResult {
    let snapshot = {
        let mut conn = db.conn();
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        f(&tx)?;
        tx.commit().map_err(|e| e.to_string())?;
        snapshot(&conn).map_err(|e| e.to_string())?
    };
    let _ = app.emit("todo://changed", &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub fn todo_snapshot(store: State<'_, Db>) -> CmdResult {
    snapshot(&store.conn()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn todo_create_list(app: AppHandle, store: State<'_, Db>, name: String) -> CmdResult {
    let name = clean_title(&name)?;
    change(&app, &store, |c| {
        c.execute(
            "INSERT INTO lists (name, position, created_at)
             VALUES (?1, (SELECT IFNULL(MAX(position), 0) + 1 FROM lists), ?2)",
            params![name, now_ms()],
        )
        .map(drop)
        .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn todo_rename_list(app: AppHandle, store: State<'_, Db>, id: i64, name: String) -> CmdResult {
    let name = clean_title(&name)?;
    change(&app, &store, |c| {
        c.execute("UPDATE lists SET name = ?1 WHERE id = ?2", params![name, id])
            .map(drop)
            .map_err(|e| e.to_string())
    })
}

/// Deletes a list and every task in it (their deadline reminders go with them). The
/// Inbox cannot be deleted.
#[tauri::command]
pub fn todo_delete_list(app: AppHandle, store: State<'_, Db>, id: i64) -> CmdResult {
    if id == INBOX_ID {
        return Err("The Inbox cannot be deleted".into());
    }
    let snapshot =
        change(&app, &store, |c| c.execute("DELETE FROM lists WHERE id = ?1", [id]).map(drop).map_err(|e| e.to_string()))?;
    reminders::changed(&app);
    Ok(snapshot)
}

/// Swaps a list with its neighbour; `direction` is -1 (up) or 1 (down).
#[tauri::command]
pub fn todo_move_list(app: AppHandle, store: State<'_, Db>, id: i64, direction: i64) -> CmdResult {
    change(&app, &store, |c| {
        let ids: Vec<i64> = c
            .prepare("SELECT id FROM lists ORDER BY position, id")
            .and_then(|mut s| s.query_map([], |r| r.get(0))?.collect())
            .map_err(|e| e.to_string())?;
        let Some(from) = ids.iter().position(|&x| x == id) else { return Ok(()) };
        let to = from as i64 + direction.signum();
        if to < 0 || to >= ids.len() as i64 {
            return Ok(());
        }
        let mut ids = ids;
        ids.swap(from, to as usize);
        for (position, list) in ids.iter().enumerate() {
            c.execute("UPDATE lists SET position = ?1 WHERE id = ?2", params![position as i64, list])
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    })
}

/// New tasks go to the top of their list; new subtasks go to the bottom of their parent.
/// `note_path` links a task added from a meeting note's action items back to that note.
#[tauri::command]
pub fn todo_create_task(
    app: AppHandle,
    store: State<'_, Db>,
    list_id: i64,
    title: String,
    parent_id: Option<i64>,
    note_path: Option<String>,
) -> CmdResult {
    let title = clean_title(&title)?;
    change(&app, &store, |c| {
        let (list_id, position) = match parent_id {
            Some(parent) => {
                let list: i64 = c
                    .query_row("SELECT list_id FROM tasks WHERE id = ?1 AND parent_id IS NULL", [parent], |r| r.get(0))
                    .map_err(|_| "Subtasks can only be added to a top-level task".to_string())?;
                let next: i64 = c
                    .query_row("SELECT IFNULL(MAX(position), 0) + 1 FROM tasks WHERE parent_id = ?1", [parent], |r| r.get(0))
                    .map_err(|e| e.to_string())?;
                (list, next)
            }
            None => {
                let first: i64 = c
                    .query_row(
                        "SELECT IFNULL(MIN(position), 1) - 1 FROM tasks WHERE list_id = ?1 AND parent_id IS NULL",
                        [list_id],
                        |r| r.get(0),
                    )
                    .map_err(|e| e.to_string())?;
                (list_id, first)
            }
        };
        c.execute(
            "INSERT INTO tasks (list_id, parent_id, title, position, created_at, note_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![list_id, parent_id, title, position, now_ms(), note_path],
        )
        .map(drop)
        .map_err(|e| e.to_string())
    })
}

/// Updates any of title, done, or list. Moving a task moves its subtasks with it.
/// Completing a task cancels its deadline reminder.
#[tauri::command]
pub fn todo_update_task(
    app: AppHandle,
    store: State<'_, Db>,
    id: i64,
    title: Option<String>,
    done: Option<bool>,
    list_id: Option<i64>,
) -> CmdResult {
    let title = title.as_deref().map(clean_title).transpose()?;
    let snapshot = change(&app, &store, |c| {
        let exists = c
            .query_row("SELECT 1 FROM tasks WHERE id = ?1", [id], |_| Ok(()))
            .optional()
            .map_err(|e| e.to_string())?;
        if exists.is_none() {
            return Err("Task not found".into());
        }
        if let Some(title) = title {
            c.execute("UPDATE tasks SET title = ?1 WHERE id = ?2", params![title, id]).map_err(|e| e.to_string())?;
        }
        if let Some(done) = done {
            let completed_at = done.then(now_ms);
            c.execute("UPDATE tasks SET done = ?1, completed_at = ?2 WHERE id = ?3", params![done, completed_at, id])
                .map_err(|e| e.to_string())?;
        }
        if let Some(list_id) = list_id {
            c.execute(
                "UPDATE tasks SET list_id = ?1 WHERE (id = ?2 AND parent_id IS NULL) OR parent_id = ?2",
                params![list_id, id],
            )
            .map_err(|e| e.to_string())?;
        }
        reminders::sync_task(c, id).map_err(|e| e.to_string())
    })?;
    reminders::changed(&app);
    Ok(snapshot)
}

/// Sets or clears (`due_at: None`) the due date. A due date with a time gets a linked
/// deadline reminder.
#[tauri::command]
pub fn todo_set_due(app: AppHandle, store: State<'_, Db>, id: i64, due_at: Option<i64>, has_time: bool) -> CmdResult {
    let snapshot = change(&app, &store, |c| {
        c.execute(
            "UPDATE tasks SET due_at = ?1, due_has_time = ?2 WHERE id = ?3",
            params![due_at, due_at.is_some() && has_time, id],
        )
        .map_err(|e| e.to_string())?;
        reminders::sync_task(c, id).map_err(|e| e.to_string())
    })?;
    reminders::changed(&app);
    Ok(snapshot)
}

/// Deletes a task and its subtasks (and its deadline reminder).
#[tauri::command]
pub fn todo_delete_task(app: AppHandle, store: State<'_, Db>, id: i64) -> CmdResult {
    let snapshot =
        change(&app, &store, |c| c.execute("DELETE FROM tasks WHERE id = ?1", [id]).map(drop).map_err(|e| e.to_string()))?;
    reminders::changed(&app);
    Ok(snapshot)
}
