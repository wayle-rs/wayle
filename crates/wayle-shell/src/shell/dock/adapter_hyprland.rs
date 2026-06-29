//! Hyprland-specific dock adapter.

use std::sync::Arc;

use indexmap::IndexMap;
use wayle_hyprland::HyprlandService;

use super::{
    DockAppData,
    adapter::{DockAdapter, DockWindow},
};

pub struct HyprlandDockAdapter {
    pub(crate) hyprland: Arc<HyprlandService>,
}

impl HyprlandDockAdapter {
    pub fn new(hyprland: Arc<HyprlandService>) -> Self {
        Self { hyprland }
    }
}

impl DockAdapter for HyprlandDockAdapter {
    fn compute_running_apps(&self) -> Vec<DockAppData> {
        let clients = self.hyprland.clients.get();

        let mut groups: IndexMap<String, (u32, bool)> = IndexMap::new();
        for client in clients.iter() {
            let class = client.class.get();
            if class.is_empty() {
                continue;
            }
            let entry = groups.entry(class).or_insert((0, false));
            entry.0 += 1;
        }

        let apps: Vec<DockAppData> = groups
            .into_iter()
            .map(|(app_id, (window_count, is_active))| DockAppData {
                app_id,
                is_active,
                window_count,
            })
            .collect();

        tracing::debug!(app_count = apps.len(), "Computed Hyprland running apps");
        apps
    }

    fn focus_app(&self, app_id: &str) {
        let hyprland = self.hyprland.clone();
        let app_id = app_id.to_string();
        tokio::spawn(async move {
            let class_windows: Vec<String> = hyprland
                .clients
                .get()
                .iter()
                .filter(|c| c.class.get() == app_id)
                .map(|c| c.address.get().to_string())
                .collect();

            if class_windows.is_empty() {
                let _ = hyprland
                    .dispatch(&format!("exec,gtk-launch {}", app_id))
                    .await;
                return;
            }

            let focused = hyprland.active_window().await;
            let is_focused = focused
                .as_ref()
                .map(|f| f.address.get().to_string())
                .is_some_and(|addr| class_windows.contains(&addr));

            if !is_focused {
                let _ = hyprland
                    .dispatch(&format!("focuswindow,class:^{}$", app_id))
                    .await;
            }
        });
    }

    fn get_windows(&self, app_id: &str) -> Vec<DockWindow> {
        self.hyprland
            .clients
            .get()
            .iter()
            .filter(|c| c.class.get() == app_id)
            .map(|c| DockWindow {
                identifier: c.address.get().to_string(),
                title: c.title.get().to_string(),
            })
            .collect()
    }

    fn focus_window(&self, identifier: &str) {
        let hyprland = self.hyprland.clone();
        let identifier = identifier.to_string();
        tokio::spawn(async move {
            let _ = hyprland
                .dispatch(&format!("focuswindow,address:{}", identifier))
                .await;
        });
    }
}
