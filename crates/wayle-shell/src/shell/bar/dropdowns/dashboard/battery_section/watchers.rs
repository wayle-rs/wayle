use std::sync::Arc;

use relm4::ComponentSender;
use wayle_battery::types::DeviceState;
use wayle_core::Property;
use wayle_power_profiles::PowerProfilesService;
use wayle_widgets::watch;

use super::{BatterySection, messages::BatterySectionCmd};
use crate::services::BatteryService;

pub(super) fn spawn(sender: &ComponentSender<BatterySection>, battery: &Arc<BatteryService>) {
    let device = battery.device.clone();
    let percentage = device.percentage.clone();
    let state = device.state.clone();
    let time_to_empty = device.time_to_empty.clone();
    let time_to_full = device.time_to_full.clone();

    watch!(
        sender,
        [
            percentage.watch(),
            state.watch(),
            time_to_empty.watch(),
            time_to_full.watch()
        ],
        |out| {
            let pct = percentage.get();
            let device_state = state.get();
            let seconds = match &device_state {
                DeviceState::Discharging => time_to_empty.get(),
                DeviceState::Charging => time_to_full.get(),
                _ => 0,
            };

            let _ = out.send(BatterySectionCmd::StateChanged {
                percentage: pct,
                state: device_state,
                time_remaining_secs: seconds,
            });
        }
    );
}

pub(super) fn spawn_power_profiles_watcher(
    sender: &ComponentSender<BatterySection>,
    power_profiles: &Property<Option<Arc<PowerProfilesService>>>,
) {
    let profiles_prop = power_profiles.clone();

    watch!(sender, [profiles_prop.watch()], |out| {
        match profiles_prop.get() {
            Some(service) => {
                let _ = out.send(BatterySectionCmd::PowerProfilesAvailable(service));
            }
            None => {
                let _ = out.send(BatterySectionCmd::PowerProfilesUnavailable);
            }
        }
    });
}

pub(super) fn spawn_active_profile_watcher(
    sender: &ComponentSender<BatterySection>,
    service: &Arc<PowerProfilesService>,
    token: tokio_util::sync::CancellationToken,
) {
    let active_profile = service.power_profiles.active_profile.clone();

    wayle_widgets::watch_cancellable!(sender, token, [active_profile.watch()], |out| {
        let _ = out.send(BatterySectionCmd::PowerProfileChanged(active_profile.get()));
    });
}
