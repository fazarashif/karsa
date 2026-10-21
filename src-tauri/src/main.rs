// Release builds run without a console window.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod db;
mod focus;
mod hooks;
mod island;
mod localtime;
mod media;
mod notes;
mod reminders;
mod settings;
mod shelf;
mod todo;
mod tray;

use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;

use hooks::HookEvent;
use tauri::{AppHandle, Emitter, Manager, WindowEvent};

pub const ISLAND: &str = "island";

#[tauri::command]
fn ui_ready(app: AppHandle, engine: tauri::State<'_, Arc<reminders::ReminderEngine>>) {
    island::ui_ready();
    engine.start();
    accept_file_drops(&app);
}

/// Puts the shelf's drop target on the island's windows now and once more shortly after,
/// since WebView2 may set up its own (which refuses outside drops) once the island shows.
fn accept_file_drops(app: &AppHandle) {
    shelf::register_drop_target(app);
    let app = app.clone();
    let _ = std::thread::Builder::new().name("karsa-drop".into()).spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        shelf::register_drop_target(&app);
    });
}

#[tauri::command]
fn set_hit_region(width: f64, height: f64) {
    island::set_box_size(width, height);
}

#[tauri::command]
fn set_interactive(active: bool, restore_focus: bool) {
    island::set_interactive(active, restore_focus);
}

#[tauri::command]
fn uses_12_hour_clock() -> bool {
    island::uses_12_hour_clock()
}

/// Reacts to hook notifications off the hook thread, so the hooks never wait on the UI.
fn forward_hook_events(app: AppHandle, events: Receiver<HookEvent>) {
    let mut fullscreen = false;
    for event in events {
        match event {
            HookEvent::ForegroundChanged => {
                if island::foreground_changed() {
                    let _ = app.emit_to(ISLAND, "island://blur", ());
                }
                let active = island::hides_in_fullscreen() && island::fullscreen_app_active();
                if active != fullscreen {
                    fullscreen = active;
                    island::set_hidden(active);
                    let _ = app.emit_to(ISLAND, "island://fullscreen", active);
                    if !active {
                        accept_file_drops(&app);
                    }
                }
            }
            HookEvent::ClockChanged => {
                app.state::<Arc<focus::Focus>>().notify();
                app.state::<Arc<reminders::ReminderEngine>>().poke();
            }
            // While a fullscreen app runs the island is hidden, so quick capture waits.
            HookEvent::Hotkey if !fullscreen => {
                let _ = app.emit_to(ISLAND, "island://capture", ());
            }
            HookEvent::Hotkey => {}
            HookEvent::Copied(text) => clipboard::record(&app, text),
            HookEvent::Housekeeping => clipboard::purge(&app),
        }
    }
}

/// Start with Windows is on by default, but only for installed copies: a build run
/// from Cargo's target/ folder must not register itself to run at sign-in.
#[cfg(not(debug_assertions))]
fn enable_autostart_on_first_run(app: &AppHandle) {
    use tauri_plugin_autostart::ManagerExt;
    let local_build = std::env::current_exe()
        .map(|exe| exe.components().any(|part| part.as_os_str() == "target"))
        .unwrap_or(true);
    if local_build {
        return;
    }
    let Ok(dir) = app.path().app_config_dir() else { return };
    let marker = dir.join("autostart-initialized");
    if marker.exists() {
        return;
    }
    let _ = app.autolaunch().enable();
    let _ = std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(&marker, b""));
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app.emit_to(ISLAND, "island://open", ());
        }))
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .invoke_handler(tauri::generate_handler![
            ui_ready,
            set_hit_region,
            set_interactive,
            uses_12_hour_clock,
            todo::todo_snapshot,
            todo::todo_create_list,
            todo::todo_rename_list,
            todo::todo_delete_list,
            todo::todo_move_list,
            todo::todo_create_task,
            todo::todo_update_task,
            todo::todo_set_due,
            todo::todo_delete_task,
            focus::focus_state,
            focus::focus_start,
            focus::focus_pause,
            focus::focus_resume,
            focus::focus_skip,
            focus::focus_stop,
            focus::focus_stats,
            focus::focus_history,
            focus::focus_update_settings,
            reminders::reminders_list,
            reminders::reminders_save,
            reminders::reminders_delete,
            reminders::reminders_snooze,
            notes::notes_list,
            notes::notes_read,
            notes::notes_save,
            notes::notes_delete,
            notes::notes_reveal,
            notes::notes_open_window,
            settings::settings_get,
            settings::settings_update,
            settings::settings_choose_notes_folder,
            settings::settings_reset_notes_folder,
            settings::settings_open_notes_folder,
            clipboard::clipboard_list,
            clipboard::clipboard_copy,
            clipboard::clipboard_delete,
            clipboard::clipboard_clear,
            shelf::shelf_list,
            shelf::shelf_add,
            shelf::shelf_paste,
            shelf::shelf_pick,
            shelf::shelf_drop_status,
            shelf::shelf_remove,
            shelf::shelf_clear,
            shelf::shelf_open,
            shelf::shelf_reveal,
            shelf::shelf_copy,
            shelf::shelf_drag,
            media::media_state,
            media::media_control,
        ])
        .on_window_event(|_window, event| {
            if let WindowEvent::ScaleFactorChanged { .. } = event {
                island::place();
            }
        })
        .setup(|app| {
            // KARSA_DATA_DIR points a test or development instance at its own database.
            let data_dir = match std::env::var_os("KARSA_DATA_DIR") {
                Some(dir) => std::path::PathBuf::from(dir),
                None => app.path().app_data_dir()?,
            };
            std::fs::create_dir_all(&data_dir)?;
            let db = db::Db::open(&data_dir.join("karsa.db"))?;
            let focus = Arc::new(focus::Focus::new(focus::Settings::load(&db)));
            island::set_hide_in_fullscreen(settings::hide_in_fullscreen(&db));
            island::set_position(settings::island_position(&db));
            // Both take effect when the hook thread starts below.
            let _ = hooks::set_hotkey(&settings::capture_hotkey(&db));
            clipboard::apply_recording(&db);
            app.manage(notes::NotesFolder::new(notes::resolve_folder(app.handle(), &db)));
            app.manage(db);
            clipboard::purge(app.handle());
            app.manage(focus.clone());
            focus::spawn_timer(app.handle().clone(), focus)?;
            let reminder_engine = Arc::new(reminders::ReminderEngine::new());
            app.manage(reminder_engine.clone());
            reminders::spawn(app.handle().clone(), reminder_engine)?;

            let window = app
                .get_webview_window(ISLAND)
                .expect("the island window is defined in tauri.conf.json");
            island::init(&window)?;

            let (sender, receiver) = mpsc::channel();
            hooks::start(sender)?;
            let handle = app.handle().clone();
            std::thread::Builder::new()
                .name("karsa-events".into())
                .spawn(move || forward_hook_events(handle, receiver))?;

            media::spawn(app.handle().clone())?;
            tray::create(app.handle())?;
            #[cfg(not(debug_assertions))]
            enable_autostart_on_first_run(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Karsa");
}
