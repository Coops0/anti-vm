use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    ptr::null_mut,
};

use anyhow::{Context, bail};
use windows::Win32::{
    Foundation::MAX_PATH,
    Globalization::CP_ACP,
    System::Com::IPersistFile,
    UI::Shell::{IShellLinkW, SLGP_RELATIVEPRIORITY},
};
use windows::Win32::{
    Globalization::{MULTI_BYTE_TO_WIDE_CHAR_FLAGS, MultiByteToWideChar},
    System::Com::STGM_READ,
};
use windows::core::Interface;
use windows::{
    Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, CoInitialize},
    core::GUID,
};
use windows_core::PCWSTR;

use crate::{debug_println, flags::Flags, inspect};

pub fn score_installed_apps(flags: &mut Flags) -> anyhow::Result<()> {
    let programs_dir = dirs::data_dir()
        .context("ndd")?
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs");

    let mut installed = Vec::with_capacity(10);
    if recurse_dir(&programs_dir, &mut installed).is_err() || installed.is_empty() {
        flags.large_penalty();
        return Ok(());
    }

    let mut found_steam_exe = false;
    let mut steam_games = 0u32;
    let mut valid_programs = 0u32;

    for p in installed {
        // lnk or url
        let Some(ext) = p
            .extension()
            .and_then(OsStr::to_str)
            .map(str::to_lowercase)
        else {
            continue;
        };

        if ext == "url" {
            if validate_url(&p).is_ok() {
                steam_games += 1;
            }
            continue;
        } else if ext != "lnk" {
            continue;
        }

        if let Ok(exe_path) = validate_lnk(&p) {
            let executable = exe_path
                .file_name()
                .and_then(OsStr::to_str)
                .unwrap_or_default()
                .to_lowercase();
            if executable == "steam.exe" {
                found_steam_exe = true;
            }

            valid_programs += 1;
        }
    }

    debug_println!("found {valid_programs} valid programs, steam = {found_steam_exe}, {steam_games} steam games");
    if found_steam_exe {
        match steam_games {
            0 => flags.large_penalty(),
            1..4 => flags.small_penalty(),
            4..=12 => {},
            _ => flags.medium_bonus(),
        }
    }

    match valid_programs {
        0 => flags.large_penalty(),
        1 => flags.small_penalty(),
        2..=6 => {},
        _ => flags.medium_bonus(),
    }

    Ok(())
}

fn recurse_dir(dir: &Path, ret: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    for entry in dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let _ = recurse_dir(&path, ret);
        } else {
            ret.push(path);
        }
    }
    Ok(())
}

// Returns steam game "url"
fn validate_url(path: &Path) -> anyhow::Result<String> {
    let s = fs::read_to_string(path)?;

    let mut lines = s.lines();
    let url = lines
        .clone()
        .find_map(|line| line.strip_prefix("URL="))
        .map(str::trim)
        .context("nurl")?;

    let icon = lines
        .find_map(|line| line.strip_prefix("IconFile="))
        .map(str::trim)
        .context("nicn")?;

    if url.starts_with("steam://") && 
    // Game still installed
    fs::exists(icon).unwrap_or_default()
    {
        return Ok(url.to_owned());
    }

    Err(anyhow::anyhow!("bad url"))
}

const CLSID_SHELL_LINK: GUID = GUID::from_values(
    0x00021401,
    0x0000,
    0x0000,
    [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
);

fn validate_lnk(path: &Path) -> anyhow::Result<PathBuf> {
    // let ret = unsafe { CoInitialize(None) };

    // if ret.is_err() {
    //     bail!("Failed to initialize COM: {ret:?}");
    // }

    let psl: IShellLinkW =
        unsafe { CoCreateInstance(&CLSID_SHELL_LINK, None, CLSCTX_INPROC_SERVER)? };
    let ppf: IPersistFile = psl.cast()?;

    let mut wsz = [0u16; MAX_PATH as usize];
    let lpsz_link_file = path.to_str().context("nps")?;

    let mbtwc_ret = unsafe {
        MultiByteToWideChar(
            CP_ACP,
            MULTI_BYTE_TO_WIDE_CHAR_FLAGS(0),
            lpsz_link_file.as_bytes(),
            Some(&mut wsz[..]),
        )
    };

    if mbtwc_ret <= 0 {
        bail!("Failed to convert string to wide char: {lpsz_link_file}");
    }

    let wsz_pcwstr = PCWSTR::from_raw(wsz.as_ptr());
    unsafe {
        ppf.Load(wsz_pcwstr, STGM_READ)?;
    }

    let mut sz_got_path = [0u16; MAX_PATH as usize];
    // let mut wfd: WIN32_FIND_DATAW = unsafe { core::mem::zeroed() };

    unsafe {
        psl.GetPath(&mut sz_got_path, null_mut(), SLGP_RELATIVEPRIORITY.0 as u32)?;
    }

    let sz_got_path_str = unsafe { PCWSTR::from_raw(sz_got_path.as_ptr()).to_string()? };

    let exe_path = PathBuf::from(sz_got_path_str);
    if !exe_path.exists() || !exe_path.is_file() {
        bail!("bad file");
    }

    let normalized = exe_path.to_str().unwrap_or_default().to_lowercase();
    if normalized.contains("\\system32\\") {
        bail!("sys32");
    }

    Ok(exe_path)
}
