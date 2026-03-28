/// WireGuard configuration file parser.
pub mod config_parser;
mod controls;
mod monitoring;
mod types;

use std::sync::Arc;

use controls::WireGuardControls;
use derive_more::Debug;
pub(crate) use types::{LiveWireGuardParams, WireGuardParams};
use wayle_common::Property;
use wayle_traits::{ModelMonitoring, Reactive};
use zbus::{Connection, zvariant::OwnedObjectPath};

use super::{
    core::{
        settings::Settings,
        settings_connection::ConnectionSettings,
    },
    error::Error,
    types::{connectivity::ConnectionType, states::NetworkStatus},
};

/// WireGuard VPN service.
///
/// Manages WireGuard tunnel connections through NetworkManager. Unlike WiFi or
/// wired devices, WireGuard interfaces are virtual and created on-demand when
/// a connection profile is activated.
#[derive(Clone, Debug)]
pub struct WireGuard {
    #[debug(skip)]
    pub(crate) zbus_connection: Connection,
    #[debug(skip)]
    pub(crate) cancellation_token: Option<tokio_util::sync::CancellationToken>,
    #[debug(skip)]
    settings: Arc<Settings>,
    /// Saved WireGuard tunnel configurations with their runtime state.
    pub tunnels: Property<Vec<Arc<WireGuardTunnel>>>,
}

/// A WireGuard tunnel combining a saved connection profile with runtime state.
#[derive(Clone, Debug)]
pub struct WireGuardTunnel {
    /// The saved connection profile.
    pub profile: ConnectionSettings,
    /// Whether this tunnel is currently active.
    pub active: Property<bool>,
    /// Current connectivity status when active.
    pub connectivity: Property<NetworkStatus>,
    /// IPv4 address when active.
    pub ip4_address: Property<Option<String>>,
    /// Interface name when active (e.g., "wg0").
    pub interface_name: Property<Option<String>>,
    /// Active connection D-Bus path (when active).
    pub(crate) active_connection_path: Property<Option<OwnedObjectPath>>,
}

impl WireGuardTunnel {
    pub(crate) fn new(profile: ConnectionSettings) -> Self {
        Self {
            profile,
            active: Property::new(false),
            connectivity: Property::new(NetworkStatus::Disconnected),
            ip4_address: Property::new(None),
            interface_name: Property::new(None),
            active_connection_path: Property::new(None),
        }
    }
}

impl PartialEq for WireGuardTunnel {
    fn eq(&self, other: &Self) -> bool {
        self.profile.object_path == other.profile.object_path
    }
}

impl PartialEq for WireGuard {
    fn eq(&self, other: &Self) -> bool {
        self.tunnels.get().len() == other.tunnels.get().len()
    }
}

impl Reactive for WireGuard {
    type Context<'a> = WireGuardParams<'a>;
    type LiveContext<'a> = LiveWireGuardParams<'a>;
    type Error = Error;

    async fn get(params: Self::Context<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            zbus_connection: params.connection.clone(),
            cancellation_token: None,
            settings: params.settings,
            tunnels: Property::new(vec![]),
        })
    }

    async fn get_live(params: Self::LiveContext<'_>) -> Result<Arc<Self>, Self::Error> {
        let wg = Self {
            zbus_connection: params.connection.clone(),
            cancellation_token: Some(params.cancellation_token.child_token()),
            settings: params.settings,
            tunnels: Property::new(vec![]),
        };

        let wg = Arc::new(wg);
        wg.clone().start_monitoring().await?;

        Ok(wg)
    }
}

impl WireGuard {
    /// Activate a WireGuard tunnel.
    ///
    /// # Errors
    ///
    /// Returns `Error::OperationFailed` if the activation fails.
    pub async fn activate(
        &self,
        connection_path: &OwnedObjectPath,
    ) -> Result<OwnedObjectPath, Error> {
        WireGuardControls::activate(&self.zbus_connection, connection_path).await
    }

    /// Deactivate an active WireGuard tunnel.
    ///
    /// # Errors
    ///
    /// Returns `Error::OperationFailed` if the deactivation fails.
    pub async fn deactivate(
        &self,
        tunnel: &WireGuardTunnel,
    ) -> Result<(), Error> {
        let Some(ref active_path) = tunnel.active_connection_path.get() else {
            return Ok(());
        };

        WireGuardControls::deactivate(&self.zbus_connection, active_path).await
    }

    /// Import a WireGuard `.conf` file as a new connection.
    ///
    /// # Arguments
    ///
    /// * `name` - Human-readable name for the connection.
    /// * `content` - The raw text content of the `.conf` file.
    ///
    /// # Errors
    ///
    /// Returns `Error::DataConversionFailed` if the config is malformed.
    /// Returns `Error::OperationFailed` if saving the connection fails.
    pub async fn import(
        &self,
        name: &str,
        content: &str,
    ) -> Result<OwnedObjectPath, Error> {
        WireGuardControls::import(
            &self.zbus_connection,
            name,
            content,
            &self.settings,
        )
        .await
    }

    /// Delete a WireGuard connection profile.
    ///
    /// If the connection is active, it will be deactivated first.
    ///
    /// # Errors
    ///
    /// Returns `Error::OperationFailed` if the deletion fails.
    pub async fn delete(&self, tunnel: &WireGuardTunnel) -> Result<(), Error> {
        if tunnel.active.get() {
            self.deactivate(tunnel).await?;
        }
        tunnel.profile.delete().await
    }

    /// Create a new WireGuard connection from NM settings.
    ///
    /// # Errors
    ///
    /// Returns `Error::OperationFailed` if creating the connection fails.
    pub async fn create(
        &self,
        nm_settings: std::collections::HashMap<
            String,
            std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
        >,
    ) -> Result<OwnedObjectPath, Error> {
        WireGuardControls::create(&self.settings, nm_settings).await
    }

    /// Get all saved WireGuard connection profiles from settings.
    pub fn saved_connections(&self) -> Vec<ConnectionSettings> {
        self.settings
            .connections
            .get()
            .into_iter()
            .filter(|c| c.connection_type.get() == ConnectionType::WireGuard)
            .collect()
    }
}
