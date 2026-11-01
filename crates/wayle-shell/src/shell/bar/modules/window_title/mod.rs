//! Window-title bar module: shows the focused window's title and icon.
//! Compositor-agnostic via the [`sources::FocusedWindowSource`] trait.

mod component;
mod factory;
mod helpers;
mod messages;
mod methods;
mod sources;
mod watchers;

pub(crate) use self::{component::WindowTitle, factory::Factory, messages::WindowTitleInit};
