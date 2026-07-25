use gtk::prelude::*;
use relm4::{gtk, spawn_local};
use wayle_notification::core::types::Action;

use super::NotificationItem;
use crate::{
    i18n::t,
    shell::notification_popup::helpers::{RelativeTime, ResolvedIcon, load_scaled_file_icon},
};

const MAX_ACTIONS_PER_ROW: usize = 3;
const DROPDOWN_ICON_TEXTURE_SIZE_PX: i32 = 48;

impl NotificationItem {
    pub(super) fn apply_icon(&self, icon: &gtk::Image, icon_container: &gtk::Box) {
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
        let actions = self.notification.actions.get();
        let visible_actions: Vec<_> = actions
            .iter()
            .filter(|action| action.id != Action::DEFAULT_ID)
            .collect();

        if visible_actions.is_empty() {
            actions_box.set_visible(false);
            return;
        }

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
        let action_id = action.id.clone();

        button.connect_clicked(move |_| {
            let notif = notification.clone();
            let aid = action_id.clone();

            spawn_local(async move {
                if let Err(err) = notif.invoke(&aid).await {
                    tracing::warn!(action = %aid, error = %err, "action invocation failed");
                }
                notif.dismiss();
            });
        });

        button
    }

    pub(super) fn setup_default_action(&self, main_row: &gtk::Box) {
        if self.notification.default_action.get().is_none() {
            return;
        }

        main_row.set_cursor_from_name(Some("pointer"));

        let notification = self.notification.clone();
        let click = gtk::GestureClick::new();

        click.connect_released(move |gesture, _, _, _| {
            gesture.set_state(gtk::EventSequenceState::Claimed);

            let notif = notification.clone();
            spawn_local(async move {
                if let Err(err) = notif.invoke(Action::DEFAULT_ID).await {
                    tracing::warn!(error = %err, "default action invocation failed");
                }
                notif.dismiss();
            });
        });

        main_row.add_controller(click);
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
