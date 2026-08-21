use relm4::prelude::*;
use tracing::warn;
use wayle_iwd::{ConnectionState, Error, SecurityType, SignalStrength};
use zbus::zvariant::OwnedObjectPath;

use crate::{
    i18n::t,
    shell::bar::dropdowns::iwd::{
        available_networks::{
            AvailableNetworks, ListState,
            messages::{
                AvailableNetworksCmd, AvailableNetworksInput, AvailableNetworksOutput,
                SelectedNetwork,
            },
            network_item::{NetworkItemInit, NetworkItemOutput},
            watchers,
        },
        helpers,
        password_form::{PasswordFormInput, PasswordFormOutput},
    },
};

impl AvailableNetworks {
    pub(super) fn clear_selection(&mut self) {
        self.selection = None;
    }

    pub(super) fn handle_connection_failure(
        &mut self,
        message: String,
        sender: &ComponentSender<Self>,
    ) {
        // Capture the failed target before clearing the selection.
        let ssid = self.selection.as_ref().map(|selection| selection.ssid.clone());

        self.state = ListState::Normal;
        self.clear_selection();
        self.rebuild_network_list();

        // Only surface a card error if the attempt left us disconnected. If IWD
        // stayed on (or returned to) a connection, the service already
        // reconciled `connection` to it, so the failure is moot. Read live —
        // `connect()` sets this synchronously before returning the error.
        let still_idle = match self.iwd.station.get() {
            Some(station) => matches!(station.connection.get(), ConnectionState::Idle),
            None => true,
        };

        if let Some(ssid) = ssid
            && still_idle
        {
            let _ = sender.output(AvailableNetworksOutput::ConnectionFailed { ssid, message });
        }
    }

    pub(super) fn connect_to_selected(
        &mut self,
        password: Option<String>,
        sender: &ComponentSender<Self>,
    ) {
        let Some(selection) = &self.selection else {
            return;
        };

        let Some(station) = self.iwd.station.get() else {
            return;
        };

        let network_path = selection.network_path.clone();
        let secured = selection.secured;

        // The service publishes `connection = Connecting{ssid}` as the first step
        // of `connect()`, which drives the card and the list exclusion — no
        // optimistic shell state needed here.
        sender.command(move |out, _shutdown| async move {
            // If the user supplied a passphrase (e.g. retrying after an auth
            // failure), clear any saved credentials first. IWD reuses a known
            // network's stored passphrase and never asks our agent, so a
            // re-entered password would otherwise be silently ignored. This is a
            // no-op when the network has no saved credentials.
            if password.is_some()
                && let Some(network) = station
                    .networks
                    .get()
                    .into_iter()
                    .find(|network| network.object_path() == &network_path)
            {
                let _ = helpers::forget_network(&network).await;
            }

            match station.connect(network_path, password).await {
                // Connected, or aborted (cancelled/superseded) — both just reset
                // the list with no error on the card.
                Ok(()) | Err(Error::ConnectionAborted) => {
                    let _ = out.send(AvailableNetworksCmd::ConnectionSettled);
                }
                Err(Error::ConnectionFailed) if secured => {
                    // IWD reports a rejected passphrase only as the generic
                    // `Failed` error, so re-prompt for the password. Every other
                    // error (Timeout, NoAgent, NotConfigured, ...) falls through
                    // to the generic message below and never re-prompts.
                    let _ = out.send(AvailableNetworksCmd::ConnectionAuthFailed);
                }
                Err(err) => {
                    warn!(error = %err, "iwd connection failed");
                    let _ = out.send(AvailableNetworksCmd::ConnectionFailed(t!(
                        "dropdown-iwd-error-generic"
                    )));
                }
            }
        });
    }

    pub(super) fn handle_wifi_availability(
        &mut self,
        available: bool,
        sender: &ComponentSender<Self>,
    ) {
        self.wifi_available = available;

        let station = self.iwd.station.get();
        self.powered = station.as_ref().is_some_and(|station| station.powered.get());

        let token = self.networks_watcher.reset();
        if let Some(station) = station {
            watchers::spawn(sender, &station, token);
        }

        if !available {
            self.state = ListState::Normal;
            self.clear_selection();
            self.password_form.emit(PasswordFormInput::Hide);
        }

        self.rebuild_network_list();
    }

