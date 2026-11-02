use std::{
    collections::{BTreeMap, btree_map},
    ops::Deref,
};

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Deserializer, Serialize};
use tracing::warn;
use wayle_derive::{wayle_config, wayle_enum};

use crate::{
    ConfigProperty,
    docs::{ConfigGroup, GroupDefaults, ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken, ScaleFactor, Spacing},
};

/// What identifies a workspace in the UI.
#[wayle_enum(default)]
pub enum DisplayMode {
    /// Show workspace number, or name for named and special workspaces.
    #[default]
    Label,
    /// Show icon from `workspace-map` (falls back to label if unmapped).
    Icon,
    /// Show nothing - only app icons visible.
    None,
}

/// How workspace numbers are displayed.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    wayle_derive::EnumVariants,
)]
#[serde(rename_all = "kebab-case")]
pub enum Numbering {
    /// Show actual Hyprland workspace numbers (1, 2, 3, 4, 5, 6...).
    #[default]
    Absolute,
    /// Show numbers relative to monitor's starting workspace.
    ///
    /// If monitor has workspaces 4, 5, 6 assigned, they display as 1, 2, 3.
    /// Useful when keybinds use per-monitor numbering (Shift+1 for ws 4, etc.).
    Relative,
}

/// Where the urgent pulse animation is applied.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    wayle_derive::EnumVariants,
)]
#[serde(rename_all = "kebab-case")]
pub enum UrgentMode {
    /// Pulse the entire workspace.
    #[default]
    Workspace,
    /// Pulse only the app icon(s) belonging to the urgent window.
    ///
    /// Falls back to `workspace` when app icons are disabled.
    Application,
}

/// Visual indicator style for the active workspace.
#[wayle_enum(default)]
pub enum ActiveIndicator {
    /// Entire button gets a colored background.
    #[default]
    Background,
    /// Small colored bar under the workspace button.
    Underline,
}

impl ActiveIndicator {
    /// CSS class for this indicator style.
    pub fn css_class(self) -> &'static str {
        match self {
            Self::Background => "indicator-background",
            Self::Underline => "indicator-underline",
        }
    }
}

/// Per-workspace styling override.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceStyle {
    /// Custom icon for this workspace.
    pub icon: Option<String>,
    /// Custom background color for this workspace when active.
    pub color: Option<ColorValue>,
    /// Text shown instead of the workspace's name or index.
    pub label: Option<String>,
}

/// Per-workspace icon and color overrides, keyed by workspace name or number.
///
/// Each workspace is looked up by its name first, then by its number.
/// Special workspaces are named with their `special:` prefix, such as
/// `"special:magic"`. Named and special workspaces have no number. Workspaces
/// that match no key fall back to the defaults set by
/// [`HyprlandWorkspacesConfig::display_mode`] and the active, occupied and
/// empty colors.
///
/// ## Examples
///
/// ```toml
/// [modules.hyprland-workspaces.workspace-map]
/// # Whole entry on one line with an inline table
/// 1 = { label = "Web", icon = "ld-globe-symbolic", color = "#4a90d9" }
/// 2 = { icon = "ld-terminal-symbolic" }
/// chat = { icon = "ld-message-circle-symbolic", color = "accent" }
/// "special:magic" = { icon = "ld-layers-symbolic" }
///
/// # Or spread the entry across its own subtable
/// [modules.hyprland-workspaces.workspace-map.3]
/// label = "Code"
/// icon = "ld-code-symbolic"
/// color = "status-success"
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct WorkspaceMap(BTreeMap<String, WorkspaceStyle>);

impl Deref for WorkspaceMap {
    type Target = BTreeMap<String, WorkspaceStyle>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> IntoIterator for &'a WorkspaceMap {
    type Item = (&'a String, &'a WorkspaceStyle);
    type IntoIter = btree_map::Iter<'a, String, WorkspaceStyle>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'de> Deserialize<'de> for WorkspaceMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let entries: BTreeMap<String, WorkspaceStyle> = BTreeMap::deserialize(deserializer)?;

        for key in entries.keys().filter(|key| is_negative_number(key)) {
            warn!(
                key = %key,
                "workspace-map key is a negative workspace ID and matches nothing; \
                 key named and special workspaces by name, such as \"special:magic\""
            );
        }

        Ok(Self(entries))
    }
}

fn is_negative_number(key: &str) -> bool {
    key.parse::<i64>().is_ok_and(|number| number < 0)
}

/// Hyprland workspace indicators with click-to-switch.
#[wayle_config(i18n_prefix = "settings-modules-hyprland-workspaces")]
pub struct HyprlandWorkspacesConfig {
    /// Minimum number of workspace buttons to display.
    ///
    /// When set to 0 (default), only active and occupied workspaces are shown.
    /// When set to N, numbered workspaces 1 to N are always visible, with empty
    /// ones using `empty-color` styling. Named workspaces are shown only while
    /// they exist; a Hyprland `persistent` workspace rule keeps one alive.
    #[serde(rename = "min-workspace-count")]
    #[default(0)]
    pub min_workspace_count: ConfigProperty<u8>,

