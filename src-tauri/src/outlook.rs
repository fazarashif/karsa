//! Outlook meetings, read from classic Outlook on this PC while it runs, through its COM
//! object model (late binding). Karsa only reads: nothing is written to Outlook, nothing is
//! stored in Karsa's database and nothing leaves the PC. Karsa never starts Outlook; while
//! Outlook is closed it keeps what it last read.
//!
//! One thread reads the calendar from the 1st of last month to the end of next month every
//! few minutes (and at once when asked), and fires a meeting alert `lead` minutes before
//! each meeting starts.

use std::collections::{HashMap, HashSet};
use std::mem::ManuallyDrop;
use std::ptr::null_mut;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Duration;

use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager};
use windows::core::{w, Interface, BSTR, GUID, PCWSTR};
use windows::Win32::Foundation::{SYSTEMTIME, VARIANT_BOOL};
use windows::Win32::System::Com::{
    CLSIDFromProgID, CoInitializeEx, IDispatch, COINIT_MULTITHREADED, DISPATCH_FLAGS, DISPATCH_METHOD,
    DISPATCH_PROPERTYGET, DISPATCH_PROPERTYPUT, DISPPARAMS,
};
use windows::Win32::System::Ole::{GetActiveObject, DISPID_PROPERTYPUT};
use windows::Win32::System::Variant::{
    VariantChangeType, VariantClear, VariantTimeToSystemTime, VARIANT, VAR_CHANGE_FLAGS, VT_BOOL, VT_BSTR, VT_DATE,
    VT_DISPATCH, VT_I4,
};
use windows_sys::Win32::Globalization::{GetDateFormatEx, GetTimeFormatEx, DATE_SHORTDATE, TIME_NOSECONDS};

use crate::db::{now_ms, Db};
use crate::localtime::{local_to_ms, to_local};
use crate::reminders::ReminderEngine;
use crate::ISLAND;

pub const ENABLED: &str = "outlook.enabled";
pub const LEAD: &str = "outlook.lead_min";
/// Minutes before a meeting that its alert goes off; -1 means no alerts.
const DEFAULT_LEAD_MIN: i64 = 10;
/// How often the calendar is read again while Outlook runs, and after a failed read.
const READ_EVERY_MS: i64 = 5 * 60_000;
const RETRY_MS: i64 = 60_000;
/// The longest Karsa sleeps, so it notices Outlook starting.
const IDLE_WAKE_MS: i64 = 5 * 60_000;
/// Names kept per meeting; a meeting note takes the first few.
const MAX_ATTENDEES: usize = 30;
/// Items read per pass at most, against a runaway calendar.
const MAX_ITEMS: usize = 1000;

// Outlook's own numbers (OlDefaultFolders, OlObjectClass, OlMeetingStatus, OlResponseStatus).
const OL_FOLDER_CALENDAR: i32 = 9;
const OL_APPOINTMENT: i32 = 26;
const OL_MEETING_CANCELED: i32 = 5;
const OL_MEETING_RECEIVED_AND_CANCELED: i32 = 7;
const OL_RESPONSE_DECLINED: i32 = 4;
const LOCALE_USER_DEFAULT: u32 = 0x0400;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Meeting {
    /// Stable for this meeting: a hash of Outlook's GlobalAppointmentID, plus the local
    /// date for an occurrence of a recurring one. A meeting note keeps it as `outlook:`.
    key: String,
    title: String,
    /// Unix ms.
    start: i64,
    end: i64,
    all_day: bool,
    location: String,
    organizer: String,
    /// Required attendees, then optional ones, without the user or the organizer.
    attendees: Vec<String>,
    /// All of them, when `attendees` was cut short.
    attendee_count: usize,
    /// The organizer canceled it, or the user declined it: shown, never alerted.
    canceled: bool,
    declined: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlookState {
    enabled: bool,
    /// "off", "reading", "ok", "closed" (Outlook is not running) or "error".
    status: &'static str,
    message: String,
    /// When the meetings were last read (Unix ms), or 0.
    read_at: i64,
    lead_min: i64,
    meetings: Vec<Meeting>,
}

#[derive(Default)]
struct Wake {
    dirty: bool,
    /// Read the calendar now instead of at the next scheduled read.
    read: bool,
}

pub struct Outlook {
    state: Mutex<OutlookState>,
    wake: Mutex<Wake>,
    signal: Condvar,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl OutlookState {
    fn off(lead_min: i64) -> Self {
        Self { enabled: false, status: "off", message: String::new(), read_at: 0, lead_min, meetings: Vec::new() }
    }
}

impl Outlook {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(OutlookState::off(DEFAULT_LEAD_MIN)),
            wake: Mutex::new(Wake::default()),
            signal: Condvar::new(),
        }
    }

    /// Settings changed or the UI became ready: look again (and read, if `read`).
    pub fn poke(&self, read: bool) {
        let mut wake = lock(&self.wake);
        wake.dirty = true;
        wake.read |= read;
        self.signal.notify_all();
    }

    fn snapshot(&self) -> OutlookState {
        lock(&self.state).clone()
    }
}

