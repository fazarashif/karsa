//! The file shelf: files dropped on the island, kept as links to the originals until the
//! user removes them. They can be opened, shown in Explorer, copied (then pasted as files
//! anywhere) or dragged straight out to another app.

use std::path::{Path, PathBuf};

use rusqlite::params;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::db::{now_ms, Db};
use crate::island;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShelfItem {
    id: i64,
    path: String,
    name: String,
    folder: bool,
    /// Bytes, for files that still exist.
    size: Option<u64>,
    /// False when the original was moved or deleted.
    exists: bool,
    added_at: i64,
}

fn notify(app: &AppHandle) {
    let _ = app.emit("shelf://changed", ());
}

fn paths_of(db: &Db, ids: &[i64]) -> Result<Vec<PathBuf>, String> {
    let conn = db.conn();
    let mut paths = Vec::new();
    for id in ids {
        let path: String = conn
            .query_row("SELECT path FROM shelf WHERE id = ?1", [id], |r| r.get(0))
            .map_err(|_| "This item is no longer on the shelf".to_string())?;
        let path = PathBuf::from(path);
        if !path.exists() {
            return Err(format!("{} was moved or deleted", path.file_name().unwrap_or_default().to_string_lossy()));
        }
        paths.push(path);
    }
    Ok(paths)
}

/// Newest first.
#[tauri::command]
pub fn shelf_list(db: State<'_, Db>) -> Result<Vec<ShelfItem>, String> {
    let conn = db.conn();
    let mut stmt = conn.prepare("SELECT id, path, added_at FROM shelf ORDER BY added_at DESC, id DESC").map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?)))
        .map_err(|e| e.to_string())?;
    Ok(rows
        .filter_map(Result::ok)
        .map(|(id, path, added_at)| {
            let meta = std::fs::metadata(&path).ok();
            let name = Path::new(&path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| path.clone());
            ShelfItem {
                id,
                name,
                folder: meta.as_ref().is_some_and(|m| m.is_dir()),
                size: meta.as_ref().filter(|m| m.is_file()).map(|m| m.len()),
                exists: meta.is_some(),
                path,
                added_at,
            }
        })
        .collect())
}

/// Puts files on the shelf (again at the top, if already there). Returns how many.
fn add_paths(app: &AppHandle, paths: &[String]) -> Result<usize, String> {
    let now = now_ms();
    let mut added = 0;
    {
        let db = app.state::<Db>();
        let conn = db.conn();
        for path in paths.iter().filter(|p| Path::new(p).is_absolute() && Path::new(p).exists()) {
            conn.execute(
                "INSERT INTO shelf (path, added_at) VALUES (?1, ?2)
                 ON CONFLICT(path) DO UPDATE SET added_at = excluded.added_at",
                params![path, now],
            )
            .map_err(|e| e.to_string())?;
            added += 1;
        }
    }
    notify(app);
    Ok(added)
}

#[tauri::command]
pub fn shelf_add(app: AppHandle, paths: Vec<String>) -> Result<usize, String> {
    add_paths(&app, &paths)
}

/// Files copied in Explorer (Ctrl+C), pasted onto the shelf. Returns how many.
#[tauri::command]
pub fn shelf_paste(app: AppHandle) -> Result<usize, String> {
    use windows_sys::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard};
    use windows_sys::Win32::System::Ole::CF_HDROP;
    let paths = unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return Err("Another app is using the clipboard; try again".into());
        }
        let handle = GetClipboardData(CF_HDROP as u32);
        let paths = if handle.is_null() { Vec::new() } else { hdrop_paths(handle as _) };
        CloseClipboard();
        paths
    };
    if paths.is_empty() {
        return Err("No files on the clipboard. Copy files in Explorer first".into());
    }
    add_paths(&app, &paths)
}

/// "Add files…": the Windows file picker, several files at once. Returns how many.
#[tauri::command]
pub async fn shelf_pick(app: AppHandle) -> Result<usize, String> {
    let owner = island::handle();
    let paths = tauri::async_runtime::spawn_blocking(move || pick_files(owner)).await.map_err(|e| e.to_string())?;
    if paths.is_empty() {
        return Ok(0);
    }
    add_paths(&app, &paths)
}

