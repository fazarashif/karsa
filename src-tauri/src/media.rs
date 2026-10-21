//! Media controls: what Windows' media sessions (the ones behind the volume flyout) say is
//! playing, from Spotify, a browser tab, Media Player and so on. Every session is listed;
//! the one shown is the session Windows calls current, unless the user picked another.
//! Play/pause, previous, next, seeking, shuffle and repeat go through the session, when
//! the app supports them; volume goes through the app's Windows audio session. One thread
//! waits for Windows' change events; with nothing playing, nothing runs.

use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use windows::Foundation::TypedEventHandler;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as Manager,
    GlobalSystemMediaTransportControlsSessionMediaProperties as Properties,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
};
use windows::Media::MediaPlaybackAutoRepeatMode as RepeatMode;
use windows::Storage::Streams::DataReader;

/// One app with media, for the session switcher.
#[derive(Serialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    id: String,
    app: String,
    title: String,
    playing: bool,
}

#[derive(Serialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaState {
    /// Something has a media session (playing or paused).
    active: bool,
    playing: bool,
    title: String,
    artist: String,
    album: String,
    /// A friendly name for the app, e.g. "Spotify" or "Chrome".
    app: String,
    /// The site, for a browser, when it can be told from the window title ("YouTube Music").
    site: String,
    /// The shown session's id, as listed in `sessions`.
    id: String,
    /// The app's icon as a PNG data: URL, or empty.
    app_icon: String,
    /// The album art as a data: URL, or empty.
    art: String,
    can_previous: bool,
    can_next: bool,
    can_seek: bool,
    can_shuffle: bool,
    can_repeat: bool,
    shuffle: bool,
    /// "off", "all" or "one".
    repeat: String,
    /// The position when the app last reported it, the track length, and when that was
    /// (Unix ms); the UI moves the position on from there at `rate`.
    position_ms: i64,
    duration_ms: i64,
    position_at: i64,
    rate: f64,
    sessions: Vec<SessionInfo>,
    /// The app's volume (0 to 1) and mute, when Windows has an audio session for it.
    volume: Option<f32>,
    muted: bool,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    Toggle,
    Next,
    Previous,
    Seek { ms: i64 },
    Shuffle { on: bool },
    Repeat { mode: String },
    Select { id: String },
    Volume { level: f32 },
    Mute { on: bool },
}

enum Msg {
    Changed,
    Timeline,
    Act(Action),
}

static SIGNAL: OnceLock<Mutex<Sender<Msg>>> = OnceLock::new();
static STATE: Mutex<Option<MediaState>> = Mutex::new(None);

fn signal(msg: Msg) {
    if let Some(sender) = SIGNAL.get() {
        let _ = sender.lock().unwrap_or_else(|p| p.into_inner()).send(msg);
    }
}

pub fn spawn(app: AppHandle) -> std::io::Result<()> {
    let (sender, receiver) = channel();
    let _ = SIGNAL.set(Mutex::new(sender));
    std::thread::Builder::new().name("karsa-media".into()).spawn(move || {
        use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        if let Err(e) = run(&app, receiver) {
            eprintln!("media controls unavailable: {e}");
        }
    })?;
    Ok(())
}

fn on<T, U>(msg: fn() -> Msg) -> TypedEventHandler<T, U>
where
    T: windows::core::RuntimeType + 'static,
    U: windows::core::RuntimeType + 'static,
{
    TypedEventHandler::new(move |_, _| {
        signal(msg());
        Ok(())
    })
}

/// A session with the event registrations Karsa holds on it.
struct Watched {
    session: Session,
    id: String,
    tokens: (i64, i64),
    timeline: Option<i64>,
}

