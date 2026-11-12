use gtk::prelude::*;
use relm4::{gtk, gtk::gio, spawn_local};
use wayle_config::schemas::modules::notification::{PopupCloseBehavior, UrgencyBarThreshold};
use wayle_notification::core::types::{Action, InvokeSource};

use super::NotificationPopupCard;
use crate::{
    i18n::t,
    shell::notification_popup::helpers::{
        RelativeTime, ResolvedIcon, load_scaled_file_icon, mint_activation_token,
        priority_bar_visible, priority_css_class, resolve_notification_icon,
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

        if priority_bar_visible(self.notification.view.get().classification.priority, urgency_bar) {
            root.add_css_class("urgency-bar");
        }
    }

    pub(super) fn apply_icon(&self, icon: &gtk::Image, icon_container: &gtk::Box) {
        // Reset first so reactive re-application doesn't accumulate the file-icon class
        // or leave a stale image when the source changes.
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
                let path = path.clone();
                let icon = icon.clone();
                let icon_container = icon_container.clone();

                spawn_local(async move {
                    let texture = gio::spawn_blocking(move || {
                        load_scaled_file_icon(&path, POPUP_ICON_TEXTURE_SIZE_PX)
                    })
                    .await
                    .ok()
                    .flatten();

                    let Some(texture) = texture else {
                        icon.set_icon_name(Some("ld-bell-symbolic"));
                        return;
                    };

                    icon.set_paintable(Some(&texture));
                    icon_container.add_css_class("file-icon");
                });
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

    /// The activation context for interactions on this popup card: the banner is always
    /// dismissed on activation, and the configured [`PopupCloseBehavior`] decides whether it is
    /// also removed from history. `invoke`/`activate_default` honor `resident` on top of this.
    fn invoke_source(&self) -> InvokeSource {
        InvokeSource::Popup {
            remove_from_history: matches!(self.close_behavior, PopupCloseBehavior::Remove),
        }
    }

    fn build_action_button(&self, action: &Action) -> gtk::Button {
        let button = gtk::Button::with_label(&action.label);
        button.add_css_class("notification-popup-action-btn");
        button.set_cursor_from_name(Some("pointer"));

        let notification = self.notification.clone();
        let action = action.clone();
        let source = self.invoke_source();

        button.connect_clicked(move |_| {
            let action = action.clone();
            tracing::debug!(action = %action.label, "action button clicked");
            let notif = notification.clone();
            let token = mint_activation_token();
            spawn_local(async move {
                // `invoke` dismisses per the popup's close policy (and honors `resident`, so a
                // media-control card survives its own action).
                if let Err(err) = notif.invoke(&action, source, token.as_deref()).await {
                    tracing::warn!(action = %action.label, error = %err, "action invocation failed");
                }
            });
        });

        button
    }

    pub(super) fn setup_default_action(&self, root: &gtk::Box) -> Option<gtk::GestureClick> {
        let default_action = self.notification.view.get().actions.default;
        if default_action.is_none() {
            root.set_cursor_from_name(None);
            return None;
        }

        root.set_cursor_from_name(Some("pointer"));

        let notification = self.notification.clone();
        let source = self.invoke_source();

        let click = gtk::GestureClick::new();
        click.connect_released(move |gesture, _, _, _| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            let notif = notification.clone();
            let token = mint_activation_token();
            spawn_local(async move {
                // `activate_default` dismisses per the popup's close policy (banner-only vs
                // remove-from-history), honoring `resident`.
                if let Err(err) = notif.activate_default(source, token.as_deref()).await {
                    tracing::warn!(error = %err, "default action invocation failed");
                }
            });
        });
        root.add_controller(click.clone());
        Some(click)
    }

    pub(super) fn apply_priority_class(&self, root: &gtk::Box) {
        // Idempotent: drop any previously-applied priority class before adding current.
        for class in ["low", "normal", "high", "urgent"] {
            root.remove_css_class(class);
        }
        root.add_css_class(priority_css_class(
            self.notification.view.get().classification.priority,
        ));
    }

    /// Re-renders the card in place from the current notification state (icon, app
    /// label, action buttons, priority, default-click gesture). Summary/body labels
    /// refresh declaratively via `#[watch]`.
    pub(super) fn refresh_notification(&mut self, root: &gtk::Box) {
        self.resolved_icon = resolve_notification_icon(self.icon_source, &self.notification);
        self.app_label = self
            .notification
            .view.get().origin
            .name
            .unwrap_or_else(|| t!("notification-popup-unknown-app"));

        if let (Some(icon), Some(container)) = (self.icon.clone(), self.icon_container.clone()) {
            self.apply_icon(&icon, &container);
        }
        if let Some(actions_box) = self.actions_box.clone() {
            self.setup_action_buttons(&actions_box);
        }

        self.apply_priority_class(root);
        if priority_bar_visible(
            self.notification.view.get().classification.priority,
            self.urgency_bar,
        ) {
            root.add_css_class("urgency-bar");
        } else {
            root.remove_css_class("urgency-bar");
        }

        if let Some(gesture) = self.default_gesture.take() {
            root.remove_controller(&gesture);
        }
        root.set_cursor_from_name(None);
        let gesture = self.setup_default_action(root);
        self.default_gesture = gesture;
    }

    pub(super) fn setup_hover_controller(&self, root: &gtk::Box) {
        if !self.hover_pause {
            return;
        }

        let hover = gtk::EventControllerMotion::new();
        let notif_enter = self.notification.clone();
        let notif_leave = self.notification.clone();

        hover.connect_enter(move |_, _, _| {
            notif_enter.inhibit_popup();
        });
        hover.connect_leave(move |_| {
            notif_leave.release_popup();
        });
        root.add_controller(hover);
    }
}
