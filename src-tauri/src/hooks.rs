//! A background thread that tells the core about things only Windows knows: the foreground
//! window changing, a fullscreen app opening or closing, sleep and clock changes, the
//! quick-capture hotkey, and text being copied. It is event-driven: nothing runs while the
//! user is idle, apart from a housekeeping tick every 15 minutes.

use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Mutex, OnceLock};

use windows_sys::Win32::Foundation::{GlobalFree, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::DataExchange::{
    AddClipboardFormatListener, CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable,
    OpenClipboard, RegisterClipboardFormatW, RemoveClipboardFormatListener, SetClipboardData,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE};
use windows_sys::Win32::System::Ole::CF_UNICODETEXT;
use windows_sys::Win32::System::Threading::{GetCurrentThreadId, Sleep};
use windows_sys::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, MOD_WIN, VK_F1, VK_SPACE,
};
use windows_sys::Win32::UI::Shell::{SHAppBarMessage, ABM_NEW, ABM_REMOVE, ABN_FULLSCREENAPP, APPBARDATA};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, KillTimer, PostThreadMessageW,
    RegisterClassW, SendMessageW, SetTimer, TranslateMessage, EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MOVESIZEEND,
    MSG, OBJID_WINDOW, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WM_APP, WM_CLIPBOARDUPDATE, WM_HOTKEY,
    WM_POWERBROADCAST, WM_QUIT, WM_TIMECHANGE, WM_TIMER, WNDCLASSW, WS_EX_TOOLWINDOW, WS_POPUP,
};

pub enum HookEvent {
    /// The foreground window changed, finished moving, or toggled fullscreen.
    ForegroundChanged,
    /// The PC woke from sleep or the system clock changed; timers should check their
    /// deadlines now.
    ClockChanged,
    /// The quick-capture hotkey was pressed.
    Hotkey,
    /// Text was copied by another app and not marked as private.
    Copied(String),
    /// Every 15 minutes: time to drop clipboard items older than 24 hours.
    Housekeeping,
}

// wParam values of WM_POWERBROADCAST.
const PBT_APMRESUMESUSPEND: usize = 0x7;
const PBT_APMRESUMEAUTOMATIC: usize = 0x12;

static EVENTS: OnceLock<Sender<HookEvent>> = OnceLock::new();
static THREAD_ID: AtomicU32 = AtomicU32::new(0);
/// The hidden listener window, once the thread has created it.
static LISTENER: AtomicIsize = AtomicIsize::new(0);
/// The quick-capture hotkey as (modifiers, virtual key); (0, 0) when off.
static HOTKEY: Mutex<(u32, u32)> = Mutex::new((0, 0));
static HOTKEY_OK: AtomicBool = AtomicBool::new(true);
/// Record copied text (Settings) and not paused (Shelf).
static RECORD_CLIPBOARD: AtomicBool = AtomicBool::new(false);
/// Set while Karsa itself puts text on the clipboard, so it is not recorded again.
static SETTING_CLIPBOARD: AtomicBool = AtomicBool::new(false);
static CLIPBOARD_TEXT: Mutex<Vec<u16>> = Mutex::new(Vec::new());

const WM_APPBAR_CALLBACK: u32 = WM_APP + 1;
const WM_SET_HOTKEY: u32 = WM_APP + 3;
const WM_SET_CLIPBOARD: u32 = WM_APP + 4;
const HOTKEY_ID: i32 = 1;
const HOUSEKEEPING_TIMER: usize = 1;
const HOUSEKEEPING_MS: u32 = 15 * 60 * 1000;
/// Longer copies (whole documents, logs) are not kept.
const MAX_CLIP_CHARS: usize = 20_000;

fn send(event: HookEvent) {
    if let Some(events) = EVENTS.get() {
        let _ = events.send(event);
    }
}

/// Re-evaluates the foreground and fullscreen state now, e.g. after a setting changed.
pub fn recheck() {
    send(HookEvent::ForegroundChanged);
}

fn listener() -> HWND {
    LISTENER.load(Ordering::Relaxed) as HWND
}

/// "Win+Shift+K", "Ctrl+Alt+Space", "Alt+F9"… into (modifiers, virtual key). At least one
/// modifier is required, so typing never triggers it.
pub fn parse_hotkey(spec: &str) -> Option<(u32, u32)> {
    let (mut mods, mut vk) = (0, 0);
    for part in spec.split('+').map(|p| p.trim().to_ascii_lowercase()) {
        match part.as_str() {
            "ctrl" | "control" => mods |= MOD_CONTROL,
            "alt" => mods |= MOD_ALT,
            "shift" => mods |= MOD_SHIFT,
            "win" => mods |= MOD_WIN,
            "space" => vk = VK_SPACE as u32,
            k if k.len() == 1 && k.chars().all(|c| c.is_ascii_alphanumeric()) => vk = k.to_ascii_uppercase().as_bytes()[0] as u32,
            k => match k.strip_prefix('f').and_then(|n| n.parse::<u32>().ok()) {
                Some(n @ 1..=12) => vk = VK_F1 as u32 + n - 1,
                _ => return None,
            },
        }
    }
    (vk != 0 && mods != 0).then_some((mods, vk))
}

