//! Aggregated battery service built from physical UPower power sources.
//!
//! UPower's `DisplayDevice` is useful for a desktop status icon, but it is not
//! a reliable source for per-battery charge-limit controls. This service
//! enumerates physical batteries and UPS devices and derives the overall
//! battery state from those devices instead.

use std::sync::Arc;

use futures::{StreamExt, stream};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};
use wayle_battery::{
    BatteryService as PhysicalBatteryService,
    core::device::Device as PhysicalDevice,
    types::{DeviceState, DeviceType, WarningLevel},
};
use wayle_core::Property;
use zbus::{Connection, proxy, zvariant::OwnedObjectPath};

const DISPLAY_DEVICE_PATH: &str = "/org/freedesktop/UPower/devices/DisplayDevice";

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
    fn is_rechargeable(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn power_supply(&self) -> zbus::Result<bool>;
}

/// Battery service used by the shell.
///
/// The service keeps one live `wayle-battery` device for every physical
/// battery and exposes a single aggregate device to the UI.
pub(crate) struct BatteryService {
    pub(crate) device: Arc<BatteryDevice>,
    _sources: Vec<Arc<PhysicalBatteryService>>,
    cancellation_token: CancellationToken,
}

/// Overall battery state derived from all physical batteries.
pub(crate) struct BatteryDevice {
    pub(crate) percentage: Property<f64>,
    pub(crate) state: Property<DeviceState>,
    pub(crate) time_to_empty: Property<i64>,
    pub(crate) time_to_full: Property<i64>,
    pub(crate) energy_rate: Property<f64>,
    pub(crate) energy: Property<f64>,
    pub(crate) energy_full: Property<f64>,
    pub(crate) capacity: Property<f64>,
    pub(crate) warning_level: Property<WarningLevel>,
    pub(crate) is_present: Property<bool>,
    pub(crate) charge_end_threshold: Property<u32>,
    pub(crate) charge_threshold_supported: Property<bool>,
    pub(crate) charge_threshold_enabled: Property<bool>,
    threshold_devices: Vec<Arc<PhysicalDevice>>,
}

#[derive(Clone, Debug)]
struct BatteryReading {
    percentage: f64,
    state: DeviceState,
    time_to_empty: i64,
    time_to_full: i64,
    energy_rate: f64,
    energy: f64,
    energy_full: f64,
    energy_full_design: f64,
    capacity: f64,
    warning_level: WarningLevel,
    is_present: bool,
    power_supply: bool,
    charge_start_threshold: u32,
    charge_end_threshold: u32,
    charge_threshold_supported: bool,
    charge_threshold_enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct AggregatedReading {
    percentage: f64,
    state: DeviceState,
    time_to_empty: i64,
    time_to_full: i64,
    energy_rate: f64,
    energy: f64,
    energy_full: f64,
    capacity: f64,
    warning_level: WarningLevel,
    is_present: bool,
    charge_end_threshold: u32,
    charge_threshold_supported: bool,
    charge_threshold_enabled: bool,
}

impl BatteryService {
    /// Builds a service from physical UPower battery devices.
    pub(crate) async fn new() -> Result<Self, String> {
        let sources = enumerate_physical_batteries().await?;
        if sources.is_empty() {
            return Err(String::from("no physical UPower batteries found"));
        }

        let readings = readings_from_sources(&sources);
        let initial = aggregate_readings(&readings);
        let threshold_devices = sources.iter().map(|source| source.device.clone()).collect();
        let device = Arc::new(BatteryDevice::new(initial, threshold_devices));
        let cancellation_token = CancellationToken::new();

        spawn_aggregator(
            device.clone(),
            sources.iter().map(|source| source.device.clone()).collect(),
            cancellation_token.clone(),
        );

        debug!(
            battery_count = sources.len(),
            "physical battery aggregate ready"
        );

        Ok(Self {
            device,
            _sources: sources,
            cancellation_token,
        })
    }
}

impl Drop for BatteryService {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}

impl BatteryDevice {
    fn new(reading: AggregatedReading, threshold_devices: Vec<Arc<PhysicalDevice>>) -> Self {
        Self {
            percentage: Property::new(reading.percentage),
            state: Property::new(reading.state),
            time_to_empty: Property::new(reading.time_to_empty),
            time_to_full: Property::new(reading.time_to_full),
            energy_rate: Property::new(reading.energy_rate),
            energy: Property::new(reading.energy),
            energy_full: Property::new(reading.energy_full),
            capacity: Property::new(reading.capacity),
            warning_level: Property::new(reading.warning_level),
            is_present: Property::new(reading.is_present),
            charge_end_threshold: Property::new(reading.charge_end_threshold),
            charge_threshold_supported: Property::new(reading.charge_threshold_supported),
            charge_threshold_enabled: Property::new(reading.charge_threshold_enabled),
            threshold_devices,
        }
    }

