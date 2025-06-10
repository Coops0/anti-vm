use crate::{
    battery::get_battery, displays::score_displays, flags::Flags, os::score_os,
    registry::score_registry, sysinfo::score_sysinfo, system_devices::score_system_devices,
    usb_devices::score_usb_devices, wifi_adapters::get_wifi_adapters_len,
};

mod activated;
mod battery;
mod displays;
mod flags;
mod os;
mod registry;
mod registry_macros;
mod sysinfo;
mod system_devices;
mod usb_devices;
mod util;
mod wifi_adapters;

// TODO check across many (real) systems
// TODO check across virtual box, hyperv, (and maybe even UTM?)

// TODO use obfstr
// TODO transition all prints to debug_println!
// TODO Can fully remove all #[derive(Debug)] strings??

// TODO strip binary with build step too
// TODO setup clippy checks
fn main() -> anyhow::Result<()> {
    let mut flags = Flags::new();

    if inspect!("os", score_os(&mut flags)).is_err() {
        flags.large_penalty();
    }

    match inspect!("# wifi adapters", get_wifi_adapters_len()) {
        Ok(len) => match len {
            0 => flags.medium_penalty(),
            1 => flags.medium_bonus(),
            _ => flags.large_bonus(),
        },
        Err(_) => flags.medium_penalty(),
    };

    match inspect!("displays", score_displays(&mut flags)) {
        Ok(()) => {}
        Err(_) => flags.large_penalty(),
    }

    if inspect!("battery info", get_battery()).unwrap_or_default() {
        flags.extreme_bonus();
    }

    if inspect!("sysinfo", score_sysinfo(&mut flags)).is_err() {
        flags.large_penalty();
    }

    if inspect!("usb devices", score_usb_devices(&mut flags)).is_err() {
        flags.large_penalty();
    }

    if inspect!("system devices", score_system_devices(&mut flags)).is_err() {
        flags.large_penalty();
    }

    if inspect!("registry", score_registry(&mut flags)).is_err() {
        flags.large_penalty();
    }

    println!("penalties: {:?}", flags.penalties());
    println!("bonuses: {:?}", flags.bonuses());

    println!("score: {}", flags.score());

    Ok(())
}
