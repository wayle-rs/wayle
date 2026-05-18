//! [`NiriWorkspaces`] private impl methods: rebuild, border classes, action dispatch.

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use gtk::prelude::*;
use relm4::{ComponentSender, gtk};
use tokio_util::sync::CancellationToken;
use tracing::warn;
use wayle_config::{
    ClickAction,
    schemas::{
        bar::BorderLocation,
        modules::{
            ActiveIndicator, DisplayMode, LabelStrategy, NiriWorkspaceMap, UrgentMode,
            WorkspaceClickAction,
        },
    },
};
use wayle_niri::{Action, WorkspaceReferenceArg, core::Window};

use super::{
    BLINK_INTERVAL, NiriWorkspaces, NiriWorkspacesCmd,
    button::{AppIconInit, NiriWorkspaceButtonInit},
    filtering::{self, FilterContext, WorkspaceSnapshot},
    helpers,
};
use crate::{process, shell::bar::dropdowns};

const REM_BASE_PX: f32 = 16.0;

impl NiriWorkspaces {
    pub(super) fn bar_monitor(&self) -> Option<&str> {
        self.settings.monitor_name.as_deref()
    }

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

    pub(super) fn rebuild_buttons(&mut self) {
        let workspaces = self.niri.workspaces.get();
        let windows = self.niri.windows.get();
        let config = self.config.config();
        let ws_config = &config.modules.niri_workspaces;

        let ignore_patterns = ws_config.workspace_ignore.get();
        let ctx = FilterContext {
            monitor_specific: ws_config.monitor_specific.get(),
            bar_monitor: self.bar_monitor(),
            hide_trailing_empty: ws_config.hide_trailing_empty.get(),
            min_workspace_count: usize::from(ws_config.min_workspace_count.get()),
            ignore_patterns: &ignore_patterns,
        };

        let snapshots = workspaces
            .values()
            .map(|workspace| WorkspaceSnapshot {
                id: workspace.id.get(),
                idx: workspace.idx.get(),
                name: workspace.name.get(),
                output: workspace.output.get(),
                is_urgent: workspace.is_urgent.get(),
                is_active: workspace.is_active.get(),
                is_focused: workspace.is_focused.get(),
                has_windows: workspace.active_window_id.get().is_some(),
            })
            .collect();
        let displayed = filtering::collect_displayed(snapshots, &ctx);

        let urgent_show = ws_config.urgent_show.get();
        self.urgent_present = urgent_show && any_urgent(&displayed, &windows);

        let layout = ButtonLayout {
            label_strategy: ws_config.label_strategy.get(),
            display_mode: ws_config.display_mode.get(),
            active_indicator: ws_config.active_indicator.get(),
            urgent_show,
            urgent_mode: ws_config.urgent_mode.get(),
            is_vertical: self.is_vertical(),
            app_icons_show: ws_config.app_icons_show.get(),
            divider: ws_config.divider.get(),
            icon_gap_px: (ws_config.icon_gap.get().value() * REM_BASE_PX).round() as i32,
            empty_icon: ws_config.app_icons_empty.get(),
            app_icons_dedupe: ws_config.app_icons_dedupe.get(),
            app_icons_fallback: ws_config.app_icons_fallback.get(),
            app_icon_map: ws_config.app_icon_map.get(),
            workspace_map: ws_config.workspace_map.get(),
            blink_on: self.blink_on,
        };

        {
            let mut guard = self.buttons.guard();
            guard.clear();
            for snapshot in &displayed {
                let init = build_button_init(snapshot, &layout, &windows);
                guard.push_back(init);
            }
        }

        self.update_border_classes(ws_config.border_show.get());
    }

    pub(super) fn sync_blink(&mut self, sender: &ComponentSender<NiriWorkspaces>) {
        match (self.urgent_present, self.blink_token.is_some()) {
            (true, false) => self.start_blink_timer(sender),
            (false, true) => self.stop_blink_timer(),
            _ => {}
        }
    }

