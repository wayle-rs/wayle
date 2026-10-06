use std::sync::Arc;

use wayle_config::ConfigService;
use wayle_iwd::{ConnectionState, IwdService, SignalStrength, Station, StationError};

pub(crate) struct ActiveConnectionsInit {
    pub iwd: Arc<IwdService>,
    pub config: Arc<ConfigService>,
}

pub(super) struct StationState {
    /// The station's connection, as IWD reports it: what the card shows as the
    /// active connection.
    pub connection: ConnectionState,
    pub strength: Option<SignalStrength>,
    pub frequency: Option<u32>,
    /// The station's most recent failed request.
    pub last_error: Option<StationError>,
    pub hovered: bool,
}

impl StationState {
    pub(super) fn from_station(station: &Station) -> Self {
        Self {
            connection: station.connection.get(),
            strength: station.strength.get(),
            frequency: station.frequency.get(),
            last_error: station.last_error.get(),
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
            last_error: None,
            hovered: false,
        }
    }
}

#[derive(Debug)]
pub(crate) enum ActiveConnectionsInput {
    Disconnect,
    Forget,
    DismissError,
    CardHovered(bool),
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum ActiveConnectionsCmd {
    StateChanged {
        connection: ConnectionState,
        strength: Option<SignalStrength>,
        frequency: Option<u32>,
        last_error: Option<StationError>,
    },
    StationDeviceChanged,
    /// A configured icon changed; re-render so `effective_wifi_icon` re-reads it.
    ConfigChanged,
}
