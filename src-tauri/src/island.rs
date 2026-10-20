//! The island window: placement, click-through, focus, and fullscreen detection.
//!
//! The window never resizes. It covers the largest state and sits in the bottom-right
//! corner of the primary monitor's work area. A window region limits it to the visible
//! box, so everywhere else clicks go straight to the apps underneath. Only the web
//! content animates.
//!
//! Visibility, focusability and click-through are managed here with Win32 calls, never
//! through the windowing library's setters: those re-apply every window style and call
//! `ShowWindow`, which would hide the island or steal focus.

use std::mem::{size_of, zeroed};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::{Mutex, MutexGuard};

use tauri::WebviewWindow;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Globalization::{GetLocaleInfoEx, LOCALE_STIMEFORMAT};
use windows_sys::Win32::Graphics::Gdi::{
    CreateRectRgn, GetMonitorInfoW, MonitorFromPoint, MonitorFromWindow, SetWindowRgn, HMONITOR,
    MONITORINFO, MONITOR_DEFAULTTONULL, MONITOR_DEFAULTTOPRIMARY,
};
use windows_sys::Win32::System::Threading::{AttachThreadInput, GetCurrentProcessId, GetCurrentThreadId};
use windows_sys::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows_sys::Win32::UI::Shell::{
    DefSubclassProc, SHQueryUserNotificationState, SetWindowSubclass, QUNS_PRESENTATION_MODE,
    QUNS_RUNNING_D3D_FULL_SCREEN,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetForegroundWindow, GetWindowLongPtrW, GetWindowLongW, GetWindowRect,
    GetWindowThreadProcessId, IsWindow, IsZoomed, PostMessageW, SetForegroundWindow, SetWindowLongPtrW,
    SetWindowPos, ShowWindow, GWL_EXSTYLE, GWL_STYLE, HWND_TOPMOST, MA_NOACTIVATE, SPI_SETWORKAREA,
    STYLESTRUCT, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE, SWP_NOZORDER,
    SW_HIDE, SW_SHOWNOACTIVATE, WA_INACTIVE, WM_ACTIVATE, WM_APP, WM_DISPLAYCHANGE, WM_MOUSEACTIVATE,
    WM_NCACTIVATE, WM_SETTINGCHANGE, WM_STYLECHANGING, WS_CAPTION, WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};

/// Logical size of the island window. Must match `WINDOW` in src/lib/config.ts.
pub const WINDOW_SIZE: (f64, f64) = (400.0, 540.0);
/// Gap between the island and the edges of the work area, in logical px.
const MARGIN: f64 = 12.0;
/// Extra room above and left of the box so the spring overshoot is not clipped.
const REGION_PADDING: f64 = 10.0;

/// Posted to ourselves to hand activation back to the user's app (see `WM_ACTIVATE`).
const WM_RETURN_FOREGROUND: u32 = WM_APP + 2;

static ISLAND_HWND: AtomicIsize = AtomicIsize::new(0);
/// While true the window never takes focus, so clicking it leaves the user's app focused.
static NO_ACTIVATE: AtomicBool = AtomicBool::new(true);
/// The app that had focus before the panel opened, so Esc can hand focus back.
static PREVIOUS_FOREGROUND: AtomicIsize = AtomicIsize::new(0);
/// The most recent foreground window owned by another process.
static LAST_OTHER_FOREGROUND: AtomicIsize = AtomicIsize::new(0);
static READY: AtomicBool = AtomicBool::new(false);
static HIDDEN: AtomicBool = AtomicBool::new(false);
/// Settings → "Hide during fullscreen".
static HIDE_IN_FULLSCREEN: AtomicBool = AtomicBool::new(true);

#[derive(Clone, Copy)]
struct Placement {
    width: i32,
    height: i32,
    scale: f64,
    monitor: isize,
}

