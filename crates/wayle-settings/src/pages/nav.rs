//! Nav entry types plus the declarative nav layout that drives the sidebar
//! and the page factory registry.

use wayle_config::Config;

use super::{bar, general, modules, notifications, osd, spec::PageSpec, styling, wallpaper};

pub(crate) struct LeafEntry {
    pub(crate) id: &'static str,
    pub(crate) i18n_key: &'static str,
    pub(crate) icon: &'static str,
    pub(crate) spec: PageSpec,
}

pub(crate) type PageFactory = fn(&Config) -> LeafEntry;

pub(crate) struct NavSectionLayout {
    pub(crate) i18n_key: &'static str,
    pub(crate) factories: Vec<PageFactory>,
}

pub(crate) fn layout() -> Vec<NavSectionLayout> {
    vec![
        NavSectionLayout {
            i18n_key: "settings-nav-system",
            factories: vec![general::entry],
        },
        NavSectionLayout {
            i18n_key: "settings-nav-appearance",
            factories: vec![styling::entry, wallpaper::entry],
        },
        NavSectionLayout {
            i18n_key: "settings-nav-bar-section",
            factories: bar::factories(),
        },
        NavSectionLayout {
            i18n_key: "settings-nav-overlays",
            factories: vec![notifications::entry, osd::entry],
        },
        NavSectionLayout {
            i18n_key: "settings-nav-modules",
            factories: modules::factories(),
        },
    ]
}
