use std::sync::Arc;

use relm4::ComponentSender;
use tokio_util::sync::CancellationToken;
use wayle_config::ConfigService;
use wayle_iwd::Station;
use wayle_widgets::{watch, watch_cancellable};

use crate::shell::bar::dropdowns::iwd::available_networks::{
    AvailableNetworks, messages::AvailableNetworksCmd,
};

pub(super) fn spawn(
    sender: &ComponentSender<AvailableNetworks>,
    station: &Arc<Station>,
    token: CancellationToken,
) {
    let networks = station.networks.clone();
    let connection = station.connection.clone();
    watch_cancellable!(
        sender,
        token,
        [networks.watch(), connection.watch()],
        |out| {
            let _ = out.send(AvailableNetworksCmd::NetworksChanged);
        }
    );
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
