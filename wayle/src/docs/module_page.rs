//! Emits one module's reference page as VitePress markdown.
//!
//! Each page is assembled top to bottom:
//!
//! 1. Frontmatter + H1 hook (the schema's top-level rustdoc).
//! 2. Optional "Add it to your layout" snippet for bar modules.
//! 3. One H2 per [`ConfigGroup`] containing a field table and, for fields
//!    with rich rustdoc, a `::: details` subsection.
//! 4. A default-values TOML block.

use std::collections::BTreeSet;

use serde_json::Value;
use tracing::warn;

use super::{generator::Error, registry::ModuleEntry, rustdoc, types_page::type_slug};
use crate::config::docs::{ConfigGroup, GroupRule};

const PAGE_H1_DEPTH: usize = 1;
const FIELD_SUBSECTION_DEPTH: usize = 3;

/// Renders the full markdown page for a single module.
///
/// `known_types` is the set of schema type names the types page documents.
/// Fields typed with one of those names render as a link to its anchor;
/// everything else renders as plain inline code.
///
/// # Errors
///
/// Returns [`Error::SerializeSchema`] if the schema can't be converted to JSON.
pub fn generate_module_page(
    entry: &ModuleEntry,
    known_types: &BTreeSet<String>,
) -> Result<String, Error> {
    let schema =
        serde_json::to_value((entry.info.schema)()).map_err(|source| Error::SerializeSchema {
            module: entry.info.name.clone(),
            source,
        })?;

    let fields = collect_fields(&schema, known_types);
    let buckets = bucket_fields(&fields, &entry.groups);

    let mut page = String::new();
    render_frontmatter(&mut page, &entry.info.name);
    page.push_str("<div v-pre>\n\n");
    render_hook(&mut page, &entry.info.name, &schema);
    render_layout_snippet(&mut page, entry.info.layout_id.as_deref());

    for (group, fields_in_group) in entry.groups.iter().zip(buckets.iter()) {
        if fields_in_group.is_empty() {
            continue;
        }
        render_group(&mut page, group, fields_in_group);
    }

    render_default_toml(&mut page, entry, &schema);

    page.push_str("\n</div>\n");
    Ok(page)
}

/// Everything the renderer needs about one property on the schema.
struct Field<'a> {
    /// Kebab-case name as it appears in TOML.
    name: &'a str,

    /// Full rustdoc body (summary plus any subsequent paragraphs).
    description: &'a str,

    /// Pre-rendered default value cell, wrapped in backticks or `required`.
    default_repr: String,

    /// Pre-rendered type cell, either an anchor link or an inline code token.
    type_repr: String,

    /// Named type this field references via `$ref`, if any. `None` for
    /// primitives and inline schemas.
    schema_type_name: Option<String>,
}

fn collect_fields<'a>(schema: &'a Value, known_types: &BTreeSet<String>) -> Vec<Field<'a>> {
    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return Vec::new();
    };

    properties
        .iter()
        .map(|(field_name, field_schema)| Field {
            name: field_name,
            description: field_schema
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or(""),
            default_repr: render_default_cell(field_schema.get("default")),
            type_repr: render_type_cell(field_schema, known_types),
            schema_type_name: extract_ref_type_name(field_schema),
        })
        .collect()
}

/// Extracts the named type a field points at through `$ref` or `allOf[0].$ref`.
fn extract_ref_type_name(field_schema: &Value) -> Option<String> {
    direct_ref(field_schema).or_else(|| all_of_ref(field_schema))
}

fn direct_ref(field_schema: &Value) -> Option<String> {
    let ref_path = field_schema.get("$ref")?.as_str()?;
    Some(ref_path_tail(ref_path).to_string())
}

fn all_of_ref(field_schema: &Value) -> Option<String> {
    let first_variant = field_schema.get("allOf")?.as_array()?.first()?;
    let ref_path = first_variant.get("$ref")?.as_str()?;
    Some(ref_path_tail(ref_path).to_string())
}

fn ref_path_tail(ref_path: &str) -> &str {
    ref_path.rsplit('/').next().unwrap_or(ref_path)
}

