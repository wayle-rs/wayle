//! Niri workspace switcher configuration.

use std::{collections::HashMap, ops::Deref};

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use wayle_derive::{wayle_config, wayle_enum};

use super::{ActiveIndicator, DisplayMode, UrgentMode, WorkspaceStyle};
use crate::{
    ConfigProperty,
    docs::{ConfigGroup, GroupDefaults, ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken, ScaleFactor, Spacing},
};

/// What identifies each workspace's label.
#[wayle_enum(default)]
pub enum LabelStrategy {
    /// Always show the index (e.g. `1`, `2`, `3`).
    Index,
    /// Show the name when set, fall back to the index.
    #[default]
    NameOrIndex,
    /// Show only the name; unnamed workspaces show nothing.
    NameOnly,
    /// Show both, joined as `"1: web"`. Unnamed workspaces show the index alone.
    IndexAndName,
}

/// Action wired to a click or scroll event on a workspace button.
///
/// Serializes to/from a string for TOML compatibility:
///
/// - `""` → [`None`](Self::None)
/// - `"focus:this"` → [`FocusWorkspace`](Self::FocusWorkspace)
/// - `"focus:next"` → [`FocusNext`](Self::FocusNext)
/// - `"focus:previous"` → [`FocusPrevious`](Self::FocusPrevious)
/// - `"focus:last"` → [`FocusLast`](Self::FocusLast)
/// - `"dropdown:NAME"` → [`Dropdown("NAME")`](Self::Dropdown)
/// - anything else → [`Shell(cmd)`](Self::Shell)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum WorkspaceClickAction {
    /// Do nothing.
    #[default]
    None,
    /// Focus the workspace under the cursor.
    FocusWorkspace,
    /// Focus the next workspace on this output.
    FocusNext,
    /// Focus the previous workspace on this output.
    FocusPrevious,
    /// Focus the last previously focused workspace (toggle).
    FocusLast,
    /// Open a named dropdown panel.
    Dropdown(String),
    /// Execute a shell command.
    Shell(String),
}

impl WorkspaceClickAction {
    fn from_str(value: &str) -> Self {
        if value.is_empty() {
            return Self::None;
        }
        match value {
            "focus:this" => Self::FocusWorkspace,
            "focus:next" => Self::FocusNext,
            "focus:previous" => Self::FocusPrevious,
            "focus:last" => Self::FocusLast,
            _ => match value.strip_prefix("dropdown:") {
                Some(name) => Self::Dropdown(name.to_owned()),
                None => Self::Shell(value.to_owned()),
            },
        }
    }
}

impl Serialize for WorkspaceClickAction {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::None => serializer.serialize_str(""),
            Self::FocusWorkspace => serializer.serialize_str("focus:this"),
            Self::FocusNext => serializer.serialize_str("focus:next"),
            Self::FocusPrevious => serializer.serialize_str("focus:previous"),
            Self::FocusLast => serializer.serialize_str("focus:last"),
            Self::Dropdown(name) => serializer.serialize_str(&format!("dropdown:{name}")),
            Self::Shell(cmd) => serializer.serialize_str(cmd),
        }
    }
}

impl<'de> Deserialize<'de> for WorkspaceClickAction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Ok(Self::from_str(&value))
    }
}

impl JsonSchema for WorkspaceClickAction {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("WorkspaceClickAction")
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "description": "Click/scroll action: focus:this | focus:next | focus:previous | focus:last | dropdown:NAME | shell command | empty for none",
            "type": "string"
        })
    }
}

/// Per-workspace icon and color overrides, keyed by workspace name or id-as-string.
///
/// Lookup tries the name first, then the id as a string. Workspaces that
/// match neither fall back to the defaults set by
/// [`NiriWorkspacesConfig::display_mode`] and the active/occupied/empty
/// color knobs.
///
/// ## Example
///
/// ```toml
/// [modules.niri-workspaces.workspace-map]
/// web = { icon = "ld-globe-symbolic", color = "#4a90d9" }
/// terminal = { icon = "ld-terminal-symbolic" }
///
/// # Or target by stable id when no name is set
/// [modules.niri-workspaces.workspace-map.5]
/// icon = "ld-code-symbolic"
/// color = "accent"
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct WorkspaceMap(HashMap<String, WorkspaceStyle>);

