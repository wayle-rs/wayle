//! Compositor detection for compositor-dependent modules.

use std::env;

/// Detected Wayland compositor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Compositor {
    /// Hyprland compositor.
    Hyprland,
    /// niri compositor.
    Niri,
    /// Triad compositor.
    Triad,
    /// Unknown or unsupported compositor.
    Unknown(String),
}

impl Compositor {
    /// Detects the running Wayland compositor.
    pub(crate) fn detect() -> Self {
        if env::var("TRIAD_SOCKET").is_ok() || desktop_mentions_triad() {
            return Self::Triad;
        }

        if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            return Self::Hyprland;
        }

        if env::var("NIRI_SOCKET").is_ok() {
            return Self::Niri;
        }

        let desktop = env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
        Self::Unknown(desktop)
    }
}

fn desktop_mentions_triad() -> bool {
    [
        "XDG_CURRENT_DESKTOP",
        "XDG_SESSION_DESKTOP",
        "DESKTOP_SESSION",
    ]
    .iter()
    .filter_map(|name| env::var(name).ok())
    .any(|desktop| {
        desktop
            .split([':', ';', ','])
            .any(|part| part.eq_ignore_ascii_case("triad"))
            || desktop.to_ascii_lowercase().contains("triad")
    })
}