pub fn enabled(db: &Db) -> bool {
    db.setting(ENABLED).as_deref() == Some("true")
}

pub fn lead_min(db: &Db) -> i64 {
    db.setting(LEAD).and_then(|v| v.parse().ok()).unwrap_or(DEFAULT_LEAD_MIN).clamp(-1, 24 * 60)
}

pub fn spawn(app: AppHandle, outlook: Arc<Outlook>) -> std::io::Result<()> {
    std::thread::Builder::new().name("karsa-outlook".into()).spawn(move || {
        // Outlook runs in its own process, so its objects work from the multithreaded
        // apartment, which needs no message loop.
        let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        let mut next_read = 0;
        let mut fired: HashSet<String> = HashSet::new();
        loop {
            let db = app.state::<Db>();
            let (on, lead) = (enabled(&db), lead_min(&db));
            let now = now_ms();
            let read_now = std::mem::take(&mut lock(&outlook.wake).read);
            if !on {
                next_read = 0;
                fired.clear();
                let mut state = lock(&outlook.state);
                if state.enabled || state.lead_min != lead {
                    // Turned off: forget the meetings too.
                    *state = OutlookState::off(lead);
                    drop(state);
                    emit(&app, &outlook);
                }
            } else if read_now || now >= next_read {
                {
                    let mut state = lock(&outlook.state);
                    state.enabled = true;
                    state.lead_min = lead;
                    if state.read_at == 0 {
                        state.status = "reading";
                    }
                }
                let result = read_calendar(now);
                let mut state = lock(&outlook.state);
                next_read = now + READ_EVERY_MS;
                match result {
                    Ok(meetings) => {
                        state.status = "ok";
                        state.message.clear();
                        state.read_at = now;
                        state.meetings = meetings;
                    }
                    Err(Failure::Closed) => {
                        state.status = "closed";
                        state.message.clear();
                    }
                    Err(Failure::Error(message)) => {
                        state.status = "error";
                        state.message = message;
                        next_read = now + RETRY_MS;
                    }
                }
                drop(state);
                emit(&app, &outlook);
            } else if lock(&outlook.state).lead_min != lead {
                lock(&outlook.state).lead_min = lead;
                emit(&app, &outlook);
            }

            let ready = app.state::<Arc<ReminderEngine>>().is_ready();
            let next_alert = if on && lead >= 0 && ready { fire_due(&app, &outlook, lead, now, &mut fired) } else { None };
            let until = [Some(if on { next_read } else { now + IDLE_WAKE_MS }), next_alert]
                .into_iter()
                .flatten()
                .min()
                .unwrap_or(now + IDLE_WAKE_MS);
            let wait = Duration::from_millis((until - now).clamp(1_000, IDLE_WAKE_MS) as u64);
            let mut wake = lock(&outlook.wake);
            if !wake.dirty {
                wake = match outlook.signal.wait_timeout(wake, wait) {
                    Ok((guard, _)) => guard,
                    Err(poisoned) => poisoned.into_inner().0,
                };
            }
            wake.dirty = false;
        }
    })?;
    Ok(())
}

fn emit(app: &AppHandle, outlook: &Outlook) {
    let _ = app.emit_to(ISLAND, "outlook://changed", outlook.snapshot());
}

/// Alerts for meetings whose lead time has begun (once each, and only before they start).
/// Returns when the next alert is due.
fn fire_due(app: &AppHandle, outlook: &Outlook, lead: i64, now: i64, fired: &mut HashSet<String>) -> Option<i64> {
    let meetings = lock(&outlook.state).meetings.clone();
    let mut next = None;
    for m in meetings.iter().filter(|m| !m.all_day && !m.canceled && !m.declined && m.start > now) {
        // A moved meeting alerts again at its new time.
        let id = format!("{}@{}", m.key, m.start);
        if fired.contains(&id) {
            continue;
        }
        let at = m.start - lead * 60_000;
        if at <= now {
            fired.insert(id);
            let _ = app.emit_to(ISLAND, "island://alert", alert_for(m, now));
        } else {
            next = Some(next.map_or(at, |n: i64| n.min(at)));
        }
    }
    // Forget meetings long past, so the set stays small.
    fired.retain(|id| id.rsplit_once('@').and_then(|(_, at)| at.parse::<i64>().ok()).is_some_and(|at| at > now - 86_400_000));
    next
}