    /// Show only workspaces belonging to the bar's monitor.
    ///
    /// When true, each bar shows only its monitor's workspaces.
    /// When false, all workspaces from all monitors are shown.
    #[serde(rename = "monitor-specific")]
    #[default(true)]
    pub monitor_specific: ConfigProperty<bool>,

    /// Include special workspaces (scratchpads) in the display.
    #[serde(rename = "show-special")]
    #[default(true)]
    pub show_special: ConfigProperty<bool>,

    /// Pulse animation on workspaces with urgent windows.
    ///
    /// When a window requests attention (e.g., terminal bell), the workspace
    /// button pulses until you switch to it.
    #[serde(rename = "urgent-show")]
    #[default(true)]
    pub urgent_show: ConfigProperty<bool>,

    /// Where the urgent pulse is applied.
    ///
    /// - `workspace`: Entire workspace pulses (default)
    /// - `application`: Only the app icon(s) belonging to the urgent window
    ///   pulse, falling back to `workspace` when app icons are disabled
    #[serde(rename = "urgent-mode")]
    #[default(UrgentMode::Workspace)]
    pub urgent_mode: ConfigProperty<UrgentMode>,

    /// What identifies each workspace button.
    ///
    /// - `label`: Shows workspace number (or name if `label-use-name` is true).
    ///   Named and special workspaces always show their name.
    /// - `icon`: Shows icon from `workspace-map` (falls back to label if unmapped)
    /// - `none`: Shows nothing - only app icons visible
    #[serde(rename = "display-mode")]
    #[default(DisplayMode::Label)]
    pub display_mode: ConfigProperty<DisplayMode>,

    /// Use workspace name instead of number when displaying labels.
    ///
    /// Affects numbered workspaces only; named and special workspaces always
    /// show their name.
    ///
    /// Only applies when `display-mode = "label"` or as fallback for unmapped
    /// workspaces in `display-mode = "icon"`.
    #[serde(rename = "label-use-name")]
    #[default(false)]
    pub label_use_name: ConfigProperty<bool>,

    /// How workspace numbers are displayed.
    ///
    /// - `absolute`: Show actual Hyprland workspace numbers (1, 2, 3, 4, 5, 6...)
    /// - `relative`: Show numbers relative to monitor's starting workspace.
    ///   If a monitor has workspaces 4, 5, 6 assigned, they display as 1, 2, 3.
    ///   Useful when keybinds use per-monitor numbering.
    #[serde(rename = "numbering")]
    #[default(Numbering::Absolute)]
    pub numbering: ConfigProperty<Numbering>,

    /// Text separator between workspace identity and app icons.
    ///
    /// Only shown when both `display-mode` is not `none` and `app-icons-show`
    /// is enabled. Common values: `"|"`, `"·"`, `"-"`.
    #[serde(rename = "divider")]
    #[default(String::from(" "))]
    pub divider: ConfigProperty<String>,

    /// Show application icons for windows in each workspace.
    ///
    /// When enabled, displays icons for running applications.
    /// Icons are resolved via `app-icon-map` configuration.
    #[serde(rename = "app-icons-show")]
    #[default(false)]
    pub app_icons_show: ConfigProperty<bool>,

    /// Deduplicate application icons within a workspace.
    ///
    /// When true, shows only one icon per unique window class.
    /// When false, shows an icon for every window.
    #[serde(rename = "app-icons-dedupe")]
    #[default(true)]
    pub app_icons_dedupe: ConfigProperty<bool>,

    /// Fallback icon for applications not matched by `app-icon-map`.
    #[serde(rename = "app-icons-fallback")]
    #[default(String::from("ld-app-window-symbolic"))]
    pub app_icons_fallback: ConfigProperty<String>,

    /// Icon shown for empty workspaces when `app-icons-show` is enabled.
    ///
    /// When a workspace has no windows but is displayed (via `min-workspace-count`),
    /// this icon appears as a placeholder.
    #[serde(rename = "app-icons-empty")]
    #[default(String::from("tb-minus-symbolic"))]
    pub app_icons_empty: ConfigProperty<String>,

    /// Gap between app icons within a workspace button.
    ///
    /// Only applies to spacing between app icons.
    #[serde(rename = "icon-gap")]
    #[default(Spacing::new(0.3))]
    pub icon_gap: ConfigProperty<Spacing>,

    /// Padding for workspace content along the bar direction.
    ///
    /// For horizontal bars, controls horizontal (left/right) padding.
    /// For vertical bars, controls vertical (top/bottom) padding.
    #[serde(rename = "workspace-padding")]
    #[default(Spacing::new(0.5))]
    pub workspace_padding: ConfigProperty<Spacing>,

    /// Scale multiplier for workspace icons.
    ///
    /// Applies to workspace identity icons and custom icons from `workspace-map`.
    /// Range: 0.25-3.0.
    #[serde(rename = "icon-size")]
    #[default(ScaleFactor::default())]
    pub icon_size: ConfigProperty<ScaleFactor>,

