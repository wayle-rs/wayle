//! Bar settings pages: layout and appearance, per-button styling, and dropdown behavior.

pub mod button;
pub mod dropdown;
pub mod general;

use wayle_config::Config;

use super::nav::LeafEntry;

pub(crate) fn entries(config: &Config) -> Vec<LeafEntry> {
    vec![
        general::entry(config),
        button::entry(config),
        dropdown::entry(config),
    ]
}
