use super::{
    NotificationDropdown,
    helpers::{NotificationGroupData, group_by_app},
    notification_group::messages::{NotificationGroupInit, NotificationGroupInput},
};

impl NotificationDropdown {
    pub(super) fn rebuild_groups(&mut self) {
        let notifications = self.notification.notifications.get();
        self.has_notifications = !notifications.is_empty();

        let icon_source = self.config.config().modules.notifications.icon_source.get();

        let new_groups = group_by_app(&notifications);
        let new_keys: Vec<Option<String>> = new_groups
            .iter()
            .map(|group| group.app_name.clone())
            .collect();

        self.remove_stale_groups(&new_keys);
        self.update_or_insert_groups(&new_groups, icon_source);
        self.reorder_groups(&new_keys);
    }

    pub(super) fn force_rebuild_groups(&mut self) {
        let notifications = self.notification.notifications.get();
        self.has_notifications = !notifications.is_empty();

        let icon_source = self.config.config().modules.notifications.icon_source.get();

        let grouped = group_by_app(&notifications);

        let mut guard = self.groups.guard();
        guard.clear();

        for group_data in grouped {
            guard.push_back(NotificationGroupInit {
                app_name: group_data.app_name,
                notifications: group_data.notifications,
                icon_source,
            });
        }
    }

    fn remove_stale_groups(&mut self, active_keys: &[Option<String>]) {
        let indices_to_remove: Vec<usize> = (0..self.groups.len())
            .rev()
            .filter(|&idx| {
                self.groups
                    .get(idx)
                    .map(|group| !active_keys.contains(&group.app_name))
                    .unwrap_or(false)
            })
            .collect();

        if !indices_to_remove.is_empty() {
            let mut guard = self.groups.guard();
            for idx in indices_to_remove {
                guard.remove(idx);
            }
        }
    }

    fn update_or_insert_groups(
        &mut self,
        new_groups: &[NotificationGroupData],
        icon_source: wayle_config::schemas::modules::notification::IconSource,
    ) {
        let mut to_add = Vec::new();

        for group_data in new_groups {
            let existing_idx = self.find_group_by_app(&group_data.app_name);

            if let Some(idx) = existing_idx {
                self.groups.send(
                    idx,
                    NotificationGroupInput::UpdateNotifications(group_data.notifications.clone()),
                );
            } else {
                to_add.push(NotificationGroupInit {
                    app_name: group_data.app_name.clone(),
                    notifications: group_data.notifications.clone(),
                    icon_source,
                });
            }
        }

        if !to_add.is_empty() {
            let mut guard = self.groups.guard();
            for init in to_add {
                guard.push_back(init);
            }
        }
    }

    fn reorder_groups(&mut self, target_order: &[Option<String>]) {
        let mut guard = self.groups.guard();
        for (target_idx, key) in target_order.iter().enumerate() {
            let current_idx = (target_idx..guard.len()).find(|&idx| {
                guard
                    .get(idx)
                    .map(|group| &group.app_name == key)
                    .unwrap_or(false)
            });

            if let Some(current) = current_idx.filter(|&pos| pos != target_idx) {
                guard.move_to(current, target_idx);
            }
        }
    }

    fn find_group_by_app(&self, app_name: &Option<String>) -> Option<usize> {
        (0..self.groups.len()).find(|&idx| {
            self.groups
                .get(idx)
                .map(|group| &group.app_name == app_name)
                .unwrap_or(false)
        })
    }
}
