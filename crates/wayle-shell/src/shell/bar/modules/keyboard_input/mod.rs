//! Keyboard-input bar module: shows the active keyboard layout.
//! Compositor-agnostic via the [`sources::KeyboardLayoutSource`] trait.

mod component;
mod factory;
mod helpers;
mod messages;
mod methods;
mod sources;
mod watchers;

pub(crate) use self::{component::KeyboardInput, factory::Factory, messages::KeyboardInputInit};
