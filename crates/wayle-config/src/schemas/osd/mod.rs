mod types;

pub use types::{OsdMonitor, OsdPosition};
use wayle_derive::wayle_config;

use crate::{ConfigProperty, schemas::styling::Spacing};

/// On-screen display configuration.
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
