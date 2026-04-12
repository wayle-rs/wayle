//! Bar settings pages: layout and appearance, per-button styling, and dropdown behavior.

pub(crate) mod button;
pub(crate) mod dropdown;
pub(crate) mod general;

use wayle_config::Config;

use super::nav::LeafEntry;

pub(crate) fn entries(config: &Config) -> Vec<LeafEntry> {
    vec![
        general::entry(config),
        button::entry(config),
        dropdown::entry(config),
    ]
}
