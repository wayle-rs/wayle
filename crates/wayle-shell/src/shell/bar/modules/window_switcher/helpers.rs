//! Pure helpers: window counting/filtering and label formatting.

use std::{collections::HashMap, sync::Arc};

use wayle_wlr_toplevel::core::Toplevel;

use crate::glob;

/// Returns `true` when the toplevel's app-id matches any of the ignore
/// patterns.
pub(super) fn is_ignored(app_id: Option<&str>, patterns: &[String]) -> bool {
    let Some(app_id) = app_id else {
        return false;
    };
    patterns
        .iter()
        .any(|pattern| glob::matches(pattern, app_id))
}

/// Counts toplevels not matched by `ignore_patterns`.
pub(super) fn count_visible(
    toplevels: &HashMap<u32, Arc<Toplevel>>,
    ignore_patterns: &[String],
) -> usize {
    toplevels
        .values()
        .filter(|toplevel| !is_ignored(toplevel.app_id.get().as_deref(), ignore_patterns))
        .count()
}

/// Renders the count label, honoring `hide_when_empty`.
pub(super) fn count_label(count: usize, hide_when_empty: bool) -> String {
    if count == 0 && hide_when_empty {
        String::new()
    } else {
        count.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignore_matches_app_id_glob() {
        let patterns = vec![String::from("org.gnome.*")];
        assert!(is_ignored(Some("org.gnome.Nautilus"), &patterns));
        assert!(!is_ignored(Some("firefox"), &patterns));
    }

    #[test]
    fn ignore_no_app_id_never_matches() {
        let patterns = vec![String::from("*")];
        assert!(!is_ignored(None, &patterns));
    }

    #[test]
    fn label_shows_count_by_default() {
        assert_eq!(count_label(0, false), "0");
        assert_eq!(count_label(3, false), "3");
    }

    #[test]
    fn label_hides_when_empty_and_configured() {
        assert_eq!(count_label(0, true), "");
        assert_eq!(count_label(2, true), "2");
    }
}
