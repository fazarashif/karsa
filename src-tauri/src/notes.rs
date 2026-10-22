//! Notes and meeting notes, stored as Markdown files in a folder the user owns, so they
//! open in any editor (Obsidian, VS Code) and can sync through OneDrive.
//!
//! Plain notes are just text, titled by their first line. Meeting notes carry a small
//! YAML header (type, title, date, topic, attendees) and live in `Meetings/`. File names
//! follow the title (and the meeting date), and are renamed when those change.
//!
//! Projects are the folders directly inside the notes folder (all but `Meetings` and
//! hidden ones), laid out the same way: `Project/Title.md` and
//! `Project/Meetings/YYYY-MM-DD Title.md`. A folder made in File Explorer is a project too.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::mem::zeroed;
use std::path::{Component, Path, PathBuf};
use std::sync::RwLock;
use std::time::UNIX_EPOCH;

use rusqlite::params;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};
use windows_sys::Win32::UI::Shell::{
    SHFileOperationW, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT, FO_DELETE, SHFILEOPSTRUCTW,
};

use crate::db::Db;
use crate::localtime::local_to_ms;

/// The folder holding the notes. Settings can point it elsewhere while Karsa runs.
pub struct NotesFolder(RwLock<PathBuf>);

impl NotesFolder {
    pub fn new(path: PathBuf) -> Self {
        Self(RwLock::new(path))
    }

    pub fn get(&self) -> PathBuf {
        self.0.read().unwrap_or_else(|poisoned| poisoned.into_inner()).clone()
    }

    pub fn set(&self, path: PathBuf) {
        *self.0.write().unwrap_or_else(|poisoned| poisoned.into_inner()) = path;
    }
}