    pub(crate) async fn enable_charge_threshold(&self, enabled: bool) -> Result<(), String> {
        let mut controlled_devices = 0;

        for device in &self.threshold_devices {
            if !device.is_present.get() {
                continue;
            }

            if !device.charge_threshold_supported.get() {
                return Err(String::from(
                    "not all physical batteries support charge thresholds",
                ));
            }

            device
                .enable_charge_threshold(enabled)
                .await
                .map_err(|error| error.to_string())?;
            controlled_devices += 1;
        }

        if controlled_devices == 0 {
            return Err(String::from(
                "no present physical battery supports charge thresholds",
            ));
        }

        Ok(())
    }
}

async fn enumerate_physical_batteries() -> Result<Vec<Arc<PhysicalBatteryService>>, String> {
    let connection = Connection::system()
        .await
        .map_err(|error| format!("cannot connect to UPower: {error}"))?;
    let upower = UPowerProxy::new(&connection)
        .await
        .map_err(|error| format!("cannot create UPower proxy: {error}"))?;
    let paths = upower
        .enumerate_devices()
        .await
        .map_err(|error| format!("cannot enumerate UPower devices: {error}"))?;

    let mut sources = Vec::new();
    for path in paths {
        if !is_system_power_source(&connection, &path).await {
            continue;
        }

        match PhysicalBatteryService::builder()
            .device_path(path.clone())
            .build()
            .await
        {
            Ok(service) => sources.push(Arc::new(service)),
            Err(error) => warn!(?path, %error, "cannot initialize physical UPower battery"),
        }
    }

    Ok(sources)
}

async fn is_system_power_source(connection: &Connection, path: &OwnedObjectPath) -> bool {
    if path.as_str() == DISPLAY_DEVICE_PATH {
        return false;
    }

    let proxy = match UPowerDeviceProxy::builder(connection).path(path) {
        Ok(builder) => match builder.build().await {
            Ok(proxy) => proxy,
            Err(error) => {
                debug!(?path, %error, "cannot inspect UPower device");
                return false;
            }
        },
        Err(error) => {
            debug!(?path, %error, "cannot create UPower device proxy");
            return false;
        }
    };

    let (device_type, rechargeable, power_supply) = tokio::join!(
        proxy.device_type(),
        proxy.is_rechargeable(),
        proxy.power_supply()
    );
    let system_device = matches!(
        device_type.ok().map(DeviceType::from),
        Some(DeviceType::Battery | DeviceType::Ups)
    );
    matches!(
        (system_device, rechargeable.ok(), power_supply.ok()),
        (true, Some(true), Some(true))
    )
}

fn readings_from_sources(sources: &[Arc<PhysicalBatteryService>]) -> Vec<BatteryReading> {
    sources
        .iter()
        .map(|source| {
            let device = &source.device;
            BatteryReading {
                percentage: device.percentage.get(),
                state: device.state.get(),
                time_to_empty: device.time_to_empty.get(),
                time_to_full: device.time_to_full.get(),
                energy_rate: device.energy_rate.get(),
                energy: device.energy.get(),
                energy_full: device.energy_full.get(),
                energy_full_design: device.energy_full_design.get(),
                capacity: device.capacity.get(),
                warning_level: device.warning_level.get(),
                is_present: device.is_present.get(),
                power_supply: device.power_supply.get(),
                charge_start_threshold: device.charge_start_threshold.get(),
                charge_end_threshold: device.charge_end_threshold.get(),
                charge_threshold_supported: device.charge_threshold_supported.get(),
                charge_threshold_enabled: device.charge_threshold_enabled.get(),
            }
        })
        .collect()
}

fn readings_from_devices(sources: &[Arc<PhysicalDevice>]) -> Vec<BatteryReading> {
    sources
        .iter()
        .map(|device| BatteryReading {
            percentage: device.percentage.get(),
            state: device.state.get(),
            time_to_empty: device.time_to_empty.get(),
            time_to_full: device.time_to_full.get(),
            energy_rate: device.energy_rate.get(),
            energy: device.energy.get(),
            energy_full: device.energy_full.get(),
            energy_full_design: device.energy_full_design.get(),
            capacity: device.capacity.get(),
            warning_level: device.warning_level.get(),
            is_present: device.is_present.get(),
            power_supply: device.power_supply.get(),
            charge_start_threshold: device.charge_start_threshold.get(),
            charge_end_threshold: device.charge_end_threshold.get(),
            charge_threshold_supported: device.charge_threshold_supported.get(),
            charge_threshold_enabled: device.charge_threshold_enabled.get(),
        })
        .collect()
}

fn spawn_aggregator(
    aggregate: Arc<BatteryDevice>,
    sources: Vec<Arc<PhysicalDevice>>,
    cancellation_token: CancellationToken,
) {
    let mut streams = Vec::new();
    for device in &sources {
        streams.push(device.percentage.watch().map(|_| ()).boxed());
        streams.push(device.state.watch().map(|_| ()).boxed());
        streams.push(device.time_to_empty.watch().map(|_| ()).boxed());
        streams.push(device.time_to_full.watch().map(|_| ()).boxed());
        streams.push(device.energy_rate.watch().map(|_| ()).boxed());
        streams.push(device.energy.watch().map(|_| ()).boxed());
        streams.push(device.energy_full.watch().map(|_| ()).boxed());
        streams.push(device.energy_full_design.watch().map(|_| ()).boxed());
        streams.push(device.capacity.watch().map(|_| ()).boxed());
        streams.push(device.warning_level.watch().map(|_| ()).boxed());
        streams.push(device.is_present.watch().map(|_| ()).boxed());
        streams.push(device.power_supply.watch().map(|_| ()).boxed());
        streams.push(device.charge_start_threshold.watch().map(|_| ()).boxed());
        streams.push(device.charge_end_threshold.watch().map(|_| ()).boxed());
        streams.push(
            device
                .charge_threshold_supported
                .watch()
                .map(|_| ())
                .boxed(),
        );
        streams.push(device.charge_threshold_enabled.watch().map(|_| ()).boxed());
    }

    tokio::spawn(async move {
        let mut updates = stream::select_all(streams);
        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => return,
                update = updates.next() => {
                    if update.is_none() {
                        return;
                    }
                    update_aggregate(&aggregate, &sources);
                }
            }
        }
    });
}

