use serde::Deserialize;
use serde_json::Value;
use wayle_config::schemas::modules::CustomModuleDefinition;

const MAX_JSON_PARSE_BYTES: usize = 64 * 1024;

/// Reserved fields extracted from JSON output via serde deserialization.
#[derive(Debug, Clone, Default, Deserialize)]
struct ReservedFields {
    text: Option<String>,
    alt: Option<String>,
    percentage: Option<u8>,
    tooltip: Option<String>,
    #[serde(default)]
    class: ClassValue,
}

/// Flexible class field that accepts string or array of strings.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(untagged)]
enum ClassValue {
    #[default]
    None,
    Single(String),
    Multiple(Vec<String>),
}

impl ClassValue {
    fn into_vec(self) -> Vec<String> {
        match self {
            ClassValue::None => Vec::new(),
            ClassValue::Single(s) => vec![s],
            ClassValue::Multiple(v) => v,
        }
    }
}

/// Parsed output from a custom module command.
///
/// Contains extracted fields from JSON output or raw text for plain output.
#[derive(Debug, Clone, Default)]
pub struct ParsedOutput {
    /// Raw command output string.
    pub raw: String,
    /// Extracted `text` field from JSON, overrides format result.
    pub text: Option<String>,
    /// Extracted `alt` field from JSON, used for icon-map lookup.
    pub alt: Option<String>,
    /// Extracted `percentage` field from JSON (0-100), used for icon-names array.
    pub percentage: Option<u8>,
    /// Extracted `tooltip` field from JSON, overrides tooltip-format result.
    pub tooltip: Option<String>,
    /// Extracted `class` field from JSON, added as CSS classes.
    pub class: Vec<String>,
    /// Parsed JSON value for template rendering.
    pub json: Option<Value>,
}

impl ParsedOutput {
    /// Parses command output, auto-detecting JSON if output starts with `{` or `[`.
    pub fn parse(output: &str) -> Self {
        let trimmed = output.trim();
        let looks_like_json = trimmed.starts_with('{') || trimmed.starts_with('[');

        if looks_like_json
            && trimmed.len() <= MAX_JSON_PARSE_BYTES
            && let Ok(json) = serde_json::from_str::<Value>(trimmed)
        {
            return Self::from_json(trimmed.to_string(), json);
        }

        Self {
            raw: trimmed.to_string(),
            ..Default::default()
        }
    }

    fn from_json(raw: String, json: Value) -> Self {
        let reserved: ReservedFields = serde_json::from_value(json.clone()).unwrap_or_default();

        Self {
            raw,
            text: reserved.text,
            alt: reserved.alt,
            percentage: reserved.percentage.map(|p| p.min(100)),
            tooltip: reserved.tooltip,
            class: reserved.class.into_vec(),
            json: Some(json),
        }
    }

    /// Builds a template context merging JSON fields with the raw output.
    ///
    /// JSON fields are accessible as top-level variables. The raw output
    /// is always available as `output`.
    fn template_context(&self) -> Value {
        let mut ctx = self
            .json
            .clone()
            .and_then(|v| if v.is_object() { Some(v) } else { None })
            .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

        if let Value::Object(map) = &mut ctx {
            map.insert("output".to_string(), Value::String(self.raw.clone()));
        }

        ctx
    }
}

fn render_format(template: &str, parsed: &ParsedOutput) -> String {
    crate::template::render(template, parsed.template_context()).unwrap_or_default()
}

/// Finds a custom module definition by ID.
pub fn find_definition(
    definitions: &[CustomModuleDefinition],
    id: &str,
) -> Option<CustomModuleDefinition> {
    definitions.iter().find(|d| d.id == id).cloned()
}

/// Determines if the module should be hidden based on output.
pub fn should_hide(output: &str, hide_if_empty: bool) -> bool {
    if !hide_if_empty {
        return false;
    }
    output.is_empty() || output == "0" || output.eq_ignore_ascii_case("false")
}