fn alert_for(m: &Meeting, now: i64) -> serde_json::Value {
    let minutes = (m.start - now + 30_000).div_euclid(60_000);
    let subtitle = match minutes {
        0 => "Meeting · now".to_string(),
        1..=59 => format!("Meeting · starts in {minutes} min"),
        _ if minutes % 60 == 0 => format!("Meeting · starts in {} h", minutes / 60),
        _ => format!("Meeting · starts in {} h {} min", minutes / 60, minutes % 60),
    };
    json!({ "title": m.title, "subtitle": subtitle, "kind": "meeting", "at": m.start, "outlook": m.key })
}

// ---- Reading Outlook --------------------------------------------------------

enum Failure {
    /// Outlook is not running.
    Closed,
    Error(String),
}

impl From<windows::core::Error> for Failure {
    fn from(e: windows::core::Error) -> Self {
        Failure::Error(format!("Outlook did not answer ({})", e.message()))
    }
}

/// The calendar from the 1st of last month to the end of next month, by start time.
fn read_calendar(now: i64) -> Result<Vec<Meeting>, Failure> {
    let app = unsafe {
        let clsid = CLSIDFromProgID(w!("Outlook.Application")).map_err(|_| Failure::Error("Classic Outlook is not installed".into()))?;
        let mut unknown: Option<windows::core::IUnknown> = None;
        // Only a running Outlook: GetActiveObject never starts one.
        if GetActiveObject(&clsid, None, &mut unknown).is_err() {
            return Err(Failure::Closed);
        }
        unknown.ok_or(Failure::Closed)?.cast::<IDispatch>()?
    };
    let mut d = Dispatcher::default();
    let ns = d.call(&app, "GetNamespace", vec![Var::str("MAPI")])?.object()?;
    let me = d
        .get(&ns, "CurrentUser")
        .ok()
        .and_then(|user| user.dispatch())
        .and_then(|user| d.get(&user, "Name").ok())
        .map(|name| name.string())
        .unwrap_or_default();
    let folder = d.call(&ns, "GetDefaultFolder", vec![Var::long(OL_FOLDER_CALENDAR)])?.object()?;
    let items = d.get(&folder, "Items")?.object()?;
    // In this order, so recurring meetings come as their occurrences (Outlook's own advice).
    d.call(&items, "Sort", vec![Var::str("[Start]")])?;
    d.put(&items, "IncludeRecurrences", Var::bool(true))?;
    let (from, to) = window(now);
    let filter = format!("[Start] < '{}' AND [End] > '{}'", outlook_date(to), outlook_date(from));
    let found = d.call(&items, "Restrict", vec![Var::str(&filter)])?.object()?;

    let mut meetings = Vec::new();
    let mut item = d.call(&found, "GetFirst", vec![])?;
    while let Some(appt) = item.dispatch() {
        if meetings.len() >= MAX_ITEMS {
            break;
        }
        if d.get(&appt, "Class")?.int() == OL_APPOINTMENT {
            meetings.push(d.meeting(&appt, &me)?);
        }
        item = d.call(&found, "GetNext", vec![])?;
    }
    Ok(meetings)
}

/// Local midnight on the 1st of last month, and on the 1st of the month after next.
fn window(now: i64) -> (i64, i64) {
    let t = to_local(now);
    let month = |offset: i32| {
        let index = t.wYear as i32 * 12 + t.wMonth as i32 - 1 + offset;
        local_to_ms((index / 12) as u16, (index % 12 + 1) as u16, 1, 0, 0)
    };
    (month(-1), month(2))
}

