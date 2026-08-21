use std::sync::Arc;

use wayle_config::ConfigService;
use wayle_iwd::{IwdService, SignalStrength};
use zbus::zvariant::OwnedObjectPath;

use crate::shell::bar::dropdowns::iwd::password_form::PasswordFormOutput;

pub(super) struct SelectedNetwork {
    pub network_path: OwnedObjectPath,
    pub ssid: String,
    pub security_label: String,
    /// Signal bucket — the source of truth for the password-form icon, which is
    /// recomputed from it (rather than cached) whenever it is needed, so an icon
    /// config change has a single place to take effect.
    pub strength: SignalStrength,
    pub secured: bool,
}

pub(crate) struct AvailableNetworksInit {
    pub iwd: Arc<IwdService>,
    pub config: Arc<ConfigService>,
}

#[derive(Debug)]
pub(crate) enum AvailableNetworksInput {
    StationAvailabilityChanged(bool),
    PoweredChanged(bool),
    NetworkSelected(usize),
    ForgetNetwork(OwnedObjectPath),
    PasswordForm(PasswordFormOutput),
}

#[derive(Debug)]
pub(crate) enum AvailableNetworksCmd {
    /// The service connection state or the scan list changed; re-dismiss a stale
    /// password prompt and rebuild the list.
    NetworksChanged,
    /// The attempt reached a stable outcome with no error to show — connected
    /// successfully, or aborted (cancelled via Disconnect / superseded). Reset
    /// the list to normal browsing.
    ConnectionSettled,
    ConnectionAuthFailed,
    ConnectionFailed(String),
    /// A configured signal icon changed; rebuild the list (and refresh the
    /// password-form icon) so it re-reads the new config.
    ConfigChanged,
}

#[derive(Debug)]
pub(crate) enum AvailableNetworksOutput {
    /// A genuine (non-auth) failure that left the station disconnected; the card
    /// displays it.
    ConnectionFailed { ssid: String, message: String },
}
