//! Hyprland workspace buttons for the bar.
//!
//! High level flow tldr:
//!   Workspace switch -> clear urgency, rebuild or update state
//!   Window open/close/move -> full rebuild (icons change)
//!   Urgent -> addr into urgent_windows, start blink timer,
//!     update_active_state sends UpdateState w/ urgent_addresses.
//!     urgent-mode=application -> button matches addrs, blinks individual icons.
//!     urgent-mode=workspace (default) -> empty addrs, whole workspace blinks.
//!   BlinkTick -> flip blink_on, update_active_state again (the pulse)
//!   Switch to urgent ws -> clear addrs, stop timer if set empty

mod button;
mod factory;
mod filtering;
mod helpers;
mod messages;
mod methods;
mod preview;
mod styling;
mod watchers;

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::Arc,
    time::Duration,
};

use gtk::prelude::*;
use relm4::{factory::FactoryVecDeque, prelude::*};
use tokio_util::sync::CancellationToken;
use tracing::debug;
use wayle_config::ConfigService;
use wayle_hyprland::{Address, HyprlandService, WorkspaceId};
use wayle_widgets::{prelude::BarSettings, utils::force_window_resize};

use self::{
    button::{WorkspaceButton, WorkspaceButtonOutput},
    helpers::should_update_for_monitor,
    preview::{WorkspacePreview, WorkspacePreviewInit, WorkspacePreviewOutput},
};
use crate::shell::bar::dropdowns::DropdownInstance;
pub(crate) use self::{
    factory::Factory,
    messages::{WorkspacesCmd, WorkspacesInit, WorkspacesMsg},
};

const BLINK_INTERVAL: Duration = Duration::from_millis(500);

pub(crate) struct HyprlandWorkspaces {
    hyprland: Option<Arc<HyprlandService>>,
    config: Arc<ConfigService>,
    settings: BarSettings,
    active_workspace_id: WorkspaceId,
    focused_monitor: Option<String>,
    workspace_monitor_rules: HashMap<WorkspaceId, String>,
    urgent_windows: HashSet<Address>,
    blink_on: bool,
    blink_token: Option<CancellationToken>,
    css_provider: gtk::CssProvider,
    buttons: FactoryVecDeque<WorkspaceButton>,
    preview: Option<Controller<WorkspacePreview>>,
    preview_dropdown: Option<Rc<DropdownInstance>>,
    hovered_button_ws: Option<WorkspaceId>,
    pointer_in_popover: bool,
    current_preview_ws: Option<WorkspaceId>,
    preview_close_timer: Option<glib::SourceId>,
    preview_dwell_timer: Option<glib::SourceId>,
}

#[relm4::component(pub(crate))]
impl Component for HyprlandWorkspaces {
    type Init = WorkspacesInit;
    type Input = WorkspacesMsg;
    type Output = ();
    type CommandOutput = WorkspacesCmd;