fn run(app: &AppHandle, messages: Receiver<Msg>) -> windows::core::Result<()> {
    let manager = Manager::RequestAsync()?.join()?;
    manager.CurrentSessionChanged(&on(|| Msg::Changed))?;
    manager.SessionsChanged(&on(|| Msg::Changed))?;
    signal(Msg::Changed);

    let mut watched: Vec<Watched> = Vec::new();
    let mut selected: Option<String> = None;
    let mut art_key = String::new();
    let mut art = String::new();
    let mut site_key = String::new();
    let mut site = String::new();
    let mut icons: HashMap<String, String> = HashMap::new();
    let mut state = MediaState::default();

    while let Ok(first) = messages.recv() {
        // Windows often sends several changes at once; act on the actions, refresh once.
        let mut full = false;
        for msg in std::iter::once(first).chain(messages.try_iter()) {
            match msg {
                Msg::Changed => full = true,
                Msg::Timeline => {}
                Msg::Act(action) => {
                    full = true;
                    let shown = watched.iter().find(|w| w.id == state.id).map(|w| w.session.clone());
                    act(action, shown.as_ref(), &state, &mut selected);
                }
            }
        }

        if full {
            // Re-list the sessions and keep the event registrations in step with them.
            let list = manager.GetSessions()?;
            let mut sessions = Vec::new();
            for i in 0..list.Size()? {
                sessions.push(list.GetAt(i)?);
            }
            let mut ids = Vec::new();
            for s in &sessions {
                let base = s.SourceAppUserModelId().map(|h| h.to_string()).unwrap_or_default();
                let mut id = base.clone();
                let mut n = 2;
                while ids.contains(&id) {
                    id = format!("{base}#{n}");
                    n += 1;
                }
                ids.push(id);
            }
            watched.retain(|w| {
                let keep = sessions.iter().zip(&ids).any(|(s, id)| s == &w.session && id == &w.id);
                if !keep {
                    let _ = w.session.RemoveMediaPropertiesChanged(w.tokens.0);
                    let _ = w.session.RemovePlaybackInfoChanged(w.tokens.1);
                    if let Some(t) = w.timeline {
                        let _ = w.session.RemoveTimelinePropertiesChanged(t);
                    }
                }
                keep
            });
            for (s, id) in sessions.iter().zip(&ids) {
                if !watched.iter().any(|w| &w.session == s) {
                    let tokens = (s.MediaPropertiesChanged(&on(|| Msg::Changed))?, s.PlaybackInfoChanged(&on(|| Msg::Changed))?);
                    watched.push(Watched { session: s.clone(), id: id.clone(), tokens, timeline: None });
                }
            }

            // The shown session: the user's pick while it lasts, else Windows' current one.
            if selected.as_ref().is_some_and(|id| !watched.iter().any(|w| &w.id == id)) {
                selected = None;
            }
            let current = manager.GetCurrentSession().ok();
            let shown_index = selected
                .as_ref()
                .and_then(|id| watched.iter().position(|w| &w.id == id))
                .or_else(|| current.as_ref().and_then(|c| watched.iter().position(|w| &w.session == c)))
                .or(if watched.is_empty() { None } else { Some(0) });
            for (i, w) in watched.iter_mut().enumerate() {
                let want = Some(i) == shown_index;
                if want && w.timeline.is_none() {
                    w.timeline = w.session.TimelinePropertiesChanged(&on(|| Msg::Timeline)).ok();
                } else if !want {
                    if let Some(t) = w.timeline.take() {
                        let _ = w.session.RemoveTimelinePropertiesChanged(t);
                    }
                }
            }

            let infos: Vec<SessionInfo> = watched
                .iter()
                .map(|w| {
                    let props = w.session.TryGetMediaPropertiesAsync().and_then(|op| op.join()).ok();
                    SessionInfo {
                        id: w.id.clone(),
                        app: friendly_app(&w.id),
                        title: props.and_then(|p| p.Title().ok()).map(|h| h.to_string()).unwrap_or_default(),
                        playing: w.session.GetPlaybackInfo().and_then(|i| i.PlaybackStatus()).ok() == Some(Status::Playing),
                    }
                })
                .collect();

            state = match shown_index.map(|i| &watched[i]) {
                None => MediaState::default(),
                Some(w) => {
                    let s = &w.session;
                    let props = s.TryGetMediaPropertiesAsync().and_then(|op| op.join()).ok();
                    let info = s.GetPlaybackInfo().ok();
                    let controls = info.as_ref().and_then(|i| i.Controls().ok());
                    let text = |f: fn(&Properties) -> windows::core::Result<windows::core::HSTRING>| {
                        props.as_ref().and_then(|p| f(p).ok()).map(|h| h.to_string()).unwrap_or_default()
                    };
                    let flag = |f: fn(&windows::Media::Control::GlobalSystemMediaTransportControlsSessionPlaybackControls) -> windows::core::Result<bool>| {
                        controls.as_ref().and_then(|c| f(c).ok()).unwrap_or(false)
                    };
                    let title = text(Properties::Title);
                    let artist = text(Properties::Artist);
                    let app_name = friendly_app(&w.id);
                    // Album art is read once per track (and again while it is still missing).
                    let key = format!("{app_name}\n{title}\n{artist}");
                    if key != art_key || art.is_empty() {
                        art = props.as_ref().and_then(read_art).unwrap_or_default();
                        art_key = key.clone();
                    }
                    // The site is looked up once per track, from the browser's window titles.
                    if key != site_key {
                        site = if is_browser(&w.id) { browser_site(&w.id, &title) } else { String::new() };
                        site_key = key;
                    }
                    let base_id = w.id.split('#').next().unwrap_or(&w.id).to_string();
                    let icon = icons.entry(base_id.clone()).or_insert_with(|| app_icon(&base_id)).clone();
                    let (volume, muted) = audio_volume(&base_id);
                    let mut next = MediaState {
                        active: !title.is_empty(),
                        playing: info.as_ref().and_then(|i| i.PlaybackStatus().ok()) == Some(Status::Playing),
                        title,
                        artist,
                        album: text(Properties::AlbumTitle),
                        app: app_name,
                        site: site.clone(),
                        id: w.id.clone(),
                        app_icon: icon,
                        art: art.clone(),
                        can_previous: flag(|c| c.IsPreviousEnabled()),
                        can_next: flag(|c| c.IsNextEnabled()),
                        can_seek: flag(|c| c.IsPlaybackPositionEnabled()),
                        can_shuffle: flag(|c| c.IsShuffleEnabled()),
                        can_repeat: flag(|c| c.IsRepeatEnabled()),
                        shuffle: info.as_ref().and_then(|i| i.IsShuffleActive().ok()).and_then(|v| v.Value().ok()).unwrap_or(false),
                        repeat: match info.as_ref().and_then(|i| i.AutoRepeatMode().ok()).and_then(|v| v.Value().ok()) {
                            Some(RepeatMode::Track) => "one",
                            Some(RepeatMode::List) => "all",
                            _ => "off",
                        }
                        .into(),
                        rate: info.as_ref().and_then(|i| i.PlaybackRate().ok()).and_then(|v| v.Value().ok()).unwrap_or(1.0),
                        sessions: infos,
                        volume,
                        muted,
                        ..Default::default()
                    };
                    read_timeline(s, &mut next);
                    next
                }
            };
        } else if let Some(w) = watched.iter().find(|w| w.id == state.id) {
            // Only the position moved.
            read_timeline(&w.session, &mut state);
        }

        let mut stored = STATE.lock().unwrap_or_else(|p| p.into_inner());
        if stored.as_ref() != Some(&state) {
            *stored = Some(state.clone());
            drop(stored);
            let _ = app.emit("media://changed", state.clone());
        }
    }
    Ok(())
}

