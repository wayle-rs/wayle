//! Custom modules settings.

use wayle_config::Config;

use crate::{
    editors::toml_editor::toml_editor_sized,
    pages::{
        nav::LeafEntry,
        spec::{SectionSpec, page_spec},
    },
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let mut editor = toml_editor_sized(
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