/// Largest file read for listing and searching; anything bigger is not a note.
const MAX_NOTE_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteSummary {
    /// Relative to the notes folder, with forward slashes.
    path: String,
    kind: &'static str,
    title: String,
    snippet: String,
    topic: String,
    /// The meeting's date, or when the file last changed.
    date: i64,
    open_actions: u32,
    done_actions: u32,
    /// The Outlook meeting a meeting note was started from (its `outlook:` key), or "".
    outlook: String,
    /// The project (folder) it is in, or "" for none.
    project: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteFile {
    path: String,
    content: String,
    modified: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Saved {
    path: String,
    modified: i64,
}

/// `settings.notes.folder`, or `<KARSA_DATA_DIR>/Notes` for test instances, or
/// `Documents\Karsa\Notes`.
pub fn resolve_folder(app: &AppHandle, db: &Db) -> PathBuf {
    if let Some(folder) = db.setting(crate::settings::NOTES_FOLDER).filter(|f| !f.trim().is_empty()) {
        return PathBuf::from(folder);
    }
    if let Some(data) = std::env::var_os("KARSA_DATA_DIR") {
        return PathBuf::from(data).join("Notes");
    }
    app.path()
        .document_dir()
        .map(|d| d.join("Karsa").join("Notes"))
        .unwrap_or_else(|_| PathBuf::from("Notes"))
}

// ---- Parsing --------------------------------------------------------------

/// Splits a YAML header (`---` lines) from the body. Values may be plain or JSON-quoted;
/// that subset is all Karsa writes.
fn front_matter(text: &str) -> (HashMap<String, String>, &str) {
    let mut fields = HashMap::new();
    let Some(rest) = text.strip_prefix("---\n").or_else(|| text.strip_prefix("---\r\n")) else {
        return (fields, text);
    };
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        offset += line.len();
        let line = line.trim_end_matches(['\r', '\n']);
        if line == "---" {
            return (fields, &rest[offset..]);
        }
        if let Some((key, value)) = line.split_once(':') {
            fields.insert(key.trim().to_string(), unquote(value));
        }
    }
    (HashMap::new(), text) // no closing line: not a header after all
}

fn unquote(value: &str) -> String {
    let value = value.trim();
    if value.starts_with('"') {
        serde_json::from_str(value).unwrap_or_else(|_| value.trim_matches('"').to_string())
    } else {
        value.to_string()
    }
}

fn clean_line(line: &str) -> &str {
    line.trim().trim_start_matches('#').trim_start_matches(['-', '*', ' ']).trim()
}

fn truncate(text: &str, max: usize) -> String {
    let mut out: String = text.chars().take(max).collect();
    if text.chars().count() > max {
        out.push('…');
    }
    out
}

/// "2026-10-19T14:00" (local) to Unix ms.
fn parse_local(value: &str) -> Option<i64> {
    let (date, time) = value.split_once('T').unwrap_or((value, "00:00"));
    let mut d = date.split('-').map(|p| p.parse::<u16>().ok());
    let mut t = time.split(':').map(|p| p.parse::<u16>().ok());
    Some(local_to_ms(d.next()??, d.next()??, d.next()??, t.next().flatten().unwrap_or(0), t.next().flatten().unwrap_or(0)))
}

struct Parsed {
    meeting: bool,
    title: String,
    snippet: String,
    topic: String,
    date: Option<i64>,
    date_key: Option<String>,
    open_actions: u32,
    done_actions: u32,
    outlook: String,
}

fn parse(text: &str) -> Parsed {
    let (fields, body) = front_matter(text);
    let meeting = fields.get("type").is_some_and(|t| t == "meeting");
    let mut lines = body.lines().map(clean_line).filter(|l| !l.is_empty());
    let (title, snippet) = if meeting {
        let title = fields.get("title").cloned().unwrap_or_default();
        let snippet = lines.find(|l| !l.eq_ignore_ascii_case("notes")).unwrap_or_default().to_string();
        (title, snippet)
    } else {
        (lines.next().unwrap_or_default().to_string(), lines.next().unwrap_or_default().to_string())
    };
    let (mut open_actions, mut done_actions) = (0, 0);
    if meeting {
        for line in body.lines().map(str::trim) {
            if line.starts_with("- [ ]") {
                open_actions += 1;
            } else if line.starts_with("- [x]") || line.starts_with("- [X]") {
                done_actions += 1;
            }
        }
    }
    let date_value = fields.get("date").cloned();
    Parsed {
        meeting,
        title: truncate(if title.is_empty() { "Untitled" } else { &title }, 120),
        snippet: truncate(&snippet, 120),
        topic: truncate(fields.get("topic").map(String::as_str).unwrap_or(""), 120),
        date: date_value.as_deref().and_then(parse_local),
        date_key: date_value.map(|d| d.chars().take(10).collect()),
        open_actions,
        done_actions,
        outlook: if meeting { fields.get("outlook").cloned().unwrap_or_default() } else { String::new() },
    }
}

// ---- Files ------------------------------------------------------------------

/// Turns a relative note path from the UI into a path inside the folder, refusing
/// anything that could escape it.
fn resolve(folder: &Path, rel: &str) -> Result<PathBuf, String> {
    let rel_path = Path::new(rel);
    let ok = !rel.is_empty()
        && rel_path.components().all(|c| matches!(c, Component::Normal(_)))
        && rel_path.extension().is_some_and(|e| e.eq_ignore_ascii_case("md"));
    if ok {
        Ok(folder.join(rel_path))
    } else {
        Err("Invalid note path".into())
    }
}

fn relative(folder: &Path, path: &Path) -> String {
    path.strip_prefix(folder).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

/// The project a note is in: the folder it sits in under the notes folder, or "" for notes
/// at the top (and in the top `Meetings` folder).
fn project_of(rel: &str) -> String {
    match rel.split_once('/') {
        Some((first, _)) if !first.eq_ignore_ascii_case("Meetings") => first.to_string(),
        _ => String::new(),
    }
}

/// A project name the UI sent, as a folder name; "" is no project (the notes folder).
fn checked_project(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Ok(String::new());
    }
    if name.starts_with('.') || name.eq_ignore_ascii_case("Meetings") || file_stem(name) != name {
        return Err("Pick another name: without \\ / : * ? \" < > |, and not Meetings".into());
    }
    Ok(name.to_string())
}

fn project_dir(folder: &Path, project: &str) -> PathBuf {
    if project.is_empty() {
        folder.to_path_buf()
    } else {
        folder.join(project)
    }
}

/// The projects: folders directly inside the notes folder, but not `Meetings` or hidden ones.
fn projects(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(folder)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| !name.starts_with('.') && !name.eq_ignore_ascii_case("Meetings"))
        .collect();
    names.sort_by_key(|name| name.to_lowercase());
    names
}

