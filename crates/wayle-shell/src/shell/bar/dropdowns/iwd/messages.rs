use std::sync::Arc;

use wayle_config::ConfigService;
use wayle_iwd::IwdService;

use super::available_networks::AvailableNetworksOutput;

pub(crate) struct IwdDropdownInit {
    pub iwd: Arc<IwdService>,
    pub config: Arc<ConfigService>,
}

#[derive(Debug)]
pub(crate) enum IwdDropdownMsg {
    PowerToggled(bool),
    /// Trigger a scan (from the Scan button).
    ScanRequested,
    AvailableNetworks(AvailableNetworksOutput),
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum IwdDropdownCmd {
    ScaleChanged(f32),
    StationDeviceChanged,
    /// Combined `Device.Powered` + `Station.Scanning` update.
    StationFlags { powered: bool, scanning: bool },
}
