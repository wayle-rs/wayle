//! [`ExtWorkspaces`] private impl methods: rebuild, border classes, action
//! dispatch.

use gtk::prelude::*;
use relm4::gtk;
use tracing::warn;
use wayle_config::{
    ClickAction,
    schemas::{bar::BorderLocation, modules::WorkspaceClickAction},
};

use super::{
    ExtWorkspaces,
    button::ExtWorkspaceButtonInit,
    filtering::{self, FilterContext, WorkspaceSnapshot},
    helpers,
};
use crate::{process, shell::bar::dropdowns};

impl ExtWorkspaces {
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
        let workspaces = self.service.workspaces.get();
        let config = self.config.config();
        let ws_config = &config.modules.ext_workspaces;

        let ignore_patterns = ws_config.workspace_ignore.get();
        let ctx = FilterContext {
            show_hidden: ws_config.show_hidden.get(),
            ignore_patterns: &ignore_patterns,
        };

        let snapshots = workspaces
            .values()
            .map(|workspace| WorkspaceSnapshot {
                key: workspace.key,
                name: workspace.name.get(),
                coordinates: workspace.coordinates.get(),
                is_active: workspace.is_active(),
                is_urgent: workspace.is_urgent(),
                is_hidden: workspace.is_hidden(),
            })
            .collect();
        let displayed = filtering::collect_displayed(snapshots, &ctx);

        let urgent_show = ws_config.urgent_show.get();
        let display_mode = ws_config.display_mode.get();
        let active_indicator = ws_config.active_indicator.get();
        let workspace_map = ws_config.workspace_map.get();
        let is_vertical = self.is_vertical();

        {
            let mut guard = self.buttons.guard();
            guard.clear();
            for snapshot in &displayed {
                let style = helpers::workspace_style(snapshot.name.as_deref(), snapshot.key, &workspace_map);
                let label = style
                    .and_then(|style| style.label.clone())
                    .or_else(|| Some(helpers::label_for(snapshot.key, snapshot.name.as_deref())));
                let icon = style.and_then(|style| style.icon.clone());

                guard.push_back(ExtWorkspaceButtonInit {
                    key: snapshot.key,
                    name: snapshot.name.clone(),
                    label,
                    icon,
                    is_active: snapshot.is_active,
                    is_urgent: snapshot.is_urgent && urgent_show,
                    is_hidden: snapshot.is_hidden,
                    is_vertical,
                    display_mode,
                    active_indicator,
                });
            }
        }

        self.update_border_classes(ws_config.border_show.get());
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

    pub(super) fn dispatch_click_action(&self, action: WorkspaceClickAction, click_key: u32) {
        match action {
            WorkspaceClickAction::None => {}
            WorkspaceClickAction::FocusWorkspace => self.service.activate_workspace(click_key),
            WorkspaceClickAction::FocusNext => self.activate_relative(1),
            WorkspaceClickAction::FocusPrevious => self.activate_relative(-1),
            WorkspaceClickAction::FocusLast => {
                warn!(
                    "ext-workspaces does not track focus history; FocusLast is not supported, ignoring"
                );
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
            WorkspaceClickAction::FocusNext => self.activate_relative(1),
            WorkspaceClickAction::FocusPrevious => self.activate_relative(-1),
            WorkspaceClickAction::FocusLast => {
                warn!(
                    "ext-workspaces does not track focus history; FocusLast is not supported, ignoring"
                );
            }
            WorkspaceClickAction::Dropdown(name) => {
                let action = ClickAction::Dropdown(name);
                dropdowns::dispatch_click_widget(&action, &self.dropdowns, self.buttons.widget());
            }
            WorkspaceClickAction::Shell(cmd) => process::run_if_set(&cmd),
        }
    }

    fn activate_relative(&self, direction: i64) {
        let config = self.config.config();
        let ws_config = &config.modules.ext_workspaces;
        let ignore_patterns = ws_config.workspace_ignore.get();
        let ctx = FilterContext {
            show_hidden: ws_config.show_hidden.get(),
            ignore_patterns: &ignore_patterns,
        };

        let workspaces = self.service.workspaces.get();
        let snapshots = workspaces
            .values()
            .map(|workspace| WorkspaceSnapshot {
                key: workspace.key,
                name: workspace.name.get(),
                coordinates: workspace.coordinates.get(),
                is_active: workspace.is_active(),
                is_urgent: workspace.is_urgent(),
                is_hidden: workspace.is_hidden(),
            })
            .collect();
        let displayed = filtering::collect_displayed(snapshots, &ctx);

        if displayed.is_empty() {
            return;
        }

        let current_idx = displayed
            .iter()
            .position(|ws| ws.is_active)
            .unwrap_or(0);
        let new_idx = calculate_navigation_index(current_idx, direction, displayed.len());

        if let Some(ws) = displayed.get(new_idx) {
            self.service.activate_workspace(ws.key);
        }
    }
}

fn calculate_navigation_index(current_idx: usize, direction: i64, total: usize) -> usize {
    if direction > 0 {
        (current_idx + 1) % total
    } else if current_idx == 0 {
        total - 1
    } else {
        current_idx - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_forward_at_end() {
        assert_eq!(calculate_navigation_index(4, 1, 5), 0);
    }

    #[test]
    fn wraps_backward_at_start() {
        assert_eq!(calculate_navigation_index(0, -1, 5), 4);
    }

    #[test]
    fn moves_forward() {
        assert_eq!(calculate_navigation_index(2, 1, 5), 3);
    }

    #[test]
    fn moves_backward() {
        assert_eq!(calculate_navigation_index(2, -1, 5), 1);
    }
}