static PLACEMENT: Mutex<Placement> = Mutex::new(Placement { width: 0, height: 0, scale: 1.0, monitor: 0 });
/// Logical size of the visible box, as last reported by the UI.
static BOX_SIZE: Mutex<(f64, f64)> = Mutex::new((0.0, 0.0));

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn hwnd() -> HWND {
    ISLAND_HWND.load(Ordering::Relaxed) as HWND
}

/// The island's window handle, e.g. to own a dialog so it stays above the island.
pub fn handle() -> isize {
    ISLAND_HWND.load(Ordering::Relaxed)
}

pub fn set_hide_in_fullscreen(hide: bool) {
    HIDE_IN_FULLSCREEN.store(hide, Ordering::Relaxed);
}

pub fn hides_in_fullscreen() -> bool {
    HIDE_IN_FULLSCREEN.load(Ordering::Relaxed)
}

pub fn init(window: &WebviewWindow) -> tauri::Result<()> {
    ISLAND_HWND.store(window.hwnd()?.0 as isize, Ordering::Relaxed);
    // The app in front when Karsa starts, so activation can always be handed back.
    foreground_changed();
    unsafe {
        SetWindowSubclass(hwnd(), Some(subclass_proc), 1, 0);
        refresh_ex_style();
    }
    place();
    Ok(())
}

/// Called once the UI has painted its first frame.
pub fn ui_ready() {
    READY.store(true, Ordering::Relaxed);
    if !HIDDEN.load(Ordering::Relaxed) {
        unsafe { ShowWindow(hwnd(), SW_SHOWNOACTIVATE) };
    }
}

/// Moves the window to the bottom-right of the primary monitor's work area.
pub fn place() {
    let hwnd = hwnd();
    if hwnd.is_null() {
        return;
    }
    unsafe {
        let monitor = MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY);
        let Some(info) = monitor_info(monitor) else { return };
        let (mut dpi_x, mut dpi_y) = (96, 96);
        GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
        let scale = f64::from(dpi_x) / 96.0;

        let width = (WINDOW_SIZE.0 * scale).round() as i32;
        let height = (WINDOW_SIZE.1 * scale).round() as i32;
        let margin = (MARGIN * scale).round() as i32;
        let x = info.rcWork.right - margin - width;
        let y = info.rcWork.bottom - margin - height;
        SetWindowPos(hwnd, HWND_TOPMOST, x, y, width, height, SWP_NOACTIVATE | SWP_NOOWNERZORDER);

        *lock(&PLACEMENT) = Placement { width, height, scale, monitor: monitor as isize };
    }
    apply_region();
}

pub fn set_box_size(width: f64, height: f64) {
    *lock(&BOX_SIZE) = (width, height);
    apply_region();
}

/// Clips the window to the visible box, which is anchored to its bottom-right corner.
/// Outside the region the window neither draws nor receives clicks.
fn apply_region() {
    let p = *lock(&PLACEMENT);
    let (width, height) = *lock(&BOX_SIZE);
    if p.width == 0 {
        return;
    }
    let (left, top) = if width > 0.0 && height > 0.0 {
        let padding = (REGION_PADDING * p.scale).round() as i32;
        (
            (p.width - (width * p.scale).round() as i32 - padding).max(0),
            (p.height - (height * p.scale).round() as i32 - padding).max(0),
        )
    } else {
        (p.width, p.height) // empty until the UI reports its size
    };
    unsafe {
        let region = CreateRectRgn(left, top, p.width, p.height);
        // The system owns the region after this call.
        SetWindowRgn(hwnd(), region, 1);
    }
}

/// Hides the island while a fullscreen app runs, so games and videos keep the screen.
pub fn set_hidden(hidden: bool) {
    HIDDEN.store(hidden, Ordering::Relaxed);
    if hidden {
        unsafe { ShowWindow(hwnd(), SW_HIDE) };
    } else if READY.load(Ordering::Relaxed) {
        unsafe { ShowWindow(hwnd(), SW_SHOWNOACTIVATE) };
    }
}

