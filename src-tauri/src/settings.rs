//! The Settings screen. Focus timer settings are saved by `focus_update_settings`
//! (focus.rs); everything else is here. Values live in the `settings` table.

use std::path::{Path, PathBuf};

use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt;

use crate::db::Db;
use crate::notes::{self, NotesFolder};
use crate::{clipboard, hooks, island, reminders, tray};

const HIDE_IN_FULLSCREEN: &str = "island.hide_in_fullscreen";
const POSITION: &str = "island.position";
pub const NOTES_FOLDER: &str = "notes.folder";
const NAME: &str = "profile.name";
const BIRTHDAY: &str = "profile.birthday";
const RAMPUNG_ENABLED: &str = "rampung.enabled";
const RAMPUNG_TIME: &str = "rampung.time";
const RAMPUNG_DONE: &str = "rampung.done_date";
const CAPTURE_HOTKEY: &str = "capture.hotkey";
const MEDIA_IN_PILL: &str = "media.in_pill";
const DEFAULT_HOTKEY: &str = "Win+Shift+K";

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    autostart: bool,
    hide_in_fullscreen: bool,
    /// "corner" (bottom-right) or "top" (the notch).
    island_position: String,
    reminder_lead_min: i64,
    notes_folder: String,
    /// False while the notes folder is the default one.
    notes_folder_custom: bool,
    version: String,
    /// Shown in the greeting; empty when not set.
    name: String,
    /// Local "YYYY-MM-DD", or empty. Used for the birthday and weton greetings.
    birthday: String,
    rampung_enabled: bool,
    /// Local "HH:MM" on weekdays.
    rampung_time: String,
    /// The local date ("YYYY-MM-DD") the Rampung ritual was last finished.
    rampung_done_date: String,
    /// "Win+Shift+K" and the like; empty when quick capture has no hotkey.
    capture_hotkey: String,
    /// False when Windows refused the hotkey (another app holds it).
    capture_hotkey_ok: bool,
    clipboard_enabled: bool,
    clipboard_paused: bool,
    /// Show what is playing in the pill when nothing else needs it.
    media_in_pill: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patch {
    autostart: Option<bool>,
    hide_in_fullscreen: Option<bool>,
    island_position: Option<String>,
    reminder_lead_min: Option<i64>,
    name: Option<String>,
    birthday: Option<String>,
    rampung_enabled: Option<bool>,
    rampung_time: Option<String>,
    rampung_done_date: Option<String>,
    capture_hotkey: Option<String>,
    clipboard_enabled: Option<bool>,
    clipboard_paused: Option<bool>,
    media_in_pill: Option<bool>,
}

/// The quick-capture hotkey, Win+Shift+K unless changed; empty when turned off.
pub fn capture_hotkey(db: &Db) -> String {
    match db.setting(CAPTURE_HOTKEY) {
        Some(value) if value == "off" => String::new(),
        Some(value) => value,
        None => DEFAULT_HOTKEY.into(),
    }
}

pub fn hide_in_fullscreen(db: &Db) -> bool {
    db.setting(HIDE_IN_FULLSCREEN).as_deref() != Some("false")
}

pub fn island_position(db: &Db) -> island::Position {
    match db.setting(POSITION).as_deref() {
        Some("top") => island::Position::Top,
        _ => island::Position::Corner,
    }
}

fn position_name(position: island::Position) -> &'static str {
    match position {
        island::Position::Top => "top",
        island::Position::Corner => "corner",
    }
}

/// "YYYY-MM-DD" with a real month and day, or empty.
fn valid_date(value: &str) -> bool {
    let parts: Vec<&str> = value.split('-').collect();
    match parts.as_slice() {
        [y, m, d] if y.len() == 4 && m.len() == 2 && d.len() == 2 => {
            let (Ok(y), Ok(m), Ok(d)) = (y.parse::<i32>(), m.parse::<u32>(), d.parse::<u32>()) else { return false };
            let days = [31, if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
            (1900..=2200).contains(&y) && (1..=12).contains(&m) && d >= 1 && d <= days[m as usize - 1]
        }
        _ => false,
    }
}

/// "HH:MM", 24-hour.
fn valid_time(value: &str) -> bool {
    match value.split_once(':') {
        Some((h, m)) if h.len() == 2 && m.len() == 2 => {
            matches!((h.parse::<u32>(), m.parse::<u32>()), (Ok(h), Ok(m)) if h < 24 && m < 60)
        }
        _ => false,
    }
}

fn put(db: &Db, key: &str, value: Option<&str>) -> Result<(), String> {
    let conn = db.conn();
    match value {
        Some(value) => conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        ),
        None => conn.execute("DELETE FROM settings WHERE key = ?1", [key]),
    }
    .map(drop)
    .map_err(|e| e.to_string())
}

