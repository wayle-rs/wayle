pub(super) mod messages;
mod methods;

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_bluetooth::{BluetoothService, types::agent::PairingRequest};
use wayle_core::DeferredService;
use wayle_widgets::prelude::*;

use self::messages::{PairingCardInit, PairingCardMsg};
use crate::i18n::{t, td};

/// How many digits a passkey has.
const PASSKEY_DIGITS: u16 = 6;

/// Which kind of prompt the card shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PairingVariant {
    None,
    DisplayPin,
    RequestPasskey,
    DisplayPasskey,
    RequestConfirmation,
    RequestAuthorization,
    RequestServiceAuthorization,
    RequestPinCode,
}

/// Shows the service's pairing request and answers it. Everything it shows
/// is read from the request and the service's device it is about.
pub(super) struct PairingCard {
    bluetooth: DeferredService<BluetoothService>,
    /// The request shown: the service's, as of its latest change.
    request: Option<PairingRequest>,
    pin_entries: [gtk::Entry; 6],
    legacy_pin_entry: gtk::Entry,
    /// Guards the card after it appears or changes under the pointer; set
    /// once the card's widget exists.
    click_guard: Option<ClickGuard>,
}

#[relm4::component(pub(super))]
impl SimpleComponent for PairingCard {
    type Init = PairingCardInit;
    type Input = PairingCardMsg;
    type Output = ();

