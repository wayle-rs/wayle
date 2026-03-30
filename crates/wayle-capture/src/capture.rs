use std::io::{Read, Seek, SeekFrom};
use std::os::fd::AsFd;
use std::sync::mpsc;

use tracing::{debug, warn};
use wayland_client::protocol::{wl_buffer, wl_registry, wl_shm, wl_shm_pool};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle, WEnum};
use wayland_protocols_hyprland::toplevel_export::v1::client::{
    hyprland_toplevel_export_frame_v1::{self, HyprlandToplevelExportFrameV1},
    hyprland_toplevel_export_manager_v1::HyprlandToplevelExportManagerV1,
};

use crate::{CaptureCommand, CaptureRequest, CaptureResult, STREAM_INTERVAL, WindowThumbnail};

struct CaptureState {
    shm: Option<wl_shm::WlShm>,
    export_manager: Option<HyprlandToplevelExportManagerV1>,
    frame_format: Option<wl_shm::Format>,
    frame_width: u32,
    frame_height: u32,
    frame_stride: u32,
    buffer_done: bool,
    frame_ready: bool,
    frame_failed: bool,
}

impl CaptureState {
    fn new() -> Self {
        Self {
            shm: None,
            export_manager: None,
            frame_format: None,
            frame_width: 0,
            frame_height: 0,
            frame_stride: 0,
            buffer_done: false,
            frame_ready: false,
            frame_failed: false,
        }
    }

    fn reset_frame(&mut self) {
        self.frame_format = None;
        self.frame_width = 0;
        self.frame_height = 0;
        self.frame_stride = 0;
        self.buffer_done = false;
        self.frame_ready = false;
        self.frame_failed = false;
    }
}

// --- Wayland dispatch implementations ---

