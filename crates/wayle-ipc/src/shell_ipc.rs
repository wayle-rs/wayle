//! D-Bus client proxy for shell IPC commands.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use zbus::{Result, proxy, zvariant::Type};

/// D-Bus service name for shell IPC.
pub const SERVICE_NAME: &str = "com.wayle.Shell1";

/// D-Bus object path for shell IPC.
pub const SERVICE_PATH: &str = "/com/wayle/Shell";

/// A device the OSD show methods can target.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OsdDeviceInfo {
    /// Device class: `speaker`, `microphone` or `brightness`.
    pub kind: String,

    /// Stable identifier accepted by the show methods: the PulseAudio node
    /// name for audio, the sysfs directory name for backlights.
    pub id: String,

    /// Human-readable name, as shown on the overlay.
    pub label: String,

    /// Whether this is the device used when no device is named.
    pub is_default: bool,
}

#[proxy(
    interface = "com.wayle.Shell1",
    default_service = "com.wayle.Shell1",
    default_path = "/com/wayle/Shell",
    gen_blocking = false
)]
pub trait ShellIpc {
    async fn bar_hide(&self, monitor: &str) -> Result<()>;

    async fn bar_show(&self, monitor: &str) -> Result<()>;

    async fn bar_toggle(&self, monitor: &str) -> Result<()>;

    async fn osd_show_speaker(&self, device: &str) -> Result<String>;

    async fn osd_show_microphone(&self, device: &str) -> Result<String>;

    async fn osd_show_brightness(&self, device: &str) -> Result<String>;

    async fn osd_devices(&self) -> Result<Vec<OsdDeviceInfo>>;

    #[zbus(property)]
    fn bar_hidden(&self) -> Result<Vec<String>>;

    #[zbus(property)]
    fn connectors(&self) -> Result<Vec<String>>;
}