/// A date and time as Outlook's filters expect it: in the user's short date and time format.
fn outlook_date(ms: i64) -> String {
    let t = to_local(ms);
    let mut date = [0u16; 64];
    let mut time = [0u16; 64];
    unsafe {
        let d = GetDateFormatEx(std::ptr::null(), DATE_SHORTDATE, &t, std::ptr::null(), date.as_mut_ptr(), 64, std::ptr::null());
        let h = GetTimeFormatEx(std::ptr::null(), TIME_NOSECONDS, &t, std::ptr::null(), time.as_mut_ptr(), 64);
        let text = |buf: &[u16], len: i32| String::from_utf16_lossy(&buf[..(len.max(1) - 1) as usize]);
        format!("{} {}", text(&date, d), text(&time, h))
    }
}

/// A 64-bit FNV-1a hash in hex: a short, stable stand-in for Outlook's long ids.
fn short_hash(text: &str) -> String {
    let hash = text.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3));
    format!("{hash:016x}")
}

/// "Alex Doe; Sam Lee" to names.
fn names(list: &str) -> impl Iterator<Item = String> + '_ {
    list.split(';').map(str::trim).filter(|n| !n.is_empty()).map(String::from)
}

// ---- Late binding ----------------------------------------------------------------

/// Calls by name, remembering each name's DISPID: every Outlook item shares one interface,
/// so asking once saves a round trip to Outlook per property per item.
#[derive(Default)]
struct Dispatcher {
    ids: HashMap<&'static str, i32>,
}

impl Dispatcher {
    fn id(&mut self, object: &IDispatch, name: &'static str) -> windows::core::Result<i32> {
        if let Some(&id) = self.ids.get(name) {
            return Ok(id);
        }
        let wide: Vec<u16> = name.encode_utf16().chain([0]).collect();
        let names = [PCWSTR(wide.as_ptr())];
        let mut id = 0;
        unsafe { object.GetIDsOfNames(&GUID::zeroed(), names.as_ptr(), 1, LOCALE_USER_DEFAULT, &mut id)? };
        self.ids.insert(name, id);
        Ok(id)
    }

    fn invoke(&mut self, object: &IDispatch, name: &'static str, flags: DISPATCH_FLAGS, args: Vec<Var>) -> windows::core::Result<Var> {
        let id = self.id(object, name)?;
        // DISPPARAMS lists the arguments last first.
        let mut args: Vec<Var> = args.into_iter().rev().collect();
        let mut put = DISPID_PROPERTYPUT;
        let is_put = flags == DISPATCH_PROPERTYPUT;
        let params = DISPPARAMS {
            rgvarg: args.as_mut_ptr().cast(),
            rgdispidNamedArgs: if is_put { &mut put } else { null_mut() },
            cArgs: args.len() as u32,
            cNamedArgs: is_put as u32,
        };
        let mut result = Var::default();
        let out = if is_put { None } else { Some(&mut result.0 as *mut VARIANT) };
        unsafe { object.Invoke(id, &GUID::zeroed(), LOCALE_USER_DEFAULT, flags, &params, out, None, None)? };
        Ok(result)
    }

    fn call(&mut self, object: &IDispatch, name: &'static str, args: Vec<Var>) -> windows::core::Result<Var> {
        self.invoke(object, name, DISPATCH_METHOD, args)
    }

    fn get(&mut self, object: &IDispatch, name: &'static str) -> windows::core::Result<Var> {
        self.invoke(object, name, DISPATCH_PROPERTYGET, vec![])
    }

    fn put(&mut self, object: &IDispatch, name: &'static str, value: Var) -> windows::core::Result<()> {
        self.invoke(object, name, DISPATCH_PROPERTYPUT, vec![value]).map(drop)
    }

    fn meeting(&mut self, appt: &IDispatch, me: &str) -> Result<Meeting, Failure> {
        let text = |d: &mut Self, name| d.get(appt, name).map(|v| v.string());
        let start = self.get(appt, "Start")?.time().unwrap_or(0);
        let gid = text(self, "GlobalAppointmentID")?;
        let id = if gid.is_empty() { text(self, "EntryID")? } else { gid };
        let mut key = short_hash(&id);
        if self.get(appt, "IsRecurring")?.boolean() {
            let t = to_local(start);
            key = format!("{key}-{:04}{:02}{:02}", t.wYear, t.wMonth, t.wDay);
        }
        let organizer = text(self, "Organizer")?;
        let mut attendees: Vec<String> = names(&text(self, "RequiredAttendees")?)
            .chain(names(&text(self, "OptionalAttendees")?))
            .filter(|n| !n.eq_ignore_ascii_case(me) && !n.eq_ignore_ascii_case(&organizer))
            .collect();
        let attendee_count = attendees.len();
        attendees.truncate(MAX_ATTENDEES);
        let status = self.get(appt, "MeetingStatus")?.int();
        Ok(Meeting {
            key,
            title: text(self, "Subject")?.trim().to_string(),
            start,
            end: self.get(appt, "End")?.time().unwrap_or(start),
            all_day: self.get(appt, "AllDayEvent")?.boolean(),
            location: text(self, "Location")?.trim().to_string(),
            organizer: if organizer.eq_ignore_ascii_case(me) { String::new() } else { organizer },
            attendees,
            attendee_count,
            canceled: status == OL_MEETING_CANCELED || status == OL_MEETING_RECEIVED_AND_CANCELED,
            declined: self.get(appt, "ResponseStatus")?.int() == OL_RESPONSE_DECLINED,
        })
    }
}

