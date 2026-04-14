//! CSS theming for Wayle.
//!
//! Static CSS is compiled from SCSS at build time and embedded in the binary.
//! Runtime theming is done via CSS custom properties.

mod errors;
mod palette_provider;

use std::path::PathBuf;

pub use errors::Error;
use tracing::error;
use wayle_config::{
    infrastructure::themes::Palette,
    schemas::{
        bar::BarConfig,
        general::GeneralConfig,
        styling::{StylingConfig, ThemeProvider},
    },
};

/// Static CSS compiled at build time.
pub const STATIC_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));

/// Returns the SCSS source directory path.
///
/// Only useful during development for hot-reload watching.
pub fn scss_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scss")
}

/// Generates CSS custom property overrides for the current theme.
///
/// Returns a `:root { ... }` block that sets all dynamic values.
pub fn theme_css(
    palette: &Palette,
    general: &GeneralConfig,
    bar: &BarConfig,
    styling: &StylingConfig,
) -> String {
    let resolved = resolve_palette(palette, styling);

    let global_rounding = styling.rounding.get();
    let bar_rounding = bar.rounding.get();
    let button_rounding = bar.button_rounding.get();
    let group_rounding = bar.button_group_rounding.get();
    let global = global_rounding.to_css_values();
    let bar_values = bar_rounding.to_bar_css_values();
    let bar_button_values = button_rounding.to_bar_element_css_values();
    let bar_group_values = group_rounding.to_bar_element_css_values();

    format!(
        r#":root {{
    --palette-bg: {bg};
    --palette-surface: {surface};
    --palette-elevated: {elevated};
    --palette-fg: {fg};
    --palette-fg-muted: {fg_muted};
    --palette-primary: {primary};
    --palette-red: {red};
    --palette-yellow: {yellow};
    --palette-green: {green};
    --palette-blue: {blue};

    --cfg-font-sans: "{font_sans}";
    --cfg-font-mono: "{font_mono}";

    --global-scale: {global_scale};
    --bar-scale: {bar_scale};
    --bar-btn-icon-scale: {btn_icon_scale};
    --bar-btn-icon-padding-scale: {btn_icon_padding_scale};
    --bar-btn-label-scale: {btn_label_scale};
    --bar-btn-label-padding-scale: {btn_label_padding_scale};
    --bar-btn-gap-scale: {btn_gap_scale};

    --cfg-rounding-element: {rounding_element};
    --cfg-rounding-container: {rounding_container};
    --cfg-bar-rounding-element: {bar_rounding_element};
    --cfg-bar-rounding-container: {bar_rounding_container};
    --cfg-bar-button-rounding-element: {bar_button_rounding_element};
    --cfg-bar-group-rounding-element: {bar_group_rounding_element};
}}"#,
        bg = resolved.bg,
        surface = resolved.surface,
        elevated = resolved.elevated,
        fg = resolved.fg,
        fg_muted = resolved.fg_muted,
        primary = resolved.primary,
        red = resolved.red,
        yellow = resolved.yellow,
        green = resolved.green,
        blue = resolved.blue,
        font_sans = general.font_sans.get(),
        font_mono = general.font_mono.get(),
        global_scale = styling.scale.get(),
        bar_scale = bar.scale.get(),
        btn_icon_scale = bar.button_icon_size.get(),
        btn_icon_padding_scale = bar.button_icon_padding.get(),
        btn_label_scale = bar.button_label_size.get(),
        btn_label_padding_scale = bar.button_label_padding.get(),
        btn_gap_scale = bar.button_gap.get(),
        rounding_element = global.element,
        rounding_container = global.container,
        bar_rounding_element = bar_values.element,
        bar_rounding_container = bar_values.container,
        bar_button_rounding_element = bar_button_values.element,
        bar_group_rounding_element = bar_group_values.element,
    )
}

/// Compiles SCSS from disk (development only).
///
/// Used when `WAYLE_DEV=1` for hot-reload of SCSS structure changes.
///
/// # Errors
///
/// Returns error if SCSS files cannot be read or compilation fails.
pub fn compile_dev() -> Result<String, Error> {
    use std::fs;

    let scss_path = scss_dir();
    let main_path = scss_path.join("main.scss");
    let main_content = fs::read_to_string(&main_path).map_err(Error::Io)?;
    let options = grass::Options::default().load_path(&scss_path);

    grass::from_string(&main_content, &options).map_err(Error::Compilation)
}

/// Resolves the active palette based on the current theme provider.
///
/// Reads colors from the configured provider (wallust, matugen, pywal) or
/// falls back to the built-in palette. Used by components that need palette
/// colors outside of CSS (e.g., cairo drawing).
///
/// # Errors
///
/// Never errors -- falls back to the built-in palette if provider loading fails.
pub fn resolve_palette(fallback: &Palette, styling: &StylingConfig) -> Palette {
    use palette_provider::{matugen, pywal, wallust};

    match styling.theme_provider.get() {
        ThemeProvider::Wayle => fallback.clone(),

        ThemeProvider::Matugen => {
            let is_light = styling.matugen_light.get();
            matugen::MatugenProvider::load(is_light).unwrap_or_else(|err| {
                error!(error = %err, "matugen palette load failed");
                fallback.clone()
            })
        }

        ThemeProvider::Wallust => {
            let is_light = styling.wallust_palette.get().is_light();
            wallust::WallustProvider::load(is_light).unwrap_or_else(|err| {
                error!(error = %err, "wallust palette load failed");
                fallback.clone()
            })
        }

        ThemeProvider::Pywal => {
            let is_light = styling.pywal_light.get();
            pywal::PywalProvider::load(is_light).unwrap_or_else(|err| {
                error!(error = %err, "pywal palette load failed");
                fallback.clone()
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use wayle_config::{
        infrastructure::themes::palettes,
        schemas::{bar::BarConfig, styling::StylingConfig},
    };

    use super::*;

    #[test]
    #[ignore = "requires display server"]
    fn css_loads_into_gtk4() {
        gtk4::init().unwrap();

        let provider = gtk4::CssProvider::new();

        provider.load_from_string(STATIC_CSS);

        let theme = palettes::builtins().into_iter().next().unwrap();
        let general = GeneralConfig::default();
        let bar = BarConfig::default();
        let styling = StylingConfig::default();
        let theme_str = theme_css(&theme.palette, &general, &bar, &styling);

        provider.load_from_string(&theme_str);

        let combined = format!("{STATIC_CSS}\n{theme_str}");
        provider.load_from_string(&combined);
    }
}
