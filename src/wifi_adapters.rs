use windows::{Devices::WiFi::WiFiAdapter, Networking::Connectivity::NetworkTypes};

pub fn get_wifi_adapters_len() -> anyhow::Result<usize> {
    let wifi_adapters = WiFiAdapter::FindAllAdaptersAsync()?.get()?;
    let size = wifi_adapters.Size()?;

    let mut len = 0;
    for i in 0..size {
        let wifi_adapter = wifi_adapters.GetAt(i)?;
        let network_adapter = wifi_adapter.NetworkAdapter()?;

        let interface_type = network_adapter.IanaInterfaceType()?;
        let network_item = network_adapter.NetworkItem()?;
        let network_types = network_item.GetNetworkTypes()?;

        if network_types.contains(NetworkTypes::Internet) && interface_type == 71 {
            len += 1;
        }
    }

    Ok(len)
}
