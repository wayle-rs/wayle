use wayle_config::schemas::modules::MullvadConfig;
use wayle_mullvad::ConnectionStatus;

use crate::i18n::t;

/// Picks the configured icon name for the current status. `LoggedOut`/`Revoked`
/// (the account can't tunnel) use the disabled icon; `Error` uses the blocked icon.
pub(crate) fn select_icon(config: &MullvadConfig, status: &ConnectionStatus) -> String {
    match status {
        ConnectionStatus::LoggedOut | ConnectionStatus::Revoked => {
            config.disabled_icon.get().clone()
        }
        ConnectionStatus::Connected(_) => config.connected_icon.get().clone(),
        ConnectionStatus::Connecting(_) | ConnectionStatus::Disconnecting => {
            config.connecting_icon.get().clone()
        }
        ConnectionStatus::Disconnected => config.disconnected_icon.get().clone(),
        ConnectionStatus::Error(_) => config.blocked_icon.get().clone(),
    }
}

/// The short bar label for the current status. When connected, prefers the relay
/// city; otherwise a translated status word.
pub(crate) fn format_label(status: &ConnectionStatus) -> String {
    match status {
        ConnectionStatus::LoggedOut => t!("bar-mullvad-logged-out"),
        ConnectionStatus::Revoked => t!("bar-mullvad-revoked"),
        ConnectionStatus::Connected(relay) => relay
            .city
            .clone()
            .filter(|city| !city.is_empty())
            .unwrap_or_else(|| t!("bar-mullvad-connected")),
        ConnectionStatus::Connecting(_) => t!("bar-mullvad-connecting"),
        ConnectionStatus::Disconnecting => t!("bar-mullvad-disconnecting"),
        ConnectionStatus::Disconnected => t!("bar-mullvad-disconnected"),
        ConnectionStatus::Error(_) => t!("bar-mullvad-blocked"),
    }
}
