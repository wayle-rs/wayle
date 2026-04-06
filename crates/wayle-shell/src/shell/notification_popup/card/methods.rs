use gtk::prelude::*;
use relm4::{gtk, spawn_local};
use wayle_config::schemas::modules::notification::{PopupCloseBehavior, UrgencyBarThreshold};
use wayle_notification::core::types::Action;

use super::NotificationPopupCard;
use crate::{
    i18n::t,
    shell::notification_popup::helpers::{
        RelativeTime, ResolvedIcon, load_scaled_file_icon, urgency_bar_visible,
    },
};

const POPUP_ICON_TEXTURE_SIZE_PX: i32 = 64;

impl NotificationPopupCard {
    pub(super) fn apply_css_classes(
        &self,
        root: &gtk::Box,
        shadow: bool,
        urgency_bar: UrgencyBarThreshold,
    ) {
        if shadow {
            root.add_css_class("shadow");
        }

        if urgency_bar_visible(self.notification.urgency.get(), urgency_bar) {
            root.add_css_class("urgency-bar");
        }
    }

    pub(super) fn apply_icon(&self, icon: &gtk::Image, icon_container: &gtk::Box) {
        match &self.resolved_icon {
            ResolvedIcon::Named(name) => {
                icon.set_icon_name(Some(name));
                if !name.ends_with("-symbolic") {
                    icon_container.add_css_class("file-icon");
                }
            }

            ResolvedIcon::File(path) => {
                if let Some(texture) = load_scaled_file_icon(path, POPUP_ICON_TEXTURE_SIZE_PX) {
                    icon.set_paintable(Some(&texture));
                    icon_container.add_css_class("file-icon");
                } else {
                    icon.set_icon_name(Some("ld-bell-symbolic"));
                }
            }
        }
    }

    pub(super) fn format_time_label(time: RelativeTime) -> String {
        match time {
            RelativeTime::JustNow => t!("notification-popup-time-just-now"),
            RelativeTime::Minutes(minutes) => {
                t!(
                    "notification-popup-time-minutes-ago",
                    minutes = minutes.to_string()
                )
            }
            RelativeTime::Hours(hours) => {
                t!(
                    "notification-popup-time-hours-ago",
                    hours = hours.to_string()
                )
            }
        }
    }

    pub(super) fn setup_action_buttons(&self, actions_box: &gtk::Box) {
        let actions = self.notification.actions.get();
        let visible_actions: Vec<_> = actions
            .iter()
            .filter(|action| action.id != Action::DEFAULT_ID)
            .collect();

        if visible_actions.is_empty() {
            actions_box.set_visible(false);
            return;
        }

        const MAX_PER_ROW: usize = 3;

        for chunk in visible_actions.chunks(MAX_PER_ROW) {
            let row = self.build_action_row(chunk);
            actions_box.append(&row);
        }
    }

    fn build_action_row(&self, actions: &[&Action]) -> gtk::Box {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        row.add_css_class("notification-popup-action-row");
        row.set_homogeneous(true);

        for action in actions {
            let button = self.build_action_button(action);
            row.append(&button);
        }

        row
    }

    fn build_action_button(&self, action: &Action) -> gtk::Button {
        let button = gtk::Button::with_label(&action.label);
        button.add_css_class("notification-popup-action-btn");
        button.set_cursor_from_name(Some("pointer"));

        let notification = self.notification.clone();
        let action_id = action.id.clone();
        let service = self.service.clone();
        let notif_id = self.notification.id;

        button.connect_clicked(move |_| {
            let notif = notification.clone();
            let aid = action_id.clone();
            tracing::debug!(id = notif_id, action = %aid, "action button clicked");
            service.dismiss_popup(notif_id);
            spawn_local(async move {
                if let Err(err) = notif.invoke(&aid).await {
                    tracing::warn!(action = %aid, error = %err, "action invocation failed");
                }
                notif.dismiss();
            });
        });

        button
    }

    pub(super) fn setup_default_action(&self, root: &gtk::Box) {
        let default_action = self.notification.default_action.get();
        if default_action.is_none() {
            return;
        }

        root.set_cursor_from_name(Some("pointer"));

        let notification = self.notification.clone();
        let service = self.service.clone();
        let notif_id = self.notification.id;
        let close_behavior = self.close_behavior;

        let click = gtk::GestureClick::new();
        click.connect_released(move |gesture, _, _, _| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            let notif = notification.clone();
            match close_behavior {
                PopupCloseBehavior::Dismiss => service.dismiss_popup(notif_id),
                PopupCloseBehavior::Remove => notif.dismiss(),
            }
            spawn_local(async move {
                if let Err(err) = notif.invoke(Action::DEFAULT_ID).await {
                    tracing::warn!(error = %err, "default action invocation failed");
                }
            });
        });
        root.add_controller(click);
    }

    pub(super) fn setup_hover_controller(&self, root: &gtk::Box) {
        if !self.hover_pause {
            return;
        }

        let hover = gtk::EventControllerMotion::new();
        let service_enter = self.service.clone();
        let notif_id = self.notification.id;
        let service_leave = self.service.clone();

        hover.connect_enter(move |_, _, _| {
            service_enter.inhibit_popup(notif_id);
        });
        hover.connect_leave(move |_| {
            service_leave.release_popup(notif_id);
        });
        root.add_controller(hover);
    }
}
