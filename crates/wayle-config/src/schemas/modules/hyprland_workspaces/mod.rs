use std::{collections::HashMap, ops::Deref};

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use wayle_derive::wayle_config;

use crate::{
    ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken, ScaleFactor, Spacing},
};

/// What identifies a workspace in the UI.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum DisplayMode {
    /// Show workspace number or name.
    #[default]
    Label,
    /// Show icon from `workspace-map` (falls back to label if unmapped).
    Icon,
    /// Show nothing - only app icons visible.
    None,
}

/// How workspace numbers are displayed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Numbering {
    /// Show actual Hyprland workspace IDs (1, 2, 3, 4, 5, 6...).
    #[default]
    Absolute,
    /// Show numbers relative to monitor's starting workspace.
    ///
    /// If monitor has workspaces 4, 5, 6 assigned, they display as 1, 2, 3.
    /// Useful when keybinds use per-monitor numbering (Shift+1 for ws 4, etc.).
    Relative,
}

/// Where the urgent pulse animation is applied.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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

/// How the workspace preview popup is triggered.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PreviewTrigger {
    /// Cursor dwell with configurable delay.
    #[default]
    Hover,
    /// Right-click on workspace button.
    RightClick,
}

/// Visual indicator style for the active workspace.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
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
}

/// Per-workspace icon and color mappings.
///
/// TOML table keys are always strings, so this type handles parsing
/// string keys like "1" into i32 workspace IDs.
#[derive(Debug, Clone, Default, PartialEq, JsonSchema)]
#[schemars(transparent)]
pub struct WorkspaceMap(HashMap<i32, WorkspaceStyle>);

impl Deref for WorkspaceMap {
    type Target = HashMap<i32, WorkspaceStyle>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> IntoIterator for &'a WorkspaceMap {
    type Item = (&'a i32, &'a WorkspaceStyle);
    type IntoIter = std::collections::hash_map::Iter<'a, i32, WorkspaceStyle>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Serialize for WorkspaceMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string_map: HashMap<String, &WorkspaceStyle> = self
            .0
            .iter()
            .map(|(key, val)| (key.to_string(), val))
            .collect();
        string_map.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for WorkspaceMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string_map: HashMap<String, WorkspaceStyle> = HashMap::deserialize(deserializer)?;
        let mut result = HashMap::with_capacity(string_map.len());
        for (key, value) in string_map {
            let id: i32 = key.parse().map_err(serde::de::Error::custom)?;
            result.insert(id, value);
        }
        Ok(WorkspaceMap(result))
    }
}

/// Hyprland workspaces module configuration.
#[wayle_config]
pub struct HyprlandWorkspacesConfig {
    /// Minimum number of workspace buttons to display.
    ///
    /// When set to 0 (default), only active and occupied workspaces are shown.
    /// When set to N, at least N buttons are always visible, with empty ones
    /// using `empty-color` styling.
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
    ///
    /// Special workspaces have negative IDs in Hyprland.
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
    /// - `label`: Shows workspace number (or name if `label-use-name` is true)
    /// - `icon`: Shows icon from `workspace-map` (falls back to label if unmapped)
    /// - `none`: Shows nothing - only app icons visible
    #[serde(rename = "display-mode")]
    #[default(DisplayMode::Label)]
    pub display_mode: ConfigProperty<DisplayMode>,

    /// Use workspace name instead of number when displaying labels.
    ///
    /// Only applies when `display-mode = "label"` or as fallback for unmapped
    /// workspaces in `display-mode = "icon"`.
    #[serde(rename = "label-use-name")]
    #[default(false)]
    pub label_use_name: ConfigProperty<bool>,

    /// How workspace numbers are displayed.
    ///
    /// - `absolute`: Show actual Hyprland workspace IDs (1, 2, 3, 4, 5, 6...)
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
    /// Glob patterns matching workspace IDs. Examples:
    /// - `"10"` - hide workspace 10
    /// - `"1?"` - hide workspaces 10-19
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
    /// Keys are workspace IDs (use negative for special workspaces).
    ///
    /// ## Example
    ///
    /// ```toml
    /// [modules.hyprland-workspaces.workspace-map]
    /// 1 = { icon = "ld-globe-symbolic", color = "#4a90d9" }
    /// 2 = { icon = "ld-terminal-symbolic" }
    /// ```
    #[serde(rename = "workspace-map")]
    #[default(WorkspaceMap::default())]
    pub workspace_map: ConfigProperty<WorkspaceMap>,

    /// Show a live preview popup when hovering over workspace buttons.
    ///
    /// Displays thumbnails of all windows in the workspace, captured via the
    /// `hyprland_toplevel_export_manager_v1` Wayland protocol.
    #[serde(rename = "preview-show")]
    #[default(true)]
    pub preview_show: ConfigProperty<bool>,

    /// Width in pixels of the preview popup composite image.
    #[serde(rename = "preview-width")]
    #[default(640u32)]
    pub preview_width: ConfigProperty<u32>,

    /// Delay in milliseconds before the preview popup appears.
    ///
    /// Prevents accidental popups when the cursor passes through the
    /// workspace area without intent to preview.
    #[serde(rename = "preview-open-delay")]
    #[default(300u32)]
    pub preview_open_delay: ConfigProperty<u32>,

    /// Delay in milliseconds before the preview popup closes after
    /// the cursor leaves the workspace button or popup.
    #[serde(rename = "preview-close-delay")]
    #[default(300u32)]
    pub preview_close_delay: ConfigProperty<u32>,

    /// How the workspace preview is triggered.
    ///
    /// - `hover`: Cursor dwell on workspace button (with `preview-open-delay`)
    /// - `right-click`: Right-click on workspace button
    #[serde(rename = "preview-trigger")]
    #[default(PreviewTrigger::Hover)]
    pub preview_trigger: ConfigProperty<PreviewTrigger>,

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
    #[default(HashMap::new())]
    pub app_icon_map: ConfigProperty<HashMap<String, String>>,
}

impl ModuleInfoProvider for HyprlandWorkspacesConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("hyprland-workspaces"),
            icon: String::from("󰖯"),
            description: String::from("Hyprland workspace indicators"),
            behavior_configs: vec![(String::from("hyprland-workspaces"), || {
                schema_for!(HyprlandWorkspacesConfig)
            })],
            styling_configs: vec![],
        }
    }
}
