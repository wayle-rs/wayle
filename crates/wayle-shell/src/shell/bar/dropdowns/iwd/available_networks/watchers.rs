use std::sync::Arc;

use relm4::ComponentSender;
use tokio_util::sync::CancellationToken;
use wayle_config::ConfigService;
use wayle_iwd::{IwdService, Station};
use wayle_widgets::{watch, watch_cancellable};

use crate::shell::bar::dropdowns::iwd::available_networks::{
    AvailableNetworks, messages::AvailableNetworksCmd,
};

pub(super) fn spawn(
    sender: &ComponentSender<AvailableNetworks>,
    station: &Arc<Station>,
    token: CancellationToken,
) {
    let powered = station.powered.clone();
    let networks = station.networks.clone();
    let connection = station.connection.clone();
    // A hardware block shows only as a failed power request.
    let last_error = station.last_error.clone();
    watch_cancellable!(
        sender,
        token,
        [
            powered.watch(),
            networks.watch(),
            connection.watch(),
            last_error.watch()
        ],
        |out| {
            let _ = out.send(AvailableNetworksCmd::NetworksChanged);
        }
    );
}

/// Follow IWD's device appearing, going away or being replaced.
pub(super) fn spawn_station_watcher(
    sender: &ComponentSender<AvailableNetworks>,
    iwd: &Arc<IwdService>,
) {
    let station = iwd.station.clone();
    watch!(sender, [station.watch()], |out| {
        let _ = out.send(AvailableNetworksCmd::StationChanged);
    });
}

/// Follow IWD's passphrase requests, which the password form shows.
pub(super) fn spawn_passphrase_watcher(
    sender: &ComponentSender<AvailableNetworks>,
    iwd: &Arc<IwdService>,
) {
    let request = iwd.passphrase_request.clone();
    watch!(sender, [request.watch()], |out| {
        let _ = out.send(AvailableNetworksCmd::PassphraseRequestChanged);
    });
}

/// Rebuild the list when the configured signal icons change at runtime.
pub(super) fn spawn_config_watchers(
    sender: &ComponentSender<AvailableNetworks>,
    config: &Arc<ConfigService>,
) {
    let icons = &config.config().modules.iwd;
    let signal = icons.wifi_signal_icons.clone();
    let connected = icons.wifi_connected_icon.clone();
    let disabled = icons.wifi_disabled_icon.clone();

    watch!(
        sender,
        [signal.watch(), connected.watch(), disabled.watch()],
        |out| {
            let _ = out.send(AvailableNetworksCmd::ConfigChanged);
        }
    );
}
