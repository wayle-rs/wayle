use std::sync::Arc;

use relm4::ComponentSender;
use tokio_util::sync::CancellationToken;
use wayle_config::schemas::modules::IwdConfig;
use wayle_iwd::IwdService;
use wayle_widgets::{watch, watch_cancellable};

use super::{IwdModule, messages::IwdCmd};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<IwdModule>,
    config: &IwdConfig,
    iwd: &Arc<IwdService>,
) {
    let station = iwd.station.clone();
    watch!(sender, [station.watch()], |out| {
        let _ = out.send(IwdCmd::StationDeviceChanged);
    });

    spawn_icon_config_watchers(sender, config);
}

pub(super) fn spawn_station_watchers(
    sender: &ComponentSender<IwdModule>,
    iwd: &Arc<IwdService>,
    token: CancellationToken,
) {
    let Some(station) = iwd.station.get() else {
        return;
    };

    let powered = station.powered.clone();
    let connection = station.connection.clone();
    let strength = station.strength.clone();

    watch_cancellable!(
        sender,
        token,
        [powered.watch(), connection.watch(), strength.watch()],
        |out| {
            let _ = out.send(IwdCmd::StateChanged);
        }
    );
}

fn spawn_icon_config_watchers(sender: &ComponentSender<IwdModule>, config: &IwdConfig) {
    let wifi_disabled_icon = config.wifi_disabled_icon.clone();
    let wifi_acquiring_icon = config.wifi_acquiring_icon.clone();
    let wifi_offline_icon = config.wifi_offline_icon.clone();
    let wifi_connected_icon = config.wifi_connected_icon.clone();
    let wifi_signal_icons = config.wifi_signal_icons.clone();

    watch!(
        sender,
        [
            wifi_disabled_icon.watch(),
            wifi_acquiring_icon.watch(),
            wifi_offline_icon.watch(),
            wifi_connected_icon.watch(),
            wifi_signal_icons.watch()
        ],
        |out| {
            let _ = out.send(IwdCmd::IconConfigChanged);
        }
    );
}
