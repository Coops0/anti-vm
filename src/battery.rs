use anyhow::bail;
use windows::{
    Devices::{Enumeration::DeviceInformation, Power::Battery},
    System::Power::BatteryStatus,
};

use crate::util::get_devices_iter;

pub fn get_battery() -> anyhow::Result<boolean> {
    let selector = Battery::GetDeviceSelector()?;
    let batteries = get_devices_iter(&selector)?;

    for device in batteries {
        if get_device_as_battery(&device).unwrap_or_default() {
            return Ok(true);
        }
    }

    Err(false)
}

fn get_device_as_battery(device: &DeviceInformation) -> anyhow::Result<bool> {
    let battery = Battery::FromIdAsync(&device.Id()?)?.get()?;
    let battery_report = battery.GetReport()?;

    if battery_report.Status()? == BatteryStatus::NotPresent {
        return Ok(false);
    }

    let values = [
        battery_report.ChargeRateInMilliwatts(),
        battery_report.DesignCapacityInMilliwattHours(),
        battery_report.FullChargeCapacityInMilliwattHours(),
        battery_report.RemainingCapacityInMilliwattHours(),
    ];

    let found_valid_value = values
        .into_iter()
        .flatten()
        .filter_map(|value| value.GetInt32().ok())
        .any(|value| value != 0);

    Ok(found_valid_value)
}
