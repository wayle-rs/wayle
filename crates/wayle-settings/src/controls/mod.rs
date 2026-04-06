//! One control per config type: toggle, dropdown, slider, spin button, font picker.
//! Each owns its `ConfigProperty` and writes back on user interaction.

pub mod enum_select;
pub mod font;
pub mod number;
pub mod slider;
pub mod toggle;

/// Messages emitted by control components to their parent.
#[derive(Debug)]
pub enum ControlOutput {
    ValueChanged,
}