/// The panel needs keyboard input, so it may take focus; every other state never does.
pub fn set_interactive(active: bool, restore_focus: bool) {
    let hwnd = hwnd();
    unsafe {
        if active {
            let foreground = GetForegroundWindow();
            if foreground != hwnd {
                PREVIOUS_FOREGROUND.store(foreground as isize, Ordering::Relaxed);
            }
            NO_ACTIVATE.store(false, Ordering::Relaxed);
            refresh_ex_style();
            bring_to_foreground(hwnd);
        } else {
            NO_ACTIVATE.store(true, Ordering::Relaxed);
            refresh_ex_style();
            let previous = PREVIOUS_FOREGROUND.swap(0, Ordering::Relaxed) as HWND;
            if restore_focus && !previous.is_null() && IsWindow(previous) != 0 {
                SetForegroundWindow(previous);
            }
        }
    }
}

/// Called on every foreground change. Returns true when the panel was open and the user
/// moved to another app, which is when the panel should close. The windowing library's
/// own focus events are not used for this: WebView2 moves keyboard focus into its child
/// window, which reports the island as unfocused while it is still the active window.
pub fn foreground_changed() -> bool {
    let foreground = unsafe { GetForegroundWindow() };
    if foreground.is_null() || owned_by_this_process(foreground) {
        return false;
    }
    LAST_OTHER_FOREGROUND.store(foreground as isize, Ordering::Relaxed);
    !NO_ACTIVATE.load(Ordering::Relaxed)
}

fn owned_by_this_process(hwnd: HWND) -> bool {
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
    pid == unsafe { GetCurrentProcessId() }
}

/// `SetForegroundWindow` succeeds when the user just clicked the island. When the panel
/// is opened another way (tray, second launch), borrow the foreground thread's input state.
unsafe fn bring_to_foreground(hwnd: HWND) {
    if SetForegroundWindow(hwnd) != 0 {
        return;
    }
    let foreground_thread = GetWindowThreadProcessId(GetForegroundWindow(), null_mut());
    let this_thread = GetCurrentThreadId();
    if foreground_thread != 0 && foreground_thread != this_thread {
        AttachThreadInput(this_thread, foreground_thread, 1);
        SetForegroundWindow(hwnd);
        AttachThreadInput(this_thread, foreground_thread, 0);
    }
}

/// True when the foreground app covers the island's whole monitor (a game, a fullscreen
/// video, F11 in a browser) or Windows reports presentation mode.
pub fn fullscreen_app_active() -> bool {
    unsafe {
        let mut state = 0;
        if SHQueryUserNotificationState(&mut state) == 0
            && (state == QUNS_PRESENTATION_MODE || state == QUNS_RUNNING_D3D_FULL_SCREEN)
        {
            return true;
        }

        let foreground = GetForegroundWindow();
        if foreground.is_null() || foreground == hwnd() || is_shell_window(foreground) {
            return false;
        }
        let monitor = lock(&PLACEMENT).monitor as HMONITOR;
        if MonitorFromWindow(foreground, MONITOR_DEFAULTTONULL) != monitor {
            return false;
        }
        // A maximized window with a title bar is not fullscreen, even with an auto-hidden taskbar.
        let style = GetWindowLongW(foreground, GWL_STYLE) as u32;
        if style & WS_CAPTION == WS_CAPTION && IsZoomed(foreground) != 0 {
            return false;
        }
        let Some(info) = monitor_info(monitor) else { return false };
        let mut rect: RECT = zeroed();
        GetWindowRect(foreground, &mut rect);
        let m = info.rcMonitor;
        rect.left <= m.left && rect.top <= m.top && rect.right >= m.right && rect.bottom >= m.bottom
    }
}

