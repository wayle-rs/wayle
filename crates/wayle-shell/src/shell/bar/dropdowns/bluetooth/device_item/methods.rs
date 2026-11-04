use wayle_bluetooth::types::device::{DeviceAction, DeviceActivity, DeviceError};
use zbus::zvariant::OwnedObjectPath;

use super::DeviceItem;
use crate::{
    i18n::{t, td},
    shell::bar::dropdowns::bluetooth::helpers::DeviceSnapshot,
};

impl DeviceItem {
    pub(crate) fn device_path(&self) -> &OwnedObjectPath {
        &self.snapshot.device.object_path
    }

    pub(crate) fn differs_from(&self, snapshot: &DeviceSnapshot) -> bool {
        self.snapshot != *snapshot
    }

    pub(crate) fn update_from_snapshot(&mut self, snapshot: DeviceSnapshot) {
        self.snapshot = snapshot;
    }

    /// Makes the row unclickable for a moment because it moved under the
    /// pointer.
    pub(crate) fn arm_click_guard(&self) {
        if let Some(guard) = &self.click_guard {
            guard.arm();
        }
    }

    /// Makes just the row's buttons unclickable for a moment: they took the
    /// place of the Dismiss button the user just clicked. The rest of the row
    /// stays clickable, so clicking again retries the connection (a click on
    /// a guarded button falls through to the row).
    pub(crate) fn arm_actions_guard(&self) {
        if let Some(guard) = &self.actions_guard {
            guard.arm();
        }
    }

    pub(crate) fn disarm_click_guard(&self) {
        for guard in [&self.click_guard, &self.actions_guard]
            .into_iter()
            .flatten()
        {
            guard.disarm();
        }
    }

    pub(super) fn battery_text(&self) -> String {
        self.snapshot
            .battery
            .map(|percent| t!("dropdown-bluetooth-battery", percent = percent))
            .unwrap_or_default()
    }

    /// Whether the service is running an operation on this device, during
    /// which its actions are disabled.
    pub(super) fn is_busy(&self) -> bool {
        self.snapshot.activity != DeviceActivity::Idle
    }

    pub(crate) fn is_my_device(&self) -> bool {
        self.snapshot.is_mine()
    }

    pub(super) fn status_label(&self) -> String {
        match self.snapshot.activity {
            // Connecting to an unpaired device pairs it as part of the connection.
            DeviceActivity::Connecting | DeviceActivity::Pairing => {
                return t!("dropdown-bluetooth-status-connecting");
            }
            DeviceActivity::Disconnecting => {
                return t!("dropdown-bluetooth-status-disconnecting");
            }
            DeviceActivity::Forgetting => return t!("dropdown-bluetooth-status-forgetting"),
            DeviceActivity::Idle => {}
        }

        if self.shown_error().is_some() {
            return t!("dropdown-bluetooth-status-error");
        }

        if self.snapshot.connected {
            return t!("dropdown-bluetooth-connected");
        }

        if self.snapshot.paired {
            return t!("dropdown-bluetooth-paired");
        }

        String::new()
    }

    pub(super) fn status_visible(&self) -> bool {
        self.snapshot.connected
            || self.snapshot.paired
            || self.is_busy()
            || self.shown_error().is_some()
    }

