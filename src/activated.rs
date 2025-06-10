use windows::Win32::System::WindowsProgramming::{self, IEditionUpgradeBroker, IEditionUpgradeHelper, IEditionUpgradeHelper_Vtbl};
pub fn check_is_activated() -> anyhow::Result<bool> {
    unsafe {
        // IEditionUpgradeHelper_Impl::GetGenuineLocalStatus();
        // let _ = IEditionUpgradeBroker::;
        // let helper = IEditionUpgradeHelper::new();
    }

    Ok(true)
}
