//! Triad state types exposed by the service.

use serde::{Deserialize, Deserializer};
use wayle_core::Property;

/// Desktop capabilities reported by Triad.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Capabilities {
    /// Whether the native event stream is available.
    #[serde(default)]
    pub event_stream: bool,
    /// Whether full state snapshots are available.
    #[serde(default)]
    pub state: bool,
    /// Whether layout snapshots are available.
    #[serde(default)]
    pub layout_state: bool,
    /// Whether overview state is available.
    #[serde(default)]
    pub overview: bool,
    /// Whether workspace switching is supported.
    #[serde(default)]
    pub workspace_switching: bool,
    /// Whether window focusing is supported.
    #[serde(default)]
    pub window_focus: bool,
    /// Whether window closing is supported.
    #[serde(default)]
    pub window_close: bool,
    /// Whether commands can be spawned.
    #[serde(default)]
    pub spawn: bool,
    /// Whether keyboard layout switching is supported.
    #[serde(default)]
    pub keyboard_layout: bool,
    /// Whether workspace urgency is supported.
    #[serde(default)]
    pub workspace_urgency: bool,
}

/// Keyboard layouts reported by Triad.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct KeyboardLayouts {
    /// Layout names in compositor order.
    #[serde(default)]
    pub names: Vec<String>,
    /// Active layout index.
    #[serde(default)]
    pub current_idx: u32,
}

/// Keyboard layout switch target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardLayoutTarget {
    /// Switch to the next layout.
    Next,
    /// Switch to the previous layout.
    Previous,
    /// Switch to a specific layout index.
    Index(u32),
}

/// Integer point.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
pub struct Point {
    /// X coordinate.
    #[serde(default)]
    pub x: i32,
    /// Y coordinate.
    #[serde(default)]
    pub y: i32,
}

/// Integer size.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
pub struct Size {
    /// Width.
    #[serde(default)]
    pub width: i32,
    /// Height.
    #[serde(default)]
    pub height: i32,
}

/// Integer rectangle.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
pub struct Geometry {
    /// X coordinate.
    #[serde(default)]
    pub x: i32,
    /// Y coordinate.
    #[serde(default)]
    pub y: i32,
    /// Width.
    #[serde(default)]
    pub width: i32,
    /// Height.
    #[serde(default)]
    pub height: i32,
}

/// Window position inside a workspace layout.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
pub struct WindowPosition {
    /// Column index, when the active layout exposes one.
    #[serde(default)]
    pub column_idx: Option<u32>,
    /// Window index inside its column or container.
    #[serde(default)]
    pub window_idx: Option<u32>,
}

/// A Triad workspace with reactive state.
#[derive(Debug, Clone)]
pub struct Workspace {
    /// Stable tag id.
    pub id: Property<u64>,
    /// User-facing workspace index.
    pub idx: Property<u32>,
    /// Optional workspace name.
    pub name: Property<Option<String>>,
    /// Connector name for the output this workspace belongs to.
    pub output: Property<Option<String>>,
    /// Current layout name.
    pub layout: Property<String>,
    /// Whether this workspace has focus.
    pub is_active: Property<bool>,
    /// Whether this workspace is visible on an output.
    pub is_output_visible: Property<bool>,
    /// Whether this workspace contains windows.
    pub occupied: Property<bool>,
    /// Whether any window on this workspace requested attention.
    pub is_urgent: Property<bool>,
    /// Focused window id on this workspace.
    pub focused_window_id: Property<Option<u64>>,
}

impl Workspace {
    pub(crate) fn from_raw(workspace: RawWorkspace) -> Self {
        Self {
            id: Property::new(workspace.tag_id),
            idx: Property::new(workspace.workspace_idx),
            name: Property::new(workspace.name),
            output: Property::new(workspace.output),
            layout: Property::new(workspace.layout),
            is_active: Property::new(workspace.is_active),
            is_output_visible: Property::new(workspace.is_output_visible),
            occupied: Property::new(workspace.occupied),
            is_urgent: Property::new(workspace.is_urgent),
            focused_window_id: Property::new(workspace.focused_window_id),
        }
    }

    pub(crate) fn refresh_from_raw(&self, workspace: RawWorkspace) {
        self.id.set(workspace.tag_id);
        self.idx.set(workspace.workspace_idx);
        self.name.set(workspace.name);
        self.output.set(workspace.output);
        self.layout.set(workspace.layout);
        self.is_active.set(workspace.is_active);
        self.is_output_visible.set(workspace.is_output_visible);
        self.occupied.set(workspace.occupied);
        self.is_urgent.set(workspace.is_urgent);
        self.focused_window_id.set(workspace.focused_window_id);
    }
}

impl PartialEq for Workspace {
    fn eq(&self, other: &Self) -> bool {
        self.id.get() == other.id.get()
    }
}

/// A Triad toplevel window with reactive state.
#[derive(Debug, Clone)]
pub struct Window {
    /// Stable window id.
    pub id: Property<u64>,
    /// Window title if set by the application.
    pub title: Property<Option<String>>,
    /// Wayland application id if set.
    pub app_id: Property<Option<String>>,
    /// PID if Triad can determine it.
    pub pid: Property<Option<i32>>,
    /// Workspace tag id.
    pub tag_id: Property<Option<u64>>,
    /// User-facing workspace index.
    pub workspace_idx: Property<Option<u32>>,
    /// Connector name for the output this window belongs to.
    pub output: Property<Option<String>>,
    /// Position metadata inside the layout.
    pub position: Property<WindowPosition>,
    /// Whether this window has input focus.
    pub is_focused: Property<bool>,
    /// Whether this window is floating.
    pub is_floating: Property<bool>,
}