fn update_aggregate(aggregate: &BatteryDevice, sources: &[Arc<PhysicalDevice>]) {
    let reading = aggregate_readings(&readings_from_devices(sources));
    aggregate.percentage.set(reading.percentage);
    aggregate.state.set(reading.state);
    aggregate.time_to_empty.set(reading.time_to_empty);
    aggregate.time_to_full.set(reading.time_to_full);
    aggregate.energy_rate.set(reading.energy_rate);
    aggregate.energy.set(reading.energy);
    aggregate.energy_full.set(reading.energy_full);
    aggregate.capacity.set(reading.capacity);
    aggregate.warning_level.set(reading.warning_level);
    aggregate.is_present.set(reading.is_present);
    aggregate
        .charge_end_threshold
        .set(reading.charge_end_threshold);
    aggregate
        .charge_threshold_supported
        .set(reading.charge_threshold_supported);
    aggregate
        .charge_threshold_enabled
        .set(reading.charge_threshold_enabled);
}

fn aggregate_readings(readings: &[BatteryReading]) -> AggregatedReading {
    let active: Vec<&BatteryReading> = readings
        .iter()
        .filter(|reading| reading.is_present && reading.power_supply)
        .collect();
    let is_present = !active.is_empty();

    let energy = active.iter().map(|reading| reading.energy.max(0.0)).sum();
    let energy_full = active
        .iter()
        .map(|reading| reading.energy_full.max(0.0))
        .sum();
    let energy_rate = active.iter().map(|reading| reading.energy_rate.abs()).sum();

    let percentage_weight = active
        .iter()
        .map(|reading| reading.energy_full.max(0.0))
        .sum::<f64>();
    let percentage = if percentage_weight > 0.0 {
        active
            .iter()
            .map(|reading| reading.percentage.clamp(0.0, 100.0) * reading.energy_full.max(0.0))
            .sum::<f64>()
            / percentage_weight
    } else if active.is_empty() {
        0.0
    } else {
        active
            .iter()
            .map(|reading| reading.percentage.clamp(0.0, 100.0))
            .sum::<f64>()
            / active.len() as f64
    };

    let capacity_weight = active
        .iter()
        .map(|reading| reading.energy_full_design.max(0.0))
        .sum::<f64>();
    let capacity = if capacity_weight > 0.0 {
        active
            .iter()
            .map(|reading| reading.capacity.clamp(0.0, 100.0) * reading.energy_full_design.max(0.0))
            .sum::<f64>()
            / capacity_weight
    } else if active.is_empty() {
        0.0
    } else {
        active
            .iter()
            .map(|reading| reading.capacity.clamp(0.0, 100.0))
            .sum::<f64>()
            / active.len() as f64
    };

    let state = aggregate_state(&active);
    let time_to_empty = aggregate_discharge_time(&active, energy, energy_rate, state);
    let time_to_full = aggregate_charge_time(&active, energy, energy_full, energy_rate, state);
    let warning_level = active
        .iter()
        .map(|reading| reading.warning_level)
        .max_by_key(|level| warning_rank(*level))
        .unwrap_or(WarningLevel::Unknown);

    let threshold_supported = !active.is_empty()
        && active
            .iter()
            .all(|reading| reading.charge_threshold_supported)
        && active.windows(2).all(|pair| {
            pair[0].charge_start_threshold == pair[1].charge_start_threshold
                && pair[0].charge_end_threshold == pair[1].charge_end_threshold
        });
    let charge_end_threshold = if threshold_supported {
        active[0].charge_end_threshold
    } else {
        0
    };
    let charge_threshold_enabled = threshold_supported
        && active
            .iter()
            .all(|reading| reading.charge_threshold_enabled);

    AggregatedReading {
        percentage,
        state,
        time_to_empty,
        time_to_full,
        energy_rate,
        energy,
        energy_full,
        capacity,
        warning_level,
        is_present,
        charge_end_threshold,
        charge_threshold_supported: threshold_supported,
        charge_threshold_enabled,
    }
}

