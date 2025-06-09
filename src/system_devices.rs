use std::ptr::null;

use windows::Devices::Enumeration::DeviceInformation;
use windows::Win32::Devices::PortableDevices::{self, IPortableDeviceManager};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::core::GUID;
use windows_core::PWSTR;
use windows_registry::{Key, LOCAL_MACHINE};

use crate::flags::Flags;

pub fn score_system_devices(flags: &mut Flags) -> anyhow::Result<()> {
    let devices = DeviceInformation::FindAllAsync()?.get()?;
    for device in devices {
        let Ok(name) = device.Name() else { continue };
        let lc = name.to_string_lossy().to_lowercase();
        if lc.contains("vmware") || lc.contains("virtualbox") {
            println!("found virtual machine device: {name}");
            flags.end_all_penalty();
        }
    }

    for pci in get_registry_pci()? {
        if pci.device_desc.to_lowercase().contains("vmware") {
            flags.end_all_penalty();
        }

        if let Some(service) = &pci.service && service == "vmci" {
            flags.extreme_penalty();
        }


    }
    Ok(())
}

struct PciDevice {
    class_guid: String,
    device_desc: String, // ...VMware VMCI Bus Device
    hardware_id: Vec<String>,
    manufacturer: String,
    service: Option<String>, // vmci
    driver: String,          // also a guid
}

fn get_registry_pci() -> anyhow::Result<Vec<PciDevice>> {
    //HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Enum\PCI\
    let root = LOCAL_MACHINE.open("SYSTEM\\CurrentControlSet\\Enum\\PCI")?;
    let pci_devices = root
        .keys()?
        .filter_map(|key| root.open(key).ok())
        .flat_map(|key| {
            let key_parents = key.keys().ok()?;
            let keys = key_parents.filter_map(|s| key.open(s).ok()).collect::<Vec<_>>();
            Some(keys)
        })
        .flatten()
        .flat_map(|key| {
            let class_guid = key.get_string("ClassGuid").ok()?;
            let device_desc = key.get_string("DeviceDesc").ok()?;
            let hardware_id = key.get_multi_string("HardwareID").ok()?;
            let manufacturer = key.get_string("Mfg").ok()?;
            let service = key.get_string("Service").ok();
            let driver = key.get_string("Driver").ok()?;

            Some(PciDevice {
                class_guid,
                device_desc,
                hardware_id,
                manufacturer,
                service,
                driver,
            })
        })
        .collect::<Vec<_>>();

    Ok(pci_devices)
}
