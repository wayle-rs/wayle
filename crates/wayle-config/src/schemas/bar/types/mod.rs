mod shadow;

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
pub use shadow::ShadowPreset;

/// Layout configuration for a bar on a specific monitor.
///
/// ## Examples
///
/// ```toml
/// # Single modules
/// [[bar.layout]]
/// monitor = "*"
/// left = ["dashboard"]
/// center = ["clock"]
/// right = ["systray"]
///
/// # Module with custom CSS class for per-instance styling
/// [[bar.layout]]
/// monitor = "DP-1"
/// left = [{ module = "clock", class = "primary-clock" }, "clock"]
/// center = ["media"]
///
/// # Grouped modules (share a visual container, CSS-targetable by name)
/// [[bar.layout]]
/// monitor = "DP-2"
/// left = [{ name = "status", modules = ["battery", "network"] }]
///
/// # Groups can also contain classed modules
/// [[bar.layout]]
/// monitor = "DP-3"
/// left = [{ name = "clocks", modules = [
///   { module = "clock", class = "local" },
///   { module = "world-clock", class = "remote" }
/// ]}]
///
/// # Inherit from another layout
/// [[bar.layout]]
/// monitor = "*"
/// left = ["dashboard"]
/// center = ["clock"]
/// right = ["systray"]
///
/// [[bar.layout]]
/// monitor = "HDMI-1"
/// extends = "*"
/// right = ["volume", "systray"]  # Override just this section
///
/// # Hide bar on a specific monitor
/// [[bar.layout]]
/// monitor = "HDMI-2"
/// show = false
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(default)]
pub struct BarLayout {
    /// Monitor connector name (e.g., `"DP-1"`) or `"*"` for all monitors.
    pub monitor: String,
    /// Inherit from another layout by its monitor value (e.g., `"*"`).
    pub extends: Option<String>,
    /// Whether the bar is visible on this monitor.
    pub show: bool,

    /// Modules in the left section.
    pub left: Vec<BarItem>,
    /// Modules in the center section.
    pub center: Vec<BarItem>,
    /// Modules in the right section.
    pub right: Vec<BarItem>,
}

impl Default for BarLayout {
    fn default() -> Self {
        Self {
            monitor: String::from("*"),
            extends: None,
            show: true,
            left: vec![BarItem::Module(ModuleRef::Plain(BarModule::Media))],
            center: vec![BarItem::Module(ModuleRef::Plain(BarModule::Clock))],
            right: vec![
                BarItem::Module(ModuleRef::Plain(BarModule::Battery)),
                BarItem::Module(ModuleRef::Plain(BarModule::Bluetooth)),
                BarItem::Module(ModuleRef::Plain(BarModule::Network)),
                BarItem::Module(ModuleRef::Plain(BarModule::Microphone)),
                BarItem::Module(ModuleRef::Plain(BarModule::Volume)),
            ],
        }
    }
}

/// A bar item: either a standalone module or a named group of modules.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum BarItem {
    /// A single module (plain or with custom CSS class).
    Module(ModuleRef),
    /// A named group of modules with shared visual container.
    Group(BarGroup),
}

/// Named group of modules. The name becomes a CSS ID selector.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BarGroup {
    /// Unique name for CSS targeting (becomes `#name` selector).
    pub name: String,
    /// Modules contained in this group.
    pub modules: Vec<ModuleRef>,
}

/// Reference to a module, optionally with a custom CSS class.
///
/// ## Examples
///
/// ```toml
/// # Plain module (just the name)
/// left = ["clock"]
///
/// # Module with custom CSS class
/// left = [{ module = "clock", class = "primary-clock" }]
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ModuleRef {
    /// Module with a custom CSS class.
    Classed(ClassedModule),
    /// Plain module reference.
    Plain(BarModule),
}

impl ModuleRef {
    /// Returns the underlying module type.
    pub fn module(&self) -> &BarModule {
        match self {
            Self::Plain(m) => m,
            Self::Classed(c) => &c.module,
        }
    }

    /// Returns the custom CSS class, if any.
    pub fn class(&self) -> Option<&str> {
        match self {
            Self::Plain(_) => None,
            Self::Classed(c) => Some(&c.class),
        }
    }
}