fn aggregate_state(readings: &[&BatteryReading]) -> DeviceState {
    if readings.is_empty() {
        return DeviceState::Unknown;
    }
    if readings
        .iter()
        .any(|reading| reading.state == DeviceState::Charging)
    {
        return DeviceState::Charging;
    }
    if readings
        .iter()
        .any(|reading| reading.state == DeviceState::PendingCharge)
    {
        return DeviceState::PendingCharge;
    }
    if readings
        .iter()
        .any(|reading| reading.state == DeviceState::Discharging)
    {
        return DeviceState::Discharging;
    }
    if readings
        .iter()
        .any(|reading| reading.state == DeviceState::PendingDischarge)
    {
        return DeviceState::PendingDischarge;
    }
    if readings
        .iter()
        .all(|reading| reading.state == DeviceState::FullyCharged)
    {
        return DeviceState::FullyCharged;
    }
    if readings
        .iter()
        .any(|reading| reading.state == DeviceState::Empty)
    {
        return DeviceState::Empty;
    }

    DeviceState::Unknown
}

fn aggregate_discharge_time(
    readings: &[&BatteryReading],
    energy: f64,
    energy_rate: f64,
    state: DeviceState,
) -> i64 {
    if !matches!(
        state,
        DeviceState::Discharging | DeviceState::PendingDischarge
    ) {
        return 0;
    }
    if energy_rate > 0.0 {
        return ((energy / energy_rate) * 3600.0).round() as i64;
    }
    readings
        .iter()
        .map(|reading| reading.time_to_empty)
        .filter(|seconds| *seconds > 0)
        .max()
        .unwrap_or(-1)
}

