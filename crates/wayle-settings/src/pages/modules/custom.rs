//! Custom modules settings.

use wayle_config::Config;

use crate::pages::{
    helpers::{self, SectionSpec, page_spec},
    nav::LeafEntry,
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let mut editor = helpers::toml_editor_sized(
        &config.modules.custom,
        "custom",
        40,
        &config.styling.palette.bg,
    );
    editor.i18n_key = Some("settings-custom-modules-editor");

    LeafEntry {
        id: "custom",
        i18n_key: "settings-nav-custom",
        icon: "ld-code-symbolic",
        spec: page_spec(
            "settings-page-custom",
            vec![SectionSpec {
                title_key: "settings-section-general",
                items: vec![editor],
            }],
        ),
    }
}
