use gtk::{pango, prelude::*};
use relm4::{gtk, prelude::*};
use wayle_iwd::{SecurityType, SignalStrength};
use wayle_widgets::prelude::*;
use zbus::zvariant::OwnedObjectPath;

use super::methods;
use crate::{
    i18n::t,
    shell::bar::dropdowns::iwd::helpers::{self, NetworkSnapshot},
};

const HOVER_TRANSITION_MS: u32 = 150;

pub(super) struct NetworkItemInit {
    pub snapshot: NetworkSnapshot,
    /// Configured signal-strength icon, resolved by the parent.
    pub icon: String,
}

pub(super) struct NetworkItem {
    ssid: String,
    icon: String,
    security_label: String,
    object_path: OwnedObjectPath,

    /// Raw security/strength carried so the parent can build a [`SelectedNetwork`]
    /// straight from the chosen row — the row is the single source of per-network
    /// state (there is no parallel snapshot cache).
    ///
    /// [`SelectedNetwork`]: super::messages::SelectedNetwork
    security: SecurityType,
    strength: SignalStrength,
    known: bool,
    hovered: bool,
}

impl NetworkItem {
    /// SSID, the stable identity used to reconcile the list in place.
    pub(super) fn ssid(&self) -> &str {
        &self.ssid
    }

    /// D-Bus object path of this network (the connect/forget target).
    pub(super) fn object_path(&self) -> &OwnedObjectPath {
        &self.object_path
    }

    /// Security classification, used to derive the connect flow.
    pub(super) fn security(&self) -> SecurityType {
        self.security
    }

    /// Signal bucket, carried through to the password form's icon.
    pub(super) fn strength(&self) -> SignalStrength {
        self.strength
    }

    /// Whether credentials for this network are already saved.
    pub(super) fn known(&self) -> bool {
        self.known
    }

    /// Whether this row offers the Forget action: credentials are saved, and they
    /// are credentials the shell could put back afterwards (see
    /// [`helpers::forgettable`]).
    fn offers_forget(&self) -> bool {
        offers_forget(self.known, self.security)
    }

    /// Whether this row can be updated in place for `snapshot`, or must be
    /// recreated. Whether Forget is offered is the only thing wired up at
    /// construction time (it gates the hover-to-forget controller in
    /// `init_widgets`), so a change there requires a fresh row; everything else
    /// updates via [`Self::refresh`].
    pub(super) fn reusable_for(&self, snapshot: &NetworkSnapshot) -> bool {
        self.offers_forget() == offers_forget(snapshot.known, snapshot.security)
    }

    /// Adopt `snapshot` in place, avoiding a destroy/recreate of the row widget.
    /// Every field is copied — a reused row must not keep state from the network
    /// it previously showed, since `known` and `security` also drive the connect
    /// flow (see [`super::methods`]'s `select_network`), not just the display.
    pub(super) fn refresh(&mut self, snapshot: &NetworkSnapshot, icon: String) {
        self.icon = icon;
        self.security = snapshot.security;
        self.strength = snapshot.strength;
        self.known = snapshot.known;
        self.security_label = security_label(snapshot);
        self.object_path = snapshot.object_path.clone();
    }
}

/// Whether a network with these properties offers the Forget action — see
/// [`NetworkItem::offers_forget`]. Free-standing so a row and the snapshot it
/// would be rebuilt from can be compared.
fn offers_forget(known: bool, security: SecurityType) -> bool {
    known && helpers::forgettable(security)
}

/// Security label for a network, marking saved secured networks distinctly.
fn security_label(snapshot: &NetworkSnapshot) -> String {
    let base = methods::translate_security_type(snapshot.security);
    if snapshot.known && helpers::requires_password(snapshot.security) {
        t!("dropdown-iwd-security-saved", security = base)
    } else {
        base
    }
}

#[derive(Debug)]
pub(super) enum NetworkItemInput {
    Hovered(bool),
    ForgetClicked,
}

#[derive(Debug)]
pub(super) enum NetworkItemOutput {
    Selected(DynamicIndex),
    ForgetRequested(OwnedObjectPath),
}

