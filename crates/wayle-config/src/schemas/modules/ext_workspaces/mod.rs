//! Generic Wayland workspace switcher configuration, backed by
//! `ext-workspace-v1` rather than a compositor-specific IPC.

use std::{collections::BTreeMap, ops::Deref};

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use wayle_derive::wayle_config;

use super::{ActiveIndicator, DisplayMode, WorkspaceClickAction, WorkspaceStyle};
use crate::{
    ConfigProperty,
    docs::{ConfigGroup, GroupDefaults, ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken, ScaleFactor, Spacing},
};

/// Per-workspace icon and color overrides, keyed by workspace name.
///
/// `ext-workspace-v1` does not guarantee a stable numeric id, so unlike
/// `hyprland-workspaces`/`niri-workspaces` this is keyed by name only.
///
/// ## Example
///
/// ```toml
/// [modules.ext-workspaces.workspace-map]
/// web = { icon = "ld-globe-symbolic", color = "#4a90d9" }
/// "1" = { label = "Term" }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct ExtWorkspaceMap(BTreeMap<String, WorkspaceStyle>);

impl Deref for ExtWorkspaceMap {
    type Target = BTreeMap<String, WorkspaceStyle>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> IntoIterator for &'a ExtWorkspaceMap {
    type Item = (&'a String, &'a WorkspaceStyle);
    type IntoIter = std::collections::btree_map::Iter<'a, String, WorkspaceStyle>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// Workspace switcher backed by the generic `ext-workspace-v1` Wayland
/// protocol. Works on any compositor advertising `ext_workspace_manager_v1`
/// (e.g. Sway), unlike the compositor-specific `hyprland-workspaces` and
/// `niri-workspaces` modules.
///
/// The protocol carries no window list, so there is no occupied/empty
/// distinction or per-window app icons here - only the active/urgent/hidden
/// state that the compositor reports for each workspace.
#[wayle_config(i18n_prefix = "settings-modules-ext-workspaces")]
pub struct ExtWorkspacesConfig {
    /// What identifies each workspace button.
    ///
    /// - `label` (default): show the workspace name
    /// - `icon`: show an icon from `workspace-map` (falls back to label if unmapped)
    /// - `none`: show nothing
    #[serde(rename = "display-mode")]
    #[default(DisplayMode::Label)]
    pub display_mode: ConfigProperty<DisplayMode>,

    /// Pulse animation on workspaces the compositor marks urgent.
    #[serde(rename = "urgent-show")]
    #[default(true)]
    pub urgent_show: ConfigProperty<bool>,

    /// Visual indicator for the active workspace.
    #[serde(rename = "active-indicator")]
    #[default(ActiveIndicator::Background)]
    pub active_indicator: ConfigProperty<ActiveIndicator>,

    /// Show workspaces the compositor marks hidden.
    ///
    /// Per the protocol, hidden workspaces are not currently visible in
    /// their workspace group; most compositors do not use this state.
    #[serde(rename = "show-hidden")]
    #[default(true)]
    pub show_hidden: ConfigProperty<bool>,

    /// Scale multiplier for workspace icons. Range: 0.25-3.0.
    #[serde(rename = "icon-size")]
    #[default(ScaleFactor::default())]
    pub icon_size: ConfigProperty<ScaleFactor>,

    /// Scale multiplier for workspace labels. Range: 0.25-3.0.
    #[serde(rename = "label-size")]
    #[default(ScaleFactor::default())]
    pub label_size: ConfigProperty<ScaleFactor>,

    /// Padding for workspace content along the bar direction.
    #[serde(rename = "workspace-padding")]
    #[default(Spacing::new(0.5))]
    pub workspace_padding: ConfigProperty<Spacing>,

    /// Workspaces to hide from the display.
    ///
    /// Glob patterns matched against the workspace's name.
    #[serde(rename = "workspace-ignore")]
    #[default(Vec::new())]
    pub workspace_ignore: ConfigProperty<Vec<String>>,

    /// Color for the active workspace.
    ///
    /// In `background` indicator mode, also used as the button background.
    #[serde(rename = "active-color")]
    #[default(ColorValue::Token(CssToken::Accent))]
    pub active_color: ConfigProperty<ColorValue>,

    /// Color for inactive workspaces.
    #[serde(rename = "inactive-color")]
    #[default(ColorValue::Token(CssToken::FgMuted))]
    pub inactive_color: ConfigProperty<ColorValue>,

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

    /// Per-workspace icon and color overrides, keyed by name.
    #[serde(rename = "workspace-map")]
    #[default(ExtWorkspaceMap::default())]
    pub workspace_map: ConfigProperty<ExtWorkspaceMap>,

    /// Action on left click. Defaults to activating the clicked workspace.
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

impl ModuleInfoProvider for ExtWorkspacesConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("ext-workspaces"),
            schema: || schema_for!(ExtWorkspacesConfig),
            layout_id: Some(String::from("ext-workspaces")),
            array_entry: false,
        }
    }

    fn groups() -> Vec<ConfigGroup> {
        GroupDefaults::standard()
    }
}

crate::register_module!(ExtWorkspacesConfig);
