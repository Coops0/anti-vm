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
    Storage::FileSystem::{WIN32_FIND_DATAA, WIN32_FIND_DATAW},
    System::Com::IPersistFile,
    UI::Shell::{IShellLinkA, IShellLinkW, SLGP_SHORTPATH},
};
use windows::Win32::{
    Globalization::{MB_COMPOSITE, MULTI_BYTE_TO_WIDE_CHAR_FLAGS, MultiByteToWideChar},
    System::Com::STGM_READ,
};
use windows::core::Interface;
use windows::{
    Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, CoInitialize},
    core::GUID,
};
use windows_core::PCWSTR;

use crate::flags::Flags;

pub fn score_installed_apps(flags: &mut Flags) -> anyhow::Result<()> {
    // C:\Users\cooper\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Steam
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

    for p in installed {
        // lnk, url,
        let Some(ext) = p
            .extension()
            .and_then(OsStr::to_str)
            .map(|ext| ext.to_lowercase())
        else {
            continue;
        };

        println!("app: {:?} {:?}", p.file_name(), p.display());
        if ext == "lnk" {
            // todo change to .unwrap_or_default()
            if !validate_lnk(&p).unwrap() {
                continue;
            }
        } else if ext == "url" {
            if !validate_url(&p).unwrap_or_default() {
                continue;
            }
        } else {
            continue;
        }
    }

    todo!();
    Ok(())
}

fn recurse_dir(dir: &Path, ret: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    for entry in dir.read_dir().context("Failed to read directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.is_dir() {
            let _ = recurse_dir(&path, ret);
        } else {
            ret.push(path);
        }
    }
    Ok(())
}

fn validate_url(path: &Path) -> Option<bool> {
    let s = fs::read_to_string(path).ok()?;

    let mut lines = s.lines();

    let ret = match (
        lines.clone().find_map(|line| line.strip_prefix("URL=")),
        lines.find_map(|line| line.strip_prefix("IconFile=")),
    ) {
        (None, None) => false,
        (Some(""), None) => false,
        (None, Some("")) => false,

        (Some(url), None) if url.contains("//") => true,
        (Some(url), None) => fs::exists(Path::new(url)).is_ok_and(|x| x),

        (None, Some(icon)) => fs::exists(Path::new(icon)).is_ok_and(|x| x),
        _ => true,
    };

    Some(ret)
}

const CLSID_SHELL_LINK: GUID = GUID::from_values(
    0x00021401,
    0x0000,
    0x0000,
    [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
);

fn validate_lnk(path: &Path) -> anyhow::Result<bool> {
    let ret = unsafe { CoInitialize(None) };

    if ret.is_err() {
        bail!("Failed to initialize COM: {ret:?}");
    }

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
        ppf.Load(wsz_pcwstr.clone(), STGM_READ)?;
    }

    let mut sz_got_path = [0u16; MAX_PATH as usize];
    // let mut wfd: WIN32_FIND_DATAW = unsafe { core::mem::zeroed() };

    unsafe {
        psl.GetPath(&mut sz_got_path, null_mut(), SLGP_SHORTPATH.0 as u32)?;
    }

    let sz_got_path_str = unsafe { PCWSTR::from_raw(sz_got_path.as_ptr()).to_string()? };

    let exe_path = PathBuf::from(sz_got_path_str);
    if !exe_path.exists() || !exe_path.is_file() {
        return Ok(false);
    }

    if exe_path.ancestors().any(|a| {
        let mut a = a.to_str().unwrap_or_default().to_lowercase();
        a = a.replace('\\', "");
        a = a.replace("/", "");
        a = a.trim().to_string();

        a == "system32"
    }) {
        println!("skipping system32 app: {}", exe_path.display());
        return Ok(false);
    }

    println!("valid app: {}", exe_path.display());
    Ok(true)
}
