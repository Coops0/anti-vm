use windows::Win32::Security::Authentication::Identity::{
    SL_GEN_STATE_INVALID_LICENSE, SL_GEN_STATE_IS_GENUINE, SLIsGenuineLocal,
};
use windows_core::GUID;

const RPC: GUID = GUID::from_values(
    0x55c92734,
    0xd682,
    0x4d71,
    [0x98, 0x3e, 0xd6, 0xec, 0x3f, 0x16, 0x05, 0x9f],
);

pub fn check_is_activated() -> anyhow::Result<bool> {
    let mut sl_genuine_state = SL_GEN_STATE_INVALID_LICENSE;

    unsafe {
        SLIsGenuineLocal(&RPC, &raw mut sl_genuine_state, None)?;
    }

    Ok(sl_genuine_state == SL_GEN_STATE_IS_GENUINE)
}
