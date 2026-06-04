//! Dock service for managing dock items from compositor window data.

use std::sync::Arc;

use indexmap::IndexMap;

use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::debug;
use wayle_core::Property;
use wayle_hyprland::{HyprlandEvent, HyprlandService};
use wayle_niri::{Event as NiriEvent, NiriService};

/// Running application information for dock items.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DockApp {
    /// Application ID (e.g., "org.gnome.Settings").
    pub app_id: String,
    /// Whether the app is currently focused/active.
    pub is_active: bool,
    /// Number of windows for this application.
    pub window_count: u32,
}

/// State for dock items (pinned + running).
pub(crate) struct DockServiceState {
    /// Currently pinned app IDs.
    pub pinned_apps: Property<Vec<String>>,
    /// Currently running applications.
    pub running_apps: Property<Vec<DockApp>>,
}

impl Clone for DockServiceState {
    fn clone(&self) -> Self {
        Self {
            pinned_apps: self.pinned_apps.clone(),
            running_apps: self.running_apps.clone(),
        }
    }
}

/// Service that manages dock items from compositor window data.
#[derive(Clone)]
pub struct DockService {
    state: DockServiceState,
    hyprland: Option<Arc<HyprlandService>>,
    niri: Option<Arc<NiriService>>,
}

impl DockService {
    pub(crate) fn new(
        pinned_apps: Vec<String>,
        hyprland: Option<Arc<HyprlandService>>,
        niri: Option<Arc<NiriService>>,
    ) -> Self {
        Self {
            state: DockServiceState {
                pinned_apps: Property::new(pinned_apps),
                running_apps: Property::new(Vec::new()),
            },
            hyprland,
            niri,
        }
    }

    /// Returns combined pinned + running dock items.
    pub(crate) fn get_items(&self) -> Vec<DockItemData> {
        let pinned = self.state.pinned_apps.get();
        let running = self.state.running_apps.get();

        let mut items: Vec<DockItemData> = pinned
            .iter()
            .map(|app_id| {
                let is_running = running.iter().any(|a| a.app_id == *app_id);
                let is_active = running
                    .iter()
                    .find(|a| a.app_id == *app_id)
                    .map_or(false, |a| a.is_active);
                DockItemData {
                    app_id: app_id.clone(),
                    is_pinned: true,
                    is_running,
                    is_active,
                    window_count: running
                        .iter()
                        .find(|a| a.app_id == *app_id)
                        .map_or(0, |a| a.window_count),
                }
            })
            .collect();

        let show_running = true;
        if show_running {
            let pinned_set: std::collections::HashSet<&str> =
                pinned.iter().map(|s| s.as_str()).collect();

            for app in running.iter() {
                if !pinned_set.contains(app.app_id.as_str()) {
                    items.push(DockItemData {
                        app_id: app.app_id.clone(),
                        is_pinned: false,
                        is_running: true,
                        is_active: app.is_active,
                        window_count: app.window_count,
                    });
                }
            }
        }

        items
    }

    /// Reactive state accessor.
    pub(crate) fn state(&self) -> DockServiceState {
        self.state.clone()
    }

    /// Updates running apps from compositor events, preserving existing order.
    pub(crate) fn update_running_apps(&self, new_apps: Vec<DockApp>) {
        let old_apps = self.state.running_apps.get();

        let mut ordered: Vec<DockApp> = Vec::new();
        let mut app_map: std::collections::HashMap<String, DockApp> =
            new_apps.into_iter().map(|a| (a.app_id.clone(), a)).collect();

        for app in old_apps.iter() {
            if let Some(updated) = app_map.remove(&app.app_id) {
                ordered.push(updated);
            }
        }

        ordered.extend(app_map.into_values());

        self.state.running_apps.set(ordered);
    }

    /// Spawns async tasks to subscribe to compositor events and update running_apps.
    pub(crate) fn spawn_event_watcher(&self, tx: mpsc::UnboundedSender<()>) {
        let niri = self.niri.clone();
        let hyprland = self.hyprland.clone();
        let service = self.clone();

        let mut handles = Vec::new();

        if let Some(niri) = niri {
            let niri_handle = tokio::spawn(Self::watch_niri(niri, tx.clone(), service.clone()));
            handles.push(niri_handle);
        }

        if let Some(hyprland) = hyprland {
            let hyprland_handle =
                tokio::spawn(Self::watch_hyprland(hyprland, tx.clone(), service.clone()));
            handles.push(hyprland_handle);
        }

        if !handles.is_empty() {
            tokio::spawn(async move {
                for handle in handles {
                    let _ = handle.await;
                }
            });
        }
    }

    async fn watch_niri(
        niri: Arc<NiriService>,
        tx: mpsc::UnboundedSender<()>,
        service: DockService,
    ) {
        let mut events = niri.events();
        let mut windows_changed = niri.windows.watch();
        let cancellation = CancellationToken::new();
        let shutdown = cancellation.cancelled();
        tokio::pin!(shutdown);

        if let Some(_value) = windows_changed.next().await {
            let apps = Self::compute_niri_apps(&niri);
            service.update_running_apps(apps);
            let _ = tx.send(());
        }

        loop {
            tokio::select! {
                _ = &mut shutdown => break,
                event = events.next() => {
                    if let Some(event) = event {
                        if Self::niri_event_needs_update(&event) {
                            let apps = Self::compute_niri_apps(&niri);
                            service.update_running_apps(apps);
                            let _ = tx.send(());
                        }
                    } else {
                        break;
                    }
                }
                Some(_value) = windows_changed.next() => {
                    let apps = Self::compute_niri_apps(&niri);
                    service.update_running_apps(apps);
                    let _ = tx.send(());
                }
            }
        }

        debug!("Niri dock watcher stopped");
    }

