use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use gtk::prelude::*;
use relm4::prelude::*;
use tracing::{debug, error, warn};
use wayle_config::schemas::{
    bar::BorderLocation,
    modules::{HyprlandWorkspacesConfig, Numbering, UrgentMode},
};
use wayle_hyprland::{Address, HyprlandService, WorkspaceAddress, WorkspaceId};
use wayle_widgets::prelude::BarSettings;

use super::{
    BLINK_INTERVAL, HyprlandWorkspaces,
    button::{ButtonBuildContext, WorkspaceButtonInput, build_button_init},
    filtering::{
        ActiveWorkspace, FilterContext, FilteredWorkspace, WorkspaceData,
        calculate_navigation_index, filter_workspaces, monitor_workspaces_sorted,
    },
    helpers::{
        addresses_in_workspace, compute_display_id, has_title_patterns, prune_stale_addresses,
        should_update_for_monitor, workspace_contains_urgent_address,
    },
    messages::WorkspacesCmd,
};

impl HyprlandWorkspaces {
    pub(super) fn is_vertical(&self) -> bool {
        self.settings.is_vertical.get()
    }

    pub(super) fn orientation(&self) -> gtk::Orientation {
        if self.is_vertical() {
            gtk::Orientation::Vertical
        } else {
            gtk::Orientation::Horizontal
        }
    }

    pub(super) fn spawn_load_workspace_rules(
        sender: &ComponentSender<Self>,
        hyprland: &Option<Arc<HyprlandService>>,
    ) {
        let Some(hyprland) = hyprland.clone() else {
            return;
        };

        sender.oneshot_command(async move {
            match hyprland.workspace_rules().await {
                Ok(rules) => {
                    let map = rules
                        .into_iter()
                        .filter_map(|rule| {
                            let id = rule.workspace_string.parse::<WorkspaceId>().ok()?;
                            let monitor = rule.monitor?;
                            if id > 0 { Some((id, monitor)) } else { None }
                        })
                        .collect();
                    WorkspacesCmd::WorkspaceRulesLoaded(map)
                }
                Err(err) => {
                    warn!(error = %err, "cannot load workspace rules");
                    WorkspacesCmd::WorkspaceRulesLoaded(HashMap::new())
                }
            }
        });
    }

    pub(super) fn workspace_monitor_name(&self, workspace: &ActiveWorkspace) -> Option<String> {
        if let Some(hyprland) = &self.hyprland {
            let live_workspaces = hyprland.workspaces.get();
            let live_monitor = live_workspaces
                .iter()
                .find(|live| live.address.get() == workspace.address)
                .map(|live| live.monitor.get());

            if live_monitor.is_some() {
                return live_monitor;
            }
        }

        let id = workspace.id?;

        self.workspace_monitor_rules.get(&id).cloned()
    }

    pub(super) fn display_id(
        &self,
        id: Option<WorkspaceId>,
        numbering: Numbering,
    ) -> Option<WorkspaceId> {
        let id = id?;
        let monitor_workspaces = self
            .settings
            .monitor_name
            .as_ref()
            .map(|monitor| monitor_workspaces_sorted(monitor, &self.workspace_monitor_rules))
            .unwrap_or_default();

        let display_id = compute_display_id(
            id,
            numbering,
            self.settings.monitor_name.as_deref(),
            &monitor_workspaces,
        );

        Some(display_id)
    }

    pub(super) fn initial_focused_monitor(
        hyprland: &Option<Arc<HyprlandService>>,
    ) -> Option<String> {
        let hyprland = hyprland.as_ref()?;
        hyprland
            .monitors
            .get()
            .into_iter()
            .find(|monitor| monitor.focused.get())
            .map(|monitor| monitor.name.get())
    }

