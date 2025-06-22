use anyhow::Context;
use windows::Win32::{
    Storage::FileSystem::{DISK_SPACE_INFORMATION, GetDiskSpaceInformationW},
    System::SystemInformation::{
        GetIntegratedDisplaySize, GetNativeSystemInfo, GetPhysicallyInstalledSystemMemory,
        GetTickCount, SYSTEM_INFO,
    },
};
use windows_core::w;

use crate::{debug_println, flags::Flags, inspect};

pub fn score_sysinfo(flags: &mut Flags) -> anyhow::Result<()> {
    let mut memory_in_kilos = 0u64;

    if unsafe { GetPhysicallyInstalledSystemMemory(&raw mut memory_in_kilos) }.is_ok() {
        let memory_in_gigs = memory_in_kilos / (1024 * 1024);
        debug_println!("memory installed: {memory_in_gigs}GB");

        match memory_in_gigs {
            0..=2 => flags.extreme_penalty(),
            3..=6 => flags.large_penalty(),
            7..=8 => flags.medium_penalty(),
            _ => {}
        }

        if memory_in_gigs % 2 != 0 {
            flags.large_penalty();
        }
    } else {
        flags.large_penalty();
    }

    let mut system_info: SYSTEM_INFO = unsafe { core::mem::zeroed() };
    unsafe {
        // Maybe should use GetSystemInfo instead?
        GetNativeSystemInfo(&raw mut system_info);
    }

    let processors = system_info.dwNumberOfProcessors;
    debug_println!("number of processors: {processors}");
    match processors {
        0..=1 => flags.large_penalty(),
        2 => flags.medium_penalty(),
        _ => {}
    }

    // if processors % 2 != 0 {
    //     flags.large_penalty();
    // }

    let tick_count_ms = unsafe { GetTickCount() };
    let tick_count_sec = tick_count_ms / 1000;
    debug_println!("tick count: {tick_count_sec}s");

    match tick_count_sec {
        0..=60 => flags.extreme_penalty(),
        61..=180 => flags.large_penalty(),
        _ => {}
    }

    // If has valid integrated display like a laptop
    // This is also checked in displays.rs but in a different way
    if matches!(inspect!("integrated display size", unsafe { GetIntegratedDisplaySize() }), Ok(size) if size > 256.0)
    {
        flags.medium_bonus();
    }

    let disk_space = inspect!("disk space", get_disk_space(flags)).context("gds")?;
    match disk_space.total_space_gig {
        // Windows 11 requires >= 64gb disk to even install
        0..=64 => flags.extreme_penalty(),
        65..127 => flags.large_penalty(),
        127..512 => {}
        512..=1024 => flags.small_bonus(),
        _ => flags.large_bonus(),
    }

    // (size on disk) could be anywhere from ~13-27gb
    // On a fresh VM install it's 13gb
    const EST_WINDOWS_DIR_SIZE_GIG: u64 = 16;
    let used_space_minus_windows_installation = disk_space
        .total_space_gig
        .saturating_sub(disk_space.free_space_gig + EST_WINDOWS_DIR_SIZE_GIG);

    debug_println!(
        "used space minus windows installation: {used_space_minus_windows_installation}GB"
    );

    match used_space_minus_windows_installation {
        0..=3 => flags.large_penalty(),
        4..=16 => flags.medium_penalty(),
        17..=64 => {}
        65..=128 => flags.small_bonus(),
        _ => flags.medium_bonus(),
    }

    Ok(())
}

#[cfg_attr(debug_assertions, derive(Debug))]
struct DiskSpaceReport {
    total_space_gig: u64,
    free_space_gig: u64,
}

fn get_disk_space(flags: &mut Flags) -> anyhow::Result<DiskSpaceReport> {
    let mut disk_space_information: DISK_SPACE_INFORMATION = unsafe { core::mem::zeroed() };
    unsafe {
        // Initally try to use main disk in case we are being executed from a USB drive or network
        if let Err(err) = GetDiskSpaceInformationW(w!("C:/"), &raw mut disk_space_information) {
            debug_println!("Error getting C: disk space information: {err:?}");
            flags.large_penalty();

            // Fallback to current disk
            GetDiskSpaceInformationW(None, &raw mut disk_space_information)?;
        }
    }

    let bytes_per_unit = (disk_space_information.SectorsPerAllocationUnit
        * disk_space_information.BytesPerSector) as u64;

    let total_space = disk_space_information.CallerTotalAllocationUnits * bytes_per_unit;
    let free_space = disk_space_information.CallerAvailableAllocationUnits * bytes_per_unit;

    const GIGA_BYTE: u64 = 1024 * 1024 * 1024;

    let total_space_gig = total_space / GIGA_BYTE;
    let free_space_gig = free_space / GIGA_BYTE;

    Ok(DiskSpaceReport {
        total_space_gig,
        free_space_gig,
    })
}
