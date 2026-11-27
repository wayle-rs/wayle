use std::sync::Arc;

use relm4::ComponentSender;
use tokio_util::sync::CancellationToken;
use wayle_network::{
    NetworkService,
    types::{connectivity::ConnectionType, states::NetworkStatus},
    wifi::Wifi,
    wired::Wired,
};
use wayle_sysinfo::SysinfoService;
use wayle_widgets::watch_cancellable;

use super::{NetworkSection, helpers, messages::NetworkSectionCmd};

pub(super) fn spawn(
    sender: &ComponentSender<NetworkSection>,
    network: &Option<Arc<NetworkService>>,
    sysinfo: &Arc<SysinfoService>,
    token: CancellationToken,
) {
    // When NetworkManager is unavailable (e.g. systemd-networkd systems),
    // connection state is inferred from sysinfo interface activity instead so
    // the speed readout is not permanently gated off.
    let has_network_manager = network.is_some();

    if let Some(network) = network {
        let primary = network.primary.clone();
        let wifi = network.wifi.clone();
        let wired = network.wired.clone();

        watch_cancellable!(
            sender,
            token.clone(),
            [primary.watch(), wifi.watch(), wired.watch()],
            |out| {
                let connection_type = primary.get();
                let connected = resolve_connected(connection_type, &wired.get(), &wifi.get());

                let _ = out.send(NetworkSectionCmd::ConnectionChanged { connected });
            }
        );
    }

    let net_data = sysinfo.network.clone();

    watch_cancellable!(sender, token, [net_data.watch()], |out| {
        let data = net_data.get();

        let (total_rx, total_tx) = data.iter().fold((0u64, 0u64), |(rx, tx), iface| {
            (rx + iface.rx_bytes_per_sec, tx + iface.tx_bytes_per_sec)
        });

        let upload = helpers::format_speed(total_tx);
        let download = helpers::format_speed(total_rx);

        let _ = out.send(NetworkSectionCmd::SpeedChanged {
            upload: upload.value,
            upload_is_megabytes: upload.is_megabytes,
            download: download.value,
            download_is_megabytes: download.is_megabytes,
        });

        if !has_network_manager {
            let _ = out.send(NetworkSectionCmd::ConnectionChanged {
                connected: helpers::infer_connected(&data),
            });
        }
    });
}

fn is_wired_connected(wired: &Option<Arc<Wired>>) -> bool {
    wired
        .as_ref()
        .is_some_and(|device| device.connectivity.get() == NetworkStatus::Connected)
}

fn is_wifi_connected(wifi: &Option<Arc<Wifi>>) -> bool {
    wifi.as_ref().is_some_and(|device| {
        device.enabled.get() && device.ssid.get().is_some_and(|ssid| !ssid.is_empty())
    })
}

fn resolve_connected(
    connection_type: ConnectionType,
    wired: &Option<Arc<Wired>>,
    wifi: &Option<Arc<Wifi>>,
) -> bool {
    match connection_type {
        ConnectionType::Wired => is_wired_connected(wired),
        ConnectionType::Wifi => is_wifi_connected(wifi),
        ConnectionType::None => false,
        _ => is_wired_connected(wired) || is_wifi_connected(wifi),
    }
}
