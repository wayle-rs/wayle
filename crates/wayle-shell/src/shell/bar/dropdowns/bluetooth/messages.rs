use std::sync::Arc;

use wayle_bluetooth::{BluetoothService, types::agent::PairingRequest};
use wayle_config::ConfigService;
use wayle_core::DeferredService;

pub(crate) struct BluetoothDropdownInit {
    pub bluetooth: DeferredService<BluetoothService>,
    pub config: Arc<ConfigService>,
}

#[derive(Debug)]
pub(crate) enum BluetoothDropdownMsg {
    VisibilityChanged(bool),
    ScanRequested,
}

#[derive(Debug)]
pub(crate) enum BluetoothDropdownCmd {
    ServiceReady(Arc<BluetoothService>),
    ScaleChanged(f32),
    /// The service's `available`, `enabled`, `powered` or `discovering`
    /// changed, or an adapter's `last_error` did (a power request that failed
    /// may have changed nothing else).
    StateChanged,
    /// The service's `radio_block` changed. Unlike `StateChanged`, this
    /// leaves the switch's knob alone: rfkill reports a block lifted before
    /// BlueZ has powered the adapters back on.
    RadioBlockChanged,
    DevicesChanged,
    DevicePropertyChanged,
    PairingRequested {
        request: Option<PairingRequest>,
        /// Whether it's a different prompt (or none) rather than an update of
        /// the one shown, such as a passkey's digits being typed.
        new_prompt: bool,
    },
}