    /// Scale multiplier for workspace labels and dividers.
    ///
    /// Applies to workspace number/name labels and the divider text.
    /// Range: 0.25-3.0.
    #[serde(rename = "label-size")]
    #[default(ScaleFactor::default())]
    pub label_size: ConfigProperty<ScaleFactor>,

    /// Workspaces to hide from the display.
    ///
    /// Glob patterns matching a workspace's name or number. Special workspaces
    /// include their `special:` prefix. Examples:
    /// - `"10"` - hide workspace 10
    /// - `"1?"` - hide workspaces 10-19
    /// - `"chat"` - hide the workspace named chat
    /// - `"special:*"` - hide all special workspaces
    #[serde(rename = "workspace-ignore")]
    #[default(Vec::new())]
    pub workspace_ignore: ConfigProperty<Vec<String>>,

    /// Visual indicator for the active workspace.
    #[serde(rename = "active-indicator")]
    #[default(ActiveIndicator::Background)]
    pub active_indicator: ConfigProperty<ActiveIndicator>,

    /// Color for the active (focused) workspace.
    ///
    /// Applied to icons and labels. In `background` indicator mode,
    /// also used as the button background.
    #[serde(rename = "active-color")]
    #[default(ColorValue::Token(CssToken::Accent))]
    pub active_color: ConfigProperty<ColorValue>,

    /// Color for occupied workspaces (has windows but not focused).
    ///
    /// Applied to icons and labels.
    #[serde(rename = "occupied-color")]
    #[default(ColorValue::Token(CssToken::FgMuted))]
    pub occupied_color: ConfigProperty<ColorValue>,

    /// Color for empty workspaces.
    ///
    /// Applied to the empty placeholder icon and labels.
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

    /// Per-workspace icon and color overrides.
    ///
    /// Keys are workspace names or numbers. A workspace is matched by its name
    /// first, then by its number. Special workspaces include their `special:`
    /// prefix.
    ///
    /// ## Example
    ///
    /// ```toml
    /// [modules.hyprland-workspaces.workspace-map]
    /// 1 = { icon = "ld-globe-symbolic", color = "#4a90d9" }
    /// 2 = { icon = "ld-terminal-symbolic" }
    /// chat = { icon = "ld-message-circle-symbolic" }
    /// "special:magic" = { icon = "ld-layers-symbolic" }
    /// ```
    #[serde(rename = "workspace-map")]
    #[default(WorkspaceMap::default())]
    pub workspace_map: ConfigProperty<WorkspaceMap>,

    /// Application icon mapping with glob pattern support.
    ///
    /// Maps window class or title to symbolic icon names. Supports:
    /// - No prefix: Matches window class (e.g., `"*firefox*"`)
    /// - `class:` prefix: Explicit class match (e.g., `"class:org.mozilla.*"`)
    /// - `title:` prefix: Matches window title (e.g., `"title:*YouTube*"`)
    ///
    /// User mappings are merged with built-in defaults for common applications.
    ///
    /// ## Example
    ///
    /// ```toml
    /// [modules.hyprland-workspaces.app-icon-map]
    /// "*firefox*" = "ld-globe-symbolic"
    /// "title:*YouTube*" = "ld-youtube-symbolic"
    /// ```
    #[serde(rename = "app-icon-map")]
    #[default(BTreeMap::new())]
    pub app_icon_map: ConfigProperty<BTreeMap<String, String>>,
}

impl ModuleInfoProvider for HyprlandWorkspacesConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("hyprland-workspaces"),
            schema: || schema_for!(HyprlandWorkspacesConfig),
            layout_id: Some(String::from("hyprland-workspaces")),
            array_entry: false,
        }
    }

    fn groups() -> Vec<ConfigGroup> {
        GroupDefaults::standard()
    }
}

crate::register_module!(HyprlandWorkspacesConfig);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_map_accepts_numbers_and_names() -> Result<(), toml::de::Error> {
        let workspace_map: WorkspaceMap = toml::from_str(
            r#"
            1 = { icon = "one" }
            chat = { icon = "chat" }
            "special:magic" = { icon = "magic" }
            "#,
        )?;

        let keys: Vec<&str> = workspace_map.keys().map(String::as_str).collect();
        assert_eq!(keys, vec!["1", "chat", "special:magic"]);
        Ok(())
    }

    #[test]
    fn workspace_map_still_loads_negative_keys() -> Result<(), toml::de::Error> {
        let workspace_map: WorkspaceMap = toml::from_str(r#"-99 = { icon = "scratch" }"#)?;
        assert!(workspace_map.contains_key("-99"));
        Ok(())
    }

    #[test]
    fn detects_negative_number_keys() {
        assert!(is_negative_number("-99"));
        assert!(!is_negative_number("3"));
        assert!(!is_negative_number("special:magic"));
    }
}
