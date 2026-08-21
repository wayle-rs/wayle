use std::sync::Arc;

use relm4::ComponentSender;
use tokio_util::sync::CancellationToken;
use wayle_config::ConfigService;
use wayle_iwd::IwdService;
use wayle_widgets::{watch, watch_cancellable};

use crate::shell::bar::dropdowns::iwd::active_connections::{
    ActiveConnections, messages::ActiveConnectionsCmd,
};

pub(super) fn spawn_station_watchers(
    sender: &ComponentSender<ActiveConnections>,
    iwd: &Arc<IwdService>,
    token: CancellationToken,
) {
    let Some(station) = iwd.station.get() else {
        return;
    };

    let connection = station.connection.clone();
    let strength = station.strength.clone();
    let frequency = station.frequency.clone();

    watch_cancellable!(
        sender,
        token,
        [connection.watch(), strength.watch(), frequency.watch()],
        |out| {
            let _ = out.send(ActiveConnectionsCmd::StateChanged {
                connection: connection.get(),
                strength: strength.get(),
                frequency: frequency.get(),
            });
        }
    );
}

pub(super) fn spawn_device_watchers(
    sender: &ComponentSender<ActiveConnections>,
    iwd: &Arc<IwdService>,
) {
    let station = iwd.station.clone();
    watch!(sender, [station.watch()], |out| {
        let _ = out.send(ActiveConnectionsCmd::StationDeviceChanged);
    });
}

/// Re-render the card when any configured connection icon changes at runtime.
pub(super) fn spawn_config_watchers(
    sender: &ComponentSender<ActiveConnections>,
    config: &Arc<ConfigService>,
) {
    let icons = &config.config().modules.iwd;
    let offline = icons.wifi_offline_icon.clone();
    let acquiring = icons.wifi_acquiring_icon.clone();
    let connected = icons.wifi_connected_icon.clone();
    let signal = icons.wifi_signal_icons.clone();

    watch!(
        sender,
        [
            offline.watch(),
            acquiring.watch(),
            connected.watch(),
            signal.watch()
        ],
        |out| {
            let _ = out.send(ActiveConnectionsCmd::ConfigChanged);
        }
    );
}
