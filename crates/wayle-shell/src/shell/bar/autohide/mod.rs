//! Bar autohide state management.

pub(crate) mod hover_trigger;
pub(crate) mod state;

pub(crate) use hover_trigger::HoverTrigger;
pub(crate) use state::{AutohideAction, AutohideState};
