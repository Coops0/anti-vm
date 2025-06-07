use std::path::{Path, PathBuf};
use std::{fs, io};

use anyhow::{Context, bail};
use windows::Win32::Storage::FileSystem::{
    self, GetCompressedFileSizeW, GetDiskSpaceInformationW, INVALID_FILE_SIZE,
};
use windows::Win32::{Storage::FileSystem::DISK_SPACE_INFORMATION, System::SystemInformation::*};
use windows_core::{HSTRING, PCWSTR, w};

use crate::{flags::Flags, util::inspect};

pub fn score_sysinfo(flags: &mut Flags) -> anyhow::Result<()> {
    let mut memory_in_kilos = 0u64;
    unsafe {
        GetPhysicallyInstalledSystemMemory(&mut memory_in_kilos)?;
    };

    let memory_in_gigs = memory_in_kilos / (1024 * 1024);
    println!("memory installed: {memory_in_gigs}GB");

    match memory_in_gigs {
        0..=2 => flags.extreme_penalty(),
        3..=6 => flags.large_penalty(),
        7 => flags.medium_penalty(),
        8..=12 => {}
        13..=24 => flags.small_bonus(),
        25..=64 => flags.large_bonus(),
        _ => flags.extreme_bonus(),
    };

    let mut system_info = SYSTEM_INFO::default();
    unsafe {
        // Maybe should use GetSystemInfo instead?
        GetNativeSystemInfo(&mut system_info);
    }

    // Only useful field is processors
    // The architecture is the same as host architecture
    println!("Number of Processors: {}", system_info.dwNumberOfProcessors);
    match system_info.dwNumberOfProcessors {
        0..=1 => flags.large_penalty(),
        2 => flags.medium_penalty(),
        3..=4 => {}
        5..=8 => flags.medium_bonus(),
        _ => flags.extreme_bonus(),
    };

    let tick_count_ms = unsafe { GetTickCount() };
    let tick_count_sec = tick_count_ms / 1000;
    println!("Tick Count: {tick_count_sec}s");

    match tick_count_sec {
        0..=60 => flags.extreme_penalty(),
        61..=180 => flags.large_penalty(),
        _ => {}
    }

    // If has valid integrated display like a laptop
    // This is also checked in displays.rs but in a different way
    if matches!(inspect("integrated display size", unsafe { GetIntegratedDisplaySize() }), Ok(size) if size > 256.0)
    {
        flags.medium_bonus();
    }

    let disk_space = inspect("disk space", get_disk_space(flags))?;
    match disk_space.total_space_gig {
        0..64 => flags.extreme_penalty(),
        64..127 => flags.large_penalty(),
        127..512 => {}
        512..=1024 => flags.small_bonus(),
        _ => flags.large_bonus(),
    }

    // let windows_files_space =
    // if disk_space.free_space_gig > disk_space.total_space_gig
    inspect("windows dir size", get_windows_dir_size_gigs())?;

    Ok(())
}

#[derive(Debug)]
struct DiskSpaceReport {
    total_space_gig: u64,
    free_space_gig: u64,
}

const GIGA_BYTE: u64 = 1024 * 1024 * 1024;

fn get_disk_space(flags: &mut Flags) -> anyhow::Result<DiskSpaceReport> {
    let mut disk_space_information = DISK_SPACE_INFORMATION::default();
    unsafe {
        // Initally try to use main disk in case we are being executed from a USB drive or network
        if let Err(err) = GetDiskSpaceInformationW(w!("C:/"), &mut disk_space_information) {
            println!("Error getting C: disk space information: {err:?}");
            flags.large_penalty();

            // Fallback to current disk
            GetDiskSpaceInformationW(None, &mut disk_space_information)?;
        }
    }

    let bytes_per_unit = (disk_space_information.SectorsPerAllocationUnit
        * disk_space_information.BytesPerSector) as u64;

    let total_space = disk_space_information.CallerTotalAllocationUnits * bytes_per_unit;
    let free_space = disk_space_information.CallerAvailableAllocationUnits * bytes_per_unit;

    let total_space_gig = total_space / GIGA_BYTE;
    let free_space_gig = free_space / GIGA_BYTE;

    Ok(DiskSpaceReport {
        total_space_gig,
        free_space_gig,
    })
}

fn get_windows_dir_size_gigs() -> anyhow::Result<u64> {
    // GetSystemDirectoryW
    // GetWindowsDirectoryW

    let mut buf = [0u16; 16383];
    let out_size = unsafe { GetWindowsDirectoryW(Some(&mut buf)) };

    if out_size == 0 {
        bail!("get windows directory returned 0 len");
    }

    let windows_dir = String::from_utf16_lossy(&buf[..out_size as usize]);
    println!("got windows directory: {}", windows_dir);

    let dir_size_bytes = dir_size(windows_dir)?;
    Ok(dir_size_bytes / GIGA_BYTE)
}

fn dir_size(path: impl Into<PathBuf>) -> anyhow::Result<u64> {
    fn dir_size(mut dir: fs::ReadDir) -> anyhow::Result<u64> {
        dir.try_fold(0, |acc, file| {
            let file = file?;
            let size = match file.file_type()? {
                ft if ft.is_dir() => dir_size(fs::read_dir(file.path())?)?,
                ft if ft.is_file() => compressed_file_size(&file.path())? as u64,
                _ => 0,
            };
            Ok(acc + size)
        })
    }

    dir_size(fs::read_dir(path.into())?)
}

fn compressed_file_size(path: &Path) -> anyhow::Result<u32> {
    let h_str = HSTRING::from(
        path.canonicalize()?
            .to_str()
            .context("failed to get abs path")?,
    );
    let path_pcw_str = PCWSTR(h_str.as_ptr());

    let size = unsafe { GetCompressedFileSizeW(path_pcw_str, None) };
    if size == INVALID_FILE_SIZE {
        bail!("get invalid file size");
    }

    Ok(size)
}
