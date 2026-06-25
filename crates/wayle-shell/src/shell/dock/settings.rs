use wayle_config::{
    ConfigProperty,
    schemas::{dock::DockPosition, styling::RoundingLevel},
};

/// Settings for dock items.
#[derive(Clone)]
pub(crate) struct DockSettings {
    /// Corner rounding level for dock items.
    pub item_rounding: ConfigProperty<RoundingLevel>,
    /// Padding between dock items.
    pub item_padding: ConfigProperty<wayle_config::schemas::styling::Spacing>,
    /// Dock size in pixels.
    pub size: ConfigProperty<u32>,
    /// Dock position for layout orientation.
    pub dock_position: DockPosition,
}
