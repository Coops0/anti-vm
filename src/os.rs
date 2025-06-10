use anyhow::Context;
use chrono::{DateTime, FixedOffset, Utc};
use serde::Deserialize;
use windows::Win32::{
    Foundation::{FILETIME, SYSTEMTIME},
    System::Time::SystemTimeToFileTime,
};
use windows_registry::{Key, USERS};
use wmi::{COMLibrary, WMIConnection};

use crate::flags::Flags;

pub fn score_os(flags: &mut Flags) -> anyhow::Result<()> {
    let registry_date = get_registry_days_since_installation()?.date_naive();
    let wmi_date = get_wmi_os_stats()?.to_utc().date_naive();

    let installations_diff = wmi_date
        .signed_duration_since(registry_date)
        .abs()
        .num_days();

    println!("the installations differ by {installations_diff} days");
    if installations_diff > 2 {
        flags.large_penalty();
    }

    let days_since_installation = registry_date
        .min(wmi_date)
        .signed_duration_since(Utc::now().date_naive())
        .num_days();

    println!("days since installation: {days_since_installation}");

    match days_since_installation {
        0 => flags.extreme_penalty(),
        1..=6 => flags.large_penalty(),
        // Okay...
        7..=60 => {}
        _ => flags.small_bonus(),
    }

    Ok(())
}

fn get_registry_days_since_installation() -> anyhow::Result<DateTime<Utc>> {
    USERS
        .keys()?
        .filter_map(|name| try_get_registry_logon_stats(&name, USERS).ok())
        .min()
        .context("nm")
}

fn try_get_registry_logon_stats(name: &str, root: &Key) -> anyhow::Result<DateTime<Utc>> {
    let user_root = root.open(name)?;
    let key =
        user_root.open("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\LogonStats")?;

    let mut flt = None;

    if let Ok(first_logon_time) = key.get_value("FirstLogonTime") {
        let wide = first_logon_time.as_wide();
        let time = parse_registry_system_time(wide.try_into()?)?;
        flt = Some(time);
    }

    if let Ok(first_logon_time_on_current_installation) =
        key.get_value("FirstLogonTimeOnCurrentInstallation")
    {
        let wide = first_logon_time_on_current_installation.as_wide();
        let time = parse_registry_system_time(wide.try_into()?)?;
        // This is more useful
        flt = Some(time);
    }

    flt.context("failed to find logon time")
}

fn parse_registry_system_time(bytes: [u16; 8]) -> anyhow::Result<DateTime<Utc>> {
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

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Win32OperatingSystem {
    caption: String,       // Microsoft Windows 11 Pro
    name: String,          // Microsoft Windows 11 Pro|C:\\WINDOWS|\\Device\\Harddisk0\\Partition3
    install_date: String,  // 20240704035336.000000-240
    serial_number: String, // TODO 00330-80000-00000-AA359
    os_type: u16, // https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/win32-operatingsystem?redirectedfrom=MSDN#examples
}

fn get_wmi_os_stats() -> anyhow::Result<DateTime<FixedOffset>> {
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con.into())?;

    let results =
        wmi_con.raw_query::<Win32OperatingSystem>("SELECT * FROM Win32_OperatingSystem")?;

    let os = results.first().context("nf")?;

    parse_wmi_install_date(&os.install_date)
}

// 20240704035336.000000-240
fn parse_wmi_install_date(date_str: &str) -> anyhow::Result<DateTime<FixedOffset>> {
    let (datetime_part, tz_part) = date_str.rsplit_once('-').context("ns")?;

    let tz_minutes: i32 = tz_part.parse()?;
    let tz_hours = tz_minutes / 60;
    let tz_mins = tz_minutes % 60;
    let tz_formatted = format!("-{:02}:{:02}", tz_hours, tz_mins);

    let formatted_date = format!("{}{}", datetime_part, tz_formatted);

    DateTime::parse_from_str(&formatted_date, "%Y%m%d%H%M%S%.f%z").map_err(Into::into)
}