/// Windows' 100-ns ticks since 1601 to Unix milliseconds.
fn unix_ms(ticks: i64) -> i64 {
    (ticks - 116_444_736_000_000_000) / 10_000
}

fn read_timeline(s: &Session, state: &mut MediaState) {
    let Ok(t) = s.GetTimelineProperties() else { return };
    let start = t.StartTime().map(|d| d.Duration).unwrap_or(0);
    let end = t.EndTime().map(|d| d.Duration).unwrap_or(0);
    let position = t.Position().map(|d| d.Duration).unwrap_or(0);
    state.duration_ms = ((end - start) / 10_000).max(0);
    state.position_ms = ((position - start) / 10_000).clamp(0, state.duration_ms.max(0));
    state.position_at = t.LastUpdatedTime().map(|d| unix_ms(d.UniversalTime)).ok().filter(|ms| *ms > 0).unwrap_or_else(crate::db::now_ms);
    // An app that reports no length has no timeline to show or seek.
    if state.duration_ms == 0 {
        state.can_seek = false;
    }
}

fn act(action: Action, session: Option<&Session>, state: &MediaState, selected: &mut Option<String>) {
    match action {
        Action::Select { id } => *selected = Some(id),
        Action::Volume { level } => set_audio(&state.id, Some(level.clamp(0.0, 1.0)), None),
        Action::Mute { on } => set_audio(&state.id, None, Some(on)),
        action => {
            let Some(s) = session else { return };
            let op = match action {
                Action::Toggle => s.TryTogglePlayPauseAsync(),
                Action::Next => s.TrySkipNextAsync(),
                Action::Previous => s.TrySkipPreviousAsync(),
                Action::Seek { ms } => s.TryChangePlaybackPositionAsync(ms.max(0) * 10_000),
                Action::Shuffle { on } => s.TryChangeShuffleActiveAsync(on),
                Action::Repeat { mode } => s.TryChangeAutoRepeatModeAsync(match mode.as_str() {
                    "one" => RepeatMode::Track,
                    "all" => RepeatMode::List,
                    _ => RepeatMode::None,
                }),
                _ => return,
            };
            let _ = op.and_then(|op| op.join());
        }
    }
}

