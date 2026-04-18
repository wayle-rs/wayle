use std::sync::Arc;

use wayle_config::ConfigService;
use wayle_network::NetworkService;
use zbus::zvariant::OwnedObjectPath;

/// UI-friendly snapshot of a VPN tunnel's current state.
#[derive(Debug, Clone)]
pub(crate) struct TunnelState {
    pub uuid: String,
    pub name: String,
    pub active: bool,
    /// Whether this tunnel is managed externally (e.g. by wg-quick)
    /// rather than by NetworkManager. Externally managed tunnels
    /// should not be toggled via NM as it can corrupt their config.
    pub externally_managed: bool,
    pub ip4_address: Option<String>,
    pub interface_name: Option<String>,
    pub connection_path: OwnedObjectPath,
}

/// Configuration for a specific VPN provider's UI presentation and behavior.
///
/// Each provider (WireGuard, OpenVPN, etc.) supplies one of these to
/// customize the generic VPN tunnels component.
pub(crate) struct VpnProviderConfig {
    /// Localized section title (e.g., "WireGuard Tunnels").
    pub section_label: String,
    /// Fallback display name when a tunnel has no connection ID
    /// (e.g., "WireGuard", "OpenVPN").
    pub fallback_name: &'static str,
    /// GTK icon name for tunnel cards.
    pub icon: &'static str,
    /// Glob pattern for the import file dialog (e.g., "*.conf").
    pub import_filter: &'static str,
    /// Human-readable label for the file filter (e.g., "WireGuard Config (*.conf)").
    pub import_filter_label: &'static str,
}

pub(crate) struct VpnTunnelsInit {
    pub network: Arc<NetworkService>,
    pub config: Arc<ConfigService>,
    pub provider: VpnProviderConfig,
}

#[derive(Debug)]
pub(crate) enum VpnTunnelsInput {
    /// Toggle tunnel (index, desired_active_state).
    ToggleTunnel(usize, bool),
    RenameTunnel(usize, String),
    ImportConfig,
    FileSelected(String, String),
}

#[derive(Debug)]
pub(crate) enum VpnTunnelsCmd {
    TunnelsChanged(Vec<TunnelState>),
    ServiceChanged,
    ToggleResult(usize, Result<(), String>),
    ImportResult(Result<String, String>),
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum VpnTunnelsOutput {
    Activated(String),
    Deactivated(String),
    Error(String),
}
