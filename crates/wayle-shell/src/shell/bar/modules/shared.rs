//! Shared utilities for bar modules.
//!
//! This module contains common functionality used across multiple bar modules,
//! such as color resolution and unit conversion.

use wayle_config::{
    ConfigService,
    schemas::styling::{ColorValue, CssToken},
};
use wayle_styling::resolve_palette;
use wayle_widgets::primitives::chart::Rgba;

const REM_BASE: f32 = 16.0;

/// Converts rem units to pixels.
pub(super) fn rem_to_px(rem: f32, scale: f32) -> f64 {
    f64::from(rem * scale * REM_BASE)
}

/// Resolves a ColorValue to an RGBA color for rendering.
pub(super) fn resolve_rgba(color: &ColorValue, config: &ConfigService) -> Rgba {
    let hex = match color {
        ColorValue::Token(token) => {
            let raw_palette = config.config().styling.palette();
            let palette = resolve_palette(&raw_palette, &config.config().styling);
            match token {
                CssToken::BgBase => palette.bg.clone(),
                CssToken::BgSurface | CssToken::BgSurfaceElevated => palette.surface.clone(),
                CssToken::BgElevated
                | CssToken::BgOverlay
                | CssToken::BgHover
                | CssToken::BgActive
                | CssToken::BgSelected => palette.elevated.clone(),

                CssToken::FgDefault | CssToken::FgOnAccent => palette.fg.clone(),
                CssToken::FgMuted | CssToken::FgSubtle => palette.fg_muted.clone(),

                CssToken::Accent | CssToken::AccentSubtle | CssToken::AccentHover => {
                    palette.primary.clone()
                }

                CssToken::Red
                | CssToken::StatusError
                | CssToken::StatusErrorSubtle
                | CssToken::StatusErrorHover
                | CssToken::BorderError => palette.red.clone(),

                CssToken::Yellow | CssToken::StatusWarning | CssToken::StatusWarningSubtle => {
                    palette.yellow.clone()
                }

                CssToken::Green | CssToken::StatusSuccess | CssToken::StatusSuccessSubtle => {
                    palette.green.clone()
                }

                CssToken::Blue | CssToken::StatusInfo | CssToken::StatusInfoSubtle => {
                    palette.blue.clone()
                }

                CssToken::BorderSubtle
                | CssToken::BorderDefault
                | CssToken::BorderStrong
                | CssToken::BorderAccent => palette.primary.clone(),
            }
        }
        ColorValue::Custom(hex) => hex.to_string(),
        ColorValue::Transparent => {
            return Rgba {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
                alpha: 0.0,
            };
        }
        ColorValue::Auto => {
            let raw_palette = config.config().styling.palette();
            let palette = resolve_palette(&raw_palette, &config.config().styling);
            palette.primary.clone()
        }
    };

    parse_hex_rgba(&hex)
}

fn parse_hex_rgba(hex: &str) -> Rgba {
    let hex = hex.trim_start_matches('#');
    let (r, g, b, a) = match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
            (r, g, b, 255u8)
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
            let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255);
            (r, g, b, a)
        }
        _ => (255, 255, 255, 255),
    };

    Rgba {
        red: f64::from(r) / 255.0,
        green: f64::from(g) / 255.0,
        blue: f64::from(b) / 255.0,
        alpha: f64::from(a) / 255.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_6_digit() {
        let color = parse_hex_rgba("#ff0000");
        assert!((color.red - 1.0).abs() < f64::EPSILON);
        assert!(color.green.abs() < f64::EPSILON);
        assert!(color.blue.abs() < f64::EPSILON);
        assert!((color.alpha - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_hex_8_digit_with_alpha() {
        let color = parse_hex_rgba("#00ff0080");
        assert!(color.red.abs() < f64::EPSILON);
        assert!((color.green - 1.0).abs() < f64::EPSILON);
        assert!(color.blue.abs() < f64::EPSILON);
        let expected_alpha = 128.0 / 255.0;
        assert!((color.alpha - expected_alpha).abs() < 0.01);
    }
}