/// A VARIANT that is cleared when dropped.
#[repr(transparent)]
#[derive(Default)]
struct Var(VARIANT);

impl Drop for Var {
    fn drop(&mut self) {
        let _ = unsafe { VariantClear(&mut self.0) };
    }
}

impl Var {
    fn str(value: &str) -> Self {
        let mut v = Var::default();
        unsafe {
            let inner = &mut *v.0.Anonymous.Anonymous;
            inner.vt = VT_BSTR;
            inner.Anonymous.bstrVal = ManuallyDrop::new(BSTR::from(value));
        }
        v
    }

    fn long(value: i32) -> Self {
        let mut v = Var::default();
        unsafe {
            let inner = &mut *v.0.Anonymous.Anonymous;
            inner.vt = VT_I4;
            inner.Anonymous.lVal = value;
        }
        v
    }

    fn bool(value: bool) -> Self {
        let mut v = Var::default();
        unsafe {
            let inner = &mut *v.0.Anonymous.Anonymous;
            inner.vt = VT_BOOL;
            inner.Anonymous.boolVal = VARIANT_BOOL(if value { -1 } else { 0 });
        }
        v
    }

    fn vt(&self) -> u16 {
        unsafe { self.0.Anonymous.Anonymous.vt.0 }
    }

    fn dispatch(&self) -> Option<IDispatch> {
        if self.vt() != VT_DISPATCH.0 {
            return None;
        }
        unsafe { (*self.0.Anonymous.Anonymous.Anonymous.pdispVal).clone() }
    }

    fn object(&self) -> Result<IDispatch, Failure> {
        self.dispatch().ok_or_else(|| Failure::Error("Outlook gave an unexpected answer".into()))
    }

    /// Any value as text ("" when empty).
    fn string(&self) -> String {
        if self.vt() == VT_BSTR.0 {
            return unsafe { self.0.Anonymous.Anonymous.Anonymous.bstrVal.to_string() };
        }
        self.converted(VT_BSTR.0).map(|v| v.string()).unwrap_or_default()
    }

    fn int(&self) -> i32 {
        if self.vt() == VT_I4.0 {
            return unsafe { self.0.Anonymous.Anonymous.Anonymous.lVal };
        }
        self.converted(VT_I4.0).map(|v| v.int()).unwrap_or(0)
    }

    fn boolean(&self) -> bool {
        self.vt() == VT_BOOL.0 && unsafe { self.0.Anonymous.Anonymous.Anonymous.boolVal.0 } != 0
    }

    /// An Outlook date (local wall-clock time) as Unix ms.
    fn time(&self) -> Option<i64> {
        if self.vt() != VT_DATE.0 {
            return None;
        }
        let mut t = SYSTEMTIME::default();
        let ok = unsafe { VariantTimeToSystemTime(self.0.Anonymous.Anonymous.Anonymous.date, &mut t) };
        (ok != 0).then(|| local_to_ms(t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute))
    }

    fn converted(&self, vt: u16) -> Option<Var> {
        if self.vt() == vt {
            return None;
        }
        let mut out = Var::default();
        unsafe { VariantChangeType(&mut out.0, &self.0, VAR_CHANGE_FLAGS(0), windows::Win32::System::Variant::VARENUM(vt)) }
            .ok()
            .map(|_| out)
    }
}

// ---- Commands --------------------------------------------------------------

#[tauri::command]
pub fn outlook_state(outlook: tauri::State<'_, Arc<Outlook>>) -> OutlookState {
    outlook.snapshot()
}

/// Reads the calendar again now (the Calendar tab asks when it opens).
#[tauri::command]
pub fn outlook_refresh(outlook: tauri::State<'_, Arc<Outlook>>) {
    outlook.poke(true);
}
