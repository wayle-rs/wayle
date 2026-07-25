use gtk::prelude::*;
use relm4::{gtk, spawn_local};
use wayle_notification::core::types::{Action, InvokeSource};

use super::NotificationItem;
use crate::{
    i18n::t,
    shell::notification_popup::helpers::{
        RelativeTime, ResolvedIcon, load_scaled_file_icon, mint_activation_token,
        priority_css_class, resolve_notification_icon,
    },
};

const MAX_ACTIONS_PER_ROW: usize = 3;
const DROPDOWN_ICON_TEXTURE_SIZE_PX: i32 = 48;

impl NotificationItem {
    pub(super) fn apply_icon(&self, icon: &gtk::Image, icon_container: &gtk::Box) {
        // Reset first so repeated calls (reactive refresh) don't accumulate state or
        // leave a stale image when the source changes (e.g. Named -> File).
        icon.clear();
        icon_container.remove_css_class("file-icon");

        match &self.resolved_icon {
            ResolvedIcon::Named(name) => {
                icon.set_icon_name(Some(name));
                if !name.ends_with("-symbolic") {
                    icon_container.add_css_class("file-icon");
                }
            }

            ResolvedIcon::File(path) => {
                if let Some(texture) = load_scaled_file_icon(path, DROPDOWN_ICON_TEXTURE_SIZE_PX) {
                    icon.set_paintable(Some(&texture));
                    icon_container.add_css_class("file-icon");
                } else {
                    icon.set_icon_name(Some("ld-bell-symbolic"));
                }
            }
        }
    }

    pub(super) fn build_action_buttons(&self, actions_box: &gtk::Box) {
        // Clear existing rows so this is idempotent on reactive refresh.
        while let Some(child) = actions_box.first_child() {
            actions_box.remove(&child);
        }

        // The `buttons` facet already excludes the body/default action.
        let actions = self.notification.view.get().actions;
        let visible_actions: Vec<_> = actions.buttons.iter().collect();

        if visible_actions.is_empty() {
            actions_box.set_visible(false);
            return;
        }
        actions_box.set_visible(true);

        for chunk in visible_actions.chunks(MAX_ACTIONS_PER_ROW) {
            let row = self.build_action_row(chunk);
            actions_box.append(&row);
        }
    }

    fn build_action_row(&self, actions: &[&Action]) -> gtk::Box {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        row.add_css_class("notification-dropdown-item-action-row");
        row.set_homogeneous(true);

        for action in actions {
            let button = self.build_action_button(action);
            row.append(&button);
        }

        row
    }

    fn build_action_button(&self, action: &Action) -> gtk::Button {
        let button = gtk::Button::with_label(&action.label);
        button.add_css_class("notification-dropdown-item-action-btn");
        button.set_cursor_from_name(Some("pointer"));

        let notification = self.notification.clone();
        let action = action.clone();

        button.connect_clicked(move |_| {
            let notif = notification.clone();
            let action = action.clone();
            let token = mint_activation_token();

            spawn_local(async move {
                // `invoke` removes it from history itself (unless resident), so no explicit
                // dismiss — and a failed action no longer force-removes the notification.
                if let Err(err) = notif.invoke(&action, InvokeSource::History, token.as_deref()).await {
                    tracing::warn!(action = %action.label, error = %err, "action invocation failed");
                }
            });
        });

        button
    }

    pub(super) fn setup_default_action(&self, main_row: &gtk::Box) -> Option<gtk::GestureClick> {
        if self.notification.view.get().actions.default.is_none() {
            main_row.set_cursor_from_name(None);
            return None;
        }

        main_row.set_cursor_from_name(Some("pointer"));

        let notification = self.notification.clone();
        let click = gtk::GestureClick::new();

        click.connect_released(move |gesture, _, _, _| {
            gesture.set_state(gtk::EventSequenceState::Claimed);

            let notif = notification.clone();
            let token = mint_activation_token();
            spawn_local(async move {
                // `activate_default` owns removal from history (unless resident); no explicit
                // dismiss.
                if let Err(err) = notif.activate_default(InvokeSource::History, token.as_deref()).await {
                    tracing::warn!(error = %err, "default action invocation failed");
                }
            });
        });

        main_row.add_controller(click.clone());
        Some(click)
    }

    pub(super) fn apply_priority(&self, root: &gtk::Box) {
        // Idempotent: drop any previously-applied priority class before adding current.
        for class in ["low", "normal", "high", "urgent"] {
            root.remove_css_class(class);
        }
        root.add_css_class(priority_css_class(
            self.notification.view.get().classification.priority,
        ));
    }

    /// Re-renders the imperative parts of the item in place from the current
    /// notification state (icon, action buttons, urgency, default-click gesture).
    /// Declarative fields (summary/body/time) refresh via `#[watch]`.
    pub(super) fn refresh_widgets(&mut self) {
        self.resolved_icon = resolve_notification_icon(self.icon_source, &self.notification);

        if let (Some(icon), Some(container)) = (self.icon.clone(), self.icon_container.clone()) {
            self.apply_icon(&icon, &container);
        }

        if let Some(actions_box) = self.actions_box.clone() {
            self.build_action_buttons(&actions_box);
        }

        if let Some(root) = self.root.clone() {
            self.apply_priority(&root);
        }

        if let Some(main_row) = self.main_row.clone() {
            if let Some(gesture) = self.default_gesture.take() {
                main_row.remove_controller(&gesture);
            }
            let gesture = self.setup_default_action(&main_row);
            self.default_gesture = gesture;
        }
    }

    pub(crate) fn time_to_string(time: RelativeTime) -> String {
        match time {
            RelativeTime::JustNow => t!("notification-dropdown-time-just-now"),

            RelativeTime::Minutes(minutes) => {
                t!(
                    "notification-dropdown-time-minutes-ago",
                    minutes = minutes.to_string()
                )
            }

            RelativeTime::Hours(hours) => {
                t!(
                    "notification-dropdown-time-hours-ago",
                    hours = hours.to_string()
                )
            }
        }
    }
}
