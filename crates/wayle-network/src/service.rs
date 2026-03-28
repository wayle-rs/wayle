use std::sync::Arc;

use derive_more::Debug;
use futures::{Stream, StreamExt};
use tokio_util::sync::CancellationToken;
use tracing::{instrument, warn};
use wayle_core::Property;
use wayle_traits::{Reactive, ServiceMonitoring, Static};
use zbus::{Connection, zvariant::OwnedObjectPath};

use super::{
    core::access_point::types::{AccessPointParams, LiveAccessPointParams},
    error::Error,
    types::connectivity::ConnectionType,
    wifi::Wifi,
    wired::Wired,
    wireguard::WireGuard,
};
use crate::{
    core::{
        access_point::AccessPoint,
        config::{
            dhcp4_config::{Dhcp4Config, Dhcp4ConfigParams},
            dhcp6_config::{Dhcp6Config, Dhcp6ConfigParams},
            ip4_config::{Ip4Config, Ip4ConfigParams},
            ip6_config::{Ip6Config, Ip6ConfigParams},
        },
        connection::{ActiveConnection, ActiveConnectionParams, LiveActiveConnectionParams},
        device::{
            Device, DeviceParams, LiveDeviceParams,
            wifi::{DeviceWifi, DeviceWifiParams, LiveDeviceWifiParams},
            wired::{DeviceWired, DeviceWiredParams, LiveDeviceWiredParams},
        },
        settings::{LiveSettingsParams, Settings},
        settings_connection::{
            ConnectionSettings, ConnectionSettingsParams, LiveConnectionSettingsParams,
        },
    },
    discovery::NetworkServiceDiscovery,
    proxy::manager::NetworkManagerProxy,
    types::states::NMState,
    wifi::LiveWifiParams,
    wired::LiveWiredParams,
    wireguard::LiveWireGuardParams,
};

/// Entry point for network management. See [crate-level docs](crate) for usage.
#[derive(Debug)]
pub struct NetworkService {
    #[debug(skip)]
    pub(crate) zbus_connection: Connection,
    #[debug(skip)]
    pub(crate) cancellation_token: CancellationToken,
    /// Connection profile management.
    pub settings: Arc<Settings>,
    /// WiFi device, if present (live-updated on hot-plug).
    pub wifi: Property<Option<Arc<Wifi>>>,
    /// Wired device, if present (live-updated on hot-plug).
    pub wired: Property<Option<Arc<Wired>>>,
    /// WireGuard VPN service for managing WireGuard tunnels.
    pub wireguard: Property<Option<Arc<WireGuard>>>,
    /// Primary connection type as reported by NetworkManager.
    pub primary: Property<ConnectionType>,
}

impl NetworkService {
    /// Starts the network service and initializes all components.
    ///
    /// Performs device discovery, creates WiFi and wired service instances
    /// for available devices, and sets up property monitoring. Handles
    /// the actual initialization logic for the service.
    ///
    /// # Errors
    /// Returns `NetworkError::ServiceInitializationFailed` if:
    /// - D-Bus connection fails
    /// - Device discovery encounters errors
    /// - Device proxy creation fails
    #[instrument]
    pub async fn new() -> Result<Self, Error> {
        let connection = Connection::system().await.map_err(|err| {
            Error::ServiceInitializationFailed(format!("D-Bus connection failed: {err}"))
        })?;

        let cancellation_token = CancellationToken::new();

        let settings = Settings::get_live(LiveSettingsParams {
            zbus_connection: &connection,
            cancellation_token: &cancellation_token,
        })
        .await
        .map_err(|err| {
            Error::ServiceInitializationFailed(format!("cannot initialize Settings: {err}"))
        })?;

        let wifi_device_path = NetworkServiceDiscovery::wifi_device_path(&connection).await?;
        let wired_device_path = NetworkServiceDiscovery::wired_device_path(&connection).await?;

        let wifi = if let Some(path) = wifi_device_path {
            match Wifi::get_live(LiveWifiParams {
                connection: &connection,
                device_path: path.clone(),
                cancellation_token: &cancellation_token,
                settings: settings.clone(),
            })
            .await
            {
                Ok(wifi) => Some(wifi),
                Err(e) => {
                    warn!(error = %e, path = %path, "cannot create WiFi service");
                    None
                }
            }
        } else {
            None
        };

        let wired = if let Some(path) = wired_device_path {
            match Wired::get_live(LiveWiredParams {
                connection: &connection,
                device_path: path.clone(),
                cancellation_token: &cancellation_token,
            })
            .await
            {
                Ok(wired) => Some(wired),
                Err(e) => {
                    warn!(error = %e, path = %path, "cannot create wired service");
                    None
                }
            }
        } else {
            None
        };

        let wireguard = match WireGuard::get_live(LiveWireGuardParams {
            connection: &connection,
            cancellation_token: &cancellation_token,
            settings: settings.clone(),
        })
        .await
        {
            Ok(wg) => Some(wg),
            Err(e) => {
                warn!(error = %e, "cannot create WireGuard service");
                None
            }
        };

        let primary = Property::new(ConnectionType::None);

        let service = Self {
            zbus_connection: connection.clone(),
            cancellation_token,
            settings,
            wifi: Property::new(wifi),
            wired: Property::new(wired),
            wireguard: Property::new(wireguard),
            primary,
        };

        service.start_monitoring().await?;

        Ok(service)
    }

