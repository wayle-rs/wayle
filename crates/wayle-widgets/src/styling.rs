//! Inline CSS styling traits and helpers for Wayle components.
//!
//! Components with runtime-configurable styling implement [`InlineStyling`]
//! to standardize the CSS custom property injection pattern.

use std::borrow::Cow;

use gtk4 as gtk;
use wayle_config::{ConfigProperty, schemas::styling::ColorValue};

/// Trait for components that inject CSS custom properties at runtime.
///
/// Implementors define styling via [`build_css`](Self::build_css) and
/// set up config watchers via [`spawn_style_watcher`](Self::spawn_style_watcher),
/// keeping the subscription list co-located with the CSS generation.
pub trait InlineStyling {
    /// Sender type for dispatching style change commands.
    type Sender;

    /// Command type sent when style-affecting config changes.
    type Cmd: Send + 'static;

    /// Returns a reference to the component's CSS provider.
    fn css_provider(&self) -> &gtk::CssProvider;

    /// Builds CSS string containing custom property definitions.
    fn build_css(&self) -> String;

    /// Spawns a watcher that triggers style reload on config changes.
    ///
    /// Every property read in [`build_css`](Self::build_css) should be
    /// subscribed here to ensure runtime updates work correctly.
    fn spawn_style_watcher(&self, sender: &Self::Sender);

    /// Recompiles CSS and loads it into the provider.
    fn reload_css(&self) {
        self.css_provider().load_from_string(&self.build_css());
    }
}

/// Returns the CSS string for a color property.
///
/// `Token`, `Transparent`, and `Auto` pass through. They resolve through CSS
/// variables, so a palette swap (matugen, pywal, wallust) follows automatically.
///
/// `Custom(hex)` is used as-is under Wayle's built-in palette. Under a dynamic
/// provider it falls back to the field's default value, since a fixed hex has
/// no meaningful mapping to a generated palette.
pub fn resolve_color(prop: &ConfigProperty<ColorValue>, is_wayle_theme: bool) -> Cow<'static, str> {
    match prop.get() {
        ColorValue::Custom(_) if !is_wayle_theme => prop.default().to_css(),
        other => other.to_css(),
    }
}
