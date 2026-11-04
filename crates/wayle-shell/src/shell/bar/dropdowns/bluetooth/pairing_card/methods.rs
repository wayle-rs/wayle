use gtk::prelude::*;
use relm4::{
    gtk,
    gtk::{gdk, glib},
};
use wayle_bluetooth::types::agent::PairingRequest;

use super::{PairingCard, PairingVariant};
use crate::{
    i18n::{t, td},
    shell::bar::dropdowns::bluetooth::helpers::{
        DeviceDisplayInfo, format_passkey, resolve_device_display, service_name_key,
    },
};

enum PinKeyAction {
    Digit(char),
    Backspace,
    PassThrough,
}

impl PairingCard {
    pub(super) fn clear_inputs(&self) {
        for entry in &self.pin_entries {
            entry.set_text("");
        }
        self.legacy_pin_entry.set_text("");
    }

    pub(super) fn variant(&self) -> PairingVariant {
        match &self.request {
            None => PairingVariant::None,
            Some(PairingRequest::DisplayPinCode { .. }) => PairingVariant::DisplayPin,
            Some(PairingRequest::RequestPasskey { .. }) => PairingVariant::RequestPasskey,
            Some(PairingRequest::DisplayPasskey { .. }) => PairingVariant::DisplayPasskey,
            Some(PairingRequest::RequestConfirmation { .. }) => PairingVariant::RequestConfirmation,
            Some(PairingRequest::RequestAuthorization { .. }) => {
                PairingVariant::RequestAuthorization
            }
            Some(PairingRequest::RequestServiceAuthorization { .. }) => {
                PairingVariant::RequestServiceAuthorization
            }
            Some(PairingRequest::RequestPinCode { .. }) => PairingVariant::RequestPinCode,
        }
    }

    /// How the request's device is shown, as the service has it now.
    pub(super) fn display(&self) -> DeviceDisplayInfo {
        self.request
            .as_ref()
            .zip(self.bluetooth.get())
            .and_then(|(request, bluetooth)| bluetooth.device(request.device_path()))
            .map(|device| resolve_device_display(&device.info.get()))
            .unwrap_or_default()
    }

    /// The PIN or passkey shown, if the request shows one.
    pub(super) fn pin_code(&self) -> String {
        match &self.request {
            Some(PairingRequest::DisplayPinCode { pincode, .. }) => pincode.clone(),
            Some(
                PairingRequest::DisplayPasskey { passkey, .. }
                | PairingRequest::RequestConfirmation { passkey, .. },
            ) => format_passkey(*passkey),
            _ => String::new(),
        }
    }

    /// How many digits of a displayed passkey were typed on the device.
    pub(super) fn passkey_entered(&self) -> u16 {
        match &self.request {
            Some(PairingRequest::DisplayPasskey { entered, .. }) => *entered,
            _ => 0,
        }
    }

    /// The service a service authorization is for.
    pub(super) fn service_name(&self) -> String {
        match &self.request {
            Some(PairingRequest::RequestServiceAuthorization { uuid, .. }) => {
                td!(service_name_key(uuid))
            }
            _ => String::new(),
        }
    }

    pub(super) fn left_action_label(&self) -> String {
        match self.variant() {
            PairingVariant::RequestConfirmation | PairingVariant::RequestPasskey => {
                t!("dropdown-bluetooth-reject")
            }
            PairingVariant::RequestAuthorization | PairingVariant::RequestServiceAuthorization => {
                t!("dropdown-bluetooth-deny")
            }
            _ => t!("dropdown-bluetooth-cancel"),
        }
    }

    pub(super) fn right_action_label(&self) -> String {
        match self.variant() {
            PairingVariant::RequestPasskey | PairingVariant::RequestPinCode => {
                t!("dropdown-bluetooth-pair")
            }
            PairingVariant::RequestAuthorization | PairingVariant::RequestServiceAuthorization => {
                t!("dropdown-bluetooth-allow")
            }
            _ => t!("dropdown-bluetooth-confirm"),
        }
    }

    pub(super) fn has_confirm_action(&self) -> bool {
        !matches!(
            self.variant(),
            PairingVariant::DisplayPin | PairingVariant::DisplayPasskey | PairingVariant::None
        )
    }