    /// Objects that implement the Connection.Active interface represent an attempt to
    /// connect to a network using the details provided by a Connection object.
    ///
    /// # Errors
    /// Returns `NetworkError::ObjectNotFound` if the connection doesn't exist.
    /// Returns `NetworkError::DbusError` if DBus operations fail.
    #[instrument(skip(self, path), fields(path = ?path), err)]
    pub async fn connection(&self, path: OwnedObjectPath) -> Result<ActiveConnection, Error> {
        ActiveConnection::get(ActiveConnectionParams {
            connection: &self.zbus_connection,
            path,
        })
        .await
    }

    /// Objects that implement the Connection.Active interface represent an attempt to
    /// connect to a network using the details provided by a Connection object.
    /// This variant monitors the connection for changes.
    ///
    /// # Errors
    /// Returns `NetworkError::ObjectNotFound` if the connection doesn't exist.
    /// Returns `NetworkError::DbusError` if DBus operations fail.
    #[instrument(skip(self, path), fields(path = ?path), err)]
    pub async fn connection_monitored(
        &self,
        path: OwnedObjectPath,
    ) -> Result<Arc<ActiveConnection>, Error> {
        ActiveConnection::get_live(LiveActiveConnectionParams {
            connection: &self.zbus_connection,
            path,
            cancellation_token: &self.cancellation_token,
        })
        .await
    }

    /// Wi-Fi Access Point.
    ///
    /// # Errors
    /// Returns `NetworkError::ObjectNotFound` if the access point doesn't exist.
    /// Returns `NetworkError::ObjectCreationFailed` if access point creation fails.
    #[instrument(skip(self, path), fields(path = ?path), err)]
    pub async fn access_point(&self, path: OwnedObjectPath) -> Result<AccessPoint, Error> {
        AccessPoint::get(AccessPointParams {
            connection: &self.zbus_connection,
            path,
        })
        .await
    }

    /// Wi-Fi Access Point.
    /// This variant monitors the access point for changes.
    ///
    /// # Errors
    /// Returns `NetworkError::ObjectNotFound` if the access point doesn't exist.
    /// Returns `NetworkError::ObjectCreationFailed` if access point creation fails.
    #[instrument(skip(self, path), fields(path = ?path), err)]
    pub async fn access_point_monitored(
        &self,
        path: OwnedObjectPath,
    ) -> Result<Arc<AccessPoint>, Error> {
        AccessPoint::get_live(LiveAccessPointParams {
            connection: &self.zbus_connection,
            path,
            cancellation_token: &self.cancellation_token,
        })
        .await
    }

    /// Represents a single network connection configuration.
    ///
    /// # Errors
    /// Returns `NetworkError::ObjectNotFound` if the connection profile doesn't exist.
    /// Returns `NetworkError::DbusError` if DBus operations fail.
    #[instrument(skip(self, path), fields(path = ?path), err)]
    pub async fn connection_settings(
        &self,
        path: OwnedObjectPath,
    ) -> Result<ConnectionSettings, Error> {
        ConnectionSettings::get(ConnectionSettingsParams {
            connection: &self.zbus_connection,
            path,
        })
        .await
    }

