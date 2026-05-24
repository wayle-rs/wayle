//! Emits `config/types.md`: one H2 section per named schema type referenced
//! anywhere in the config. Types are collected from the union of `$defs`
//! across every registered module, so shared types like `ColorValue` get a
//! single section no matter how many modules reference them.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;
use tracing::warn;

use super::{registry::ModuleEntry, rustdoc};

/// Heading depth of each type section on the page.
const TYPE_SECTION_DEPTH: usize = 2;

/// Converts a CamelCase type name to a kebab-case anchor slug.
pub fn type_slug(name: &str) -> String {
    let mut slug = String::with_capacity(name.len() + 4);
    for (index, character) in name.char_indices() {
        if character.is_ascii_uppercase() && index > 0 {
            slug.push('-');
        }
        for lowered in character.to_lowercase() {
            slug.push(lowered);
        }
    }
    slug
}

/// Merges every module's `$defs` into a single map keyed by type name.
///
/// Skips schemars' synthetic wrappers (`Array_of_*`, `Nullable_*`, `Map_of_*`)
/// and its shared primitive refs, since those render inline on the field
/// rather than as their own type section.
pub fn collect_type_defs(modules: &[ModuleEntry]) -> BTreeMap<String, Value> {
    let mut defs: BTreeMap<String, Value> = BTreeMap::new();

    for entry in modules {
        let schema = match serde_json::to_value((entry.info.schema)()) {
            Ok(schema) => schema,
            Err(err) => {
                warn!(
                    module = %entry.info.name,
                    error = %err,
                    "schema serialization failed; types page will skip this module's $defs",
                );
                continue;
            }
        };

        let Some(module_defs) = schema.get("$defs").and_then(Value::as_object) else {
            continue;
        };

        for (type_name, definition) in module_defs {
            if is_synthetic_wrapper(type_name) {
                continue;
            }
            defs.entry(type_name.clone())
                .or_insert_with(|| definition.clone());
        }
    }

    defs
}

/// Set of type names the generator will emit anchors for.
pub fn known_type_names(defs: &BTreeMap<String, Value>) -> BTreeSet<String> {
    defs.keys().cloned().collect()
}

/// Full text of `config/types.md`.
pub fn render_types_page(defs: &BTreeMap<String, Value>) -> String {
    let mut page = String::new();

    page.push_str("---\ntitle: Types\noutline: [2, 3]\n---\n\n");
    page.push_str("<div v-pre>\n\n");
    page.push_str("# Types\n\n");
    page.push_str(
        "Named types referenced across the config. Every field in [`/config/`](/config/) that shows a type like `Color` or `ClickAction` links here.\n\n",
    );

    for (type_name, definition) in defs {
        page.push_str(&render_type_section(type_name, definition));
    }

    page.push_str("\n</div>\n");
    page
}

fn render_type_section(type_name: &str, definition: &Value) -> String {
    let slug = type_slug(type_name);
    let mut section = format!("## {type_name} {{#{slug}}}\n\n");

    append_description(&mut section, definition);

    if let Some(body) = render_type_body(definition) {
        section.push_str(&body);
        return section;
    }

    if description_has_structured_content(description_of(definition)) {
        return section;
    }

    section.push_str("See the schema for valid values.\n\n");
    section
}

fn append_description(section: &mut String, definition: &Value) {
    let Some(description) = definition.get("description").and_then(Value::as_str) else {
        return;
    };
    let trimmed = description.trim();
    if trimmed.is_empty() {
        return;
    }
    section.push_str(&rustdoc::rehost_rustdoc(trimmed, TYPE_SECTION_DEPTH));
    section.push('\n');
}

/// Picks the first type-body renderer that matches `definition`'s shape.
fn render_type_body(definition: &Value) -> Option<String> {
    render_any_of_branches(definition)
        .or_else(|| render_one_of_variants(definition))
        .or_else(|| render_enum_values(definition))
        .or_else(|| render_numeric_range(definition))
        .or_else(|| render_string_pattern(definition))
        .or_else(|| render_object_shape(definition))
}

fn description_of(definition: &Value) -> &str {
    definition
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or_default()
}

/// `true` for `$def` names schemars synthesises around stdlib types.
///
/// These are never user-facing: `Array_of_T` stands in for `Vec<T>`,
/// `Nullable_T` for `Option<T>`, `Map_of_T` for `HashMap<String, T>`, and the
/// lowercase primitive names (`boolean`, `integer`, `number`, `string`) are
/// schemars' shared refs for Rust primitives.
fn is_synthetic_wrapper(type_name: &str) -> bool {
    if type_name.starts_with("Array_of_")
        || type_name.starts_with("Nullable_")
        || type_name.starts_with("Map_of_")
    {
        return true;
    }

    matches!(
        type_name,
        "boolean"
            | "integer"
            | "number"
            | "string"
            | "int"
            | "int8"
            | "int16"
            | "int32"
            | "int64"
            | "uint"
            | "uint8"
            | "uint16"
            | "uint32"
            | "uint64"
            | "float"
            | "double"
    )
}

