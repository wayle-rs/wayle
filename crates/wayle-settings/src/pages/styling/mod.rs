//! Theme settings page: provider selection, palette, and provider-specific tuning.

use wayle_config::{Config, schemas::styling::PywalContrast};

use crate::{
    editors::{
        color::color,
        enum_select::enum_select,
        number::{number_newtype, number_u8},
        slider::{normalized, percentage, signed_normalized},
        text::text,
        theme_selector::theme_selector,
        toggle::toggle,
    },
    pages::{
        nav::LeafEntry,
        spec::{SectionSpec, page_spec},
    },
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
                        enum_select(&styling.theme_provider),
                        text(&styling.theming_monitor),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-palette",
                    items: vec![
                        theme_selector(&styling.available, palette, "settings-theme-preset"),
                        color(&palette.bg),
                        color(&palette.surface),
                        color(&palette.elevated),
                        color(&palette.fg),
                        color(&palette.fg_muted),
                        color(&palette.primary),
                        color(&palette.red),
                        color(&palette.yellow),
                        color(&palette.green),
                        color(&palette.blue),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-matugen",
                    items: vec![
                        enum_select(&styling.matugen_scheme),
                        signed_normalized(&styling.matugen_contrast),
                        number_u8(&styling.matugen_source_color),
                        toggle(&styling.matugen_light),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-wallust",
                    items: vec![
                        enum_select(&styling.wallust_palette),
                        percentage(&styling.wallust_saturation),
                        toggle(&styling.wallust_check_contrast),
                        enum_select(&styling.wallust_backend),
                        enum_select(&styling.wallust_colorspace),
                        toggle(&styling.wallust_apply_globally),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-pywal",
                    items: vec![
                        normalized(&styling.pywal_saturation),
                        number_newtype(
                            &styling.pywal_contrast,
                            1.0,
                            21.0,
                            0.5,
                            1,
                            |v: &PywalContrast| v.value(),
                            PywalContrast::new,
                        ),
                        toggle(&styling.pywal_light),
                        toggle(&styling.pywal_apply_globally),
                    ],
                },
            ],
        ),
    }
}
