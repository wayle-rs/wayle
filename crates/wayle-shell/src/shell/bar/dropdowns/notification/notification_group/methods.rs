use std::sync::Arc;

use wayle_config::schemas::modules::notification::IconSource;
use wayle_notification::core::notification::Notification;

use super::{super::notification_item::messages::NotificationItemInit, NotificationGroup};
use crate::shell::notification_popup::helpers::{ResolvedIcon, resolve_notification_icon};

const MAX_VISIBLE_ITEMS: usize = 3;

impl NotificationGroup {
    pub(super) fn reconcile_items(&mut self, notifications: Vec<Arc<Notification>>) {
        self.update_metadata(&notifications);

        let cap = self.visible_cap(notifications.len());
        let visible = &notifications[..notifications.len().min(cap)];
        self.overflow_count = notifications.len().saturating_sub(cap);

        self.rebuild_items_if_changed(visible);
        self.notifications = notifications;
    }

    pub(super) fn reset_to_default_cap(&mut self) {
        if self.items.len() <= MAX_VISIBLE_ITEMS {
            return;
        }

        {
            let mut guard = self.items.guard();
            while guard.len() > MAX_VISIBLE_ITEMS {
                guard.pop_back();
            }
        }

        self.overflow_count = self.notifications.len().saturating_sub(MAX_VISIBLE_ITEMS);
    }

    pub(super) fn show_all_items(&mut self) {
        let remaining = &self.notifications[self.items.len()..];
        if remaining.is_empty() {
            self.overflow_count = 0;
            return;
        }

        let inits: Vec<_> = remaining
            .iter()
            .map(|notification| build_item_init(self.icon_source, notification))
            .collect();

        {
            let mut guard = self.items.guard();
            for init in inits {
                guard.push_back(init);
            }
        }

        self.overflow_count = 0;
    }

    pub(super) fn resolve_group_icon(
        _icon_source: IconSource,
        notifications: &[Arc<Notification>],
    ) -> Option<String> {
        let first = notifications.first()?;
        let resolved = resolve_notification_icon(IconSource::Mapped, first);

        match resolved {
            ResolvedIcon::Named(name) => Some(name),
            _ => None,
        }
    }

    fn update_metadata(&mut self, notifications: &[Arc<Notification>]) {
        self.total_count = notifications.len();
        self.count = notifications.len();

        self.preview = notifications
            .first()
            .map(|notification| notification.view.get().content.summary)
            .unwrap_or_default();
    }

    fn visible_cap(&self, total: usize) -> usize {
        let showing_all = self.items.len() > MAX_VISIBLE_ITEMS;
        if showing_all {
            total
        } else {
            MAX_VISIBLE_ITEMS
        }
    }

    fn rebuild_items_if_changed(&mut self, visible: &[Arc<Notification>]) {
        // Unchanged if the item list already matches `visible` by identity (`PartialEq` =
        // same notification id) in the same order.
        let unchanged = self.items.len() == visible.len()
            && (0..self.items.len()).all(|idx| {
                self.items
                    .get(idx)
                    .zip(visible.get(idx))
                    .is_some_and(|(item, notification)| &item.notification == notification)
            });
        if unchanged {
            return;
        }

        // Reconcile the factory in place within a single guard scope (one render): drop
        // items that are no longer visible, then move/insert so each slot matches
        // `visible`. Items whose notification persists keep their existing widget and
        // reactive watchers rather than being destroyed and rebuilt.
        let icon_source = self.icon_source;
        let mut guard = self.items.guard();

        for idx in (0..guard.len()).rev() {
            let still_visible = guard.get(idx).is_some_and(|item| {
                visible
                    .iter()
                    .any(|notification| &item.notification == notification)
            });
            if !still_visible {
                guard.remove(idx);
            }
        }

        for (target_idx, notification) in visible.iter().enumerate() {
            let existing = (target_idx..guard.len()).find(|&idx| {
                guard
                    .get(idx)
                    .is_some_and(|item| &item.notification == notification)
            });

            match existing {
                Some(idx) if idx == target_idx => {}
                Some(idx) => {
                    guard.move_to(idx, target_idx);
                }
                None => {
                    guard.insert(target_idx, build_item_init(icon_source, notification));
                }
            }
        }
    }
}

fn build_item_init(
    icon_source: IconSource,
    notification: &Arc<Notification>,
) -> NotificationItemInit {
    let resolved_icon = resolve_notification_icon(icon_source, notification);

    NotificationItemInit {
        notification: notification.clone(),
        resolved_icon,
        icon_source,
    }
}
