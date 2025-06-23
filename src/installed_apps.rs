use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};
use windows::Win32::{
    Foundation::MAX_PATH, Globalization::CP_ACP, System::Com::IPersistFile, UI::Shell::IShellLinkW,
};
use windows::Win32::{
    Globalization::{MB_COMPOSITE, MULTI_BYTE_TO_WIDE_CHAR_FLAGS, MultiByteToWideChar},
    System::Com::STGM_READ,
};
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
        if ext != "lnk" && ext != "url" {
            continue;
        }

        if ext == "lnk" {
        } else if ext == "url" {
            if !validate_url(&p).unwrap_or_default() {
                continue;
            }
        } else {
            continue;
        }

        println!("found installed app: {n} ({t})");
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
    // URL=steam://rungameid/1222670
    // IconFile

    let ret = match (
        lines.clone().find_map(|line| line.strip_prefix("URL=")),
        lines.find_map(|line| line.strip_prefix("IconFile=")),
    ) {
        (None, None) => false,
        (None, Some(icon)) if icon.is_empty() => false,
        (None, Some(icon)) => fs::exists(Path::new(icon)).is_ok_and(|x| x),
        (Some(url), None) if url.is_empty() => false,
        _ => true,
    };

    Some(ret)
}

const CLSID_SHELL_LINK: GUID = GUID::from_values(
    0x000214EE,
    0x0000,
    0x0000,
    [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
);

// IID_I_PERSIST_FILE, 0x0000010B, 0x0000, 0x0000, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46
const IID_I_PERSIST_FILE: GUID = GUID::from_values(
    0x0000010b,
    0x0000,
    0x0000,
    [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
);

fn validate_lnk(path: &Path) -> anyhow::Result<bool> {
    //     HRESULT ResolveIt(HWND hwnd, LPCSTR lpszLinkFile, LPWSTR lpszPath, int iPathBufferSize)
    // {
    //     HRESULT hres;
    //     IShellLink* psl;
    //     WCHAR szGotPath[MAX_PATH];
    //     WCHAR szDescription[MAX_PATH];
    //     WIN32_FIND_DATA wfd;

    //     *lpszPath = 0; // Assume failure

    let ret = unsafe { CoInitialize(None) };

    if ret.is_err() {
        bail!("Failed to initialize COM: {ret:?}");
    }

    // let psl: IShellLinkW =
    // unsafe { CoCreateInstance(&CLSID_SHELL_LINK, None, CLSCTX_INPROC_SERVER)? };

    let ppf: IPersistFile =
        unsafe { CoCreateInstance(&IID_I_PERSIST_FILE, None, CLSCTX_INPROC_SERVER)? };

    //     // Get a pointer to the IShellLink interface. It is assumed that CoInitialize
    //     // has already been called.
    //     hres = CoCreateInstance(CLSID_ShellLink, NULL, CLSCTX_INPROC_SERVER, IID_IShellLink, (LPVOID*)&psl);
    //     if (SUCCEEDED(hres))
    //     {
    //         IPersistFile* ppf;

    //         // Get a pointer to the IPersistFile interface.
    //         hres = psl->QueryInterface(IID_IPersistFile, (void**)&ppf);

    //         if (SUCCEEDED(hres))
    //         {
    //             WCHAR wsz[MAX_PATH];

    let mut wsz = [0u16; MAX_PATH as usize];
    let lpsz_link_file = path.to_str().context("nps")?;
    // pub unsafe fn MultiByteToWideChar(codepage: u32, dwflags: MULTI_BYTE_TO_WIDE_CHAR_FLAGS, lpmultibytestr: &[u8], lpwidecharstr: Option<&mut [u16]>) -> i32 {
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
    //             // Ensure that the string is Unicode.
    //             MultiByteToWideChar(CP_ACP, 0, lpszLinkFile, -1, wsz, MAX_PATH);

    //             // Add code here to check return value from MultiByteWideChar
    //             // for success.

    //             // Load the shortcut.
    //             hres = ppf->Load(wsz, STGM_READ);

    let wsz_pcwstr = PCWSTR::from_raw(wsz.as_ptr());
    unsafe {
        ppf.Load(wsz_pcwstr, STGM_READ)?;
    }

    //             if (SUCCEEDED(hres))
    //             {
    //                 // Resolve the link.
    //                 hres = psl->Resolve(hwnd, 0);

    //                 if (SUCCEEDED(hres))
    //                 {
    //                     // Get the path to the link target.
    //                     hres = psl->GetPath(szGotPath, MAX_PATH, (WIN32_FIND_DATA*)&wfd, SLGP_SHORTPATH);

    //                     if (SUCCEEDED(hres))
    //                     {
    //                         // Get the description of the target.
    //                         hres = psl->GetDescription(szDescription, MAX_PATH);

    //                         if (SUCCEEDED(hres))
    //                         {
    //                             hres = StringCbCopy(lpszPath, iPathBufferSize, szGotPath);
    //                             if (SUCCEEDED(hres))
    //                             {
    //                                 // Handle success
    //                             }
    //                             else
    //                             {
    //                                 // Handle the error
    //                             }
    //                         }
    //                     }
    //                 }
    //             }

    //             // Release the pointer to the IPersistFile interface.
    //             ppf->Release();
    //         }

    //         // Release the pointer to the IShellLink interface.
    //         psl->Release();
    //     }
    //     return hres;
    // }

    todo!();
}