impl Dispatch<wl_registry::WlRegistry, ()> for CaptureState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "wl_shm" => {
                    state.shm = Some(registry.bind(name, version.min(1), qh, ()));
                }
                "hyprland_toplevel_export_manager_v1" => {
                    state.export_manager = Some(registry.bind(name, version.min(2), qh, ()));
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<wl_shm::WlShm, ()> for CaptureState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm::WlShm,
        _event: wl_shm::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for CaptureState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm_pool::WlShmPool,
        _event: wl_shm_pool::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for CaptureState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_buffer::WlBuffer,
        _event: wl_buffer::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<HyprlandToplevelExportManagerV1, ()> for CaptureState {
    fn event(
        _state: &mut Self,
        _proxy: &HyprlandToplevelExportManagerV1,
        _event: <HyprlandToplevelExportManagerV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<HyprlandToplevelExportFrameV1, ()> for CaptureState {
    fn event(
        state: &mut Self,
        _proxy: &HyprlandToplevelExportFrameV1,
        event: hyprland_toplevel_export_frame_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            hyprland_toplevel_export_frame_v1::Event::Buffer {
                format: WEnum::Value(fmt),
                width,
                height,
                stride,
            } => {
                // Only accept BGRA-compatible formats. Argb8888 is preferred
                // over Xrgb8888 (has alpha), and both are native byte order
                // for the compositing pipeline.
                let dominated = match state.frame_format {
                    None => true,
                    Some(wl_shm::Format::Xrgb8888)
                        if fmt == wl_shm::Format::Argb8888 =>
                    {
                        true
                    }
                    _ => false,
                };
                if dominated
                    && (fmt == wl_shm::Format::Argb8888
                        || fmt == wl_shm::Format::Xrgb8888)
                {
                    state.frame_format = Some(fmt);
                    state.frame_width = width;
                    state.frame_height = height;
                    state.frame_stride = stride;
                }
            }
            hyprland_toplevel_export_frame_v1::Event::BufferDone => {
                state.buffer_done = true;
            }
            hyprland_toplevel_export_frame_v1::Event::Ready { .. } => {
                state.frame_ready = true;
            }
            hyprland_toplevel_export_frame_v1::Event::Failed => {
                state.frame_failed = true;
            }
            _ => {}
        }
    }
}

// --- Capture logic ---

fn parse_window_handle(address: &str) -> Option<u32> {
    let hex = address.strip_prefix("0x").unwrap_or(address);
    u64::from_str_radix(hex, 16).ok().map(|v| v as u32)
}

/// Reusable shared-memory buffer for a single window capture.
struct ShmBuffer {
    mfd: memfd::Memfd,
    size: usize,
}

impl ShmBuffer {
    fn new(size: usize) -> Option<Self> {
        let mfd = memfd::MemfdOptions::default().create("capture").ok()?;
        mfd.as_file().set_len(size as u64).ok()?;
        Some(Self { mfd, size })
    }

}

fn capture_single_window(
    state: &mut CaptureState,
    event_queue: &mut EventQueue<CaptureState>,
    qh: &QueueHandle<CaptureState>,
    handle: u32,
    shm_buf: &mut Option<ShmBuffer>,
) -> Option<(Vec<u8>, u32, u32, u32)> {
    let manager = state.export_manager.clone()?;
    let shm = state.shm.clone()?;

    state.reset_frame();

    let frame = manager.capture_toplevel(0, handle, qh, ());

    // Dispatch until BufferDone or Failed.
    while !state.buffer_done && !state.frame_failed {
        if event_queue.blocking_dispatch(state).is_err() {
            frame.destroy();
            return None;
        }
    }

    if state.frame_failed || state.frame_format.is_none() {
        frame.destroy();
        return None;
    }

    let width = state.frame_width;
    let height = state.frame_height;
    let stride = state.frame_stride;
    let format = state.frame_format?;
    let buf_size = (stride * height) as usize;

    // Reuse or allocate shared memory buffer.
    // Reuse existing buffer if large enough, otherwise allocate.
    let needs_alloc = match shm_buf {
        Some(existing) => existing.size < buf_size,
        None => true,
    };
    if needs_alloc {
        *shm_buf = ShmBuffer::new(buf_size);
    }
    let buf = match shm_buf.as_mut() {
        Some(b) => b,
        None => {
            frame.destroy();
            return None;
        }
    };

    let pool = shm.create_pool(buf.mfd.as_file().as_fd(), buf_size as i32, qh, ());
    let buffer = pool.create_buffer(
        0,
        width as i32,
        height as i32,
        stride as i32,
        format,
        qh,
        (),
    );

    // Reset ready/failed for the copy phase.
    state.frame_ready = false;
    state.frame_failed = false;

    frame.copy(&buffer, 1);

    // Dispatch until Ready or Failed.
    while !state.frame_ready && !state.frame_failed {
        if event_queue.blocking_dispatch(state).is_err() {
            frame.destroy();
            buffer.destroy();
            pool.destroy();
            return None;
        }
    }

    frame.destroy();

    if state.frame_failed {
        buffer.destroy();
        pool.destroy();
        return None;
    }

    // Read pixels from the shared buffer.
    let mut file = buf.mfd.as_file();
    file.seek(SeekFrom::Start(0)).ok()?;
    let mut data = vec![0u8; buf_size];
    file.read_exact(&mut data).ok()?;

    buffer.destroy();
    pool.destroy();

    Some((data, width, height, stride))
}

fn capture_workspace(
    state: &mut CaptureState,
    event_queue: &mut EventQueue<CaptureState>,
    qh: &QueueHandle<CaptureState>,
    request: &CaptureRequest,
    shm_buf: &mut Option<ShmBuffer>,
) -> Option<CaptureResult> {
    if request.clients.is_empty() {
        return None;
    }

    let mut thumbnails = Vec::new();

    for client in &request.clients {
        let handle = match parse_window_handle(&client.address) {
            Some(h) => h,
            None => {
                debug!(address = %client.address, "skipping client with unparseable address");
                continue;
            }
        };

        if let Some((data, width, height, stride)) =
            capture_single_window(state, event_queue, qh, handle, shm_buf)
        {
            thumbnails.push(WindowThumbnail {
                data,
                width,
                height,
                stride,
                x: client.x,
                y: client.y,
                win_width: client.width,
                win_height: client.height,
                address: client.address.clone(),
            });
        }
    }

    if thumbnails.is_empty() {
        return None;
    }

    Some(CaptureResult {
        session: request.session,
        ws_id: request.ws_id,
        thumbnails,
        monitor_width: request.monitor_width,
        monitor_height: request.monitor_height,
    })
}

/// Spawn the capture background thread.
pub(crate) fn spawn() -> (
    mpsc::Sender<CaptureCommand>,
    mpsc::Receiver<CaptureResult>,
) {
    let (cmd_tx, cmd_rx) = mpsc::channel::<CaptureCommand>();
    let (res_tx, res_rx) = mpsc::channel::<CaptureResult>();

    std::thread::Builder::new()
        .name("wayle-capture".into())
        .spawn(#[allow(clippy::cognitive_complexity)] move || {
            let conn = match Connection::connect_to_env() {
                Ok(c) => c,
                Err(e) => {
                    warn!("capture thread: failed to connect to Wayland display: {e}");
                    return;
                }
            };

            let display = conn.display();
            let mut event_queue = conn.new_event_queue::<CaptureState>();
            let qh = event_queue.handle();
            let mut state = CaptureState::new();

            display.get_registry(&qh, ());

            if event_queue.roundtrip(&mut state).is_err() {
                warn!("capture thread: Wayland roundtrip failed");
                return;
            }

            if state.export_manager.is_none() {
                warn!(
                    "capture thread: hyprland_toplevel_export_manager_v1 not available — \
                     workspace previews will not show thumbnails"
                );
                return;
            }
            if state.shm.is_none() {
                warn!("capture thread: wl_shm not available");
                return;
            }

            // Reusable shared-memory buffer across captures.
            let mut shm_buf: Option<ShmBuffer> = None;

            loop {
                // Block until the first command arrives.
                let cmd = match cmd_rx.recv() {
                    Ok(c) => c,
                    Err(_) => return,
                };

                // Drain to latest command.
                let mut latest = cmd;
                while let Ok(newer) = cmd_rx.try_recv() {
                    latest = newer;
                }

                let mut request = match latest {
                    CaptureCommand::StartStreaming(req) => req,
                    CaptureCommand::StopStreaming => continue,
                };

                // Streaming loop: capture, send result, wait for interval or
                // new command. Captures are serial — if one frame takes longer
                // than the interval, the next frame starts immediately (no
                // overlap, no frame queue).
                loop {
                    if let Some(result) = capture_workspace(
                        &mut state,
                        &mut event_queue,
                        &qh,
                        &request,
                        &mut shm_buf,
                    )
                        && res_tx.send(result).is_err()
                    {
                        return; // receiver dropped
                    }

                    // Wait for either a new command or the stream interval.
                    match cmd_rx.recv_timeout(STREAM_INTERVAL) {
                        Ok(CaptureCommand::StartStreaming(new_req)) => {
                            // New streaming request — drain to latest and switch.
                            request = new_req;
                            let mut stopped = false;
                            while let Ok(newer) = cmd_rx.try_recv() {
                                match newer {
                                    CaptureCommand::StartStreaming(r) => request = r,
                                    CaptureCommand::StopStreaming => {
                                        stopped = true;
                                        break;
                                    }
                                }
                            }
                            if stopped {
                                break; // Exit streaming loop → back to idle.
                            }
                        }
                        Ok(CaptureCommand::StopStreaming) => {
                            break; // Return to idle (outer blocking recv).
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            // Interval elapsed, capture next frame.
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => {
                            return; // sender dropped
                        }
                    }
                }
            }
        })
        .map_err(|e| {
            warn!("capture thread: failed to spawn thread: {e}");
        })
        .ok();

    (cmd_tx, res_rx)
}
