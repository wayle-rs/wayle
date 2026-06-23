//! Dock settings page: position, size, visibility, rounding, background, pinned apps.

use wayle_config::Config;

use crate::{
    editors::{
        color_value::color_value, enum_select::enum_select,
        number::{dock_size, number_u64},
        slider::percentage,
        toggle::toggle, vec_string::vec_string,
    },
    editors::number::spacing,
    pages::{
        nav::LeafEntry,
        spec::{SectionSpec, page_spec},
    },
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let dock = &config.dock;

    LeafEntry {
        id: "dock",
        i18n_key: "settings-nav-dock",
        icon: "com.github.ratml3.wayle-symbolic",
        spec: page_spec(
            "settings-page-dock",
            vec![
                SectionSpec {
                    title_key: "settings-section-general",
                    items: vec![
                        enum_select(&dock.position),
                        enum_select(&dock.visibility),
                        dock_size(&dock.size),
                        number_u64(&dock.autohide_delay),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-appearance",
                    items: vec![
                        spacing(&dock.item_padding),
                        enum_select(&dock.item_rounding),
                        percentage(&dock.background_opacity),
                        color_value(&dock.bg),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-items",
                    items: vec![
                        toggle(&dock.show_running),
                        toggle(&dock.show_preview),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-pinned-apps",
                    items: vec![vec_string(&dock.pinned_apps)],
                },
            ],
        ),
    }
}
