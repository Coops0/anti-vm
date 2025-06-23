use wmi::{COMLibrary, WMIConnection};

pub fn has_microsoft_account() -> anyhow::Result<bool> {
    let com_con = unsafe { COMLibrary::assume_initialized() };
    let wmi_con = WMIConnection::new(com_con)?;

    let accounts = wmi_con
        .raw_query::<()>("SELECT LocalAccount FROM Win32_Account WHERE LocalAccount = FALSE")?;
    Ok(!accounts.is_empty())
}
