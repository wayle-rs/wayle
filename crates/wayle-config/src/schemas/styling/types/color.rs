//! Color-related styling types.
//!
//! CSS color tokens and color values for theming.

use std::{borrow::Cow, fmt, str::FromStr};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use wayle_derive::wayle_enum;

use super::validated::HexColor;

/// CSS color token names from the design system.
///
/// Provides type-safe access to CSS custom properties defined in SCSS.
/// Derived tokens are computed in CSS via `color-mix()` and cannot be
/// resolved to hex at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CssToken {
    /// `--bg-base` - Application background.
    BgBase,
    /// `--bg-surface` - Elevated surfaces.
    BgSurface,
    /// `--bg-surface-elevated` - Subtle elevation from surface (buttons on surface).
    BgSurfaceElevated,
    /// `--bg-elevated` - Higher elevation surfaces.
    BgElevated,
    /// `--bg-overlay` - Popovers, dialogs.
    BgOverlay,
    /// `--bg-hover` - Hover state background.
    BgHover,
    /// `--bg-active` - Active/pressed state background.
    BgActive,
    /// `--bg-selected` - Selected item background.
    BgSelected,

    /// `--fg-default` - Primary text color.
    FgDefault,
    /// `--fg-muted` - Secondary text color.
    FgMuted,
    /// `--fg-subtle` - Tertiary/hint text color.
    FgSubtle,
    /// `--fg-on-accent` - Text color on accent backgrounds.
    FgOnAccent,

    /// `--accent` - Primary accent color.
    Accent,
    /// `--accent-subtle` - Subtle accent background.
    AccentSubtle,
    /// `--accent-hover` - Accent hover state.
    AccentHover,

    /// `--status-error` - Error state color.
    StatusError,
    /// `--status-warning` - Warning state color.
    StatusWarning,
    /// `--status-success` - Success state color.
    StatusSuccess,
    /// `--status-info` - Info state color.
    StatusInfo,
    /// `--status-error-subtle` - Subtle error background.
    StatusErrorSubtle,
    /// `--status-warning-subtle` - Subtle warning background.
    StatusWarningSubtle,
    /// `--status-success-subtle` - Subtle success background.
    StatusSuccessSubtle,
    /// `--status-info-subtle` - Subtle info background.
    StatusInfoSubtle,
    /// `--status-error-hover` - Error hover state.
    StatusErrorHover,

    /// `--red` - Red color for stylistic/decorative use.
    Red,
    /// `--yellow` - Yellow color for stylistic/decorative use.
    Yellow,
    /// `--green` - Green color for stylistic/decorative use.
    Green,
    /// `--blue` - Blue color for stylistic/decorative use.
    Blue,

    /// `--border-subtle` - Subtle border color.
    BorderSubtle,
    /// `--border-default` - Default border color.
    BorderDefault,
    /// `--border-strong` - Strong border color.
    BorderStrong,
    /// `--border-accent` - Accent-colored border.
    BorderAccent,
    /// `--border-error` - Error state border.
    BorderError,
}

