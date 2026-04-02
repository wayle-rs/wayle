use std::fmt::{self, Display, Formatter};

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use tracing::warn;
use wayle_derive::EnumVariants;

/// Matugen color scheme type.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, EnumVariants,
)]
#[serde(rename_all = "kebab-case")]
pub enum MatugenScheme {
    /// Adapts to image content.
    Content,
    /// Bold, dramatic palette.
    Expressive,
    /// Stays close to source colors.
    Fidelity,
    /// Playful multi-color palette.
    FruitSalad,
    /// Single-hue grayscale palette.
    Monochrome,
    /// Muted, understated palette.
    Neutral,
    /// Broad hue spread.
    Rainbow,
    /// Balanced Material You default.
    #[default]
    TonalSpot,
    /// High-saturation palette.
    Vibrant,
}

impl MatugenScheme {
    /// Returns the CLI value for matugen's `--type` flag.
    pub fn cli_value(self) -> &'static str {
        match self {
            Self::Content => "scheme-content",
            Self::Expressive => "scheme-expressive",
            Self::Fidelity => "scheme-fidelity",
            Self::FruitSalad => "scheme-fruit-salad",
            Self::Monochrome => "scheme-monochrome",
            Self::Neutral => "scheme-neutral",
            Self::Rainbow => "scheme-rainbow",
            Self::TonalSpot => "scheme-tonal-spot",
            Self::Vibrant => "scheme-vibrant",
        }
    }
}

impl Display for MatugenScheme {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.cli_value())
    }
}

/// Wallust palette mode.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, EnumVariants,
)]
#[serde(rename_all = "lowercase")]
pub enum WallustPalette {
    /// 8 dark colors with 16-color trick.
    #[default]
    Dark16,
    /// 8 dark colors, dark background and light contrast.
    Dark,
    /// Dark with complementary counterparts.
    Darkcomp,
    /// Dark complementary with 16-color trick.
    Darkcomp16,
    /// Dark with hard hue colors.
    Harddark,
    /// Hard dark with 16-color trick.
    Harddark16,
    /// Hard dark complementary variant.
    Harddarkcomp,
    /// Hard dark complementary with 16-color trick.
    Harddarkcomp16,
    /// Light background, dark foreground.
    Light,
    /// Light with 16-color trick.
    Light16,
    /// Light with complementary colors.
    Lightcomp,
    /// Light complementary with 16-color trick.
    Lightcomp16,
    /// Lightest colors with dark background.
    Softdark,
    /// Soft dark with 16-color trick.
    Softdark16,
    /// Soft dark complementary variant.
    Softdarkcomp,
    /// Soft dark complementary with 16-color trick.
    Softdarkcomp16,
    /// Light with soft pastel colors.
    Softlight,
    /// Soft light with 16-color trick.
    Softlight16,
    /// Soft light with complementary colors.
    Softlightcomp,
    /// Soft light complementary with 16-color trick.
    Softlightcomp16,
    /// ANSI-ordered dark palette for LS_COLORS.
    Ansidark,
    /// ANSI dark with 16-color trick.
    Ansidark16,
}

impl WallustPalette {
    /// Whether this palette produces a light background.
    pub fn is_light(self) -> bool {
        matches!(
            self,
            Self::Light
                | Self::Light16
                | Self::Lightcomp
                | Self::Lightcomp16
                | Self::Softlight
                | Self::Softlight16
                | Self::Softlightcomp
                | Self::Softlightcomp16
        )
    }

    /// Returns the wallust config value.
    pub fn config_value(self) -> &'static str {
        match self {
            Self::Dark16 => "dark16",
            Self::Dark => "dark",
            Self::Darkcomp => "darkcomp",
            Self::Darkcomp16 => "darkcomp16",
            Self::Harddark => "harddark",
            Self::Harddark16 => "harddark16",
            Self::Harddarkcomp => "harddarkcomp",
            Self::Harddarkcomp16 => "harddarkcomp16",
            Self::Light => "light",
            Self::Light16 => "light16",
            Self::Lightcomp => "lightcomp",
            Self::Lightcomp16 => "lightcomp16",
            Self::Softdark => "softdark",
            Self::Softdark16 => "softdark16",
            Self::Softdarkcomp => "softdarkcomp",
            Self::Softdarkcomp16 => "softdarkcomp16",
            Self::Softlight => "softlight",
            Self::Softlight16 => "softlight16",
            Self::Softlightcomp => "softlightcomp",
            Self::Softlightcomp16 => "softlightcomp16",
            Self::Ansidark => "ansidark",
            Self::Ansidark16 => "ansidark16",
        }
    }
}

impl Display for WallustPalette {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.config_value())
    }
}

/// Wallust image sampling backend.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum WallustBackend {
    /// Reads every pixel.
    Full,
    /// Resizes image before sampling.
    Resized,
    /// Uses ImageMagick convert (pywal method).
    Wal,
    /// Fixed 512x512 thumbnail.
    Thumb,
    /// SIMD-accelerated resize.
    #[default]
    Fastresize,
    /// K-means clustering.
    Kmeans,
}

impl WallustBackend {
    /// Returns the wallust config value.
    pub fn config_value(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Resized => "resized",
            Self::Wal => "wal",
            Self::Thumb => "thumb",
            Self::Fastresize => "fastresize",
            Self::Kmeans => "kmeans",
        }
    }
}

impl Display for WallustBackend {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.config_value())
    }
}

