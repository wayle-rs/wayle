//! Compositor-agnostic data source for the keyboard-input module.
//!
//! The widget renders the same UI on every compositor; only "where does the
//! current keyboard layout come from" differs. The [`KeyboardLayoutSource`]
//! trait is that boundary, with per-compositor impls in sibling files.

mod hyprland;
mod niri;

use futures::stream::BoxStream;

pub(crate) use self::{hyprland::HyprlandKeyboardLayoutSource, niri::NiriKeyboardLayoutSource};

/// Currently active keyboard layout.
#[derive(Debug, Clone)]
pub(crate) struct CurrentLayout {
    pub label: String,
}

/// Provides the current keyboard layout, abstracted across compositors.
///
/// `None` from either method means no keyboard layout is currently available
/// (no keyboard device, service unavailable, or no events received yet).
pub(crate) trait KeyboardLayoutSource: Send + Sync + 'static {
    fn snapshot(&self) -> Option<CurrentLayout>;
    fn changes(&self) -> BoxStream<'static, Option<CurrentLayout>>;
}
