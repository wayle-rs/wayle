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
mod styling;
mod watchers;

use std::{
    collections::{HashMap, HashSet},
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
};
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
    active_workspace_any_monitor: HashSet<WorkspaceId>,
    focused_monitor: Option<String>,
    workspace_monitor_rules: HashMap<WorkspaceId, String>,
    urgent_windows: HashSet<Address>,
    blink_on: bool,
    blink_token: Option<CancellationToken>,
    css_provider: gtk::CssProvider,
    buttons: FactoryVecDeque<WorkspaceButton>,
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

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = init.config.config();
        let workspaces_config = &config.modules.hyprland_workspaces;
        let monitor_specific = workspaces_config.monitor_specific.get();
        let monitor_specific_highlight = workspaces_config.highlight_active_on_other_monitor.get();
        let theme_provider = config.styling.theme_provider.clone();

        let active_id = Self::initial_active_workspace(
            &init.hyprland,
            &init.settings,
            monitor_specific || monitor_specific_highlight,
        );
        let active_any_monitor = Self::initial_active_workspace_other_monitor(&init.hyprland);
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
            },
        );

        let mut model = Self {
            hyprland: init.hyprland,
            config: init.config,
            settings: init.settings,
            active_workspace_id: active_id,
            active_workspace_any_monitor: active_any_monitor,
            focused_monitor,
            workspace_monitor_rules: HashMap::new(),
            urgent_windows: HashSet::new(),
            blink_on: false,
            blink_token: None,
            css_provider,
            buttons,
        };

        styling::apply_styling(&model.css_provider, &model.config, &model.settings);
        model.rebuild_buttons();

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
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
                let config_hypr = &self.config.config().modules.hyprland_workspaces;
                let monitor_specific = config_hypr.monitor_specific.get();
                let monitor_specific_highlight =
                    config_hypr.highlight_active_on_other_monitor.get();
                let has_min_workspace_count = config_hypr.min_workspace_count.get() > 0;

                if !self.should_apply_active_workspace_change(
                    id,
                    monitor_specific || monitor_specific_highlight,
                ) {
                    if (!monitor_specific) && monitor_specific_highlight {
                        self.rebuild_buttons();
                    }
                    return;
                }

                self.clear_urgent_windows_for_workspace(id);
                self.stop_blink_if_no_urgent();
                self.active_workspace_any_monitor
                    .remove(&self.active_workspace_id);
                self.active_workspace_any_monitor.insert(id);
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
                let monitor_specific_highlight = config
                    .modules
                    .hyprland_workspaces
                    .highlight_active_on_other_monitor
                    .get();
                let has_min_workspace_count =
                    config.modules.hyprland_workspaces.min_workspace_count.get() > 0;

                if should_update_for_monitor(
                    monitor_specific || monitor_specific_highlight,
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
