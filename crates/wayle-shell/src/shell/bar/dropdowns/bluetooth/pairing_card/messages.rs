use wayle_bluetooth::{BluetoothService, types::agent::PairingRequest};
use wayle_core::DeferredService;

pub(crate) struct PairingCardInit {
    pub bluetooth: DeferredService<BluetoothService>,
}

#[derive(Debug)]
pub(crate) enum PairingCardMsg {
    /// Shows `request` (or hides the card, for none).
    SetRequest {
        request: Option<PairingRequest>,
        /// Whether it's a different prompt rather than an update of the one
        /// shown; clears what was typed.
        new_prompt: bool,
        /// Whether to guard the card's buttons against a click aimed at
        /// whatever was under the pointer before the card appeared or changed.
        guard: bool,
    },
    /// The device may have changed its name or icon.
    Refresh,
    Confirm,
    /// Reject, deny or close: all turn the request down.
    Cancel,
}
