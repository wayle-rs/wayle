//! D-Bus server interface implementation.

use std::{collections::HashMap, sync::Arc};

use tracing::instrument;
use zbus::{fdo, interface};

use crate::{
    service::AudioService,
    types::device::{DeviceKey, DeviceType},
    volume::types::Volume,
};

#[derive(Debug)]
pub(crate) struct AudioDaemon {
    pub service: Arc<AudioService>,
}

#[interface(name = "com.wayle.Audio1")]
impl AudioDaemon {
    /// Sets the volume for the default output device.
    ///
    /// Volume is specified as a percentage (0.0 to 100.0 for normal range,
    /// up to 400.0 for amplification). Returns the volume that was set.
    #[instrument(skip(self), fields(volume = %volume))]
    pub async fn set_output_volume(&self, volume: f64) -> fdo::Result<f64> {
        let device = self
            .service
            .default_output
            .get()
            .ok_or_else(|| fdo::Error::Failed("No default output device".to_string()))?;

        let clamped = volume.clamp(0.0, 100.0);
        let channels = device.volume.get().channels();
        let vol = Volume::from_percentage(clamped, channels);

        device
            .set_volume(vol)
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;

        Ok(clamped)
    }

    /// Adjusts the output volume by a relative delta.
    ///
    /// Delta is specified as percentage points (e.g., +5.0 or -10.0).
    /// Result is clamped to 0-100% range. Returns the new volume.
    #[instrument(skip(self), fields(delta = %delta))]
    pub async fn adjust_output_volume(&self, delta: f64) -> fdo::Result<f64> {
        let device = self
            .service
            .default_output
            .get()
            .ok_or_else(|| fdo::Error::Failed("No default output device".to_string()))?;

        let current = device.volume.get();
        let current_pct = current.average() * 100.0;
        let new_pct = (current_pct + delta).clamp(0.0, 100.0);

        let vol = Volume::from_percentage(new_pct, current.channels());

        device
            .set_volume(vol)
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;

        Ok(new_pct)
    }

