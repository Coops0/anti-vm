use windows::Devices::Display::{
    DisplayMonitor, DisplayMonitorConnectionKind, DisplayMonitorPhysicalConnectorKind,
    DisplayMonitorUsageKind,
};
use windows::Devices::Enumeration::DeviceInformation;

use crate::flags::Flags;
use crate::{debug_println, inspect};
use crate::util::get_devices_iter;

// TODO score adapters
// DisplayAdapterId
// Gets the locally unique identifier (LUID) of the display adapter to which the monitor is connected. Note that the display adapter to which the monitor is connected may not be the most suitable one for rendering.
// An LUID is a 64-bit value guaranteed to be unique only on the system on which it was generated. The uniqueness of an LUID is guaranteed only until the system is restarted. This LUID is compatible with Direct3D, DisplayConfig, and HolographicSpace APIs.
// DisplayAdapterTargetId
// Gets an opaque ID used by the display adapter to identify which connector the monitor is attached to. This target ID can be used with DisplayConfig APIs.

pub fn score_displays(flags: &mut Flags) -> anyhow::Result<()> {
    let selector = DisplayMonitor::GetDeviceSelector()?;
    let mut valid_displays = 0;

    for display in get_devices_iter(&selector)? {
        if score_display(&display, flags).is_ok() {
            valid_displays += 1;
        } else {
            flags.large_penalty();
        }
    }

    debug_println!("found {valid_displays} valid displays");

    match valid_displays {
        0 => flags.extreme_penalty(),
        1 => {}
        2..=4 => flags.small_bonus(),
        // Over 6 monitors is crazy
        _ => flags.large_penalty(),
    }

    Ok(())
}

fn score_display(device: &DeviceInformation, flags: &mut Flags) -> anyhow::Result<()> {
    let monitor = DisplayMonitor::FromInterfaceIdAsync(&device.Id()?)?.get()?;

    match inspect!("(init) physical connector", monitor.PhysicalConnector())? {
        DisplayMonitorPhysicalConnectorKind::Unknown => {
            flags.large_penalty();
        }

        DisplayMonitorPhysicalConnectorKind::AnalogTV
        | DisplayMonitorPhysicalConnectorKind::Sdi
        | DisplayMonitorPhysicalConnectorKind::Lvds => {
            // Maybe increase in the future. This is Composite, SDI (???), or LVDS (???)
            flags.medium_penalty();
        }

        DisplayMonitorPhysicalConnectorKind::Dvi => {
            flags.small_penalty();
        }

        // VGA
        DisplayMonitorPhysicalConnectorKind::HD15 => {
            // This is the type that VMware & Vbox uses
            flags.large_penalty();
        }

        DisplayMonitorPhysicalConnectorKind::Hdmi
        | DisplayMonitorPhysicalConnectorKind::DisplayPort => {
            flags.medium_bonus();
        }

        _ => flags.large_penalty(),
    }

    match inspect!("connection kind", monitor.ConnectionKind()?) {
        // Laptop
        DisplayMonitorConnectionKind::Internal => flags.medium_bonus(),
        DisplayMonitorConnectionKind::Wired => {}
        DisplayMonitorConnectionKind::Wireless => flags.small_penalty(),
        DisplayMonitorConnectionKind::Virtual => flags.large_penalty(),
        _ => flags.medium_penalty(),
    }

    match inspect!("usage kind", monitor.UsageKind())? {
        DisplayMonitorUsageKind::Standard => {}
        // DisplayMonitorUsageKind::HeadMounted | DisplayMonitorUsageKind::SpecialPurpose{
        _ => flags.large_penalty(),
    }

    if monitor
        .IsDolbyVisionSupportedInHdrMode()
        .unwrap_or_default()
    {
        flags.large_bonus();
    }

    // VMware & Vbox fails this
    if inspect!("display name", monitor.DisplayName())?.is_empty() {
        flags.large_penalty();
    }

    // "An error code of zero - S_OK - just means that the API returned a null pointer value on the ABI so there was no interface to populate the Ok variant of Result."
    // https://github.com/microsoft/windows-rs/issues/3322#issuecomment-2408606524
    match inspect!(
        "physical size in in",
        monitor.PhysicalSizeInInches().and_then(|s| s.GetSize())
    ) {
        Ok(_resolution) => {}
        Err(_err) => {
            // VMware & Vbox: err.code() == HRESULT(0)
            flags.large_penalty();
        }
    }

    let max_luminance = inspect!("max lum nits", monitor.MaxLuminanceInNits());
    match (monitor.MinLuminanceInNits(), &max_luminance) {
        (Ok(0.0), Ok(0.0)) => {
            // VMware & Vbox
            flags.large_penalty();
        }
        (Err(_), _) | (_, Err(_)) => flags.medium_penalty(),
        _ => {}
    }

    match inspect!(
        "max avg full frame lum nits",
        monitor.MaxAverageFullFrameLuminanceInNits()
    ) {
        Ok(0.0) | Err(_) => flags.medium_penalty(),
        Ok(l) => {
            // If these match up then that's a good sign
            if let Ok(ml) = &max_luminance {
                if (l - *ml).abs() < 0.01 {
                    flags.small_bonus();
                } else {
                    flags.medium_penalty();
                }
            }
        }
    }

    match inspect!("native res px", monitor.NativeResolutionInRawPixels()) {
        Ok(resolution) => {
            score_display_size(resolution.Width, resolution.Height, flags);
        }
        Err(_) => flags.large_penalty(),
    }

    Ok(())
}

// https://store.steampowered.com/hwsurvey
// 800 x 1280 0.56%
// 1280 x 720 0.23%
// 1280 x 1024 0.26%
// 1280 x 800 0.35%
// 1360 x 768 0.56%
// 1366 x 768 2.91%
// 1440 x 900 0.91%
// 1470 x 956 0.27%
// 1512 x 982 0.25%
// 1600 x 900 0.86%
// 1680 x 1050 0.52%
// 1920 x 1080 55.35%
// 1920 x 1200 1.66%
// 2560 x 1440 19.49%
// 2560 x 1600 4.20%
// 2560 x 1080 0.84%
// 2880 x 1800 0.39%
// 3440 x 1440 2.86%
// 3840 x 2160 4.48%
// 5120 x 1440 0.39%
// Other 2.67%

fn score_display_size(width: i32, height: i32, flags: &mut Flags) {
    if width < 256 || height < 256 {
        flags.extreme_penalty();
    } else if width < 1366 || height < 768 {
        flags.large_penalty();
    } else if width < 1920 || height < 1080 {
        flags.small_penalty();
    }
}