    pub(super) fn handle_wifi_enabled(&mut self, enabled: bool) {
        self.powered = enabled;

        if enabled {
            self.rebuild_network_list();
            return;
        }

        self.network_list.guard().clear();
        self.state = ListState::Normal;
        self.clear_selection();
        self.password_form.emit(PasswordFormInput::Hide);
    }

    /// The SSID currently shown as the Active Connection, and therefore excluded
    /// from the available list: the in-progress connecting target or the
    /// connected network. Sourced from the service's reconciled `connection`,
    /// which already favours the in-progress target over the network IWD still
    /// reports as connected during a transition.
    fn active_ssid(&self) -> Option<String> {
        self.iwd
            .station
            .get()
            .and_then(|station| station.connection.get().ssid().map(str::to_string))
    }

    /// The configured "WiFi disabled" icon, shown in the powered-off and
    /// no-networks empty states.
    pub(super) fn disabled_icon(&self) -> String {
        self.config.config().modules.iwd.wifi_disabled_icon.get()
    }

    /// Resolve the configured signal-strength icon for a bucket.
    pub(super) fn signal_icon(&self, strength: SignalStrength) -> String {
        let iwd = &self.config.config().modules.iwd;
        helpers::signal_strength_icon(
            strength,
            &iwd.wifi_signal_icons.get(),
            &iwd.wifi_connected_icon.get(),
        )
    }

    pub(super) fn rebuild_network_list(&mut self) {
        let active_ssid = self.active_ssid();
        let mut snapshots = match self.iwd.station.get() {
            Some(station) => {
                helpers::unique_networks(&station.networks.get(), active_ssid.as_deref())
            }
            None => vec![],
        };

        // While entering a password, hide the targeted network too — it is shown
        // in the password form, not the available list.
        if self.state == ListState::PasswordEntry
            && let Some(selection) = &self.selection
        {
            snapshots.retain(|network| network.ssid != selection.ssid);
        }

        // Resolve the configured signal icons once for this rebuild.
        let (signal_icons, connected_icon) = {
            let iwd = &self.config.config().modules.iwd;
            (iwd.wifi_signal_icons.get(), iwd.wifi_connected_icon.get())
        };
        let icon_for = |snapshot: &helpers::NetworkSnapshot| {
            helpers::signal_strength_icon(snapshot.strength, &signal_icons, &connected_icon)
        };

        // Reconcile the factory in place rather than clearing and rebuilding it.
        // A wholesale destroy/recreate of every row while the dropdown popover is
        // open disturbs the popover and closes it; reusing rows also preserves
        // scroll position. Only genuinely added/removed/moved rows change.
        let mut guard = self.network_list.guard();

        // 1. Drop rows whose SSID is gone (reverse, to keep indices stable).
        let mut idx = guard.len();
        while idx > 0 {
            idx -= 1;
            let gone = guard
                .get(idx)
                .is_some_and(|item| !snapshots.iter().any(|s| s.ssid == item.ssid()));
            if gone {
                guard.remove(idx);
            }
        }

        // 2. Walk the target order, reusing rows where possible.
        for (i, snapshot) in snapshots.iter().enumerate() {
            let at_i = guard
                .get(i)
                .map(|item| (item.ssid() == snapshot.ssid, item.reusable_for(snapshot)));
            match at_i {
                // Same network already here: update in place.
                Some((true, true)) => {
                    if let Some(item) = guard.get_mut(i) {
                        item.refresh(snapshot, icon_for(snapshot));
                    }
                }
                // Same SSID but needs a fresh widget (e.g. it became known).
                Some((true, false)) => {
                    guard.remove(i);
                    guard.insert(i, NetworkItemInit { snapshot: snapshot.clone(), icon: icon_for(snapshot) });
                }
                // Wrong row here: pull the matching one up if it exists further
                // down, otherwise insert a new row at this position.
                Some((false, _)) => {
                    if let Some(j) =
                        (i + 1..guard.len()).find(|&j| guard.get(j).is_some_and(|item| item.ssid() == snapshot.ssid))
                    {
                        guard.remove(j);
                    }
                    guard.insert(i, NetworkItemInit { snapshot: snapshot.clone(), icon: icon_for(snapshot) });
                }
                // Past the end: append.
                None => {
                    guard.push_back(NetworkItemInit { snapshot: snapshot.clone(), icon: icon_for(snapshot) });
                }
            }
        }

        // 3. Drop any trailing rows (safety net; step 1 + 2 normally leave none).
        while guard.len() > snapshots.len() {
            guard.remove(guard.len() - 1);
        }

        drop(guard);
    }

