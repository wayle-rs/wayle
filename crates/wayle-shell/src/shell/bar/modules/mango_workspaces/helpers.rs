//! Pure helpers: app-icon resolution, per-tag style lookup, CSS class naming.

use std::collections::HashMap;

use wayle_config::schemas::modules::WorkspaceStyle;

use crate::{glob, shell::bar::icons::DEFAULT_APP_ICON_MAP};

const TITLE_PREFIX: &str = "title:";
const APP_PREFIX: &str = "app:";

/// CSS class used to address a tag button by index, for example `tag-3`.
pub(super) fn tag_css_class(index: u32) -> String {
    format!("tag-{index}")
}

/// Looks up a per-tag style override, keyed by the one-based tag index.
pub(super) fn tag_style(
    index: u32,
    map: &HashMap<String, WorkspaceStyle>,
) -> Option<&WorkspaceStyle> {
    map.get(&index.to_string())
}

/// Resolves the icon name for a client using the configured icon map.
///
/// Lookup order: `title:`-prefixed patterns against the title, then app-prefixed
/// or unprefixed patterns against the app id, then the built-in defaults. Falls
/// back to `fallback` when nothing matches.
pub(super) fn resolve_app_icon(
    app_id: Option<&str>,
    title: Option<&str>,
    user_map: &HashMap<String, String>,
    fallback: &str,
) -> String {
    let (title_entries, app_entries): (Vec<_>, Vec<_>) = user_map
        .iter()
        .partition(|(pattern, _)| pattern.starts_with(TITLE_PREFIX));

    if let Some(title) = title
        && let Some(icon) = match_prefixed(&title_entries, TITLE_PREFIX, title)
    {
        return icon.to_string();
    }

    let Some(app_id) = app_id else {
        return fallback.to_string();
    };

    if let Some(icon) = match_prefixed(&app_entries, APP_PREFIX, app_id) {
        return icon.to_string();
    }

    if let Some(icon) = glob::find_match(DEFAULT_APP_ICON_MAP.iter().copied(), app_id) {
        return icon.to_string();
    }

    fallback.to_string()
}

/// Matches `query` against `entries`, stripping `prefix` from each pattern
/// first, and returns the matched icon name.
///
/// ```text
/// prefix:  "app:"
/// entries: [("app:*firefox*", "ld-globe")]
/// query:   "org.mozilla.firefox"
/// returns: Some("ld-globe")
/// ```
fn match_prefixed<'a>(
    entries: &[(&'a String, &'a String)],
    prefix: &str,
    query: &str,
) -> Option<&'a str> {
    let candidates = entries.iter().map(|(pattern, icon)| {
        let stripped = pattern.strip_prefix(prefix).unwrap_or(pattern);
        (stripped, icon.as_str())
    });

    glob::find_match(candidates, query)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_css_class_format() {
        assert_eq!(tag_css_class(1), "tag-1");
        assert_eq!(tag_css_class(9), "tag-9");
    }

    #[test]
    fn tag_style_keys_on_index() {
        let mut map = HashMap::new();
        map.insert(
            String::from("2"),
            WorkspaceStyle {
                icon: Some(String::from("ld-terminal-symbolic")),
                color: None,
                label: None,
            },
        );

        let icon = tag_style(2, &map).and_then(|style| style.icon.clone());
        assert_eq!(icon, Some(String::from("ld-terminal-symbolic")));
        assert!(tag_style(1, &map).is_none());
    }

    #[test]
    fn resolve_app_icon_unprefixed_matches_app_id() {
        let mut map = HashMap::new();
        map.insert(String::from("*firefox*"), String::from("ld-globe"));
        assert_eq!(
            resolve_app_icon(Some("org.mozilla.firefox"), None, &map, "fallback"),
            "ld-globe",
        );
    }

    #[test]
    fn resolve_app_icon_title_takes_priority_over_app() {
        let mut map = HashMap::new();
        map.insert(String::from("title:*YouTube*"), String::from("si-youtube"));
        map.insert(String::from("*firefox*"), String::from("ld-globe"));
        assert_eq!(
            resolve_app_icon(
                Some("org.mozilla.firefox"),
                Some("YouTube - Firefox"),
                &map,
                "fallback",
            ),
            "si-youtube",
        );
    }

    #[test]
    fn resolve_app_icon_falls_back_when_no_match() {
        let map = HashMap::new();
        assert_eq!(
            resolve_app_icon(Some("unknown.app"), Some("Unknown"), &map, "ld-default"),
            "ld-default",
        );
    }
}
