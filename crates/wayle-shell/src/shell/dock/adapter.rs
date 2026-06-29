//! Compositor-agnostic dock adapter trait and Niri implementation.

use std::{rc::Rc, sync::Arc};

use indexmap::IndexMap;
use relm4::{gtk, gtk::prelude::*};
use wayle_niri::NiriService;

use super::DockAppData;

/// Shared state for tracking the currently open dock popover across all
/// dock items. Since all GTK operations run on the main thread, Rc<RefCell<...>>
/// is sufficient (no Arc/Mutex needed).
///
/// This replaces the previous static registry approach which failed because
/// gtk::Popover is not Send+Sync and cannot be stored in a static Mutex.
pub type OpenPopoverTracker = Rc<std::cell::RefCell<Option<Rc<gtk::Popover>>>>;

/// Create a new popover tracker.
pub fn create_open_popover_tracker() -> OpenPopoverTracker {
    Rc::new(std::cell::RefCell::new(None))
}

/// Set the currently open popover in the tracker.
pub fn set_open_popover(tracker: &OpenPopoverTracker, popover: &gtk::Popover) {
    // Unparent the old popover if it's still parented (prevents GTK warnings
    // about popup() on already-parented popovers)
    if let Some(ref old) = *tracker.borrow() {
        if old.parent().is_some() {
            old.unparent();
        }
    }
    let new_ref = Rc::new(popover.clone());
    *tracker.borrow_mut() = Some(new_ref);
}

/// Window info for the dock window popover.
#[derive(Debug, Clone)]
pub struct DockWindow {
    pub identifier: String,
    pub title: String,
}

/// Trait for compositor-specific dock data retrieval and actions.
pub trait DockAdapter {
    /// Compute the current list of running apps from compositor state.
    fn compute_running_apps(&self) -> Vec<DockAppData>;

    /// Focus windows belonging to `app_id`. If no windows exist,
    /// launch the app. Spawns a task, returns immediately.
    fn focus_app(&self, app_id: &str);

    /// Get the list of windows for `app_id`.
    fn get_windows(&self, app_id: &str) -> Vec<DockWindow>;

    /// Focus a specific window by its identifier (u64 for Niri,
    /// address for Hyprland).
    fn focus_window(&self, identifier: &str);
}

/// Niri-specific dock adapter.
pub struct NiriDockAdapter {
    pub(crate) niri: Arc<NiriService>,
}

impl NiriDockAdapter {
    pub fn new(niri: Arc<NiriService>) -> Self {
        Self { niri }
    }
}

impl DockAdapter for NiriDockAdapter {
    fn compute_running_apps(&self) -> Vec<DockAppData> {
        let windows = self.niri.windows.get();
        tracing::debug!(" --> niri windows: {:#?}", windows);
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

        tracing::debug!(" --> niri apps: {:#?}", apps);
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
                let _ = niri.spawn(vec!["gtk-launch".to_string(), app_id]).await;
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

    fn get_windows(&self, app_id: &str) -> Vec<DockWindow> {
        self.niri
            .windows
            .get()
            .iter()
            .filter(|(_, w)| w.app_id.get().as_deref() == Some(app_id))
            .map(|(id, w)| DockWindow {
                identifier: id.to_string(),
                title: w.title.get().unwrap_or_default(),
            })
            .collect()
    }

    fn focus_window(&self, identifier: &str) {
        let niri = self.niri.clone();
        let window_id: u64 = match identifier.parse() {
            Ok(id) => id,
            Err(e) => {
                tracing::debug!(
                    dock = "focus_window",
                    identifier = %identifier,
                    parse_error = ?e,
                    "Failed to parse window identifier"
                );
                return;
            }
        };
        tokio::spawn(async move {
            if window_id != 0 {
                let _ = niri.focus_window(window_id).await;
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

impl DockAdapter for DockAdapterRef {
    fn compute_running_apps(&self) -> Vec<DockAppData> {
        match self {
            DockAdapterRef::Niri(a) => a.compute_running_apps(),
            DockAdapterRef::Hyprland(a) => a.compute_running_apps(),
        }
    }

    fn focus_app(&self, app_id: &str) {
        match self {
            DockAdapterRef::Niri(a) => a.focus_app(app_id),
            DockAdapterRef::Hyprland(a) => a.focus_app(app_id),
        }
    }

    fn get_windows(&self, app_id: &str) -> Vec<DockWindow> {
        match self {
            DockAdapterRef::Niri(a) => a.get_windows(app_id),
            DockAdapterRef::Hyprland(a) => a.get_windows(app_id),
        }
    }

    fn focus_window(&self, identifier: &str) {
        match self {
            DockAdapterRef::Niri(a) => a.focus_window(identifier),
            DockAdapterRef::Hyprland(a) => a.focus_window(identifier),
        }
    }
}

impl Clone for DockAdapterRef {
    fn clone(&self) -> Self {
        match self {
            DockAdapterRef::Niri(niri) => {
                DockAdapterRef::Niri(NiriDockAdapter::new(niri.niri.clone()))
            }
            DockAdapterRef::Hyprland(hyprland) => DockAdapterRef::Hyprland(
                crate::shell::dock::adapter_hyprland::HyprlandDockAdapter::new(
                    hyprland.hyprland.clone(),
                ),
            ),
        }
    }
}
