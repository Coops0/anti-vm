use windows::Devices::Display::{
    DisplayMonitor, DisplayMonitorConnectionKind, DisplayMonitorPhysicalConnectorKind,
    DisplayMonitorUsageKind,
};
use windows::Devices::Enumeration::DeviceInformation;
use windows::Graphics::SizeInt32;

use crate::flags::Flags;
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
            flags.medium_penalty();
        }
    }

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

    match monitor.PhysicalConnector()? {
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

        DisplayMonitorPhysicalConnectorKind::HD15 => {
            // This is the type that VMware uses
            flags.large_penalty();
        }

        DisplayMonitorPhysicalConnectorKind::Hdmi
        | DisplayMonitorPhysicalConnectorKind::DisplayPort => {
            flags.medium_bonus();
        }

        _ => flags.large_penalty(),
    };

    match monitor.ConnectionKind()? {
        DisplayMonitorConnectionKind::Internal | DisplayMonitorConnectionKind::Wired => {}
        DisplayMonitorConnectionKind::Wireless => flags.small_penalty(),
        DisplayMonitorConnectionKind::Virtual => flags.large_penalty(),
        _ => flags.medium_penalty(),
    };

    match monitor.UsageKind()? {
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

    // VMware fails this
    if monitor.DisplayName()?.is_empty() {
        flags.large_penalty();
    }

    match monitor.PhysicalSizeInInches().map(|s| s.GetSize()) {
        // TODO look at this
        Ok(_resolution) => {}
        // VMware fails this
        Err(_) => flags.large_penalty(),
    }

    let max_luminance = monitor.MaxLuminanceInNits();
    match (monitor.MinLuminanceInNits(), &max_luminance) {
        (Ok(0.0), Ok(0.0)) => {
            // VMware
            flags.large_penalty();
        }
        (Err(_), _) | (_, Err(_)) => flags.medium_penalty(),
        _ => {}
    }

    match monitor.MaxAverageFullFrameLuminanceInNits() {
        Ok(l) => {
            if let Ok(ml) = &max_luminance {
                if l == *ml {
                    flags.medium_bonus();
                } else {
                    flags.medium_penalty();
                }
            }
        }
        Err(_) => flags.medium_penalty(),
    }

    if let Ok(SizeInt32 {
        Width: width,
        Height: height,
    }) = monitor.NativeResolutionInRawPixels()
    {
        if width == 0 || height == 0 {
            flags.large_penalty();
        } else {
            // todo
        }
    }

    Ok(())
}
