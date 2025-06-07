
use windows::Devices::Enumeration::{DeviceInformation, DeviceInformationKind};
use windows_core::HSTRING;

pub fn inspect<T: std::fmt::Debug>(name: &str, value: T) -> T {
    println!("{name}: {value:?}");
    value
}

pub fn get_devices_iter(
    selector: &HSTRING,
) -> anyhow::Result<impl Iterator<Item = DeviceInformation>> {
    let devices_collection = DeviceInformation::FindAllAsyncAqsFilter(selector)?.get()?;
    let devices_size = devices_collection.Size()? as usize;

    let mut devices = vec![None; devices_size];
    if devices_size != 0 {
        devices_collection.GetMany(0, &mut devices)?;
    }

    let devices_iter = devices
        .into_iter()
        .flatten()
        .filter(|device| device.Kind() == Ok(DeviceInformationKind::DeviceInterface));

    Ok(devices_iter)
}
