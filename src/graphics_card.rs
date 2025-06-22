use serde::Deserialize;
use wmi::{COMLibrary, WMIConnection};

use crate::flags::Flags;

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct GraphicsCard {
    #[serde(default)]
    description: String,
    #[serde(default)]
    caption: String,
    #[serde(default)]
    video_processor: String,
    #[serde(default, rename = "DeviceID")]
    device_id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    installed_display_drivers: String,
    #[serde(default)]
    inf_section: String,
    #[serde(default)]
    adapter_dac_type: String,
}

pub fn score_graphics_cards(flags: &mut Flags) -> anyhow::Result<()> {
    // https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/win32-videocontroller
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con)?;

    // Description, Caption, DitherType, VideoProcessor, DeviceID, Name: contains vmware
    // InstalledDisplayDrivers, MaxNumbersControlled contains vm3dum,
    // InfSection, StatusInfo contains VM3D
    // AdapterDACType exists and isn't n/a

    // Description, Caption, DitherType, VideoProcessor, DeviceID, Name, InstalledDisplayDrivers, InfSection, StatusInfo, AdapterDACType
    let graphics_cards = wmi_con.raw_query::<GraphicsCard>("SELECT Description, Caption, VideoProcessor, DeviceID, Name, InstalledDisplayDrivers, InfSection, AdapterDACType FROM Win32_VideoController")?;
    if graphics_cards.is_empty() {
        flags.large_penalty();
        return Ok(());
    }

    for gc in graphics_cards {
        score_graphics_card(&gc, flags);
    }

    Ok(())
}

#[rustfmt::skip]
fn score_graphics_card(gc: &GraphicsCard, flags: &mut Flags) {
    if gc.description.contains("VMware") { flags.large_penalty(); }
    if gc.caption.contains("VMware") { flags.large_penalty(); }
    if gc.video_processor.contains("VMware") { flags.large_penalty(); }
    if gc.device_id.contains("VMware") { flags.large_penalty(); }
    if gc.name.contains("VMware") { flags.large_penalty(); }
    if gc.installed_display_drivers.contains("vm3dum") { flags.large_penalty(); }
    if gc.inf_section.contains("VM3D") { flags.large_penalty(); }
    if gc.adapter_dac_type.is_empty() || gc.adapter_dac_type == "n/a" { flags.large_penalty(); }
}
