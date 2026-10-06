use std::sync::Arc;

use relm4::ComponentSender;
use wayle_iwd::{ConnectionState, Network, StationAction, StationError};

use super::ActiveConnections;
use crate::{i18n::t, shell::bar::dropdowns::iwd::helpers};

impl ActiveConnections {
    /// The failed connect to display, and the network it was for. Shown only
    /// while the station is otherwise [`Idle`](ConnectionState::Idle): a
    /// failed connect that left IWD on its previous network isn't news.
    fn shown_error(&self) -> Option<(&StationError, &Arc<Network>)> {
        let error = self.station.last_error.as_ref()?;
        let StationAction::Connect { network } = &error.action else {
            return None;
        };
        matches!(self.station.connection, ConnectionState::Idle).then_some((error, network))
    }

    /// Whether a failed attempt should be shown on the card.
    pub(super) fn has_wifi_error(&self) -> bool {
        self.shown_error().is_some()
    }

    pub(super) fn is_connecting(&self) -> bool {
        matches!(self.station.connection, ConnectionState::Connecting { .. })
    }

    pub(super) fn is_connected(&self) -> bool {
        matches!(self.station.connection, ConnectionState::Connected { .. })
    }

    pub(super) fn is_roaming(&self) -> bool {
        matches!(self.station.connection, ConnectionState::Roaming { .. })
    }

    /// Whether there is an active connection (connected or roaming) — both show
    /// the signal-strength icon and the "actions" controls.
    pub(super) fn is_active(&self) -> bool {
        self.is_connected() || self.is_roaming()
    }

    /// Label shown for an active connection: "Roaming" while roaming, otherwise
    /// "Connected".
    pub(super) fn active_status_label(&self) -> String {
        if self.is_roaming() {
            t!("dropdown-iwd-roaming")
        } else {
            t!("dropdown-iwd-connected")
        }
    }

    /// Whether the active-connection card should be shown at all.
    pub(super) fn card_visible(&self) -> bool {
        self.is_connecting() || self.is_active() || self.has_wifi_error()
    }

    /// Whether the card offers Forget for the active network: its credentials
    /// are saved, and they are ones the shell could put back (see
    /// [`helpers::forgettable`]).
    pub(super) fn offers_forget(&self) -> bool {
        self.station.connection.network().is_some_and(|network| {
            network.known.get() && helpers::forgettable(network.security.get())
        })
    }

    pub(super) fn reset_station_watchers(&mut self, sender: &ComponentSender<Self>) {
        let token = self.station_watcher.reset();

        super::watchers::spawn_station_watchers(sender, &self.iwd, token);
    }

    pub(super) fn display_wifi_name(&self) -> String {
        if let Some((_, network)) = self.shown_error() {
            return network.ssid.get();
        }

        if let Some(network) = self.station.connection.network() {
            return network.ssid.get();
        }

        t!("dropdown-iwd-wifi")
    }

    pub(super) fn status_label(&self) -> String {
        if self.has_wifi_error() {
            return t!("dropdown-iwd-error");
        }

        if self.is_connecting() {
            return t!("dropdown-iwd-connecting");
        }

        String::new()
    }

    pub(super) fn wifi_detail_visible(&self) -> bool {
        // While connecting, `wifi.frequency` may still hold the *previous*
        // network's band until fresh diagnostics arrive, so only show the band
        // once on an active connection (connected or roaming).
        self.has_wifi_error() || (self.is_active() && self.station.frequency.is_some())
    }

    pub(super) fn wifi_detail(&self) -> String {
        if self.shown_error().is_some() {
            return t!("dropdown-iwd-error-generic");
        }

        self.station
            .frequency
            .and_then(helpers::frequency_to_band)
            .map(str::to_string)
            .unwrap_or_default()
    }

    /// IWD's own description of the failure (e.g. `net.connman.iwd.Timeout`).
    pub(super) fn error_tooltip(&self) -> Option<String> {
        let (error, _) = self.shown_error()?;
        Some(match error.error.iwd_error() {
            Some((name, Some(message))) if !message.is_empty() => format!("{name}: {message}"),
            Some((name, _)) => name.to_owned(),
            None => error.error.to_string(),
        })
    }

    pub(super) fn wifi_detail_classes(&self) -> Vec<&'static str> {
        let mut classes = vec!["network-connection-detail"];

        if self.has_wifi_error() {
            classes.push("error");
        }

        classes
    }

    pub(super) fn wifi_icon_classes(&self) -> Vec<&'static str> {
        let mut classes = vec!["network-connection-icon"];

        if self.has_wifi_error() {
            classes.push("error");
        } else {
            classes.push("wifi");
        }

        classes
    }

    pub(super) fn effective_wifi_icon(&self) -> String {
        let iwd = &self.config.config().modules.iwd;

        if self.has_wifi_error() {
            return iwd.wifi_offline_icon.get();
        }

        if self.is_connecting() {
            return iwd.wifi_acquiring_icon.get();
        }

        helpers::connected_signal_icon(
            self.station.strength,
            &iwd.wifi_signal_icons.get(),
            &iwd.wifi_connected_icon.get(),
        )
    }

    pub(super) fn disconnect(&self) {
        if let Some(station) = self.iwd.station.get() {
            station.disconnect();
        }
    }

    pub(super) fn dismiss_error(&self) {
        if let Some(station) = self.iwd.station.get() {
            station.dismiss_error();
        }
    }

    /// Forgets the active network, then disconnects from it (both queued, in
    /// that order). Only when the card offers it: the user asked to forget and
    /// disconnect, not to be dropped off a network whose credentials stay (or
    /// that has none saved).
    pub(super) fn forget(&self) {
        if !self.offers_forget() {
            return;
        }
        let (Some(network), Some(station)) =
            (self.station.connection.network(), self.iwd.station.get())
        else {
            return;
        };
        if helpers::forget_network(network) {
            station.disconnect();
        }
    }

    pub(super) fn status_classes(&self) -> Vec<&'static str> {
        let mut classes = vec!["badge-subtle", "network-connection-status"];

        if self.has_wifi_error() {
            classes.push("error");
        } else if self.is_connecting() {
            classes.push("warning");
        }

        classes
    }
}