pub fn uses_12_hour_clock() -> bool {
    let mut buf = [0u16; 80];
    // A null locale name means the user's regional settings.
    let len = unsafe { GetLocaleInfoEx(std::ptr::null(), LOCALE_STIMEFORMAT, buf.as_mut_ptr(), buf.len() as i32) };
    // "HH:mm:ss" is a 24-hour format; "h:mm:ss tt" is 12-hour.
    len > 0 && !String::from_utf16_lossy(&buf[..len as usize]).contains('H')
}

unsafe fn monitor_info(monitor: HMONITOR) -> Option<MONITORINFO> {
    let mut info: MONITORINFO = zeroed();
    info.cbSize = size_of::<MONITORINFO>() as u32;
    (GetMonitorInfoW(monitor, &mut info) != 0).then_some(info)
}

unsafe fn is_shell_window(hwnd: HWND) -> bool {
    let mut buf = [0u16; 64];
    let len = GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
    let class = String::from_utf16_lossy(&buf[..len.max(0) as usize]);
    matches!(class.as_str(), "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd")
}

/// Tool window: no taskbar button and no Alt+Tab entry. No-activate: see `NO_ACTIVATE`.
fn adjust_ex_style(style: u32) -> u32 {
    let style = (style | WS_EX_TOOLWINDOW) & !WS_EX_APPWINDOW;
    if NO_ACTIVATE.load(Ordering::Relaxed) {
        style | WS_EX_NOACTIVATE
    } else {
        style & !WS_EX_NOACTIVATE
    }
}

unsafe fn refresh_ex_style() {
    let hwnd = hwnd();
    let style = adjust_ex_style(GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32);
    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style as isize);
    SetWindowPos(
        hwnd,
        null_mut(),
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
    );
}

/// Keeps our extended styles in place if anything rewrites them, refuses activation on
/// click, and re-places the island when the screen layout changes.
unsafe extern "system" fn subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    _data: usize,
) -> LRESULT {
    match msg {
        WM_STYLECHANGING if wparam as i32 == GWL_EXSTYLE => {
            let change = &mut *(lparam as *mut STYLESTRUCT);
            change.styleNew = adjust_ex_style(change.styleNew);
        }
        WM_MOUSEACTIVATE if NO_ACTIVATE.load(Ordering::Relaxed) => return MA_NOACTIVATE as LRESULT,
        // WebView2 can still activate the window when its content takes focus. Outside the
        // panel that must never stick, so hand activation straight back to the user's app.
        WM_ACTIVATE if NO_ACTIVATE.load(Ordering::Relaxed) && (wparam & 0xFFFF) as u32 != WA_INACTIVE => {
            let deactivated = lparam as HWND;
            let target = if deactivated.is_null() || owned_by_this_process(deactivated) {
                LAST_OTHER_FOREGROUND.load(Ordering::Relaxed) as HWND
            } else {
                deactivated
            };
            if !target.is_null() {
                PostMessageW(hwnd, WM_RETURN_FOREGROUND, target as WPARAM, 0);
            }
        }
        // The undecorated window still has WS_CAPTION, so when its activation changes the
        // default handler paints a classic title bar ("Karsa" and a close button) over the
        // top of the window. lParam -1 tells it not to repaint the non-client area.
        WM_NCACTIVATE => return DefSubclassProc(hwnd, msg, wparam, -1),
        WM_RETURN_FOREGROUND => {
            let target = wparam as HWND;
            if NO_ACTIVATE.load(Ordering::Relaxed) && IsWindow(target) != 0 {
                SetForegroundWindow(target);
            }
            return 0;
        }
        WM_DISPLAYCHANGE => {
            let result = DefSubclassProc(hwnd, msg, wparam, lparam);
            place();
            return result;
        }
        WM_SETTINGCHANGE if wparam as u32 == SPI_SETWORKAREA => {
            let result = DefSubclassProc(hwnd, msg, wparam, lparam);
            place();
            return result;
        }
        _ => {}
    }
    DefSubclassProc(hwnd, msg, wparam, lparam)
}
