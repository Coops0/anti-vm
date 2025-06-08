use windows::Devices::Enumeration::{DeviceInformation, DeviceInformationKind};
use windows_core::HSTRING;

pub fn inspect<T: std::fmt::Debug>(name: &str, value: T) -> T {
    println!("{name}: {value:?}");
    value
}

pub fn get_devices_iter(
    selector: &HSTRING,
) -> anyhow::Result<impl Iterator<Item = DeviceInformation> + 'static + use<>>  {
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

#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!($($arg)*);
    };
}
