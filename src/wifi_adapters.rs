use windows::{
    Devices::{
        Enumeration::DeviceInformation,
        WiFi::WiFiAdapter,
    },
    Networking::Connectivity::NetworkTypes,
};

use crate::util::get_devices_iter;

pub fn get_wifi_adapters_len() -> anyhow::Result<usize> {
    let selector = WiFiAdapter::GetDeviceSelector()?;

    let wifi_adapter_count = get_devices_iter(&selector)?
        .filter(|device| is_valid_wifi_adapter(device).unwrap_or_default())
        .count();

    Ok(wifi_adapter_count)
}

fn is_valid_wifi_adapter(device: &DeviceInformation) -> anyhow::Result<bool> {
    let wifi_adapter = WiFiAdapter::FromIdAsync(&device.Id()?)?.get()?;
    let network_adapter = wifi_adapter.NetworkAdapter()?;

    let interface_type = network_adapter.IanaInterfaceType()?;
    let network_item = network_adapter.NetworkItem()?;
    let network_types = network_item.GetNetworkTypes()?;

    Ok(network_types.contains(NetworkTypes::Internet) && interface_type == 71)
}