    /// Answers the request with what the card holds, if it's complete.
    pub(super) fn confirm(&self) {
        let (Some(bluetooth), Some(request)) = (self.bluetooth.get(), &self.request) else {
            return;
        };

        match request {
            PairingRequest::RequestPasskey { .. } => {
                if let Some(passkey) = self.typed_passkey() {
                    bluetooth.provide_passkey(passkey);
                }
            }
            PairingRequest::RequestPinCode { .. } => {
                if let Some(pin) = self.typed_pin() {
                    bluetooth.provide_pin(pin);
                }
            }
            PairingRequest::RequestConfirmation { .. } => bluetooth.provide_confirmation(true),
            PairingRequest::RequestAuthorization { .. } => bluetooth.provide_authorization(true),
            PairingRequest::RequestServiceAuthorization { .. } => {
                bluetooth.provide_service_authorization(true);
            }
            PairingRequest::DisplayPinCode { .. } | PairingRequest::DisplayPasskey { .. } => {}
        }
    }

    /// Turns the request down (or stops displaying it).
    pub(super) fn cancel(&self) {
        if let Some(bluetooth) = self.bluetooth.get() {
            bluetooth.cancel_pending_request();
        }
    }

    /// The passkey typed, once all six digits are.
    fn typed_passkey(&self) -> Option<u32> {
        let digits: String = self
            .pin_entries
            .iter()
            .map(|entry| entry.text().to_string())
            .collect();

        (digits.len() == self.pin_entries.len())
            .then(|| digits.parse().ok())
            .flatten()
    }

    /// The legacy PIN typed, if it's one BlueZ accepts (1 to 16 bytes).
    fn typed_pin(&self) -> Option<String> {
        let pin = self.legacy_pin_entry.text().to_string();
        (1..=16).contains(&pin.len()).then_some(pin)
    }
}

fn classify_key(key: gdk::Key) -> Option<PinKeyAction> {
    match key {
        gdk::Key::_0 | gdk::Key::KP_0 => Some(PinKeyAction::Digit('0')),
        gdk::Key::_1 | gdk::Key::KP_1 => Some(PinKeyAction::Digit('1')),
        gdk::Key::_2 | gdk::Key::KP_2 => Some(PinKeyAction::Digit('2')),
        gdk::Key::_3 | gdk::Key::KP_3 => Some(PinKeyAction::Digit('3')),
        gdk::Key::_4 | gdk::Key::KP_4 => Some(PinKeyAction::Digit('4')),
        gdk::Key::_5 | gdk::Key::KP_5 => Some(PinKeyAction::Digit('5')),
        gdk::Key::_6 | gdk::Key::KP_6 => Some(PinKeyAction::Digit('6')),
        gdk::Key::_7 | gdk::Key::KP_7 => Some(PinKeyAction::Digit('7')),
        gdk::Key::_8 | gdk::Key::KP_8 => Some(PinKeyAction::Digit('8')),
        gdk::Key::_9 | gdk::Key::KP_9 => Some(PinKeyAction::Digit('9')),
        gdk::Key::BackSpace => Some(PinKeyAction::Backspace),
        gdk::Key::Tab | gdk::Key::ISO_Left_Tab => Some(PinKeyAction::PassThrough),
        _ => None,
    }
}

fn handle_pin_key(entries: &[gtk::Entry; 6], index: usize, key: gdk::Key) -> glib::Propagation {
    let Some(action) = classify_key(key) else {
        return glib::Propagation::Stop;
    };

    match action {
        PinKeyAction::Digit(ch) => {
            entries[index].set_text(&ch.to_string());
            if index < entries.len() - 1 {
                entries[index + 1].grab_focus();
            }
            glib::Propagation::Stop
        }

        PinKeyAction::Backspace => {
            if entries[index].text().is_empty() && index > 0 {
                entries[index - 1].set_text("");
                entries[index - 1].grab_focus();
            } else {
                entries[index].set_text("");
            }
            glib::Propagation::Stop
        }

        PinKeyAction::PassThrough => glib::Propagation::Proceed,
    }
}

pub(super) fn setup_pin_entries(entries: [gtk::Entry; 6]) {
    for (index, entry) in entries.iter().enumerate() {
        let entries = entries.clone();
        let key_controller = gtk::EventControllerKey::new();
        key_controller.set_propagation_phase(gtk::PropagationPhase::Capture);
        key_controller
            .connect_key_pressed(move |_, key, _, _| handle_pin_key(&entries, index, key));
        entry.add_controller(key_controller);
    }
}
