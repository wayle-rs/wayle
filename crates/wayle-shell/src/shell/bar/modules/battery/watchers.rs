use std::sync::Arc;

use relm4::ComponentSender;
use wayle_battery::BatteryService;
use wayle_config::schemas::modules::BatteryConfig;
use wayle_widgets::watch;

use super::{
    BatteryModule,
    helpers::{IconContext, format_label, select_icon},
    messages::BatteryCmd,
};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<BatteryModule>,
    config: &BatteryConfig,
    battery: &Arc<BatteryService>,
) {
    let level_icons = config.level_icons.clone();
    let charging_icon = config.charging_icon.clone();
    let alert_icon = config.alert_icon.clone();
    let format = config.format.clone();

    let device = battery.device.clone();

    let percentage_stream = device.percentage.watch();
    let state_stream = device.state.watch();
    let is_present_stream = device.is_present.watch();
    let level_icons_stream = level_icons.watch();
    let charging_icon_stream = charging_icon.watch();
    let alert_icon_stream = alert_icon.watch();
    let format_stream = format.watch();

    watch!(
        sender,
        [
            percentage_stream,
            state_stream,
            is_present_stream,
            level_icons_stream,
            charging_icon_stream,
            alert_icon_stream,
            format_stream
        ],
        |out| {
            let percentage = device.percentage.get();
            let state = device.state.get();
            let is_present = device.is_present.get();

            let label = format_label(&format.get(), percentage, is_present);
            let _ = out.send(BatteryCmd::UpdateLabel(label));

            let level_icons_val = level_icons.get();
            let charging_icon_val = charging_icon.get();
            let alert_icon_val = alert_icon.get();
            let icon = select_icon(&IconContext {
                percentage,
                state,
                is_present,
                level_icons: &level_icons_val,
                charging_icon: &charging_icon_val,
                alert_icon: &alert_icon_val,
            });
            let _ = out.send(BatteryCmd::UpdateIcon(icon));
        }
    );
}