#[relm4::factory(pub(super))]
impl FactoryComponent for NetworkItem {
    type Init = NetworkItemInit;
    type Input = NetworkItemInput;
    type Output = NetworkItemOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        gtk::Box {
            add_css_class: "network-item",
            set_cursor_from_name: Some("pointer"),

            #[name = "signal_icon"]
            gtk::Image {
                add_css_class: "network-item-signal",
                #[watch]
                set_icon_name: Some(self.icon.as_str()),
                set_valign: gtk::Align::Center,
            },

            #[name = "info_column"]
            gtk::Box {
                add_css_class: "network-item-info",
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,

                #[name = "ssid_label"]
                gtk::Label {
                    add_css_class: "network-item-name",
                    set_halign: gtk::Align::Start,
                    set_ellipsize: pango::EllipsizeMode::End,
                    #[watch]
                    set_label: &self.ssid,
                },

                #[name = "security_label"]
                gtk::Label {
                    add_css_class: "network-item-security",
                    set_halign: gtk::Align::Start,
                    #[watch]
                    set_label: &self.security_label,
                },
            },

            #[name = "trailing_stack"]
            gtk::Stack {
                add_css_class: "network-item-trailing",
                set_transition_type: gtk::StackTransitionType::Crossfade,
                set_transition_duration: HOVER_TRANSITION_MS,
                set_valign: gtk::Align::Center,
                set_hexpand: false,
                #[watch]
                set_visible: helpers::requires_password(self.security) || self.offers_forget(),

                add_named[Some("lock")] = &gtk::Box {
                    set_halign: gtk::Align::End,
                    set_valign: gtk::Align::Center,

                    #[name = "lock_icon"]
                    gtk::Image {
                        add_css_class: "network-item-lock",
                        set_icon_name: Some("ld-lock-symbolic"),
                        set_valign: gtk::Align::Center,
                        #[watch]
                        set_visible: helpers::requires_password(self.security),
                    },
                },

                add_named[Some("actions")] = &gtk::Box {
                    add_css_class: "network-item-actions",
                    set_valign: gtk::Align::Center,

                    #[template]
                    GhostButton {
                        add_css_class: "network-item-forget",
                        #[template_child]
                        label {
                            set_label: &t!("dropdown-iwd-forget"),
                        },
                        connect_clicked => NetworkItemInput::ForgetClicked,
                    },
                },

                #[watch]
                set_visible_child_name:
                    if self.hovered && self.offers_forget() {
                        "actions"
                    } else {
                        "lock"
                    },
            },
        }
    }

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        let NetworkItemInit { snapshot, icon } = init;
        Self {
            icon,
            security: snapshot.security,
            strength: snapshot.strength,
            known: snapshot.known,
            hovered: false,
            security_label: security_label(&snapshot),
            ssid: snapshot.ssid,
            object_path: snapshot.object_path,
        }
    }

    fn update(&mut self, msg: NetworkItemInput, sender: FactorySender<Self>) {
        match msg {
            NetworkItemInput::Hovered(hovered) => {
                self.hovered = hovered;
            }

            NetworkItemInput::ForgetClicked => {
                let _ = sender.output(NetworkItemOutput::ForgetRequested(self.object_path.clone()));
            }
        }
    }

    fn init_widgets(
        &mut self,
        index: &Self::Index,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let click = gtk::GestureClick::new();
        let idx = index.clone();
        let click_sender = sender.output_sender().clone();

        click.connect_released(move |gesture, _, _, _| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            click_sender.emit(NetworkItemOutput::Selected(idx.clone()));
        });

        root.add_controller(click);

        if self.offers_forget() {
            let hover = gtk::EventControllerMotion::new();
            let hover_sender = sender.input_sender().clone();

            hover.connect_enter(move |_, _, _| {
                hover_sender.emit(NetworkItemInput::Hovered(true));
            });

            let leave_sender = sender.input_sender().clone();

            hover.connect_leave(move |_| {
                leave_sender.emit(NetworkItemInput::Hovered(false));
            });

            root.add_controller(hover);
        }

        let widgets = view_output!();
        widgets
    }
}
