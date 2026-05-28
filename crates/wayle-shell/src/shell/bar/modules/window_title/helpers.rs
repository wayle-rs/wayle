//! Pure helpers for the window-title module: template rendering and icon
//! resolution. No Relm4, no GTK, no service types — easy to unit-test.

use std::collections::HashMap;

use serde_json::json;
use wayle_config::schemas::modules::WINDOW_TITLE_BUILTIN_MAPPINGS;

use crate::{glob, i18n::t, template};

const TITLE_PREFIX: &str = "title:";

pub(super) fn format_label(format: &str, title: &str, app_id: &str) -> String {
    let ctx = json!({
        "title": title,
        "app": app_id,
    });
    let label = template::render(format, ctx).unwrap_or_default();
    if label.trim().is_empty() {
        t!("bar-window-title-empty")
    } else {
        label
    }
}

pub(super) struct IconContext<'a> {
    pub title: &'a str,
    pub app_id: &'a str,
    pub user_mappings: &'a HashMap<String, String>,
    pub fallback: &'a str,
}

pub(super) fn resolve_icon(ctx: &IconContext<'_>) -> String {
    let (title_mappings, app_id_mappings): (Vec<_>, Vec<_>) = ctx
        .user_mappings
        .iter()
        .partition(|(pattern, _)| pattern.starts_with(TITLE_PREFIX));

    if let Some(icon) = glob::find_match(
        title_mappings.iter().map(|(pattern, icon)| {
            let stripped = pattern.strip_prefix(TITLE_PREFIX).unwrap_or(pattern);
            (stripped, icon.as_str())
        }),
        ctx.title,
    ) {
        return icon.to_string();
    }

    if let Some(icon) = glob::find_match(
        app_id_mappings
            .iter()
            .map(|(pattern, icon)| (pattern.as_str(), icon.as_str())),
        ctx.app_id,
    ) {
        return icon.to_string();
    }

    if let Some(icon) = glob::find_match(WINDOW_TITLE_BUILTIN_MAPPINGS.iter().copied(), ctx.app_id)
    {
        return icon.to_string();
    }

    ctx.fallback.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::t;

    #[test]
    fn format_title_only() {
        assert_eq!(format_label("{{ title }}", "Firefox", "firefox"), "Firefox");
    }

    #[test]
    fn format_app_only() {
        assert_eq!(format_label("{{ app }}", "Firefox", "firefox"), "firefox");
    }

    #[test]
    fn format_both_placeholders() {
        assert_eq!(
            format_label(
                "{{ app }}: {{ title }}",
                "Home - Mozilla Firefox",
                "firefox"
            ),
            "firefox: Home - Mozilla Firefox"
        );
    }

    #[test]
    fn format_with_prefix() {
        assert_eq!(
            format_label("Window: {{ title }}", "My App", "myapp"),
            "Window: My App"
        );
    }

    #[test]
    fn format_multiple_same_placeholder() {
        assert_eq!(
            format_label("{{ title }} | {{ title }}", "Hello", "app"),
            "Hello | Hello"
        );
    }

    #[test]
    fn format_empty_values_returns_placeholder() {
        let placeholder = t!("bar-window-title-empty");
        assert_eq!(format_label("{{ title }}", "", ""), placeholder);
        assert_eq!(format_label("{{ app }}", "", ""), placeholder);
    }

    #[test]
    fn resolve_icon_user_title_mapping_takes_priority() {
        let mut mappings = HashMap::new();
        mappings.insert("title:*Spotify*".to_string(), "user-spotify".to_string());
        mappings.insert("*spotify*".to_string(), "user-class-spotify".to_string());

        let icon = resolve_icon(&IconContext {
            title: "Spotify - Playing Music",
            app_id: "spotify",
            user_mappings: &mappings,
            fallback: "fallback-icon",
        });

        assert_eq!(icon, "user-spotify");
    }

    #[test]
    fn resolve_icon_user_class_mapping_over_builtin() {
        let mut mappings = HashMap::new();
        mappings.insert("*firefox*".to_string(), "my-firefox-icon".to_string());

        let icon = resolve_icon(&IconContext {
            title: "Home - Firefox",
            app_id: "firefox",
            user_mappings: &mappings,
            fallback: "fallback-icon",
        });

        assert_eq!(icon, "my-firefox-icon");
    }

    #[test]
    fn resolve_icon_builtin_class_mapping() {
        let icon = resolve_icon(&IconContext {
            title: "Home - Firefox",
            app_id: "firefox",
            user_mappings: &HashMap::new(),
            fallback: "fallback-icon",
        });

        assert_eq!(icon, "si-firefox-symbolic");
    }

    #[test]
    fn resolve_icon_fallback_when_no_match() {
        let icon = resolve_icon(&IconContext {
            title: "Unknown App",
            app_id: "unknown-app-class",
            user_mappings: &HashMap::new(),
            fallback: "fallback-icon",
        });

        assert_eq!(icon, "fallback-icon");
    }

    #[test]
    fn resolve_icon_wildcard_for_static_icon() {
        let mut mappings = HashMap::new();
        mappings.insert("*".to_string(), "my-static-icon".to_string());

        let icon = resolve_icon(&IconContext {
            title: "Any Title",
            app_id: "any-class",
            user_mappings: &mappings,
            fallback: "fallback-icon",
        });

        assert_eq!(icon, "my-static-icon");
    }
}
