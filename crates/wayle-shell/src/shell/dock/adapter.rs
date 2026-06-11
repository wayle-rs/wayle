//! Compositor-agnostic dock adapter trait and Niri implementation.

use std::sync::Arc;

use indexmap::IndexMap;
use wayle_niri::NiriService;

use super::DockAppData;

/// Trait for compositor-specific dock data retrieval and actions.
pub trait DockAdapter {
    /// Compute the current list of running apps from compositor state.
    fn compute_running_apps(&self) -> Vec<DockAppData>;

    /// Focus windows belonging to `app_id`. If no windows exist,
    /// launch the app. Spawns a task, returns immediately.
    fn focus_app(&self, app_id: &str);
}

/// Niri-specific dock adapter.
pub struct NiriDockAdapter {
    niri: Arc<NiriService>,
}

impl NiriDockAdapter {
    pub fn new(niri: Arc<NiriService>) -> Self {
        Self { niri }
    }
    
    pub(crate) fn niri(&self) -> Arc<NiriService> {
        self.niri.clone()
    }
}

impl DockAdapter for NiriDockAdapter {
    fn compute_running_apps(&self) -> Vec<DockAppData> {
        let windows = self.niri.windows.get();
        let focused_id = self.niri.focused_window_id.get();

        let mut groups: IndexMap<String, (u32, bool)> = IndexMap::new();
        for window in windows.values() {
            if let Some(app_id) = window.app_id.get() {
                if app_id.is_empty() {
                    continue;
                }
                let entry = groups.entry(app_id.to_string()).or_insert((0, false));
                entry.0 += 1;
            }
        }

        if let Some(focused_id) = focused_id {
            for window in windows.values() {
                if window.id.get() == focused_id {
                    if let Some(app_id) = window.app_id.get() {
                        if !app_id.is_empty() {
                            if let Some(entry) = groups.get_mut(app_id.as_str()) {
                                entry.1 = true;
                            }
                        }
                    }
                    break;
                }
            }
        }

        let apps: Vec<DockAppData> = groups
            .into_iter()
            .map(|(app_id, (window_count, is_active))| DockAppData {
                app_id,
                is_active,
                window_count,
            })
            .collect();

        tracing::debug!(app_count = apps.len(), "Computed Niri running apps");
        apps
    }

    fn focus_app(&self, app_id: &str) {
        let niri = self.niri.clone();
        let app_id = app_id.to_string();
        tokio::spawn(async move {
            let focused_id = niri.focused_window_id.get();
            let app_windows: Vec<(u64,)> = niri
                .windows
                .get()
                .iter()
                .filter(|(_, w)| w.app_id.get().as_deref() == Some(app_id.as_str()))
                .map(|(id, _)| (*id,))
                .collect();

            if app_windows.is_empty() {
                let _ = niri
                    .spawn(vec![format!("/usr/bin/xdg-open {}.desktop", app_id)])
                    .await;
                return;
            }

            let is_focused = if let Some(focused_id) = focused_id {
                app_windows.iter().any(|(wid,)| *wid == focused_id)
            } else {
                false
            };

            if !is_focused {
                if let Some((id,)) = app_windows.first() {
                    let _ = niri.focus_window(*id).await;
                }
            }
        });
    }
}

/// Enum holding whichever compositor adapter is active.
#[allow(clippy::large_enum_variant)]
pub enum DockAdapterRef {
    Niri(NiriDockAdapter),
    Hyprland(crate::shell::dock::adapter_hyprland::HyprlandDockAdapter),
}

impl DockAdapterRef {
    pub fn compute_running_apps(&self) -> Vec<DockAppData> {
        match self {
            DockAdapterRef::Niri(a) => a.compute_running_apps(),
            DockAdapterRef::Hyprland(a) => a.compute_running_apps(),
        }
    }

    pub fn focus_app(&self, app_id: &str) {
        match self {
            DockAdapterRef::Niri(a) => a.focus_app(app_id),
            DockAdapterRef::Hyprland(a) => a.focus_app(app_id),
        }
    }

    pub(crate) fn niri(&self) -> Option<Arc<NiriService>> {
        match self {
            DockAdapterRef::Niri(a) => Some(a.niri()),
            DockAdapterRef::Hyprland(_) => None,
        }
    }

    pub(crate) fn hyprland(&self) -> Option<Arc<wayle_hyprland::HyprlandService>> {
        match self {
            DockAdapterRef::Niri(_) => None,
            DockAdapterRef::Hyprland(a) => Some(a.hyprland()),
        }
    }
}

impl Clone for DockAdapterRef {
    fn clone(&self) -> Self {
        match self {
            DockAdapterRef::Niri(niri) => {
                DockAdapterRef::Niri(NiriDockAdapter::new(niri.niri.clone()))
            }
            DockAdapterRef::Hyprland(hyprland) => {
                DockAdapterRef::Hyprland(
                    crate::shell::dock::adapter_hyprland::HyprlandDockAdapter::new(
                        hyprland.hyprland(),
                    ),
                )
            }
        }
    }
}
