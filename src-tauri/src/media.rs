//! Media controls: what Windows' media session (the one behind the volume flyout) says is
//! playing, from Spotify, a browser tab, Media Player and so on, with play/pause, next and
//! previous. One thread waits for Windows' change events; with nothing playing, nothing runs.

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
use windows::Storage::Streams::DataReader;

#[derive(Serialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaState {
    /// Something has a media session (playing or paused).
    active: bool,
    playing: bool,
    title: String,
    artist: String,
    /// A friendly name for the app, e.g. "Spotify" or "Microsoft Edge".
    app: String,
    /// The album art as a data: URL, or empty.
    art: String,
    can_previous: bool,
    can_next: bool,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Toggle,
    Next,
    Previous,
}

enum Msg {
    Changed,
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

fn on_change<T, U>() -> TypedEventHandler<T, U>
where
    T: windows::core::RuntimeType + 'static,
    U: windows::core::RuntimeType + 'static,
{
    TypedEventHandler::new(|_, _| {
        signal(Msg::Changed);
        Ok(())
    })
}

fn run(app: &AppHandle, messages: Receiver<Msg>) -> windows::core::Result<()> {
    let manager = Manager::RequestAsync()?.join()?;
    manager.CurrentSessionChanged(&on_change())?;
    signal(Msg::Changed);

    let mut session: Option<Session> = None;
    let mut tokens = (0, 0);
    let mut art_key = String::new();
    let mut art = String::new();

    while let Ok(first) = messages.recv() {
        // Windows often sends several changes at once; act on the actions, refresh once.
        for msg in std::iter::once(first).chain(messages.try_iter()) {
            if let (Msg::Act(action), Some(s)) = (msg, &session) {
                let op = match action {
                    Action::Toggle => s.TryTogglePlayPauseAsync(),
                    Action::Next => s.TrySkipNextAsync(),
                    Action::Previous => s.TrySkipPreviousAsync(),
                };
                let _ = op.and_then(|op| op.join());
            }
        }

        let current = manager.GetCurrentSession().ok();
        if current != session {
            if let Some(old) = &session {
                let _ = old.RemoveMediaPropertiesChanged(tokens.0);
                let _ = old.RemovePlaybackInfoChanged(tokens.1);
            }
            if let Some(new) = &current {
                tokens = (new.MediaPropertiesChanged(&on_change())?, new.PlaybackInfoChanged(&on_change())?);
            }
            session = current;
        }

        let state = match &session {
            None => MediaState::default(),
            Some(s) => {
                let props = s.TryGetMediaPropertiesAsync().and_then(|op| op.join()).ok();
                let info = s.GetPlaybackInfo().ok();
                let controls = info.as_ref().and_then(|i| i.Controls().ok());
                let text = |f: fn(&Properties) -> windows::core::Result<windows::core::HSTRING>| {
                    props.as_ref().and_then(|p| f(p).ok()).map(|h| h.to_string()).unwrap_or_default()
                };
                let title = text(Properties::Title);
                let artist = text(Properties::Artist);
                let app_name = friendly_app(&s.SourceAppUserModelId().map(|h| h.to_string()).unwrap_or_default());
                // Album art is read once per track (and again while it is still missing).
                let key = format!("{app_name}\n{title}\n{artist}");
                if key != art_key || art.is_empty() {
                    art = props.as_ref().and_then(read_art).unwrap_or_default();
                    art_key = key;
                }
                MediaState {
                    active: !title.is_empty(),
                    playing: info.as_ref().and_then(|i| i.PlaybackStatus().ok()) == Some(Status::Playing),
                    title,
                    artist,
                    app: app_name,
                    art: art.clone(),
                    can_previous: controls.as_ref().and_then(|c| c.IsPreviousEnabled().ok()).unwrap_or(false),
                    can_next: controls.as_ref().and_then(|c| c.IsNextEnabled().ok()).unwrap_or(false),
                }
            }
        };

        let mut stored = STATE.lock().unwrap_or_else(|p| p.into_inner());
        if stored.as_ref() != Some(&state) {
            *stored = Some(state.clone());
            drop(stored);
            let _ = app.emit("media://changed", state);
        }
    }
    Ok(())
}

/// "Spotify.exe" → "Spotify", "MSEdge" → "Microsoft Edge", a Store app id → its name.
fn friendly_app(id: &str) -> String {
    let base = id.rsplit('!').next().unwrap_or(id);
    let base = base.split('_').next().unwrap_or(base);
    let base = base.strip_suffix(".exe").or_else(|| base.strip_suffix(".EXE")).unwrap_or(base);
    let base = base.rsplit('.').next().unwrap_or(base);
    match base.to_ascii_lowercase().as_str() {
        "msedge" => "Microsoft Edge".into(),
        "chrome" => "Chrome".into(),
        "firefox" => "Firefox".into(),
        "spotify" => "Spotify".into(),
        "zunemusic" | "media" => "Media Player".into(),
        _ => base.to_string(),
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

fn base64(bytes: &[u8]) -> String {
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
