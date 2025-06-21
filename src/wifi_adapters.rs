use std::{ptr::null_mut};

use anyhow::bail;
use windows::{
    Win32::{
        Foundation::{ERROR_SUCCESS, HANDLE},
    },
};

use windows::Win32::NetworkManagement::WiFi::{
    WLAN_INTERFACE_INFO_LIST, WlanEnumInterfaces, WlanOpenHandle,
};
use windows_core::{Free, GUID};

use crate::{debug_println, flags::Flags};

pub fn score_wifi_adapters(flags: &mut Flags) -> anyhow::Result<()> {
    let devices_len = get_w_lan_devices(flags)?;
    debug_println!("found {devices_len} valid WiFi adapters");

    match devices_len {
        0 => flags.medium_penalty(),
        _ => flags.medium_bonus(),
    }

    Ok(())
}

const DW_MAX_CLIENT: u32 = 2;

fn get_w_lan_devices(flags: &mut Flags) -> anyhow::Result<usize> {
    let mut h_client = HandleWrapper::new();
    let mut dw_cur_version = 0;

    let mut p_if_list: *mut WLAN_INTERFACE_INFO_LIST = null_mut();

    let dw_result =
        unsafe { WlanOpenHandle(DW_MAX_CLIENT, None, &mut dw_cur_version, &mut h_client.0) };

    if dw_result != ERROR_SUCCESS.0 {
        bail!("WlanOpenHandle failed with error: {dw_result:?}");
    }

    let dw_result = unsafe {
        WlanEnumInterfaces(
            h_client.0,
            None,
            &raw mut p_if_list as *mut *mut WLAN_INTERFACE_INFO_LIST,
        )
    };

    if dw_result != ERROR_SUCCESS.0 {
        bail!("WlanEnumInterfaces failed with error: {dw_result:?}");
    }

    let len = unsafe { (*p_if_list).dwNumberOfItems } as usize;

    let devices_len = (0..len)
        .map(|i| unsafe { &(*p_if_list).InterfaceInfo[i] })
        .filter(|p_if_info| {
            if p_if_info.InterfaceGuid == GUID::zeroed() {
                flags.large_penalty();
                return false;
            }

            let description =
                String::from_utf16_lossy(&p_if_info.strInterfaceDescription).to_lowercase();
            if description.is_empty()
                || description.contains("vmware")
                || description.contains("virtualbox")
                || description.contains("vbox")
                || description.contains("hyper-v")
            {
                flags.large_penalty();
                return false;
            }

            true
        })
        .count();

    // wlan_interface_state_ad_hoc_network_formed = 2
    // wlan_interface_state_associating = 5
    // wlan_interface_state_authenticating = 7
    // wlan_interface_state_connected = 1
    // wlan_interface_state_disconnected = 4
    // wlan_interface_state_disconnecting = 3
    // wlan_interface_state_discovering = 6
    // wlan_interface_state_not_ready = 0

    Ok(devices_len)
}

struct HandleWrapper(pub HANDLE);

impl HandleWrapper {
    fn new() -> Self {
        Self(unsafe { core::mem::zeroed() })
    }
}

impl Drop for HandleWrapper {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                self.0.free();
            }
        }
    }
}
