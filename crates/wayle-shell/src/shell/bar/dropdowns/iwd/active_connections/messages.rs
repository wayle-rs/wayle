use std::sync::Arc;

use wayle_config::ConfigService;
use wayle_iwd::{ConnectionState, IwdService, SignalStrength, Station};

pub(crate) struct ActiveConnectionsInit {
    pub iwd: Arc<IwdService>,
    pub config: Arc<ConfigService>,
}

pub(super) struct StationState {
    /// Service-owned, attempt-aware connection state — the single source of
    /// truth for what the card shows as the active connection.
    pub connection: ConnectionState,
    pub strength: Option<SignalStrength>,
    pub frequency: Option<u32>,
    pub hovered: bool,
}

impl StationState {
    pub(super) fn from_station(station: &Station) -> Self {
        Self {
            connection: station.connection.get(),
            strength: station.strength.get(),
            frequency: station.frequency.get(),
            hovered: false,
        }
    }
}

impl Default for StationState {
    fn default() -> Self {
        Self {
            connection: ConnectionState::Idle,
            strength: None,
            frequency: None,
            hovered: false,
        }
    }
}

/// A failed connection attempt to display on the card. Transient and
/// shell-owned (mirrors the NetworkManager dropdown's `ConnectionProgress`);
/// shown only while the station is otherwise idle.
pub(super) struct ConnectionError {
    pub ssid: String,
    pub message: String,
}

#[derive(Debug)]
pub(crate) enum ActiveConnectionsInput {
    Disconnect,
    Forget,
    DismissError,
    CardHovered(bool),
    /// Show a failed-connection error on the card, routed from the
    /// available-networks list (which owns the connect command and its result).
    ShowError { ssid: String, message: String },
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum ActiveConnectionsCmd {
    StateChanged {
        connection: ConnectionState,
        strength: Option<SignalStrength>,
        frequency: Option<u32>,
    },
    StationDeviceChanged,
    /// A configured icon changed; re-render so `effective_wifi_icon` re-reads it.
    ConfigChanged,
}