    pub(super) fn initial_active_workspace(
        hyprland: &Option<Arc<HyprlandService>>,
        settings: &BarSettings,
        monitor_specific: bool,
    ) -> Option<ActiveWorkspace> {
        let hyprland = hyprland.as_ref()?;

        if monitor_specific && let Some(bar_monitor) = &settings.monitor_name {
            let monitors = hyprland.monitors.get();
            let bar_monitor_state = monitors
                .iter()
                .find(|monitor| &monitor.name.get() == bar_monitor);

            if let Some(monitor) = bar_monitor_state {
                let active_info = monitor.active_workspace.get();

                return Some(ActiveWorkspace::from(active_info));
            }
        }

        let runtime = tokio::runtime::Handle::current();
        let workspace = runtime.block_on(hyprland.active_workspace())?;

        Some(ActiveWorkspace {
            id: workspace.id.get(),
            address: workspace.address.get(),
        })
    }

    pub(super) fn should_apply_workspace_event(&self) -> bool {
        let Some(bar_monitor) = self.settings.monitor_name.as_ref() else {
            return true;
        };

        if let Some(focused_monitor) = self.focused_monitor.as_ref() {
            return focused_monitor == bar_monitor;
        }

        let Some(hyprland) = &self.hyprland else {
            return true;
        };

        hyprland
            .monitors
            .get()
            .into_iter()
            .find(|monitor| monitor.focused.get())
            .map(|monitor| monitor.name.get() == bar_monitor.as_str())
            .unwrap_or(true)
    }

    pub(super) fn should_apply_active_workspace_change(
        &self,
        workspace: &ActiveWorkspace,
        monitor_specific: bool,
    ) -> bool {
        if !monitor_specific {
            return true;
        }

        let Some(bar_monitor) = self.settings.monitor_name.as_ref() else {
            return self.should_apply_workspace_event();
        };

        match self.workspace_monitor_name(workspace) {
            Some(workspace_monitor) => workspace_monitor == *bar_monitor,
            None => self.should_apply_workspace_event(),
        }
    }

    pub(super) fn apply_active_workspace_change(&mut self, workspace: ActiveWorkspace) {
        let config = self.config.config();
        let ws_config = &config.modules.hyprland_workspaces;
        let monitor_specific = ws_config.monitor_specific.get();
        let has_min_workspace_count = ws_config.min_workspace_count.get() > 0;

        if !self.should_apply_active_workspace_change(&workspace, monitor_specific) {
            return;
        }

        self.clear_urgent_windows_for_workspace(&workspace.address);
        self.stop_blink_if_no_urgent();
        self.active_workspace = Some(workspace);
        self.sync_after_active_workspace_change(has_min_workspace_count);
    }

    /// Hyprland before 0.57 sends no address for named workspaces on monitor
    /// focus, so `workspace` is `None` there; the rebuild then reads the
    /// active workspace from the monitor list instead.
    pub(super) fn apply_monitor_focus(
        &mut self,
        monitor: String,
        workspace: Option<ActiveWorkspace>,
    ) {
        let config = self.config.config();
        let ws_config = &config.modules.hyprland_workspaces;
        let monitor_specific = ws_config.monitor_specific.get();
        let has_min_workspace_count = ws_config.min_workspace_count.get() > 0;
        let bar_monitor = self.settings.monitor_name.clone();

        self.focused_monitor = Some(monitor.clone());

        if !should_update_for_monitor(monitor_specific, bar_monitor.as_deref(), &monitor) {
            return;
        }

        let Some(workspace) = workspace else {
            self.rebuild_buttons();
            return;
        };

        self.clear_urgent_windows_for_workspace(&workspace.address);
        self.stop_blink_if_no_urgent();
        self.active_workspace = Some(workspace);
        self.sync_after_active_workspace_change(has_min_workspace_count);
    }