/// Moves a file or folder to the Recycle Bin, so a mistaken delete can be undone.
fn recycle(path: &Path) -> bool {
    let from: Vec<u16> = path.as_os_str().encode_wide_null2();
    unsafe {
        let mut op: SHFILEOPSTRUCTW = zeroed();
        op.wFunc = FO_DELETE as _;
        op.pFrom = from.as_ptr();
        op.fFlags = (FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_SILENT | FOF_NOERRORUI) as _;
        SHFileOperationW(&mut op) == 0
    }
}

fn modified_ms(path: &Path) -> i64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// A file name Windows accepts, from a title.
fn file_stem(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| if c.is_control() || r#"<>:"/\|?*"#.contains(c) { ' ' } else { c })
        .collect();
    let mut stem = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    stem = stem.chars().take(80).collect::<String>().trim_matches(['.', ' ']).to_string();
    const RESERVED: [&str; 22] = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1",
        "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if stem.is_empty() {
        "Untitled".to_string()
    } else if RESERVED.iter().any(|r| r.eq_ignore_ascii_case(&stem)) {
        format!("{stem}_")
    } else {
        stem
    }
}

/// Where a note with this content belongs in a project's folder (or the notes folder):
/// `Title.md`, or `Meetings/YYYY-MM-DD Title.md`.
fn desired_path(folder: &Path, parsed: &Parsed) -> PathBuf {
    let stem = file_stem(&parsed.title);
    if parsed.meeting {
        let name = match &parsed.date_key {
            Some(day) if day.len() == 10 => format!("{day} {stem}.md"),
            _ => format!("{stem}.md"),
        };
        folder.join("Meetings").join(name)
    } else {
        folder.join(format!("{stem}.md"))
    }
}

/// Same file? Windows paths ignore case, and the UI's relative paths use `/` where
/// joined paths use `\`.
fn same_path(a: &Path, b: &Path) -> bool {
    let norm = |p: &Path| p.to_string_lossy().replace('/', "\\").to_lowercase();
    norm(a) == norm(b)
}

/// The desired path, or "Name 2.md", "Name 3.md"… if another file has it.
fn unique_path(desired: PathBuf, current: Option<&Path>) -> PathBuf {
    let taken = |p: &Path| p.exists() && current.is_none_or(|c| !same_path(c, p));
    if !taken(&desired) {
        return desired;
    }
    let stem = desired.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let dir = desired.parent().map(Path::to_path_buf).unwrap_or_default();
    (2..1000)
        .map(|n| dir.join(format!("{stem} {n}.md")))
        .find(|p| !taken(p))
        .unwrap_or(desired)
}

/// Writes through a temporary file, so a crash never leaves half a note behind.
fn write_atomic(path: &Path, content: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("md.tmp");
    fs::write(&tmp, content)?;
    fs::rename(&tmp, path)
}

fn collect(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let hidden = entry.file_name().to_string_lossy().starts_with('.');
        if path.is_dir() && !hidden && depth > 0 {
            collect(&path, depth - 1, out);
        } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("md")) && out.len() < 5000 {
            out.push(path);
        }
    }
}

pub fn notify(app: &AppHandle) {
    let _ = app.emit("notes://changed", ());
}

// ---- Commands -------------------------------------------------------------

/// All notes, newest first; with a query, only those whose title or text contains it.
#[tauri::command]
pub async fn notes_list(folder: State<'_, NotesFolder>, query: Option<String>) -> Result<Vec<NoteSummary>, String> {
    let folder = &folder.get();
    let needle = query.map(|q| q.trim().to_lowercase()).filter(|q| !q.is_empty());
    let mut files = Vec::new();
    collect(folder, 3, &mut files);
    let mut notes: Vec<NoteSummary> = files
        .into_iter()
        .filter(|p| fs::metadata(p).map(|m| m.len() <= MAX_NOTE_BYTES).unwrap_or(false))
        .filter_map(|path| {
            let text = fs::read_to_string(&path).ok()?;
            if let Some(needle) = &needle {
                if !text.to_lowercase().contains(needle.as_str()) {
                    return None;
                }
            }
            let parsed = parse(&text);
            Some(NoteSummary {
                path: relative(folder, &path),
                kind: if parsed.meeting { "meeting" } else { "note" },
                date: parsed.date.unwrap_or_else(|| modified_ms(&path)),
                title: parsed.title,
                snippet: parsed.snippet,
                topic: parsed.topic,
                open_actions: parsed.open_actions,
                done_actions: parsed.done_actions,
                outlook: parsed.outlook,
                project: project_of(&relative(folder, &path)),
            })
        })
        .collect();
    notes.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(notes)
}

