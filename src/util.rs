use windows::Devices::Enumeration::{DeviceInformation, DeviceInformationKind};
use windows_core::HSTRING;

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        std::println!($($arg)*);
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {};
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! inspect {
    ($type: literal, $name:literal, $value:expr) => {{
        use std::fmt::Write;

        let before = std::time::Instant::now();
        let value = $value;
        let t = before.elapsed().as_millis();

        let t_str = if t != 0 {
            format!(" (took {t}ms)")
        } else {
            String::new()
        };

        let value_str = format!("{:?}", value);

        let mut prefix = String::new();

        if $type == "outer" {
            let _ = write!(
                &mut prefix,
                "{}{}@ {}",
                $crate::util::colors::DARK_CYAN,
                $crate::util::colors::BOLD,
                $crate::util::colors::DEFAULT
            );
        }

        // https://learn.microsoft.com/en-us/windows/win32/com/com-error-codes-1
        if value_str.contains("0x800") && !value_str.contains("0x80004002") {
            let _ = write!(
                &mut prefix,
                "{}!!! ERROR: {}",
                $crate::util::colors::DARK_RED,
                $crate::util::colors::DEFAULT
            );
        }

        if t > 40 {
            let _ = write!(
                &mut prefix,
                "{}SLOW CHECK > {}",
                $crate::util::colors::DARK_YELLOW,
                $crate::util::colors::DEFAULT
            );
        }

        let location = if cfg!(debug_assertions) {
            format!(" ({}:{}:{})", file!(), line!(), column!())
        } else {
            String::new()
        };

        $crate::debug_println!("{prefix}{}: {value_str}{t_str}{location}", $name);
        value
    }};
    (inner, $name:literal, $value:expr) => {
        inspect!("inner", $name, $value)
    };
    ($name:literal, $value:expr) => {
        inspect!("outer", $name, $value)
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! inspect {
    (inner, $name:literal, $value:expr) => {
        $value
    };
    ($name:literal, $value:expr) => {
        $value
    };
}

pub fn get_devices_iter(
    selector: &HSTRING,
) -> anyhow::Result<impl Iterator<Item = DeviceInformation>> {
    let devices_collection =
        DeviceInformation::FindAllAsyncWithKindAqsFilterAndAdditionalProperties(
            selector,
            None,
            DeviceInformationKind::DeviceInterface,
        )?
        .get()?;
    let devices_size = devices_collection.Size()? as usize;

    let mut devices = vec![None; devices_size];
    let fetched_size = if devices_size != 0 {
        devices_collection.GetMany(0, &mut devices)? as usize
    } else {
        0
    };

    let devices_iter = devices.into_iter().take(fetched_size).flatten();
    Ok(devices_iter)
}

#[cfg(debug_assertions)]
#[allow(dead_code)]
pub mod colors {
    pub const DEFAULT: &str = "\x1b[0m";
    pub const BLACK: &str = "\x1b[30m";
    pub const BG_BLACK: &str = "\x1b[40m";
    pub const DARK_RED: &str = "\x1b[31m";
    pub const BG_DARK_RED: &str = "\x1b[41m";
    pub const DARK_GREEN: &str = "\x1b[32m";
    pub const BG_DARK_GREEN: &str = "\x1b[42m";
    pub const DARK_YELLOW: &str = "\x1b[33m";
    pub const BG_DARK_YELLOW: &str = "\x1b[43m";
    pub const DARK_BLUE: &str = "\x1b[34m";
    pub const BG_DARK_BLUE: &str = "\x1b[44m";
    pub const DARK_MAGENTA: &str = "\x1b[35m";
    pub const BG_DARK_MAGENTA: &str = "\x1b[45m";
    pub const DARK_CYAN: &str = "\x1b[36m";
    pub const BG_DARK_CYAN: &str = "\x1b[46m";
    pub const DARK_WHITE: &str = "\x1b[37m";
    pub const BG_DARK_WHITE: &str = "\x1b[47m";
    pub const BRIGHT_BLACK: &str = "\x1b[90m";
    pub const BG_BRIGHT_BLACK: &str = "\x1b[100m";
    pub const BRIGHT_RED: &str = "\x1b[91m";
    pub const BG_BRIGHT_RED: &str = "\x1b[101m";
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    pub const BG_BRIGHT_GREEN: &str = "\x1b[102m";
    pub const BRIGHT_YELLOW: &str = "\x1b[93m";
    pub const BG_BRIGHT_YELLOW: &str = "\x1b[103m";
    pub const BRIGHT_BLUE: &str = "\x1b[94m";
    pub const BG_BRIGHT_BLUE: &str = "\x1b[104m";
    pub const BRIGHT_MAGENTA: &str = "\x1b[95m";
    pub const BG_BRIGHT_MAGENTA: &str = "\x1b[105m";
    pub const BRIGHT_CYAN: &str = "\x1b[96m";
    pub const BG_BRIGHT_CYAN: &str = "\x1b[106m";
    pub const WHITE: &str = "\x1b[97m";
    pub const BG_WHITE: &str = "\x1b[107m";
    pub const BOLD: &str = "\x1b[1m";
    pub const UNDERLINE: &str = "\x1b[4m";
    pub const NO_UNDERLINE: &str = "\x1b[24m";
    pub const REVERSE_TEXT: &str = "\x1b[7m";
    pub const POSITIVE_TEXT: &str = "\x1b[27m";
}