fn aggregate_charge_time(
    readings: &[&BatteryReading],
    energy: f64,
    energy_full: f64,
    energy_rate: f64,
    state: DeviceState,
) -> i64 {
    if !matches!(state, DeviceState::Charging | DeviceState::PendingCharge) {
        return 0;
    }
    let device_time = readings
        .iter()
        .map(|reading| reading.time_to_full)
        .filter(|seconds| *seconds > 0)
        .max();
    if let Some(seconds) = device_time {
        return seconds;
    }
    if energy_rate > 0.0 {
        return (((energy_full - energy).max(0.0) / energy_rate) * 3600.0).round() as i64;
    }
    -1
}

fn warning_rank(level: WarningLevel) -> u8 {
    match level {
        WarningLevel::Unknown => 0,
        WarningLevel::None => 1,
        WarningLevel::Discharging => 2,
        WarningLevel::Low => 3,
        WarningLevel::Critical => 4,
        WarningLevel::Action => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::{BatteryReading, aggregate_readings};
    use wayle_battery::types::{DeviceState, WarningLevel};

    fn reading(percentage: f64, energy_full: f64) -> BatteryReading {
        BatteryReading {
            percentage,
            state: DeviceState::Discharging,
            time_to_empty: 0,
            time_to_full: 0,
            energy_rate: 10.0,
            energy: energy_full * percentage / 100.0,
            energy_full,
            energy_full_design: energy_full,
            capacity: 90.0,
            warning_level: WarningLevel::None,
            is_present: true,
            power_supply: true,
            charge_start_threshold: 75,
            charge_end_threshold: 80,
            charge_threshold_supported: true,
            charge_threshold_enabled: true,
        }
    }

    #[test]
    fn aggregates_percentage_by_battery_capacity() {
        let result = aggregate_readings(&[reading(50.0, 40.0), reading(100.0, 60.0)]);

        assert_eq!(result.percentage, 80.0);
        assert_eq!(result.energy, 80.0);
        assert_eq!(result.energy_full, 100.0);
    }

    #[test]
    fn charging_takes_precedence_over_discharging() {
        let mut discharging = reading(50.0, 100.0);
        let mut charging = reading(60.0, 100.0);
        discharging.state = DeviceState::Discharging;
        charging.state = DeviceState::Charging;

        assert_eq!(
            aggregate_readings(&[discharging, charging]).state,
            DeviceState::Charging
        );
    }

    #[test]
    fn global_threshold_requires_all_batteries_to_match() {
        let mut unsupported = reading(80.0, 100.0);
        unsupported.charge_threshold_supported = false;

        assert!(
            !aggregate_readings(&[reading(80.0, 100.0), unsupported]).charge_threshold_supported
        );
    }

    #[test]
    fn global_threshold_is_available_when_all_batteries_match() {
        let result = aggregate_readings(&[reading(80.0, 100.0), reading(80.0, 50.0)]);

        assert!(result.charge_threshold_supported);
        assert!(result.charge_threshold_enabled);
        assert_eq!(result.charge_end_threshold, 80);
    }

    #[test]
    fn ignores_non_system_batteries() {
        let mut accessory = reading(10.0, 10.0);
        accessory.power_supply = false;

        let result = aggregate_readings(&[reading(80.0, 100.0), accessory]);

        assert_eq!(result.percentage, 80.0);
        assert_eq!(result.energy_full, 100.0);
    }
}