impl CssToken {
    /// CSS variable reference (e.g., `var(--accent)`).
    pub fn css_var(self) -> &'static str {
        match self {
            Self::BgBase => "var(--bg-base)",
            Self::BgSurface => "var(--bg-surface)",
            Self::BgSurfaceElevated => "var(--bg-surface-elevated)",
            Self::BgElevated => "var(--bg-elevated)",
            Self::BgOverlay => "var(--bg-overlay)",
            Self::BgHover => "var(--bg-hover)",
            Self::BgActive => "var(--bg-active)",
            Self::BgSelected => "var(--bg-selected)",

            Self::FgDefault => "var(--fg-default)",
            Self::FgMuted => "var(--fg-muted)",
            Self::FgSubtle => "var(--fg-subtle)",
            Self::FgOnAccent => "var(--fg-on-accent)",

            Self::Accent => "var(--accent)",
            Self::AccentSubtle => "var(--accent-subtle)",
            Self::AccentHover => "var(--accent-hover)",

            Self::StatusError => "var(--status-error)",
            Self::StatusWarning => "var(--status-warning)",
            Self::StatusSuccess => "var(--status-success)",
            Self::StatusInfo => "var(--status-info)",
            Self::StatusErrorSubtle => "var(--status-error-subtle)",
            Self::StatusWarningSubtle => "var(--status-warning-subtle)",
            Self::StatusSuccessSubtle => "var(--status-success-subtle)",
            Self::StatusInfoSubtle => "var(--status-info-subtle)",
            Self::StatusErrorHover => "var(--status-error-hover)",

            Self::Red => "var(--red)",
            Self::Yellow => "var(--yellow)",
            Self::Green => "var(--green)",
            Self::Blue => "var(--blue)",

            Self::BorderSubtle => "var(--border-subtle)",
            Self::BorderDefault => "var(--border-default)",
            Self::BorderStrong => "var(--border-strong)",
            Self::BorderAccent => "var(--border-accent)",
            Self::BorderError => "var(--border-error)",
        }
    }

    /// Token name without `var()` wrapper (e.g., `--accent`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BgBase => "--bg-base",
            Self::BgSurface => "--bg-surface",
            Self::BgSurfaceElevated => "--bg-surface-elevated",
            Self::BgElevated => "--bg-elevated",
            Self::BgOverlay => "--bg-overlay",
            Self::BgHover => "--bg-hover",
            Self::BgActive => "--bg-active",
            Self::BgSelected => "--bg-selected",

            Self::FgDefault => "--fg-default",
            Self::FgMuted => "--fg-muted",
            Self::FgSubtle => "--fg-subtle",
            Self::FgOnAccent => "--fg-on-accent",

            Self::Accent => "--accent",
            Self::AccentSubtle => "--accent-subtle",
            Self::AccentHover => "--accent-hover",

            Self::StatusError => "--status-error",
            Self::StatusWarning => "--status-warning",
            Self::StatusSuccess => "--status-success",
            Self::StatusInfo => "--status-info",
            Self::StatusErrorSubtle => "--status-error-subtle",
            Self::StatusWarningSubtle => "--status-warning-subtle",
            Self::StatusSuccessSubtle => "--status-success-subtle",
            Self::StatusInfoSubtle => "--status-info-subtle",
            Self::StatusErrorHover => "--status-error-hover",

            Self::Red => "--red",
            Self::Yellow => "--yellow",
            Self::Green => "--green",
            Self::Blue => "--blue",

            Self::BorderSubtle => "--border-subtle",
            Self::BorderDefault => "--border-default",
            Self::BorderStrong => "--border-strong",
            Self::BorderAccent => "--border-accent",
            Self::BorderError => "--border-error",
        }
    }
}

impl fmt::Display for CssToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Error when parsing an invalid CSS token name.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown CSS token: '{0}' (see documentation for valid values)")]
pub struct InvalidCssToken(pub String);

impl FromStr for CssToken {
    type Err = InvalidCssToken;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bg-base" => Ok(Self::BgBase),
            "bg-surface" => Ok(Self::BgSurface),
            "bg-surface-elevated" => Ok(Self::BgSurfaceElevated),
            "bg-elevated" => Ok(Self::BgElevated),
            "bg-overlay" => Ok(Self::BgOverlay),
            "bg-hover" => Ok(Self::BgHover),
            "bg-active" => Ok(Self::BgActive),
            "bg-selected" => Ok(Self::BgSelected),

            "fg-default" => Ok(Self::FgDefault),
            "fg-muted" => Ok(Self::FgMuted),
            "fg-subtle" => Ok(Self::FgSubtle),
            "fg-on-accent" => Ok(Self::FgOnAccent),

            "accent" => Ok(Self::Accent),
            "accent-subtle" => Ok(Self::AccentSubtle),
            "accent-hover" => Ok(Self::AccentHover),

            "status-error" => Ok(Self::StatusError),
            "status-warning" => Ok(Self::StatusWarning),
            "status-success" => Ok(Self::StatusSuccess),
            "status-info" => Ok(Self::StatusInfo),
            "status-error-subtle" => Ok(Self::StatusErrorSubtle),
            "status-warning-subtle" => Ok(Self::StatusWarningSubtle),
            "status-success-subtle" => Ok(Self::StatusSuccessSubtle),
            "status-info-subtle" => Ok(Self::StatusInfoSubtle),
            "status-error-hover" => Ok(Self::StatusErrorHover),