#[tauri::command]
pub fn notes_read(folder: State<'_, NotesFolder>, path: String) -> Result<NoteFile, String> {
    let full = resolve(&folder.get(), &path)?;
    let content = fs::read_to_string(&full).map_err(|_| "This note no longer exists".to_string())?;
    Ok(NoteFile { modified: modified_ms(&full), path, content })
}

/// Saves a note and renames its file to follow the title (and meeting date). With
/// `base_modified`, refuses ("conflict") if the file changed on disk since it was read,
/// unless `force` is set. A new note goes into `project` ("" or none: no project); a
/// saved one stays in its own.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn notes_save(
    app: AppHandle,
    folder: State<'_, NotesFolder>,
    db: State<'_, Db>,
    path: Option<String>,
    content: String,
    base_modified: Option<i64>,
    force: bool,
    project: Option<String>,
) -> Result<Saved, String> {
    let folder = &folder.get();
    let current = path.as_deref().map(|p| resolve(folder, p)).transpose()?;
    if let (Some(current), Some(base), false) = (&current, base_modified, force) {
        if current.exists() && modified_ms(current) != base {
            return Err("conflict".into());
        }
    }
    let project = match (&path, project) {
        (Some(rel), _) => project_of(rel),
        (None, Some(name)) => checked_project(&name)?,
        (None, None) => String::new(),
    };
    let target = unique_path(desired_path(&project_dir(folder, &project), &parse(&content)), current.as_deref());
    write_atomic(&target, &content).map_err(|e| format!("Could not save the note: {e}"))?;
    let new_rel = relative(folder, &target);
    if let (Some(current), Some(old_rel)) = (&current, &path) {
        if !same_path(current, &target) {
            let _ = fs::remove_file(current);
            // Tasks added from this note keep pointing at it.
            let _ = db.conn().execute("UPDATE tasks SET note_path = ?1 WHERE note_path = ?2", params![new_rel, old_rel]);
        }
    }
    notify(&app);
    Ok(Saved { modified: modified_ms(&target), path: new_rel })
}

/// Moves a note to the Recycle Bin, so a mistaken delete can be undone.
#[tauri::command]
pub fn notes_delete(app: AppHandle, folder: State<'_, NotesFolder>, path: String) -> Result<(), String> {
    let full = resolve(&folder.get(), &path)?;
    if !recycle(&full) {
        return Err("Could not move the note to the Recycle Bin".into());
    }
    notify(&app);
    Ok(())
}

/// Moves a note into another project's folder ("" for none), keeping its name unless one
/// there already has it. Tasks added from the note keep pointing at it.
#[tauri::command]
pub fn notes_move(
    app: AppHandle,
    folder: State<'_, NotesFolder>,
    db: State<'_, Db>,
    path: String,
    project: String,
) -> Result<Saved, String> {
    let folder = &folder.get();
    let current = resolve(folder, &path)?;
    let project = checked_project(&project)?;
    let content = fs::read_to_string(&current).map_err(|_| "This note no longer exists".to_string())?;
    let target = unique_path(desired_path(&project_dir(folder, &project), &parse(&content)), Some(&current));
    let new_rel = relative(folder, &target);
    if !same_path(&current, &target) {
        if let Some(dir) = target.parent() {
            fs::create_dir_all(dir).map_err(|e| format!("Could not move the note: {e}"))?;
        }
        fs::rename(&current, &target).map_err(|e| format!("Could not move the note: {e}"))?;
        let _ = db.conn().execute("UPDATE tasks SET note_path = ?1 WHERE note_path = ?2", params![new_rel, path]);
    }
    notify(&app);
    Ok(Saved { modified: modified_ms(&target), path: new_rel })
}

#[tauri::command]
pub fn notes_projects(folder: State<'_, NotesFolder>) -> Vec<String> {
    projects(&folder.get())
}

/// Makes a project: a new folder in the notes folder.
#[tauri::command]
pub fn notes_project_create(app: AppHandle, folder: State<'_, NotesFolder>, name: String) -> Result<Vec<String>, String> {
    let folder = folder.get();
    let name = checked_project(&name)?;
    if name.is_empty() {
        return Err("Name the project".into());
    }
    if projects(&folder).iter().any(|p| p.eq_ignore_ascii_case(&name)) {
        return Err("There is already a project with this name".into());
    }
    fs::create_dir_all(folder.join(&name)).map_err(|e| format!("Could not make the folder: {e}"))?;
    notify(&app);
    Ok(projects(&folder))
}

