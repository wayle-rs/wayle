#[derive(Debug)]
pub(crate) enum PasswordFormInput {
    Show {
        ssid: String,
        security_label: String,
        signal_icon: String,
        error_message: Option<String>,
    },
    ConnectClicked,
    CancelClicked,
    /// Close and reset the form without emitting an output, for when the target
    /// disappears out from under it (WiFi disabled, or the station device gone).
    /// This releases the entry's focus so the popover's focus/grab machinery does
    /// not later trip over a hidden-but-focused entry.
    Hide,
    /// Update the displayed signal icon without resetting the entry (used when
    /// the icon config changes while the form is open).
    SetSignalIcon(String),
}

#[derive(Debug)]
pub(crate) enum PasswordFormOutput {
    Connect { password: String },
    Cancel,
}
