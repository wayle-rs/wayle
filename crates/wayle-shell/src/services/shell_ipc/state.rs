//! Reactive state for shell IPC.

use std::{collections::HashSet, sync::Arc};

use wayle_audio::core::device::{input::InputDevice, output::OutputDevice};
use wayle_brightness::BacklightDevice;
use wayle_core::Property;

/// Shared reactive state exposed to shell components via `ShellIpcService`.
///
/// Bar watchers subscribe to these properties to react to IPC commands.
#[derive(Clone)]
pub struct ShellIpcState {
    /// Connectors whose bars are currently hidden via CLI.
    pub hidden_bars: Property<HashSet<String>>,

    /// All active monitor connectors. Updated by the shell when bars are
    /// created or destroyed.
    pub connectors: Property<Vec<String>>,

    /// Most recent CLI request to display an OSD, or `None` before the first
    /// request. Written with [`Property::replace`] so repeat requests for an
    /// unchanged device still notify.
    pub osd_request: Property<Option<OsdRequest>>,
}

/// A CLI request to display an OSD for an already-resolved device.
#[derive(Debug, Clone)]
pub struct OsdRequest {
    /// Monotonic counter, starting at 1.
    ///
    /// [`Property::watch`] replays the current value to every new subscriber,
    /// so consumers track the sequence they have seen and ignore anything at
    /// or below it.
    pub seq: u64,

    /// Device to display.
    pub device: OsdDevice,
}

/// A device the OSD can report on.
#[derive(Debug, Clone, PartialEq)]
pub enum OsdDevice {
    /// Output device volume and mute state.
    Speaker(Arc<OutputDevice>),

    /// Input device volume and mute state.
    Microphone(Arc<InputDevice>),

    /// Backlight device brightness.
    Brightness(Arc<BacklightDevice>),
}

impl ShellIpcState {
    pub(crate) fn new() -> Self {
        Self {
            hidden_bars: Property::new(HashSet::new()),
            connectors: Property::new(Vec::new()),
            osd_request: Property::new(None),
        }
    }
}