    pub(super) fn refresh_active_workspace(&mut self) {
        let Some(hyprland) = &self.hyprland else {
            return;
        };

        let config = self.config.config();
        let monitor_specific = config.modules.hyprland_workspaces.monitor_specific.get();

        let monitors = hyprland.monitors.get();

        let bar_monitor_state = self.settings.monitor_name.as_ref().and_then(|bar_monitor| {
            monitors
                .iter()
                .find(|monitor| monitor.name.get() == *bar_monitor)
        });

        if monitor_specific && let Some(monitor) = bar_monitor_state {
            let active_info = monitor.active_workspace.get();

            self.active_workspace = Some(ActiveWorkspace::from(active_info));
            return;
        }

        if let Some(monitor) = monitors.iter().find(|monitor| monitor.focused.get()) {
            let active_info = monitor.active_workspace.get();

            self.active_workspace = Some(ActiveWorkspace::from(active_info));
        }
    }

    fn is_active(&self, address: &WorkspaceAddress) -> bool {
        self.active_workspace
            .as_ref()
            .is_some_and(|active| active.address == *address)
    }

    pub(super) fn rebuild_buttons(&mut self) {
        self.refresh_active_workspace();

        debug!(
            bar_monitor = ?self.settings.monitor_name,
            active_workspace = ?self.active_workspace,
            "rebuild_buttons called"
        );

        let Some(hyprland) = &self.hyprland else {
            warn!(
                module = "hyprland-workspaces",
                "HyprlandService unavailable"
            );
            return;
        };

        let config = self.config.config();
        let ws_config = &config.modules.hyprland_workspaces;
        let is_vertical = self.is_vertical();

        self.update_border_classes(ws_config.border_show.get());

        let workspaces = self.filtered_workspaces(hyprland, ws_config);
        let clients = hyprland.clients.get();

        let numbering = ws_config.numbering.get();
        let per_icon_urgent = ws_config.app_icons_show.get()
            && ws_config.urgent_mode.get() == UrgentMode::Application;
        let urgent_show = ws_config.urgent_show.get();
        let button_inits: Vec<_> = workspaces
            .iter()
            .map(|workspace| {
                let is_urgent = self.blink_on
                    && urgent_show
                    && self.workspace_has_urgent_window(&workspace.address, hyprland);
                let urgent_addrs = if is_urgent && per_icon_urgent {
                    self.urgent_windows.clone()
                } else {
                    HashSet::new()
                };
                let ctx = ButtonBuildContext {
                    id: workspace.id,
                    address: &workspace.address,
                    kind: workspace.kind,
                    display_id: self.display_id(workspace.id, numbering),
                    name: &workspace.name,
                    windows: workspace.windows,
                    is_active: self.is_active(&workspace.address),
                    is_urgent,
                    is_vertical,
                };
                build_button_init(&ctx, ws_config, &clients, urgent_addrs)
            })
            .collect();

        let mut guard = self.buttons.guard();
        guard.clear();
        for init in button_inits {
            guard.push_back(init);
        }
    }

    pub(super) fn filtered_workspaces(
        &self,
        hyprland: &Arc<HyprlandService>,
        config: &HyprlandWorkspacesConfig,
    ) -> Vec<FilteredWorkspace> {
        let all_workspaces = hyprland.workspaces.get();
        let ignore_patterns = config.workspace_ignore.get();

        let workspace_data: Vec<WorkspaceData> = all_workspaces
            .iter()
            .map(|workspace| WorkspaceData {
                id: workspace.id.get(),
                address: workspace.address.get(),
                kind: workspace.kind.get(),
                name: workspace.name.get(),
                windows: workspace.windows.get(),
                monitor: workspace.monitor.get(),
            })
            .collect();

        let ctx = FilterContext {
            show_special: config.show_special.get(),
            monitor_specific: config.monitor_specific.get(),
            min_workspace_count: usize::from(config.min_workspace_count.get()),
            active_workspace: self.active_workspace.clone(),
            bar_monitor: self.settings.monitor_name.as_deref(),
            ignore_patterns: &ignore_patterns,
            workspace_monitor_rules: &self.workspace_monitor_rules,
        };

        filter_workspaces(&workspace_data, &ctx)
    }

