mod types;

pub use types::{
    BarButtonVariant, BarGroup, BarItem, BarLayout, BarModule, BorderLocation, ClassedModule,
    IconPosition, Location, ModuleRef, ShadowPreset,
};
use wayle_derive::wayle_config;

use crate::{
    ConfigProperty,
    schemas::styling::{
        ColorValue, CssToken, FontWeightClass, Percentage, RoundingLevel, ScaleFactor, Spacing,
    },
};

/// Bar configuration.
#[wayle_config(i18n_prefix = "settings-bar")]
pub struct BarConfig {
    //
    // === === === === === === === === === ===
    // ===          BAR SETTINGS           ===
    // === === === === === === === === === ===
    //
    /// Per-monitor bar layouts.
    #[default(vec![BarLayout::default()])]
    pub layout: ConfigProperty<Vec<BarLayout>>,

    /// Bar-specific scale multiplier for spacing, radius, and other bar elements.
    #[default(ScaleFactor::new(1.0))]
    pub scale: ConfigProperty<ScaleFactor>,

    /// Gap between bar and its attached screen edge.
    ///
    /// - **Orientation**: Distance from top (horizontal bar) or left (vertical bar)
    #[serde(rename = "inset-edge")]
    #[default(Spacing::new(0.0))]
    pub inset_edge: ConfigProperty<Spacing>,

    /// Gap at the bar's ends.
    ///
    /// - **Orientation**: Left/right (horizontal bar), top/bottom (vertical bar)
    #[serde(rename = "inset-ends")]
    #[default(Spacing::new(0.0))]
    pub inset_ends: ConfigProperty<Spacing>,

    /// Internal spacing along bar thickness.
    ///
    /// - **Orientation**: Top/bottom (horizontal bar), left/right (vertical bar)
    #[default(Spacing::new(0.35))]
    pub padding: ConfigProperty<Spacing>,

    /// Internal spacing at bar ends.
    ///
    /// - **Orientation**: Left/right (horizontal bar), top/bottom (vertical bar)
    #[serde(rename = "padding-ends")]
    #[default(Spacing::new(0.5))]
    pub padding_ends: ConfigProperty<Spacing>,

    /// Gap between modules and groups on the bar.
    #[serde(rename = "module-gap")]
    #[default(Spacing::new(0.5))]
    pub module_gap: ConfigProperty<Spacing>,

    /// Bar position on screen edge.
    #[default(Location::Top)]
    pub location: ConfigProperty<Location>,

    /// Bar background color.
    #[default(ColorValue::Token(CssToken::BgSurface))]
    pub bg: ConfigProperty<ColorValue>,

    /// Bar background opacity (0-100).
    #[serde(rename = "background-opacity")]
    #[default(Percentage::new(100))]
    pub background_opacity: ConfigProperty<Percentage>,

    /// Border placement for bar.
    #[serde(rename = "border-location")]
    #[default(BorderLocation::None)]
    pub border_location: ConfigProperty<BorderLocation>,

    /// Border width for bar (pixels).
    #[serde(rename = "border-width")]
    #[default(1u8)]
    pub border_width: ConfigProperty<u8>,

    /// Border color for the bar.
    #[serde(rename = "border-color")]
    #[default(ColorValue::Token(CssToken::BorderAccent))]
    pub border_color: ConfigProperty<ColorValue>,

    /// Corner rounding level for the bar.
    #[default(RoundingLevel::None)]
    pub rounding: ConfigProperty<RoundingLevel>,

    /// Shadow style for the bar.
    #[default(ShadowPreset::None)]
    pub shadow: ConfigProperty<ShadowPreset>,

    //
    // === === === === === === === === === === ===
    // ===       BUTTON/MODULE SETTINGS        ===
    // === === === === === === === === === === ===
    //
    /// Visual style variant for bar buttons.
    #[serde(rename = "button-variant")]
    #[default(BarButtonVariant::BlockPrefix)]
    pub button_variant: ConfigProperty<BarButtonVariant>,

    /// Button opacity (0-100).
    #[serde(rename = "button-opacity")]
    #[default(Percentage::new(100))]
    pub button_opacity: ConfigProperty<Percentage>,

    /// Button background opacity (0-100).
    #[serde(rename = "button-bg-opacity")]
    #[default(Percentage::new(100))]
    pub button_bg_opacity: ConfigProperty<Percentage>,

    /// Button icon size.
    #[serde(rename = "button-icon-size")]
    #[default(ScaleFactor::new(1.0))]
    pub button_icon_size: ConfigProperty<ScaleFactor>,

