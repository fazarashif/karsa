//! The Settings screen. Focus timer settings are saved by `focus_update_settings`
//! (focus.rs); everything else is here. Values live in the `settings` table.

use std::path::{Path, PathBuf};

use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt;

use crate::db::Db;
use crate::notes::{self, NotesFolder};
use crate::{hooks, island, reminders, tray};

const HIDE_IN_FULLSCREEN: &str = "island.hide_in_fullscreen";
pub const NOTES_FOLDER: &str = "notes.folder";

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    autostart: bool,
    hide_in_fullscreen: bool,
    reminder_lead_min: i64,
    notes_folder: String,
    /// False while the notes folder is the default one.
    notes_folder_custom: bool,
    version: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patch {
    autostart: Option<bool>,
    hide_in_fullscreen: Option<bool>,
    reminder_lead_min: Option<i64>,
}

pub fn hide_in_fullscreen(db: &Db) -> bool {
    db.setting(HIDE_IN_FULLSCREEN).as_deref() != Some("false")
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
        reminder_lead_min,
        notes_folder: folder.display().to_string(),
        notes_folder_custom: db.setting(NOTES_FOLDER).is_some_and(|f| !f.trim().is_empty()),
        version: app.package_info().version.to_string(),
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
    if let Some(lead) = patch.reminder_lead_min {
        put(&db, reminders::DEFAULT_LEAD_KEY, Some(&lead.clamp(0, 24 * 60).to_string()))?;
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
