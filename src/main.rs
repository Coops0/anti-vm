#![feature(stmt_expr_attributes)]

use windows::Win32::System::Com::CoInitialize;

use crate::{
    activated::check_is_activated, auto_logon::is_auto_logon_enabled, battery::get_battery,
    bluetooth_adapters::score_bluetooth_adapters, displays::score_displays, flags::Flags,
    graphics_card::score_graphics_cards, installed_apps::score_installed_apps,
    microsoft_account::has_microsoft_account, os::score_os, registry::score_registry,
    sysinfo::score_sysinfo, system_devices::score_system_devices, usb_devices::score_usb_devices,
    various_wmi::score_various_wmi, wifi_adapters::score_wifi_adapters,
};

mod activated;
mod auto_logon;
mod battery;
mod bluetooth_adapters;
mod displays;
mod flags;
mod graphics_card;
mod installed_apps;
mod microsoft_account;
mod os;
mod registry;
mod registry_macros;
mod sysinfo;
mod system_devices;
mod usb_devices;
mod util;
mod various_wmi;
mod wifi_adapters;

// TODO check across many (real) systems
// TODO check across virtual box, hyperv, (and maybe even UTM?)

// TODO use obfstr

// TODO strip binary with build step too
// TODO get rid of unused windows crate features
// TODO!!! IS WIN32 THERAD SAFE????
fn main() {
    let start = std::time::Instant::now();
    let mut flags = Flags::new();

    let ret = unsafe { CoInitialize(None) };
    if ret.is_err() {
        debug_println!("WARNING: Failed to initialize COM: {ret:?}");
    }

    // VERY SLOW CHECK: Takes 150-400ms
    let system_devices_t = std::thread::spawn(|| {
        let mut f = Flags::new();
        if inspect!("system devices", score_system_devices(&mut f)).is_err() {
            f.large_penalty();
        }
        f
    });

    // SLOW CHECK: Takes 60-150ms
    // BUT: CANNOT be threaded, because it uses COM
    let os_t = std::thread::spawn(|| {
        let mut f = Flags::new();
        if inspect!("os", score_os(&mut f)).is_err() {
            f.large_penalty();
        }
        f
    });

    // SLOW CHECK: Takes ~40-400ms
    let installed_apps_t = std::thread::spawn(|| {
        let mut f = Flags::new();
        if inspect!("installed apps", score_installed_apps(&mut f)).is_err() {
            f.large_penalty();
        }
        f
    });

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

    // SLOW CHECK: Takes ~66ms
    if inspect!("micorosft account", has_microsoft_account()).unwrap_or_default() {
        flags.large_bonus();
    } else {
        flags.small_penalty();
    }

    if inspect!("activated", check_is_activated()).unwrap_or_default() {
        flags.small_bonus();
    } else {
        flags.medium_penalty();
    }

    // this can be spoofed, and either way laptops can have discrete graphics cards
    if inspect!("graphics card", score_graphics_cards(&mut flags)).is_err() {
        flags.medium_penalty();
    }

    if inspect!("bluetooth", score_bluetooth_adapters(&mut flags)).is_err() {
        flags.large_penalty();
    }

    // SLOW CHECK: Takes ~53ms
    if inspect!("various wmi", score_various_wmi(&mut flags)).is_err() {
        flags.large_penalty();
    }

    if inspect!("auto logon", is_auto_logon_enabled()).unwrap_or_default() {
        flags.medium_penalty();
    }

    match system_devices_t.join() {
        Ok(mut f) => flags.merge(&mut f),
        Err(why) => {
            debug_println!("failed to join system devices thread: {why:?}");
        }
    }

    match os_t.join() {
        Ok(mut f) => flags.merge(&mut f),
        Err(why) => {
            debug_println!("failed to join os thread: {why:?}");
        }
    }

    match installed_apps_t.join() {
        Ok(mut f) => flags.merge(&mut f),
        Err(why) => {
            debug_println!("failed to join installed apps thread: {why:?}");
        }
    }

    debug_println!("penalties: {:?}", flags.penalties());
    debug_println!("bonuses: {:?}", flags.bonuses());

    println!("score: {}", flags.score());

    println!("TOTAL EXECUTION TIME: {}ms", start.elapsed().as_millis());
}