fn current(app: &AppHandle) -> AppSettings {
    let db = app.state::<Db>();
    let folder = app.state::<NotesFolder>().get();
    let reminder_lead_min = reminders::default_lead_min(&db.conn());
    AppSettings {
        autostart: app.autolaunch().is_enabled().unwrap_or(false),
        hide_in_fullscreen: hide_in_fullscreen(&db),
        island_position: position_name(island_position(&db)).into(),
        reminder_lead_min,
        notes_folder: folder.display().to_string(),
        notes_folder_custom: db.setting(NOTES_FOLDER).is_some_and(|f| !f.trim().is_empty()),
        version: app.package_info().version.to_string(),
        name: db.setting(NAME).unwrap_or_default(),
        birthday: db.setting(BIRTHDAY).filter(|b| valid_date(b)).unwrap_or_default(),
        rampung_enabled: db.setting(RAMPUNG_ENABLED).as_deref() != Some("false"),
        rampung_time: db.setting(RAMPUNG_TIME).filter(|t| valid_time(t)).unwrap_or_else(|| "17:00".into()),
        rampung_done_date: db.setting(RAMPUNG_DONE).unwrap_or_default(),
        capture_hotkey: capture_hotkey(&db),
        capture_hotkey_ok: hooks::hotkey_ok(),
        clipboard_enabled: db.setting(clipboard::ENABLED).as_deref() != Some("false"),
        clipboard_paused: db.setting(clipboard::PAUSED).as_deref() == Some("true"),
        media_in_pill: db.setting(MEDIA_IN_PILL).as_deref() != Some("false"),
    }
}

/// Tells every view that settings changed (the tray can change Start with Windows too).
pub fn changed(app: &AppHandle) -> AppSettings {
    let settings = current(app);
    let _ = app.emit("settings://changed", settings.clone());
    settings
}

#[tauri::command]
pub fn settings_get(app: AppHandle) -> AppSettings {
    current(&app)
}

#[tauri::command]
pub fn settings_update(app: AppHandle, db: State<'_, Db>, patch: Patch) -> Result<AppSettings, String> {
    if let Some(enabled) = patch.autostart {
        let manager = app.autolaunch();
        let result = if enabled { manager.enable() } else { manager.disable() };
        tray::sync_autostart(&app);
        result.map_err(|_| "Windows did not accept the change to Start with Windows".to_string())?;
    }
    if let Some(hide) = patch.hide_in_fullscreen {
        put(&db, HIDE_IN_FULLSCREEN, Some(if hide { "true" } else { "false" }))?;
        island::set_hide_in_fullscreen(hide);
        hooks::recheck();
    }
    if let Some(name) = patch.island_position {
        let position = match name.as_str() {
            "top" => island::Position::Top,
            "corner" => island::Position::Corner,
            _ => return Err("Unknown position".into()),
        };
        put(&db, POSITION, Some(position_name(position)))?;
        island::set_position(position);
    }
    if let Some(lead) = patch.reminder_lead_min {
        put(&db, reminders::DEFAULT_LEAD_KEY, Some(&lead.clamp(0, 24 * 60).to_string()))?;
    }
    if let Some(name) = patch.name {
        let name: String = name.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(40).collect();
        put(&db, NAME, (!name.is_empty()).then_some(name.as_str()))?;
    }
    if let Some(birthday) = patch.birthday {
        if !birthday.is_empty() && !valid_date(&birthday) {
            return Err("Pick a valid birthday".into());
        }
        put(&db, BIRTHDAY, (!birthday.is_empty()).then_some(birthday.as_str()))?;
    }
    if let Some(enabled) = patch.rampung_enabled {
        put(&db, RAMPUNG_ENABLED, Some(if enabled { "true" } else { "false" }))?;
    }
    if let Some(time) = patch.rampung_time {
        if !valid_time(&time) {
            return Err("Pick a valid time".into());
        }
        put(&db, RAMPUNG_TIME, Some(&time))?;
    }
    if let Some(date) = patch.rampung_done_date {
        if !valid_date(&date) {
            return Err("Invalid date".into());
        }
        put(&db, RAMPUNG_DONE, Some(&date))?;
    }
    if let Some(spec) = patch.capture_hotkey {
        if !spec.is_empty() && hooks::parse_hotkey(&spec).is_none() {
            return Err("Unknown shortcut".into());
        }
        let result = hooks::set_hotkey(&spec);
        // Kept even when Windows refuses it, so Settings can say which shortcut is taken.
        put(&db, CAPTURE_HOTKEY, Some(if spec.is_empty() { "off" } else { spec.as_str() }))?;
        if let Err(message) = result {
            changed(&app);
            return Err(message);
        }
    }
    if let Some(enabled) = patch.clipboard_enabled {
        put(&db, clipboard::ENABLED, Some(if enabled { "true" } else { "false" }))?;
        if !enabled {
            clipboard::clear_all(&app)?;
        }
    }
    if let Some(paused) = patch.clipboard_paused {
        put(&db, clipboard::PAUSED, Some(if paused { "true" } else { "false" }))?;
    }
    clipboard::apply_recording(&db);
    if let Some(show) = patch.media_in_pill {
        put(&db, MEDIA_IN_PILL, Some(if show { "true" } else { "false" }))?;
    }
    Ok(changed(&app))
}

