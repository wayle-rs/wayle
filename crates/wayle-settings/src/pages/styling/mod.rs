//! Theme settings page: provider selection, palette, scale, and provider-specific tuning.

use wayle_config::{Config, schemas::styling::PywalContrast};

use super::{
    helpers::{self, SectionSpec, page_spec},
    nav::LeafEntry,
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let styling = &config.styling;
    let palette = &styling.palette;

    LeafEntry {
        id: "theme",
        i18n_key: "settings-nav-theme",
        icon: "ld-palette-symbolic",
        spec: page_spec(
            "settings-page-theme",
            vec![
                SectionSpec {
                    title_key: "settings-section-provider",
                    items: vec![
                        helpers::enum_select(&styling.theme_provider),
                        helpers::text(&styling.theming_monitor),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-palette",
                    items: vec![
                        helpers::theme_selector(
                            &styling.available,
                            palette,
                            "settings-theme-preset",
                        ),
                        helpers::color(&palette.bg),
                        helpers::color(&palette.surface),
                        helpers::color(&palette.elevated),
                        helpers::color(&palette.fg),
                        helpers::color(&palette.fg_muted),
                        helpers::color(&palette.primary),
                        helpers::color(&palette.red),
                        helpers::color(&palette.yellow),
                        helpers::color(&palette.green),
                        helpers::color(&palette.blue),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-scale-rounding",
                    items: vec![
                        helpers::scale(&styling.scale),
                        helpers::enum_select(&styling.rounding),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-matugen",
                    items: vec![
                        helpers::enum_select(&styling.matugen_scheme),
                        helpers::signed_normalized(&styling.matugen_contrast),
                        helpers::number_u8(&styling.matugen_source_color),
                        helpers::toggle(&styling.matugen_light),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-wallust",
                    items: vec![
                        helpers::enum_select(&styling.wallust_palette),
                        helpers::percentage(&styling.wallust_saturation),
                        helpers::toggle(&styling.wallust_check_contrast),
                        helpers::enum_select(&styling.wallust_backend),
                        helpers::enum_select(&styling.wallust_colorspace),
                        helpers::toggle(&styling.wallust_apply_globally),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-pywal",
                    items: vec![
                        helpers::normalized(&styling.pywal_saturation),
                        helpers::number_newtype(
                            &styling.pywal_contrast,
                            1.0,
                            21.0,
                            0.5,
                            1,
                            |v: &PywalContrast| v.value(),
                            PywalContrast::new,
                        ),
                        helpers::toggle(&styling.pywal_light),
                        helpers::toggle(&styling.pywal_apply_globally),
                    ],
                },
            ],
        ),
    }
}