/// The paths in an HDROP from the clipboard.
unsafe fn hdrop_paths(hdrop: windows_sys::Win32::UI::Shell::HDROP) -> Vec<String> {
    use windows_sys::Win32::UI::Shell::DragQueryFileW;
    let count = DragQueryFileW(hdrop, u32::MAX, std::ptr::null_mut(), 0);
    (0..count)
        .map(|i| {
            let len = DragQueryFileW(hdrop, i, std::ptr::null_mut(), 0) as usize;
            let mut buf = vec![0u16; len + 1];
            DragQueryFileW(hdrop, i, buf.as_mut_ptr(), buf.len() as u32);
            String::from_utf16_lossy(&buf[..len])
        })
        .collect()
}

fn pick_files(owner: isize) -> Vec<String> {
    use windows::core::w;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
        COINIT_DISABLE_OLE1DDE,
    };
    use windows::Win32::UI::Shell::{FileOpenDialog, IFileOpenDialog, FOS_ALLOWMULTISELECT, FOS_FILEMUSTEXIST, SIGDN_FILESYSPATH};

    unsafe {
        if CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).is_err() {
            return Vec::new();
        }
        let result = (|| -> windows::core::Result<Vec<String>> {
            let dialog: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
            dialog.SetOptions(dialog.GetOptions()? | FOS_ALLOWMULTISELECT | FOS_FILEMUSTEXIST)?;
            dialog.SetTitle(w!("Add files to the shelf"))?;
            dialog.SetOkButtonLabel(w!("Add to shelf"))?;
            dialog.Show(Some(HWND(owner as _)))?;
            let items = dialog.GetResults()?;
            let mut paths = Vec::new();
            for i in 0..items.GetCount()? {
                let name = items.GetItemAt(i)?.GetDisplayName(SIGDN_FILESYSPATH)?;
                if let Ok(path) = name.to_string() {
                    paths.push(path);
                }
                CoTaskMemFree(Some(name.0 as _));
            }
            Ok(paths)
        })();
        CoUninitialize();
        result.unwrap_or_default()
    }
}

/// What the island tells the UI once dropped files are on the shelf.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DragNote {
    kind: &'static str,
    count: usize,
    kept: usize,
}

/// What the page posts, with the dropped files attached, when files are dropped on the
/// island (`dropFiles` in src/lib/shelf.svelte.ts).
const DROP_MESSAGE: &str = "karsa:shelf-drop";

/// Files dropped on the island come in through WebView2, which accepts drops from
/// Explorer the way Edge does: the page posts the dropped `File`s with
/// `chrome.webview.postMessageWithAdditionalObjects`, and WebView2 gives this side their
/// paths. An OLE drop target on Karsa's own windows is never reached by a real drag:
/// the window under the cursor belongs to the WebView2 process, and Windows does not go
/// up from it to its parent in Karsa's process.
pub fn accept_drops(app: &AppHandle, window: &tauri::WebviewWindow) -> tauri::Result<()> {
    let app = app.clone();
    window.with_webview(move |webview| unsafe {
        use webview2_com::Microsoft::Web::WebView2::Win32::{
            ICoreWebView2File, ICoreWebView2WebMessageReceivedEventArgs2,
        };
        use webview2_com::{take_pwstr, WebMessageReceivedEventHandler};
        use windows::core::{Interface, PWSTR};

        let Ok(core) = webview.controller().CoreWebView2() else { return };
        let handler = WebMessageReceivedEventHandler::create(Box::new(move |_, args| {
            let Some(args) = args else { return Ok(()) };
            // Tauri's own IPC messages pass through here too.
            let mut message = PWSTR::null();
            if args.TryGetWebMessageAsString(&mut message).is_err() || take_pwstr(message) != DROP_MESSAGE {
                return Ok(());
            }
            let files = args.cast::<ICoreWebView2WebMessageReceivedEventArgs2>()?.AdditionalObjects()?;
            let mut count = 0;
            files.Count(&mut count)?;
            let mut paths = Vec::new();
            for i in 0..count {
                let Ok(file) = files.GetValueAtIndex(i)?.cast::<ICoreWebView2File>() else { continue };
                let mut path = PWSTR::null();
                if file.Path(&mut path).is_ok() {
                    paths.push(take_pwstr(path));
                }
            }
            let kept = add_paths(&app, &paths).unwrap_or(0);
            let _ = app.emit("shelf://drag", DragNote { kind: "drop", count: paths.len(), kept });
            Ok(())
        }));
        let mut token = 0;
        let _ = core.add_WebMessageReceived(&handler, &mut token);
    })
}