/// Shows the system folder picker; on OK, notes are read from and saved to that folder.
/// Existing notes are not moved. Returns None when the picker was cancelled.
#[tauri::command]
pub async fn settings_choose_notes_folder(app: AppHandle) -> Result<Option<AppSettings>, String> {
    let start = app.state::<NotesFolder>().get();
    let owner = island::handle();
    let picked = tauri::async_runtime::spawn_blocking(move || pick_folder(owner, &start))
        .await
        .map_err(|e| e.to_string())?;
    let Some(folder) = picked else { return Ok(None) };
    std::fs::create_dir_all(&folder).map_err(|e| format!("Karsa cannot use that folder: {e}"))?;
    put(&app.state::<Db>(), NOTES_FOLDER, Some(&folder.display().to_string()))?;
    app.state::<NotesFolder>().set(folder);
    notes::notify(&app);
    Ok(Some(changed(&app)))
}

/// Goes back to the default notes folder (`Documents\Karsa\Notes`).
#[tauri::command]
pub fn settings_reset_notes_folder(app: AppHandle, db: State<'_, Db>) -> Result<AppSettings, String> {
    put(&db, NOTES_FOLDER, None)?;
    app.state::<NotesFolder>().set(notes::resolve_folder(&app, &db));
    notes::notify(&app);
    Ok(changed(&app))
}

#[tauri::command]
pub fn settings_open_notes_folder(folder: State<'_, NotesFolder>) -> Result<(), String> {
    let folder = folder.get();
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    std::process::Command::new("explorer").arg(&folder).spawn().map(drop).map_err(|e| e.to_string())
}

/// The modern Explorer-style folder picker (`IFileOpenDialog` with `FOS_PICKFOLDERS`),
/// owned by the island so it stays above it. Runs on its own COM apartment.
fn pick_folder(owner: isize, start: &Path) -> Option<PathBuf> {
    use windows::core::{w, HSTRING, PCWSTR};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
    };
    use windows::Win32::UI::Shell::{
        FileOpenDialog, IFileOpenDialog, IShellItem, SHCreateItemFromParsingName, FOS_FORCEFILESYSTEM,
        FOS_PATHMUSTEXIST, FOS_PICKFOLDERS, SIGDN_FILESYSPATH,
    };

    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).ok().ok()?;
        let result = (|| -> windows::core::Result<PathBuf> {
            let dialog: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
            dialog.SetOptions(dialog.GetOptions()? | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST)?;
            dialog.SetTitle(w!("Choose the notes folder"))?;
            dialog.SetOkButtonLabel(w!("Use this folder"))?;
            // Start in the current folder, or its parent if it does not exist yet.
            let existing = start.ancestors().find(|p| p.is_dir()).unwrap_or(start);
            let path = HSTRING::from(existing.as_os_str());
            if let Ok(item) = SHCreateItemFromParsingName::<_, _, IShellItem>(PCWSTR(path.as_ptr()), None) {
                let _ = dialog.SetFolder(&item);
            }
            dialog.Show(Some(HWND(owner as _)))?;
            let name = dialog.GetResult()?.GetDisplayName(SIGDN_FILESYSPATH)?;
            let path = name.to_string();
            CoTaskMemFree(Some(name.0 as _));
            Ok(PathBuf::from(path?))
        })();
        CoUninitialize();
        result.ok()
    }
}