fn render_default_cell(default: Option<&Value>) -> String {
    match default {
        None => String::from("required"),
        Some(Value::String(value)) => format!("`\"{value}\"`"),
        Some(Value::Bool(value)) => format!("`{value}`"),
        Some(Value::Number(value)) => format!("`{}`", format_number(value)),
        Some(Value::Array(values)) if values.is_empty() => String::from("`[]`"),
        Some(Value::Array(_)) => String::from("`[...]`"),
        Some(Value::Object(entries)) if entries.is_empty() => String::from("`{}`"),
        Some(Value::Object(_)) => String::from("`{...}`"),
        Some(Value::Null) => String::from("`null`"),
    }
}

/// Formats a JSON number at its natural precision. Detects values that
/// originated as `f32` (via schemars' f32-to-f64 widening) so
/// `0.3499999940395355` reads as `0.35`.
fn format_number(number: &serde_json::Number) -> String {
    if let Some(integer_value) = number.as_i64() {
        return integer_value.to_string();
    }
    if let Some(unsigned_value) = number.as_u64() {
        return unsigned_value.to_string();
    }

    let Some(float_value) = number.as_f64() else {
        return number.to_string();
    };

    let narrowed = float_value as f32;
    if (narrowed as f64 - float_value).abs() < f64::EPSILON * 64.0 {
        return format!("{narrowed}");
    }

    format!("{float_value}")
}

fn render_type_cell(field_schema: &Value, known_types: &BTreeSet<String>) -> String {
    if let Some(type_name) = extract_ref_type_name(field_schema) {
        return render_type_link(&type_name, known_types);
    }

    let schema_type = field_schema
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let format_hint = field_schema.get("format").and_then(Value::as_str);

    match primitive_name(schema_type, format_hint) {
        Some(primitive) => String::from(primitive),
        None => String::from(schema_type),
    }
}

/// Renders a schema-defined type name as either an anchor link (if it's on the
/// types page) or plain code. Handles schemars' synthetic wrappers for
/// `Vec<T>`, `Option<T>`, and `HashMap<String, T>` recursively.
fn render_type_link(type_name: &str, known_types: &BTreeSet<String>) -> String {
    if let Some(primitive) = primitive_name_from_alias(type_name) {
        return String::from(primitive);
    }

    if let Some(inner) = type_name.strip_prefix("Array_of_") {
        return format!("array of {}", render_type_link(inner, known_types));
    }
    if let Some(inner) = type_name.strip_prefix("Nullable_") {
        return format!("{} or null", render_type_link(inner, known_types));
    }
    if let Some(inner) = type_name.strip_prefix("Map_of_") {
        return format!("map of {}", render_type_link(inner, known_types));
    }

    if known_types.contains(type_name) {
        let slug = type_slug(type_name);
        return format!("[`{type_name}`](/config/types#{slug})");
    }

    format!("`{type_name}`")
}

/// Rust name for a JSON Schema `type` + `format` pair, if the combination is
/// one the generator recognises.
fn primitive_name(schema_type: &str, format_hint: Option<&str>) -> Option<&'static str> {
    match (schema_type, format_hint) {
        ("boolean", _) => Some("bool"),
        ("string", _) => Some("string"),
        ("integer", Some("uint32")) => Some("u32"),
        ("integer", Some("int32")) => Some("i32"),
        ("integer", Some("uint64")) => Some("u64"),
        ("integer", Some("int64")) => Some("i64"),
        ("integer", Some("uint")) => Some("usize"),
        ("integer", Some("int")) => Some("isize"),
        ("integer", _) => Some("integer"),
        ("number", Some("float")) => Some("f32"),
        ("number", Some("double")) => Some("f64"),
        ("number", _) => Some("number"),
        ("array", _) => Some("array"),
        ("object", _) => Some("object"),
        _ => None,
    }
}

/// Rust name for a schemars primitive alias (`"uint32"`, `"double"`, etc.) if
/// the name is one of schemars' shared $defs for stdlib primitives.
fn primitive_name_from_alias(type_name: &str) -> Option<&'static str> {
    match type_name {
        "boolean" => Some("bool"),
        "string" => Some("string"),
        "uint8" => Some("u8"),
        "uint16" => Some("u16"),
        "uint32" => Some("u32"),
        "uint64" => Some("u64"),
        "int8" => Some("i8"),
        "int16" => Some("i16"),
        "int32" => Some("i32"),
        "int64" => Some("i64"),
        "uint" => Some("usize"),
        "int" => Some("isize"),
        "float" => Some("f32"),
        "double" => Some("f64"),
        _ => None,
    }
}