/// `true` when `description` already renders enough structure to stand on
/// its own. Any ATX heading, fenced code block, bullet list, or table row
/// qualifies. Used to suppress the "See the schema for valid values"
/// fallback when the rustdoc already explains the type.
fn description_has_structured_content(description: &str) -> bool {
    for line in description.lines() {
        if rustdoc::is_fence_line(line) {
            return true;
        }

        let trimmed = line.trim_start();
        if trimmed.starts_with('#')
            || trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("| ")
        {
            return true;
        }
    }

    false
}

/// Renders an `anyOf` union by dispatching each branch to the renderer that
/// fits its shape and concatenating the results.
fn render_any_of_branches(definition: &Value) -> Option<String> {
    let branches = definition.get("anyOf")?.as_array()?;
    if branches.is_empty() {
        return None;
    }

    let mut output = String::new();
    for branch in branches {
        let rendered = render_one_of_variants(branch)
            .or_else(|| render_enum_values(branch))
            .or_else(|| render_numeric_range(branch))
            .or_else(|| render_string_pattern(branch))
            .or_else(|| render_object_shape(branch));
        if let Some(text) = rendered {
            output.push_str(&text);
        }
    }

    (!output.is_empty()).then_some(output)
}

/// Renders a `oneOf` array as a variant table.
///
/// Variants without a `const` value (`$ref` branches, for example) are
/// skipped rather than discarding the whole table; a mixed `oneOf`
/// shouldn't erase the rows we could otherwise render.
fn render_one_of_variants(definition: &Value) -> Option<String> {
    let variants = definition.get("oneOf")?.as_array()?;
    if variants.is_empty() {
        return None;
    }

    let mut table = String::from("| Value | Meaning |\n|---|---|\n");
    let mut any_row = false;

    for variant in variants {
        let Some(value) = variant.get("const") else {
            continue;
        };
        let description = variant
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("");
        table.push_str(&format!(
            "| `{}` | {} |\n",
            render_json_literal(value),
            description,
        ));
        any_row = true;
    }

    if any_row {
        table.push('\n');
        Some(table)
    } else {
        None
    }
}

/// Renders a plain `enum` array as a comma-separated list.
fn render_enum_values(definition: &Value) -> Option<String> {
    let values = definition.get("enum")?.as_array()?;
    if values.is_empty() {
        return None;
    }

    let joined = values
        .iter()
        .map(render_json_literal)
        .map(|literal| format!("`{literal}`"))
        .collect::<Vec<_>>()
        .join(", ");

    Some(format!("One of: {joined}.\n\n"))
}

/// Renders a numeric type with optional min/max constraints.
fn render_numeric_range(definition: &Value) -> Option<String> {
    let schema_type = definition.get("type").and_then(Value::as_str)?;
    if schema_type != "integer" && schema_type != "number" {
        return None;
    }

    let minimum = definition.get("minimum");
    let maximum = definition.get("maximum");

    let mut block = match (minimum, maximum) {
        (Some(low), Some(high)) => format!(
            "Number in `[{}, {}]`.\n\n",
            render_json_literal(low),
            render_json_literal(high),
        ),
        (Some(low), None) => format!("Number `>= {}`.\n\n", render_json_literal(low)),
        (None, Some(high)) => format!("Number `<= {}`.\n\n", render_json_literal(high)),
        (None, None) => String::from("Any number.\n\n"),
    };

    if let Some(format_hint) = definition.get("format").and_then(Value::as_str) {
        block.push_str(&format!("Serialises as `{format_hint}`.\n\n"));
    }

    Some(block)
}

/// Renders a string type with optional regex constraint.
fn render_string_pattern(definition: &Value) -> Option<String> {
    if definition.get("type").and_then(Value::as_str)? != "string" {
        return None;
    }

    let mut block = String::from("String");
    if let Some(pattern) = definition.get("pattern").and_then(Value::as_str) {
        block.push_str(&format!(" matching `{pattern}`"));
    }
    block.push_str(".\n\n");

    Some(block)
}

/// Renders a nested-object type as a compact field table.
fn render_object_shape(definition: &Value) -> Option<String> {
    if definition.get("type").and_then(Value::as_str)? != "object" {
        return None;
    }

    let properties = definition.get("properties").and_then(Value::as_object)?;
    if properties.is_empty() {
        return None;
    }

    let mut table = String::from("| Field | Description |\n|---|---|\n");
    for (field_name, field_schema) in properties {
        let summary = field_schema
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("")
            .lines()
            .next()
            .unwrap_or("")
            .trim();
        table.push_str(&format!("| `{field_name}` | {summary} |\n"));
    }
    table.push('\n');

    Some(table)
}

fn render_json_literal(value: &Value) -> String {
    match value {
        Value::String(text) => format!("\"{text}\""),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::Null => String::from("null"),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_slug_camel_to_kebab() {
        assert_eq!(type_slug("Color"), "color");
        assert_eq!(type_slug("ColorValue"), "color-value");
        assert_eq!(type_slug("ClickAction"), "click-action");
        assert_eq!(type_slug("ScaleFactor"), "scale-factor");
        assert_eq!(type_slug("URL"), "u-r-l");
    }

    #[test]
    fn type_slug_preserves_single_word() {
        assert_eq!(type_slug("spacing"), "spacing");
        assert_eq!(type_slug("Spacing"), "spacing");
    }
}