impl Deref for WorkspaceMap {
    type Target = HashMap<String, WorkspaceStyle>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> IntoIterator for &'a WorkspaceMap {
    type Item = (&'a String, &'a WorkspaceStyle);
    type IntoIter = std::collections::hash_map::Iter<'a, String, WorkspaceStyle>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// Niri workspace indicators with click-to-switch.
#[wayle_config(i18n_prefix = "settings-modules-niri-workspaces")]
pub struct NiriWorkspacesConfig {
    /// Always-visible ceiling for numerically-named workspaces.
    ///
    /// Workspaces whose name parses to an integer at or below this value
    /// stay visible even when empty. Workspaces with non-numeric names
    /// and those above the ceiling follow the normal empty/occupied
    /// filtering. When `0` (default), no extra workspaces are forced
    /// visible.
    #[serde(rename = "min-workspace-count")]
    #[default(0)]
    pub min_workspace_count: ConfigProperty<u8>,

    /// Show only workspaces on this bar's monitor.
    ///
    /// When `true` (default), each bar shows only its own output's
    /// workspaces. When `false`, all workspaces from every output are shown.
    #[serde(rename = "monitor-specific")]
    #[default(true)]
    pub monitor_specific: ConfigProperty<bool>,

    /// Hide niri's auto-allocated trailing empty workspace.
    ///
    /// Niri keeps one empty workspace at the tail of every output for
    /// dynamic allocation.
    #[serde(rename = "hide-trailing-empty")]
    #[default(true)]
    pub hide_trailing_empty: ConfigProperty<bool>,

    /// What identifies each workspace button.
    ///
    /// - `label` (default): show the workspace label per `label-strategy`
    /// - `icon`: show an icon from `workspace-map` (falls back to label if unmapped)
    /// - `none`: show nothing — only app icons visible (if enabled)
    #[serde(rename = "display-mode")]
    #[default(DisplayMode::Label)]
    pub display_mode: ConfigProperty<DisplayMode>,

    /// How to compose the workspace label when `display-mode = "label"`.
    ///
    /// - `index`: index only (`"1"`, `"2"`)
    /// - `name-or-index` (default): name when set, index otherwise
    /// - `name-only`: name only; unnamed workspaces show nothing
    /// - `index-and-name`: `"1: web"` form; unnamed workspaces show the index alone
    #[serde(rename = "label-strategy")]
    #[default(LabelStrategy::NameOrIndex)]
    pub label_strategy: ConfigProperty<LabelStrategy>,

    /// Pulse animation on workspaces with urgent windows.
    #[serde(rename = "urgent-show")]
    #[default(true)]
    pub urgent_show: ConfigProperty<bool>,

    /// Where the urgent pulse is applied.
    ///
    /// - `workspace` (default): whole button pulses
    /// - `application`: only the urgent app icon pulses, falling back to
    ///   `workspace` when app icons are disabled
    #[serde(rename = "urgent-mode")]
    #[default(UrgentMode::Workspace)]
    pub urgent_mode: ConfigProperty<UrgentMode>,

    /// Visual indicator for the active workspace.
    #[serde(rename = "active-indicator")]
    #[default(ActiveIndicator::Background)]
    pub active_indicator: ConfigProperty<ActiveIndicator>,

    /// Text separator between workspace identity and app icons.
    #[serde(rename = "divider")]
    #[default(String::from(" "))]
    pub divider: ConfigProperty<String>,

    /// Show application icons for windows on each workspace.
    #[serde(rename = "app-icons-show")]
    #[default(false)]
    pub app_icons_show: ConfigProperty<bool>,

    /// Deduplicate application icons within a workspace.
    ///
    /// When `true`, one icon per unique `app_id`. When `false`, one icon
    /// per window.
    #[serde(rename = "app-icons-dedupe")]
    #[default(true)]
    pub app_icons_dedupe: ConfigProperty<bool>,

    /// Fallback icon for applications not matched by `app-icon-map`.
    #[serde(rename = "app-icons-fallback")]
    #[default(String::from("ld-app-window-symbolic"))]
    pub app_icons_fallback: ConfigProperty<String>,

    /// Icon shown when a workspace has no application windows.
    #[serde(rename = "app-icons-empty")]
    #[default(String::from("tb-minus-symbolic"))]
    pub app_icons_empty: ConfigProperty<String>,

    /// Gap between app icons within a workspace button.
    #[serde(rename = "icon-gap")]
    #[default(Spacing::new(0.3))]
    pub icon_gap: ConfigProperty<Spacing>,

    /// Padding for workspace content along the bar direction.
    #[serde(rename = "workspace-padding")]
    #[default(Spacing::new(0.5))]
    pub workspace_padding: ConfigProperty<Spacing>,

    /// Scale multiplier for workspace icons.
    ///
    /// Applies to identity icons and custom icons from `workspace-map`.
    /// Range: 0.25-3.0.
    #[serde(rename = "icon-size")]
    #[default(ScaleFactor::default())]
    pub icon_size: ConfigProperty<ScaleFactor>,

