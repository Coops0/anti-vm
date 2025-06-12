use crate::{
    activated::check_is_activated, battery::get_battery,
    bluetooth_adapters::check_if_bluetooth_adapter, displays::score_displays, flags::Flags,
    graphics_card::score_graphics_cards, local::get_is_local_account, os::score_os,
    registry::score_registry, sysinfo::score_sysinfo, system_devices::score_system_devices,
    usb_devices::score_usb_devices, wifi_adapters::get_wifi_adapters_len,
};

mod activated;
mod battery;
mod bluetooth_adapters;
mod displays;
mod flags;
mod graphics_card;
mod local;
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

    if inspect!("local account", get_is_local_account()).unwrap_or_default() {
        flags.medium_penalty();
    } else {
        flags.large_bonus();
    }

    if !inspect!("activated", check_is_activated()).unwrap_or_default() {
        flags.medium_penalty();
    }

    // this can be spoofed, and laptops can have discrete graphics cards
    if inspect!("graphics card", score_graphics_cards(&mut flags)).is_err() {
        flags.medium_penalty();
    }

    if !inspect!("bluetooth", check_if_bluetooth_adapter()).unwrap_or_default() {
        flags.large_penalty();
    }

    println!("penalties: {:?}", flags.penalties());
    println!("bonuses: {:?}", flags.bonuses());

    println!("score: {}", flags.score());

    Ok(())
}
