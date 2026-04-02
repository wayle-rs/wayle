mod types;

pub use types::{OsdMonitor, OsdPosition};
use wayle_derive::wayle_config;

use crate::{ConfigProperty, schemas::styling::Spacing};

/// On-screen display configuration.
#[wayle_config]
pub struct OsdConfig {
    /// Show OSD overlays for volume, brightness, and keyboard toggles.
    #[i18n("settings-osd-enabled")]
    #[default(true)]
    pub enabled: ConfigProperty<bool>,

    /// Screen anchor position.
    #[i18n("settings-osd-position")]
    #[default(OsdPosition::default())]
    pub position: ConfigProperty<OsdPosition>,

    /// Auto-dismiss delay in milliseconds.
    #[i18n("settings-osd-duration")]
    #[default(2500u32)]
    pub duration: ConfigProperty<u32>,

    /// Target monitor: "primary" or a connector name like "DP-1".
    #[i18n("settings-osd-monitor")]
    #[default(OsdMonitor::default())]
    pub monitor: ConfigProperty<OsdMonitor>,

    /// Margin from screen edges.
    #[i18n("settings-osd-margin")]
    #[default(Spacing::new(150.0))]
    pub margin: ConfigProperty<Spacing>,

    /// Show a border around the OSD.
    #[i18n("settings-osd-border")]
    #[default(true)]
    pub border: ConfigProperty<bool>,
}