/// Sets the quick-capture hotkey ("" turns it off). Fails when the shortcut is not valid
/// or another app already holds it.
pub fn set_hotkey(spec: &str) -> Result<(), String> {
    let parsed = if spec.is_empty() { (0, 0) } else { parse_hotkey(spec).ok_or("Unknown shortcut")? };
    *HOTKEY.lock().unwrap_or_else(|p| p.into_inner()) = parsed;
    let window = listener();
    if !window.is_null() {
        // Registered on the hook thread, which owns the window that receives WM_HOTKEY.
        unsafe { SendMessageW(window, WM_SET_HOTKEY, 0, 0) };
    }
    if hotkey_ok() {
        Ok(())
    } else {
        Err("Another app already uses this shortcut".into())
    }
}

pub fn hotkey_ok() -> bool {
    HOTKEY_OK.load(Ordering::Relaxed)
}

/// Whether copied text is passed on as `HookEvent::Copied`.
pub fn set_recording(record: bool) {
    RECORD_CLIPBOARD.store(record, Ordering::Relaxed);
}

/// Puts text on the clipboard without recording it as a new copy.
pub fn set_clipboard_text(text: &str) -> Result<(), String> {
    *CLIPBOARD_TEXT.lock().unwrap_or_else(|p| p.into_inner()) = text.encode_utf16().chain([0]).collect();
    let window = listener();
    if window.is_null() || unsafe { SendMessageW(window, WM_SET_CLIPBOARD, 0, 0) } == 0 {
        return Err("Could not copy; another app is using the clipboard".into());
    }
    Ok(())
}

unsafe fn register_hotkey(window: HWND) {
    UnregisterHotKey(window, HOTKEY_ID);
    let (mods, vk) = *HOTKEY.lock().unwrap_or_else(|p| p.into_inner());
    let ok = vk == 0 || RegisterHotKey(window, HOTKEY_ID, mods | MOD_NOREPEAT, vk) != 0;
    HOTKEY_OK.store(ok, Ordering::Relaxed);
}

/// Opens the clipboard, retrying briefly while another app holds it.
unsafe fn open_clipboard(window: HWND) -> bool {
    for _ in 0..5 {
        if OpenClipboard(window) != 0 {
            return true;
        }
        Sleep(15);
    }
    false
}

fn format(name: &str) -> u32 {
    let wide: Vec<u16> = name.encode_utf16().chain([0]).collect();
    unsafe { RegisterClipboardFormatW(wide.as_ptr()) }
}

/// The copied text, unless the copying app marked it as private (password managers do,
/// following Windows' clipboard history conventions), or it is empty or very long.
unsafe fn read_copied_text(window: HWND) -> Option<String> {
    if IsClipboardFormatAvailable(CF_UNICODETEXT as u32) == 0 {
        return None;
    }
    let private = ["ExcludeClipboardContentFromMonitorProcessing", "Clipboard Viewer Ignore"];
    if private.iter().any(|name| IsClipboardFormatAvailable(format(name)) != 0) {
        return None;
    }
    if !open_clipboard(window) {
        return None;
    }
    let mut text = None;
    let history_flag = format("CanIncludeInClipboardHistory");
    let excluded = !GetClipboardData(history_flag).is_null() && {
        let handle = GetClipboardData(history_flag);
        let value = GlobalLock(handle) as *const u32;
        let excluded = !value.is_null() && *value == 0;
        GlobalUnlock(handle);
        excluded
    };
    if !excluded {
        let handle = GetClipboardData(CF_UNICODETEXT as u32);
        let data = GlobalLock(handle) as *const u16;
        if !data.is_null() {
            let max = GlobalSize(handle) / 2;
            let len = (0..max).take_while(|&i| *data.add(i) != 0).count();
            if len <= MAX_CLIP_CHARS {
                let s = String::from_utf16_lossy(std::slice::from_raw_parts(data, len));
                if !s.trim().is_empty() {
                    text = Some(s);
                }
            }
            GlobalUnlock(handle);
        }
    }
    CloseClipboard();
    text
}

unsafe fn write_clipboard_text(window: HWND) -> bool {
    let text = CLIPBOARD_TEXT.lock().unwrap_or_else(|p| p.into_inner()).clone();
    if !open_clipboard(window) {
        return false;
    }
    SETTING_CLIPBOARD.store(true, Ordering::Relaxed);
    EmptyClipboard();
    let bytes = text.len() * 2;
    let handle = GlobalAlloc(GMEM_MOVEABLE, bytes);
    let mut ok = false;
    if !handle.is_null() {
        let dest = GlobalLock(handle) as *mut u16;
        if !dest.is_null() {
            std::ptr::copy_nonoverlapping(text.as_ptr(), dest, text.len());
            GlobalUnlock(handle);
            ok = !SetClipboardData(CF_UNICODETEXT as u32, handle).is_null();
        }
        if !ok {
            GlobalFree(handle);
        }
    }
    CloseClipboard();
    ok
}

