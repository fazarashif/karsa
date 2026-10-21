use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Wry};
use tauri_plugin_autostart::ManagerExt;

use crate::{hooks, settings, ISLAND};

// Drawn for the tray (design/assets/icon-tray-*.svg), readable on dark and light taskbars.
const TRAY_16: &[u8] = include_bytes!("../icons/tray/tray-16.png");
const TRAY_32: &[u8] = include_bytes!("../icons/tray/tray-32.png");

/// The "Start with Windows" check item, so the Settings screen can keep it in step.
struct AutostartItem(CheckMenuItem<Wry>);

/// Sets the tray's check mark to what Windows actually has registered.
pub fn sync_autostart(app: &AppHandle) {
    if let Some(item) = app.try_state::<AutostartItem>() {
        let _ = item.0.set_checked(app.autolaunch().is_enabled().unwrap_or(false));
    }
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);

    let open = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
    let rampung = MenuItem::with_id(app, "rampung", "Wrap up the day", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let autostart =
        CheckMenuItem::with_id(app, "autostart", "Start with Windows", true, autostart_enabled, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &rampung, &settings_item, &PredefinedMenuItem::separator(app)?, &autostart])?;
    // Development builds can show an alert on demand, to check the alert layout.
    #[cfg(debug_assertions)]
    menu.append(&MenuItem::with_id(app, "test-alert", "Show test alert", true, None::<&str>)?)?;
    menu.append_items(&[&PredefinedMenuItem::separator(app)?, &quit])?;
    app.manage(AutostartItem(autostart.clone()));

    // The tray draws 16 px at 100% scaling; above that the 32 px version scales down more cleanly.
    let scale = app.primary_monitor().ok().flatten().map(|m| m.scale_factor()).unwrap_or(1.0);
    let icon = Image::from_bytes(if scale > 1.0 { TRAY_32 } else { TRAY_16 })?;

    TrayIconBuilder::with_id("karsa")
        .icon(icon)
        .tooltip("Karsa")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "open" => {
                let _ = app.emit_to(ISLAND, "island://open", ());
            }
            "settings" => {
                let _ = app.emit_to(ISLAND, "island://open", "settings");
            }
            "rampung" => {
                let _ = app.emit_to(ISLAND, "island://open", "rampung");
            }
            "test-alert" => {
                let alert = serde_json::json!({ "title": "Weekly sync", "subtitle": "Meeting · starts in 5 min", "kind": "meeting" });
                let _ = app.emit_to(ISLAND, "island://alert", alert);
            }
            "autostart" => {
                // The menu toggles the check mark itself; mirror it in the registry.
                let manager = app.autolaunch();
                let wanted = autostart.is_checked().unwrap_or(false);
                let _ = if wanted { manager.enable() } else { manager.disable() };
                sync_autostart(app);
                settings::changed(app);
            }
            "quit" => {
                hooks::stop();
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                let _ = tray.app_handle().emit_to(ISLAND, "island://open", ());
            }
        })
        .build(app)?;
    Ok(())
}
