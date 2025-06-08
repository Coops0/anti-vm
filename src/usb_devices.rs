use nusb::{DeviceInfo, InterfaceInfo};

use crate::flags::Flags;

pub fn score_usb_devices(flags: &mut Flags) -> anyhow::Result<()> {
    let mut valid_devices = 0u32;
    for dev in nusb::list_devices()? {
        let score_before = flags.score();
        score_device(&dev, flags);

        // if score was not decreased by at least 5, count it as valid
        if (flags.score() - score_before) < 5 {
            valid_devices += 1;
        }
    }

    match valid_devices {
        0 => flags.large_penalty(),
        1..=5 => {}
        _ => flags.large_bonus(),
    }

    Ok(())
}

fn score_device(dev: &DeviceInfo, flags: &mut Flags) {
    // VMware
    if dev.vendor_id() == 0x0E0F {
        flags.end_all_penalty();
    }

    if let Some(product) = dev.product_string()
        && product.contains("VMware")
    {
        flags.end_all_penalty();
    }

    // VMware
    if dev
        .instance_id()
        .eq_ignore_ascii_case("USB\\VID_0E0F&PID_0003\\6&39D724FE")
    {
        flags.end_all_penalty();
    }

    for interface in dev.interfaces() {
        score_interface(interface, flags);
    }
}

fn score_interface(int: &InterfaceInfo, flags: &mut Flags) {
    if let Some(str) = int.interface_string()
        && str.contains("VMware")
    {
        flags.end_all_penalty();
    }
}