/// Renames a project's folder. Tasks added from its notes keep pointing at them.
#[tauri::command]
pub fn notes_project_rename(
    app: AppHandle,
    folder: State<'_, NotesFolder>,
    db: State<'_, Db>,
    name: String,
    new_name: String,
) -> Result<Vec<String>, String> {
    let folder = folder.get();
    let (from, to) = (checked_project(&name)?, checked_project(&new_name)?);
    if from.is_empty() || to.is_empty() {
        return Err("Name the project".into());
    }
    if from != to {
        // Only a change of case may reuse an existing name: its own.
        if !from.eq_ignore_ascii_case(&to) && projects(&folder).iter().any(|p| p.eq_ignore_ascii_case(&to)) {
            return Err("There is already a project with this name".into());
        }
        fs::rename(folder.join(&from), folder.join(&to)).map_err(|e| format!("Could not rename the folder: {e}"))?;
        let (old_prefix, new_prefix) = (format!("{from}/"), format!("{to}/"));
        let _ = db.conn().execute(
            "UPDATE tasks SET note_path = ?1 || substr(note_path, ?2) WHERE substr(note_path, 1, ?3) = ?4",
            params![new_prefix, old_prefix.chars().count() as i64 + 1, old_prefix.chars().count() as i64, old_prefix],
        );
        notify(&app);
    }
    Ok(projects(&folder))
}

/// Deletes an empty project (its folder goes to the Recycle Bin). One with notes in it
/// is refused: its notes are moved or deleted first.
#[tauri::command]
pub fn notes_project_delete(app: AppHandle, folder: State<'_, NotesFolder>, name: String) -> Result<Vec<String>, String> {
    let folder = folder.get();
    let name = checked_project(&name)?;
    if name.is_empty() {
        return Err("Name the project".into());
    }
    let dir = folder.join(&name);
    let mut inside = Vec::new();
    collect(&dir, 3, &mut inside);
    if !inside.is_empty() {
        return Err("Move or delete its notes first".into());
    }
    if dir.exists() && !recycle(&dir) {
        return Err("Could not move the folder to the Recycle Bin".into());
    }
    notify(&app);
    Ok(projects(&folder))
}

/// Shows the note's file selected in File Explorer.
#[tauri::command]
pub fn notes_reveal(folder: State<'_, NotesFolder>, path: String) -> Result<(), String> {
    let full = resolve(&folder.get(), &path)?;
    std::process::Command::new("explorer")
        .arg(format!("/select,{}", full.display()))
        .spawn()
        .map(drop)
        .map_err(|e| e.to_string())
}

/// Opens a note in its own resizable window (or focuses it if already open). The window
/// is created on demand and destroyed when closed, so it costs nothing afterwards.
#[tauri::command]
pub async fn notes_open_window(app: AppHandle, path: String, title: String) -> Result<(), String> {
    let mut hasher = DefaultHasher::new();
    path.to_lowercase().hash(&mut hasher);
    let label = format!("note-{:016x}", hasher.finish());
    if let Some(window) = app.get_webview_window(&label) {
        let _ = window.unminimize();
        return window.set_focus().map_err(|e| e.to_string());
    }
    let url = format!("index.html#note={}", percent_encode(&path));
    let mut builder = WebviewWindowBuilder::new(&app, &label, WebviewUrl::App(url.into()))
        .title(format!("{title} — Karsa"))
        .inner_size(720.0, 640.0)
        .min_inner_size(420.0, 360.0)
        .theme(Some(tauri::Theme::Dark))
        .background_color(tauri::window::Color(0, 0, 0, 255));
    // Every webview in the app must share the same browser arguments.
    if let Some(args) = app.config().app.windows.first().and_then(|w| w.additional_browser_args.clone()) {
        builder = builder.additional_browser_args(&args);
    }
    builder.build().map(drop).map_err(|e| e.to_string())
}

fn percent_encode(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

trait WideNull2 {
    /// UTF-16 with two terminating NULs, as SHFileOperation's file lists expect.
    fn encode_wide_null2(&self) -> Vec<u16>;
}

impl WideNull2 for std::ffi::OsStr {
    fn encode_wide_null2(&self) -> Vec<u16> {
        use std::os::windows::ffi::OsStrExt;
        self.encode_wide().chain([0, 0]).collect()
    }
}
