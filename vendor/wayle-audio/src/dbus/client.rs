//! D-Bus client proxy for the audio service.
#![allow(missing_docs)]

use std::collections::HashMap;

use zbus::{Result, proxy};

/// D-Bus client proxy for controlling the audio service.
///
/// Connects to a running audio daemon and allows external control
/// of volume, mute state, and default device selection.
#[proxy(
    interface = "com.wayle.Audio1",
    default_service = "com.wayle.Audio1",
    default_path = "/com/wayle/Audio",
    gen_blocking = false
)]
pub trait Audio {
    /// Sets the volume for the default output device.
    ///
    /// Volume is specified as a percentage (0.0 to 100.0 for normal range).
    /// Returns the volume that was set (clamped to valid range).
    async fn set_output_volume(&self, volume: f64) -> Result<f64>;

    /// Adjusts the output volume by a relative delta.
    ///
    /// Delta is specified as percentage points (e.g., +5.0 or -10.0).
    /// Returns the new volume (clamped to 0-100%).
    async fn adjust_output_volume(&self, delta: f64) -> Result<f64>;

    /// Sets the mute state for the default output device.
    async fn set_output_mute(&self, muted: bool) -> Result<()>;

    /// Toggles mute for the default output device. Returns the new mute state.
    async fn toggle_output_mute(&self) -> Result<bool>;

    /// Sets the volume for the default input device.
    ///
    /// Volume is specified as a percentage (0.0 to 100.0).
    /// Returns the volume that was set (clamped to valid range).
    async fn set_input_volume(&self, volume: f64) -> Result<f64>;

    /// Adjusts the input volume by a relative delta.
    ///
    /// Delta is specified as percentage points (e.g., +5.0 or -10.0).
    /// Returns the new volume (clamped to 0-100%).
    async fn adjust_input_volume(&self, delta: f64) -> Result<f64>;

    /// Sets the mute state for the default input device.
    async fn set_input_mute(&self, muted: bool) -> Result<()>;

    /// Toggles mute for the default input device. Returns the new mute state.
    async fn toggle_input_mute(&self) -> Result<bool>;

    /// Sets the default output device (sink) by index.
    async fn set_default_sink(&self, device_index: u32) -> Result<()>;

    /// Sets the default input device (source) by index.
    async fn set_default_source(&self, device_index: u32) -> Result<()>;

    /// Lists all output devices (sinks).
    ///
    /// Returns a list of tuples: (device_index, name, description).
    async fn list_sinks(&self) -> Result<Vec<(u32, String, String)>>;

    /// Lists all input devices (sources).
    ///
    /// Returns a list of tuples: (device_index, name, description).
    async fn list_sources(&self) -> Result<Vec<(u32, String, String)>>;

    /// Gets detailed information about the default output device.
    async fn get_default_sink_info(&self) -> Result<HashMap<String, String>>;

    /// Gets detailed information about the default input device.
    async fn get_default_source_info(&self) -> Result<HashMap<String, String>>;

    /// Current volume of the default output as a percentage.
    #[zbus(property)]
    fn output_volume(&self) -> Result<f64>;

    /// Whether the default output is muted.
    #[zbus(property)]
    fn output_muted(&self) -> Result<bool>;

    /// Current volume of the default input as a percentage.
    #[zbus(property)]
    fn input_volume(&self) -> Result<f64>;

    /// Whether the default input is muted.
    #[zbus(property)]
    fn input_muted(&self) -> Result<bool>;

    /// Name of the default output device.
    #[zbus(property)]
    fn default_sink(&self) -> Result<String>;

    /// Name of the default input device.
    #[zbus(property)]
    fn default_source(&self) -> Result<String>;

    /// Number of output devices.
    #[zbus(property)]
    fn sink_count(&self) -> Result<u32>;

    /// Number of input devices.
    #[zbus(property)]
    fn source_count(&self) -> Result<u32>;
}