    pub(super) fn update_active_state(&mut self) {
        let Some(hyprland) = &self.hyprland else {
            return;
        };

        let config = self.config.config();
        let ws_config = &config.modules.hyprland_workspaces;
        let per_icon_urgent = ws_config.app_icons_show.get()
            && ws_config.urgent_mode.get() == UrgentMode::Application;
        let urgent_show = ws_config.urgent_show.get();

        for button_index in 0..self.buttons.len() {
            let Some(button) = self.buttons.get(button_index) else {
                continue;
            };

            let button_address = button.address().clone();
            let is_urgent = self.blink_on
                && urgent_show
                && self.workspace_has_urgent_window(&button_address, hyprland);

            let urgent_addrs = if is_urgent && per_icon_urgent {
                self.urgent_windows.clone()
            } else {
                HashSet::new()
            };

            self.buttons.send(
                button_index,
                WorkspaceButtonInput::UpdateState {
                    windows: self.window_count_for_workspace(&button_address, hyprland),
                    is_active: self.is_active(&button_address),
                    is_urgent,
                    urgent_addresses: urgent_addrs,
                },
            );
        }
    }

    pub(super) fn sync_after_active_workspace_change(&mut self, has_min_workspace_count: bool) {
        if has_min_workspace_count {
            self.rebuild_buttons();
            return;
        }

        self.update_active_state();
    }

    pub(super) fn update_app_icons_on_title_change(&mut self) {
        let config = self.config.config();
        let ws_config = &config.modules.hyprland_workspaces;

        if !ws_config.app_icons_show.get() {
            return;
        }

        if !has_title_patterns(&ws_config.app_icon_map.get()) {
            return;
        }

        self.rebuild_buttons();
    }

    pub(super) fn focus_workspace(&self, address: &WorkspaceAddress) {
        let Some(hyprland) = &self.hyprland else {
            return;
        };

        let config = self.config.config();
        let ws_config = &config.modules.hyprland_workspaces;
        let workspaces = self.filtered_workspaces(hyprland, ws_config);

        let Some(workspace) = workspaces
            .iter()
            .find(|workspace| workspace.address == *address)
        else {
            warn!(workspace = %address, "clicked workspace is no longer shown");
            return;
        };

        self.switch_to_workspace(workspace);
    }

    fn switch_to_workspace(&self, workspace: &FilteredWorkspace) {
        let Some(hyprland) = &self.hyprland else {
            return;
        };

        let Some(selector) = workspace.dispatch_selector() else {
            warn!(workspace = %workspace.address, "no resolvable selector, skipping dispatch");
            return;
        };

        let hyprland = hyprland.clone();
        tokio::spawn(async move {
            dispatch_workspace_focus(&hyprland, &selector).await;
        });
    }

    pub(super) fn navigate_workspace(&self, direction: i64) {
        let Some(hyprland) = &self.hyprland else {
            return;
        };

        let config = self.config.config();
        let ws_config = &config.modules.hyprland_workspaces;
        let workspaces = self.filtered_workspaces(hyprland, ws_config);

        if workspaces.is_empty() {
            return;
        }

        let current_idx = workspaces
            .iter()
            .position(|workspace| self.is_active(&workspace.address))
            .unwrap_or(0);

        let new_idx = calculate_navigation_index(current_idx, direction, workspaces.len());

        if let Some(workspace) = workspaces.get(new_idx) {
            self.switch_to_workspace(workspace);
        }
    }

    pub(super) fn start_blink_timer(&mut self, sender: &ComponentSender<Self>) {
        self.stop_blink_timer();
        self.blink_on = true;

        let token = tokio_util::sync::CancellationToken::new();
        let cancel = token.clone();
        self.blink_token = Some(token);

        sender.command(move |out, shutdown| async move {
            let shutdown_fut = shutdown.wait();
            tokio::pin!(shutdown_fut);

            let mut interval = tokio::time::interval(BLINK_INTERVAL);
            interval.tick().await;

            loop {
                tokio::select! {
                    () = &mut shutdown_fut => break,
                    () = cancel.cancelled() => break,
                    _ = interval.tick() => {
                        let _ = out.send(WorkspacesCmd::BlinkTick);
                    }
                }
            }
        });
    }