/// Resolves the icon name based on definition and parsed output.
///
/// Priority: icon-map (alt) > icon-names (percentage) > icon-map (default) > icon-name (static)
///
/// This ordering allows state-specific icons (like muted) to override percentage-based
/// icons while still using percentage icons as the default for other states.
pub fn resolve_icon(definition: &CustomModuleDefinition, parsed: &ParsedOutput) -> String {
    if let Some(icon) = resolve_from_alt(definition, parsed) {
        return icon;
    }

    if let Some(icon) = resolve_from_percentage(definition, parsed) {
        return icon;
    }

    if let Some(icon) = resolve_from_default(definition) {
        return icon;
    }

    definition.icon_name.clone()
}

fn resolve_from_alt(definition: &CustomModuleDefinition, parsed: &ParsedOutput) -> Option<String> {
    let map = definition.icon_map.as_ref()?;
    let alt = parsed.alt.as_ref()?;
    map.get(alt).cloned()
}

fn resolve_from_percentage(
    definition: &CustomModuleDefinition,
    parsed: &ParsedOutput,
) -> Option<String> {
    let icons = definition.icon_names.as_ref().filter(|i| !i.is_empty())?;
    let pct = parsed.percentage?;
    let idx = (pct as usize * icons.len()) / 101;
    icons.get(idx).cloned()
}

fn resolve_from_default(definition: &CustomModuleDefinition) -> Option<String> {
    definition
        .icon_map
        .as_ref()
        .and_then(|m| m.get("default").cloned())
}

/// Resolves dynamic CSS classes from class-format and parsed output.
pub fn resolve_classes(definition: &CustomModuleDefinition, parsed: &ParsedOutput) -> Vec<String> {
    let mut classes = parsed.class.clone();

    if let Some(class_format) = &definition.class_format {
        let formatted = render_format(class_format, parsed);
        for class in formatted.split_whitespace() {
            if !class.is_empty() && !classes.contains(&class.to_string()) {
                classes.push(class.to_string());
            }
        }
    }

    classes
}

/// Formats the label using the definition's format string and parsed output.
pub fn format_label(definition: &CustomModuleDefinition, parsed: &ParsedOutput) -> String {
    if let Some(text) = &parsed.text {
        return text.clone();
    }

    render_format(&definition.format, parsed)
}

