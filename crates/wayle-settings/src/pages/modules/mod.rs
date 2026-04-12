//! Per-module settings pages. Each module exports an `entry()` returning a `LeafEntry`.

mod battery;
mod bluetooth;
mod cava;
mod clock;
mod cpu;
mod custom;
mod dashboard;
mod hyprland_workspaces;
mod hyprsunset;
mod idle_inhibit;
mod keybind_mode;
mod keyboard_input;
mod media;
mod microphone;
mod netstat;
mod network;
mod notification_module;
mod power;
mod ram;
mod separator;
mod storage;
mod systray;
mod volume;
mod weather;
mod window_title;
mod world_clock;

use wayle_config::Config;

use super::nav::LeafEntry;

pub(crate) fn all_entries(config: &Config) -> Vec<LeafEntry> {
    vec![
        battery::entry(config),
        bluetooth::entry(config),
        cava::entry(config),
        clock::entry(config),
        cpu::entry(config),
        custom::entry(config),
        dashboard::entry(config),
        hyprland_workspaces::entry(config),
        hyprsunset::entry(config),
        idle_inhibit::entry(config),
        keybind_mode::entry(config),
        keyboard_input::entry(config),
        media::entry(config),
        microphone::entry(config),
        netstat::entry(config),
        network::entry(config),
        notification_module::entry(config),
        power::entry(config),
        ram::entry(config),
        separator::entry(config),
        storage::entry(config),
        systray::entry(config),
        volume::entry(config),
        weather::entry(config),
        window_title::entry(config),
        world_clock::entry(config),
    ]
}