    view! {
        gtk::Box {
            add_css_class: "workspaces",
            #[watch]
            set_orientation: model.orientation(),
            #[watch]
            set_hexpand: model.is_vertical(),
            #[watch]
            set_vexpand: !model.is_vertical(),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = init.config.config();
        let workspaces_config = &config.modules.hyprland_workspaces;
        let monitor_specific = workspaces_config.monitor_specific.get();
        let theme_provider = config.styling.theme_provider.clone();

        let active_id =
            Self::initial_active_workspace(&init.hyprland, &init.settings, monitor_specific);
        let focused_monitor = Self::initial_focused_monitor(&init.hyprland);
        let bar_scale = config.bar.scale.clone();

        Self::spawn_load_workspace_rules(&sender, &init.hyprland);

        watchers::spawn_watchers(
            &sender,
            workspaces_config,
            &init.hyprland,
            theme_provider,
            bar_scale,
            &init.settings,
        );

        let css_provider = gtk::CssProvider::new();
        gtk::style_context_add_provider_for_display(
            &root.display(),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER + 1,
        );

        let buttons = FactoryVecDeque::builder().launch(root.clone()).forward(
            sender.input_sender(),
            |output| match output {
                WorkspaceButtonOutput::Clicked(id) => WorkspacesMsg::WorkspaceClicked(id),
                WorkspaceButtonOutput::ScrollUp => WorkspacesMsg::ScrollUp,
                WorkspaceButtonOutput::ScrollDown => WorkspacesMsg::ScrollDown,
                WorkspaceButtonOutput::PreviewHoverEnter(id) => {
                    WorkspacesMsg::ButtonHoverEnter(id)
                }
                WorkspaceButtonOutput::PreviewHoverLeave(id) => {
                    WorkspacesMsg::ButtonHoverLeave(id)
                }
                WorkspaceButtonOutput::PreviewRequest(id) => {
                    WorkspacesMsg::WorkspacePreviewRequest(id)
                }
            },
        );

        // Initialize preview popup if enabled.
        let (preview, preview_dropdown) = if workspaces_config.preview_show.get() {
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

            // Wrap the preview in a DropdownInstance — same infrastructure
            // used by all other dropdowns (audio, network, etc.).
            let wrapper = gtk::Box::new(gtk::Orientation::Vertical, 0);
            wrapper.add_css_class("dropdown");
            wrapper.add_css_class("ws-preview-dropdown");
            wrapper.append(preview_ctrl.widget());

            let popover = gtk::Popover::new();
            popover.set_child(Some(&wrapper));
            popover.set_has_arrow(false);
            popover.add_css_class("dropdown");

            let dropdown = Rc::new(DropdownInstance::new(popover, Box::new(())));

            // Attach hover tracking to the content wrapper.
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

            // When the popover closes, stop streaming.
            let hide_sender = sender.input_sender().clone();
            dropdown.popover().connect_closed(move |_| {
                hide_sender.emit(WorkspacesMsg::WorkspacePreviewClosed);
            });

            (Some(preview_ctrl), Some(dropdown))
        } else {
            (None, None)
        };

        let mut model = Self {
            hyprland: init.hyprland,
            config: init.config,
            settings: init.settings,
            active_workspace_id: active_id,
            focused_monitor,
            workspace_monitor_rules: HashMap::new(),
            urgent_windows: HashSet::new(),
            blink_on: false,
            blink_token: None,
            css_provider,
            buttons,
            preview,
            preview_dropdown,
            hovered_button_ws: None,
            pointer_in_popover: false,
            current_preview_ws: None,
            preview_close_timer: None,
            preview_dwell_timer: None,
        };

        styling::apply_styling(&model.css_provider, &model.config, &model.settings);
        model.rebuild_buttons();

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            WorkspacesMsg::WorkspaceClicked(id) => {
                self.switch_to_workspace(id);
            }
            WorkspacesMsg::ScrollUp => {
                self.navigate_workspace(-1);
            }
            WorkspacesMsg::ScrollDown => {
                self.navigate_workspace(1);
            }

            // --- Preview lifecycle ---
            WorkspacesMsg::ButtonHoverEnter(id) => {
                self.hovered_button_ws = Some(id);
                self.cancel_preview_close_timer();
                self.cancel_preview_dwell_timer();

                let is_visible = self.preview_dropdown.as_ref().is_some_and(|d| d.is_visible());

                if self.current_preview_ws == Some(id) && is_visible {
                    // Already showing this workspace — nothing to do.
                } else if is_visible {
                    // Popover open for a different workspace — switch immediately.
                    self.show_preview_for_workspace(id);
                } else {
                    // Not visible — start dwell timer.
                    self.start_preview_dwell_timer(&sender, id);
                }
            }
            WorkspacesMsg::ButtonHoverLeave(id) => {
                if self.hovered_button_ws != Some(id) {
                    return;
                }
                self.hovered_button_ws = None;
                self.cancel_preview_dwell_timer();

                let is_visible = self.preview_dropdown.as_ref().is_some_and(|d| d.is_visible());
                if is_visible && !self.pointer_in_popover {
                    self.start_preview_close_timer(&sender);
                }
            }
            WorkspacesMsg::PopoverEnter => {
                self.pointer_in_popover = true;
                self.cancel_preview_close_timer();
            }
            WorkspacesMsg::PopoverLeave => {
                self.pointer_in_popover = false;
                if self.hovered_button_ws.is_none() {
                    self.start_preview_close_timer(&sender);
                }
            }
            WorkspacesMsg::PreviewDwellFired(id) => {
                self.preview_dwell_timer = None;
                if self.hovered_button_ws != Some(id) {
                    return;
                }
                self.show_preview_for_workspace(id);
            }
            WorkspacesMsg::PreviewCloseTimerFired => {
                self.preview_close_timer = None;
                if self.pointer_in_popover || self.hovered_button_ws.is_some() {
                    return;
                }
                if let Some(ref dropdown) = self.preview_dropdown
                    && dropdown.is_visible()
                {
                    dropdown.popdown();
                }
            }
            WorkspacesMsg::WorkspacePreviewRequest(id) => {
                // Immediate show (right-click path).
                self.cancel_preview_dwell_timer();
                self.cancel_preview_close_timer();
                self.show_preview_for_workspace(id);
            }
            WorkspacesMsg::WorkspacePreviewClosed => {
                self.cancel_preview_close_timer();
                self.cancel_preview_dwell_timer();
                if let Some(ref preview) = self.preview {
                    preview.emit(preview::WorkspacePreviewMsg::Hide);
                }
                self.current_preview_ws = None;
                self.pointer_in_popover = false;
            }
            WorkspacesMsg::WorkspacePreviewDismiss => {
                // Component requested close (window clicked).
                self.cancel_preview_close_timer();
                self.cancel_preview_dwell_timer();
                if let Some(ref preview) = self.preview {
                    preview.emit(preview::WorkspacePreviewMsg::Hide);
                }
                if let Some(ref dropdown) = self.preview_dropdown
                    && dropdown.is_visible()
                {
                    dropdown.popdown();
                }
                self.current_preview_ws = None;
            }
            WorkspacesMsg::FocusWindow(address) => {
                self.focus_window_by_address(&address);
            }
        }
    }

