//! Battery service initialization.
//!
//! UPower's `DisplayDevice` aggregates battery information but may not expose
//! the charge-threshold properties of the physical battery. Select a physical
//! battery with threshold support so charge-limit controls use the right device.

#![allow(missing_docs)]

use tracing::debug;
use wayle_battery::{BatteryService, types::DeviceType};
use zbus::{Connection, proxy, zvariant::OwnedObjectPath};

#[proxy(
    interface = "org.freedesktop.UPower",
    default_service = "org.freedesktop.UPower",
    default_path = "/org/freedesktop/UPower"
)]
trait UPower {
    fn enumerate_devices(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
}

#[proxy(
    interface = "org.freedesktop.UPower.Device",
    default_service = "org.freedesktop.UPower"
)]
trait UPowerDevice {
    #[zbus(property, name = "Type")]
    fn device_type(&self) -> zbus::Result<u32>;

    #[zbus(property)]
    fn is_present(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn is_rechargeable(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn charge_threshold_supported(&self) -> zbus::Result<bool>;
}

/// Build a battery service backed by a physical UPower battery when possible.
pub(super) async fn build() -> Result<BatteryService, String> {
    let display_device = BatteryService::new()
        .await
        .map_err(|error| error.to_string())?;

    if display_device.device.charge_threshold_supported.get() {
        return Ok(display_device);
    }

    let Some(device_path) = find_threshold_device().await else {
        return Ok(display_device);
    };

    debug!(
        ?device_path,
        "using physical UPower battery for charge thresholds"
    );

    match BatteryService::builder()
        .device_path(device_path.clone())
        .build()
        .await
    {
        Ok(service) => Ok(service),
        Err(error) => {
            debug!(?device_path, %error, "cannot initialize physical UPower battery; using DisplayDevice");
            Ok(display_device)
        }
    }
}

async fn find_threshold_device() -> Option<OwnedObjectPath> {
    let connection = connect_to_upower().await?;
    let paths = enumerate_devices(&connection).await?;
    let candidates = inspect_devices(&connection, paths).await;

    select_candidate(&candidates).cloned()
}

async fn connect_to_upower() -> Option<Connection> {
    match Connection::system().await {
        Ok(connection) => Some(connection),
        Err(error) => {
            debug!(%error, "cannot connect to UPower while looking for charge thresholds");
            None
        }
    }
}

async fn enumerate_devices(connection: &Connection) -> Option<Vec<OwnedObjectPath>> {
    let upower = match UPowerProxy::new(connection).await {
        Ok(upower) => upower,
        Err(error) => {
            debug!(%error, "cannot create UPower proxy while looking for charge thresholds");
            return None;
        }
    };

    match upower.enumerate_devices().await {
        Ok(paths) => Some(paths),
        Err(error) => {
            debug!(%error, "cannot enumerate UPower devices while looking for charge thresholds");
            None
        }
    }
}

async fn inspect_devices(
    connection: &Connection,
    paths: Vec<OwnedObjectPath>,
) -> Vec<(OwnedObjectPath, bool)> {
    let mut candidates = Vec::new();

    for path in paths {
        if let Some(candidate) = inspect_device(connection, path).await {
            candidates.push(candidate);
        }
    }

    candidates
}

async fn inspect_device(
    connection: &Connection,
    path: OwnedObjectPath,
) -> Option<(OwnedObjectPath, bool)> {
    let proxy = create_device_proxy(connection, &path).await?;
    if !is_battery(&proxy, &path).await {
        return None;
    }

    let (present, rechargeable) = battery_state(&proxy, &path).await?;
    if !present || !rechargeable {
        return None;
    }

    let threshold_supported = proxy.charge_threshold_supported().await.unwrap_or(false);
    Some((path, threshold_supported))
}

async fn create_device_proxy<'a>(
    connection: &'a Connection,
    path: &'a OwnedObjectPath,
) -> Option<UPowerDeviceProxy<'a>> {
    let builder = match UPowerDeviceProxy::builder(connection).path(path) {
        Ok(builder) => builder,
        Err(error) => {
            debug!(?path, %error, "cannot create UPower battery proxy");
            return None;
        }
    };

    match builder.build().await {
        Ok(proxy) => Some(proxy),
        Err(error) => {
            debug!(?path, %error, "cannot inspect UPower battery device");
            None
        }
    }
}

async fn is_battery(proxy: &UPowerDeviceProxy<'_>, path: &OwnedObjectPath) -> bool {
    match proxy.device_type().await {
        Ok(device_type) => matches!(DeviceType::from(device_type), DeviceType::Battery),
        Err(error) => {
            debug!(?path, %error, "cannot read UPower device type");
            false
        }
    }
}

async fn battery_state(
    proxy: &UPowerDeviceProxy<'_>,
    path: &OwnedObjectPath,
) -> Option<(bool, bool)> {
    let present = match proxy.is_present().await {
        Ok(present) => present,
        Err(error) => {
            debug!(?path, %error, "cannot read UPower battery presence");
            return None;
        }
    };
    let rechargeable = match proxy.is_rechargeable().await {
        Ok(rechargeable) => rechargeable,
        Err(error) => {
            debug!(?path, %error, "cannot read UPower battery rechargeability");
            return None;
        }
    };

    Some((present, rechargeable))
}

fn select_candidate<T>(candidates: &[(T, bool)]) -> Option<&T> {
    candidates
        .iter()
        .find(|(_, threshold_supported)| *threshold_supported)
        .map(|(path, _)| path)
}

#[cfg(test)]
mod tests {
    use super::select_candidate;

    #[test]
    fn prefers_battery_with_charge_threshold_support() {
        let candidates = vec![(String::from("BAT0"), false), (String::from("BAT1"), true)];

        assert_eq!(
            select_candidate(&candidates).map(String::as_str),
            Some("BAT1")
        );
    }

    #[test]
    fn returns_none_without_threshold_support() {
        let candidates = vec![(String::from("BAT0"), false), (String::from("BAT1"), false)];

        assert!(select_candidate(&candidates).is_none());
    }

    #[test]
    fn returns_none_without_battery_candidates() {
        let candidates: Vec<(String, bool)> = Vec::new();

        assert!(select_candidate(&candidates).is_none());
    }
}
