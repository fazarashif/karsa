//! Clipboard history: text copied in the last 24 hours, newest first. The hook thread
//! (hooks.rs) passes on each copy that the copying app did not mark as private; this module
//! stores it encrypted with the user's Windows account (DPAPI), so the database alone does
//! not reveal it, and deletes it after 24 hours.

use std::ptr::{null, null_mut};

use rusqlite::params;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
};

use crate::db::{now_ms, Db};
use crate::hooks;

pub const ENABLED: &str = "clipboard.enabled";
pub const PAUSED: &str = "clipboard.paused";

const KEEP_MS: i64 = 24 * 60 * 60 * 1000;
const MAX_ITEMS: i64 = 200;
/// Mixed into the encryption, so other apps calling DPAPI for this user cannot simply
/// decrypt the column without knowing it.
const ENTROPY: &[u8] = b"karsa clipboard history";

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClipItem {
    id: i64,
    text: String,
    copied_at: i64,
}

fn blob(data: &[u8]) -> CRYPT_INTEGER_BLOB {
    CRYPT_INTEGER_BLOB { cbData: data.len() as u32, pbData: data.as_ptr() as *mut u8 }
}

fn protect(text: &str) -> Option<Vec<u8>> {
    let input = blob(text.as_bytes());
    let entropy = blob(ENTROPY);
    let mut out = CRYPT_INTEGER_BLOB { cbData: 0, pbData: null_mut() };
    unsafe {
        if CryptProtectData(&input, null(), &entropy, null(), null(), CRYPTPROTECT_UI_FORBIDDEN, &mut out) == 0 {
            return None;
        }
        let bytes = std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec();
        LocalFree(out.pbData as _);
        Some(bytes)
    }
}

fn unprotect(data: &[u8]) -> Option<String> {
    let input = blob(data);
    let entropy = blob(ENTROPY);
    let mut out = CRYPT_INTEGER_BLOB { cbData: 0, pbData: null_mut() };
    unsafe {
        if CryptUnprotectData(&input, null_mut(), &entropy, null(), null(), CRYPTPROTECT_UI_FORBIDDEN, &mut out) == 0 {
            return None;
        }
        let text = String::from_utf8_lossy(std::slice::from_raw_parts(out.pbData, out.cbData as usize)).into_owned();
        LocalFree(out.pbData as _);
        Some(text)
    }
}

/// Recording is on unless turned off in Settings or paused on the Shelf.
pub fn apply_recording(db: &Db) {
    let enabled = db.setting(ENABLED).as_deref() != Some("false");
    let paused = db.setting(PAUSED).as_deref() == Some("true");
    hooks::set_recording(enabled && !paused);
}

fn notify(app: &AppHandle) {
    let _ = app.emit("clipboard://changed", ());
}

/// Stores a copy. Copying the same text as the newest item again only moves it to now.
pub fn record(app: &AppHandle, text: String) {
    let db = app.state::<Db>();
    let now = now_ms();
    {
        let conn = db.conn();
        let newest: Option<(i64, Vec<u8>)> = conn
            .query_row("SELECT id, data FROM clipboard ORDER BY copied_at DESC LIMIT 1", [], |r| Ok((r.get(0)?, r.get(1)?)))
            .ok();
        match newest {
            Some((id, data)) if unprotect(&data).as_deref() == Some(text.as_str()) => {
                let _ = conn.execute("UPDATE clipboard SET copied_at = ?1 WHERE id = ?2", params![now, id]);
            }
            _ => {
                let Some(data) = protect(&text) else { return };
                let _ = conn.execute("INSERT INTO clipboard (data, copied_at) VALUES (?1, ?2)", params![data, now]);
            }
        }
    }
    purge(app);
    notify(app);
}

/// Deletes items older than 24 hours and keeps at most MAX_ITEMS. Tells the UI only when
/// something changed, so the 15-minute housekeeping tick does not wake it for nothing.
pub fn purge(app: &AppHandle) {
    let db = app.state::<Db>();
    let conn = db.conn();
    let old = conn.execute("DELETE FROM clipboard WHERE copied_at < ?1", [now_ms() - KEEP_MS]).unwrap_or(0);
    let extra = conn
        .execute(
            "DELETE FROM clipboard WHERE id NOT IN (SELECT id FROM clipboard ORDER BY copied_at DESC LIMIT ?1)",
            [MAX_ITEMS],
        )
        .unwrap_or(0);
    drop(conn);
    if old + extra > 0 {
        notify(app);
    }
}

/// Newest first; with a query, only items containing it (case-insensitive).
#[tauri::command]
pub fn clipboard_list(db: State<'_, Db>, query: Option<String>) -> Result<Vec<ClipItem>, String> {
    let needle = query.map(|q| q.trim().to_lowercase()).filter(|q| !q.is_empty());
    let conn = db.conn();
    let mut stmt = conn
        .prepare("SELECT id, data, copied_at FROM clipboard WHERE copied_at >= ?1 ORDER BY copied_at DESC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([now_ms() - KEEP_MS], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, Vec<u8>>(1)?, r.get::<_, i64>(2)?)))
        .map_err(|e| e.to_string())?;
    Ok(rows
        .filter_map(Result::ok)
        .filter_map(|(id, data, copied_at)| unprotect(&data).map(|text| ClipItem { id, text, copied_at }))
        .filter(|item| needle.as_ref().is_none_or(|n| item.text.to_lowercase().contains(n.as_str())))
        .collect())
}

/// Puts an item back on the clipboard and moves it to the top.
#[tauri::command]
pub fn clipboard_copy(app: AppHandle, db: State<'_, Db>, id: i64) -> Result<(), String> {
    let data: Vec<u8> = db
        .conn()
        .query_row("SELECT data FROM clipboard WHERE id = ?1", [id], |r| r.get(0))
        .map_err(|_| "This item is gone".to_string())?;
    let text = unprotect(&data).ok_or("Could not read this item")?;
    hooks::set_clipboard_text(&text)?;
    let _ = db.conn().execute("UPDATE clipboard SET copied_at = ?1 WHERE id = ?2", params![now_ms(), id]);
    notify(&app);
    Ok(())
}

#[tauri::command]
pub fn clipboard_delete(app: AppHandle, db: State<'_, Db>, id: i64) -> Result<(), String> {
    db.conn().execute("DELETE FROM clipboard WHERE id = ?1", [id]).map_err(|e| e.to_string())?;
    notify(&app);
    Ok(())
}

/// Deletes the whole history (Clear, or turning the feature off in Settings).
pub fn clear_all(app: &AppHandle) -> Result<(), String> {
    let db = app.state::<Db>();
    db.conn().execute("DELETE FROM clipboard", []).map_err(|e| e.to_string())?;
    // With secure_delete on (db.rs), freed pages are zeroed; folding the WAL back in makes
    // sure the old copies there go too.
    let _ = db.conn().execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
    notify(app);
    Ok(())
}

#[tauri::command]
pub fn clipboard_clear(app: AppHandle) -> Result<(), String> {
    clear_all(&app)
}
