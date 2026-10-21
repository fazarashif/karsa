//! Lyrics for what is playing, from LRCLIB (lrclib.net): community lyrics with timestamps,
//! free and without an API key. This is Karsa's only use of the internet, so it is off
//! until the user turns it on in Settings, and it sends nothing but the title, artist,
//! album and length of the track. Every answer, "not found" included, is cached in the
//! database, so a song is asked for once.

use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::db::{now_ms, Db};

pub const ENABLED: &str = "lyrics.enabled";
pub const FOCUS: &str = "lyrics.focus";

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Lyrics {
    /// The cache key, used to save the timing offset.
    key: String,
    /// "timed", "plain", "instrumental", "missing", "off" or "error".
    status: String,
    /// Timed lines: start in ms and the text.
    lines: Vec<(i64, String)>,
    plain: String,
    /// The user's timing nudge for this song, in ms.
    offset_ms: i64,
    /// Why the lookup failed, for "error".
    message: String,
}

impl Lyrics {
    fn status(key: &str, status: &str) -> Self {
        Lyrics { key: key.into(), status: status.into(), lines: Vec::new(), plain: String::new(), offset_ms: 0, message: String::new() }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    #[serde(default)]
    track_name: String,
    #[serde(default)]
    artist_name: String,
    #[serde(default)]
    synced_lyrics: Option<String>,
    #[serde(default)]
    plain_lyrics: Option<String>,
    #[serde(default)]
    instrumental: bool,
    #[serde(default)]
    duration: Option<f64>,
}

pub fn enabled(db: &Db) -> bool {
    db.setting(ENABLED).as_deref() == Some("true")
}

/// Letters and digits only, lowercase: "Guy Sebastian" and "guysebastian" compare equal.
fn norm(value: &str) -> String {
    value.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

fn similar(a: &str, b: &str) -> bool {
    let (a, b) = (norm(a), norm(b));
    !a.is_empty() && !b.is_empty() && (a == b || a.contains(&b) || b.contains(&a))
}

/// The first of several artists ("A, B", "A & B", "A x B", "A feat. B").
fn first_artist(artist: &str) -> String {
    let lower = artist.to_ascii_lowercase();
    let end = [", ", " & ", " x ", " feat", " ft.", " ft ", " and ", " with "]
        .iter()
        .filter_map(|sep| lower.find(sep))
        .min()
        .unwrap_or(artist.len());
    artist[..end].trim().to_string()
}

/// Drops what a video title adds to the song's name: bracketed extras such as
/// "(Official Video)", "[Lyrics]" or "(feat. X)", a trailing "| Official Music Video",
/// and "ft. X" after the title. Searching is ASCII-only, so byte offsets stay valid.
fn strip_extras(title: &str) -> String {
    const WORDS: [&str; 16] = [
        "official", "lyric", "audio", "video", "visuali", "hd", "4k", "hq", "mv", "m/v", "remaster", "explicit", "clean", "feat", "ft.", "color coded",
    ];
    let mut title = title.trim().to_string();
    // Bracketed segments that hold any of the words.
    let mut i = 0;
    while let Some(open) = title[i..].find(['(', '[']).map(|o| o + i) {
        let close_char = if title[open..].starts_with('(') { ')' } else { ']' };
        let Some(close) = title[open..].find(close_char).map(|c| c + open) else { break };
        let inside = title[open + 1..close].to_ascii_lowercase();
        if WORDS.iter().any(|w| inside.contains(w)) {
            title = format!("{}{}", title[..open].trim_end(), &title[close + 1..]);
            i = open.min(title.len());
        } else {
            i = close + 1;
        }
    }
    // "Song | Official Music Video", "Song // Lyrics"
    for sep in [" | ", " // "] {
        if let Some(at) = title.find(sep) {
            title.truncate(at);
        }
    }
    // "Song ft. X", "Song feat X"
    let lower = title.to_ascii_lowercase();
    if let Some(at) = [" ft. ", " ft ", " feat. ", " feat ", " featuring "].iter().filter_map(|w| lower.find(w)).min() {
        title.truncate(at);
    }
    title.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// What to ask LRCLIB for: the song's title and artist, and, for a video titled
/// "A - B", the other reading in case the channel put the song first.
struct Query {
    title: String,
    artist: String,
    swapped: Option<(String, String)>,
    /// A plain video (no album): its length often differs from the song's.
    video: bool,
}

/// "Guy Sebastian - Battle Scars (Official Video) ft. Lupe Fiasco" from the channel
/// "guysebastianVEVO" becomes title "Battle Scars", artist "Guy Sebastian".
fn clean(title: &str, artist: &str, album: &str) -> Query {
    let mut channel = artist.trim().to_string();
    for suffix in [" - Topic", "VEVO", "Vevo", " Official", "Official"] {
        if let Some(stripped) = channel.strip_suffix(suffix) {
            channel = stripped.trim().to_string();
        }
    }
    let song = strip_extras(title);
    let video = album.trim().is_empty();
    if video {
        for sep in [" - ", " – ", " — ", " : "] {
            if let Some((left, right)) = song.split_once(sep) {
                let (left, right) = (strip_extras(left), strip_extras(right));
                // The channel names the artist when it matches one side; else "Artist - Song".
                let (a, t) = if similar(&right, &channel) && !similar(&left, &channel) { (right, left) } else { (left, right) };
                let swapped = Some((a.clone(), t.clone()));
                return Query { title: t, artist: a, swapped, video };
            }
        }
    }
    Query { title: song, artist: channel, swapped: None, video }
}

fn cache_key(title: &str, artist: &str) -> String {
    format!("{}\n{}", norm(artist), norm(title))
}

fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

enum Reply {
    Body(String),
    NotFound,
}

/// One GET through Windows' own HTTP stack (so a company proxy is used). LRCLIB turns
/// away bursts with 503 or 429, so those are retried twice after a pause.
fn get(url: &str) -> Result<Reply, String> {
    use windows::core::HSTRING;
    use windows::Foundation::Uri;
    use windows::Web::Http::HttpClient;
    for attempt in 0..3 {
        let fetch = || -> windows::core::Result<(i32, String)> {
            let client = HttpClient::new()?;
            let _ = client.DefaultRequestHeaders()?.UserAgent()?.TryParseAdd(&HSTRING::from(format!(
                "Karsa/{} (https://github.com/fazarashif/karsa)",
                env!("CARGO_PKG_VERSION")
            )));
            let response = client.GetAsync(&Uri::CreateUri(&HSTRING::from(url))?)?.join()?;
            let status = response.StatusCode()?.0;
            let body = response.Content()?.ReadAsStringAsync()?.join()?.to_string();
            Ok((status, body))
        };
        match fetch() {
            Ok((200, body)) => return Ok(Reply::Body(body)),
            Ok((404, _)) => return Ok(Reply::NotFound),
            Ok((429 | 503, _)) if attempt < 2 => std::thread::sleep(std::time::Duration::from_millis(1500 * (attempt + 1))),
            Ok((429 | 503, _)) => return Err("lrclib.net is busy right now. Try again in a moment.".into()),
            Ok(_) | Err(_) => return Err("Karsa could not reach lrclib.net".into()),
        }
    }
    Err("lrclib.net is busy right now. Try again in a moment.".into())
}

/// The best search result: the same song by the same artist, of about the same length
/// (a video can run 30 s longer than the song), timed lyrics first.
fn pick(records: Vec<Record>, title: &str, artist: &str, duration_s: i64, video: bool) -> Option<Record> {
    let tolerance = if video { 30.0 } else { 6.0 };
    let lead = first_artist(artist);
    let mut matches: Vec<Record> = records
        .into_iter()
        .filter(|r| similar(&r.track_name, title))
        .filter(|r| lead.is_empty() || similar(&r.artist_name, &lead))
        .filter(|r| duration_s == 0 || r.duration.is_none_or(|d| (d - duration_s as f64).abs() <= tolerance))
        .collect();
    matches.sort_by(|a, b| {
        let gap = |r: &Record| r.duration.map(|d| (d - duration_s as f64).abs()).unwrap_or(f64::MAX);
        (a.synced_lyrics.is_none(), gap(a)).partial_cmp(&(b.synced_lyrics.is_none(), gap(b))).unwrap_or(std::cmp::Ordering::Equal)
    });
    matches.into_iter().next()
}

fn search(params: &str, title: &str, artist: &str, duration_s: i64, video: bool) -> Result<Option<Record>, String> {
    // A short pause between requests keeps LRCLIB from turning Karsa away.
    std::thread::sleep(std::time::Duration::from_millis(250));
    match get(&format!("https://lrclib.net/api/search?{params}"))? {
        Reply::Body(body) => Ok(pick(serde_json::from_str(&body).unwrap_or_default(), title, artist, duration_s, video)),
        Reply::NotFound => Ok(None),
    }
}

/// Asks LRCLIB: the exact match first for a music app (which reports the album and the
/// song's length), then searches by title and artist, the other reading of a video
/// title, and the title alone.
fn lookup(q: &Query, album: &str, duration_s: i64) -> Result<Option<Record>, String> {
    if !q.video && duration_s > 0 {
        let url = format!(
            "https://lrclib.net/api/get?track_name={}&artist_name={}&album_name={}&duration={duration_s}",
            encode(&q.title),
            encode(&q.artist),
            encode(album)
        );
        if let Reply::Body(body) = get(&url)? {
            if let Ok(record) = serde_json::from_str::<Record>(&body) {
                return Ok(Some(record));
            }
        }
    }
    let by = |title: &str, artist: &str| format!("track_name={}&artist_name={}", encode(title), encode(&first_artist(artist)));
    if let Some(found) = search(&by(&q.title, &q.artist), &q.title, &q.artist, duration_s, q.video)? {
        return Ok(Some(found));
    }
    if let Some((artist, title)) = &q.swapped {
        // `swapped` holds the reading already tried; try it the other way round.
        let (title, artist) = if title == &q.title { (artist.as_str(), title.as_str()) } else { (title.as_str(), artist.as_str()) };
        if let Some(found) = search(&by(title, artist), title, artist, duration_s, q.video)? {
            return Ok(Some(found));
        }
    }
    search(&format!("track_name={}", encode(&q.title)), &q.title, &q.artist, duration_s, q.video)
}

/// "[01:23.45] text" lines, sorted by time.
fn parse_lrc(lrc: &str) -> Vec<(i64, String)> {
    let mut lines = Vec::new();
    for raw in lrc.lines() {
        let mut rest = raw.trim();
        let mut stamps = Vec::new();
        while let Some(stripped) = rest.strip_prefix('[') {
            let Some(end) = stripped.find(']') else { break };
            let stamp = &stripped[..end];
            if let Some((m, s)) = stamp.split_once(':') {
                if let (Ok(m), Ok(s)) = (m.parse::<i64>(), s.parse::<f64>()) {
                    stamps.push(m * 60_000 + (s * 1000.0).round() as i64);
                }
            }
            rest = stripped[end + 1..].trim_start();
        }
        for at in stamps {
            lines.push((at, rest.to_string()));
        }
    }
    lines.sort_by_key(|(at, _)| *at);
    lines
}

fn from_cache(db: &Db, key: &str) -> Option<Lyrics> {
    db.conn()
        .query_row("SELECT synced, plain, found, offset_ms, fetched_at FROM lyrics WHERE key = ?1", [key], |r| {
            let synced: Option<String> = r.get(0)?;
            let plain: Option<String> = r.get(1)?;
            let found: i64 = r.get(2)?;
            let offset: i64 = r.get(3)?;
            let fetched: i64 = r.get(4)?;
            Ok((synced, plain, found, offset, fetched))
        })
        .optional()
        .ok()
        .flatten()
        // A "not found" is asked again after three days: LRCLIB keeps growing.
        .filter(|(_, _, found, _, fetched)| *found != 0 || now_ms() - fetched < 3 * 86_400_000)
        .map(|(synced, plain, found, offset, _)| {
            let mut lyrics = Lyrics::status(key, "missing");
            lyrics.offset_ms = offset;
            match (found, synced, plain) {
                (2, _, _) => lyrics.status = "instrumental".into(),
                (_, Some(s), _) if !s.trim().is_empty() => {
                    lyrics.status = "timed".into();
                    lyrics.lines = parse_lrc(&s);
                }
                (_, _, Some(p)) if !p.trim().is_empty() => {
                    lyrics.status = "plain".into();
                    lyrics.plain = p;
                }
                _ => {}
            }
            lyrics
        })
}

/// The lyrics for a track, from the cache or LRCLIB. `again` asks LRCLIB even when the
/// cache says "not found" ("Search again").
#[tauri::command]
pub async fn lyrics_get(app: AppHandle, title: String, artist: String, album: String, duration_ms: i64, again: bool) -> Result<Lyrics, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let db = app.state::<Db>();
        let query = clean(&title, &artist, &album);
        let key = cache_key(&query.title, &query.artist);
        if !enabled(&db) {
            return Ok(Lyrics::status(&key, "off"));
        }
        if query.title.is_empty() {
            return Ok(Lyrics::status(&key, "missing"));
        }
        if let Some(cached) = from_cache(&db, &key) {
            if !again || cached.status != "missing" {
                return Ok(cached);
            }
        }
        unsafe {
            let _ = windows::Win32::System::Com::CoInitializeEx(None, windows::Win32::System::Com::COINIT_MULTITHREADED);
        }
        let record = match lookup(&query, &album, (duration_ms / 1000).max(0)) {
            Ok(record) => record,
            Err(message) => {
                let mut lyrics = Lyrics::status(&key, "error");
                lyrics.message = message;
                return Ok(lyrics);
            }
        };
        let (synced, plain, found) = match &record {
            Some(r) if r.instrumental => (None, None, 2),
            Some(r) => (r.synced_lyrics.clone(), r.plain_lyrics.clone(), 1),
            None => (None, None, 0),
        };
        db.conn()
            .execute(
                "INSERT INTO lyrics (key, synced, plain, found, fetched_at) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(key) DO UPDATE SET synced = excluded.synced, plain = excluded.plain,
                   found = excluded.found, fetched_at = excluded.fetched_at",
                params![key, synced, plain, found, now_ms()],
            )
            .map_err(|e| e.to_string())?;
        Ok(from_cache(&db, &key).unwrap_or_else(|| Lyrics::status(&key, "missing")))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// The timing nudge for one song, remembered.
#[tauri::command]
pub fn lyrics_offset(db: tauri::State<'_, Db>, key: String, offset_ms: i64) -> Result<(), String> {
    db.conn()
        .execute("UPDATE lyrics SET offset_ms = ?2 WHERE key = ?1", params![key, offset_ms.clamp(-30_000, 30_000)])
        .map(drop)
        .map_err(|e| e.to_string())
}
