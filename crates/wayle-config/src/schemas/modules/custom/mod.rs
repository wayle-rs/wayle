mod types;

use std::collections::HashMap;

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

pub use self::types::{ExecutionMode, RestartDelay, RestartPolicy};
use crate::{
    docs::{ConfigGroup, ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken},
};

/// User-defined module that runs a shell command and renders the output in the bar.
///
/// Full walkthrough with examples at <https://wayle.dev/guide/custom-modules>.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct CustomModuleDefinition {
    /// Unique identifier for this module.
    ///
    /// Referenced in bar layouts as `custom-<id>`. Must be unique across
    /// all custom module definitions.
    ///
    /// ## Example
    ///
    /// ```toml
    /// [[modules.custom]]
    /// id = "gpu-temp"
    ///
    /// # Reference in layout:
    /// # layout = ["custom-gpu-temp", "clock"]
    /// ```
    pub id: String,

    /// Shell command to execute.
    ///
    /// The command runs via `sh -c` and should output to stdout.
    /// Stderr is discarded. Commands have a 30-second timeout.
    ///
    /// ## Output Parsing
    ///
    /// - If output starts with `{` or `[`: parsed as JSON
    /// - Otherwise: treated as plain text
    ///
    /// ## Behavior by Mode
    ///
    /// - **poll**: Executed every `interval-ms` milliseconds
    /// - **watch**: Spawned once, each stdout line triggers a display update.
    ///   Restarts are controlled by `restart-policy`.
    #[serde(default)]
    pub command: Option<String>,

    /// Execution mode for the command.
    ///
    /// | Mode | Behavior |
    /// |------|----------|
    /// | `poll` | Run command every `interval-ms` (default) |
    /// | `watch` | Spawn long-running process, update on each stdout line |
    ///
    /// Use `poll` for commands that return current state and exit.
    /// Use `watch` for commands that stream updates (e.g., `pactl subscribe`).
    #[serde(default)]
    pub mode: ExecutionMode,

    /// Polling interval in milliseconds.
    ///
    /// Only applies to `poll` mode. Ignored in `watch` mode.
    ///
    /// Set to `0` for manual polling mode: no timer is started. In manual
    /// mode, the command still runs once at startup.
    #[serde(rename = "interval-ms", default = "default_interval")]
    pub interval_ms: u64,

    /// Restart policy for watch mode.
    ///
    /// Only applies to `watch` mode. Ignored in `poll` mode.
    ///
    /// | Policy | Behavior |
    /// |--------|----------|
    /// | `never` | Do not restart after exit |
    /// | `on-exit` | Restart after any exit |
    /// | `on-failure` | Restart only after non-zero/signal exit |
    #[serde(rename = "restart-policy", default)]
    pub restart_policy: RestartPolicy,

    /// Base restart delay in milliseconds for watch mode.
    ///
    /// Only applies to `watch` mode. Ignored in `poll` mode.
    ///
    /// Used when `restart-policy` is `on-exit` or `on-failure`.
    /// Delay increases exponentially on rapid failures, capped at 30 seconds.
    #[serde(rename = "restart-interval-ms", default)]
    pub restart_interval_ms: RestartDelay,

    /// Format string for the label using Jinja2 template syntax.
    ///
    /// ## Variables
    ///
    /// - `{{ output }}` - Raw command output
    /// - `{{ field }}` - JSON field access
    /// - `{{ nested.field }}` - Nested field access
    /// - `{{ items.0 }}` - Array index access
    ///
    /// ## Filters
    ///
    /// - `{{ val | default('fallback') }}` - Fallback for missing values
    /// - `{{ "%02d" | format(val) }}` - Zero-padding
    /// - `{{ val | upper }}`, `| lower`, `| trim` - String transforms
    ///
    /// ## Examples
    ///
    /// - `"{{ output }}°C"` - Plain text: "72°C"
    /// - `"{{ percentage }}%"` - JSON field: "75%"
    /// - `"{{ data.temp }}°C"` - Nested: "22°C"
    ///
    /// If JSON output contains a `text` field, it overrides this format.
    #[serde(default = "default_format")]
    pub format: String,

    /// Format string for the tooltip (hover text).
    ///
    /// Supports the same Jinja2 syntax as `format`. If not set, no tooltip is shown.
    /// If JSON output contains a `tooltip` field, it overrides this format.
    ///
    /// ## Example
    ///
    /// ```toml
    /// format = "{{ percentage }}%"
    /// tooltip-format = "Volume: {{ percentage }}% on {{ device }}"
    /// ```
    #[serde(rename = "tooltip-format", default)]
    pub tooltip_format: Option<String>,

    /// Hide module when output is empty, "0", or "false".
    ///
    /// When enabled, the module (including its gap in the bar layout) is
    /// completely hidden if the output indicates an empty/disabled state.
    #[serde(rename = "hide-if-empty", default)]
    pub hide_if_empty: bool,

    /// Static symbolic icon name.
    ///
    /// Used when `icon-names` and `icon-map` don't provide a match.
    /// Should be a symbolic icon name from the icon theme (e.g., `"ld-gpu-symbolic"`).
    ///
    /// ## Example
    ///
    /// ```toml
    /// icon-name = "ld-temperature-symbolic"
    /// ```
    #[serde(rename = "icon-name", default)]
    pub icon_name: String,

    /// Array of icon names indexed by percentage (0-100).
    ///
    /// Requires JSON output with a `percentage` field (0-100).
    /// The array is divided evenly across the percentage range.
    ///
    /// ## Resolution
    ///
    /// For N icons, icon at index `floor(percentage * N / 101)` is selected:
    ///
    /// - 4 icons: 0-24% → [0], 25-49% → [1], 50-74% → [2], 75-100% → [3]
    /// - 5 icons: 0-19% → [0], 20-39% → [1], 40-59% → [2], 60-79% → [3], 80-100% → [4]
    ///
    /// ## Example
    ///
    /// ```toml
    /// icon-names = [
    ///   "battery-empty-symbolic",
    ///   "battery-caution-symbolic",
    ///   "battery-low-symbolic",
    ///   "battery-good-symbolic",
    ///   "battery-full-symbolic"
    /// ]
    /// ```
    #[serde(rename = "icon-names", default)]
    pub icon_names: Option<Vec<String>>,

    /// Map of icon names keyed by the `alt` field value.
    ///
    /// Requires JSON output with an `alt` field. The `alt` value is looked up
    /// in this map. Use `"default"` as a fallback key.
    ///
    /// **Priority**: `icon-map[alt]` takes precedence over `icon-names[percentage]`,
    /// allowing state-specific icons to override percentage-based icons.
    ///
    /// ## Example
    ///
    /// ```toml
    /// # Volume with muted state override
    /// icon-names = ["vol-0", "vol-33", "vol-66", "vol-100"]
    /// icon-map = { "muted" = "audio-volume-muted-symbolic" }
    ///
    /// # Output: {"percentage": 50, "alt": "muted"}
    /// # Result: Uses "audio-volume-muted-symbolic" (alt match beats percentage)
    ///
    /// # Output: {"percentage": 50}
    /// # Result: Uses "vol-33" (percentage-based, no alt)
    /// ```
    #[serde(rename = "icon-map", default)]
    pub icon_map: Option<HashMap<String, String>>,

    /// Format string for dynamic CSS classes.
    ///
    /// Supports the same Jinja2 syntax as `format`. The formatted result is
    /// split on whitespace and each word is added as a CSS class.
    ///
    /// Combined with the `class` field from JSON output (if present).
    ///
    /// ## Example
    ///
    /// ```toml
    /// class-format = "volume-{{ alt }}"
    /// # Output: {"alt": "muted"} → adds class "volume-muted"
    /// ```
    #[serde(rename = "class-format", default)]
    pub class_format: Option<String>,

    /// Display module icon.
    #[serde(rename = "icon-show", default = "default_true")]
    pub icon_show: bool,

    /// Icon foreground color.
    #[serde(rename = "icon-color", default = "default_auto_color")]
    pub icon_color: ColorValue,

    /// Icon container background color.
    #[serde(rename = "icon-bg-color", default = "default_auto_color")]
    pub icon_bg_color: ColorValue,

    /// Display text label.
    #[serde(rename = "label-show", default = "default_true")]
    pub label_show: bool,

    /// Label text color.
    #[serde(rename = "label-color", default = "default_auto_color")]
    pub label_color: ColorValue,

    /// Maximum label length in characters before truncation.
    ///
    /// When exceeded, label is truncated with ellipsis. Set to `0` to disable.
    #[serde(rename = "label-max-length", default)]
    pub label_max_length: u32,

    /// Button background color.
    #[serde(rename = "button-bg-color", default = "default_button_bg")]
    pub button_bg_color: ColorValue,

    /// Display border around button.
    #[serde(rename = "border-show", default)]
    pub border_show: bool,

    /// Border color.
    #[serde(rename = "border-color", default = "default_auto_color")]
    pub border_color: ColorValue,

    /// Shell command executed on left click.
    ///
    /// If `on-action` is set, it runs after this command completes.
    #[serde(rename = "left-click", default)]
    pub left_click: String,

    /// Shell command executed on right click.
    ///
    /// If `on-action` is set, it runs after this command completes.
    #[serde(rename = "right-click", default)]
    pub right_click: String,

    /// Shell command executed on middle click.
    ///
    /// If `on-action` is set, it runs after this command completes.
    #[serde(rename = "middle-click", default)]
    pub middle_click: String,

    /// Shell command executed on scroll up.
    ///
    /// Scroll events are debounced (50ms) to coalesce rapid scrolls.
    /// If `on-action` is set, it runs after this command completes.
    #[serde(rename = "scroll-up", default)]
    pub scroll_up: String,

    /// Shell command executed on scroll down.
    ///
    /// Scroll events are debounced (50ms) to coalesce rapid scrolls.
    /// If `on-action` is set, it runs after this command completes.
    #[serde(rename = "scroll-down", default)]
    pub scroll_down: String,

    /// Shell command to run after any click/scroll action completes.
    ///
    /// Executes after the action handler finishes, and its output updates
    /// the display immediately. Useful for reflecting state changes without
    /// waiting for the next poll interval.
    ///
    /// ## Example
    ///
    /// ```toml
    /// # Volume control with immediate feedback
    /// scroll-up = "pactl set-sink-volume @DEFAULT_SINK@ +5%"
    /// scroll-down = "pactl set-sink-volume @DEFAULT_SINK@ -5%"
    /// on-action = '''
    /// vol=$(pactl get-sink-volume @DEFAULT_SINK@ | grep -oP '\d+(?=%)' | head -1)
    /// echo "{\"percentage\": $vol}"
    /// '''
    /// ```
    #[serde(rename = "on-action", default)]
    pub on_action: Option<String>,
}

fn default_interval() -> u64 {
    5000
}

fn default_format() -> String {
    String::from("{{ output }}")
}

fn default_true() -> bool {
    true
}

fn default_auto_color() -> ColorValue {
    ColorValue::Auto
}

fn default_button_bg() -> ColorValue {
    ColorValue::Token(CssToken::BgSurfaceElevated)
}

impl ModuleInfoProvider for CustomModuleDefinition {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("custom"),
            schema: || schema_for!(CustomModuleDefinition),
            layout_id: Some(String::from("custom-<id>")),
            array_entry: true,
        }
    }

    fn groups() -> Vec<ConfigGroup> {
        vec![
            ConfigGroup::general(),
            ConfigGroup::colors(),
            ConfigGroup::click(),
            ConfigGroup::prefix("Icons", "icon-"),
            ConfigGroup::prefix("Restart", "restart-"),
        ]
    }
}

crate::register_module!(CustomModuleDefinition);
