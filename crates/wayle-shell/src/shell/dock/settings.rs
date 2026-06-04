use wayle_config::{
    ConfigProperty,
    schemas::{bar::IconPosition, styling::RoundingLevel},
};

/// Settings for dock items.
#[derive(Clone)]
pub(crate) struct DockSettings {
    /// Theme provider for color resolution.
    pub theme_provider: ConfigProperty<wayle_config::schemas::styling::ThemeProvider>,
    /// Icon position relative to label.
    pub icon_position: ConfigProperty<IconPosition>,
    /// Corner rounding level for dock items.
    pub item_rounding: ConfigProperty<RoundingLevel>,
    /// Padding between dock items.
    pub item_padding: ConfigProperty<wayle_config::schemas::styling::Spacing>,
    /// Dock size in pixels.
    pub size: ConfigProperty<u32>,
    /// Monitor connector name.
    pub monitor_name: Option<String>,
}
