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
mod aurora;

use wayle_config::Config;

use super::nav::LeafEntry;

pub(crate) fn factories() -> Vec<fn(&Config) -> LeafEntry> {
    vec![
        battery::entry,
        bluetooth::entry,
        cava::entry,
        clock::entry,
        cpu::entry,
        custom::entry,
        dashboard::entry,
        hyprland_workspaces::entry,
        hyprsunset::entry,
        idle_inhibit::entry,
        keybind_mode::entry,
        keyboard_input::entry,
        media::entry,
        microphone::entry,
        netstat::entry,
        network::entry,
        notification_module::entry,
        power::entry,
        ram::entry,
        separator::entry,
        storage::entry,
        systray::entry,
        volume::entry,
        weather::entry,
        window_title::entry,
        world_clock::entry,
        aurora::entry,
    ]
}
