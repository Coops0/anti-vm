use wmi::{COMLibrary, WMIConnection};

pub fn get_is_local_account() -> anyhow::Result<bool> {
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con)?;

    let accounts = wmi_con.raw_query::<()>("SELECT * FROM Win32_Account WHERE LocalAccount = TRUE")?;
    Ok(!accounts.is_empty())
}
