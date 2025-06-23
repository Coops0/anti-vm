use windows_registry::LOCAL_MACHINE;

pub fn is_auto_logon_enabled() -> anyhow::Result<bool> {
    let key = LOCAL_MACHINE.open("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon")?;
    let value = key.get_string("AutoAdminLogon")?;
    
    Ok(value == "1")
}