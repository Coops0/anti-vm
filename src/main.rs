use crate::{
    battery::get_battery, first_logon_time::days_since_installation, flags::Flags, util::inspect, wifi_adapters::get_wifi_adapters_len
};

mod battery;
mod displays;
mod first_logon_time;
mod flags;
mod wifi_adapters;
mod util;

// TODO minify binary for size
// TODO strip binary with build step too
fn main() -> anyhow::Result<()> {
    let mut flags = Flags::new();

    match days_since_installation() {
        Some(days) => match days as u64 {
            0 => flags.extreme_penalty(),
            1..=6 => flags.large_penalty(),
            // Okay...
            7..=60 => {}
            _ => flags.small_bonus(),
        },
        None => {
            println!("Error getting first logon time");
            flags.large_penalty();
        }
    }

    match get_wifi_adapters_len() {
        Ok(len) => match len {
            0 => flags.medium_penalty(),
            1 => flags.medium_bonus(),
            _ => flags.large_bonus(),
        },
        Err(_) => flags.medium_penalty(),
    };

    match displays::score_displays(&mut flags) {
        Ok(()) => {}
        Err(_) => flags.large_penalty(),
    }

    if get_battery().unwrap_or_default() {
        flags.extreme_bonus();
    }

    println!("Score: {}", flags.score());

    Ok(())
}