            "red" => Ok(Self::Red),
            "yellow" => Ok(Self::Yellow),
            "green" => Ok(Self::Green),
            "blue" => Ok(Self::Blue),

            "border-subtle" => Ok(Self::BorderSubtle),
            "border-default" => Ok(Self::BorderDefault),
            "border-strong" => Ok(Self::BorderStrong),
            "border-accent" => Ok(Self::BorderAccent),
            "border-error" => Ok(Self::BorderError),

            _ => Err(InvalidCssToken(s.to_owned())),
        }
    }
}

/// CSS token reference, custom hex color, transparent, or context-aware auto.
///
/// Token references (e.g., `"accent"`) use CSS variables that update with themes.
/// Custom hex values (e.g., `"#414868"`) remain fixed.
#[derive(Debug, Clone, PartialEq)]
pub enum ColorValue {
    /// CSS token reference. Uses CSS variables that respond to theme changes.
    Token(CssToken),

    /// Fixed hex color (e.g., `"#414868"`). Ignores theme changes.
    Custom(HexColor),

    /// Fully transparent. Maps to CSS `transparent` keyword.
    Transparent,

    /// Context-awareness marker that defers the color resolution to
    /// its consumer.
    Auto,
}

impl Default for ColorValue {
    fn default() -> Self {
        Self::Token(CssToken::FgDefault)
    }
}

impl ColorValue {
    /// CSS value for inline styles.
    ///
    /// Token returns `var(--*)`, custom returns hex string.
    /// Auto falls back to accent - components should resolve Auto before calling this.
    pub fn to_css(&self) -> Cow<'static, str> {
        match self {
            Self::Token(token) => Cow::Borrowed(token.css_var()),
            Self::Custom(hex) => Cow::Owned(hex.to_string()),
            Self::Transparent => Cow::Borrowed("transparent"),
            Self::Auto => Cow::Borrowed(CssToken::Accent.css_var()),
        }
    }

    /// Returns true if this is the Auto variant.
    pub fn is_auto(&self) -> bool {
        matches!(self, Self::Auto)
    }
}

impl Serialize for ColorValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Token(token) => {
                let name = token.as_str().strip_prefix("--").unwrap_or(token.as_str());
                serializer.serialize_str(name)
            }
            Self::Custom(hex) => serializer.serialize_str(hex.as_str()),
            Self::Transparent => serializer.serialize_str("transparent"),
            Self::Auto => serializer.serialize_str("auto"),
        }
    }
}

impl<'de> Deserialize<'de> for ColorValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "transparent" {
            Ok(Self::Transparent)
        } else if s == "auto" {
            Ok(Self::Auto)
        } else if s.starts_with('#') {
            HexColor::new(s)
                .map(Self::Custom)
                .map_err(serde::de::Error::custom)
        } else {
            s.parse::<CssToken>()
                .map(Self::Token)
                .map_err(serde::de::Error::custom)
        }
    }
}

impl schemars::JsonSchema for ColorValue {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("ColorValue")
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        use schemars::json_schema;

        let token_schema = CssToken::json_schema(generator);
        let token_enum = token_schema
            .as_object()
            .and_then(|obj| obj.get("oneOf"))
            .cloned()
            .unwrap_or_default();

        json_schema!({
            "description": "CSS token, hex color (#rgb, #rgba, #rrggbb, or #rrggbbaa), 'transparent', or 'auto'",
            "anyOf": [
                { "oneOf": token_enum },
                { "enum": ["transparent", "auto"] },
                { "type": "string", "pattern": "^#([0-9a-fA-F]{3}|[0-9a-fA-F]{4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$" }
            ]
        })
    }
}

/// Source of color palette values.
///
/// Dynamic providers (Matugen, Pywal, Wallust) inject palette tokens at runtime.
#[wayle_enum(default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeProvider {
    /// Static theming using Wayle's built-in palettes.
    #[default]
    Wayle,
    /// Dynamic theming via Matugen.
    Matugen,
    /// Dynamic theming via Pywal.
    Pywal,
    /// Dynamic theming via Wallust.
    Wallust,
}

impl fmt::Display for ThemeProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Wayle => "wayle",
            Self::Matugen => "matugen",
            Self::Pywal => "pywal",
            Self::Wallust => "wallust",
        };
        f.write_str(s)
    }
}
