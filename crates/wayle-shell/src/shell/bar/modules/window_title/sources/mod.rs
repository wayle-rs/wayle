//! Compositor-agnostic data source for the window-title module.
//!
//! The widget renders the same UI on every compositor; only "where does the
//! focused window's title and app id come from" differs. The
//! [`FocusedWindowSource`] trait is that boundary, with per-compositor impls
//! in sibling files.

mod hyprland;
mod mango;
mod niri;

use futures::stream::BoxStream;

pub(crate) use self::{
    hyprland::HyprlandFocusedWindowSource, mango::MangoFocusedWindowSource,
    niri::NiriFocusedWindowSource,
};

/// Title and app identifier of a focused window.
///
/// An empty `title` or `app_id` inside a `Some(FocusedWindow)` means the
/// window itself has that field unset. That is distinct from "no focused
/// window," which the source signals via `None` at the [`Option`] layer.
#[derive(Debug, Clone)]
pub(crate) struct FocusedWindow {
    pub title: String,
    pub app_id: String,
}

/// Provides the focused window's identity, abstracted across compositors.
///
/// - [`snapshot`] returns the current state synchronously. Used during init
///   to render the first frame before any events arrive.
/// - [`changes`] returns a stream of subsequent values. The current snapshot
///   is NOT re-emitted; the caller already has it. The stream ends when the
///   underlying compositor connection closes.
///
/// `None` from either method means "no focused window right now" (focus on
/// a layer-shell surface, transient gaps between window close and next
/// focus event, etc.).
///
/// [`snapshot`]: FocusedWindowSource::snapshot
/// [`changes`]: FocusedWindowSource::changes
pub(crate) trait FocusedWindowSource: Send + Sync + 'static {
    fn snapshot(&self) -> Option<FocusedWindow>;
    fn changes(&self) -> BoxStream<'static, Option<FocusedWindow>>;
}