/// The part of an app id that names the app: "Spotify.exe" → "spotify", "MSEdge" →
/// "msedge", a Store id "SpotifyAB.SpotifyMusic_zpd…!Spotify" → "spotify".
fn app_base(id: &str) -> String {
    let id = id.split('#').next().unwrap_or(id);
    let base = id.rsplit('!').next().unwrap_or(id);
    let base = base.split('_').next().unwrap_or(base);
    let base = base.strip_suffix(".exe").or_else(|| base.strip_suffix(".EXE")).unwrap_or(base);
    base.rsplit('.').next().unwrap_or(base).to_ascii_lowercase()
}

/// "Spotify.exe" → "Spotify", "MSEdge" → "Microsoft Edge", a Store app id → its name.
fn friendly_app(id: &str) -> String {
    let base = app_base(id);
    match base.as_str() {
        "msedge" => "Microsoft Edge".into(),
        "chrome" => "Chrome".into(),
        "firefox" | "308046b0af4a39cb" => "Firefox".into(),
        "brave" => "Brave".into(),
        "opera" => "Opera".into(),
        "spotify" => "Spotify".into(),
        "zunemusic" | "media" => "Media Player".into(),
        _ => {
            let raw = id.split('#').next().unwrap_or(id);
            let raw = raw.rsplit('!').next().unwrap_or(raw);
            raw.strip_suffix(".exe").unwrap_or(raw).to_string()
        }
    }
}

/// The process name an app plays its audio from.
fn process_hint(id: &str) -> String {
    match app_base(id).as_str() {
        "zunemusic" | "media" => "microsoft.media.player".into(),
        "308046b0af4a39cb" => "firefox".into(),
        base => base.to_string(),
    }
}

fn is_browser(id: &str) -> bool {
    matches!(process_hint(id).as_str(), "chrome" | "msedge" | "firefox" | "brave" | "opera" | "vivaldi")
}

