//! Hyprland-specific dock adapter.

use std::sync::Arc;

use indexmap::IndexMap;
use wayle_hyprland::HyprlandService;

use super::adapter::DockAdapter;
use super::DockAppData;

pub struct HyprlandDockAdapter {
    hyprland: Arc<HyprlandService>,
}

impl HyprlandDockAdapter {
    pub fn new(hyprland: Arc<HyprlandService>) -> Self {
        Self { hyprland }
    }
    
    pub(crate) fn hyprland(&self) -> Arc<HyprlandService> {
        self.hyprland.clone()
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
}