    view! {
        #[root]
        gtk::Box {
            add_css_class: "bluetooth-pairing-card",
            set_orientation: gtk::Orientation::Vertical,
            #[watch]
            set_visible: model.request.is_some(),

            #[name = "header"]
            gtk::Box {
                add_css_class: "bluetooth-pairing-header",

                #[name = "header_icon_container"]
                gtk::Box {
                    add_css_class: "bluetooth-device-icon",
                    set_valign: gtk::Align::Center,
                    #[name = "header_icon"]
                    gtk::Image {
                        add_css_class: "bluetooth-icon",
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        #[watch]
                        set_icon_name: Some(model.display().icon),
                    },
                },

                #[name = "header_info"]
                gtk::Box {
                    add_css_class: "bluetooth-pairing-device-info",
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,
                    set_valign: gtk::Align::Center,
                    #[name = "header_device_name"]
                    gtk::Label {
                        add_css_class: "bluetooth-device-name",
                        set_halign: gtk::Align::Start,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        #[watch]
                        set_label: &model.display().name,
                    },
                    #[name = "header_device_type"]
                    gtk::Label {
                        add_css_class: "bluetooth-device-detail",
                        set_halign: gtk::Align::Start,
                        #[watch]
                        set_label: &td!(model.display().device_type_key),
                    },
                },

                #[template]
                GhostIconButton {
                    add_css_class: "bluetooth-pairing-close",
                    set_icon_name: "ld-x-symbolic",
                    set_valign: gtk::Align::Start,
                    connect_clicked => PairingCardMsg::Cancel,
                },
            },

            #[name = "display_pin_section"]
            gtk::Box {
                add_css_class: "bluetooth-pin-display",
                set_orientation: gtk::Orientation::Vertical,
                #[watch]
                set_visible: model.variant() == PairingVariant::DisplayPin,
                gtk::Label {
                    add_css_class: "bluetooth-pin-label",
                    set_label: &t!("dropdown-bluetooth-pairing-enter-pin"),
                },
                gtk::Label {
                    add_css_class: "bluetooth-pin-code",
                    #[watch]
                    set_label: &model.pin_code(),
                },
            },

            #[name = "display_pin_hint"]
            gtk::Label {
                add_css_class: "bluetooth-pairing-message",
                #[watch]
                set_visible: model.variant() == PairingVariant::DisplayPin,
                set_label: &t!("dropdown-bluetooth-pairing-type-on-device"),
                set_wrap: true,
            },

            #[name = "authorization_message"]
            gtk::Label {
                add_css_class: "bluetooth-pairing-message",
                #[watch]
                set_visible: model.variant() == PairingVariant::RequestAuthorization,
                set_label: &t!("dropdown-bluetooth-pairing-allow-pairing"),
                set_wrap: true,
            },

            #[name = "passkey_input_message"]
            gtk::Label {
                add_css_class: "bluetooth-pairing-message",
                #[watch]
                set_visible: model.variant() == PairingVariant::RequestPasskey,
                set_label: &t!("dropdown-bluetooth-pairing-enter-shown-pin"),
                set_wrap: true,
            },

            #[name = "pin_input_row"]
            gtk::Box {
                add_css_class: "bluetooth-pin-input-row",
                set_halign: gtk::Align::Center,
                #[watch]
                set_visible: model.variant() == PairingVariant::RequestPasskey,

                #[name = "pin_digit_0"]
                gtk::Entry {
                    add_css_class: "bluetooth-pin-digit",
                    set_max_length: 1,
                    set_max_width_chars: 1,
                },
                #[name = "pin_digit_1"]
                gtk::Entry {
                    add_css_class: "bluetooth-pin-digit",
                    set_max_length: 1,
                    set_max_width_chars: 1,
                },
                #[name = "pin_digit_2"]
                gtk::Entry {
                    add_css_class: "bluetooth-pin-digit",
                    set_max_length: 1,
                    set_max_width_chars: 1,
                },
                #[name = "pin_digit_3"]
                gtk::Entry {
                    add_css_class: "bluetooth-pin-digit",
                    set_max_length: 1,
                    set_max_width_chars: 1,
                },
                #[name = "pin_digit_4"]
                gtk::Entry {
                    add_css_class: "bluetooth-pin-digit",
                    set_max_length: 1,
                    set_max_width_chars: 1,
                },
                #[name = "pin_digit_5"]
                gtk::Entry {
                    add_css_class: "bluetooth-pin-digit",
                    set_max_length: 1,
                    set_max_width_chars: 1,
                },
            },

            #[name = "confirmation_message"]
            gtk::Label {
                add_css_class: "bluetooth-pairing-message",
                #[watch]
                set_visible: model.variant() == PairingVariant::RequestConfirmation,
                set_label: &t!("dropdown-bluetooth-pairing-confirm-code"),
                set_wrap: true,
            },

            #[name = "confirmation_pin_display"]
            gtk::Box {
                add_css_class: "bluetooth-pin-display",
                set_orientation: gtk::Orientation::Vertical,
                #[watch]
                set_visible: model.variant() == PairingVariant::RequestConfirmation,
                gtk::Label {
                    add_css_class: "bluetooth-pin-code",
                    #[watch]
                    set_label: &model.pin_code(),
                },
            },

            #[name = "display_passkey_section"]
            gtk::Box {
                add_css_class: "bluetooth-pin-display",
                set_orientation: gtk::Orientation::Vertical,
                #[watch]
                set_visible: model.variant() == PairingVariant::DisplayPasskey,
                gtk::Label {
                    add_css_class: "bluetooth-pin-label",
                    set_label: &t!("dropdown-bluetooth-pairing-enter-pin"),
                },
                gtk::Label {
                    add_css_class: "bluetooth-pin-code",
                    #[watch]
                    set_label: &model.pin_code(),
                },
                #[name = "progress_dots"]
                gtk::Box { add_css_class: "bluetooth-progress-dots" },
            },

            #[name = "passkey_progress_message"]
            gtk::Label {
                add_css_class: "bluetooth-pairing-message",
                #[watch]
                set_visible: model.variant() == PairingVariant::DisplayPasskey,
                #[watch]
                set_label: &t!(
                    "dropdown-bluetooth-pairing-entering",
                    entered = model.passkey_entered(),
                    total = PASSKEY_DIGITS
                ),
                set_wrap: true,
            },

            #[name = "service_auth_info"]
            gtk::Box {
                add_css_class: "bluetooth-service-info",
                set_orientation: gtk::Orientation::Vertical,
                #[watch]
                set_visible: model.variant() == PairingVariant::RequestServiceAuthorization,
                gtk::Label {
                    add_css_class: "bluetooth-service-name",
                    set_halign: gtk::Align::Start,
                    #[watch]
                    set_label: &model.service_name(),
                },
            },

            #[name = "service_auth_message"]
            gtk::Label {
                add_css_class: "bluetooth-pairing-message",
                #[watch]
                set_visible: model.variant() == PairingVariant::RequestServiceAuthorization,
                set_label: &t!("dropdown-bluetooth-pairing-service-allow"),
                set_wrap: true,
            },

            #[name = "legacy_pin_message"]
            gtk::Label {
                add_css_class: "bluetooth-pairing-message",
                #[watch]
                set_visible: model.variant() == PairingVariant::RequestPinCode,
                set_label: &t!("dropdown-bluetooth-pairing-enter-legacy-pin"),
                set_wrap: true,
            },

            #[name = "legacy_pin_entry"]
            gtk::Entry {
                add_css_class: "bluetooth-legacy-pin-input",
                #[watch]
                set_visible: model.variant() == PairingVariant::RequestPinCode,
                set_max_length: 16,
                set_placeholder_text: Some(&t!("dropdown-bluetooth-pairing-pin-placeholder")),
            },

            #[name = "legacy_pin_hint"]
            gtk::Label {
                add_css_class: "bluetooth-pin-hint",
                #[watch]
                set_visible: model.variant() == PairingVariant::RequestPinCode,
                set_label: &t!("dropdown-bluetooth-pairing-common-pins"),
            },

            #[name = "action_buttons"]
            gtk::Box {
                add_css_class: "bluetooth-pairing-actions",
                set_homogeneous: true,

                #[template]
                GhostButton {
                    set_halign: gtk::Align::Fill,
                    #[template_child]
                    label {
                        set_hexpand: true,
                        set_halign: gtk::Align::Center,
                        #[watch]
                        set_label: &model.left_action_label(),
                    },
                    connect_clicked => PairingCardMsg::Cancel,
                },

                #[template]
                PrimaryButton {
                    set_halign: gtk::Align::Fill,
                    #[watch]
                    set_visible: model.has_confirm_action(),
                    #[template_child]
                    label {
                        set_hexpand: true,
                        set_halign: gtk::Align::Center,
                        #[watch]
                        set_label: &model.right_action_label(),
                    },
                    connect_clicked => PairingCardMsg::Confirm,
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Self {
            bluetooth: init.bluetooth,
            request: None,
            pin_entries: Default::default(),
            legacy_pin_entry: gtk::Entry::default(),
            click_guard: None,
        };

        let widgets = view_output!();

        model.legacy_pin_entry = widgets.legacy_pin_entry.clone();
        model.pin_entries = [
            widgets.pin_digit_0.clone(),
            widgets.pin_digit_1.clone(),
            widgets.pin_digit_2.clone(),
            widgets.pin_digit_3.clone(),
            widgets.pin_digit_4.clone(),
            widgets.pin_digit_5.clone(),
        ];

        methods::setup_pin_entries(model.pin_entries.clone());
        model.click_guard = Some(ClickGuard::new(&root));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            PairingCardMsg::SetRequest {
                request,
                new_prompt,
                guard,
            } => {
                if new_prompt {
                    self.clear_inputs();
                }
                self.request = request;
                if guard && let Some(click_guard) = &self.click_guard {
                    click_guard.arm();
                }
            }

            // The view reads the device afresh.
            PairingCardMsg::Refresh => {}

            // A click already queued when the card changed counts as aimed
            // elsewhere, like one the guard stops the pointer from making.
            PairingCardMsg::Confirm | PairingCardMsg::Cancel
                if self.click_guard.as_ref().is_some_and(ClickGuard::is_armed) => {}

            PairingCardMsg::Confirm => self.confirm(),
            PairingCardMsg::Cancel => self.cancel(),
        }
    }
}