/// A module with an associated CSS class for custom styling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ClassedModule {
    /// The module type.
    pub module: BarModule,
    /// CSS class added to the module's GTK widget.
    pub class: String,
}

/// Available bar modules.
///
/// Built-in modules use kebab-case names (e.g., `"clock"`, `"battery"`).
/// Custom modules use the pattern `custom-<id>` where `<id>` is the module
/// ID defined in `[[modules.custom]]`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BarModule {
    /// Battery status and percentage.
    Battery,
    /// Bluetooth connection status and devices.
    Bluetooth,
    /// Audio frequency visualizer.
    Cava,
    /// Current time display.
    Clock,
    /// CPU usage and temperature.
    Cpu,
    /// CPU usage chart visualization.
    CpuChart,
    /// Quick access dashboard button.
    Dashboard,
    /// Compositor keybind mode indicator (submaps in Hyprland, modes in Sway/River).
    KeybindMode,
    /// Hyprland workspace switcher.
    HyprlandWorkspaces,
    /// Idle inhibitor to prevent screen timeout.
    IdleInhibit,
    /// Hyprsunset (night light) toggle.
    Hyprsunset,
    /// Keyboard layout indicator.
    KeyboardInput,
    /// Media player controls.
    Media,
    /// Microphone mute status.
    Microphone,
    /// Network connection status.
    Network,
    /// Network traffic statistics.
    Netstat,
    /// Notification center button.
    Notifications,
    /// Power menu button.
    Power,
    /// RAM usage indicator.
    Ram,
    /// Visual separator between modules.
    Separator,
    /// Storage usage indicator.
    Storage,
    /// System tray icons.
    Systray,
    /// System updates indicator.
    Updates,
    /// Volume control.
    Volume,
    /// Weather conditions display.
    Weather,
    /// Active window title.
    WindowTitle,
    /// World clock with multiple timezones.
    WorldClock,
    /// User-defined custom module by ID.
    Custom(String),
}

impl schemars::JsonSchema for BarModule {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("BarModule")
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "description": "Bar module name. Built-in modules or custom modules with 'custom-<id>' pattern.",
            "anyOf": [
                { "enum": BUILTIN_MODULES },
                {
                    "type": "string",
                    "pattern": "^custom-[a-z0-9-]+$",
                    "description": "Custom module ID (e.g., 'custom-gpu-temp')"
                }
            ]
        })
    }
}

impl BarModule {
    const CUSTOM_PREFIX: &str = "custom-";

    fn to_kebab_case(&self) -> &'static str {
        match self {
            Self::Battery => "battery",
            Self::Bluetooth => "bluetooth",
            Self::Cava => "cava",
            Self::Clock => "clock",
            Self::Cpu => "cpu",
            Self::CpuChart => "cpu-chart",
            Self::Dashboard => "dashboard",
            Self::KeybindMode => "keybind-mode",
            Self::HyprlandWorkspaces => "hyprland-workspaces",
            Self::IdleInhibit => "idle-inhibit",
            Self::Hyprsunset => "hyprsunset",
            Self::KeyboardInput => "keyboard-input",
            Self::Media => "media",
            Self::Microphone => "microphone",
            Self::Network => "network",
            Self::Netstat => "netstat",
            Self::Notifications => "notifications",
            Self::Power => "power",
            Self::Ram => "ram",
            Self::Separator => "separator",
            Self::Storage => "storage",
            Self::Systray => "systray",
            Self::Updates => "updates",
            Self::Volume => "volume",
            Self::Weather => "weather",
            Self::WindowTitle => "window-title",
            Self::WorldClock => "world-clock",
            Self::Custom(_) => unreachable!("Custom modules use dynamic serialization"),
        }
    }

    fn from_kebab_case(s: &str) -> Option<Self> {
        let module = match s {
            "battery" => Self::Battery,
            "bluetooth" => Self::Bluetooth,
            "cava" => Self::Cava,
            "clock" => Self::Clock,
            "cpu" => Self::Cpu,
            "cpu-chart" => Self::CpuChart,
            "dashboard" => Self::Dashboard,
            "keybind-mode" => Self::KeybindMode,
            "hyprland-workspaces" => Self::HyprlandWorkspaces,
            "idle-inhibit" => Self::IdleInhibit,
            "hyprsunset" => Self::Hyprsunset,
            "keyboard-input" => Self::KeyboardInput,
            "media" => Self::Media,
            "microphone" => Self::Microphone,
            "network" => Self::Network,
            "netstat" => Self::Netstat,
            "notifications" => Self::Notifications,
            "power" => Self::Power,
            "ram" => Self::Ram,
            "separator" => Self::Separator,
            "storage" => Self::Storage,
            "systray" => Self::Systray,
            "updates" => Self::Updates,
            "volume" => Self::Volume,
            "weather" => Self::Weather,
            "window-title" => Self::WindowTitle,
            "world-clock" => Self::WorldClock,
            _ => return None,
        };
        Some(module)
    }

    /// Returns the custom module ID if this is a custom module.
    pub fn custom_id(&self) -> Option<&str> {
        match self {
            Self::Custom(id) => Some(id),
            _ => None,
        }
    }
}

