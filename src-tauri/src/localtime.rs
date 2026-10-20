//! Local wall-clock helpers on top of Windows' time zone rules, so repeating reminders
//! keep their local time ("every day at 09:00") across daylight-saving changes.

use std::mem::zeroed;
use std::ptr::null;

use windows_sys::Win32::Foundation::{FILETIME, SYSTEMTIME};
use windows_sys::Win32::System::Time::{
    FileTimeToSystemTime, SystemTimeToFileTime, SystemTimeToTzSpecificLocalTime, TzSpecificLocalTimeToSystemTime,
};

/// Milliseconds between 1601-01-01 (FILETIME epoch) and 1970-01-01 (Unix epoch).
const EPOCH_DIFF_MS: i64 = 11_644_473_600_000;
const DAY_100NS: i64 = 24 * 60 * 60 * 10_000_000;

fn to_filetime(ticks: i64) -> FILETIME {
    FILETIME { dwLowDateTime: ticks as u32, dwHighDateTime: (ticks >> 32) as u32 }
}

fn from_filetime(ft: FILETIME) -> i64 {
    ((ft.dwHighDateTime as i64) << 32) | ft.dwLowDateTime as i64
}

/// Unix ms to local calendar fields (wDayOfWeek: 0 = Sunday).
pub fn to_local(ms: i64) -> SYSTEMTIME {
    unsafe {
        let mut utc: SYSTEMTIME = zeroed();
        FileTimeToSystemTime(&to_filetime((ms + EPOCH_DIFF_MS) * 10_000), &mut utc);
        let mut local: SYSTEMTIME = zeroed();
        SystemTimeToTzSpecificLocalTime(null(), &utc, &mut local);
        local
    }
}

/// Local calendar fields back to Unix ms.
fn from_local(local: &SYSTEMTIME) -> i64 {
    unsafe {
        let mut utc: SYSTEMTIME = zeroed();
        TzSpecificLocalTimeToSystemTime(null(), local, &mut utc);
        let mut ft: FILETIME = zeroed();
        SystemTimeToFileTime(&utc, &mut ft);
        from_filetime(ft) / 10_000 - EPOCH_DIFF_MS
    }
}

/// A local date and time to Unix ms.
pub fn local_to_ms(year: u16, month: u16, day: u16, hour: u16, minute: u16) -> i64 {
    let local = SYSTEMTIME {
        wYear: year,
        wMonth: month,
        wDayOfWeek: 0,
        wDay: day,
        wHour: hour,
        wMinute: minute,
        wSecond: 0,
        wMilliseconds: 0,
    };
    from_local(&local)
}

/// The same local time of day, `days` days later.
pub fn add_local_days(ms: i64, days: i64) -> i64 {
    unsafe {
        let local = to_local(ms);
        let mut ft: FILETIME = zeroed();
        SystemTimeToFileTime(&local, &mut ft);
        let mut shifted: SYSTEMTIME = zeroed();
        FileTimeToSystemTime(&to_filetime(from_filetime(ft) + days * DAY_100NS), &mut shifted);
        from_local(&shifted)
    }
}

pub fn is_weekend(ms: i64) -> bool {
    matches!(to_local(ms).wDayOfWeek, 0 | 6)
}

/// "15:00", or "3:00 PM" when Windows uses a 12-hour clock.
pub fn format_time(ms: i64) -> String {
    let t = to_local(ms);
    if crate::island::uses_12_hour_clock() {
        let hour = match t.wHour % 12 {
            0 => 12,
            h => h,
        };
        format!("{hour}:{:02} {}", t.wMinute, if t.wHour < 12 { "AM" } else { "PM" })
    } else {
        format!("{:02}:{:02}", t.wHour, t.wMinute)
    }
}
