use windows::Devices::{Bluetooth::BluetoothAdapter, Enumeration::DeviceInformation};

use crate::{flags::Flags, util::get_devices_iter};

pub fn score_bluetooth_adapters(flags: &mut Flags) -> anyhow::Result<()> {
    let selector = BluetoothAdapter::GetDeviceSelector()?;

    let bluetooth_adapter_count = get_devices_iter(&selector)?
        .filter(|device| is_valid_bluetooth_adapter(device).unwrap_or_default())
        .count();

    match bluetooth_adapter_count {
        0 => flags.medium_penalty(),
        _ => flags.large_bonus(),
    }

    Ok(())
}

fn is_valid_bluetooth_adapter(device: &DeviceInformation) -> anyhow::Result<bool> {
    let bluetooth_adapter = BluetoothAdapter::FromIdAsync(&device.Id()?)?.get()?;

    let r = [
        bluetooth_adapter
            .AreClassicSecureConnectionsSupported()
            .unwrap_or_default(),
        bluetooth_adapter.IsLowEnergySupported().unwrap_or_default(),
        bluetooth_adapter.IsClassicSupported().unwrap_or_default(),
        bluetooth_adapter
            .IsPeripheralRoleSupported()
            .unwrap_or_default(),
        bluetooth_adapter
            .IsCentralRoleSupported()
            .unwrap_or_default(),
        bluetooth_adapter
            .IsAdvertisementOffloadSupported()
            .unwrap_or_default(),
    ]
    .into_iter()
    .any(|x| x);

    Ok(r)
}