    pub(super) fn start_blink_timer(&mut self, sender: &ComponentSender<NiriWorkspaces>) {
        self.stop_blink_timer();
        self.blink_on = true;

        let token = CancellationToken::new();
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
                        let _ = out.send(NiriWorkspacesCmd::BlinkTick);
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

    pub(super) fn dispatch_click_action(&self, action: WorkspaceClickAction, click_id: u64) {
        match action {
            WorkspaceClickAction::None => {}
            WorkspaceClickAction::FocusWorkspace => self.spawn_focus_id(click_id),
            WorkspaceClickAction::FocusNext => {
                self.spawn_action(Action::FocusWorkspaceDown {});
            }
            WorkspaceClickAction::FocusPrevious => {
                self.spawn_action(Action::FocusWorkspaceUp {});
            }
            WorkspaceClickAction::FocusLast => {
                self.spawn_action(Action::FocusWorkspacePrevious {});
            }
            WorkspaceClickAction::Dropdown(name) => {
                let action = ClickAction::Dropdown(name);
                dropdowns::dispatch_click_widget(&action, &self.dropdowns, self.buttons.widget());
            }
            WorkspaceClickAction::Shell(cmd) => process::run_if_set(&cmd),
        }
    }

    pub(super) fn dispatch_scroll_action(&self, action: WorkspaceClickAction) {
        match action {
            WorkspaceClickAction::None => {}
            WorkspaceClickAction::FocusWorkspace => warn!(
                "WorkspaceClickAction::FocusWorkspace requires a clicked workspace; scroll ignored"
            ),
            WorkspaceClickAction::FocusNext => {
                self.spawn_action(Action::FocusWorkspaceDown {});
            }
            WorkspaceClickAction::FocusPrevious => {
                self.spawn_action(Action::FocusWorkspaceUp {});
            }
            WorkspaceClickAction::FocusLast => {
                self.spawn_action(Action::FocusWorkspacePrevious {});
            }
            WorkspaceClickAction::Dropdown(name) => {
                let action = ClickAction::Dropdown(name);
                dropdowns::dispatch_click_widget(&action, &self.dropdowns, self.buttons.widget());
            }
            WorkspaceClickAction::Shell(cmd) => process::run_if_set(&cmd),
        }
    }

    fn spawn_focus_id(&self, id: u64) {
        let niri = self.niri.clone();
        tokio::spawn(async move {
            if let Err(err) = niri.focus_workspace(WorkspaceReferenceArg::Id(id)).await {
                warn!(error = %err, workspace_id = id, "niri focus_workspace failed");
            }
        });
    }

    fn spawn_action(&self, action: Action) {
        let niri = self.niri.clone();
        let label = action_label(&action);
        tokio::spawn(async move {
            if let Err(err) = niri.dispatch_action(action).await {
                warn!(error = %err, action = label, "niri dispatch_action failed");
            }
        });
    }
}

struct ButtonLayout {
    label_strategy: LabelStrategy,
    display_mode: DisplayMode,
    active_indicator: ActiveIndicator,
    urgent_show: bool,
    urgent_mode: UrgentMode,
    is_vertical: bool,
    app_icons_show: bool,
    divider: String,
    icon_gap_px: i32,
    empty_icon: String,
    app_icons_dedupe: bool,
    app_icons_fallback: String,
    app_icon_map: HashMap<String, String>,
    workspace_map: NiriWorkspaceMap,
    blink_on: bool,
}

fn any_urgent(displayed: &[WorkspaceSnapshot], windows: &HashMap<u64, Arc<Window>>) -> bool {
    let displayed_ids: HashSet<u64> = displayed.iter().map(|snapshot| snapshot.id).collect();

    let urgent_workspace = displayed.iter().any(|snapshot| snapshot.is_urgent);
    let urgent_window = windows.values().any(|window| {
        window.is_urgent.get()
            && window
                .workspace_id
                .get()
                .is_some_and(|workspace_id| displayed_ids.contains(&workspace_id))
    });

    urgent_workspace || urgent_window
}

fn action_label(action: &Action) -> &'static str {
    match action {
        Action::FocusWorkspaceDown { .. } => "FocusWorkspaceDown",
        Action::FocusWorkspaceUp { .. } => "FocusWorkspaceUp",
        Action::FocusWorkspacePrevious { .. } => "FocusWorkspacePrevious",
        _ => "Action",
    }
}

