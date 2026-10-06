use std::{collections::HashMap, sync::Arc};

use relm4::{factory::FactoryVecDequeGuard, prelude::*};
use wayle_iwd::{ConnectionState, Network, SecurityType, SignalStrength, Station, StationAction};

use crate::{
    i18n::t,
    shell::bar::dropdowns::iwd::{
        available_networks::{
            AvailableNetworks,
            messages::AvailableNetworksInput,
            network_item::{NetworkItem, NetworkItemInit, NetworkItemOutput, snapshot_key},
            watchers,
        },
        helpers,
        password_form::{PasswordFormInput, PasswordFormOutput},
    },
};

impl AvailableNetworks {
    fn station(&self) -> Option<Arc<Station>> {
        self.iwd.station.get()
    }

    /// Whether IWD has a WiFi device.
    pub(super) fn wifi_available(&self) -> bool {
        self.station().is_some()
    }

    /// Whether WiFi is on.
    pub(super) fn powered(&self) -> bool {
        self.station().is_some_and(|station| station.powered.get())
    }

    /// Whether IWD refused to turn WiFi on because a hardware switch blocks the
    /// radio. IWD reports a hardware block only when asked to lift it.
    pub(super) fn hardware_blocked(&self) -> bool {
        self.iwd
            .station
            .get()
            .and_then(|station| station.last_error.get())
            .is_some_and(|error| error.hardware_blocked())
    }

    /// Whether IWD is waiting for a passphrase, which the password form asks
    /// for.
    pub(super) fn prompting(&self) -> bool {
        self.iwd.passphrase_request.get().is_some()
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
        let snapshots = match self.station().filter(|station| station.powered.get()) {
            Some(station) => {
                // Shown elsewhere: the network of the connection (in the
                // active-connection card), and the one IWD asks a passphrase
                // for (in the password form).
                let connection = station.connection.get();
                let request = self.iwd.passphrase_request.get();
                let excluded: Vec<&Arc<Network>> = connection
                    .network()
                    .into_iter()
                    .chain(request.as_ref().map(|request| &request.network))
                    .collect();
                helpers::list_snapshots(&station.networks.get(), &excluded)
            }
            None => vec![],
        };

        // Resolve the configured signal icons once for this rebuild.
        let (signal_icons, connected_icon) = {
            let iwd = &self.config.config().modules.iwd;
            (iwd.wifi_signal_icons.get(), iwd.wifi_connected_icon.get())
        };
        let icon_for = |snapshot: &helpers::NetworkSnapshot| {
            helpers::signal_strength_icon(snapshot.strength, &signal_icons, &connected_icon)
        };

        let previous_slots = self.visible.then(|| self.row_slots());
        reconcile_list(&mut self.network_list.guard(), &snapshots, icon_for);

        // The active-connection card above the list appearing or going away
        // moves every row, whatever its slot.
        let card_shown = self.card_shown();
        let card_changed = card_shown != self.card_shown;
        self.card_shown = card_shown;
        if let Some(previous_slots) = previous_slots {
            if card_changed {
                self.guard_all_rows();
            } else {
                self.guard_shifted_rows(&previous_slots);
            }
        }
    }

    /// Whether the active-connection card above the list is shown: a
    /// connection is under way or up, or a failed connect is reported.
    fn card_shown(&self) -> bool {
        self.iwd.station.get().is_some_and(|station| {
            station.connection.get() != ConnectionState::Idle
                || station
                    .last_error
                    .get()
                    .is_some_and(|error| matches!(error.action, StationAction::Connect { .. }))
        })
    }

    /// Every row, top to bottom.
    fn rows(&self) -> impl Iterator<Item = &NetworkItem> {
        (0..self.network_list.len()).filter_map(|idx| self.network_list.get(idx))
    }

    /// Each row's slot on screen, by key.
    fn row_slots(&self) -> HashMap<String, usize> {
        self.rows()
            .enumerate()
            .map(|(slot, row)| (row.key().to_owned(), slot))
            .collect()
    }

    /// Guards every row now in a different slot than before, so a click aimed
    /// at whatever used to be there doesn't land on it. A new row is guarded
    /// only if its slot was already occupied (a row whose buttons changed is
    /// new, as it is rebuilt).
    fn guard_shifted_rows(&self, previous_slots: &HashMap<String, usize>) {
        for (slot, row) in self.rows().enumerate() {
            let shifted = match previous_slots.get(row.key()) {
                Some(&previous_slot) => previous_slot != slot,
                None => slot < previous_slots.len(),
            };
            if shifted {
                row.arm_click_guard();
            }
        }
    }

