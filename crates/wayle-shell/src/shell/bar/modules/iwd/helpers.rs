use wayle_config::schemas::modules::IwdConfig;
use wayle_iwd::{ConnectionState, SignalStrength};

use crate::{i18n::t, shell::bar::dropdowns::connected_signal_icon};

pub(crate) struct StationContext {
    pub(crate) powered: bool,
    /// Attempt-aware connection state — the same source the dropdown card uses,
    /// so the bar shows "connecting" throughout a switch instead of flickering to
    /// "disconnected" while IWD's raw `State` passes through `disconnected`.
    pub(crate) connection: ConnectionState,
    pub(crate) strength: Option<SignalStrength>,
}

pub(crate) fn wifi_icon(config: &IwdConfig, ctx: &StationContext) -> String {
    if !ctx.powered {
        return config.wifi_disabled_icon.get().clone();
    }

    match ctx.connection {
        ConnectionState::Connecting { .. } => config.wifi_acquiring_icon.get().clone(),
        ConnectionState::Idle => config.wifi_offline_icon.get().clone(),
        // Roaming is still connected — show the signal-strength icon, treating
        // unknown strength as the weakest ("none") bucket.
        ConnectionState::Connected { .. } | ConnectionState::Roaming { .. } => connected_signal_icon(
            ctx.strength,
            &config.wifi_signal_icons.get(),
            &config.wifi_connected_icon.get(),
        ),
    }
}

pub(crate) fn wifi_label(ctx: &StationContext) -> String {
    match &ctx.connection {
        ConnectionState::Connected { ssid } | ConnectionState::Roaming { ssid }
            if !ssid.is_empty() =>
        {
            ssid.clone()
        }
        ConnectionState::Connected { .. } | ConnectionState::Roaming { .. } => {
            t!("bar-iwd-wifi-fallback")
        }
        ConnectionState::Connecting { .. } => t!("bar-iwd-connecting"),
        ConnectionState::Idle => t!("bar-iwd-disconnected"),
    }
}

