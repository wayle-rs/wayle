//! Emits `config/index.md`: the landing page that links every generated
//! reference page and groups them by top-level vs bar module.

use serde_json::Value;
use tracing::warn;

use super::registry::ModuleEntry;

/// One row in an index-page table: the module's name and its hook sentence.
struct IndexRow {
    name: String,
    hook: String,
}

/// Index-page rows partitioned by where the schema lives in the config.
struct IndexRows {
    top_level: Vec<IndexRow>,
    bar_modules: Vec<IndexRow>,
}

/// Full text of `config/index.md`.
pub fn render_config_index(modules: &[ModuleEntry]) -> String {
    let rows = partition_rows(modules);

    let mut page = String::new();

    page.push_str("---\ntitle: Config reference\noutline: [2]\n---\n\n");
    page.push_str("<div v-pre>\n\n");
    page.push_str("# Config reference\n\n");
    page.push_str(
        "Every config file lives at `~/.config/wayle/config.toml`. Each page below covers one section. Every field has a default; start with an empty file and add only what you want to change.\n\n",
    );
    page.push_str(
        "::: tip\nEditor intellisense via JSON Schema. Install [Tombi](https://marketplace.visualstudio.com/items?itemName=tombi-toml.tombi) for VSCode or the `tombi` LSP for Neovim, Helix, or Zed. The schema is written to `~/.config/wayle/schema.json` on startup.\n:::\n\n",
    );

    page.push_str("## Top-level sections\n\n");
    page.push_str("| Section | What it controls |\n|---|---|\n");
    for row in &rows.top_level {
        page.push_str(&format!(
            "| [`{}`](/config/{}) | {} |\n",
            row.name, row.name, row.hook,
        ));
    }
    page.push('\n');

    page.push_str("## Bar modules\n\n");
    page.push_str(
        "Modules appear inside `[[bar.layout]]` arrays. Each row links to the full reference.\n\n",
    );
    page.push_str("| Module | Purpose |\n|---|---|\n");
    for row in &rows.bar_modules {
        page.push_str(&format!(
            "| [`{}`](/config/modules/{}) | {} |\n",
            row.name, row.name, row.hook,
        ));
    }
    page.push('\n');

    page.push_str("## Shared types\n\n");
    page.push_str(
        "Every named type referenced across the config (`Color`, `ClickAction`, `Spacing`, and others) is documented on the [types page](/config/types).\n",
    );

    page.push_str("\n</div>\n");
    page
}

/// Splits modules into top-level schemas and bar modules, preserving the
/// input's alphabetical order within each partition.
fn partition_rows(modules: &[ModuleEntry]) -> IndexRows {
    let mut rows = IndexRows {
        top_level: Vec::new(),
        bar_modules: Vec::new(),
    };

    for entry in modules {
        let row = IndexRow {
            name: entry.info.name.clone(),
            hook: module_hook(entry),
        };

        if entry.info.layout_id.is_some() {
            rows.bar_modules.push(row);
        } else {
            rows.top_level.push(row);
        }
    }

    rows
}

/// First non-empty line of the schema's top-level description, used as a
/// one-line hook in the index table.
fn module_hook(entry: &ModuleEntry) -> String {
    let schema = match serde_json::to_value((entry.info.schema)()) {
        Ok(schema) => schema,
        Err(err) => {
            warn!(
                module = %entry.info.name,
                error = %err,
                "schema serialization failed; index entry will have an empty hook",
            );
            return String::new();
        }
    };

    schema
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("")
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
        .trim()
        .to_string()
}
