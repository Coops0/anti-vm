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
    ($name:literal, $value:expr) => {{
        // let before = std::time::Instant::now();
        let value = $value;
        // let t = before.elapsed().as_millis();

        // let t = if t != 0 {
        //     format!(" (took {t}ms)")
        // } else {
        //     String::new()
        // };

        // let location = if cfg!(debug_assertions) {
        //     format!(" ({}:{}:{})", file!(), line!(), column!())
        // } else {
        //     String::new()
        // };

        // println!( "{}: {:?}{t}{location}\n", $name, value);
        // $crate::debug_println!("{}: {:?}{t}{location}\n", $name, value);
        value
    }}
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! inspect {
    ($name:literal, $value:expr) => {
        $value
    }
}


pub fn get_devices_iter(
    selector: &HSTRING,
) -> anyhow::Result<impl Iterator<Item = DeviceInformation> + 'static + use<>> {
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