/// Wallust color space for dominant color selection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum WallustColorspace {
    /// CIELAB perceptual color space.
    Lab,
    /// LAB with mixing for sparse images.
    #[default]
    Labmixed,
    /// Cylindrical LAB (hue/chroma/lightness).
    Lch,
    /// LCH with mixing.
    Lchmixed,
    /// LCH mapped to ANSI color ordering.
    Lchansi,
}

impl WallustColorspace {
    /// Returns the wallust config value.
    pub fn config_value(self) -> &'static str {
        match self {
            Self::Lab => "lab",
            Self::Labmixed => "labmixed",
            Self::Lch => "lch",
            Self::Lchmixed => "lchmixed",
            Self::Lchansi => "lchansi",
        }
    }
}

impl Display for WallustColorspace {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.config_value())
    }
}

const SIGNED_MIN: f64 = -1.0;
const SIGNED_MAX: f64 = 1.0;

/// Floating-point value clamped to -1.0 to 1.0.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct SignedNormalizedF64(#[schemars(range(min = SIGNED_MIN, max = SIGNED_MAX))] f64);

impl SignedNormalizedF64 {
    /// Creates a value, clamping to -1.0 to 1.0.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(SIGNED_MIN, SIGNED_MAX))
    }

    /// Returns the inner f64 value.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl Default for SignedNormalizedF64 {
    fn default() -> Self {
        Self(0.0)
    }
}

impl Display for SignedNormalizedF64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<f64> for SignedNormalizedF64 {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for SignedNormalizedF64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = f64::deserialize(deserializer)?;
        if !(SIGNED_MIN..=SIGNED_MAX).contains(&raw) {
            warn!(
                "signed normalized value {} out of range (valid: {}-{}), clamped to {}",
                raw,
                SIGNED_MIN,
                SIGNED_MAX,
                raw.clamp(SIGNED_MIN, SIGNED_MAX)
            );
        }
        Ok(Self::new(raw))
    }
}

const PYWAL_CONTRAST_MIN: f64 = 1.0;
const PYWAL_CONTRAST_MAX: f64 = 21.0;

/// Pywal contrast ratio clamped to 1.0-21.0 (WCAG range).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct PywalContrast(
    #[schemars(range(min = PYWAL_CONTRAST_MIN, max = PYWAL_CONTRAST_MAX))] f64,
);

impl PywalContrast {
    /// Creates a contrast value, clamping to 1.0-21.0.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(PYWAL_CONTRAST_MIN, PYWAL_CONTRAST_MAX))
    }

    /// Returns the inner f64 value.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl Default for PywalContrast {
    fn default() -> Self {
        Self(3.0)
    }
}

impl Display for PywalContrast {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<f64> for PywalContrast {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for PywalContrast {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = f64::deserialize(deserializer)?;
        if !(PYWAL_CONTRAST_MIN..=PYWAL_CONTRAST_MAX).contains(&raw) {
            warn!(
                "pywal contrast {} out of range (valid: {}-{}), clamped to {}",
                raw,
                PYWAL_CONTRAST_MIN,
                PYWAL_CONTRAST_MAX,
                raw.clamp(PYWAL_CONTRAST_MIN, PYWAL_CONTRAST_MAX)
            );
        }
        Ok(Self::new(raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matugen_scheme_cli_values() {
        assert_eq!(MatugenScheme::TonalSpot.cli_value(), "scheme-tonal-spot");
        assert_eq!(MatugenScheme::FruitSalad.cli_value(), "scheme-fruit-salad");
        assert_eq!(MatugenScheme::Content.cli_value(), "scheme-content");
    }

    #[test]
    fn matugen_scheme_serde_roundtrip() {
        let json = serde_json::to_string(&MatugenScheme::FruitSalad).unwrap();
        assert_eq!(json, r#""fruit-salad""#);
        let parsed: MatugenScheme = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, MatugenScheme::FruitSalad);
    }

    #[test]
    fn wallust_palette_config_values() {
        assert_eq!(WallustPalette::Dark16.config_value(), "dark16");
        assert_eq!(WallustPalette::Harddark.config_value(), "harddark");
        assert_eq!(
            WallustPalette::Softlightcomp16.config_value(),
            "softlightcomp16"
        );
    }

    #[test]
    fn wallust_palette_serde_roundtrip() {
        let json = serde_json::to_string(&WallustPalette::Harddark).unwrap();
        assert_eq!(json, r#""harddark""#);
        let parsed: WallustPalette = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, WallustPalette::Harddark);
    }

    #[test]
    fn signed_normalized_clamps() {
        assert_eq!(SignedNormalizedF64::new(-2.0).value(), -1.0);
        assert_eq!(SignedNormalizedF64::new(2.0).value(), 1.0);
        assert_eq!(SignedNormalizedF64::new(0.5).value(), 0.5);
        assert_eq!(SignedNormalizedF64::new(-0.5).value(), -0.5);
    }

    #[test]
    fn wallust_palette_is_light() {
        assert!(WallustPalette::Light.is_light());
        assert!(WallustPalette::Light16.is_light());
        assert!(WallustPalette::Softlight.is_light());
        assert!(WallustPalette::Softlightcomp16.is_light());

        assert!(!WallustPalette::Dark16.is_light());
        assert!(!WallustPalette::Harddark.is_light());
        assert!(!WallustPalette::Softdark.is_light());
        assert!(!WallustPalette::Ansidark.is_light());
    }

    #[test]
    fn pywal_contrast_clamps() {
        assert_eq!(PywalContrast::new(0.5).value(), 1.0);
        assert_eq!(PywalContrast::new(25.0).value(), 21.0);
        assert_eq!(PywalContrast::new(3.0).value(), 3.0);
    }
}
