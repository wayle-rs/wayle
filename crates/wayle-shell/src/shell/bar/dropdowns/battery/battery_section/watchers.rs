use std::sync::Arc;

use relm4::ComponentSender;
use wayle_widgets::watch;

use super::{BatterySection, messages::BatterySectionCmd};
use crate::services::BatteryService;

pub(super) fn spawn(sender: &ComponentSender<BatterySection>, battery: &Arc<BatteryService>) {
    let device = &battery.device;

    let percentage = device.percentage.clone();
    let state = device.state.clone();
    let time_to_empty = device.time_to_empty.clone();
    let time_to_full = device.time_to_full.clone();
    let energy_rate = device.energy_rate.clone();
    let energy = device.energy.clone();
    let energy_full = device.energy_full.clone();
    let capacity = device.capacity.clone();
    let warning_level = device.warning_level.clone();
    let is_present = device.is_present.clone();
    let charge_end_threshold = device.charge_end_threshold.clone();
    let charge_threshold_supported = device.charge_threshold_supported.clone();
    let charge_threshold_enabled = device.charge_threshold_enabled.clone();

    watch!(
        sender,
        [
            percentage.watch(),
            state.watch(),
            time_to_empty.watch(),
            time_to_full.watch(),
            energy_rate.watch(),
            energy.watch(),
            energy_full.watch(),
            capacity.watch(),
            warning_level.watch(),
            is_present.watch(),
            charge_end_threshold.watch(),
            charge_threshold_supported.watch(),
            charge_threshold_enabled.watch()
        ],
        |out| {
            let _ = out.send(BatterySectionCmd::BatteryStateChanged);
        }
    );
}
