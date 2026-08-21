use relm4::ComponentSender;
use tracing::warn;
use wayle_iwd::{ConnectionState, SecurityType};

use super::{ActiveConnections, messages::ConnectionError};
use crate::{
    i18n::t,
    shell::bar::dropdowns::iwd::helpers::{self, ForgetOutcome},
};

impl ActiveConnections {
    /// The failure to display, if any. Present only while the station is
    /// otherwise [`Idle`](ConnectionState::Idle), so a connection reconciled after
    /// a failed attempt (e.g. IWD rejected a passphrase without leaving the
    /// current network) suppresses the phantom failed target.
    fn shown_error(&self) -> Option<&ConnectionError> {
        self.error
            .as_ref()
            .filter(|_| matches!(self.station.connection, ConnectionState::Idle))
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

    /// Security type of the network the card is showing, resolved from the scan
    /// list by the SSID displayed. `None` when that network is not in the list
    /// (e.g. scan results have not arrived yet).
    fn active_security(&self) -> Option<SecurityType> {
        let ssid = self.station.connection.ssid()?;
        let station = self.iwd.station.get()?;

        helpers::security_for_ssid(&station.networks.get(), ssid)
    }

    /// Whether the card offers Forget for the active network.
    ///
    /// Only for networks whose credentials the shell could put back (see
    /// [`helpers::forgettable`]). A security type that cannot be resolved is
    /// treated the same way: without knowing what we would be destroying, the
    /// action is not offered.
    pub(super) fn offers_forget(&self) -> bool {
        self.active_security().is_some_and(helpers::forgettable)
    }

    pub(super) fn reset_station_watchers(&mut self, sender: &ComponentSender<Self>) {
        let token = self.station_watcher.reset();

        super::watchers::spawn_station_watchers(sender, &self.iwd, token);
    }

    pub(super) fn display_wifi_name(&self) -> String {
        if let Some(error) = self.shown_error() {
            return error.ssid.clone();
        }

        if let Some(ssid) = self.station.connection.ssid() {
            return ssid.to_string();
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
        if let Some(error) = self.shown_error() {
            return error.message.clone();
        }

        self.station
            .frequency
            .and_then(helpers::frequency_to_band)
            .map(str::to_string)
            .unwrap_or_default()
    }

    pub(super) fn error_tooltip(&self) -> Option<&str> {
        self.shown_error().map(|error| error.message.as_str())
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

    pub(super) fn disconnect(&self, sender: &ComponentSender<Self>) {
        let iwd = self.iwd.clone();
        sender.command(|_out, _shutdown| async move {
            if let Some(station) = iwd.station.get()
                && let Err(err) = station.disconnect().await
            {
                warn!(error = %err, "wifi disconnect failed");
            }
        });
    }

    pub(super) fn forget(&self, sender: &ComponentSender<Self>) {
        let iwd = self.iwd.clone();
        // The active connection is the connecting target or the connected
        // network — both carried by `connection`.
        let ssid = self.station.connection.ssid().map(str::to_string);

        sender.command(|_out, _shutdown| async move {
            let Some(ssid) = ssid else {
                return;
            };
            let Some(station) = iwd.station.get() else {
                return;
            };

            let target = station
                .networks
                .get()
                .into_iter()
                .find(|network| network.ssid.get() == ssid);

            // A refused forget leaves the connection alone too: the user asked to
            // forget and disconnect, not to be dropped off a network whose
            // credentials we kept. The card hides the action for those networks,
            // so this is a backstop.
            if let Some(network) = target
                && helpers::forget_network(&network).await == ForgetOutcome::Refused
            {
                return;
            }

            if let Err(err) = station.disconnect().await {
                warn!(error = %err, "wifi disconnect after forget failed");
            }
        });
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
