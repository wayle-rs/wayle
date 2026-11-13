//! Committing a window's `wl_surface` directly, without waiting for GTK to paint.
//!
//! gtk4-layer-shell sends layer-surface requests (layer, keyboard mode, ...) right
//! away, but leaves the `wl_surface.commit` that applies them to GTK's next paint.
//! GTK only paints once the compositor answers the previous frame callback, and
//! compositors may withhold those from surfaces they don't show (`wl_surface.frame`:
//! "A server should avoid signaling the frame callbacks if the surface is not
//! visible in any way"). A bar hidden behind a fullscreen window would then never
//! get raised above it. gtk-layer-shell (GTK3) has `gtk_layer_try_force_commit` for
//! this; gtk4-layer-shell has no equivalent yet.
//!
//! A commit without a new `attach` keeps the current buffer, so nothing is redrawn.

#![allow(unsafe_code)]

use std::{ffi::c_void, ptr};

use relm4::gtk::{self, glib, prelude::*};
use tracing::debug;

/// `wl_surface.commit` is the seventh `wl_surface` request.
const WL_SURFACE_COMMIT: u32 = 6;

// No `#[link]`: libwayland-client and libgtk-4 are already linked, and the shell's
// build script owns the link order that gtk4-layer-shell's interposition needs.
unsafe extern "C" {
    fn gdk_wayland_surface_get_wl_surface(surface: *mut c_void) -> *mut c_void;
    fn wl_proxy_get_version(proxy: *mut c_void) -> u32;
    fn wl_proxy_marshal_flags(
        proxy: *mut c_void,
        opcode: u32,
        interface: *const c_void,
        version: u32,
        flags: u32,
        ...
    ) -> *mut c_void;
}

/// Commits the window's `wl_surface` now, applying its pending layer-surface state
/// with the buffer it already shows.
///
/// Call this on the GTK main thread from an ordinary handler (input, IPC, ...), not
/// from inside a frame clock phase. GTK and the GL/Vulkan presentation code attach
/// and commit each frame within one frame clock cycle on the main thread, so between
/// cycles nothing they sent is left uncommitted for this to publish early.
pub(crate) fn commit_now(window: &gtk::Window) {
    debug_assert!(
        glib::MainContext::default().is_owner(),
        "commit_now must run on the GTK main thread"
    );

    let Some(surface) = window.surface() else {
        return;
    };

    if !surface.type_().name().starts_with("GdkWayland") {
        debug!(surface_type = %surface.type_().name(), "not a Wayland surface, skipping commit");
        return;
    }

    // SAFETY: `surface` is a live GdkWaylandSurface (checked above).
    let wl_surface = unsafe { gdk_wayland_surface_get_wl_surface(surface.as_ptr().cast()) };
    if wl_surface.is_null() {
        return;
    }

    // SAFETY: `wl_surface` is GDK's live `wl_surface` proxy, and `commit` takes no
    // arguments. This is what libwayland's `wl_surface_commit` expands to.
    unsafe {
        wl_proxy_marshal_flags(
            wl_surface,
            WL_SURFACE_COMMIT,
            ptr::null(),
            wl_proxy_get_version(wl_surface),
            0,
        );
    }

    surface.display().flush();
}
