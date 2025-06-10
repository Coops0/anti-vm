use std::collections::HashMap;

use wmi::{COMLibrary, Variant, WMIConnection};

pub fn check_if_graphics_card() -> anyhow::Result<bool> {
    // https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/win32-videocontroller
        let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con.into())?;

    let graphics_cards = wmi_con.raw_query::<HashMap<String, Variant>>("SELECT * FROM Win32_VideoController")?;
    println!("Graphics cards {graphics_cards:?}");
    

    Ok(true)
}