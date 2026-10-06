use std::sync::Arc;

use wayle_config::ConfigService;
use wayle_iwd::IwdService;

pub(crate) struct IwdDropdownInit {
    pub iwd: Arc<IwdService>,
    pub config: Arc<ConfigService>,
}

#[derive(Debug)]
pub(crate) enum IwdDropdownMsg {
    /// Trigger a scan (from the Scan button).
    ScanRequested,
    /// The dropdown opened.
    Shown,
}

#[derive(Debug)]
pub(crate) enum IwdDropdownCmd {
    ScaleChanged(f32),
    StationDeviceChanged,
    /// IWD answered a power request, or WiFi turned on or off for another
    /// reason: the station's `powered` (adapter unblocked and device up)
    /// changed, or a power request failed (which may change nothing else).
    PowerAnswered(bool),
    /// The station started or stopped scanning.
    ScanningChanged(bool),
}
