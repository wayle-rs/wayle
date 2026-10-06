use std::sync::Arc;

use wayle_config::ConfigService;
use wayle_iwd::{IwdService, Network};

use crate::shell::bar::dropdowns::iwd::password_form::PasswordFormOutput;

pub(crate) struct AvailableNetworksInit {
    pub iwd: Arc<IwdService>,
    pub config: Arc<ConfigService>,
}

#[derive(Debug)]
pub(crate) enum AvailableNetworksInput {
    /// The popover opened or closed; rows are only click-guarded while it is
    /// open.
    VisibilityChanged(bool),
    /// A row was clicked: connect to its network.
    NetworkSelected(Arc<Network>),
    ForgetNetwork(Arc<Network>),
    PasswordForm(PasswordFormOutput),
}

#[derive(Debug)]
pub(crate) enum AvailableNetworksCmd {
    /// IWD's device appeared, went away or was replaced.
    StationChanged,
    /// The station's power, connection or networks changed, or a request
    /// failed; rebuild the list.
    NetworksChanged,
    /// IWD started or stopped asking for a passphrase.
    PassphraseRequestChanged,
    /// A configured signal icon changed; rebuild the list (and refresh the
    /// password-form icon) so it re-reads the new config.
    ConfigChanged,
}
