use std::ptr::null;

use windows::core::GUID;
use windows::Win32::Devices::PortableDevices::{self, IPortableDeviceManager};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows_core::PWSTR;

use crate::flags::Flags;
// 0x0af10cec, 0x2ecd, 0x4b92, 0x95, 0x81, 0x34, 0xf6, 0xae, 0x06, 0x37, 0xf3
const CLSID_PORTABLE_DEVICE_MANAGER: GUID = GUID::from_values(
    0x0af10cec,
    0x2ecd,
    0x4b92,
    [0x95, 0x81, 0x34, 0xf6, 0xae, 0x06, 0x37, 0xf3],
);

pub fn score_system_devices(flags: &mut Flags) -> anyhow::Result<()> {
    //     HRESULT hr = CoCreateInstance(CLSID_PortableDeviceManager,
    //                               NULL,
    //                               CLSCTX_INPROC_SERVER,
    //                               IID_PPV_ARGS(&pPortableDeviceManager));
    // if (FAILED(hr))
    // {
    //     printf("! Failed to CoCreateInstance CLSID_PortableDeviceManager, hr = 0x%lx\n",hr);
    // }

    // rclsid, punkouter, dwclscontext)
    let device_manager: IPortableDeviceManager =
        unsafe { CoCreateInstance(&CLSID_PORTABLE_DEVICE_MANAGER, None, CLSCTX_INPROC_SERVER)? };

    let mut c_pn_pdevice_ids = 0u32;
    let mut p_pn_pdevice_ids = PWSTR::default();
    unsafe {
        device_manager.GetDevices(&mut p_pn_pdevice_ids, &mut c_pn_pdevice_ids)?;
    }

    println!("number of system devices: {}", c_pn_pdevice_ids);

    Ok(())
}
