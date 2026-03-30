mod types;

use schemars::schema_for;
pub use types::{
    IconSource, PopupCloseBehavior, PopupMonitor, PopupPosition, StackingOrder, UrgencyBarThreshold,
};
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken, Spacing, ThresholdEntry},
};

/// Notification module configuration.
#[wayle_config(bar_button)]
pub struct NotificationConfig {
    /// Icon shown when no notifications and DND is off.
    #[serde(rename = "icon-name")]
    #[default(String::from("ld-bell-symbolic"))]
    pub icon_name: ConfigProperty<String>,

    /// Icon shown when notifications exist.
    #[serde(rename = "icon-unread")]
    #[default(String::from("ld-bell-dot-symbolic"))]
    pub icon_unread: ConfigProperty<String>,

    /// Icon shown when Do Not Disturb is active.
    #[serde(rename = "icon-dnd")]
    #[default(String::from("ld-bell-off-symbolic"))]
    pub icon_dnd: ConfigProperty<String>,

    /// Display border around button.
    #[serde(rename = "border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color token.
    #[serde(rename = "border-color")]
    #[default(ColorValue::Token(CssToken::Green))]
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
    #[default(ColorValue::Token(CssToken::Green))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Display notification count label.
    #[serde(rename = "label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color token.
    #[serde(rename = "label-color")]
    #[default(ColorValue::Token(CssToken::Green))]
    pub label_color: ConfigProperty<ColorValue>,

    /// Max label characters before truncation with ellipsis. Set to 0 to disable.
    #[serde(rename = "label-max-length")]
    #[default(0)]
    pub label_max_length: ConfigProperty<u32>,

    /// Button background color token.
    #[serde(rename = "button-bg-color")]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub button_bg_color: ConfigProperty<ColorValue>,

    /// Action on left click.
    #[serde(rename = "left-click")]
    #[default(ClickAction::Dropdown(String::from("notification")))]
    pub left_click: ConfigProperty<ClickAction>,

    /// Action on right click. Default toggles Do Not Disturb.
    #[serde(rename = "right-click")]
    #[default(ClickAction::Shell(String::from("wayle notify dnd")))]
    pub right_click: ConfigProperty<ClickAction>,

    /// Action on middle click.
    #[serde(rename = "middle-click")]
    #[default(ClickAction::None)]
    pub middle_click: ConfigProperty<ClickAction>,

    /// Action on scroll up.
    #[serde(rename = "scroll-up")]
    #[default(ClickAction::None)]
    pub scroll_up: ConfigProperty<ClickAction>,

    /// Action on scroll down.
    #[serde(rename = "scroll-down")]
    #[default(ClickAction::None)]
    pub scroll_down: ConfigProperty<ClickAction>,

    /// Glob patterns for app names whose notifications are blocked entirely.
    ///
    /// Matched notifications are silently dropped.
    /// Supports `*` (any characters) and `?` (single character).
    ///
    /// Examples: `["notify-send", "*chromium*", "Vivaldi*"]`
    #[serde(rename = "blocklist")]
    #[default(Vec::new())]
    pub blocklist: ConfigProperty<Vec<String>>,

    /// How notification icons are resolved.
    ///
    /// | Mode | Per-notification image | No image provided |
    /// |------|----------------------|-------------------|
    /// | `automatic` | Shows the image | Mapped icon |
    /// | `mapped` | Ignored | Mapped icon |
    /// | `application` | Shows the image | App's generic icon, then mapped fallback |
    #[serde(rename = "icon-source")]
    #[default(IconSource::default())]
    pub icon_source: ConfigProperty<IconSource>,

    /// Screen position for popup notifications.
    #[serde(rename = "popup-position")]
    #[default(PopupPosition::default())]
    pub popup_position: ConfigProperty<PopupPosition>,

    /// Maximum number of popups visible at once.
    #[serde(rename = "popup-max-visible")]
    #[default(5)]
    pub popup_max_visible: ConfigProperty<u32>,

    /// Order in which popups stack on screen.
    #[serde(rename = "popup-stacking-order")]
    #[default(StackingOrder::default())]
    pub popup_stacking_order: ConfigProperty<StackingOrder>,

    /// Maximum popup display duration in milliseconds.
    ///
    /// Applications may request a shorter timeout, which takes precedence.
    #[serde(rename = "popup-duration")]
    #[default(5000u32)]
    pub popup_duration: ConfigProperty<u32>,

    /// Pause popup auto-dismiss timer on hover.
    #[serde(rename = "popup-hover-pause")]
    #[default(true)]
    pub popup_hover_pause: ConfigProperty<bool>,

    /// Horizontal margin from screen edges.
    #[serde(rename = "popup-margin-x")]
    #[default(Spacing::new(0.0))]
    pub popup_margin_x: ConfigProperty<Spacing>,

    /// Vertical margin from screen edges.
    #[serde(rename = "popup-margin-y")]
    #[default(Spacing::new(0.0))]
    pub popup_margin_y: ConfigProperty<Spacing>,

    /// Gap between stacked popups.
    #[serde(rename = "popup-gap")]
    #[default(Spacing::new(8.0))]
    pub popup_gap: ConfigProperty<Spacing>,

    /// Target monitor: "primary" or a connector name like "DP-1".
    #[serde(rename = "popup-monitor")]
    #[default(PopupMonitor::default())]
    pub popup_monitor: ConfigProperty<PopupMonitor>,

    /// What happens when the close button on a popup is clicked.
    #[serde(rename = "popup-close-behavior")]
    #[default(PopupCloseBehavior::default())]
    pub popup_close_behavior: ConfigProperty<PopupCloseBehavior>,

    /// Display drop shadow on popup cards.
    #[serde(rename = "popup-shadow")]
    #[default(true)]
    pub popup_shadow: ConfigProperty<bool>,

    /// Minimum urgency level that displays a colored urgency bar.
    #[serde(rename = "popup-urgency-bar")]
    #[default(UrgencyBarThreshold::default())]
    pub popup_urgency_bar: ConfigProperty<UrgencyBarThreshold>,

    /// Dynamic color thresholds based on notification count.
    ///
    /// Entries are checked in order; the last matching entry wins for each
    /// color slot. Use `above` for high-value warnings (e.g., many unread
    /// notifications).
    ///
    /// ## Example
    ///
    /// ```toml
    /// [[modules.notification.thresholds]]
    /// above = 5
    /// icon-color = "status-warning"
    /// label-color = "status-warning"
    ///
    /// [[modules.notification.thresholds]]
    /// above = 20
    /// icon-color = "status-error"
    /// label-color = "status-error"
    /// ```
    #[serde(rename = "thresholds")]
    #[default(Vec::new())]
    pub thresholds: ConfigProperty<Vec<ThresholdEntry>>,
}

impl ModuleInfoProvider for NotificationConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("notification"),
            icon: String::from("󰂚"),
            description: String::from("Notification management"),
            behavior_configs: vec![(String::from("notification"), || {
                schema_for!(NotificationConfig)
            })],
            styling_configs: vec![],
        }
    }
}