impl Window {
    pub(crate) fn from_raw(window: RawWindow) -> Self {
        Self {
            id: Property::new(window.id),
            title: Property::new(window.title),
            app_id: Property::new(window.app_id),
            pid: Property::new(window.pid),
            tag_id: Property::new(window.tag_id),
            workspace_idx: Property::new(window.workspace_idx),
            output: Property::new(window.output),
            position: Property::new(window.position),
            is_focused: Property::new(window.is_focused),
            is_floating: Property::new(window.is_floating),
        }
    }

    pub(crate) fn refresh_from_raw(&self, window: RawWindow) {
        self.id.set(window.id);
        self.title.set(window.title);
        self.app_id.set(window.app_id);
        self.pid.set(window.pid);
        self.tag_id.set(window.tag_id);
        self.workspace_idx.set(window.workspace_idx);
        self.output.set(window.output);
        self.position.set(window.position);
        self.is_focused.set(window.is_focused);
        self.is_floating.set(window.is_floating);
    }
}

impl PartialEq for Window {
    fn eq(&self, other: &Self) -> bool {
        self.id.get() == other.id.get()
    }
}

/// Triad output metadata.
#[derive(Debug, Clone)]
pub struct Output {
    /// Stable output id.
    pub id: Property<u64>,
    /// Connector name.
    pub name: Property<String>,
    /// Whether this output is primary.
    pub is_primary: Property<bool>,
    /// Output geometry in logical coordinates.
    pub geometry: Property<Geometry>,
    /// Output scale factor.
    pub scale: Property<f32>,
}

impl Output {
    pub(crate) fn from_raw(output: RawOutput) -> Self {
        Self {
            id: Property::new(output.id),
            name: Property::new(output.name),
            is_primary: Property::new(output.is_primary),
            geometry: Property::new(output.geometry),
            scale: Property::new(output.scale),
        }
    }

    pub(crate) fn refresh_from_raw(&self, output: RawOutput) {
        self.id.set(output.id);
        self.name.set(output.name);
        self.is_primary.set(output.is_primary);
        self.geometry.set(output.geometry);
        self.scale.set(output.scale);
    }
}

impl PartialEq for Output {
    fn eq(&self, other: &Self) -> bool {
        self.name.get() == other.name.get()
    }
}

/// Public event emitted after Triad service properties are refreshed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriadEvent {
    /// A full state snapshot changed.
    StateChanged,
    /// The layout/workspace snapshot changed.
    LayoutStateChanged,
    /// A single window changed.
    WindowChanged {
        /// Window id, when the event contained one.
        window_id: Option<u64>,
    },
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct RawState {
    #[serde(default)]
    pub capabilities: Capabilities,
    #[serde(default)]
    pub overview: RawOverview,
    #[serde(default)]
    pub layout: RawLayoutState,
    #[serde(default, deserialize_with = "deserialize_keyboard_layouts")]
    pub keyboard_layouts: KeyboardLayouts,
    #[serde(default)]
    pub current_keyboard_layout_idx: Option<u32>,
    #[serde(default)]
    pub outputs: Vec<RawOutput>,
    #[serde(default)]
    pub windows: Vec<RawWindow>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct RawOverview {
    #[serde(default)]
    pub is_open: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct RawLayoutState {
    #[serde(default)]
    pub workspaces: Vec<RawWorkspace>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct RawWorkspace {
    #[serde(default)]
    pub tag_id: u64,
    #[serde(default)]
    pub workspace_idx: u32,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub output: Option<String>,
    #[serde(default)]
    pub layout: String,
    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub is_output_visible: bool,
    #[serde(default)]
    pub is_urgent: bool,
    #[serde(default)]
    pub occupied: bool,
    #[serde(default)]
    pub focused_window_id: Option<u64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct RawWindow {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub app_id: Option<String>,
    #[serde(default)]
    pub pid: Option<i32>,
    #[serde(default)]
    pub tag_id: Option<u64>,
    #[serde(default)]
    pub workspace_idx: Option<u32>,
    #[serde(default)]
    pub output: Option<String>,
    #[serde(default)]
    pub position: WindowPosition,
    #[serde(default)]
    pub is_focused: bool,
    #[serde(default)]
    pub is_floating: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct RawOutput {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default)]
    pub geometry: Geometry,
    #[serde(default = "default_scale")]
    pub scale: f32,
}

fn default_scale() -> f32 {
    1.0
}

fn deserialize_keyboard_layouts<'de, D>(deserializer: D) -> Result<KeyboardLayouts, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum KeyboardLayoutsWire {
        Names(Vec<String>),
        Object(KeyboardLayouts),
    }

    match KeyboardLayoutsWire::deserialize(deserializer)? {
        KeyboardLayoutsWire::Names(names) => Ok(KeyboardLayouts {
            names,
            current_idx: 0,
        }),
        KeyboardLayoutsWire::Object(layouts) => Ok(layouts),
    }
}
