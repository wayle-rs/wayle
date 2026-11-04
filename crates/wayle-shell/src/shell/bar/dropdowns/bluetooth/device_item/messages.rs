use crate::shell::bar::dropdowns::bluetooth::helpers::DeviceSnapshot;

pub(crate) struct DeviceItemInit {
    pub snapshot: DeviceSnapshot,
}

#[derive(Debug)]
pub(crate) enum DeviceItemInput {
    /// The row itself was clicked.
    Clicked,
    /// The Disconnect / Cancel button was clicked.
    ToggleClicked,
    Hovered(bool),
    ForgetClicked,
    DismissClicked,
}