    async fn watch_hyprland(
        hyprland: Arc<HyprlandService>,
        tx: mpsc::UnboundedSender<()>,
        service: DockService,
    ) {
        let mut events = hyprland.events();
        let mut clients_changed = hyprland.clients.watch();
        let cancellation = CancellationToken::new();
        let shutdown = cancellation.cancelled();
        tokio::pin!(shutdown);

        if let Some(_value) = clients_changed.next().await {
            let apps = Self::compute_hyprland_apps(&hyprland).await;
            service.update_running_apps(apps);
            let _ = tx.send(());
        }

        loop {
            tokio::select! {
                _ = &mut shutdown => break,
                event = events.next() => {
                    if let Some(event) = event {
                        if Self::hyprland_event_needs_update(&event) {
                            let apps = Self::compute_hyprland_apps(&hyprland).await;
                            service.update_running_apps(apps);
                            let _ = tx.send(());
                        }
                    } else {
                        break;
                    }
                }
                Some(_value) = clients_changed.next() => {
                    let apps = Self::compute_hyprland_apps(&hyprland).await;
                    service.update_running_apps(apps);
                    let _ = tx.send(());
                }
            }
        }

        debug!("Hyprland dock watcher stopped");
    }

  fn compute_niri_apps(niri: &NiriService) -> Vec<DockApp> {
        let windows = niri.windows.get();
        let focused_id = niri.focused_window_id.get();

        let mut groups: IndexMap<String, (u32, bool)> = IndexMap::new();
        for window in windows.values() {
            if let Some(app_id) = window.app_id.get() {
                if app_id.is_empty() {
                    continue;
                }
                let entry = groups.entry(app_id).or_insert((0, false));
                entry.0 += 1;
            }
        }

        if let Some(focused_id) = focused_id {
            for window in windows.values() {
                if window.id.get() == focused_id {
                    if let Some(app_id) = window.app_id.get() {
                        if !app_id.is_empty() {
                            if let Some(entry) = groups.get_mut(&app_id) {
                                entry.1 = true;
                            }
                        }
                    }
                    break;
                }
            }
        }

        let apps: Vec<DockApp> = groups
            .into_iter()
            .map(|(app_id, (window_count, is_active))| DockApp {
                app_id,
                is_active,
                window_count,
            })
            .collect();

        debug!(app_count = apps.len(), "Computed Niri running apps");
        apps
    }

    async fn compute_hyprland_apps(hyprland: &HyprlandService) -> Vec<DockApp> {
        let clients = hyprland.clients.get();
        let focused_address = hyprland.active_window().await;

        let mut groups: IndexMap<String, (u32, bool)> = IndexMap::new();
        for client in clients.iter() {
            let class = client.class.get();
            if class.is_empty() {
                continue;
            }
            let entry = groups.entry(class).or_insert((0, false));
            entry.0 += 1;
        }

        if let Some(focused) = focused_address {
            let class = focused.class.get();
            if !class.is_empty() {
                if let Some(entry) = groups.get_mut(&class) {
                    entry.1 = true;
                }
            }
        }

        let apps: Vec<DockApp> = groups
            .into_iter()
            .map(|(app_id, (window_count, is_active))| DockApp {
                app_id,
                is_active,
                window_count,
            })
            .collect();

        debug!(app_count = apps.len(), "Computed Hyprland running apps");
        apps
    }

    fn niri_event_needs_update(event: &NiriEvent) -> bool {
        matches!(
            event,
            NiriEvent::WindowsChanged { .. }
                | NiriEvent::WindowOpenedOrChanged { .. }
                | NiriEvent::WindowClosed { .. }
                | NiriEvent::WindowFocusChanged { .. }
                | NiriEvent::WindowLayoutsChanged { .. }
        )
    }

    fn hyprland_event_needs_update(event: &HyprlandEvent) -> bool {
        matches!(
            event,
            HyprlandEvent::OpenWindow { .. }
                | HyprlandEvent::CloseWindow { .. }
                | HyprlandEvent::MoveWindow { .. }
                | HyprlandEvent::MoveWindowV2 { .. }
                | HyprlandEvent::ActiveWindow { .. }
                | HyprlandEvent::ActiveWindowV2 { .. }
                | HyprlandEvent::Minimized { .. }
        )
    }
}

/// Data for a single dock item (pinned or running).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DockItemData {
    /// Application ID.
    pub app_id: String,
    /// Whether this is a pinned item.
    pub is_pinned: bool,
    /// Whether the app is currently running.
    pub is_running: bool,
    /// Whether the app is currently active/focused.
    pub is_active: bool,
    /// Number of windows for this app.
    pub window_count: u32,
}
