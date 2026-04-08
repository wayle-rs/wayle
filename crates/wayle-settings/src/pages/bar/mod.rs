//! Bar settings pages: layout and appearance, per-button styling, and dropdown behavior.

pub mod button;
pub mod dropdown;
pub mod general;

use wayle_config::Config;

use super::nav::GroupEntry;

pub(crate) fn entry(config: &Config) -> GroupEntry {
    GroupEntry {
        id: "bar",
        i18n_key: "settings-nav-bar",
        icon: "ld-layout-dashboard-symbolic",
        children: vec![
            general::entry(config),
            button::entry(config),
            dropdown::entry(config),
        ],
    }
}
