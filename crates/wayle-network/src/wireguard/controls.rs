use std::collections::HashMap;

use tracing::{debug, instrument};
use zbus::{
    Connection,
    zvariant::{OwnedObjectPath, OwnedValue, Value},
};

use super::config_parser::{self, WireGuardConfig};
use crate::{
    core::settings::Settings,
    error::Error,
    proxy::manager::NetworkManagerProxy,
};

type ConnectionSettings = HashMap<String, HashMap<String, OwnedValue>>;

pub(super) struct WireGuardControls;

impl WireGuardControls {
    /// Activate an existing WireGuard connection profile.
    #[instrument(skip(connection), fields(profile = %connection_path), err)]
    pub(super) async fn activate(
        connection: &Connection,
        connection_path: &OwnedObjectPath,
    ) -> Result<OwnedObjectPath, Error> {
        let proxy = NetworkManagerProxy::new(connection).await?;

        let empty_path = OwnedObjectPath::try_from("/")
            .map_err(|err| Error::DbusError(err.into()))?;

        // For WireGuard, device and specific_object are "/" (auto-detected)
        let active_connection_path = proxy
            .activate_connection(connection_path, &empty_path, &empty_path)
            .await
            .map_err(|err| Error::OperationFailed {
                operation: "activate wireguard connection",
                source: err.into(),
            })?;

        debug!(
            active = %active_connection_path,
            "WireGuard connection activated"
        );

        Ok(active_connection_path)
    }

    /// Deactivate an active WireGuard connection.
    #[instrument(skip(connection), fields(active = %active_connection_path), err)]
    pub(super) async fn deactivate(
        connection: &Connection,
        active_connection_path: &OwnedObjectPath,
    ) -> Result<(), Error> {
        let proxy = NetworkManagerProxy::new(connection).await?;

        proxy
            .deactivate_connection(active_connection_path)
            .await
            .map_err(|err| Error::OperationFailed {
                operation: "deactivate wireguard connection",
                source: err.into(),
            })?;

        debug!("WireGuard connection deactivated");

        Ok(())
    }

    /// Import a WireGuard `.conf` file as a new connection profile.
    #[instrument(skip(_connection, content, settings), fields(name = name), err)]
    pub(super) async fn import(
        _connection: &Connection,
        name: &str,
        content: &str,
        settings: &Settings,
    ) -> Result<OwnedObjectPath, Error> {
        let config = config_parser::parse_config(content)?;
        let nm_settings = build_nm_settings(name, &config)?;

        let path = settings.add_connection(nm_settings).await?;

        debug!(path = %path, "WireGuard connection imported");

        Ok(path)
    }

    /// Create a new WireGuard connection from individual parameters.
    #[instrument(skip(settings, nm_settings), err)]
    pub(super) async fn create(
        settings: &Settings,
        nm_settings: ConnectionSettings,
    ) -> Result<OwnedObjectPath, Error> {
        let path = settings.add_connection(nm_settings).await?;

        debug!(path = %path, "WireGuard connection created");

        Ok(path)
    }
}

/// Build complete NM connection settings for a WireGuard connection.
///
/// Converts parsed WireGuard configuration into the HashMap structure
/// expected by NetworkManager's D-Bus API.
///
/// # Errors
///
/// Returns `Error::OperationFailed` if value conversion fails.
pub fn build_nm_settings(
    name: &str,
    config: &WireGuardConfig,
) -> Result<ConnectionSettings, Error> {
    let to_owned = |value: Value| {
        value.try_to_owned().map_err(|e| Error::OperationFailed {
            operation: "convert to owned value",
            source: e.into(),
        })
    };

    let mut settings = HashMap::new();

    // connection section
    let mut conn = HashMap::new();
    conn.insert(String::from("type"), to_owned(Value::from("wireguard"))?);
    conn.insert(String::from("id"), to_owned(Value::from(name))?);
    conn.insert(
        String::from("interface-name"),
        to_owned(Value::from(name))?,
    );
    settings.insert(String::from("connection"), conn);

    // wireguard section
    let mut wg = HashMap::new();
    wg.insert(
        String::from("private-key"),
        to_owned(Value::from(config.interface.private_key.as_str()))?,
    );

    if let Some(port) = config.interface.listen_port {
        wg.insert(
            String::from("listen-port"),
            to_owned(Value::from(u32::from(port)))?,
        );
    }

    if let Some(mtu) = config.interface.mtu {
        wg.insert(String::from("mtu"), to_owned(Value::from(mtu))?);
    }

    settings.insert(String::from("wireguard"), wg);

    // ipv4 section
    build_ipv4_section(&config.interface, &mut settings, &to_owned)?;

    // ipv6 section
    build_ipv6_section(&config.interface, &mut settings, &to_owned)?;

    Ok(settings)
}

fn build_ipv4_section(
    iface: &config_parser::InterfaceConfig,
    settings: &mut ConnectionSettings,
    to_owned: &dyn Fn(Value) -> Result<OwnedValue, Error>,
) -> Result<(), Error> {
    let mut ipv4 = HashMap::new();

    let v4_addrs: Vec<&str> = iface
        .addresses
        .iter()
        .filter(|a| !a.contains(':'))
        .map(String::as_str)
        .collect();

    if v4_addrs.is_empty() {
        ipv4.insert(String::from("method"), to_owned(Value::from("disabled"))?);
    } else {
        ipv4.insert(String::from("method"), to_owned(Value::from("manual"))?);
    }

    settings.insert(String::from("ipv4"), ipv4);

    Ok(())
}

fn build_ipv6_section(
    iface: &config_parser::InterfaceConfig,
    settings: &mut ConnectionSettings,
    to_owned: &dyn Fn(Value) -> Result<OwnedValue, Error>,
) -> Result<(), Error> {
    let mut ipv6 = HashMap::new();

    let has_v6 = iface.addresses.iter().any(|a| a.contains(':'));

    if has_v6 {
        ipv6.insert(String::from("method"), to_owned(Value::from("manual"))?);
    } else {
        ipv6.insert(String::from("method"), to_owned(Value::from("disabled"))?);
    }

    settings.insert(String::from("ipv6"), ipv6);

    Ok(())
}