unsafe extern "system" fn win_event_proc(
    _hook: HWINEVENTHOOK,
    _event: u32,
    _hwnd: HWND,
    id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    if id_object == OBJID_WINDOW {
        send(HookEvent::ForegroundChanged);
    }
}

/// Window procedure of a hidden window registered as an app bar. The shell sends
/// app bars `ABN_FULLSCREENAPP` whenever a fullscreen app opens or closes. Being a
/// top-level window, it also receives the system's power broadcasts.
unsafe extern "system" fn appbar_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_APPBAR_CALLBACK {
        if wparam as u32 == ABN_FULLSCREENAPP {
            send(HookEvent::ForegroundChanged);
        }
        return 0;
    }
    match msg {
        WM_HOTKEY if wparam as i32 == HOTKEY_ID => {
            send(HookEvent::Hotkey);
            return 0;
        }
        WM_CLIPBOARDUPDATE => {
            // Our own copy (an item copied back from the history) is not a new copy.
            if SETTING_CLIPBOARD.swap(false, Ordering::Relaxed) {
                return 0;
            }
            if RECORD_CLIPBOARD.load(Ordering::Relaxed) {
                if let Some(text) = read_copied_text(hwnd) {
                    send(HookEvent::Copied(text));
                }
            }
            return 0;
        }
        WM_SET_HOTKEY => {
            register_hotkey(hwnd);
            return 0;
        }
        WM_SET_CLIPBOARD => return write_clipboard_text(hwnd) as LRESULT,
        WM_TIMER if wparam == HOUSEKEEPING_TIMER => {
            send(HookEvent::Housekeeping);
            return 0;
        }
        _ => {}
    }
    let resumed = msg == WM_POWERBROADCAST && matches!(wparam, PBT_APMRESUMESUSPEND | PBT_APMRESUMEAUTOMATIC);
    if resumed || msg == WM_TIMECHANGE {
        send(HookEvent::ClockChanged);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

pub fn start(events: Sender<HookEvent>) -> std::io::Result<()> {
    let _ = EVENTS.set(events);
    std::thread::Builder::new()
        .name("karsa-hooks".into())
        .spawn(|| unsafe { run() })?;
    Ok(())
}

/// Ends the hook thread's message loop so it can unregister everything cleanly.
pub fn stop() {
    let id = THREAD_ID.load(Ordering::Relaxed);
    if id != 0 {
        unsafe { PostThreadMessageW(id, WM_QUIT, 0, 0) };
    }
}

unsafe fn run() {
    THREAD_ID.store(GetCurrentThreadId(), Ordering::Relaxed);
    let instance = GetModuleHandleW(null());

    let flags = WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS;
    let foreground_hook = SetWinEventHook(
        EVENT_SYSTEM_FOREGROUND,
        EVENT_SYSTEM_FOREGROUND,
        null_mut(),
        Some(win_event_proc),
        0,
        0,
        flags,
    );
    let move_size_hook = SetWinEventHook(
        EVENT_SYSTEM_MOVESIZEEND,
        EVENT_SYSTEM_MOVESIZEEND,
        null_mut(),
        Some(win_event_proc),
        0,
        0,
        flags,
    );

    let class_name: Vec<u16> = "KarsaFullscreenListener\0".encode_utf16().collect();
    let class = WNDCLASSW {
        lpfnWndProc: Some(appbar_proc),
        hInstance: instance,
        lpszClassName: class_name.as_ptr(),
        ..zeroed()
    };
    RegisterClassW(&class);
    let listener = CreateWindowExW(
        WS_EX_TOOLWINDOW,
        class_name.as_ptr(),
        null(),
        WS_POPUP,
        0,
        0,
        0,
        0,
        null_mut(),
        null_mut(),
        instance,
        null(),
    );
    // Registering without ABM_SETPOS reserves no screen space; it only subscribes to notifications.
    let mut appbar: APPBARDATA = zeroed();
    appbar.cbSize = size_of::<APPBARDATA>() as u32;
    appbar.hWnd = listener;
    appbar.uCallbackMessage = WM_APPBAR_CALLBACK;
    SHAppBarMessage(ABM_NEW, &mut appbar);

    LISTENER.store(listener as isize, Ordering::Relaxed);
    register_hotkey(listener);
    AddClipboardFormatListener(listener);
    SetTimer(listener, HOUSEKEEPING_TIMER, HOUSEKEEPING_MS, None);

    let mut msg: MSG = zeroed();
    while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    KillTimer(listener, HOUSEKEEPING_TIMER);
    RemoveClipboardFormatListener(listener);
    UnregisterHotKey(listener, HOTKEY_ID);
    LISTENER.store(0, Ordering::Relaxed);
    SHAppBarMessage(ABM_REMOVE, &mut appbar);
    DestroyWindow(listener);
    UnhookWinEvent(move_size_hook);
    UnhookWinEvent(foreground_hook);
}