    /// Scale multiplier for workspace labels and dividers. Range: 0.25-3.0.
    #[serde(rename = "label-size")]
    #[default(ScaleFactor::default())]
    pub label_size: ConfigProperty<ScaleFactor>,

    /// Workspaces to hide from the display.
    ///
    /// Glob patterns matched against the workspace's name (if set), then
    /// its index, then its stable id. Examples:
    /// - `"scratch"` — hide the workspace named `scratch`
    /// - `"1?"` — hide indices 10-19
    #[serde(rename = "workspace-ignore")]
    #[default(Vec::new())]
    pub workspace_ignore: ConfigProperty<Vec<String>>,

    /// Color for the active (visible on its output) workspace.
    ///
    /// In `background` indicator mode, also used as the button background.
    #[serde(rename = "active-color")]
    #[default(ColorValue::Token(CssToken::Accent))]
    pub active_color: ConfigProperty<ColorValue>,

    /// Color for occupied workspaces (have windows but not active).
    #[serde(rename = "occupied-color")]
    #[default(ColorValue::Token(CssToken::FgMuted))]
    pub occupied_color: ConfigProperty<ColorValue>,

    /// Color for empty workspaces and placeholder slots.
    #[serde(rename = "empty-color")]
    #[default(ColorValue::Token(CssToken::FgSubtle))]
    pub empty_color: ConfigProperty<ColorValue>,

    /// Background color for the workspaces container.
    #[serde(rename = "container-bg-color")]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub container_bg_color: ConfigProperty<ColorValue>,

    /// Display border around the workspaces container.
    #[serde(rename = "border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color for the workspaces container.
    #[serde(rename = "border-color")]
    #[default(ColorValue::Token(CssToken::BorderDefault))]
    pub border_color: ConfigProperty<ColorValue>,

    /// Per-workspace icon and color overrides, keyed by name or id-as-string.
    ///
    /// ## Example
    ///
    /// ```toml
    /// [modules.niri-workspaces.workspace-map]
    /// web = { icon = "ld-globe-symbolic", color = "#4a90d9" }
    /// terminal = { icon = "ld-terminal-symbolic" }
    /// ```
    #[serde(rename = "workspace-map")]
    #[default(WorkspaceMap::default())]
    pub workspace_map: ConfigProperty<WorkspaceMap>,

    /// Application icon mapping with glob pattern support.
    ///
    /// Maps window `app_id` or title to symbolic icon names. Supports:
    /// - No prefix: matches `app_id` (e.g. `"*firefox*"`)
    /// - `app:` prefix: explicit `app_id` match (e.g. `"app:org.mozilla.*"`)
    /// - `title:` prefix: matches window title (e.g. `"title:*YouTube*"`)
    ///
    /// ## Example
    ///
    /// ```toml
    /// [modules.niri-workspaces.app-icon-map]
    /// "*firefox*" = "ld-globe-symbolic"
    /// "title:*YouTube*" = "ld-youtube-symbolic"
    /// ```
    #[serde(rename = "app-icon-map")]
    #[default(HashMap::new())]
    pub app_icon_map: ConfigProperty<HashMap<String, String>>,

    /// Action on left click.
    #[serde(rename = "left-click")]
    #[default(WorkspaceClickAction::FocusWorkspace)]
    pub left_click: ConfigProperty<WorkspaceClickAction>,

    /// Action on middle click.
    #[serde(rename = "middle-click")]
    #[default(WorkspaceClickAction::None)]
    pub middle_click: ConfigProperty<WorkspaceClickAction>,

    /// Action on right click.
    #[serde(rename = "right-click")]
    #[default(WorkspaceClickAction::None)]
    pub right_click: ConfigProperty<WorkspaceClickAction>,

    /// Action on scroll up.
    #[serde(rename = "scroll-up")]
    #[default(WorkspaceClickAction::FocusPrevious)]
    pub scroll_up: ConfigProperty<WorkspaceClickAction>,

    /// Action on scroll down.
    #[serde(rename = "scroll-down")]
    #[default(WorkspaceClickAction::FocusNext)]
    pub scroll_down: ConfigProperty<WorkspaceClickAction>,
}

impl ModuleInfoProvider for NiriWorkspacesConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("niri-workspaces"),
            schema: || schema_for!(NiriWorkspacesConfig),
            layout_id: Some(String::from("niri-workspaces")),
            array_entry: false,
        }
    }

    fn groups() -> Vec<ConfigGroup> {
        GroupDefaults::standard()
    }
}

crate::register_module!(NiriWorkspacesConfig);
