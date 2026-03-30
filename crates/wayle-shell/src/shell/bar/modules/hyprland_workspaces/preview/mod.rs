mod compositing;

use std::{
    cell::RefCell,
    fmt,
    rc::Rc,
    sync::{mpsc, Arc},
};

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_capture::{CaptureClient, CaptureCommand, CaptureRequest, CaptureResult};
use wayle_hyprland::{HyprlandService, WorkspaceId};
use wayle_widgets::prelude::BarSettings;

use self::compositing::ClickRegion;

// ---------------------------------------------------------------------------
// Public messages
// ---------------------------------------------------------------------------

/// Input messages for the preview component.
pub(crate) enum WorkspacePreviewMsg {
    /// Show preview for a workspace and start streaming captures.
    Show {
        ws_id: WorkspaceId,
        hyprland: Option<Arc<HyprlandService>>,
        settings: Box<BarSettings>,
        preview_width: u32,
    },
    /// Stop streaming (popup was closed externally).
    Hide,
    /// A capture result is ready to composite.
    CaptureReady(CaptureResult),
    /// User clicked a window in the composite thumbnail.
    ThumbnailClicked(String),
    /// User clicked a window label in the list.
    LabelClicked(String),
}

impl fmt::Debug for WorkspacePreviewMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Show { ws_id, .. } => f.debug_struct("Show").field("ws_id", ws_id).finish(),
            Self::Hide => write!(f, "Hide"),
            Self::CaptureReady(_) => write!(f, "CaptureReady(..)"),
            Self::ThumbnailClicked(a) => write!(f, "ThumbnailClicked({a})"),
            Self::LabelClicked(a) => write!(f, "LabelClicked({a})"),
        }
    }
}

/// Output messages sent to the parent `HyprlandWorkspaces`.
#[derive(Debug)]
pub(crate) enum WorkspacePreviewOutput {
    /// Request the parent to focus a window by address.
    FocusWindow(String),
    /// Request the parent to close the popover.
    Dismiss,
}

// ---------------------------------------------------------------------------
// Init
// ---------------------------------------------------------------------------

pub(crate) struct WorkspacePreviewInit {}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

pub(crate) struct WorkspacePreview {
    preview_width: f64,
    hovered_ws: Option<WorkspaceId>,
    session: u64,
    click_regions: Rc<RefCell<Vec<ClickRegion>>>,
    popup_items: Vec<(String, gtk::Button)>,
    highlight_items: Rc<RefCell<Vec<(String, gtk::Button)>>>,
    capture_tx: mpsc::Sender<CaptureCommand>,
    header_label: gtk::Label,
    labels_box: gtk::Box,
    preview_picture: gtk::Picture,
}

#[relm4::component(pub(crate))]
impl Component for WorkspacePreview {
    type Init = WorkspacePreviewInit;
    type Input = WorkspacePreviewMsg;
    type Output = WorkspacePreviewOutput;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            add_css_class: "ws-preview-popup",
            set_spacing: 2,

            gtk::Box {
                add_css_class: "dropdown-header",

                gtk::Box {
                    add_css_class: "dropdown-title",
                    gtk::Image {
                        set_icon_name: Some("ld-layout-grid-symbolic"),
                    },
                    #[local_ref]
                    header_label -> gtk::Label {},
                },
            },

