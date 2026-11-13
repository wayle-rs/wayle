mod types;

use schemars::schema_for;
pub use types::{OsdMonitor, OsdPosition};
use wayle_derive::wayle_config;

use crate::{
    ConfigProperty,
    docs::{ConfigGroup, ModuleInfo, ModuleInfoProvider},
    schemas::{general::Layer, styling::Spacing},
};

/// On-screen display overlay for transient events like volume and brightness.
///
/// The `auto-*` keys control whether a change *opens* the overlay. An overlay
/// that is already open always tracks its device's value, and that tracking
/// never restarts the dismiss timer — though a change that also re-triggers
/// the automatic display does.
#[wayle_config(i18n_prefix = "settings-osd")]
pub struct OsdConfig {
    /// Show OSD overlays for volume, brightness, and keyboard toggles.
    #[default(true)]
    pub enabled: ConfigProperty<bool>,

    /// Show the speaker OSD automatically when output volume or mute changes.
    ///
    /// Turn off to only show it on demand via `wayle osd speaker`.
    #[serde(rename = "auto-speaker")]
    #[default(true)]
    pub auto_speaker: ConfigProperty<bool>,

    /// Show the microphone OSD automatically when input volume or mute changes.
    ///
    /// Turn off to only show it on demand via `wayle osd mic`.
    #[serde(rename = "auto-microphone")]
    #[default(true)]
    pub auto_microphone: ConfigProperty<bool>,

    /// Show the brightness OSD automatically when display brightness changes.
    ///
    /// Turn off when an external daemon adjusts brightness continuously, so the
    /// overlay isn't permanently on screen. `wayle osd brightness` still works.
    #[serde(rename = "auto-brightness")]
    #[default(true)]
    pub auto_brightness: ConfigProperty<bool>,

    /// Show the OSD automatically when caps, num, or scroll lock is pressed.
    #[serde(rename = "auto-toggles")]
    #[default(true)]
    pub auto_toggles: ConfigProperty<bool>,

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

    /// Layer-shell layer the OSD is placed on.
    ///
    /// When `general.tearing-mode` is enabled, `overlay` is demoted to `top`
    /// to allow fullscreen tearing.
    #[default(Layer::Overlay)]
    pub layer: ConfigProperty<Layer>,
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
        vec![
            ConfigGroup::general(),
            ConfigGroup::prefix("Automatic display", "auto-"),
        ]
    }
}

crate::register_module!(OsdConfig);