    /// Represents a single network connection configuration.
    /// This variant monitors the connection settings for changes.
    ///
    /// # Errors
    /// Returns `NetworkError::ObjectNotFound` if the connection profile doesn't exist.
    /// Returns `NetworkError::DbusError` if DBus operations fail.
    #[instrument(skip(self, path), fields(path = ?path), err)]
    pub async fn connection_settings_monitored(
        &self,
        path: OwnedObjectPath,
    ) -> Result<Arc<ConnectionSettings>, Error> {
        ConnectionSettings::get_live(LiveConnectionSettingsParams {
            connection: &self.zbus_connection,
            path,
            cancellation_token: &self.cancellation_token,
        })
        .await
    }

    /// Represents a network device.
    ///
    /// # Errors
    /// Returns `NetworkError::ObjectNotFound` if the device doesn't exist.
    /// Returns `NetworkError::DbusError` if DBus operations fail.
    #[instrument(skip(self, path), fields(path = ?path), err)]
    pub async fn device(&self, path: OwnedObjectPath) -> Result<Device, Error> {
        Device::get(DeviceParams {
            connection: &self.zbus_connection,
            object_path: path,
        })
        .await
    }

    /// Represents a network device.
    /// This variant monitors the device for changes.
    ///
    /// # Errors
    /// Returns `NetworkError::ObjectNotFound` if the device doesn't exist.
    /// Returns `NetworkError::DbusError` if DBus operations fail.
    #[instrument(skip(self, path), fields(path = ?path), err)]
    pub async fn device_monitored(&self, path: OwnedObjectPath) -> Result<Arc<Device>, Error> {
        Device::get_live(LiveDeviceParams {
            connection: &self.zbus_connection,
            object_path: path,
            cancellation_token: &self.cancellation_token,
        })
        .await
    }

    /// Represents a Wi-Fi device.
    ///
    /// # Errors
    /// Returns `NetworkError::ObjectNotFound` if the device doesn't exist.
    /// Returns `NetworkError::WrongObjectType` if the device is not a WiFi device.
    #[instrument(skip(self, path), fields(path = ?path), err)]
    pub async fn device_wifi(&self, path: OwnedObjectPath) -> Result<DeviceWifi, Error> {
        DeviceWifi::get(DeviceWifiParams {
            connection: &self.zbus_connection,
            device_path: path,
        })
        .await
    }

    /// Represents a Wi-Fi device.
    /// This variant monitors the device for changes.
    ///
    /// # Errors
    /// Returns `NetworkError::ObjectNotFound` if the device doesn't exist.
    /// Returns `NetworkError::WrongObjectType` if the device is not a WiFi device.
    #[instrument(skip(self, path), fields(path = ?path), err)]
    pub async fn device_wifi_monitored(
        &self,
        path: OwnedObjectPath,
    ) -> Result<Arc<DeviceWifi>, Error> {
        DeviceWifi::get_live(LiveDeviceWifiParams {
            connection: &self.zbus_connection,
            device_path: path,
            cancellation_token: &self.cancellation_token,
        })
        .await
    }

    /// Represents a wired Ethernet device.
    ///
    /// # Errors
    /// Returns `NetworkError::ObjectNotFound` if the device doesn't exist.
    /// Returns `NetworkError::WrongObjectType` if the device is not an ethernet device.
    #[instrument(skip(self, path), fields(path = ?path), err)]
    pub async fn device_wired(&self, path: OwnedObjectPath) -> Result<DeviceWired, Error> {
        DeviceWired::get(DeviceWiredParams {
            connection: &self.zbus_connection,
            device_path: path,
        })
        .await
    }

    /// Represents a wired Ethernet device.
    /// This variant monitors the device for changes.
    ///
    /// # Errors
    /// Returns `NetworkError::ObjectNotFound` if the device doesn't exist.
    /// Returns `NetworkError::WrongObjectType` if the device is not an ethernet device.
    #[instrument(skip(self, path), fields(path = ?path), err)]
    pub async fn device_wired_monitored(
        &self,
        path: OwnedObjectPath,
    ) -> Result<Arc<DeviceWired>, Error> {
        DeviceWired::get_live(LiveDeviceWiredParams {
            connection: &self.zbus_connection,
            device_path: path,
            cancellation_token: &self.cancellation_token,
        })
        .await
    }

