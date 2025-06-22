use serde::Deserialize;
use wmi::{COMLibrary, WMIConnection};

use crate::flags::Flags;

pub fn score_various_wmi(flags: &mut Flags) -> anyhow::Result<()> {
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con)?;

    for system in wmi_con
        .raw_query::<Win32ComputerSystem>(
            "SELECT PowerOnPasswordStatus, Manufacturer, Model FROM Win32_ComputerSystem",
        )
        .unwrap_or_default()
    {
        system.score(flags);
    }

    for bios in wmi_con.raw_query::<Win32Bios>("SELECT Name, Caption, __RELPATH, __PATH, BIOSVersion, Description, Manufacturer, SMBIOSBIOSVersion, SoftwareElementID, Path, SerialNumber FROM Win32_BIOS").unwrap_or_default() {
            bios.score(flags);
        }

    for device in wmi_con.raw_query::<CimUserDevice>("SELECT __DYNASTY, __PATH, DeviceID, PNPDeviceID, Path, __RELPATH, __NAMESPACE, Caption, Description, HardwareType FROM CIM_UserDevice").unwrap_or_default() {
            device.score(flags);
        }

    for card in wmi_con
        .raw_query::<CimCard>("SELECT Product FROM CIM_Card")
        .unwrap_or_default()
    {
        card.score(flags);
    }

    for ch in wmi_con
        .raw_query::<CimChassis>("SELECT ChassisTypes, Manufacturer FROM CIM_Chassis")
        .unwrap_or_default()
    {
        ch.score(flags);
    }

    Ok(())
}

// Type 1 = VMware
// Type 2 = VMW
// Type 3 = VID_0E0F

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Win32ComputerSystem {
    power_on_password_status: u16, // == 3 (Suspicious)
    #[serde(default)]
    manufacturer: String, // Type 1 identifier
    #[serde(default)]
    model: String, // Type 1 identifier
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Win32Bios {
    #[serde(default)]
    name: String, // Type 2 identifier
    #[serde(default)]
    caption: String, // Type 2 identifier
    #[serde(default, rename = "__RELPATH")]
    alt_rel_path: String, // Type 2 identifier
    #[serde(default, rename = "__PATH")]
    alt_path: String, // Type 2 identifier
    #[serde(default, rename = "BIOSVersion")]
    bios_version: String, // Type 1 & 2 identifier
    #[serde(default)]
    description: String, // Type 2
    #[serde(default)]
    manufacturer: String, // Type 1 identifier
    #[serde(default, rename = "SMBIOSBIOSVersion")]
    sm_bios_bios_version: String, // Type 2 identifier
    #[serde(default, rename = "SoftwareElementID")]
    software_element_id: String, // Type 2
    #[serde(default)]
    path: String, // Type 2
    #[serde(default)]
    serial_number: String, // Type 1 identifier
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CimUserDevice {
    #[serde(default, rename = "__DYNASTY")]
    dynasty: String, // Type 3 identifier
    #[serde(default, rename = "__PATH")]
    alt_path: String, // Type 3 identifier
    #[serde(default, rename = "DeviceID")]
    device_id: String, // Type 2 & 3 identifier
    #[serde(default, rename = "PNPDeviceID")]
    pnp_device_id: String, // Type 2 & 3 identifier
    #[serde(default)]
    path: String, // Type 2 & 3 identifier
    #[serde(default, rename = "__RELPATH")]
    alt_rel_path: String, // Type 2 & 3 identifier
    #[serde(default, rename = "__NAMESPACE")]
    namespace: String, // Type 3 identifier
    #[serde(default)]
    caption: String, // Type 1
    #[serde(default)]
    description: String, // Type 1
    #[serde(default)]
    hardware_type: String, // Type 1
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CimCard {
    #[serde(default)]
    product: String, // ^ Desktop Reference Platform => Detection
                     // 440BX => Very suspicious
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CimChassis {
    #[serde(default)]
    chassis_types: String, // contains {1} **AND**
    #[serde(default)]
    manufacturer: String, // contains No Enclosure
}

trait Score {
    fn score(&self, flags: &mut Flags);
}

impl Score for Win32ComputerSystem {
    #[rustfmt::skip]
    fn score(&self, flags: &mut Flags) {
        if self.power_on_password_status == 0 {
            flags.small_penalty();
        }

        if is_bad(&self.manufacturer) { flags.extreme_penalty(); }
        if is_bad(&self.model) { flags.extreme_penalty(); }
    }
}

impl Score for Win32Bios {
    #[rustfmt::skip]
    fn score(&self, flags: &mut Flags) {
        if is_bad(&self.name) { flags.extreme_penalty(); }
        if is_bad(&self.caption) { flags.extreme_penalty(); }
        if is_bad(&self.alt_rel_path) { flags.extreme_penalty(); }
        if is_bad(&self.alt_path) { flags.extreme_penalty(); }
        if is_bad(&self.bios_version) { flags.extreme_penalty(); }
        if is_bad(&self.description) { flags.extreme_penalty(); }
        if is_bad(&self.manufacturer) { flags.extreme_penalty(); }
        if is_bad(&self.sm_bios_bios_version) { flags.extreme_penalty(); }
        if is_bad(&self.software_element_id) { flags.extreme_penalty(); }
        if is_bad(&self.path) { flags.extreme_penalty(); }
        if is_bad(&self.serial_number) { flags.extreme_penalty(); }
    }
}

impl Score for CimUserDevice {
    #[rustfmt::skip]
    fn score(&self, flags: &mut Flags) {
        if is_bad(&self.dynasty) { flags.extreme_penalty(); }
        if is_bad(&self.alt_path) { flags.extreme_penalty(); }
        if is_bad(&self.device_id) { flags.extreme_penalty(); }
        if is_bad(&self.pnp_device_id) { flags.extreme_penalty(); }
        if is_bad(&self.path) { flags.extreme_penalty(); }
        if is_bad(&self.alt_rel_path) { flags.extreme_penalty(); }
        if is_bad(&self.namespace) { flags.extreme_penalty(); }
        if is_bad(&self.caption) { flags.extreme_penalty(); }
        if is_bad(&self.description) { flags.extreme_penalty(); }
        if is_bad(&self.hardware_type) { flags.extreme_penalty(); }
    }
}

impl Score for CimCard {
    fn score(&self, flags: &mut Flags) {
        if self.product.contains("Desktop Reference Platform") {
            flags.extreme_penalty();
        }

        if self.product.contains("440BX") {
            flags.large_penalty();
        }
    }
}

impl Score for CimChassis {
    fn score(&self, flags: &mut Flags) {
        if self.chassis_types.contains("{1}") && self.manufacturer.contains("No Enclosure") {
            flags.medium_penalty();
        }
    }
}

#[inline]
fn is_bad(s: &str) -> bool {
    s.contains("VMware") || s.contains("VMW") || s.contains("VID_0E0F")
}
