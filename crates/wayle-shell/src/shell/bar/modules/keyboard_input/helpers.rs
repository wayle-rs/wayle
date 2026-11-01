//! Pure helper for the keyboard-input module: template rendering.

use std::collections::BTreeMap;

use serde_json::json;

use crate::template;

pub(super) fn format_label(
    layout: &str,
    format: &str,
    layout_alias_map: &BTreeMap<String, String>,
) -> String {
    let alias = layout_alias_map
        .get(layout)
        .map(String::as_str)
        .unwrap_or(layout);
    let ctx = json!({ "layout": layout, "alias": alias });
    template::render(format, ctx).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_layout_only() {
        assert_eq!(format_label("us", "{{ layout }}", &BTreeMap::new()), "us");
    }

    #[test]
    fn format_with_prefix() {
        assert_eq!(
            format_label("de", "KB: {{ layout }}", &BTreeMap::new()),
            "KB: de"
        );
    }

    #[test]
    fn format_multiple_placeholders() {
        assert_eq!(
            format_label("us", "{{ layout }} | {{ alias }}", &BTreeMap::new()),
            "us | us"
        );
    }

    #[test]
    fn format_multiple_placeholders_with_alias() {
        assert_eq!(
            format_label(
                "us",
                "{{ layout }} | {{ alias }}",
                &BTreeMap::from([("us".to_string(), "US".to_string())])
            ),
            "us | US"
        );
    }

    #[test]
    fn format_lang_name_map_match() {
        assert_eq!(
            format_label(
                "us",
                "{{ alias }}",
                &BTreeMap::from([
                    ("us".to_string(), "US".to_string()),
                    ("de".to_string(), "DE".to_string()),
                ])
            ),
            "US",
        );
    }

    #[test]
    fn format_lang_name_map_no_match() {
        assert_eq!(
            format_label(
                "cz",
                "{{ alias }}",
                &BTreeMap::from([
                    ("us".to_string(), "US".to_string()),
                    ("de".to_string(), "DE".to_string()),
                ])
            ),
            "cz",
        );
    }
}
