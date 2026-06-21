//! Window switcher module configuration, backed by
//! `wlr-foreign-toplevel-management-unstable-v1`.

use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ConfigGroup, GroupDefaults, ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken},
};

/// Open-window counter with a dropdown listing every window, backed by the
/// generic `wlr-foreign-toplevel-management-unstable-v1` Wayland protocol.
/// Works on any wlroots-based compositor (e.g. Sway).
///
/// Distinct from `window-title`, which only displays the currently focused
/// window's title and has no list/switcher of its own.
///
/// The protocol carries no focus history, so the dropdown lists windows in
/// a stable order rather than most-recently-used.
#[wayle_config(bar_button, i18n_prefix = "settings-modules-window-switcher")]
pub struct WindowSwitcherConfig {
    /// Icon shown in the bar.
    #[serde(rename = "icon")]
    #[default(String::from("ld-app-window-symbolic"))]
    pub icon: ConfigProperty<String>,

    /// Display the open-window count label next to the icon.
    #[serde(rename = "label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Max label characters before truncation with ellipsis. Set to 0 to
    /// disable. Has no practical effect on the count label itself, but
    /// follows the same convention as other bar-button modules.
    #[serde(rename = "label-max-length")]
    #[default(0)]
    pub label_max_length: ConfigProperty<u32>,

    /// Hide the count label entirely when there are zero windows, instead
    /// of showing "0".
    #[serde(rename = "hide-when-empty")]
    #[default(false)]
    pub hide_when_empty: ConfigProperty<bool>,

    /// Max title length shown per row in the dropdown before truncation.
    /// Set to 0 to disable truncation.
    #[serde(rename = "max-title-length")]
    #[default(48)]
    pub max_title_length: ConfigProperty<u32>,

    /// Windows to hide from the dropdown and count.
    ///
    /// Glob patterns matched against the window's app-id.
    #[serde(rename = "ignore-app-id")]
    #[default(Vec::new())]
    pub ignore_app_id: ConfigProperty<Vec<String>>,

    /// Display border around button.
    #[serde(rename = "border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color token.
    #[serde(rename = "border-color")]
    #[default(ColorValue::Auto)]
    pub border_color: ConfigProperty<ColorValue>,

    /// Display module icon.
    #[serde(rename = "icon-show")]
    #[default(true)]
    pub icon_show: ConfigProperty<bool>,

    /// Icon foreground color. Auto selects based on variant for contrast.
    #[serde(rename = "icon-color")]
    #[default(ColorValue::Auto)]
    pub icon_color: ConfigProperty<ColorValue>,

    /// Icon container background color token.
    #[serde(rename = "icon-bg-color")]
    #[default(ColorValue::Auto)]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Count label text color token.
    #[serde(rename = "label-color")]
    #[default(ColorValue::Auto)]
    pub label_color: ConfigProperty<ColorValue>,

    /// Button background color token.
    ///
    /// Deliberately not `Auto` (which resolves to the same accent token as
    /// the default `label-color`/`icon-bg-color`): an accent-on-accent
    /// background would make the count label invisible.
    #[serde(rename = "button-bg-color")]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub button_bg_color: ConfigProperty<ColorValue>,

    /// Action on left click. Defaults to opening the window list dropdown.
    #[serde(rename = "left-click")]
    #[default(ClickAction::Dropdown(String::from("window-switcher")))]
    pub left_click: ConfigProperty<ClickAction>,

    /// Action on middle click.
    #[serde(rename = "middle-click")]
    #[default(ClickAction::None)]
    pub middle_click: ConfigProperty<ClickAction>,

    /// Action on right click.
    #[serde(rename = "right-click")]
    #[default(ClickAction::None)]
    pub right_click: ConfigProperty<ClickAction>,

    /// Action on scroll up.
    #[serde(rename = "scroll-up")]
    #[default(ClickAction::None)]
    pub scroll_up: ConfigProperty<ClickAction>,

    /// Action on scroll down.
    #[serde(rename = "scroll-down")]
    #[default(ClickAction::None)]
    pub scroll_down: ConfigProperty<ClickAction>,
}

impl ModuleInfoProvider for WindowSwitcherConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("window-switcher"),
            schema: || schema_for!(WindowSwitcherConfig),
            layout_id: Some(String::from("window-switcher")),
            array_entry: false,
        }
    }

    fn groups() -> Vec<ConfigGroup> {
        GroupDefaults::bar_button()
    }
}

crate::register_module!(WindowSwitcherConfig);
