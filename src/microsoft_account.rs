use wmi::{COMLibrary, WMIConnection};

pub fn has_microsoft_account() -> anyhow::Result<bool> {
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con)?;

    let accounts =
        wmi_con.raw_query::<()>("SELECT LocalAccount FROM Win32_Account WHERE LocalAccount = FALSE")?;
    Ok(!accounts.is_empty())
}
