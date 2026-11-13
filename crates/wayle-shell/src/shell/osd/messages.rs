use std::sync::Arc;

use wayle_audio::{
    AudioService,
    core::device::{input::InputDevice, output::OutputDevice},
};
use wayle_brightness::{BacklightDevice, BrightnessService};
use wayle_config::ConfigService;

use crate::services::shell_ipc::{OsdRequest, ShellIpcState};

pub(crate) struct OsdInit {
    pub(crate) config: Arc<ConfigService>,
    pub(crate) audio: Option<Arc<AudioService>>,
    pub(crate) brightness: Option<Arc<BrightnessService>>,
    pub(crate) shell_ipc: ShellIpcState,

    /// Requests at or below this sequence are ignored. Non-zero only when the
    /// component is being recreated, to skip the request `Property::watch()`
    /// replays to every new subscriber.
    pub(crate) seen_seq: u64,
}

#[derive(Debug, Clone)]
pub(crate) enum OsdEvent {
    Slider {
        label: String,
        icon: String,
        percentage: f64,
        muted: bool,
    },

    Toggle {
        label: String,
        icon: String,
        active: bool,
    },
}

#[derive(Debug)]
pub(crate) enum OsdCmd {
    Ready,
    Dismiss(u32),
    ConfigChanged,
    DeviceChanged(Option<Arc<OutputDevice>>),
    VolumeChanged,
    InputDeviceChanged(Option<Arc<InputDevice>>),
    InputVolumeChanged,
    BrightnessDeviceChanged(Option<Arc<BacklightDevice>>),
    BrightnessChanged,
    ToggleChanged(ToggleEvent),
    ShowRequested(Option<OsdRequest>),
    DisplayedValueChanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum ToggleKey {
    CapsLock,
    NumLock,
    ScrollLock,
}

#[derive(Debug, Clone)]
pub(crate) struct ToggleEvent {
    pub(crate) key: ToggleKey,
    pub(crate) active: bool,
}
