use std::sync::Arc;

use relm4::ComponentSender;
use wayle_network::NetworkService;
use wayle_widgets::watch;

use super::{
    VpnConnections,
    messages::{VpnActiveInfo, VpnConnectionsCmd, VpnRowState},
};

pub(super) fn spawn(sender: &ComponentSender<VpnConnections>, network: &Arc<NetworkService>) {
    let vpn = network.vpn.clone();
    let connections = vpn.connections.clone();
    let active_connections = vpn.active_connections.clone();
    let connectivity = vpn.connectivity.clone();

    {
        let connections = connections.clone();
        let active_connections = active_connections.clone();
        watch!(
            sender,
            [
                connections.watch(),
                active_connections.watch(),
                connectivity.watch()
            ],
            |out| {
                let rows = build_rows(&connections.get(), &active_connections.get());
                let _ = out.send(VpnConnectionsCmd::RowsChanged(rows));
            }
        );
    }

    let banner = vpn.banner.clone();
    watch!(sender, [banner.watch()], |out| {
        let _ = out.send(VpnConnectionsCmd::BannerChanged(banner.get()));
    });
}

pub(super) fn build_rows(
    connections: &[wayle_network::core::settings_connection::ConnectionSettings],
    active_connections: &[Arc<wayle_network::core::connection::ActiveConnection>],
) -> Vec<VpnRowState> {
    connections
        .iter()
        .map(|settings| {
            let active = active_connections
                .iter()
                .find(|a| a.connection_path.get() == settings.object_path)
                .map(|a| VpnActiveInfo {
                    object_path: a.object_path.clone(),
                    state: a.state.get(),
                });
            VpnRowState {
                connection_path: settings.object_path.clone(),
                id: settings.id.get(),
                active,
            }
        })
        .collect()
}