    pub(super) fn stop_blink_timer(&mut self) {
        if let Some(token) = self.blink_token.take() {
            token.cancel();
        }
        self.blink_on = false;
    }

    pub(super) fn stop_blink_if_no_urgent(&mut self) {
        if self.urgent_windows.is_empty() {
            self.stop_blink_timer();
        }
    }

    pub(super) fn workspace_has_urgent_window(
        &self,
        workspace: &WorkspaceAddress,
        hyprland: &Arc<HyprlandService>,
    ) -> bool {
        let client_workspaces = client_workspace_addresses(hyprland);

        workspace_contains_urgent_address(workspace, &self.urgent_windows, &client_workspaces)
    }

    pub(super) fn window_count_for_workspace(
        &self,
        workspace: &WorkspaceAddress,
        hyprland: &Arc<HyprlandService>,
    ) -> u16 {
        hyprland
            .workspaces
            .get()
            .iter()
            .find(|live| live.address.get() == *workspace)
            .map(|live| live.windows.get())
            .unwrap_or(0)
    }

    pub(super) fn clear_urgent_windows_for_workspace(&mut self, workspace: &WorkspaceAddress) {
        let Some(hyprland) = &self.hyprland else {
            return;
        };

        let client_workspaces = client_workspace_addresses(hyprland);
        let to_clear = addresses_in_workspace(workspace, &client_workspaces);

        for address in to_clear {
            self.urgent_windows.remove(&address);
        }
    }

    pub(super) fn prune_stale_urgent_windows(&mut self) {
        if self.urgent_windows.is_empty() {
            return;
        }

        let Some(hyprland) = &self.hyprland else {
            return;
        };

        let clients = hyprland.clients.get();
        let current_addresses: HashSet<Address> =
            clients.iter().map(|client| client.address.get()).collect();

        self.urgent_windows = prune_stale_addresses(&self.urgent_windows, &current_addresses);
    }

    pub(super) fn update_border_classes(&self, show_border: bool) {
        let container = self.buttons.widget();
        for location in [
            BorderLocation::Top,
            BorderLocation::Bottom,
            BorderLocation::Left,
            BorderLocation::Right,
            BorderLocation::All,
        ] {
            if let Some(class) = location.css_class() {
                container.remove_css_class(class);
            }
        }

        if show_border && let Some(class) = self.settings.border_location.get().css_class() {
            container.add_css_class(class);
        }
    }
}

fn client_workspace_addresses(hyprland: &HyprlandService) -> Vec<(Address, WorkspaceAddress)> {
    hyprland
        .clients
        .get()
        .iter()
        .map(|client| {
            let workspace = client.workspace.get();

            (client.address.get(), workspace.address)
        })
        .collect()
}

fn lua_string_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Tries the Lua dispatcher first (Hyprland 0.55+ Lua config), falling back
/// to the legacy `workspace <selector>` form. Both paths share the same
/// parser inside Hyprland.
async fn dispatch_workspace_focus(hyprland: &HyprlandService, selector: &str) {
    let lua_arg = if selector.starts_with("name:") {
        format!("\"{}\"", lua_string_escape(selector))
    } else {
        selector.to_owned()
    };
    let lua_cmd = format!("hl.dsp.focus({{workspace = {lua_arg}}})");

    match hyprland.dispatch(&lua_cmd).await {
        Ok(resp) if resp.starts_with("error:") || resp.contains("Invalid dispatcher") => {
            let legacy_cmd = format!("workspace {selector}");
            if let Err(err) = hyprland.dispatch(&legacy_cmd).await {
                error!(error = %err, workspace = selector, "cannot switch workspace");
            }
        }
        Err(err) => {
            error!(error = %err, workspace = selector, "cannot switch workspace");
        }
        _ => {}
    }
}
