//! Reactive Triad service.

use std::{collections::HashMap, sync::Arc};

use derive_more::Debug;
use futures::Stream;
use serde_json::{Map, Value, json};
use tokio::sync::broadcast;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};
use wayle_core::Property;

use crate::{
    error::Result,
    ipc::{EventMessage, EventStream, TriadCommandClient},
    types::{
        Capabilities, KeyboardLayoutTarget, KeyboardLayouts, Output, RawLayoutState, RawOutput,
        RawState, RawWindow, RawWorkspace, TriadEvent, Window, Workspace,
    },
};

const EVENT_CHANNEL_CAPACITY: usize = 256;

/// Reactive bindings to the Triad compositor.
#[derive(Debug)]
pub struct TriadService {
    #[debug(skip)]
    cancellation_token: CancellationToken,
    #[debug(skip)]
    command_client: Arc<TriadCommandClient>,
    #[debug(skip)]
    public_event_tx: broadcast::Sender<TriadEvent>,

    /// All workspaces keyed by Triad tag id.
    pub workspaces: Property<HashMap<u64, Arc<Workspace>>>,
    /// All open toplevel windows keyed by id.
    pub windows: Property<HashMap<u64, Arc<Window>>>,
    /// Outputs keyed by connector name.
    pub outputs: Property<HashMap<String, Arc<Output>>>,
    /// Configured keyboard layouts and the active index.
    pub keyboard_layouts: Property<Option<KeyboardLayouts>>,
    /// Id of the currently focused window.
    pub focused_window_id: Property<Option<u64>>,
    /// Whether Triad's overview is visible.
    pub overview_open: Property<bool>,
    /// Current native IPC capabilities.
    pub capabilities: Property<Option<Capabilities>>,
}

impl TriadService {
    /// Connects to Triad, loads the initial state, and subscribes to events.
    ///
    /// # Errors
    ///
    /// Returns an error when Triad's native socket cannot be opened or the
    /// initial state query fails.
    pub async fn new() -> Result<Arc<Self>> {
        let cancellation_token = CancellationToken::new();
        let command_client = Arc::new(TriadCommandClient::connect()?);
        let (public_event_tx, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);

        let service = Arc::new(Self {
            cancellation_token,
            command_client,
            public_event_tx,
            workspaces: Property::new(HashMap::new()),
            windows: Property::new(HashMap::new()),
            outputs: Property::new(HashMap::new()),
            keyboard_layouts: Property::new(None),
            focused_window_id: Property::new(None),
            overview_open: Property::new(false),
            capabilities: Property::new(None),
        });

        let initial_state = service.command_client.state().await?;
        service.apply_state(initial_state);
        service.start_event_task();

        Ok(service)
    }

    /// Looks up a window by id in the current snapshot.
    pub fn window(&self, id: u64) -> Option<Arc<Window>> {
        self.windows.get().get(&id).cloned()
    }

    /// Looks up a workspace by tag id in the current snapshot.
    pub fn workspace(&self, id: u64) -> Option<Arc<Workspace>> {
        self.workspaces.get().get(&id).cloned()
    }