    /// Dismiss the password prompt when it is no longer relevant: the target
    /// network dropped out of the scan, or it became the active connection
    /// (connecting or connected) — e.g. another client such as `iwctl` connected
    /// to it under us. The password form is hidden by its `state ==
    /// PasswordEntry` visibility binding.
    pub(super) fn dismiss_stale_password_entry(&mut self) {
        if self.state != ListState::PasswordEntry {
            return;
        }

        let Some(ssid) = self.selection.as_ref().map(|selection| selection.ssid.clone()) else {
            return;
        };

        let station = self.iwd.station.get();

        // Gone from the scan results — checked against the raw networks, since
        // the displayed list intentionally filters the prompted SSID out.
        let still_visible = station.as_ref().is_some_and(|station| {
            station
                .networks
                .get()
                .iter()
                .any(|network| network.ssid.get() == ssid)
        });

        // Became the active connection via any client.
        let now_active = station
            .as_ref()
            .and_then(|station| station.connection.get().ssid().map(str::to_string))
            .is_some_and(|active| active == ssid);

        if !still_visible || now_active {
            self.state = ListState::Normal;
            self.clear_selection();
        }
    }

    pub(super) fn select_network(&mut self, index: usize, sender: &ComponentSender<Self>) {
        // The chosen factory row is the single source of per-network state; copy
        // out what we need before mutating self (ending the factory borrow).
        let Some(item) = self.network_list.get(index) else {
            return;
        };
        let network_path = item.object_path().clone();
        let ssid = item.ssid().to_string();
        let security = item.security();
        let strength = item.strength();
        let known = item.known();

        let security_label = translate_security_type(security);
        let signal_icon = self.signal_icon(strength);
        let secured = helpers::requires_password(security);

        self.selection = Some(SelectedNetwork {
            network_path,
            ssid: ssid.clone(),
            security_label: security_label.clone(),
            strength,
            secured,
        });

        if secured && !known {
            self.state = ListState::PasswordEntry;

            self.password_form.emit(PasswordFormInput::Show {
                ssid,
                security_label,
                signal_icon,
                error_message: None,
            });

            // Drop the targeted network from the list now that it is in the form.
            self.rebuild_network_list();
        } else {
            self.connect_to_selected(None, sender);
        }
    }

    pub(super) fn handle_password_form(
        &mut self,
        form_output: PasswordFormOutput,
        sender: &ComponentSender<Self>,
    ) {
        match form_output {
            PasswordFormOutput::Connect { password } => {
                // Close the form but keep the selection so an auth failure can
                // re-prompt; the service owns the Connecting state from here.
                self.state = ListState::Normal;
                self.connect_to_selected(Some(password), sender);
            }
            PasswordFormOutput::Cancel => {
                self.state = ListState::Normal;
                self.clear_selection();
                self.rebuild_network_list();
            }
        }
    }

    pub(super) fn forget_network(
        &self,
        network_path: OwnedObjectPath,
        sender: &ComponentSender<Self>,
    ) {
        let iwd = self.iwd.clone();

        sender.oneshot_command(async move {
            if let Some(station) = iwd.station.get() {
                let target = station
                    .networks
                    .get()
                    .into_iter()
                    .find(|network| network.object_path() == &network_path);

                if let Some(network) = target {
                    let _ = helpers::forget_network(&network).await;
                }
            }

            AvailableNetworksCmd::NetworksChanged
        });
    }
}

pub(super) fn translate_security_type(security: SecurityType) -> String {
    match security {
        SecurityType::None => t!("dropdown-iwd-security-open"),
        SecurityType::Wep => t!("dropdown-iwd-security-wep"),
        SecurityType::Psk => t!("dropdown-iwd-security-psk"),
        SecurityType::Enterprise => t!("dropdown-iwd-security-enterprise"),
    }
}

pub(super) fn forward_network_item_output(
    item_output: NetworkItemOutput,
) -> AvailableNetworksInput {
    match item_output {
        NetworkItemOutput::Selected(index) => {
            AvailableNetworksInput::NetworkSelected(index.current_index())
        }

        NetworkItemOutput::ForgetRequested(path) => AvailableNetworksInput::ForgetNetwork(path),
    }
}