    /// Sets the mute state for the default output device.
    #[instrument(skip(self), fields(muted = muted))]
    pub async fn set_output_mute(&self, muted: bool) -> fdo::Result<()> {
        let device = self
            .service
            .default_output
            .get()
            .ok_or_else(|| fdo::Error::Failed("No default output device".to_string()))?;

        device
            .set_mute(muted)
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    /// Toggles mute for the default output device. Returns the new mute state.
    #[instrument(skip(self))]
    pub async fn toggle_output_mute(&self) -> fdo::Result<bool> {
        let device = self
            .service
            .default_output
            .get()
            .ok_or_else(|| fdo::Error::Failed("No default output device".to_string()))?;

        let new_state = !device.muted.get();
        device
            .set_mute(new_state)
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;

        Ok(new_state)
    }

    /// Sets the default output device (sink) by index.
    #[instrument(skip(self), fields(device_index = device_index))]
    pub async fn set_default_sink(&self, device_index: u32) -> fdo::Result<()> {
        let device = self
            .service
            .output_device(DeviceKey::new(device_index, DeviceType::Output))
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;

        device
            .set_as_default()
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    /// Sets the default input device (source) by index.
    #[instrument(skip(self), fields(device_index = device_index))]
    pub async fn set_default_source(&self, device_index: u32) -> fdo::Result<()> {
        let device = self
            .service
            .input_device(DeviceKey::new(device_index, DeviceType::Input))
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;

        device
            .set_as_default()
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    /// Sets the volume for the default input device.
    ///
    /// Volume is specified as a percentage (0.0 to 100.0). Returns the volume that was set.
    #[instrument(skip(self), fields(volume = %volume))]
    pub async fn set_input_volume(&self, volume: f64) -> fdo::Result<f64> {
        let device = self
            .service
            .default_input
            .get()
            .ok_or_else(|| fdo::Error::Failed("No default input device".to_string()))?;

        let clamped = volume.clamp(0.0, 100.0);
        let channels = device.volume.get().channels();
        let vol = Volume::from_percentage(clamped, channels);

        device
            .set_volume(vol)
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;

        Ok(clamped)
    }

    /// Adjusts the input volume by a relative delta.
    ///
    /// Delta is specified as percentage points (e.g., +5.0 or -10.0).
    /// Result is clamped to 0-100% range. Returns the new volume.
    #[instrument(skip(self), fields(delta = %delta))]
    pub async fn adjust_input_volume(&self, delta: f64) -> fdo::Result<f64> {
        let device = self
            .service
            .default_input
            .get()
            .ok_or_else(|| fdo::Error::Failed("No default input device".to_string()))?;

        let current = device.volume.get();
        let current_pct = current.average() * 100.0;
        let new_pct = (current_pct + delta).clamp(0.0, 100.0);

        let vol = Volume::from_percentage(new_pct, current.channels());

        device
            .set_volume(vol)
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;

        Ok(new_pct)
    }

    /// Sets the mute state for the default input device.
    #[instrument(skip(self), fields(muted = muted))]
    pub async fn set_input_mute(&self, muted: bool) -> fdo::Result<()> {
        let device = self
            .service
            .default_input
            .get()
            .ok_or_else(|| fdo::Error::Failed("No default input device".to_string()))?;

        device
            .set_mute(muted)
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    /// Toggles mute for the default input device. Returns the new mute state.
    #[instrument(skip(self))]
    pub async fn toggle_input_mute(&self) -> fdo::Result<bool> {
        let device = self
            .service
            .default_input
            .get()
            .ok_or_else(|| fdo::Error::Failed("No default input device".to_string()))?;

        let new_state = !device.muted.get();
        device
            .set_mute(new_state)
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;

        Ok(new_state)
    }

    /// Lists all output devices (sinks).
    ///
    /// Returns a list of tuples: (device_index, name, description).
    #[instrument(skip(self))]
    pub async fn list_sinks(&self) -> Vec<(u32, String, String)> {
        self.service
            .output_devices
            .get()
            .iter()
            .map(|device| {
                (
                    device.key.index,
                    device.name.get(),
                    device.description.get(),
                )
            })
            .collect()
    }

    /// Lists all input devices (sources).
    ///
    /// Returns a list of tuples: (device_index, name, description).
    #[instrument(skip(self))]
    pub async fn list_sources(&self) -> Vec<(u32, String, String)> {
        self.service
            .input_devices
            .get()
            .iter()
            .map(|device| {
                (
                    device.key.index,
                    device.name.get(),
                    device.description.get(),
                )
            })
            .collect()
    }

    /// Gets detailed information about the default output device.
    ///
    /// Returns a dictionary with device details.
    #[instrument(skip(self))]
    pub async fn get_default_sink_info(&self) -> fdo::Result<HashMap<String, String>> {
        let device = self
            .service
            .default_output
            .get()
            .ok_or_else(|| fdo::Error::Failed("No default output device".to_string()))?;

        let mut info = HashMap::new();
        info.insert("index".to_string(), device.key.index.to_string());
        info.insert("name".to_string(), device.name.get());
        info.insert("description".to_string(), device.description.get());
        info.insert(
            "volume".to_string(),
            format!("{:.0}", device.volume.get().average() * 100.0),
        );
        info.insert("muted".to_string(), device.muted.get().to_string());
        info.insert("state".to_string(), format!("{:?}", device.state.get()));

        if let Some(port) = device.active_port.get() {
            info.insert("active_port".to_string(), port);
        }

        Ok(info)
    }

    /// Gets detailed information about the default input device.
    ///
    /// Returns a dictionary with device details.
    #[instrument(skip(self))]
    pub async fn get_default_source_info(&self) -> fdo::Result<HashMap<String, String>> {
        let device = self
            .service
            .default_input
            .get()
            .ok_or_else(|| fdo::Error::Failed("No default input device".to_string()))?;

        let mut info = HashMap::new();
        info.insert("index".to_string(), device.key.index.to_string());
        info.insert("name".to_string(), device.name.get());
        info.insert("description".to_string(), device.description.get());
        info.insert(
            "volume".to_string(),
            format!("{:.0}", device.volume.get().average() * 100.0),
        );
        info.insert("muted".to_string(), device.muted.get().to_string());
        info.insert("state".to_string(), format!("{:?}", device.state.get()));

        if let Some(port) = device.active_port.get() {
            info.insert("active_port".to_string(), port);
        }

        Ok(info)
    }

    /// Current volume of the default output as a percentage.
    #[zbus(property)]
    pub async fn output_volume(&self) -> f64 {
        self.service
            .default_output
            .get()
            .map(|d| d.volume.get().average() * 100.0)
            .unwrap_or(0.0)
    }

    /// Whether the default output is muted.
    #[zbus(property)]
    pub async fn output_muted(&self) -> bool {
        self.service
            .default_output
            .get()
            .map(|d| d.muted.get())
            .unwrap_or(false)
    }

    /// Current volume of the default input as a percentage.
    #[zbus(property)]
    pub async fn input_volume(&self) -> f64 {
        self.service
            .default_input
            .get()
            .map(|d| d.volume.get().average() * 100.0)
            .unwrap_or(0.0)
    }

    /// Whether the default input is muted.
    #[zbus(property)]
    pub async fn input_muted(&self) -> bool {
        self.service
            .default_input
            .get()
            .map(|d| d.muted.get())
            .unwrap_or(false)
    }

    /// Name of the default output device.
    #[zbus(property)]
    pub async fn default_sink(&self) -> String {
        self.service
            .default_output
            .get()
            .map(|d| d.name.get())
            .unwrap_or_default()
    }

    /// Name of the default input device.
    #[zbus(property)]
    pub async fn default_source(&self) -> String {
        self.service
            .default_input
            .get()
            .map(|d| d.name.get())
            .unwrap_or_default()
    }

    /// Number of output devices.
    #[zbus(property)]
    pub async fn sink_count(&self) -> u32 {
        self.service.output_devices.get().len() as u32
    }

    /// Number of input devices.
    #[zbus(property)]
    pub async fn source_count(&self) -> u32 {
        self.service.input_devices.get().len() as u32
    }
}