    /// Returns a stream of Triad events after properties have been refreshed.
    pub fn events(&self) -> impl Stream<Item = TriadEvent> + Send + 'static {
        let receiver = self.public_event_tx.subscribe();
        BroadcastStream::new(receiver).filter_map(|received| received.ok())
    }

    /// Dispatches a raw Triad native action.
    ///
    /// # Errors
    ///
    /// Surfaces any socket, JSON, or compositor rejection error.
    pub async fn dispatch_action(&self, action: &str, extra: Map<String, Value>) -> Result<()> {
        self.command_client.dispatch_action(action, extra).await
    }

    /// Focuses a workspace by tag id.
    ///
    /// # Errors
    /// See [`TriadService::dispatch_action`].
    pub async fn focus_tag(&self, tag: u64) -> Result<()> {
        self.dispatch_action("focus-tag", Map::from_iter([("tag".into(), json!(tag))]))
            .await
    }

    /// Focuses a workspace by user-facing workspace index.
    ///
    /// # Errors
    /// See [`TriadService::dispatch_action`].
    pub async fn focus_workspace(&self, workspace_idx: u32) -> Result<()> {
        self.dispatch_action(
            "focus-workspace",
            Map::from_iter([("workspace_idx".into(), json!(workspace_idx))]),
        )
        .await
    }

    /// Focuses the previous tag.
    ///
    /// # Errors
    /// See [`TriadService::dispatch_action`].
    pub async fn focus_tag_left(&self) -> Result<()> {
        self.dispatch_action("focus-tag-left", Map::new()).await
    }

    /// Focuses the next tag.
    ///
    /// # Errors
    /// See [`TriadService::dispatch_action`].
    pub async fn focus_tag_right(&self) -> Result<()> {
        self.dispatch_action("focus-tag-right", Map::new()).await
    }

    /// Focuses a window by id.
    ///
    /// # Errors
    /// See [`TriadService::dispatch_action`].
    pub async fn focus_window(&self, id: u64) -> Result<()> {
        self.dispatch_action("focus-window", Map::from_iter([("id".into(), json!(id))]))
            .await
    }

    /// Closes a window. `None` closes the currently focused window.
    ///
    /// # Errors
    /// See [`TriadService::dispatch_action`].
    pub async fn close_window(&self, id: Option<u64>) -> Result<()> {
        let extra = id.map_or_else(Map::new, |window_id| {
            Map::from_iter([("id".into(), json!(window_id))])
        });
        self.dispatch_action("close-window", extra).await
    }

    /// Spawns a command. The first element is the executable.
    ///
    /// # Errors
    /// See [`TriadService::dispatch_action`].
    pub async fn spawn(&self, argv: Vec<String>) -> Result<()> {
        self.dispatch_action("spawn", Map::from_iter([("argv".into(), json!(argv))]))
            .await
    }

    /// Switches keyboard layout.
    ///
    /// # Errors
    /// See [`TriadService::dispatch_action`].
    pub async fn switch_keyboard_layout(&self, target: KeyboardLayoutTarget) -> Result<()> {
        let value = match target {
            KeyboardLayoutTarget::Next => json!("next"),
            KeyboardLayoutTarget::Previous => json!("prev"),
            KeyboardLayoutTarget::Index(index) => json!(index),
        };
        self.dispatch_action(
            "switch-keyboard-layout",
            Map::from_iter([("layout".into(), value)]),
        )
        .await
    }

    /// Toggles the overview.
    ///
    /// # Errors
    /// See [`TriadService::dispatch_action`].
    pub async fn toggle_overview(&self) -> Result<()> {
        self.dispatch_action("toggle-overview", Map::new()).await
    }

    /// Switches to the next configured layout mode.
    ///
    /// # Errors
    /// See [`TriadService::dispatch_action`].
    pub async fn switch_layout(&self) -> Result<()> {
        self.dispatch_action("switch-layout", Map::new()).await
    }

    fn start_event_task(self: &Arc<Self>) {
        let service = Arc::clone(self);
        tokio::spawn(async move {
            loop {
                if service.cancellation_token.is_cancelled() {
                    break;
                }
                match EventStream::connect().await {
                    Ok(mut stream) => {
                        debug!("triad event stream connected");
                        service.drain_event_stream(&mut stream).await;
                    }
                    Err(error) => {
                        warn!(?error, "triad event stream connection failed");
                    }
                }
                tokio::select! {
                    () = service.cancellation_token.cancelled() => break,
                    () = tokio::time::sleep(std::time::Duration::from_secs(1)) => {}
                }
            }
        });
    }

    async fn drain_event_stream(&self, stream: &mut EventStream) {
        loop {
            tokio::select! {
                () = self.cancellation_token.cancelled() => break,
                message = stream.next_message() => match message {
                    Ok(Some(message)) => self.apply_event(message),
                    Ok(None) => break,
                    Err(error) => {
                        warn!(?error, "triad event stream failed");
                        break;
                    }
                }
            }
        }
    }

    fn apply_event(&self, message: EventMessage) {
        let event = message.event();
        match message {
            EventMessage::State(state) => self.apply_state(state),
            EventMessage::Layout(layout) => self.apply_layout(layout),
            EventMessage::Window(window) => self.apply_window(window),
        }
        let _ = self.public_event_tx.send(event);
    }

    fn apply_state(&self, mut state: RawState) {
        if let Some(current_idx) = state.current_keyboard_layout_idx {
            state.keyboard_layouts.current_idx = current_idx;
        }

        self.capabilities.set(Some(state.capabilities));
        self.overview_open.set(state.overview.is_open);
        self.keyboard_layouts.set(Some(state.keyboard_layouts));
        self.apply_layout(state.layout);
        self.apply_outputs(state.outputs);
        self.apply_windows(state.windows);
        self.refresh_focused_window();
    }

    fn apply_layout(&self, layout: RawLayoutState) {
        self.apply_workspaces(layout.workspaces);
        self.refresh_focused_window();
    }

    fn apply_workspaces(&self, workspaces: Vec<RawWorkspace>) {
        let mut current = self.workspaces.get();
        let mut next = HashMap::with_capacity(workspaces.len());
        for workspace in workspaces {
            if let Some(existing) = current.remove(&workspace.tag_id) {
                existing.refresh_from_raw(workspace);
                next.insert(existing.id.get(), existing);
            } else {
                let created = Arc::new(Workspace::from_raw(workspace));
                next.insert(created.id.get(), created);
            }
        }
        self.workspaces.set(next);
    }

    fn apply_windows(&self, windows: Vec<RawWindow>) {
        let mut current = self.windows.get();
        let mut next = HashMap::with_capacity(windows.len());
        for window in windows {
            if let Some(existing) = current.remove(&window.id) {
                existing.refresh_from_raw(window);
                next.insert(existing.id.get(), existing);
            } else {
                let created = Arc::new(Window::from_raw(window));
                next.insert(created.id.get(), created);
            }
        }
        self.windows.set(next);
    }

    fn apply_window(&self, window: RawWindow) {
        let mut current = self.windows.get();
        if let Some(existing) = current.get(&window.id) {
            existing.refresh_from_raw(window);
        } else {
            current.insert(window.id, Arc::new(Window::from_raw(window)));
        }
        self.windows.set(current);
        self.refresh_focused_window();
    }

    fn apply_outputs(&self, outputs: Vec<RawOutput>) {
        let mut current = self.outputs.get();
        let mut next = HashMap::with_capacity(outputs.len());
        for output in outputs {
            if let Some(existing) = current.remove(&output.name) {
                existing.refresh_from_raw(output);
                next.insert(existing.name.get(), existing);
            } else {
                let created = Arc::new(Output::from_raw(output));
                next.insert(created.name.get(), created);
            }
        }
        self.outputs.set(next);
    }

    fn refresh_focused_window(&self) {
        let focused = self
            .windows
            .get()
            .values()
            .find(|window| window.is_focused.get())
            .map(|window| window.id.get())
            .or_else(|| {
                self.workspaces
                    .get()
                    .values()
                    .find(|workspace| workspace.is_active.get())
                    .and_then(|workspace| workspace.focused_window_id.get())
            });
        self.focused_window_id.set(focused);
    }
}

impl Drop for TriadService {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}
