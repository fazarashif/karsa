//! A background thread that tells the core when the foreground window changes or a
//! fullscreen app opens or closes. It is event-driven: nothing runs while the user is idle.

use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::Sender;
use std::sync::OnceLock;

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows_sys::Win32::UI::Shell::{SHAppBarMessage, ABM_NEW, ABM_REMOVE, ABN_FULLSCREENAPP, APPBARDATA};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, PostThreadMessageW,
    RegisterClassW, TranslateMessage, EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MOVESIZEEND, MSG,
    OBJID_WINDOW, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WM_APP, WM_POWERBROADCAST, WM_QUIT,
    WM_TIMECHANGE, WNDCLASSW, WS_EX_TOOLWINDOW, WS_POPUP,
};

pub enum HookEvent {
    /// The foreground window changed, finished moving, or toggled fullscreen.
    ForegroundChanged,
    /// The PC woke from sleep or the system clock changed; timers should check their
    /// deadlines now.
    ClockChanged,
}

// wParam values of WM_POWERBROADCAST.
const PBT_APMRESUMESUSPEND: usize = 0x7;
const PBT_APMRESUMEAUTOMATIC: usize = 0x12;

static EVENTS: OnceLock<Sender<HookEvent>> = OnceLock::new();
static THREAD_ID: AtomicU32 = AtomicU32::new(0);

const WM_APPBAR_CALLBACK: u32 = WM_APP + 1;

fn send(event: HookEvent) {
    if let Some(events) = EVENTS.get() {
        let _ = events.send(event);
    }
}

/// Re-evaluates the foreground and fullscreen state now, e.g. after a setting changed.
pub fn recheck() {
    send(HookEvent::ForegroundChanged);
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

    let mut msg: MSG = zeroed();
    while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    SHAppBarMessage(ABM_REMOVE, &mut appbar);
    DestroyWindow(listener);
    UnhookWinEvent(move_size_hook);
    UnhookWinEvent(foreground_hook);
}