    /// The badge style of the status, for in-progress and error states;
    /// `None` for settled states, shown as plain text.
    pub(super) fn status_badge(&self) -> Option<&'static str> {
        if self.is_busy() {
            Some("warning")
        } else if self.shown_error().is_some() {
            Some("error")
        } else {
            None
        }
    }

    pub(super) fn status_badge_css_classes(&self) -> Vec<&'static str> {
        let mut classes = vec!["badge-subtle", "bluetooth-device-status"];
        classes.extend(self.status_badge());
        classes
    }

    /// Which page of the row's action area to show: its buttons while hovered
    /// (for a failure, Dismiss, and Disconnect while connected), else its
    /// status.
    pub(super) fn hover_page(&self) -> &'static str {
        if !self.hovered {
            "status"
        } else if self.shown_error().is_some() {
            "error-actions"
        } else if self.actions_available() {
            "actions"
        } else {
            "status"
        }
    }

    /// The failure to display: BlueZ's report of the most recent failed
    /// action, shown while no new operation is under way (starting one clears
    /// it anyway, as does any change of the device's state).
    pub(super) fn shown_error(&self) -> Option<&DeviceError> {
        self.snapshot
            .error
            .as_ref()
            .filter(|_| self.snapshot.activity == DeviceActivity::Idle)
    }

    /// The detail line: what failed, if something did, else the device type.
    pub(super) fn detail_text(&self) -> String {
        let Some(error) = self.shown_error() else {
            return td!(self.snapshot.device_type_key);
        };

        match error.action {
            DeviceAction::Connect => t!("dropdown-bluetooth-error-connect"),
            DeviceAction::Disconnect => t!("dropdown-bluetooth-error-disconnect"),
            DeviceAction::Pair | DeviceAction::CancelPairing => {
                t!("dropdown-bluetooth-error-pair")
            }
            DeviceAction::Forget => t!("dropdown-bluetooth-error-forget"),
            _ => t!("dropdown-bluetooth-error-generic"),
        }
    }

    pub(super) fn detail_css_classes(&self) -> Vec<&'static str> {
        let mut classes = vec!["bluetooth-device-detail"];
        if self.shown_error().is_some() {
            classes.push("error");
        }
        classes
    }

    /// BlueZ's own description of the failure (e.g. `le-connection-abort-by-local`).
    pub(super) fn error_tooltip(&self) -> Option<&str> {
        let error = self.shown_error()?;
        error
            .message()
            .filter(|message| !message.is_empty())
            .or_else(|| error.name())
    }

    pub(super) fn icon_css_classes(&self) -> Vec<&'static str> {
        let mut classes = vec!["bluetooth-device-icon"];
        if self.snapshot.connected {
            classes.push("connected");
        } else if self.snapshot.paired {
            classes.push("paired");
        }
        if self.shown_error().is_some() {
            classes.push("error");
        }
        classes
    }

    pub(super) fn root_css_classes(&self) -> Vec<&'static str> {
        let mut classes = vec!["bluetooth-device"];

        if !self.is_my_device() {
            classes.push("available");
        }

        if self.row_clickable() {
            classes.push("clickable");
        }

        if self.is_busy() {
            classes.push("pending");
        }

        classes
    }

    /// Whether the in-progress operation can be cancelled: BlueZ aborts a
    /// pending `Connect` (and the pairing it may involve) on `Disconnect`.
    fn cancellable(&self) -> bool {
        self.snapshot.activity == DeviceActivity::Connecting
    }

    /// Whether the row's action buttons can be used.
    pub(super) fn actions_available(&self) -> bool {
        !self.is_busy() || self.cancellable()
    }

    /// What the toggle button does, if it is shown. Connecting is done by
    /// clicking the row, so there is no Connect button.
    fn toggle_action(&self) -> Option<ToggleAction> {
        if self.cancellable() {
            Some(ToggleAction::Cancel)
        } else if self.snapshot.connected && !self.is_busy() {
            Some(ToggleAction::Disconnect)
        } else {
            None
        }
    }

    pub(super) fn toggle_visible(&self) -> bool {
        self.toggle_action().is_some()
    }

    /// Forget is offered for the user's own devices, but not while an
    /// operation is in progress: then Cancel is the only action.
    pub(super) fn forget_visible(&self) -> bool {
        self.is_my_device() && !self.is_busy()
    }

    /// Which action buttons the row shows, and what they do. When this
    /// changes (e.g. Cancel becoming Disconnect + Forget as a connection
    /// completes) a click aimed at one button could land on another, so the
    /// row is click-guarded as if it had moved.
    pub(crate) fn actions_layout(&self) -> ActionsLayout {
        ActionsLayout {
            row_clickable: self.row_clickable(),
            toggle: self.toggle_action(),
            forget: self.forget_visible(),
            dismiss: self.shown_error().is_some(),
        }
    }

    pub(super) fn toggle_label(&self) -> String {
        match self.toggle_action() {
            Some(ToggleAction::Cancel) => t!("dropdown-bluetooth-cancel"),
            Some(ToggleAction::Disconnect) | None => t!("dropdown-bluetooth-disconnect"),
        }
    }

    /// The toggle button. Both Disconnect and Cancel disconnect: BlueZ aborts
    /// a pending connect on `Disconnect`.
    pub(super) fn handle_toggle(&self) {
        if self.toggle_action().is_some() {
            self.snapshot.device.disconnect();
        }
    }

    /// Whether clicking the row connects the device (highlighted on hover).
    /// Disconnecting and cancelling take their explicit buttons, so a stray
    /// click can't drop a connection.
    pub(super) fn row_clickable(&self) -> bool {
        !self.is_busy() && !self.snapshot.connected
    }

    /// A click on the row connects the device, if [`row_clickable`](Self::row_clickable).
    pub(super) fn handle_click(&self) {
        if self.row_clickable() {
            self.snapshot.device.connect();
        }
    }

    pub(super) fn handle_forget(&self) {
        if !self.is_busy() {
            self.snapshot.device.forget();
        }
    }
}

/// See [`DeviceItem::actions_layout`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ActionsLayout {
    row_clickable: bool,
    toggle: Option<ToggleAction>,
    forget: bool,
    dismiss: bool,
}

impl ActionsLayout {
    /// Whether the only change from `previous` is Dismiss going away (the error
    /// was dismissed), which reveals the other buttons in its place but
    /// changes nothing else about the row.
    pub(crate) fn only_dismissed(self, previous: Self) -> bool {
        previous.dismiss
            && !self.dismiss
            && Self {
                dismiss: true,
                ..self
            } == previous
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToggleAction {
    Disconnect,
    Cancel,
}

#[cfg(test)]
mod tests {
    use super::{ActionsLayout, ToggleAction};

    fn layout(toggle: Option<ToggleAction>, forget: bool, dismiss: bool) -> ActionsLayout {
        ActionsLayout {
            row_clickable: true,
            toggle,
            forget,
            dismiss,
        }
    }

    #[test]
    fn dismissing_alone_is_only_dismissed() {
        let before = layout(None, true, true);
        let after = layout(None, true, false);
        assert!(after.only_dismissed(before));
    }

    #[test]
    fn other_changes_alongside_dismissal_are_not() {
        // The error cleared because the device connected: Disconnect appeared.
        let before = layout(None, true, true);
        let after = layout(Some(ToggleAction::Disconnect), true, false);
        assert!(!after.only_dismissed(before));
    }

    #[test]
    fn an_error_appearing_is_not() {
        let before = layout(None, true, false);
        let after = layout(None, true, true);
        assert!(!after.only_dismissed(before));
    }
}
