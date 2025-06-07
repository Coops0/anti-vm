use anyhow::Context;
use chrono::{DateTime, Utc};
use windows::Win32::{
    Foundation::{FILETIME, SYSTEMTIME},
    System::Time::SystemTimeToFileTime,
};
use windows_registry::{Key, USERS};

// TODO HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\InstallDate
// or TODO https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/win32-operatingsystem?redirectedfrom=MSDN
pub fn days_since_installation() -> Option<i64> {
    let oldest = USERS
        .keys()
        .ok()?
        .filter_map(|name| try_get_logon_starts(&name, USERS).ok())
        .min()?;

    Some(Utc::now().signed_duration_since(oldest).num_days())
}

fn try_get_logon_starts(name: &str, root: &Key) -> anyhow::Result<DateTime<Utc>> {
    let user_root = root.open(name)?;
    let key =
        user_root.open("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\LogonStats")?;

    let mut flt = None;

    if let Ok(first_logon_time) = key.get_value("FirstLogonTime") {
        let wide = first_logon_time.as_wide();
        let time = parse_system_time(wide.try_into()?)?;
        flt = Some(time);
    }

    if let Ok(first_logon_time_on_current_installation) =
        key.get_value("FirstLogonTimeOnCurrentInstallation")
    {
        let wide = first_logon_time_on_current_installation.as_wide();
        let time = parse_system_time(wide.try_into()?)?;
        // This is more useful
        flt = Some(time);
    }

    flt.context("failed to find logon time")
}

fn parse_system_time(bytes: [u16; 8]) -> anyhow::Result<DateTime<Utc>> {
    let system_time = SYSTEMTIME {
        wYear: bytes[0],
        wMonth: bytes[1],
        wDayOfWeek: bytes[2],
        wDay: bytes[3],
        wHour: bytes[4],
        wMinute: bytes[5],
        wSecond: bytes[6],
        wMilliseconds: bytes[7],
    };

    let mut file_time = FILETIME::default();
    unsafe {
        SystemTimeToFileTime(&system_time, &mut file_time)?;
    }

    const WINDOWS_TICK: u64 = 10_000_000;
    const SEC_TO_UNIX_EPOCH: u64 = 11_644_473_600;

    let (high, low) = (
        file_time.dwHighDateTime as u64,
        file_time.dwLowDateTime as u64,
    );

    let windows_ticks = high << 32 | low;
    let unix_time = (windows_ticks / WINDOWS_TICK) - SEC_TO_UNIX_EPOCH;

    DateTime::from_timestamp(unix_time as i64, 0)
        .context("failed to convert system time to DateTime")
}