fn windows_on_workspace(
    windows: &HashMap<u64, Arc<Window>>,
    workspace_id: u64,
) -> Vec<Arc<Window>> {
    let mut result: Vec<Arc<Window>> = windows
        .values()
        .filter(|window| window.workspace_id.get() == Some(workspace_id))
        .cloned()
        .collect();

    result.sort_by_key(|window| {
        let pos = window.layout.get().pos_in_scrolling_layout;
        let primary = pos.map(|(column, _)| column).unwrap_or(usize::MAX);
        let secondary = pos.map(|(_, tile)| tile).unwrap_or(0);
        (primary, secondary, window.id.get())
    });

    result
}

fn collect_urgent_window_ids(windows: &[Arc<Window>]) -> HashSet<u64> {
    windows
        .iter()
        .filter(|window| window.is_urgent.get())
        .map(|window| window.id.get())
        .collect()
}

fn collect_app_icons(
    windows: &[Arc<Window>],
    app_icon_map: &HashMap<String, String>,
    fallback: &str,
    dedupe: bool,
) -> Vec<AppIconInit> {
    let mut result: Vec<AppIconInit> = Vec::with_capacity(windows.len());
    for window in windows {
        let icon_name = helpers::resolve_app_icon(
            window.app_id.get().as_deref(),
            window.title.get().as_deref(),
            app_icon_map,
            fallback,
        );
        let window_id = window.id.get();
        if dedupe && let Some(existing) = result.iter_mut().find(|init| init.icon_name == icon_name)
        {
            existing.window_ids.push(window_id);
            continue;
        }
        result.push(AppIconInit {
            icon_name,
            window_ids: vec![window_id],
        });
    }
    result
}

fn build_button_init(
    snapshot: &WorkspaceSnapshot,
    layout: &ButtonLayout,
    windows: &HashMap<u64, Arc<Window>>,
) -> NiriWorkspaceButtonInit {
    let workspace_windows = windows_on_workspace(windows, snapshot.id);
    let urgent_window_ids = if layout.blink_on {
        collect_urgent_window_ids(&workspace_windows)
    } else {
        HashSet::new()
    };
    let app_icons = if layout.app_icons_show {
        collect_app_icons(
            &workspace_windows,
            &layout.app_icon_map,
            &layout.app_icons_fallback,
            layout.app_icons_dedupe,
        )
    } else {
        Vec::new()
    };

    let label = helpers::label_for(
        snapshot.idx,
        snapshot.name.as_deref(),
        layout.label_strategy,
    );
    let icon =
        helpers::workspace_style(snapshot.name.as_deref(), snapshot.id, &layout.workspace_map)
            .and_then(|style| style.icon.clone());

    NiriWorkspaceButtonInit {
        id: snapshot.id,
        name: snapshot.name.clone(),
        label,
        icon,
        is_active: snapshot.is_active,
        is_focused: snapshot.is_focused,
        is_urgent: snapshot.is_urgent && layout.blink_on,
        has_windows: snapshot.has_windows,
        is_vertical: layout.is_vertical,
        display_mode: layout.display_mode,
        active_indicator: layout.active_indicator,
        urgent_show: layout.urgent_show,
        urgent_mode: layout.urgent_mode,
        show_app_icons: layout.app_icons_show,
        app_icons,
        urgent_window_ids,
        divider: layout.divider.clone(),
        icon_gap_px: layout.icon_gap_px,
        empty_icon: layout.empty_icon.clone(),
    }
}
