use std::sync::Arc;

use relm4::ComponentSender;
use wayle_common::watch;
use wayle_config::ConfigService;
use wayle_network::NetworkService;

use super::{VpnTunnels, messages::VpnTunnelsCmd, providers};

pub(super) fn spawn(
    sender: &ComponentSender<VpnTunnels>,
    network: &Arc<NetworkService>,
    config: &Arc<ConfigService>,
) {
    // Watch for WireGuard service availability changes (hot-plug).
    // When adding new VPN providers, add their service watchers here.
    let wg_service = network.wireguard.clone();
    watch!(sender, [wg_service.watch()], |out| {
        let _ = out.send(VpnTunnelsCmd::ServiceChanged);
    });

    // Watch for alias changes (e.g. from another panel instance).
    let aliases_prop = config.config().modules.network.vpn_aliases.clone();
    let network_alias = network.clone();
    watch!(sender, [aliases_prop.watch()], |out| {
        let _ = out.send(VpnTunnelsCmd::TunnelsChanged(
            providers::wireguard_tunnels(&network_alias),
        ));
    });

    spawn_tunnel_watchers(sender, network);
}

pub(super) fn spawn_tunnel_watchers(
    sender: &ComponentSender<VpnTunnels>,
    network: &Arc<NetworkService>,
) {
    // Watch WireGuard tunnel changes.
    // When adding new VPN providers, add their tunnel watchers here.
    let Some(wg) = network.wireguard.get() else {
        return;
    };

    let network = network.clone();
    let tunnels_prop = wg.tunnels.clone();
    watch!(sender, [tunnels_prop.watch()], |out| {
        let _ = out.send(VpnTunnelsCmd::TunnelsChanged(
            providers::wireguard_tunnels(&network),
        ));
    });
}
