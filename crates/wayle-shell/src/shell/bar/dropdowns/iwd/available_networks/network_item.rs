use gtk::{pango, prelude::*};
use relm4::{gtk, prelude::*};
use std::sync::Arc;

use wayle_iwd::{Network, SecurityType};
use wayle_widgets::prelude::*;

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
    /// What the row shows, and the network it acts on.
    snapshot: NetworkSnapshot,
    /// Configured signal-strength icon, resolved by the parent.
    icon: String,
    security_label: String,
    hovered: bool,
    /// This row's position in the list, kept current by the factory; lets the
    /// parent locate a row in O(1) while reordering.
    pub(super) index: DynamicIndex,
    /// Guards the row after it moves under the pointer; set once the row's
    /// widget exists.
    click_guard: Option<ClickGuard>,
}

impl NetworkItem {
    /// The key reconciling the list in place: the network's object path,
    /// which names one network of the station (unlike its SSID, shared by
    /// networks of different security).
    pub(super) fn key(&self) -> &str {
        snapshot_key(&self.snapshot)
    }

    /// Whether clicking the row connects: see [`helpers::joinable`].
    fn joinable(&self) -> bool {
        helpers::joinable(self.snapshot.security, self.snapshot.known)
    }

    /// Whether this row offers the Forget action: credentials are saved, and they
    /// are credentials the shell could put back afterwards (see
    /// [`helpers::forgettable`]).
    fn offers_forget(&self) -> bool {
        offers_forget(self.snapshot.known, self.snapshot.security)
    }

    /// Whether this row can be updated in place for `snapshot`, or must be
    /// recreated. Whether Forget is offered is the only thing wired up at
    /// construction time (it gates the hover-to-forget controller in
    /// `init_widgets`), so a change there requires a fresh row; everything else
    /// updates via [`Self::refresh`].
    pub(super) fn reusable_for(&self, snapshot: &NetworkSnapshot) -> bool {
        self.offers_forget() == offers_forget(snapshot.known, snapshot.security)
    }

    /// Whether the row would show something different for `snapshot`.
    pub(super) fn differs_from(&self, snapshot: &NetworkSnapshot, icon: &str) -> bool {
        self.snapshot != *snapshot || self.icon != icon
    }

    /// Adopts `snapshot` in place, avoiding a destroy/recreate of the row
    /// widget.
    pub(super) fn refresh(&mut self, snapshot: &NetworkSnapshot, icon: String) {
        self.icon = icon;
        self.security_label = security_label(snapshot);
        self.snapshot = snapshot.clone();
    }

    /// Makes the row unclickable for a moment because it moved under the
    /// pointer.
    pub(super) fn arm_click_guard(&self) {
        if let Some(guard) = &self.click_guard {
            guard.arm();
        }
    }

    pub(super) fn disarm_click_guard(&self) {
        if let Some(guard) = &self.click_guard {
            guard.disarm();
        }
    }

    fn guarded(&self) -> bool {
        self.click_guard.as_ref().is_some_and(ClickGuard::is_armed)
    }
}

/// The key reconciling the list in place (see [`NetworkItem::key`]).
pub(super) fn snapshot_key(snapshot: &NetworkSnapshot) -> &str {
    snapshot.network.object_path().as_str()
}

/// Whether a network with these properties offers the Forget action — see
/// [`NetworkItem::offers_forget`]. Free-standing so a row and the snapshot it
/// would be rebuilt from can be compared.
fn offers_forget(known: bool, security: SecurityType) -> bool {
    known && helpers::forgettable(security)
}

/// Security label for a network, marking saved networks (IWD keeps a profile
/// for open networks too, once joined) and ones that can't be joined from
/// here distinctly.
fn security_label(snapshot: &NetworkSnapshot) -> String {
    let base = methods::translate_security_type(snapshot.security);
    if snapshot.known {
        t!("dropdown-iwd-security-saved", security = base)
    } else if !helpers::joinable(snapshot.security, snapshot.known) {
        t!("dropdown-iwd-security-unconfigured", security = base)
    } else {
        base
    }
}

#[derive(Debug)]
pub(super) enum NetworkItemInput {
    /// The row itself was clicked.
    Clicked,
    Hovered(bool),
    ForgetClicked,
}

#[derive(Debug)]
pub(super) enum NetworkItemOutput {
    Selected(Arc<Network>),
    ForgetRequested(Arc<Network>),
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
            // An 802.1X network IWD has no provisioning for can't be joined
            // from here (see `helpers::joinable`).
            #[watch]
            set_sensitive: self.joinable(),
            #[watch]
            set_cursor_from_name: self.joinable().then_some("pointer"),

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
                    set_label: &self.snapshot.ssid,
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
                set_visible: helpers::secured(self.snapshot.security) || self.offers_forget(),

                add_named[Some("lock")] = &gtk::Box {
                    set_halign: gtk::Align::End,
                    set_valign: gtk::Align::Center,

                    #[name = "lock_icon"]
                    gtk::Image {
                        add_css_class: "network-item-lock",
                        set_icon_name: Some("ld-lock-symbolic"),
                        set_valign: gtk::Align::Center,
                        #[watch]
                        set_visible: helpers::secured(self.snapshot.security),
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

    fn init_model(init: Self::Init, index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        let NetworkItemInit { snapshot, icon } = init;
        Self {
            icon,
            hovered: false,
            security_label: security_label(&snapshot),
            snapshot,
            index: index.clone(),
            click_guard: None,
        }
    }

    fn update(&mut self, msg: NetworkItemInput, sender: FactorySender<Self>) {
        // A guarded row can't be targeted by the pointer; this also covers a
        // click that was already queued when the row moved.
        let clicked = matches!(
            msg,
            NetworkItemInput::Clicked | NetworkItemInput::ForgetClicked
        );
        if clicked && self.guarded() {
            return;
        }

        match msg {
            NetworkItemInput::Clicked => {
                if self.joinable() {
                    let _ = sender.output(NetworkItemOutput::Selected(Arc::clone(
                        &self.snapshot.network,
                    )));
                }
            }

            NetworkItemInput::Hovered(hovered) => {
                self.hovered = hovered;
            }

            NetworkItemInput::ForgetClicked => {
                let _ = sender.output(NetworkItemOutput::ForgetRequested(Arc::clone(
                    &self.snapshot.network,
                )));
            }
        }
    }

    fn init_widgets(
        &mut self,
        _index: &Self::Index,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        self.click_guard = Some(ClickGuard::new(&root));

        let click = gtk::GestureClick::new();
        let click_sender = sender.input_sender().clone();

        click.connect_released(move |gesture, _, _, _| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            click_sender.emit(NetworkItemInput::Clicked);
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