/// The exe name (lowercase, without ".exe") of a process.
fn process_name(pid: u32) -> Option<(String, String)> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION};
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return None;
        }
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut len) != 0;
        CloseHandle(handle);
        if !ok {
            return None;
        }
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        let stem = std::path::Path::new(&path).file_stem()?.to_string_lossy().to_lowercase();
        Some((stem, path))
    }
}

/// Sites the island can name from a browser's window title ("<page> - YouTube Music - Google Chrome").
const SITES: [&str; 9] = ["YouTube Music", "YouTube", "Spotify", "SoundCloud", "Apple Music", "Deezer", "TIDAL", "Bandcamp", "JOOX"];

/// The site playing in a browser, when the playing tab is the active tab of one of its
/// windows: its title then holds the track's title. Read once per track.
fn browser_site(id: &str, title: &str) -> String {
    use windows_sys::Win32::Foundation::{HWND, LPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible};
    if title.is_empty() {
        return String::new();
    }
    let hint = process_hint(id);
    let mut titles: Vec<String> = Vec::new();
    unsafe extern "system" fn collect(hwnd: HWND, list: LPARAM) -> i32 {
        let (hint, titles) = &mut *(list as *mut (&str, &mut Vec<String>));
        if IsWindowVisible(hwnd) == 0 {
            return 1;
        }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if process_name(pid).is_some_and(|(name, _)| name == *hint) {
            let mut buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
            if len > 0 {
                titles.push(String::from_utf16_lossy(&buf[..len as usize]));
            }
        }
        1
    }
    let mut context: (&str, &mut Vec<String>) = (&hint, &mut titles);
    unsafe { EnumWindows(Some(collect), &mut context as *mut _ as LPARAM) };
    let Some(window) = titles.iter().find(|t| t.contains(title)) else { return String::new() };
    // Drop the browser's own name, then look for a known site name at the end.
    let page = window.rsplit_once(" - ").map(|(page, _)| page).unwrap_or(window);
    if let Some(site) = SITES.iter().find(|s| page.ends_with(&format!(" - {s}")) || page.ends_with(&format!(" | {s}"))) {
        return site.to_string();
    }
    // Spotify's web player titles the tab "Song • Artist", with no site name.
    if page.contains(" • ") {
        return "Spotify".into();
    }
    String::new()
}

/// The app's icon as the Start menu shows it, or its exe's icon.
fn app_icon(id: &str) -> String {
    use windows::core::HSTRING;
    use windows::Win32::System::Com::CoTaskMemFree;
    use windows::Win32::UI::Shell::{IShellItem, SHCreateItemFromParsingName, SHGetIDListFromObject};
    use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL;
    use windows_sys::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGFI_PIDL};
    use windows_sys::Win32::UI::WindowsAndMessaging::DestroyIcon;

    unsafe {
        let name = HSTRING::from(format!("shell:AppsFolder\\{id}"));
        let from_start = SHCreateItemFromParsingName::<_, _, IShellItem>(&name, None)
            .and_then(|item| SHGetIDListFromObject(&item))
            .ok()
            .and_then(|pidl| {
                let mut info: SHFILEINFOW = std::mem::zeroed();
                let got = SHGetFileInfoW(pidl as *const u16, 0, &mut info, std::mem::size_of::<SHFILEINFOW>() as u32, SHGFI_PIDL | SHGFI_ICON | SHGFI_LARGEICON);
                CoTaskMemFree(Some(pidl as _));
                if got == 0 || info.hIcon.is_null() {
                    return None;
                }
                let png = crate::shelf::icon_png(info.hIcon);
                DestroyIcon(info.hIcon);
                png
            });
        if let Some(png) = from_start {
            return png;
        }
        // Not in the Start menu: the icon of the exe that plays the audio.
        let hint = process_hint(id);
        let Some(path) = audio_sessions().into_iter().find_map(|(_, pid)| process_name(pid).filter(|(name, _)| *name == hint).map(|(_, path)| path)) else {
            return String::new();
        };
        let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
        let mut info: SHFILEINFOW = std::mem::zeroed();
        if SHGetFileInfoW(wide.as_ptr(), FILE_ATTRIBUTE_NORMAL, &mut info, std::mem::size_of::<SHFILEINFOW>() as u32, SHGFI_ICON | SHGFI_LARGEICON) == 0 || info.hIcon.is_null() {
            return String::new();
        }
        let png = crate::shelf::icon_png(info.hIcon).unwrap_or_default();
        DestroyIcon(info.hIcon);
        png
    }
}