/// Assigns each field to the first group whose rule matches it, checked in
/// the declared group order. Anything unclaimed goes to the catch-all group
/// if one exists, otherwise it's dropped.
fn bucket_fields<'a>(fields: &'a [Field<'a>], groups: &[ConfigGroup]) -> Vec<Vec<&'a Field<'a>>> {
    let catchall_index = groups
        .iter()
        .position(|group| matches!(group.rule, GroupRule::CatchAll));

    let mut buckets: Vec<Vec<&Field<'_>>> = vec![Vec::new(); groups.len()];

    for field in fields {
        let matched_index = groups.iter().enumerate().find_map(|(index, group)| {
            if Some(index) == catchall_index {
                return None;
            }
            rule_matches(&group.rule, field).then_some(index)
        });

        let target_index = matched_index.or(catchall_index);
        if let Some(index) = target_index {
            buckets[index].push(field);
        }
    }

    buckets
}

fn rule_matches(rule: &GroupRule, field: &Field<'_>) -> bool {
    match rule {
        GroupRule::CatchAll => false,
        GroupRule::Prefix(prefix) => field.name.starts_with(prefix),
        GroupRule::Standalone(target) => field.name == *target,
        GroupRule::ByType(tag) => field.schema_type_name.as_deref() == Some(tag.schema_name()),
    }
}

fn render_frontmatter(page: &mut String, name: &str) {
    page.push_str("---\n");
    page.push_str(&format!("title: {name}\n"));
    page.push_str("outline: [2, 3]\n");
    page.push_str("---\n\n");
    page.push_str(&format!("# {name}\n\n"));
}

fn render_hook(page: &mut String, name: &str, schema: &Value) {
    let description = schema
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or(name);

    page.push_str(&rustdoc::rehost_rustdoc(description.trim(), PAGE_H1_DEPTH));
    page.push('\n');
}

fn render_layout_snippet(page: &mut String, layout_id: Option<&str>) {
    let Some(id) = layout_id else { return };

    page.push_str(&format!("Add it to your layout with `{id}`:\n\n"));
    page.push_str("```toml\n");
    page.push_str("[[bar.layout]]\n");
    page.push_str("monitor = \"*\"\n");
    page.push_str(&format!("right = [\"{id}\"]\n"));
    page.push_str("```\n\n");
}

fn render_group(page: &mut String, group: &ConfigGroup, fields: &[&Field<'_>]) {
    page.push_str(&format!("## {}\n\n", group.title));
    page.push_str("| Field | Type | Default | Description |\n");
    page.push_str("|---|---|---|---|\n");

    for field in fields {
        page.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            field.name,
            field.type_repr,
            field.default_repr,
            escape_table_cell(&summary_line(field.description)),
        ));
    }

    page.push('\n');

    for field in fields {
        if has_rich_body(field.description) {
            render_rich_subsection(page, field);
        }
    }
}

fn render_rich_subsection(page: &mut String, field: &Field<'_>) {
    let body = body_after_summary(field.description);
    if body.trim().is_empty() {
        return;
    }

    page.push_str(&format!("::: details More about `{}`\n\n", field.name));
    page.push_str(&rustdoc::rehost_rustdoc(
        body.trim(),
        FIELD_SUBSECTION_DEPTH,
    ));
    page.push_str("\n:::\n\n");
}

fn render_default_toml(page: &mut String, entry: &ModuleEntry, schema: &Value) {
    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return;
    };
    if properties.is_empty() {
        return;
    }

    let mut section = toml::Table::new();
    let mut required_fields: Vec<&str> = Vec::new();

    for (key, field_schema) in properties {
        let Some(default) = field_schema.get("default") else {
            required_fields.push(key);
            continue;
        };
        if default.is_null() {
            continue;
        }
        match serde_json::from_value::<toml::Value>(default.clone()) {
            Ok(value) => {
                section.insert(key.clone(), value);
            }
            Err(err) => {
                warn!(
                    module = %entry.info.name,
                    field = %key,
                    error = %err,
                    "default value could not be serialised as TOML; omitting from defaults block",
                );
            }
        }
    }

    if section.is_empty() && required_fields.is_empty() {
        return;
    }

    page.push_str("## Default configuration\n\n");

    if !required_fields.is_empty() {
        let list = required_fields
            .iter()
            .map(|field| format!("`{field}`"))
            .collect::<Vec<_>>()
            .join(", ");
        page.push_str(&format!(
            "Required fields (must be set in your config): {list}.\n\n",
        ));
    }

    if section.is_empty() {
        return;
    }

    let root = build_default_root(entry, section);
    let rendered = match toml::to_string_pretty(&root) {
        Ok(text) => text,
        Err(err) => {
            warn!(
                module = %entry.info.name,
                error = %err,
                "defaults block serialisation failed; omitting",
            );
            return;
        }
    };

    page.push_str("```toml\n");
    page.push_str(&rendered);
    if !rendered.ends_with('\n') {
        page.push('\n');
    }
    page.push_str("```\n\n");
}

