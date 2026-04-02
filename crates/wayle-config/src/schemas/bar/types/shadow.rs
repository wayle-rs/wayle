use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use wayle_derive::EnumVariants;

use super::Location;

/// Shadow style for the bar.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    JsonSchema,
    EnumVariants,
)]
#[serde(rename_all = "kebab-case")]
pub enum ShadowPreset {
    /// No shadow.
    #[default]
    None,
    /// Directional shadow opposite the anchor edge.
    Drop,
    /// All-around shadow.
    Floating,
}

impl ShadowPreset {
    /// Margin in pixels needed for this shadow to render without clipping.
    pub fn margin_px(self) -> u32 {
        match self {
            Self::None => 0,
            Self::Drop => 4,
            Self::Floating => 4,
        }
    }

    /// CSS box-shadow value based on bar position.
    pub fn css_shadow(self, location: Location) -> &'static str {
        match self {
            Self::None => "none",
            Self::Drop => match location {
                Location::Top => "0 1px 2px 1px rgba(0, 0, 0, 0.25)",
                Location::Bottom => "0 -1px 2px 1px rgba(0, 0, 0, 0.25)",
                Location::Left => "1px 0 2px 1px rgba(0, 0, 0, 0.25)",
                Location::Right => "-1px 0 2px 1px rgba(0, 0, 0, 0.25)",
            },
            Self::Floating => "0 1px 2px 1px rgba(0, 0, 0, 0.25)",
        }
    }

    /// Margin in pixels for the edge opposite to the anchor where shadow extends.
    pub fn opposite_margin(self) -> u32 {
        self.margin_px()
    }
}
