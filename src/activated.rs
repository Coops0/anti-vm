use std::ptr::null_mut;

use anyhow::bail;
use windows::Win32::Security::Authentication::Identity::{
    SL_GEN_STATE_INVALID_LICENSE, SL_GEN_STATE_IS_GENUINE, SLClose, SLGetSLIDList, SLIsGenuineLocal,
};
use windows::Win32::Security::Authentication::Identity::{
    SL_ID_APPLICATION, SL_ID_PRODUCT_SKU, SLOpen,
};
use windows_core::GUID;

use crate::debug_println;

const RPC: GUID = GUID::from_values(
    0x55c92734,
    0xd682,
    0x4d71,
    [0x98, 0x3e, 0xd6, 0xec, 0x3f, 0x16, 0x05, 0x9f],
);

pub fn check_is_activated() -> anyhow::Result<bool> {
    get_activation_type().unwrap();
    let mut sl_genuine_state = SL_GEN_STATE_INVALID_LICENSE;

    unsafe {
        SLIsGenuineLocal(&RPC, &raw mut sl_genuine_state, None)?;
    }

    Ok(sl_genuine_state == SL_GEN_STATE_IS_GENUINE)
}

enum ActivationType {
    Unlicensed,
    LikelyGenuine,
    Pirated,
}

// 55c92734-d682-4d71-983e-d6ec3f16059f
const WIN_APP_GUID: GUID = GUID::from_values(
    0x55c92734,
    0xd682,
    0x4d71,
    [0x98, 0x3e, 0xd6, 0xec, 0x3f, 0x16, 0x05, 0x9f],
);

fn get_activation_type() -> anyhow::Result<ActivationType> {
    let mut hslc = HslcManager::new();
    unsafe {
        hslc.open()?;
    }

    if hslc.0.is_null() {
        bail!("hslc null");
    }

    debug_println!("opened");

    // pub unsafe fn SLGetSLIDList(
    //     hslc: *const c_void,
    //     equeryidtype: SLIDTYPE,
    //     pqueryid: Option<*const GUID>,
    //     ereturnidtype: SLIDTYPE,
    //     pnreturnids: *mut u32,
    //     ppreturnids: *mut *mut GUID,
    // ) -> Result<()>

    let mut pn_return_ids = 0u32;
    let mut pp_return_ids: *mut GUID = null_mut();
    unsafe {
        SLGetSLIDList(
            hslc.0,
            SL_ID_APPLICATION,
            Some(&WIN_APP_GUID),
            SL_ID_PRODUCT_SKU,
            &raw mut pn_return_ids,
            &mut pp_return_ids
        )?;
    }

    debug_println!("SLGetSLIDList returned {pn_return_ids} IDs");

    for i in 0..pn_return_ids {
        let guid = unsafe { *pp_return_ids.add(i as usize) };
        println!("GUID {i}: {guid:?}"); 
    } 

    todo!();
}

struct HslcManager(pub *mut core::ffi::c_void);

impl HslcManager {
    pub fn new() -> Self {
        Self(unsafe { core::mem::zeroed() })
    }

    pub unsafe fn open(&mut self) -> windows_core::Result<()> {
        unsafe { SLOpen(&raw mut self.0) }
    }
}

impl Drop for HslcManager {
    fn drop(&mut self) {
        debug_println!("dropping");
        let _ = unsafe { SLClose(self.0) };
    }
}

// https://github.com/massgravel/Microsoft-Activation-Scripts/blob/5ad6226f37e7af75807819d669ff5ae0e3421a87/MAS/Separate-Files-Version/Activators/HWID_Activation.cmd#L1751
// https://github.com/massgravel/Microsoft-Activation-Scripts/blob/5ad6226f37e7af75807819d669ff5ae0e3421a87/MAS/Separate-Files-Version/Activators/HWID_Activation.cmd#L1829

// https://github.com/massgravel/Microsoft-Activation-Scripts/blob/5ad6226f37e7af75807819d669ff5ae0e3421a87/MAS/Separate-Files-Version/Activators/TSforge_Activation.cmd#L986
// https://github.com/massgravel/Microsoft-Activation-Scripts/blob/5ad6226f37e7af75807819d669ff5ae0e3421a87/MAS/Separate-Files-Version/Activators/TSforge_Activation.cmd#L1099

// https://github.com/massgravel/Microsoft-Activation-Scripts/blob/5ad6226f37e7af75807819d669ff5ae0e3421a87/MAS/Separate-Files-Version/Activators/Online_KMS_Activation.cmd#L3663
// https://github.com/massgravel/Microsoft-Activation-Scripts/blob/5ad6226f37e7af75807819d669ff5ae0e3421a87/MAS/Separate-Files-Version/Activators/Online_KMS_Activation.cmd#L3962

// https://github.com/massgravel/Microsoft-Activation-Scripts/blob/5ad6226f37e7af75807819d669ff5ae0e3421a87/MAS/Separate-Files-Version/Activators/KMS38_Activation.cmd#L1890
// https://github.com/massgravel/Microsoft-Activation-Scripts/blob/5ad6226f37e7af75807819d669ff5ae0e3421a87/MAS/Separate-Files-Version/Activators/KMS38_Activation.cmd#L1955