/// Formats the tooltip using the definition's tooltip-format and parsed output.
pub fn format_tooltip(
    definition: &CustomModuleDefinition,
    parsed: &ParsedOutput,
) -> Option<String> {
    if let Some(tooltip) = &parsed.tooltip {
        return Some(tooltip.clone());
    }

    definition
        .tooltip_format
        .as_ref()
        .map(|fmt| render_format(fmt, parsed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plain_text() {
        let output = ParsedOutput::parse("hello world");
        assert!(output.json.is_none());
        assert_eq!(output.raw, "hello world");
        assert!(output.text.is_none());
    }

    #[test]
    fn parse_json_with_percentage() {
        let output = ParsedOutput::parse(r#"{"percentage": 75}"#);
        assert!(output.json.is_some());
        assert_eq!(output.percentage, Some(75));
    }

    #[test]
    fn parse_json_with_alt() {
        let output = ParsedOutput::parse(r#"{"alt": "muted"}"#);
        assert!(output.json.is_some());
        assert_eq!(output.alt, Some("muted".to_string()));
    }

    #[test]
    fn parse_json_with_class_string() {
        let output = ParsedOutput::parse(r#"{"class": "warning"}"#);
        assert_eq!(output.class, vec!["warning".to_string()]);
    }

    #[test]
    fn parse_json_with_class_array() {
        let output = ParsedOutput::parse(r#"{"class": ["warning", "urgent"]}"#);
        assert_eq!(
            output.class,
            vec!["warning".to_string(), "urgent".to_string()]
        );
    }

    #[test]
    fn parse_large_json_falls_back_to_raw_text() {
        let payload = "x".repeat(MAX_JSON_PARSE_BYTES + 1);
        let raw = format!(r#"{{"text":"{payload}"}}"#);

        let output = ParsedOutput::parse(&raw);
        assert!(output.json.is_none());
        assert!(output.text.is_none());
        assert_eq!(output.raw, raw);
    }

    #[test]
    fn format_output_placeholder() {
        let output = ParsedOutput::parse("42");
        assert_eq!(render_format("Value: {{ output }}%", &output), "Value: 42%");
    }

    #[test]
    fn format_json_field() {
        let output = ParsedOutput::parse(r#"{"percentage": 50}"#);
        assert_eq!(
            render_format("Volume: {{ percentage }}%", &output),
            "Volume: 50%"
        );
    }

    #[test]
    fn format_nested_field() {
        let output = ParsedOutput::parse(r#"{"data": {"temp": 22}}"#);
        assert_eq!(
            render_format("Temp: {{ data.temp }}C", &output),
            "Temp: 22C"
        );
    }

    #[test]
    fn format_with_default_filter() {
        let output = ParsedOutput::parse("plain text");
        assert_eq!(
            render_format("{{ missing | default('N/A') }}", &output),
            "N/A"
        );
    }

    #[test]
    fn resolve_icon_from_percentage() {
        let definition = CustomModuleDefinition {
            id: "test".to_string(),
            icon_names: Some(vec![
                "low".to_string(),
                "medium".to_string(),
                "high".to_string(),
            ]),
            ..default_definition()
        };
        let output = ParsedOutput::parse(r#"{"percentage": 0}"#);
        assert_eq!(resolve_icon(&definition, &output), "low");

        let output = ParsedOutput::parse(r#"{"percentage": 50}"#);
        assert_eq!(resolve_icon(&definition, &output), "medium");

        let output = ParsedOutput::parse(r#"{"percentage": 100}"#);
        assert_eq!(resolve_icon(&definition, &output), "high");
    }

    #[test]
    fn resolve_icon_from_alt() {
        let mut icon_map = std::collections::HashMap::new();
        icon_map.insert("muted".to_string(), "muted-icon".to_string());
        icon_map.insert("default".to_string(), "default-icon".to_string());

        let definition = CustomModuleDefinition {
            id: "test".to_string(),
            icon_map: Some(icon_map),
            ..default_definition()
        };

        let output = ParsedOutput::parse(r#"{"alt": "muted"}"#);
        assert_eq!(resolve_icon(&definition, &output), "muted-icon");

        let output = ParsedOutput::parse(r#"{"alt": "unknown"}"#);
        assert_eq!(resolve_icon(&definition, &output), "default-icon");
    }

    #[test]
    fn resolve_icon_alt_overrides_percentage() {
        let mut icon_map = std::collections::HashMap::new();
        icon_map.insert("muted".to_string(), "muted-icon".to_string());

        let definition = CustomModuleDefinition {
            id: "test".to_string(),
            icon_names: Some(vec![
                "vol-0".to_string(),
                "vol-50".to_string(),
                "vol-100".to_string(),
            ]),
            icon_map: Some(icon_map),
            ..default_definition()
        };

        let output = ParsedOutput::parse(r#"{"percentage": 50, "alt": "muted"}"#);
        assert_eq!(resolve_icon(&definition, &output), "muted-icon");

        let output = ParsedOutput::parse(r#"{"percentage": 50}"#);
        assert_eq!(resolve_icon(&definition, &output), "vol-50");

        let output = ParsedOutput::parse(r#"{"percentage": 0}"#);
        assert_eq!(resolve_icon(&definition, &output), "vol-0");
    }

    fn default_definition() -> CustomModuleDefinition {
        use wayle_config::schemas::{
            modules::{ExecutionMode, RestartDelay, RestartPolicy},
            styling::ColorValue,
        };

        CustomModuleDefinition {
            id: String::new(),
            command: None,
            mode: ExecutionMode::Poll,
            interval_ms: 5000,
            restart_policy: RestartPolicy::default(),
            restart_interval_ms: RestartDelay::default(),
            format: "{{ output }}".to_string(),
            tooltip_format: None,
            hide_if_empty: false,
            icon_name: String::new(),
            icon_names: None,
            icon_map: None,
            class_format: None,
            icon_show: true,
            icon_color: ColorValue::Auto,
            icon_bg_color: ColorValue::Auto,
            label_show: true,
            label_color: ColorValue::Auto,
            label_max_length: 0,
            button_bg_color: ColorValue::Auto,
            border_show: false,
            border_color: ColorValue::Auto,
            left_click: String::new(),
            right_click: String::new(),
            middle_click: String::new(),
            scroll_up: String::new(),
            scroll_down: String::new(),
            on_action: None,
            dropdown_list_command: None,
            dropdown_select_command: None,
            label_ellipsize: wayle_config::schemas::modules::LabelEllipsize::End,
        }
    }
}
