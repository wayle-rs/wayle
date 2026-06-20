//! [`WindowSwitcherDropdown`] private impl methods: rebuilding the row list.

use super::{WindowSwitcherDropdown, messages::WindowInfo};
use crate::glob;

impl WindowSwitcherDropdown {
    pub(super) fn rebuild_rows(&mut self) {
        let toplevels = self.service.toplevels.get();
        let config = &self.config.config().modules.window_switcher;
        let ignore_patterns = config.ignore_app_id.get();
        let max_title_length = config.max_title_length.get();

        let mut entries: Vec<_> = toplevels
            .values()
            .filter(|toplevel| {
                let app_id = toplevel.app_id.get();
                !app_id
                    .as_deref()
                    .is_some_and(|id| ignore_patterns.iter().any(|p| glob::matches(p, id)))
            })
            .map(|toplevel| {
                let title = toplevel.title.get().unwrap_or_default();
                WindowInfo {
                    key: toplevel.key,
                    title: truncate(&title, max_title_length),
                    app_id: toplevel.app_id.get().unwrap_or_default(),
                    is_active: toplevel.is_activated(),
                }
            })
            .collect();
        entries.sort_by_key(|info| info.key);

        self.ordered_keys = entries.iter().map(|info| info.key).collect();

        let mut guard = self.rows.guard();
        guard.clear();
        for info in entries {
            guard.push_back(info);
        }
    }

    pub(super) fn activate_row(&self, index: usize) {
        if let Some(key) = self.ordered_keys.get(index).copied() {
            self.service.activate_toplevel(key);
        }
    }
}

fn truncate(text: &str, max_chars: u32) -> String {
    if max_chars == 0 {
        return text.to_string();
    }
    let max_chars = max_chars as usize;
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let truncated: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{truncated}…")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_no_limit() {
        assert_eq!(truncate("hello world", 0), "hello world");
    }

    #[test]
    fn truncate_under_limit_unchanged() {
        assert_eq!(truncate("hi", 10), "hi");
    }

    #[test]
    fn truncate_over_limit_adds_ellipsis() {
        assert_eq!(truncate("hello world", 5), "hell…");
    }
}
