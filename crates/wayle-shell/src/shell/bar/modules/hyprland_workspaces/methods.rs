use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use gtk::{glib, prelude::*};
use relm4::prelude::*;
use tracing::{debug, error, warn};
use wayle_config::schemas::{
    bar::BorderLocation,
    modules::{HyprlandWorkspacesConfig, Numbering, UrgentMode},
};
use wayle_hyprland::{Address, HyprlandService, WorkspaceId};
use wayle_widgets::prelude::BarSettings;

use super::{
    BLINK_INTERVAL, HyprlandWorkspaces,
    button::{ButtonBuildContext, WorkspaceButtonInput, build_button_init},
    filtering::{
        FilterContext, FilteredWorkspace, WorkspaceData, calculate_navigation_index,
        filter_workspaces, monitor_workspaces_sorted,
    },
    helpers::{
        addresses_in_workspace, compute_display_id, has_title_patterns, prune_stale_addresses,
        workspace_contains_urgent_address,
    },
    messages::{WorkspacesCmd, WorkspacesMsg},
    preview::{WorkspacePreview, WorkspacePreviewInit, WorkspacePreviewOutput},
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
                Err(e) => {
                    warn!(error = %e, "cannot load workspace rules");
                    WorkspacesCmd::WorkspaceRulesLoaded(HashMap::new())
                }
            }
        });
    }

    pub(super) fn workspace_monitor_name(&self, id: WorkspaceId) -> Option<String> {
        if let Some(hyprland) = &self.hyprland
            && let Some(monitor_name) = hyprland
                .workspaces
                .get()
                .into_iter()
                .find(|ws| ws.id.get() == id)
                .map(|ws| ws.monitor.get())
        {
            return Some(monitor_name);
        }

        self.workspace_monitor_rules.get(&id).cloned()
    }

    pub(super) fn display_id(&self, id: WorkspaceId, numbering: Numbering) -> WorkspaceId {
        let monitor_workspaces = self
            .settings
            .monitor_name
            .as_ref()
            .map(|m| monitor_workspaces_sorted(m, &self.workspace_monitor_rules))
            .unwrap_or_default();
        compute_display_id(
            id,
            numbering,
            self.settings.monitor_name.as_deref(),
            &monitor_workspaces,
        )
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
    ) -> WorkspaceId {
        let Some(hyprland) = hyprland else {
            return 1;
        };

        if monitor_specific && let Some(bar_monitor) = &settings.monitor_name {
            let monitors = hyprland.monitors.get();
            if let Some(monitor) = monitors.iter().find(|m| &m.name.get() == bar_monitor) {
                return monitor.active_workspace.get().id;
            }
        }

        let runtime = tokio::runtime::Handle::current();
        match runtime.block_on(hyprland.active_workspace()) {
            Some(ws) => ws.id.get(),
            None => 1,
        }
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
        workspace_id: WorkspaceId,
        monitor_specific: bool,
    ) -> bool {
        if !monitor_specific {
            return true;
        }

        let Some(bar_monitor) = self.settings.monitor_name.as_ref() else {
            return self.should_apply_workspace_event();
        };

        match self.workspace_monitor_name(workspace_id) {
            Some(ws_monitor) => ws_monitor == *bar_monitor,
            None => self.should_apply_workspace_event(),
        }
    }

    pub(super) fn rebuild_buttons(&mut self) {
        debug!(
            bar_monitor = ?self.settings.monitor_name,
            active_workspace = self.active_workspace_id,
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
            .map(|ws| {
                let is_urgent = self.blink_on
                    && urgent_show
                    && self.workspace_has_urgent_window(ws.id, hyprland);
                let urgent_addrs = if is_urgent && per_icon_urgent {
                    self.urgent_windows.clone()
                } else {
                    HashSet::new()
                };
                let ctx = ButtonBuildContext {
                    id: ws.id,
                    display_id: self.display_id(ws.id, numbering),
                    name: &ws.name,
                    windows: ws.windows,
                    is_active: ws.id == self.active_workspace_id,
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
            .map(|ws| WorkspaceData {
                id: ws.id.get(),
                name: ws.name.get(),
                windows: ws.windows.get(),
                monitor: ws.monitor.get(),
            })
            .collect();

        let ctx = FilterContext {
            show_special: config.show_special.get(),
            monitor_specific: config.monitor_specific.get(),
            min_workspace_count: usize::from(config.min_workspace_count.get()),
            active_workspace_id: self.active_workspace_id,
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

        for idx in 0..self.buttons.len() {
            let Some(button) = self.buttons.get(idx) else {
                continue;
            };
            let button_id = button.id();
            let is_urgent = self.blink_on
                && urgent_show
                && self.workspace_has_urgent_window(button_id, hyprland);

            let urgent_addrs = if is_urgent && per_icon_urgent {
                self.urgent_windows.clone()
            } else {
                HashSet::new()
            };

            self.buttons.send(
                idx,
                WorkspaceButtonInput::UpdateState {
                    windows: self.window_count_for_workspace(button_id, hyprland),
                    is_active: button_id == self.active_workspace_id,
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

    pub(super) fn switch_to_workspace(&self, id: WorkspaceId) {
        let Some(hyprland) = &self.hyprland else {
            return;
        };

        let hyprland = hyprland.clone();
        tokio::spawn(async move {
            let command = format!("workspace {}", id);
            if let Err(e) = hyprland.dispatch(&command).await {
                error!(error = %e, workspace = id, "Failed to switch workspace");
            }
        });
    }

    pub(super) fn focus_window_by_address(&self, address: &str) {
        let Some(hyprland) = &self.hyprland else {
            return;
        };

        let hyprland = hyprland.clone();
        let command = format!("focuswindow address:0x{address}");
        tokio::spawn(async move {
            if let Err(e) = hyprland.dispatch(&command).await {
                error!(error = %e, "Failed to focus window");
            }
        });
    }

    // --- Preview popover lifecycle ---

    pub(super) fn cancel_preview_close_timer(&mut self) {
        if let Some(id) = self.preview_close_timer.take() {
            id.remove();
        }
    }

    pub(super) fn cancel_preview_dwell_timer(&mut self) {
        if let Some(id) = self.preview_dwell_timer.take() {
            id.remove();
        }
    }

    pub(super) fn start_preview_close_timer(&mut self, sender: &ComponentSender<Self>) {
        self.cancel_preview_close_timer();
        let config = self.config.config();
        let delay = config.modules.hyprland_workspaces.preview_close_delay.get();
        let timer_sender = sender.input_sender().clone();
        let id = glib::timeout_add_local_once(
            std::time::Duration::from_millis(u64::from(delay)),
            move || {
                timer_sender.emit(WorkspacesMsg::PreviewCloseTimerFired);
            },
        );
        self.preview_close_timer = Some(id);
    }

    pub(super) fn start_preview_dwell_timer(
        &mut self,
        sender: &ComponentSender<Self>,
        ws_id: WorkspaceId,
    ) {
        self.cancel_preview_dwell_timer();
        let config = self.config.config();
        let delay = config.modules.hyprland_workspaces.preview_open_delay.get();
        let timer_sender = sender.input_sender().clone();
        let id = glib::timeout_add_local_once(
            std::time::Duration::from_millis(u64::from(delay)),
            move || {
                timer_sender.emit(WorkspacesMsg::PreviewDwellFired(ws_id));
            },
        );
        self.preview_dwell_timer = Some(id);
    }

    #[allow(clippy::too_many_lines)]
    pub(super) fn show_preview_for_workspace(&mut self, ws_id: WorkspaceId) {
        if self.preview.is_none() {
            return;
        }

        // Already showing this workspace — just cancel close timer.
        if self.current_preview_ws == Some(ws_id) {
            self.cancel_preview_close_timer();
            return;
        }

        let is_visible = self
            .preview_dropdown
            .as_ref()
            .is_some_and(|d| d.is_visible());

        // Stop old streaming if switching workspaces.
        if is_visible
            && let Some(ref preview) = self.preview
        {
            preview.emit(super::preview::WorkspacePreviewMsg::Hide);
        }

        // Send Show to the preview component.
        if let Some(ref preview) = self.preview {
            let config = self.config.config();
            let ws_config = &config.modules.hyprland_workspaces;
            preview.emit(super::preview::WorkspacePreviewMsg::Show {
                ws_id,
                hyprland: self.hyprland.clone(),
                settings: Box::new(self.settings.clone()),
                preview_width: ws_config.preview_width.get(),
            });
        }

        // Show the popover via DropdownInstance (handles parenting, position,
        // margins, styling — same as all other dropdowns).
        if !is_visible
            && let Some(ref dropdown) = self.preview_dropdown
        {
            let style = self.preview_dropdown_style();
            let container = self.buttons.widget();
            dropdown.show_anchored_to(container, style);
        }

        self.current_preview_ws = Some(ws_id);
    }

    fn preview_dropdown_style(&self) -> crate::shell::bar::dropdowns::DropdownStyle {
        // Disable autohide — the preview uses hover-based close timers, not
        // focus-based autohide. With autohide on, the popover closes
        // immediately when hover event processing changes focus.
        crate::shell::bar::dropdowns::DropdownStyle::from_config(&self.config, false)
    }

    pub(super) fn reinitialize_preview(&mut self, sender: &ComponentSender<Self>) {
        let config = self.config.config();
        let ws_config = &config.modules.hyprland_workspaces;
        let should_show = ws_config.preview_show.get();
        let is_active = self.preview.is_some();

        // Only recreate when the enabled state actually toggled.
        if should_show == is_active {
            return;
        }

        if should_show {
            let preview_ctrl = WorkspacePreview::builder()
                .launch(WorkspacePreviewInit {})
                .forward(sender.input_sender(), |output| match output {
                    WorkspacePreviewOutput::FocusWindow(addr) => {
                        WorkspacesMsg::FocusWindow(addr)
                    }
                    WorkspacePreviewOutput::Dismiss => {
                        WorkspacesMsg::WorkspacePreviewDismiss
                    }
                });

            let wrapper = gtk::Box::new(gtk::Orientation::Vertical, 0);
            wrapper.add_css_class("dropdown");
            wrapper.add_css_class("ws-preview-dropdown");
            wrapper.append(preview_ctrl.widget());

            let popover_motion = gtk::EventControllerMotion::new();
            let enter_sender = sender.input_sender().clone();
            popover_motion.connect_enter(move |_, _, _| {
                enter_sender.emit(WorkspacesMsg::PopoverEnter);
            });
            let leave_sender = sender.input_sender().clone();
            popover_motion.connect_leave(move |_| {
                leave_sender.emit(WorkspacesMsg::PopoverLeave);
            });
            wrapper.add_controller(popover_motion);

            let popover = gtk::Popover::new();
            popover.set_child(Some(&wrapper));
            popover.set_has_arrow(false);
            popover.add_css_class("dropdown");

            let dropdown = std::rc::Rc::new(
                crate::shell::bar::dropdowns::DropdownInstance::new(popover, Box::new(())),
            );

            let hide_sender = sender.input_sender().clone();
            dropdown.popover().connect_closed(move |_| {
                hide_sender.emit(WorkspacesMsg::WorkspacePreviewClosed);
            });

            self.preview = Some(preview_ctrl);
            self.preview_dropdown = Some(dropdown);
        } else {
            self.preview_dropdown = None;
            self.preview = None;
        }
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
            .position(|ws| ws.id == self.active_workspace_id)
            .unwrap_or(0);

        let new_idx = calculate_navigation_index(current_idx, direction, workspaces.len());

        if let Some(ws) = workspaces.get(new_idx) {
            self.switch_to_workspace(ws.id);
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
        workspace_id: WorkspaceId,
        hyprland: &Arc<HyprlandService>,
    ) -> bool {
        let clients = hyprland.clients.get();
        let client_workspaces: Vec<_> = clients
            .iter()
            .map(|c| (c.address.get(), c.workspace.get().id))
            .collect();
        workspace_contains_urgent_address(workspace_id, &self.urgent_windows, &client_workspaces)
    }

    pub(super) fn window_count_for_workspace(
        &self,
        workspace_id: WorkspaceId,
        hyprland: &Arc<HyprlandService>,
    ) -> u16 {
        hyprland
            .workspaces
            .get()
            .iter()
            .find(|ws| ws.id.get() == workspace_id)
            .map(|ws| ws.windows.get())
            .unwrap_or(0)
    }

    pub(super) fn clear_urgent_windows_for_workspace(&mut self, workspace_id: WorkspaceId) {
        let Some(hyprland) = &self.hyprland else {
            return;
        };

        let clients = hyprland.clients.get();
        let client_workspaces: Vec<_> = clients
            .iter()
            .map(|c| (c.address.get(), c.workspace.get().id))
            .collect();
        let to_clear = addresses_in_workspace(workspace_id, &client_workspaces);
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
