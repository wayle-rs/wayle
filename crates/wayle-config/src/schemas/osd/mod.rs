mod types;

use schemars::schema_for;
pub use types::{OsdMonitor, OsdPosition};
use wayle_derive::wayle_config;

use crate::{
    ConfigProperty,
    docs::{ConfigGroup, GroupDefaults, ModuleInfo, ModuleInfoProvider},
    schemas::styling::Spacing,
};

/// On-screen display overlay for transient events like volume and brightness.
#[wayle_config(i18n_prefix = "settings-osd")]
pub struct OsdConfig {
    /// Show OSD overlays for volume, brightness, and keyboard toggles.
    #[default(true)]
    pub enabled: ConfigProperty<bool>,

    /// Screen anchor position.
    #[default(OsdPosition::default())]
    pub position: ConfigProperty<OsdPosition>,

    /// Auto-dismiss delay in milliseconds.
    #[default(2500u32)]
    pub duration: ConfigProperty<u32>,

    /// Target monitor: "primary" or a connector name like "DP-1".
    #[default(OsdMonitor::default())]
    pub monitor: ConfigProperty<OsdMonitor>,

    /// Margin from screen edges.
    #[default(Spacing::new(150.0))]
    pub margin: ConfigProperty<Spacing>,

    /// Show a border around the OSD.
    #[default(true)]
    pub border: ConfigProperty<bool>,
}

impl ModuleInfoProvider for OsdConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("osd"),
            schema: || schema_for!(OsdConfig),
            layout_id: None,
            array_entry: false,
        }
    }

    fn groups() -> Vec<ConfigGroup> {
        GroupDefaults::standard()
    }
}

crate::register_module!(OsdConfig);