#[tauri::command]
pub fn shelf_remove(app: AppHandle, db: State<'_, Db>, id: i64) -> Result<(), String> {
    db.conn().execute("DELETE FROM shelf WHERE id = ?1", [id]).map_err(|e| e.to_string())?;
    notify(&app);
    Ok(())
}

#[tauri::command]
pub fn shelf_clear(app: AppHandle, db: State<'_, Db>) -> Result<(), String> {
    db.conn().execute("DELETE FROM shelf", []).map_err(|e| e.to_string())?;
    notify(&app);
    Ok(())
}

/// Opens the file with its default app (or the folder in Explorer).
#[tauri::command]
pub fn shelf_open(db: State<'_, Db>, id: i64) -> Result<(), String> {
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let path = paths_of(&db, &[id])?.remove(0);
    let file: Vec<u16> = path.as_os_str().to_string_lossy().encode_utf16().chain([0]).collect();
    let verb: Vec<u16> = "open\0".encode_utf16().collect();
    let result = unsafe {
        ShellExecuteW(std::ptr::null_mut(), verb.as_ptr(), file.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL)
    };
    // ShellExecute reports success with a value above 32.
    if result as isize > 32 {
        Ok(())
    } else {
        Err("Windows could not open this file".into())
    }
}

#[tauri::command]
pub fn shelf_reveal(db: State<'_, Db>, id: i64) -> Result<(), String> {
    let path = paths_of(&db, &[id])?.remove(0);
    std::process::Command::new("explorer")
        .arg(format!("/select,{}", path.display()))
        .spawn()
        .map(drop)
        .map_err(|e| e.to_string())
}

/// Puts the files on the clipboard, to paste into Explorer, an email or a chat.
#[tauri::command]
pub fn shelf_copy(app: AppHandle, db: State<'_, Db>, ids: Vec<i64>) -> Result<(), String> {
    let paths = paths_of(&db, &ids)?;
    // OLE clipboard calls belong on the UI thread, which has OLE initialised.
    app.run_on_main_thread(move || unsafe {
        if let Ok(data) = data_object(&paths) {
            let _ = windows::Win32::System::Ole::OleSetClipboard(&data);
        }
    })
    .map_err(|e| e.to_string())
}

/// Starts a real Windows drag of the files, called while the mouse button is still down
/// on a shelf item, so they can be dropped into any app.
#[tauri::command]
pub fn shelf_drag(app: AppHandle, db: State<'_, Db>, ids: Vec<i64>) -> Result<(), String> {
    let paths = paths_of(&db, &ids)?;
    app.run_on_main_thread(move || unsafe {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::System::Ole::{DROPEFFECT_COPY, DROPEFFECT_LINK};
        use windows::Win32::UI::Shell::SHDoDragDrop;
        if let Ok(data) = data_object(&paths) {
            let owner = HWND(island::handle() as _);
            let _ = SHDoDragDrop(Some(owner), &data, None, DROPEFFECT_COPY | DROPEFFECT_LINK);
        }
    })
    .map_err(|e| e.to_string())
}

/// A shell data object for the files, the same kind Explorer uses for drag and copy.
unsafe fn data_object(paths: &[PathBuf]) -> windows::core::Result<windows::Win32::System::Com::IDataObject> {
    use windows::core::{HSTRING, PCWSTR};
    use windows::Win32::System::Com::CoTaskMemFree;
    use windows::Win32::UI::Shell::Common::ITEMIDLIST;
    use windows::Win32::UI::Shell::{BHID_DataObject, IShellItemArray, SHCreateShellItemArrayFromIDLists, SHParseDisplayName};

    let mut pidls: Vec<*const ITEMIDLIST> = Vec::new();
    for path in paths {
        let name = HSTRING::from(path.as_os_str());
        let mut pidl: *mut ITEMIDLIST = std::ptr::null_mut();
        if SHParseDisplayName(PCWSTR(name.as_ptr()), None, &mut pidl, 0, None).is_ok() && !pidl.is_null() {
            pidls.push(pidl);
        }
    }
    let result = (|| {
        let items: IShellItemArray = SHCreateShellItemArrayFromIDLists(&pidls)?;
        items.BindToHandler(None, &BHID_DataObject)
    })();
    for pidl in pidls {
        CoTaskMemFree(Some(pidl as _));
    }
    result
}
