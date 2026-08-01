//! D-Bus interface adapter for shell IPC.

use std::sync::Arc;

use wayle_audio::AudioService;
use wayle_brightness::BrightnessService;
use wayle_config::ConfigService;
use wayle_ipc::shell_ipc::OsdDeviceInfo;
use zbus::{fdo, interface};

use super::{bar::BarVisibility, osd::OsdControl, state::ShellIpcState};

/// D-Bus daemon that dispatches shell commands to domain handlers.
pub(crate) struct ShellIpcDaemon {
    bar: BarVisibility,
    osd: OsdControl,
    state: ShellIpcState,
}

impl ShellIpcDaemon {
    pub(crate) fn new(
        state: ShellIpcState,
        config: Arc<ConfigService>,
        audio: Option<Arc<AudioService>>,
        brightness: Option<Arc<BrightnessService>>,
    ) -> Self {
        Self {
            bar: BarVisibility::new(state.clone()),
            osd: OsdControl::new(state.clone(), config, audio, brightness),
            state,
        }
    }
}

#[interface(name = "com.wayle.Shell1")]
impl ShellIpcDaemon {
    /// Hides the bar on a monitor. Empty string hides all bars.
    pub async fn bar_hide(&self, monitor: &str) {
        self.bar.hide(monitor);
    }

    /// Shows the bar on a monitor. Empty string shows all bars.
    pub async fn bar_show(&self, monitor: &str) {
        self.bar.show(monitor);
    }

    /// Toggles bar visibility on a monitor. Empty string toggles all.
    pub async fn bar_toggle(&self, monitor: &str) -> fdo::Result<()> {
        self.bar.toggle(monitor)
    }

    /// Shows the speaker OSD regardless of whether the volume changed.
    ///
    /// Empty device targets the default output. Returns the resolved device
    /// description.
    pub async fn osd_show_speaker(&self, device: &str) -> fdo::Result<String> {
        self.osd.show_speaker(device)
    }

    /// Shows the microphone OSD regardless of whether the volume changed.
    ///
    /// Empty device targets the default input. Returns the resolved device
    /// description.
    pub async fn osd_show_microphone(&self, device: &str) -> fdo::Result<String> {
        self.osd.show_microphone(device)
    }

    /// Shows the brightness OSD regardless of whether brightness changed.
    ///
    /// Empty device targets the primary backlight. Returns the resolved
    /// device name.
    pub async fn osd_show_brightness(&self, device: &str) -> fdo::Result<String> {
        self.osd.show_brightness(device)
    }

    /// Devices the OSD show methods can target.
    pub async fn osd_devices(&self) -> Vec<OsdDeviceInfo> {
        self.osd.devices()
    }

    /// Currently hidden monitor connectors.
    #[zbus(property)]
    pub async fn bar_hidden(&self) -> Vec<String> {
        let mut result: Vec<String> = self.state.hidden_bars.get().into_iter().collect();
        result.sort();
        result
    }

    /// All active monitor connectors.
    #[zbus(property)]
    pub async fn connectors(&self) -> Vec<String> {
        self.state.connectors.get()
    }
}