/// Every audio session on the default output device, with the process it belongs to.
fn audio_sessions() -> Vec<(windows::Win32::Media::Audio::IAudioSessionControl2, u32)> {
    use windows::core::Interface;
    use windows::Win32::Media::Audio::{eMultimedia, eRender, IAudioSessionControl2, IAudioSessionManager2, IMMDeviceEnumerator, MMDeviceEnumerator};
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};
    unsafe {
        let result = (|| -> windows::core::Result<Vec<(IAudioSessionControl2, u32)>> {
            let devices: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let device = devices.GetDefaultAudioEndpoint(eRender, eMultimedia)?;
            let manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)?;
            let list = manager.GetSessionEnumerator()?;
            let mut out = Vec::new();
            for i in 0..list.GetCount()? {
                let control = list.GetSession(i)?.cast::<IAudioSessionControl2>()?;
                if let Ok(pid) = control.GetProcessId() {
                    out.push((control, pid));
                }
            }
            Ok(out)
        })();
        result.unwrap_or_default()
    }
}

/// The app's audio sessions (a browser may have several).
fn app_audio(id: &str) -> Vec<windows::Win32::Media::Audio::ISimpleAudioVolume> {
    use windows::core::Interface;
    let hint = process_hint(id);
    audio_sessions()
        .into_iter()
        .filter(|(_, pid)| process_name(*pid).is_some_and(|(name, _)| name == hint))
        .filter_map(|(control, _)| control.cast().ok())
        .collect()
}

fn audio_volume(id: &str) -> (Option<f32>, bool) {
    let Some(volume) = app_audio(id).into_iter().next() else { return (None, false) };
    unsafe { (volume.GetMasterVolume().ok(), volume.GetMute().map(|m| m.as_bool()).unwrap_or(false)) }
}

fn set_audio(id: &str, level: Option<f32>, mute: Option<bool>) {
    for volume in app_audio(id) {
        unsafe {
            if let Some(level) = level {
                let _ = volume.SetMasterVolume(level, std::ptr::null());
            }
            if let Some(mute) = mute {
                let _ = volume.SetMute(mute, std::ptr::null());
            }
        }
    }
}

fn read_art(props: &Properties) -> Option<String> {
    let stream = props.Thumbnail().ok()?.OpenReadAsync().ok()?.join().ok()?;
    let size = stream.Size().ok()? as u32;
    if size == 0 || size > 2_000_000 {
        return None;
    }
    let reader = DataReader::CreateDataReader(&stream.GetInputStreamAt(0).ok()?).ok()?;
    reader.LoadAsync(size).ok()?.join().ok()?;
    let mut bytes = vec![0u8; size as usize];
    reader.ReadBytes(&mut bytes).ok()?;
    let mime = if bytes.starts_with(&[0xFF, 0xD8]) { "image/jpeg" } else { "image/png" };
    Some(format!("data:{mime};base64,{}", base64(&bytes)))
}

pub(crate) fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(TABLE[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[tauri::command]
pub fn media_state() -> MediaState {
    STATE.lock().unwrap_or_else(|p| p.into_inner()).clone().unwrap_or_default()
}

#[tauri::command]
pub fn media_control(action: Action) {
    signal(Msg::Act(action));
}
