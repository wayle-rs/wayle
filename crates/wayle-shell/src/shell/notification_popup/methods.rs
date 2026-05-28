use std::sync::Arc;

use gtk::prelude::*;
use gtk4_layer_shell::{Edge, LayerShell};
use relm4::{Component, ComponentController, gtk};
use tracing::debug;
use wayle_config::schemas::modules::notification::{PopupMonitor, PopupPosition, StackingOrder};
use wayle_notification::core::notification::Notification;

use super::{
    NotificationPopupHost,
    card::{CardInit, NotificationPopupCard},
};
use crate::shell::helpers::layer_shell::{
    apply_layer as apply_window_layer, apply_monitor_by_connector, apply_primary_monitor,
    reset_anchors,
};

impl NotificationPopupHost {
    /// Reconciles the card list with the current popup state.
    pub(super) fn reconcile(&mut self, popups: Vec<Arc<Notification>>, root: &gtk::Window) {
        let max_visible = self
            .config
            .config()
            .modules
            .notifications
            .popup_max_visible
            .get() as usize;

        let mut visible_popups = popups;
        visible_popups.truncate(max_visible);

        self.remove_stale_cards(&visible_popups);

        let existing_ids: Vec<u32> = self.cards.iter().map(|(notif, _)| notif.id).collect();

        self.insert_new_cards(&visible_popups, &existing_ids);

        debug!(cards = self.cards.len(), "popup reconcile complete");

        root.set_visible(!visible_popups.is_empty());
    }

    fn remove_stale_cards(&mut self, active_popups: &[Arc<Notification>]) {
        let container = &self.card_container;
        self.cards.retain(|(stored_notif, controller)| {
            let still_active = active_popups
                .iter()
                .any(|popup| popup.id == stored_notif.id);

            if !still_active {
                container.remove(controller.widget());
            }
            still_active
        });
    }

    fn insert_new_cards(&mut self, popups: &[Arc<Notification>], existing_ids: &[u32]) {
        let config = self.config.config();
        let notif_config = &config.modules.notifications;

        let hover_pause = notif_config.popup_hover_pause.get();
        let close_behavior = notif_config.popup_close_behavior.get();
        let urgency_bar = notif_config.popup_urgency_bar.get();
        let icon_source = notif_config.icon_source.get();
        let shadow = notif_config.popup_shadow.get();
        let stacking_order = notif_config.popup_stacking_order.get();
        let use_prepend = matches!(stacking_order, StackingOrder::NewestFirst);

        for notif in popups {
            if existing_ids.contains(&notif.id) {
                continue;
            }

            let controller = NotificationPopupCard::builder()
                .launch(CardInit {
                    notification: notif.clone(),
                    service: self.notification.clone(),
                    config: self.config.clone(),
                    hover_pause,
                    close_behavior,
                    urgency_bar,
                    icon_source,
                    shadow,
                })
                .detach();

            if use_prepend {
                self.card_container.prepend(controller.widget());
                self.cards.insert(0, (notif.clone(), controller));
            } else {
                self.card_container.append(controller.widget());
                self.cards.push((notif.clone(), controller));
            }
        }
    }

    /// Applies layer-shell anchors, margins, and monitor based on config.
    pub(super) fn apply_position(&self, root: &gtk::Window) {
        let config = self.config.config();
        let notif_config = &config.modules.notifications;
        let position = notif_config.popup_position.get();
        let scale = config.styling.scale.get().value();
        let mx = (notif_config.popup_margin_x.get().value() * scale) as i32;
        let my = (notif_config.popup_margin_y.get().value() * scale) as i32;

        reset_anchors(root);

        match position {
            PopupPosition::TopLeft => {
                root.set_anchor(Edge::Top, true);
                root.set_anchor(Edge::Left, true);
                root.set_margin(Edge::Top, my);
                root.set_margin(Edge::Left, mx);
            }

            PopupPosition::TopCenter => {
                root.set_anchor(Edge::Top, true);
                root.set_margin(Edge::Top, my);
            }

            PopupPosition::TopRight => {
                root.set_anchor(Edge::Top, true);
                root.set_anchor(Edge::Right, true);
                root.set_margin(Edge::Top, my);
                root.set_margin(Edge::Right, mx);
            }

            PopupPosition::BottomLeft => {
                root.set_anchor(Edge::Bottom, true);
                root.set_anchor(Edge::Left, true);
                root.set_margin(Edge::Bottom, my);
                root.set_margin(Edge::Left, mx);
            }

            PopupPosition::BottomCenter => {
                root.set_anchor(Edge::Bottom, true);
                root.set_margin(Edge::Bottom, my);
            }

            PopupPosition::BottomRight => {
                root.set_anchor(Edge::Bottom, true);
                root.set_anchor(Edge::Right, true);
                root.set_margin(Edge::Bottom, my);
                root.set_margin(Edge::Right, mx);
            }

            PopupPosition::CenterLeft => {
                root.set_anchor(Edge::Left, true);
                root.set_margin(Edge::Left, mx);
            }

            PopupPosition::CenterRight => {
                root.set_anchor(Edge::Right, true);
                root.set_margin(Edge::Right, mx);
            }
        }

        let monitor = notif_config.popup_monitor.get();

        match &monitor {
            PopupMonitor::Primary => {
                apply_primary_monitor(root);
            }
            PopupMonitor::Connector(name) => {
                apply_monitor_by_connector(root, name);
            }
        }
    }

    pub(super) fn apply_layer(&self, root: &gtk::Window) {
        let configured = self.config.config().modules.notifications.popup_layer.get();
        apply_window_layer(root, configured, &self.config);
    }
}