    /// Button icon container padding. Only applies to `block-prefix` and `icon-square` variants.
    #[serde(rename = "button-icon-padding")]
    #[default(ScaleFactor::new(1.0))]
    pub button_icon_padding: ConfigProperty<ScaleFactor>,

    /// Button label text size.
    #[serde(rename = "button-label-size")]
    #[default(ScaleFactor::new(1.0))]
    pub button_label_size: ConfigProperty<ScaleFactor>,

    /// Button label font weight.
    #[serde(rename = "button-label-weight")]
    #[default(FontWeightClass::Semibold)]
    pub button_label_weight: ConfigProperty<FontWeightClass>,

    /// Button label container padding.
    #[serde(rename = "button-label-padding")]
    #[default(ScaleFactor::new(1.0))]
    pub button_label_padding: ConfigProperty<ScaleFactor>,

    /// Corner rounding level for the buttons in the bar.
    #[serde(rename = "button-rounding")]
    #[default(RoundingLevel::default())]
    pub button_rounding: ConfigProperty<RoundingLevel>,

    /// Gap between button icon and label.
    #[serde(rename = "button-gap")]
    #[default(ScaleFactor::new(1.0))]
    pub button_gap: ConfigProperty<ScaleFactor>,

    /// Icon position relative to label in bar buttons.
    #[serde(rename = "button-icon-position")]
    #[default(IconPosition::Start)]
    pub button_icon_position: ConfigProperty<IconPosition>,

    /// Border placement for bar buttons.
    #[serde(rename = "button-border-location")]
    #[default(BorderLocation::All)]
    pub button_border_location: ConfigProperty<BorderLocation>,

    /// Border width for bar buttons (pixels).
    #[serde(rename = "button-border-width")]
    #[default(1u8)]
    pub button_border_width: ConfigProperty<u8>,

    /// Border placement for button groups.
    #[serde(rename = "button-group-border-location")]
    #[default(BorderLocation::None)]
    pub button_group_border_location: ConfigProperty<BorderLocation>,

    /// Border width for button groups (pixels).
    #[serde(rename = "button-group-border-width")]
    #[default(1u8)]
    pub button_group_border_width: ConfigProperty<u8>,

    /// Internal padding for button groups.
    #[serde(rename = "button-group-padding")]
    #[default(Spacing::new(0.0))]
    pub button_group_padding: ConfigProperty<Spacing>,

    /// Gap between modules within a group.
    #[serde(rename = "button-group-module-gap")]
    #[default(Spacing::new(0.25))]
    pub button_group_module_gap: ConfigProperty<Spacing>,

    /// Background color for button groups.
    #[serde(rename = "button-group-background")]
    #[default(ColorValue::Token(CssToken::BgElevated))]
    pub button_group_background: ConfigProperty<ColorValue>,

    /// Button group opacity (0-100).
    #[serde(rename = "button-group-opacity")]
    #[default(Percentage::new(100))]
    pub button_group_opacity: ConfigProperty<Percentage>,

    /// Border color for button groups.
    #[serde(rename = "button-group-border-color")]
    #[default(ColorValue::Token(CssToken::BorderAccent))]
    pub button_group_border_color: ConfigProperty<ColorValue>,

    /// Corner rounding level for button groups.
    #[serde(rename = "button-group-rounding")]
    #[default(RoundingLevel::default())]
    pub button_group_rounding: ConfigProperty<RoundingLevel>,

    //
    // === === === === === === === === === ===
    // ===        DROPDOWN SETTINGS        ===
    // === === === === === === === === === ===
    //
    /// Enable dropdown panel shadow.
    #[serde(rename = "dropdown-shadow")]
    #[default(true)]
    pub dropdown_shadow: ConfigProperty<bool>,

    /// Dropdown panel opacity (0-100).
    #[serde(rename = "dropdown-opacity")]
    #[default(Percentage::new(100))]
    pub dropdown_opacity: ConfigProperty<Percentage>,

    /// Close dropdown when clicking outside it.
    #[serde(rename = "dropdown-autohide")]
    #[default(true)]
    pub dropdown_autohide: ConfigProperty<bool>,

    /// Freeze the bar button label while its dropdown is open.
    ///
    /// Prevents the button from resizing mid-interaction, which keeps the
    /// dropdown anchored in place.
    #[serde(rename = "dropdown-freeze-label")]
    #[default(true)]
    pub dropdown_freeze_label: ConfigProperty<bool>,
}