    fn update_cmd(&mut self, msg: WorkspacesCmd, sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            WorkspacesCmd::WorkspacesChanged => {
                self.rebuild_buttons();
                force_window_resize(root);
            }
            WorkspacesCmd::ConfigChanged => {
                styling::apply_styling(&self.css_provider, &self.config, &self.settings);
                self.reinitialize_preview(&sender);
                self.rebuild_buttons();
                force_window_resize(root);
            }
            WorkspacesCmd::ClientsChanged => {
                self.prune_stale_urgent_windows();

                if self.urgent_windows.is_empty() {
                    self.stop_blink_timer();
                }

                self.rebuild_buttons();
                force_window_resize(root);
            }
            WorkspacesCmd::TitleChanged => {
                self.update_app_icons_on_title_change();
            }
            WorkspacesCmd::ActiveWorkspaceChanged(id) => {
                let config = self.config.config();
                let monitor_specific = config.modules.hyprland_workspaces.monitor_specific.get();
                let has_min_workspace_count =
                    config.modules.hyprland_workspaces.min_workspace_count.get() > 0;

                if !self.should_apply_active_workspace_change(id, monitor_specific) {
                    return;
                }

                self.clear_urgent_windows_for_workspace(id);
                self.stop_blink_if_no_urgent();
                self.active_workspace_id = id;
                self.sync_after_active_workspace_change(has_min_workspace_count);
            }

            WorkspacesCmd::MonitorFocused {
                monitor,
                workspace_id,
            } => {
                self.focused_monitor = Some(monitor.clone());

                let config = self.config.config();
                let monitor_specific = config.modules.hyprland_workspaces.monitor_specific.get();
                let has_min_workspace_count =
                    config.modules.hyprland_workspaces.min_workspace_count.get() > 0;

                if should_update_for_monitor(
                    monitor_specific,
                    self.settings.monitor_name.as_deref(),
                    &monitor,
                ) {
                    self.clear_urgent_windows_for_workspace(workspace_id);
                    self.stop_blink_if_no_urgent();
                    self.active_workspace_id = workspace_id;
                    self.sync_after_active_workspace_change(has_min_workspace_count);
                }
            }
            WorkspacesCmd::HyprlandConfigReloaded => {
                debug!("Hyprland config reloaded, refreshing workspace rules");
                Self::spawn_load_workspace_rules(&sender, &self.hyprland);
            }
            WorkspacesCmd::UrgentWindow(address) => {
                let was_empty = self.urgent_windows.is_empty();
                self.urgent_windows.insert(address);

                if was_empty {
                    self.start_blink_timer(&sender);
                }

                self.update_active_state();
            }

            WorkspacesCmd::WindowFocused(address) => {
                if !self.urgent_windows.remove(&address) {
                    return;
                }

                if self.urgent_windows.is_empty() {
                    self.stop_blink_timer();
                }

                self.update_active_state();
            }

            WorkspacesCmd::BlinkTick => {
                self.blink_on = !self.blink_on;
                self.update_active_state();
            }
            WorkspacesCmd::WorkspaceRulesLoaded(rules) => {
                self.workspace_monitor_rules = rules;
                self.rebuild_buttons();
                force_window_resize(root);
            }
        }
    }
}

impl Drop for HyprlandWorkspaces {
    fn drop(&mut self) {
        gtk::style_context_remove_provider_for_display(
            &self.buttons.widget().display(),
            &self.css_provider,
        );
    }
}
