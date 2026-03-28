mod types;

mod battery;
mod bluetooth;
mod cava;
mod clock;
mod cpu;
mod cpuchart;
mod custom;
mod dashboard;
mod hyprland_workspaces;
mod hyprsunset;
mod idle_inhibit;
mod keybind_mode;
mod keyboard_input;
mod media;
mod microphone;
mod netstat;
mod network;
/// Notification module configuration and popup types.
pub mod notification;
mod power;
mod ram;
mod separator;
mod storage;
mod systray;
mod volume;
mod weather;
mod window_title;
mod world_clock;

pub use super::barchart::BarDirection;
pub use battery::BatteryConfig;
pub use bluetooth::BluetoothConfig;
pub use cava::{
    BarCount as CavaBarCount, CavaConfig, CavaInput, CavaStyle, Framerate as CavaFramerate,
    FrequencyHz,
};
pub use clock::ClockConfig;
pub use cpu::CpuConfig;
pub use cpuchart::CpuChartConfig;
pub use custom::{CustomModuleDefinition, ExecutionMode, RestartDelay, RestartPolicy};
pub use dashboard::DashboardConfig;
pub use hyprland_workspaces::{
    ActiveIndicator, DisplayMode, HyprlandWorkspacesConfig, Numbering, UrgentMode, WorkspaceStyle,
};
pub use hyprsunset::HyprsunsetConfig;
pub use idle_inhibit::IdleInhibitConfig;
pub use keybind_mode::KeybindModeConfig;
pub use keyboard_input::KeyboardInputConfig;
pub use media::{BUILTIN_MAPPINGS, MediaConfig, MediaIconType};
pub use microphone::MicrophoneConfig;
pub use netstat::NetstatConfig;
pub use network::NetworkConfig;
pub use notification::{
    IconSource, NotificationConfig, PopupCloseBehavior, PopupMonitor, PopupPosition, StackingOrder,
    UrgencyBarThreshold,
};
pub use power::PowerConfig;
pub use ram::RamConfig;
pub use separator::SeparatorConfig;
pub use storage::StorageConfig;
pub use systray::{SystrayConfig, TrayItemOverride};
pub use types::TimeFormat;
pub use volume::{AppIconSource, VolumeConfig};
use wayle_derive::wayle_config;
pub use weather::{TemperatureUnit, WeatherConfig, WeatherProvider};
pub use window_title::{BUILTIN_MAPPINGS as WINDOW_TITLE_BUILTIN_MAPPINGS, WindowTitleConfig};
pub use world_clock::WorldClockConfig;

use crate::ConfigProperty;

/// Configuration for all available Wayle modules.
#[wayle_config]
pub struct ModulesConfig {
    /// Battery status module.
    pub battery: BatteryConfig,
    /// Bluetooth connection module.
    pub bluetooth: BluetoothConfig,
    /// Cava audio visualizer module.
    pub cava: CavaConfig,
    /// Clock display module.
    pub clock: ClockConfig,
    /// CPU usage module.
    pub cpu: CpuConfig,
    /// CPU usage chart module.
    #[serde(rename = "cpu-chart")]
    pub cpuchart: CpuChartConfig,
    /// Dashboard module.
    pub dashboard: DashboardConfig,
    /// Hyprland workspace switcher module.
    #[serde(rename = "hyprland-workspaces")]
    pub hyprland_workspaces: HyprlandWorkspacesConfig,
    /// Hyprsunset (blue light filter) module.
    pub hyprsunset: HyprsunsetConfig,
    /// Idle inhibitor module.
    #[serde(rename = "idle-inhibit")]
    pub idle_inhibit: IdleInhibitConfig,
    /// Keybind mode indicator module.
    #[serde(rename = "keybind-mode")]
    pub keybind_mode: KeybindModeConfig,
    /// Keyboard input module.
    #[serde(rename = "keyboard-input")]
    pub keyboard_input: KeyboardInputConfig,
    /// Media player module.
    pub media: MediaConfig,
    /// Microphone input module.
    pub microphone: MicrophoneConfig,
    /// Network connection module.
    pub network: NetworkConfig,
    /// Network traffic statistics module.
    pub netstat: NetstatConfig,
    /// Notification center module.
    pub notification: NotificationConfig,
    /// Power menu module.
    pub power: PowerConfig,
    /// RAM usage module.
    pub ram: RamConfig,
    /// Storage usage module.
    pub storage: StorageConfig,
    /// Separator module.
    pub separator: SeparatorConfig,
    /// System tray module.
    pub systray: SystrayConfig,
    /// Volume control module.
    pub volume: VolumeConfig,
    /// Weather display module.
    pub weather: WeatherConfig,
    /// Window title module.
    #[serde(rename = "window-title")]
    pub window_title: WindowTitleConfig,
    /// World clock module.
    #[serde(rename = "world-clock")]
    pub world_clock: WorldClockConfig,
    /// Custom user-defined modules.
    #[default(Vec::new())]
    pub custom: ConfigProperty<Vec<CustomModuleDefinition>>,
}