    /// IPv4 Configuration Set.
    ///
    /// # Errors
    /// Returns `NetworkError::ObjectNotFound` if the configuration doesn't exist.
    /// Returns `NetworkError::DbusError` if DBus operations fail.
    #[instrument(skip(self, path), fields(path = ?path), err)]
    pub async fn ip4_config(&self, path: OwnedObjectPath) -> Result<Ip4Config, Error> {
        Ip4Config::get(Ip4ConfigParams {
            connection: &self.zbus_connection,
            path,
        })
        .await
    }

    /// IPv6 Configuration Set.
    ///
    /// # Errors
    /// Returns `NetworkError::ObjectNotFound` if the configuration doesn't exist.
    /// Returns `NetworkError::DbusError` if DBus operations fail.
    #[instrument(skip(self, path), fields(path = ?path), err)]
    pub async fn ip6_config(&self, path: OwnedObjectPath) -> Result<Ip6Config, Error> {
        Ip6Config::get(Ip6ConfigParams {
            connection: &self.zbus_connection,
            path,
        })
        .await
    }

    /// DHCP4 Configuration.
    ///
    /// # Errors
    /// Returns `NetworkError::ObjectNotFound` if the configuration doesn't exist.
    /// Returns `NetworkError::DbusError` if DBus operations fail.
    #[instrument(skip(self, path), fields(path = ?path), err)]
    pub async fn dhcp4_config(&self, path: OwnedObjectPath) -> Result<Dhcp4Config, Error> {
        Dhcp4Config::get(Dhcp4ConfigParams {
            connection: &self.zbus_connection,
            path,
        })
        .await
    }

    /// DHCP6 Configuration.
    ///
    /// # Errors
    /// Returns `NetworkError::ObjectNotFound` if the configuration doesn't exist.
    /// Returns `NetworkError::DbusError` if DBus operations fail.
    #[instrument(skip(self, path), fields(path = ?path), err)]
    pub async fn dhcp6_config(&self, path: OwnedObjectPath) -> Result<Dhcp6Config, Error> {
        Dhcp6Config::get(Dhcp6ConfigParams {
            connection: &self.zbus_connection,
            path,
        })
        .await
    }

    /// Emitted when system authorization details change.
    ///
    /// # Errors
    /// Returns error if D-Bus proxy creation fails.
    pub async fn check_permissions_signal(&self) -> Result<impl Stream<Item = ()>, Error> {
        let proxy = NetworkManagerProxy::new(&self.zbus_connection).await?;
        let stream = proxy.receive_check_permissions().await?;

        Ok(stream.filter_map(|_signal| async move { Some(()) }))
    }

    /// NetworkManager's state changed.
    ///
    /// # Errors
    /// Returns error if D-Bus proxy creation fails.
    pub async fn state_changed_signal(&self) -> Result<impl Stream<Item = NMState>, Error> {
        let proxy = NetworkManagerProxy::new(&self.zbus_connection).await?;
        let stream = proxy.receive_state_changed().await?;

        Ok(stream.filter_map(|signal| async move {
            signal.args().ok().map(|args| NMState::from_u32(args.state))
        }))
    }

    /// A device was added to the system.
    ///
    /// # Errors
    /// Returns error if D-Bus proxy creation fails.
    pub async fn device_added_signal(&self) -> Result<impl Stream<Item = OwnedObjectPath>, Error> {
        let proxy = NetworkManagerProxy::new(&self.zbus_connection).await?;
        let stream = proxy.receive_device_added().await?;

        Ok(stream
            .filter_map(|signal| async move { signal.args().ok().map(|args| args.device_path) }))
    }

    /// A device was removed from the system.
    ///
    /// # Errors
    /// Returns error if D-Bus proxy creation fails.
    pub async fn device_removed_signal(
        &self,
    ) -> Result<impl Stream<Item = OwnedObjectPath>, Error> {
        let proxy = NetworkManagerProxy::new(&self.zbus_connection).await?;
        let stream = proxy.receive_device_removed().await?;

        Ok(stream
            .filter_map(|signal| async move { signal.args().ok().map(|args| args.device_path) }))
    }
}

impl Drop for NetworkService {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}
