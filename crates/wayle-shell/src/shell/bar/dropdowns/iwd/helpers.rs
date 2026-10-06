use std::{cmp::Reverse, sync::Arc};

use tracing::warn;
use wayle_iwd::{Network, SecurityType, SignalStrength};

pub(crate) use crate::shell::bar::dropdowns::{
    connected_signal_icon, frequency_to_band, signal_strength_icon,
};

/// Snapshot of an IWD network for display in the network list. Equal
/// snapshots look and act the same, so a row whose snapshot is unchanged is
/// left alone.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NetworkSnapshot {
    /// The network the row shows and acts on: its identity, unlike its SSID
    /// (shared by networks of different security).
    pub network: Arc<Network>,
    pub ssid: String,
    pub strength: SignalStrength,
    pub security: SecurityType,
    pub known: bool,
}

impl NetworkSnapshot {
    pub(crate) fn of(network: &Arc<Network>) -> Self {
        Self {
            network: Arc::clone(network),
            ssid: network.ssid.get(),
            strength: SignalStrength::from_dbm(network.signal.get()),
            security: network.security.get(),
            known: network.known.get(),
        }
    }
}

/// Whether joining the network takes credentials of any kind (passphrase or
/// 802.1X), as opposed to an open network.
pub(crate) fn secured(security: SecurityType) -> bool {
    security != SecurityType::None
}

/// Whether the shell can connect to a network: an 802.1X network's
/// credentials come from an IWD provisioning file, which the shell can't
/// create, so one without (not known to IWD) can't be joined from here.
pub(crate) fn joinable(security: SecurityType, known: bool) -> bool {
    known || security != SecurityType::Enterprise
}

/// Whether saved credentials for this security type are ones the shell could put
/// back after forgetting them.
///
/// Enterprise (802.1X) networks are provisioned out of band — IWD reads their
/// credentials from a provisioning file that only the user can write, and the
/// shell has no way to recreate one. Forgetting such a network would destroy
/// configuration it cannot restore, so the Forget action is never offered for
/// them; every other type is re-establishable from the UI (open networks need
/// nothing, WEP/PSK prompt for a passphrase).
pub(crate) fn forgettable(security: SecurityType) -> bool {
    !matches!(security, SecurityType::Enterprise)
}

/// Forget `network`'s saved credentials, refusing networks whose credentials the
/// shell could not put back (see [`forgettable`]). Returns whether the forget
/// was asked for; its outcome shows in `network.known` (or the station's
/// `last_error`).
///
/// The UI never offers Forget for those networks, so the refusal here is a
/// backstop: it keeps the guarantee on the action itself, whichever path (e.g.
/// a stale request whose target changed under it) asks for the forget.
pub(crate) fn forget_network(network: &Arc<Network>) -> bool {
    if !forgettable(network.security.get()) {
        warn!(
            ssid = %network.ssid.get(),
            "refusing to forget an enterprise network: its credentials cannot be recreated"
        );
        return false;
    }

    network.forget();
    true
}

/// Orders networks for the list: known networks (ones IWD has a profile
/// for) first, then the others, each strongest signal first. Ties keep IWD's
/// order. (IWD's own order also puts known networks first, but ranks them by
/// how recently they were used.)
fn list_order(networks: &[Arc<Network>]) -> Vec<&Arc<Network>> {
    let mut ordered: Vec<&Arc<Network>> = networks.iter().collect();
    ordered.sort_by_key(|network| list_rank(network.known.get(), network.signal.get()));
    ordered
}

/// Where a network goes in the list: lower first.
fn list_rank(known: bool, signal: i16) -> (bool, Reverse<i16>) {
    (!known, Reverse(signal))
}

/// The list's rows: the networks in list order (see [`list_order`]), without
/// hidden networks (no SSID) and the `excluded` ones (e.g. the network shown in
/// the active-connection card, or in the password form).
pub(crate) fn list_snapshots(
    networks: &[Arc<Network>],
    excluded: &[&Arc<Network>],
) -> Vec<NetworkSnapshot> {
    list_order(networks)
        .into_iter()
        .filter(|network| !excluded.contains(network))
        .map(NetworkSnapshot::of)
        .filter(|snapshot| !snapshot.ssid.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_open_networks_are_unsecured() {
        assert!(!secured(SecurityType::None));
        assert!(secured(SecurityType::Enterprise));
        assert!(secured(SecurityType::Psk));
        assert!(secured(SecurityType::Wep));
    }

    #[test]
    fn known_networks_come_first_then_the_strongest() {
        let mut networks = [
            ("weak-known", true, -80),
            ("strong", false, -40),
            ("strong-known", true, -50),
            ("weak", false, -85),
        ];

        networks.sort_by_key(|&(_, known, signal)| list_rank(known, signal));

        let names: Vec<&str> = networks.iter().map(|&(name, ..)| name).collect();
        assert_eq!(names, ["strong-known", "weak-known", "strong", "weak"]);
    }

    #[test]
    fn only_enterprise_networks_need_provisioning_to_join() {
        assert!(!joinable(SecurityType::Enterprise, false));
        assert!(joinable(SecurityType::Enterprise, true));
        assert!(joinable(SecurityType::Psk, false));
        assert!(joinable(SecurityType::None, false));
    }

    #[test]
    fn only_enterprise_is_unforgettable() {
        assert!(!forgettable(SecurityType::Enterprise));
        assert!(forgettable(SecurityType::None));
        assert!(forgettable(SecurityType::Psk));
        assert!(forgettable(SecurityType::Wep));
    }
}
