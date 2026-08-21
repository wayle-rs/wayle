use std::{collections::HashSet, sync::Arc};

use tracing::warn;
use wayle_iwd::{Network, SecurityType, SignalStrength};
use zbus::zvariant::OwnedObjectPath;

pub(crate) use crate::shell::bar::dropdowns::{
    connected_signal_icon, frequency_to_band, signal_strength_icon,
};

/// Snapshot of an IWD network for display in the network list.
#[derive(Debug, Clone)]
pub(crate) struct NetworkSnapshot {
    pub ssid: String,
    pub strength: SignalStrength,
    pub security: SecurityType,
    pub object_path: OwnedObjectPath,
    pub known: bool,
}

pub(crate) fn requires_password(security: SecurityType) -> bool {
    !matches!(security, SecurityType::None | SecurityType::Enterprise)
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

/// Outcome of a [`forget_network`] request.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ForgetOutcome {
    /// The saved credentials are gone (or there were none to begin with).
    Forgotten,
    /// Refused before reaching IWD: these are credentials the shell could not put
    /// back (see [`forgettable`]).
    Refused,
    /// The request reached IWD and failed.
    Failed,
}

/// Forget `network`'s saved credentials, refusing networks whose credentials the
/// shell could not put back (see [`forgettable`]).
///
/// The UI never offers Forget for those networks, so the refusal here is a
/// backstop: it keeps the guarantee on the action itself, whichever path — a
/// stale request whose target changed under it, a re-entered passphrase clearing
/// stale credentials — asks for the forget.
pub(crate) async fn forget_network(network: &Network) -> ForgetOutcome {
    if !forgettable(network.security.get()) {
        warn!(
            ssid = %network.ssid.get(),
            "refusing to forget an enterprise network: its credentials cannot be recreated"
        );
        return ForgetOutcome::Refused;
    }

    if let Err(err) = network.forget().await {
        warn!(error = %err, "forget network failed");
        return ForgetOutcome::Failed;
    }

    ForgetOutcome::Forgotten
}

/// Security type of the visible network named `ssid`, if it is in the scan list.
///
/// The active-connection card knows only the SSID it displays, so this is how it
/// recovers the security type behind it.
pub(crate) fn security_for_ssid(networks: &[Arc<Network>], ssid: &str) -> Option<SecurityType> {
    networks
        .iter()
        .find(|network| network.ssid.get() == ssid)
        .map(|network| network.security.get())
}

/// Deduplicates networks by SSID and filters out hidden networks and the active
/// SSID (the network shown in the active-connection card — either the connected
/// network or the in-progress connecting target).
///
/// The input is expected to already be ordered strongest-first (as
/// `Station.GetOrderedNetworks` returns it), and that order is preserved — like
/// iwgtk, we keep IWD's ordering rather than re-sorting by the coarse signal
/// bucket.
pub(crate) fn unique_networks(
    networks: &[Arc<Network>],
    active_ssid: Option<&str>,
) -> Vec<NetworkSnapshot> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut snapshots: Vec<NetworkSnapshot> = Vec::new();

    for network in networks {
        let ssid = network.ssid.get();
        if ssid.is_empty() {
            continue;
        }

        let security = network.security.get();

        if active_ssid.is_some_and(|active| active == ssid) {
            continue;
        }

        // First occurrence per SSID is the strongest (input is sorted).
        if !seen.insert(ssid.clone()) {
            continue;
        }

        snapshots.push(NetworkSnapshot {
            ssid: ssid.clone(),
            strength: network.strength.get(),
            security,
            object_path: network.object_path().clone(),
            known: network.known.get(),
        });
    }

    snapshots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_password_logic() {
        assert!(!requires_password(SecurityType::None));
        assert!(!requires_password(SecurityType::Enterprise));
        assert!(requires_password(SecurityType::Psk));
        assert!(requires_password(SecurityType::Wep));
    }

    #[test]
    fn only_enterprise_is_unforgettable() {
        assert!(!forgettable(SecurityType::Enterprise));
        assert!(forgettable(SecurityType::None));
        assert!(forgettable(SecurityType::Psk));
        assert!(forgettable(SecurityType::Wep));
    }
}
