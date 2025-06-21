#![feature(stmt_expr_attributes)]

use crate::{
    activated::check_is_activated, battery::get_battery,
    bluetooth_adapters::check_if_bluetooth_adapter, displays::score_displays, flags::Flags,
    graphics_card::score_graphics_cards, local::get_is_local_account, os::score_os,
    registry::score_registry, sysinfo::score_sysinfo, system_devices::score_system_devices,
    usb_devices::score_usb_devices, wifi_adapters::score_wifi_adapters,
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

// TODO strip binary with build step too
// TODO get rid of unused windows crate features
fn main() {
    let start = std::time::Instant::now();
    let mut flags = Flags::new();

    let system_devices_t = std::thread::spawn(|| {
        let mut flags = Flags::new();
        if inspect!("system devices", score_system_devices(&mut flags)).is_err() {
            flags.large_penalty();
        }

        flags
    });

    if inspect!("os", score_os(&mut flags)).is_err() {
        flags.large_penalty();
    }

    if inspect!("wifi adapters", score_wifi_adapters(&mut flags)).is_err() {
        flags.medium_penalty();
    }

    if inspect!("displays", score_displays(&mut flags)).is_err() {
        flags.large_penalty();
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

    score_registry(&mut flags);

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

    match system_devices_t.join() {
        Ok(mut system_devices_flags) => flags.merge(&mut system_devices_flags),
        Err(why) => {
            debug_println!("failed to join system devices thread: {why:?}");
        }
    }

    debug_println!("penalties: {:?}", flags.penalties());
    debug_println!("bonuses: {:?}", flags.bonuses());

    println!("score: {}", flags.score());

    println!("TOTAL EXECUTION TIME: {}ms", start.elapsed().as_millis());
}