    /// Guards every row, for layout changes above the list (the password form
    /// or the active-connection card appearing or going away).
    fn guard_all_rows(&self) {
        if self.visible {
            for row in self.rows() {
                row.arm_click_guard();
            }
        }
    }

    pub(super) fn handle_visibility_changed(&mut self, visible: bool) {
        self.visible = visible;
        if !visible {
            for row in self.rows() {
                row.disarm_click_guard();
            }
        }
    }

    /// IWD's device appeared, went away or was replaced: follow the new one.
    pub(super) fn handle_station_changed(&mut self, sender: &ComponentSender<Self>) {
        let token = self.networks_watcher.reset();
        if let Some(station) = self.station() {
            watchers::spawn(sender, &station, token);
        }
        self.rebuild_network_list();
    }

    /// Shows IWD's passphrase request in the form, or hides the form once IWD
    /// stops asking.
    pub(super) fn show_passphrase_request(&mut self) {
        match self.iwd.passphrase_request.get() {
            Some(request) => {
                let network = &request.network;
                self.password_form.emit(PasswordFormInput::Show {
                    ssid: network.ssid.get(),
                    security_label: translate_security_type(network.security.get()),
                    signal_icon: self.signal_icon(SignalStrength::from_dbm(network.signal.get())),
                    error_message: request
                        .rejected
                        .then(|| t!("dropdown-iwd-error-connect-failed")),
                    // It may appear under the pointer.
                    guard: self.visible,
                });
            }
            None => self.password_form.emit(PasswordFormInput::Hide),
        }
        self.rebuild_network_list();
        // The form appearing or going away moves every row below it.
        self.guard_all_rows();
    }

    /// Connects to the chosen row's network. For a secured network without
    /// saved credentials, IWD then asks for the passphrase, and the form opens.
    pub(super) fn select_network(network: &Arc<Network>) {
        network.connect();
    }

    pub(super) fn handle_password_form(&self, form_output: PasswordFormOutput) {
        match form_output {
            PasswordFormOutput::Connect { password } => self.iwd.provide_passphrase(password),
            PasswordFormOutput::Cancel => self.iwd.cancel_passphrase_request(),
        }
    }

    pub(super) fn forget_network(network: &Arc<Network>) {
        helpers::forget_network(network);
    }
}

/// Reconciles the factory in place within a single guard, keyed by network
/// (see [`NetworkItem::key`]).
///
/// Rows whose network is no longer listed are dropped, and so are rows that
/// must be rebuilt (their Forget availability changed, see
/// [`NetworkItem::reusable_for`]). Then, walking the new order from the top,
/// each network's row is moved to its place (or inserted, if new); the rows
/// above it are already in place. Rows keep their widget (so the popover and
/// its scroll position are undisturbed), and only a row that would show
/// something different is marked changed, so an unchanged list renders
/// nothing. (relm4 works out the widget moves from the final order itself.)
fn reconcile_list(
    guard: &mut FactoryVecDequeGuard<'_, NetworkItem>,
    snapshots: &[helpers::NetworkSnapshot],
    icon_for: impl Fn(&helpers::NetworkSnapshot) -> String,
) {
    let listed: HashMap<&str, &helpers::NetworkSnapshot> = snapshots
        .iter()
        .map(|snapshot| (snapshot_key(snapshot), snapshot))
        .collect();

    for idx in (0..guard.len()).rev() {
        let keep = guard.get(idx).is_some_and(|item| {
            listed
                .get(item.key())
                .is_some_and(|snapshot| item.reusable_for(snapshot))
        });
        if !keep {
            guard.remove(idx);
        }
    }

    let rows: HashMap<String, DynamicIndex> = (0..guard.len())
        .filter_map(|idx| guard.get(idx))
        .map(|item| (item.key().to_owned(), item.index.clone()))
        .collect();

    for (position, snapshot) in snapshots.iter().enumerate() {
        let icon = icon_for(snapshot);
        let Some(row) = rows.get(snapshot_key(snapshot)) else {
            guard.insert(
                position,
                NetworkItemInit {
                    snapshot: snapshot.clone(),
                    icon,
                },
            );
            continue;
        };

        let current = row.current_index();
        if current != position {
            guard.move_to(current, position);
        }

        let needs_update = guard
            .get(position)
            .is_some_and(|item| item.differs_from(snapshot, &icon));
        if needs_update && let Some(item) = guard.get_mut(position) {
            item.refresh(snapshot, icon);
        }
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
        NetworkItemOutput::Selected(network) => AvailableNetworksInput::NetworkSelected(network),
        NetworkItemOutput::ForgetRequested(network) => {
            AvailableNetworksInput::ForgetNetwork(network)
        }
    }
}