impl Serialize for BarModule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Custom(id) => {
                let name = format!("{}{}", Self::CUSTOM_PREFIX, id);
                serializer.serialize_str(&name)
            }
            _ => serializer.serialize_str(self.to_kebab_case()),
        }
    }
}

impl<'de> Deserialize<'de> for BarModule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        if let Some(id) = s.strip_prefix(Self::CUSTOM_PREFIX) {
            if id.is_empty() {
                return Err(de::Error::custom("custom module ID cannot be empty"));
            }
            return Ok(Self::Custom(id.to_owned()));
        }

        Self::from_kebab_case(&s).ok_or_else(|| de::Error::unknown_variant(&s, BUILTIN_MODULES))
    }
}

impl fmt::Display for BarModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Custom(id) => write!(f, "{}{}", Self::CUSTOM_PREFIX, id),
            _ => f.write_str(self.to_kebab_case()),
        }
    }
}

const BUILTIN_MODULES: &[&str] = &[
    "battery",
    "bluetooth",
    "cava",
    "clock",
    "cpu",
    "cpu-chart",
    "dashboard",
    "hyprland-workspaces",
    "hyprsunset",
    "idle-inhibit",
    "keybind-mode",
    "keyboard-input",
    "media",
    "microphone",
    "netstat",
    "network",
    "notifications",
    "power",
    "ram",
    "separator",
    "storage",
    "systray",
    "updates",
    "volume",
    "weather",
    "window-title",
    "world-clock",
];

/// Bar position on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Location {
    /// Top edge of the screen.
    Top,
    /// Bottom edge of the screen.
    Bottom,
    /// Left edge of the screen.
    Left,
    /// Right edge of the screen.
    Right,
}

impl Location {
    /// CSS class name for this location.
    pub fn css_class(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    /// Whether this location results in a vertical bar layout.
    pub fn is_vertical(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
}

/// Border placement for bar buttons.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum BorderLocation {
    /// No border.
    #[default]
    None,
    /// Border on top edge only.
    Top,
    /// Border on bottom edge only.
    Bottom,
    /// Border on left edge only.
    Left,
    /// Border on right edge only.
    Right,
    /// Border on all edges.
    All,
}

impl BorderLocation {
    /// CSS class suffix for this border location.
    pub fn css_class(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Top => Some("border-top"),
            Self::Bottom => Some("border-bottom"),
            Self::Left => Some("border-left"),
            Self::Right => Some("border-right"),
            Self::All => Some("border-all"),
        }
    }
}

/// Visual style variants for bar buttons.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum BarButtonVariant {
    /// Icon + label, minimal background.
    #[default]
    Basic,
    /// Icon in colored pill container that blends into button edge.
    BlockPrefix,
    /// Button background with colored icon container inside.
    IconSquare,
}

impl BarButtonVariant {
    /// CSS class name for this variant.
    pub fn css_class(self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::BlockPrefix => "block-prefix",
            Self::IconSquare => "icon-square",
        }
    }
}

/// Icon position within bar buttons.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum IconPosition {
    /// Icon before label (left for horizontal, top for vertical bars).
    #[default]
    Start,
    /// Icon after label (right for horizontal, bottom for vertical bars).
    End,
}

impl IconPosition {
    /// CSS class for this position, if any.
    pub fn css_class(self) -> Option<&'static str> {
        match self {
            Self::Start => None,
            Self::End => Some("icon-end"),
        }
    }
}