            gtk::Box {
                add_css_class: "dropdown-content",
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 2,

                #[local_ref]
                preview_picture -> gtk::Picture {
                    add_css_class: "ws-preview-canvas",
                    set_can_shrink: true,
                    set_visible: false,
                },

                #[local_ref]
                labels_box -> gtk::Box {
                    add_css_class: "ws-preview-labels",
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 2,
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Spawn capture thread.
        let (capture_tx, capture_rx) = wayle_capture::spawn_capture_thread();

        let click_regions: Rc<RefCell<Vec<ClickRegion>>> = Rc::new(RefCell::new(Vec::new()));

        let header_label = gtk::Label::new(Some("Workspace"));
        let preview_picture = gtk::Picture::new();
        let labels_box = gtk::Box::default();

        // Bridge capture results into the glib main loop on demand.
        let (bridge_tx, mut bridge_rx) = futures::channel::mpsc::unbounded();
        std::thread::Builder::new()
            .name("wayle-capture-bridge".into())
            .spawn(move || {
                while let Ok(result) = capture_rx.recv() {
                    if bridge_tx.unbounded_send(result).is_err() {
                        return;
                    }
                }
            })
            .ok();
        let bridge_sender = sender.input_sender().clone();
        glib::spawn_future_local(async move {
            use futures::StreamExt;
            while let Some(result) = bridge_rx.next().await {
                bridge_sender.emit(WorkspacePreviewMsg::CaptureReady(result));
            }
        });

        // Set up click/hover controllers on the preview picture.
        let highlight_items: Rc<RefCell<Vec<(String, gtk::Button)>>> =
            Rc::new(RefCell::new(Vec::new()));
        Self::attach_thumbnail_controllers(
            &preview_picture,
            &click_regions,
            &highlight_items,
            &sender,
        );

        let model = Self {
            preview_width: 640.0,
            hovered_ws: None,
            session: 0,
            click_regions,
            popup_items: Vec::new(),
            highlight_items,
            capture_tx,
            header_label: header_label.clone(),
            labels_box: labels_box.clone(),
            preview_picture: preview_picture.clone(),
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            WorkspacePreviewMsg::Show {
                ws_id,
                hyprland,
                settings,
                preview_width,
            } => {
                self.preview_width = f64::from(preview_width);
                self.show_for_workspace(ws_id, hyprland.as_deref(), &settings, &sender);
            }
            WorkspacePreviewMsg::Hide => {
                self.stop_streaming();
                self.hovered_ws = None;
            }
            WorkspacePreviewMsg::CaptureReady(result) => {
                if result.session == self.session && self.hovered_ws == Some(result.ws_id) {
                    compositing::apply_capture_result(
                        &self.preview_picture,
                        &result,
                        &self.click_regions,
                        self.preview_width,
                    );
                }
            }
            WorkspacePreviewMsg::ThumbnailClicked(address)
            | WorkspacePreviewMsg::LabelClicked(address) => {
                self.stop_streaming();
                self.hovered_ws = None;
                sender
                    .output(WorkspacePreviewOutput::FocusWindow(address))
                    .ok();
                sender.output(WorkspacePreviewOutput::Dismiss).ok();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Private methods
// ---------------------------------------------------------------------------

impl WorkspacePreview {
    fn attach_thumbnail_controllers(
        picture: &gtk::Picture,
        click_regions: &Rc<RefCell<Vec<ClickRegion>>>,
        highlight_items: &Rc<RefCell<Vec<(String, gtk::Button)>>>,
        sender: &ComponentSender<Self>,
    ) {
        // Click on thumbnail → focus window.
        let click = gtk::GestureClick::new();
        let regions_ref = click_regions.clone();
        let click_sender = sender.input_sender().clone();
        click.connect_released(move |_, _, x, y| {
            let regions = regions_ref.borrow();
            for region in regions.iter() {
                if x >= region.x
                    && x < region.x + region.w
                    && y >= region.y
                    && y < region.y + region.h
                {
                    click_sender.emit(WorkspacePreviewMsg::ThumbnailClicked(
                        region.address.clone(),
                    ));
                    break;
                }
            }
        });
        picture.add_controller(click);

        // Hover on thumbnail → highlight corresponding label.
        let motion = gtk::EventControllerMotion::new();
        let regions_ref = click_regions.clone();
        let items_ref = highlight_items.clone();
        motion.connect_motion(move |_, x, y| {
            let regions = regions_ref.borrow();
            let mut matched_addr: Option<&str> = None;
            for region in regions.iter() {
                if x >= region.x
                    && x < region.x + region.w
                    && y >= region.y
                    && y < region.y + region.h
                {
                    matched_addr = Some(&region.address);
                    break;
                }
            }
            let items = items_ref.borrow();
            for (addr, btn) in items.iter() {
                if matched_addr == Some(addr.as_str()) {
                    btn.add_css_class("preview-highlight");
                } else {
                    btn.remove_css_class("preview-highlight");
                }
            }
        });
        let items_ref = highlight_items.clone();
        motion.connect_leave(move |_| {
            let items = items_ref.borrow();
            for (_, btn) in items.iter() {
                btn.remove_css_class("preview-highlight");
            }
        });
        picture.add_controller(motion);
    }

    fn stop_streaming(&self) {
        let _ = self.capture_tx.send(CaptureCommand::StopStreaming);
    }

    fn show_for_workspace(
        &mut self,
        ws_id: WorkspaceId,
        hyprland: Option<&HyprlandService>,
        settings: &BarSettings,
        sender: &ComponentSender<Self>,
    ) {
        self.session = self.session.wrapping_add(1);
        self.hovered_ws = Some(ws_id);
        self.header_label.set_label(&format!("Workspace {ws_id}"));

        // Clear previous labels.
        while let Some(child) = self.labels_box.first_child() {
            self.labels_box.remove(&child);
        }
        self.popup_items.clear();
        self.highlight_items.borrow_mut().clear();

        // Hide preview until capture arrives.
        self.preview_picture
            .set_paintable(None::<&gdk4::MemoryTexture>);
        self.preview_picture.set_visible(false);

        let Some(hyprland) = hyprland else {
            self.show_empty_label();
            return;
        };

        // Gather clients in this workspace.
        let clients = hyprland.clients.get();
        let ws_clients: Vec<_> = clients
            .iter()
            .filter(|c| {
                let ws = c.workspace.get();
                let sz = c.size.get();
                ws.id == ws_id && c.mapped.get() && sz.width > 0 && sz.height > 0
            })
            .collect();

        if ws_clients.is_empty() {
            self.show_empty_label();
            return;
        }

        // Build label buttons for each client.
        for client in &ws_clients {
            let class = client.class.get();
            let title = client.title.get();
            let text = if title.is_empty() {
                class.clone()
            } else {
                format!("{class}: {}", truncate_title(&title, 40))
            };

            let btn = gtk::Button::new();
            btn.add_css_class("ws-popup-item");
            let addr_str = client.address.get().to_string();
            btn.set_widget_name(&format!("ws-popup-addr-{addr_str}"));

            let label = gtk::Label::new(Some(&text));
            label.set_halign(gtk::Align::Start);
            btn.set_child(Some(&label));

            let btn_sender = sender.input_sender().clone();
            let addr_clone = addr_str.clone();
            btn.connect_clicked(move |_| {
                btn_sender.emit(WorkspacePreviewMsg::LabelClicked(addr_clone.clone()));
            });

            self.popup_items.push((addr_str, btn.clone()));
            self.labels_box.append(&btn);
        }

        // Sync highlight items for the thumbnail hover closure.
        *self.highlight_items.borrow_mut() = self.popup_items.clone();

        // Gather monitor info for the capture request.
        let monitors = hyprland.monitors.get();
        let monitor = monitors.iter().find(|m| {
            settings
                .monitor_name
                .as_deref()
                .is_some_and(|name| m.name.get() == name)
        });

        if let Some(monitor) = monitor {
            let mon_x = monitor.x.get();
            let mon_y = monitor.y.get();
            let scale = monitor.scale.get() as f64;
            let mon_w = (f64::from(monitor.width.get()) / scale) as u32;
            let mon_h = (f64::from(monitor.height.get()) / scale) as u32;

            let capture_clients: Vec<CaptureClient> = ws_clients
                .iter()
                .map(|c| {
                    let addr = c.address.get().to_string();
                    let loc = c.at.get();
                    let sz = c.size.get();
                    CaptureClient {
                        address: addr,
                        x: loc.x - mon_x,
                        y: loc.y - mon_y,
                        width: sz.width,
                        height: sz.height,
                    }
                })
                .collect();

            let _ = self.capture_tx.send(CaptureCommand::StartStreaming(
                CaptureRequest {
                    session: self.session,
                    ws_id,
                    monitor_width: mon_w,
                    monitor_height: mon_h,
                    clients: capture_clients,
                },
            ));
        }
    }

    fn show_empty_label(&self) {
        let label = gtk::Label::new(Some("(empty)"));
        label.add_css_class("ws-popup-item");
        label.add_css_class("dim");
        label.set_halign(gtk::Align::Start);
        self.labels_box.append(&label);
    }
}

fn truncate_title(title: &str, max_len: usize) -> String {
    let char_count = title.chars().count();
    if char_count <= max_len {
        return title.to_string();
    }
    let end: usize = title
        .char_indices()
        .nth(max_len)
        .map(|(i, _)| i)
        .unwrap_or(title.len());
    format!("{}...", &title[..end])
}
