//! Pure helpers: label rendering, workspace-map lookup, ignore matching, and
//! CSS class naming.

use wayle_config::schemas::modules::WorkspaceStyle;

use crate::glob;

/// Renders the label for a workspace: its name if set, otherwise its key.
pub(super) fn label_for(key: u32, name: Option<&str>) -> String {
    name.map(String::from).unwrap_or_else(|| key.to_string())
}

/// Looks up a per-workspace style override.
///
/// Tries the workspace name first, then the local key rendered as a string.
pub(super) fn workspace_style<'a>(
    name: Option<&str>,
    key: u32,
    map: &'a std::collections::BTreeMap<String, WorkspaceStyle>,
) -> Option<&'a WorkspaceStyle> {
    if let Some(name) = name
        && let Some(style) = map.get(name)
    {
        return Some(style);
    }
    map.get(&key.to_string())
}

/// CSS class used to address a workspace button by key, e.g. `ws-id-5`.
pub(super) fn workspace_id_css_class(key: u32) -> String {
    format!("ws-id-{key}")
}

/// CSS class used to address a workspace button by name, e.g. `ws-name-web`.
///
/// Non-identifier characters in the name are replaced with `_` so the
/// resulting class is always a valid CSS identifier.
pub(super) fn workspace_name_css_class(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect();
    format!("ws-name-{sanitized}")
}

/// Returns `true` when the workspace matches any of the ignore patterns.
///
/// Patterns are tried against the name, then the key.
pub(super) fn is_ignored(name: Option<&str>, key: u32, patterns: &[String]) -> bool {
    let key_str = key.to_string();

    patterns.iter().any(|pattern| {
        if let Some(name) = name
            && glob::matches(pattern, name)
        {
            return true;
        }
        glob::matches(pattern, &key_str)
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn label_uses_name_when_set() {
        assert_eq!(label_for(5, Some("web")), "web");
    }

    #[test]
    fn label_falls_back_to_key() {
        assert_eq!(label_for(5, None), "5");
    }

    #[test]
    fn workspace_style_prefers_name_match() {
        let mut map = BTreeMap::new();
        map.insert(
            String::from("web"),
            WorkspaceStyle {
                icon: Some(String::from("by-name")),
                color: None,
                label: None,
            },
        );
        map.insert(
            String::from("5"),
            WorkspaceStyle {
                icon: Some(String::from("by-id")),
                color: None,
                label: None,
            },
        );

        let icon = workspace_style(Some("web"), 5, &map).and_then(|style| style.icon.clone());
        assert_eq!(icon, Some(String::from("by-name")));
    }

    #[test]
    fn workspace_style_falls_back_to_key_when_name_missing() {
        let mut map = BTreeMap::new();
        map.insert(
            String::from("5"),
            WorkspaceStyle {
                icon: Some(String::from("by-id")),
                color: None,
                label: None,
            },
        );

        let icon = workspace_style(None, 5, &map).and_then(|style| style.icon.clone());
        assert_eq!(icon, Some(String::from("by-id")));
    }

    #[test]
    fn workspace_id_css_class_format() {
        assert_eq!(workspace_id_css_class(0), "ws-id-0");
        assert_eq!(workspace_id_css_class(42), "ws-id-42");
    }

    #[test]
    fn ignore_matches_by_name() {
        let patterns = vec![String::from("scratch")];
        assert!(is_ignored(Some("scratch"), 5, &patterns));
    }

    #[test]
    fn ignore_matches_by_key_glob() {
        let patterns = vec![String::from("1?")];
        assert!(is_ignored(None, 12, &patterns));
        assert!(!is_ignored(None, 5, &patterns));
    }

    #[test]
    fn ignore_no_match() {
        let patterns = vec![String::from("scratch"), String::from("foo")];
        assert!(!is_ignored(Some("web"), 2, &patterns));
    }
}
