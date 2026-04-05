mod palette;
mod types;

pub use palette::PaletteConfig;
pub use types::{
    ColorValue, CssToken, FontWeightClass, GapClass, HexColor, IconSizeClass, InvalidCssToken,
    InvalidHexColor, MatugenScheme, NormalizedF64, PaddingClass, Percentage, PywalContrast,
    RadiusClass, RoundingLevel, ScaleFactor, SignedNormalizedF64, Spacing, TextSizeClass,
    ThemeEntry, ThemeProvider, ThresholdColors, ThresholdEntry, WallustBackend, WallustColorspace,
    WallustPalette, evaluate_thresholds,
};
use wayle_derive::wayle_config;

use crate::{ConfigProperty, infrastructure::themes::Palette};

/// Styling configuration. Changes trigger stylesheet recompilation.
#[wayle_config]
pub struct StylingConfig {
    /// Scale multiplier for dropdowns, popovers, and dialogs.
    #[default(ScaleFactor::new(1.01))]
    pub scale: ConfigProperty<ScaleFactor>,

    /// Corner rounding for dropdowns, popovers, and dialogs.
    #[default(RoundingLevel::default())]
    pub rounding: ConfigProperty<RoundingLevel>,

    /// Theme provider (wayle, matugen, pywal, wallust).
    #[serde(rename = "theme-provider")]
    #[default(ThemeProvider::default())]
    pub theme_provider: ConfigProperty<ThemeProvider>,

    /// Monitor whose wallpaper drives color extraction. Empty uses the first available.
    #[serde(rename = "theming-monitor")]
    #[default(String::new())]
    pub theming_monitor: ConfigProperty<String>,

    /// Matugen color scheme type.
    #[serde(rename = "matugen-scheme")]
    #[default(MatugenScheme::default())]
    pub matugen_scheme: ConfigProperty<MatugenScheme>,

    /// Matugen contrast level (-1.0 to 1.0).
    #[serde(rename = "matugen-contrast")]
    #[default(SignedNormalizedF64::new(0.0))]
    pub matugen_contrast: ConfigProperty<SignedNormalizedF64>,

    /// Matugen source color index (0-3).
    #[serde(rename = "matugen-source-color")]
    #[default(0u8)]
    pub matugen_source_color: ConfigProperty<u8>,

    /// Matugen light mode.
    #[serde(rename = "matugen-light")]
    #[default(false)]
    pub matugen_light: ConfigProperty<bool>,

    /// Wallust palette mode.
    #[serde(rename = "wallust-palette")]
    #[default(WallustPalette::default())]
    pub wallust_palette: ConfigProperty<WallustPalette>,

    /// Wallust saturation boost (0-100, 0 disables).
    #[serde(rename = "wallust-saturation")]
    #[default(Percentage::new(0))]
    pub wallust_saturation: ConfigProperty<Percentage>,

    /// Wallust contrast checking against background.
    #[serde(rename = "wallust-check-contrast")]
    #[default(true)]
    pub wallust_check_contrast: ConfigProperty<bool>,

    /// Wallust image sampling backend.
    #[serde(rename = "wallust-backend")]
    #[default(WallustBackend::default())]
    pub wallust_backend: ConfigProperty<WallustBackend>,

    /// Wallust color space for dominant color selection.
    #[serde(rename = "wallust-colorspace")]
    #[default(WallustColorspace::default())]
    pub wallust_colorspace: ConfigProperty<WallustColorspace>,

    /// Apply wallust colors to terminals and external tools.
    #[serde(rename = "wallust-apply-globally")]
    #[default(true)]
    pub wallust_apply_globally: ConfigProperty<bool>,

    /// Pywal saturation adjustment (0.0-1.0).
    #[serde(rename = "pywal-saturation")]
    #[default(NormalizedF64::new(0.05))]
    pub pywal_saturation: ConfigProperty<NormalizedF64>,

    /// Pywal minimum contrast ratio (1.0-21.0).
    #[serde(rename = "pywal-contrast")]
    #[default(PywalContrast::new(3.0))]
    pub pywal_contrast: ConfigProperty<PywalContrast>,

    /// Pywal light mode.
    #[serde(rename = "pywal-light")]
    #[default(false)]
    pub pywal_light: ConfigProperty<bool>,

    /// Apply pywal colors to terminals and external tools.
    #[serde(rename = "pywal-apply-globally")]
    #[default(true)]
    pub pywal_apply_globally: ConfigProperty<bool>,

    /// Active color palette.
    pub palette: PaletteConfig,

    /// Discovered themes (runtime-populated).
    #[serde(skip)]
    #[schemars(skip)]
    #[wayle(skip)]
    #[default(Vec::new())]
    pub available: ConfigProperty<Vec<ThemeEntry>>,
}

impl StylingConfig {
    /// Assembles a palette from the individual color fields.
    pub fn palette(&self) -> Palette {
        Palette {
            bg: self.palette.bg.get().to_string(),
            surface: self.palette.surface.get().to_string(),
            elevated: self.palette.elevated.get().to_string(),
            fg: self.palette.fg.get().to_string(),
            fg_muted: self.palette.fg_muted.get().to_string(),
            primary: self.palette.primary.get().to_string(),
            red: self.palette.red.get().to_string(),
            yellow: self.palette.yellow.get().to_string(),
            green: self.palette.green.get().to_string(),
            blue: self.palette.blue.get().to_string(),
        }
    }
}
