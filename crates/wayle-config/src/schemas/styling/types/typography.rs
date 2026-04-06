//! Typography styling types.
//!
//! Text size and font weight classes for consistent typography.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use wayle_derive::wayle_enum;

/// Text size class for CSS-based typography.
///
/// Maps to CSS classes like `.text-xs`, `.text-sm`, etc.
/// The actual font sizes are defined in SCSS tokens.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TextSizeClass {
    /// Extra small text (--text-xs token).
    Xs,
    /// Small text (--text-sm token).
    Sm,
    /// Medium text (--text-md token).
    #[default]
    Md,
    /// Large text (--text-lg token).
    Lg,
    /// Extra large text (--text-xl token).
    Xl,
}

impl TextSizeClass {
    /// CSS class for font sizing (e.g., `text-md`).
    pub fn css_class(self) -> &'static str {
        match self {
            Self::Xs => "text-xs",
            Self::Sm => "text-sm",
            Self::Md => "text-md",
            Self::Lg => "text-lg",
            Self::Xl => "text-xl",
        }
    }
}

/// Font weight class for typography.
///
/// Maps to CSS classes like `.weight-normal`, `.weight-bold`, etc.
/// Uses the existing `--weight-*` tokens defined in SCSS.
#[wayle_enum(default)]
#[serde(rename_all = "lowercase")]
pub enum FontWeightClass {
    /// Normal weight (--weight-normal: 400).
    #[default]
    Normal,
    /// Medium weight (--weight-medium: 500).
    Medium,
    /// Semi-bold weight (--weight-semibold: 600).
    Semibold,
    /// Bold weight (--weight-bold: 700).
    Bold,
}

impl FontWeightClass {
    /// CSS class for font weight (e.g., `weight-medium`).
    pub fn css_class(self) -> &'static str {
        match self {
            Self::Normal => "weight-normal",
            Self::Medium => "weight-medium",
            Self::Semibold => "weight-semibold",
            Self::Bold => "weight-bold",
        }
    }

    /// CSS variable name (e.g., `--weight-medium`).
    pub fn css_var(self) -> &'static str {
        match self {
            Self::Normal => "--weight-normal",
            Self::Medium => "--weight-medium",
            Self::Semibold => "--weight-semibold",
            Self::Bold => "--weight-bold",
        }
    }
}
