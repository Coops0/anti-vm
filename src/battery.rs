use anyhow::bail;
use windows::{
    Devices::{
        Enumeration::{DeviceInformation, DeviceInformationKind},
        Power::Battery,
    },
    System::Power::BatteryStatus,
};

pub fn get_battery() -> anyhow::Result<()> {
    let selector = Battery::GetDeviceSelector()?;
    let devices = DeviceInformation::FindAllAsyncAqsFilter(&selector)?.get()?;
    let devices_size = devices.Size()?;

    for idx in 0..devices_size {
        let Ok(device) = devices.GetAt(idx) else {
            continue;
        };

        if device.Kind() != Ok(DeviceInformationKind::DeviceInterface) {
            continue;
        }

        if get_device_as_battery(&device).is_ok() {
            return Ok(());
        }
    }

    Err(anyhow::anyhow!("no valid battery found"))
}

fn get_device_as_battery(device: &DeviceInformation) -> anyhow::Result<()> {
    let battery = Battery::FromIdAsync(&device.Id()?)?.get()?;
    let battery_report = battery.GetReport()?;

    if battery_report.Status()? == BatteryStatus::NotPresent {
        bail!("battery not present");
    }

    if let Ok(charge_rate_value) = battery_report.ChargeRateInMilliwatts()
        && let Ok(charge_rate) = charge_rate_value.GetInt32()
        && charge_rate != 0
    {
        return Ok(());
    }

    if let Ok(design_capacity_value) = battery_report.DesignCapacityInMilliwattHours()
        && let Ok(design_capacity) = design_capacity_value.GetInt32()
        && design_capacity != 0
    {
        return Ok(());
    }

    if let Ok(full_charge_capacity_value) = battery_report.FullChargeCapacityInMilliwattHours()
        && let Ok(full_charge_capacity) = full_charge_capacity_value.GetInt32()
        && full_charge_capacity != 0
    {
        return Ok(());
    }

    if let Ok(remaining_capacity_value) = battery_report.RemainingCapacityInMilliwattHours()
        && let Ok(remaining_capacity) = remaining_capacity_value.GetInt32()
        && remaining_capacity != 0
    {
        return Ok(());
    }

    Err(anyhow::anyhow!("no valid field found"))
}