/// Picks the TOML shape for the defaults block: `[modules.<name>]` for bar
/// modules, `[[modules.<name>]]` for array-of-tables modules (like custom),
/// and `[<name>]` for top-level schemas.
fn build_default_root(entry: &ModuleEntry, section: toml::Table) -> toml::Table {
    let mut root = toml::Table::new();

    if entry.info.array_entry {
        let entries = toml::Value::Array(vec![toml::Value::Table(section)]);
        let mut modules_table = toml::Table::new();
        modules_table.insert(entry.info.name.clone(), entries);
        root.insert(String::from("modules"), toml::Value::Table(modules_table));
    } else if entry.info.layout_id.is_some() {
        let mut modules_table = toml::Table::new();
        modules_table.insert(entry.info.name.clone(), toml::Value::Table(section));
        root.insert(String::from("modules"), toml::Value::Table(modules_table));
    } else {
        root.insert(entry.info.name.clone(), toml::Value::Table(section));
    }

    root
}

/// Escapes the one character that breaks markdown table rows. Backslashes and
/// other prose pass through cleanly because each page is wrapped in
/// `<div v-pre>`.
fn escape_table_cell(text: &str) -> String {
    text.replace('|', "\\|")
}

/// Joins the first paragraph of `description` into a single line. Soft wraps
/// in rustdoc collapse into one sentence for the table summary; paragraphs are
/// bounded by blank lines.
fn summary_line(description: &str) -> String {
    description
        .lines()
        .skip_while(|line| line.trim().is_empty())
        .take_while(|line| !line.trim().is_empty())
        .map(str::trim)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Preserves line breaks of every paragraph past the first one.
fn body_after_summary(description: &str) -> String {
    description
        .lines()
        .skip_while(|line| line.trim().is_empty())
        .skip_while(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// `true` when `description` has a second paragraph (anything beyond the
/// summary line).
fn has_rich_body(description: &str) -> bool {
    description
        .lines()
        .skip_while(|line| line.trim().is_empty())
        .skip_while(|line| !line.trim().is_empty())
        .any(|line| !line.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_line_joins_soft_wraps_in_first_paragraph() {
        let input = "Temperature sensor label. Use `\"auto\"` for automatic detection,\nor specify a label.\nRun `sensors` to see labels.\n\nSecond paragraph.";
        assert_eq!(
            summary_line(input),
            "Temperature sensor label. Use `\"auto\"` for automatic detection, or specify a label. Run `sensors` to see labels.",
        );
    }

    #[test]
    fn summary_line_skips_blank_prefix() {
        assert_eq!(summary_line("\n\n  hello\nworld"), "hello world");
    }

    #[test]
    fn summary_line_empty_input() {
        assert_eq!(summary_line(""), "");
        assert_eq!(summary_line("   \n  "), "");
    }

    #[test]
    fn has_rich_body_only_detects_paragraph_after_first() {
        assert!(!has_rich_body("single paragraph\nwith a soft wrap"));
        assert!(!has_rich_body("just one line"));
        assert!(!has_rich_body("just one line\n"));
        assert!(!has_rich_body("just one line\n   "));
    }

    #[test]
    fn has_rich_body_detects_second_paragraph() {
        assert!(has_rich_body("summary\n\nmore"));
        assert!(has_rich_body("summary\n\n- bullet"));
        assert!(has_rich_body(
            "summary line one\nsoft wrap\n\nsecond paragraph"
        ));
    }

    #[test]
    fn body_after_summary_keeps_content_past_first_paragraph() {
        let input = "summary line one\nsoft wrap\n\n## Examples\n\n- item";
        assert_eq!(body_after_summary(input), "\n## Examples\n\n- item",);
    }
}
