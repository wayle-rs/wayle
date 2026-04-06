//! Border radius and rounding styling types.
//!
//! Global rounding preferences and per-component radius overrides.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use wayle_derive::EnumVariants;

/// CSS variable references for semantic rounding tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoundingCssValues {
    /// Rounding for interactive elements (buttons, inputs, chips).
    pub element: &'static str,
    /// Rounding for containers (cards, dialogs, dropdowns).
    pub container: &'static str,
}

/// Global rounding preference for UI components.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, EnumVariants,
)]
#[serde(rename_all = "lowercase")]
pub enum RoundingLevel {
    /// Sharp corners (no rounding).
    None,
    /// Subtle rounding.
    #[default]
    Sm,
    /// Moderate rounding (default).
    Md,
    /// Pronounced rounding.
    Lg,
    /// Pill shape (fully rounded ends).
    Full,
}

impl RoundingLevel {
    /// CSS variable references for element and container rounding.
    ///
    /// Containers get one step larger for perceptual consistency.
    pub fn to_css_values(self) -> RoundingCssValues {
        match self {
            Self::None => RoundingCssValues {
                element: "var(--radius-none)",
                container: "var(--radius-none)",
            },
            Self::Sm => RoundingCssValues {
                element: "var(--radius-sm)",
                container: "var(--radius-md)",
            },
            Self::Md => RoundingCssValues {
                element: "var(--radius-md)",
                container: "var(--radius-lg)",
            },
            Self::Lg => RoundingCssValues {
                element: "var(--radius-lg)",
                container: "var(--radius-xl)",
            },
            Self::Full => RoundingCssValues {
                element: "var(--radius-full)",
                container: "var(--radius-xl)",
            },
        }
    }

    /// Bar-specific CSS variable references using `--bar-radius-*` tokens.
    pub fn to_bar_css_values(self) -> RoundingCssValues {
        match self {
            Self::None => RoundingCssValues {
                element: "var(--radius-none)",
                container: "var(--radius-none)",
            },
            Self::Sm => RoundingCssValues {
                element: "var(--bar-radius-sm)",
                container: "var(--bar-radius-md)",
            },
            Self::Md => RoundingCssValues {
                element: "var(--bar-radius-md)",
                container: "var(--bar-radius-lg)",
            },
            Self::Lg => RoundingCssValues {
                element: "var(--bar-radius-lg)",
                container: "var(--bar-radius-xl)",
            },
            Self::Full => RoundingCssValues {
                element: "var(--radius-full)",
                container: "var(--radius-full)",
            },
        }
    }

    /// Bar-specific Button/Group CSS variable references using `--bar-button-radius-*` tokens.
    pub fn to_bar_element_css_values(self) -> RoundingCssValues {
        match self {
            Self::None => RoundingCssValues {
                element: "var(--radius-none)",
                container: "var(--radius-none)",
            },
            Self::Sm => RoundingCssValues {
                element: "var(--bar-button-radius-sm)",
                container: "var(--bar-button-radius-md)",
            },
            Self::Md => RoundingCssValues {
                element: "var(--bar-button-radius-md)",
                container: "var(--bar-button-radius-lg)",
            },
            Self::Lg => RoundingCssValues {
                element: "var(--bar-button-radius-lg)",
                container: "var(--bar-button-radius-xl)",
            },
            Self::Full => RoundingCssValues {
                element: "var(--radius-full)",
                container: "var(--radius-full)",
            },
        }
    }
}

/// Per-component radius override independent of global `RoundingLevel`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RadiusClass {
    /// No rounding (--radius-none).
    None,
    /// Small rounding (--radius-sm).
    Sm,
    /// Medium rounding (--radius-md).
    #[default]
    Md,
    /// Large rounding (--radius-lg).
    Lg,
    /// Extra large rounding (--radius-xl).
    Xl,
    /// Full rounding for pill shapes (--radius-full).
    Full,
}

impl RadiusClass {
    /// CSS class for border radius (e.g., `radius-md`).
    pub fn css_class(self) -> &'static str {
        match self {
            Self::None => "radius-none",
            Self::Sm => "radius-sm",
            Self::Md => "radius-md",
            Self::Lg => "radius-lg",
            Self::Xl => "radius-xl",
            Self::Full => "radius-full",
        }
    }

    /// CSS variable name (e.g., `--radius-md`).
    pub fn css_var(self) -> &'static str {
        match self {
            Self::None => "--radius-none",
            Self::Sm => "--radius-sm",
            Self::Md => "--radius-md",
            Self::Lg => "--radius-lg",
            Self::Xl => "--radius-xl",
            Self::Full => "--radius-full",
        }
    }
}
