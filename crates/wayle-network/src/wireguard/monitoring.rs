use std::sync::{Arc, Weak};

use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};
use wayle_traits::ModelMonitoring;
use zbus::Connection;

use super::{WireGuard, WireGuardTunnel};
use crate::{
    core::{
        config::ip4_config::Ip4Config,
        settings_connection::ConnectionSettings,
    },
    error::Error,
    proxy::{
        active_connection::ConnectionActiveProxy,
        devices::DeviceProxy,
        manager::NetworkManagerProxy,
    },
    types::{
        connectivity::ConnectionType,
        states::{NMActiveConnectionState, NetworkStatus},
    },
};

impl ModelMonitoring for WireGuard {
    type Error = Error;

    async fn start_monitoring(self: Arc<Self>) -> Result<(), Self::Error> {
        let Some(ref cancellation_token) = self.cancellation_token else {
            return Err(Error::MissingCancellationToken);
        };

        let cancel_token = cancellation_token.clone();
        let weak_self = Arc::downgrade(&self);

        // Populate initial tunnel list and active states
        populate_tunnels(&self).await;

        let nm_proxy = NetworkManagerProxy::new(&self.zbus_connection)
            .await
            .map_err(Error::DbusError)?;

        tokio::spawn(async move {
            let _ = monitor_wireguard(weak_self, nm_proxy, cancel_token).await;
        });

        Ok(())
    }
}

async fn populate_tunnels(wg: &WireGuard) {
    let wg_connections: Vec<ConnectionSettings> = wg
        .settings
        .connections
        .get()
        .into_iter()
        .filter(|c| c.connection_type.get() == ConnectionType::WireGuard)
        .collect();

    let nm_proxy = NetworkManagerProxy::new(&wg.zbus_connection).await.ok();

    let active_connections = if let Some(ref proxy) = nm_proxy {
        proxy.active_connections().await.unwrap_or_default()
    } else {
        vec![]
    };

    let mut tunnels = Vec::with_capacity(wg_connections.len());

    for profile in wg_connections {
        let tunnel = WireGuardTunnel::new(profile);

        // Check if this connection is currently active
        resolve_active_state(
            &wg.zbus_connection,
            &tunnel,
            &active_connections,
        )
        .await;

        tunnels.push(Arc::new(tunnel));
    }

    wg.tunnels.set(tunnels);
}

async fn resolve_active_state(
    connection: &Connection,
    tunnel: &WireGuardTunnel,
    active_paths: &[zbus::zvariant::OwnedObjectPath],
) {
    let tunnel_uuid = tunnel.profile.uuid.get();

    for path in active_paths {
        let Ok(proxy) = ConnectionActiveProxy::new(connection, path.clone()).await else {
            continue;
        };

        let Ok(uuid) = proxy.uuid().await else {
            continue;
        };

        if uuid == tunnel_uuid {
            tunnel.active.set(true);

            if let Ok(state) = proxy.state().await {
                let nm_state = NMActiveConnectionState::from_u32(state);
                tunnel.connectivity.set(match nm_state {
                    NMActiveConnectionState::Activated => NetworkStatus::Connected,
                    NMActiveConnectionState::Activating => NetworkStatus::Connecting,
                    _ => NetworkStatus::Disconnected,
                });
            }

            tunnel
                .active_connection_path
                .set(Some(path.clone()));

            // Resolve IP from device
            if let Ok(devices) = proxy.devices().await
                && let Some(device_path) = devices.first()
                && let Ok(device_proxy) =
                    DeviceProxy::new(connection, device_path.clone()).await
            {
                if let Ok(ip4_path) = device_proxy.ip4_config().await {
                    let ip =
                        Ip4Config::resolve_address(connection, ip4_path)
                            .await;
                    tunnel.ip4_address.set(ip);
                }

                if let Ok(iface) = device_proxy.interface().await {
                    tunnel.interface_name.set(Some(iface));
                }
            }

            return;
        }
    }
}

async fn monitor_wireguard(
    weak_wg: Weak<WireGuard>,
    nm_proxy: NetworkManagerProxy<'static>,
    cancellation_token: CancellationToken,
) -> Result<(), Error> {
    let mut active_connections_changed =
        nm_proxy.receive_active_connections_changed().await;

    loop {
        let Some(wg) = weak_wg.upgrade() else {
            return Ok(());
        };

        tokio::select! {
            _ = cancellation_token.cancelled() => {
                debug!("WireGuard monitoring cancelled");
                return Ok(());
            }
            Some(_change) = active_connections_changed.next() => {
                handle_active_connections_changed(&wg).await;
            }
            else => {
                break;
            }
        }
    }

    Ok(())
}

async fn handle_active_connections_changed(wg: &WireGuard) {
    // Refresh the WireGuard connection list from settings
    let wg_connections: Vec<ConnectionSettings> = wg
        .settings
        .connections
        .get()
        .into_iter()
        .filter(|c| c.connection_type.get() == ConnectionType::WireGuard)
        .collect();

    let nm_proxy = match NetworkManagerProxy::new(&wg.zbus_connection).await {
        Ok(proxy) => proxy,
        Err(err) => {
            warn!(error = %err, "cannot create NM proxy for active connections refresh");
            return;
        }
    };

    let active_connections = nm_proxy.active_connections().await.unwrap_or_default();

    let mut tunnels = Vec::with_capacity(wg_connections.len());

    for profile in wg_connections {
        let tunnel = WireGuardTunnel::new(profile);

        resolve_active_state(
            &wg.zbus_connection,
            &tunnel,
            &active_connections,
        )
        .await;

        tunnels.push(Arc::new(tunnel));
    }

    wg.tunnels.set(tunnels);
}
