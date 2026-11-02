use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_network::types::states::NMActiveConnectionState;
use wayle_widgets::prelude::*;
use zbus::zvariant::OwnedObjectPath;

use super::messages::VpnActiveInfo;
use crate::i18n::t;

#[derive(Debug)]
pub(crate) struct VpnRow {
    pub(crate) connection_path: OwnedObjectPath,
    pub(crate) id: String,
    pub(crate) active: Option<VpnActiveInfo>,
    pub(crate) hovered: bool,
}

#[derive(Debug)]
pub(crate) struct VpnRowInit {
    pub connection_path: OwnedObjectPath,
    pub id: String,
    pub active: Option<VpnActiveInfo>,
}

#[derive(Debug)]
pub(crate) enum VpnRowInput {
    Hovered(bool),
    ConnectClicked,
    DisconnectClicked,
    ActiveUpdated(Option<VpnActiveInfo>),
    IdUpdated(String),
}

#[derive(Debug)]
pub(crate) enum VpnRowOutput {
    Connect(OwnedObjectPath),
    Disconnect(OwnedObjectPath),
}

#[relm4::factory(pub(crate))]
impl FactoryComponent for VpnRow {
    type Init = VpnRowInit;
    type Input = VpnRowInput;
    type Output = VpnRowOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        #[name = "card"]
        gtk::Box {
            add_css_class: "network-connection-card",

            gtk::Box {
                add_css_class: "network-connection-icon",
                add_css_class: "vpn",
                set_hexpand: false,

                gtk::Image {
                    set_icon_name: Some("ld-lock-symbolic"),
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                },
            },

            gtk::Box {
                add_css_class: "network-connection-info",
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,

                gtk::Label {
                    add_css_class: "network-connection-name",
                    set_xalign: 0.0,
                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                    set_max_width_chars: 1,
                    #[watch]
                    set_label: &self.id,
                },
            },

            gtk::Stack {
                add_css_class: "network-hover-stack",
                set_transition_type: gtk::StackTransitionType::Crossfade,
                set_transition_duration: 150,
                set_valign: gtk::Align::Center,
                set_hexpand: false,

                add_named[Some("status")] = &gtk::Box {
                    set_halign: gtk::Align::End,
                    set_valign: gtk::Align::Center,

                    gtk::Label {
                        add_css_class: "network-connection-status",
                        #[watch]
                        set_visible: self.is_activated(),
                        set_label: &t!("dropdown-network-connected"),
                    },

                    #[template]
                    SubtleBadge {
                        #[watch]
                        set_css_classes: &self.status_classes(),
                        #[watch]
                        set_label: &self.status_label(),
                        #[watch]
                        set_visible: self.is_activating(),
                    },
                },

                add_named[Some("connect")] = &gtk::Box {
                    add_css_class: "network-connection-actions",
                    set_valign: gtk::Align::Center,

                    #[template]
                    GhostButton {
                        add_css_class: "network-action-connect",
                        #[template_child]
                        label {
                            set_label: &t!("dropdown-network-connect"),
                        },
                        connect_clicked => VpnRowInput::ConnectClicked,
                    },
                },

                add_named[Some("disconnect")] = &gtk::Box {
                    add_css_class: "network-connection-actions",
                    set_valign: gtk::Align::Center,

                    #[template]
                    GhostButton {
                        add_css_class: "network-action-disconnect",
                        #[template_child]
                        label {
                            set_label: &t!("dropdown-network-disconnect"),
                        },
                        connect_clicked => VpnRowInput::DisconnectClicked,
                    },
                },

                #[watch]
                set_visible_child_name: self.visible_stack_child(),
            },
        }
    }

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        Self {
            connection_path: init.connection_path,
            id: init.id,
            active: init.active,
            hovered: false,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &Self::Index,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        let hover = gtk::EventControllerMotion::new();
        let enter_sender = sender.input_sender().clone();
        hover.connect_enter(move |_, _, _| {
            enter_sender.emit(VpnRowInput::Hovered(true));
        });
        let leave_sender = sender.input_sender().clone();
        hover.connect_leave(move |_| {
            leave_sender.emit(VpnRowInput::Hovered(false));
        });
        root.add_controller(hover);

        widgets
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            VpnRowInput::Hovered(hovered) => self.hovered = hovered,
            VpnRowInput::ConnectClicked => {
                let _ = sender.output(VpnRowOutput::Connect(self.connection_path.clone()));
            }
            VpnRowInput::DisconnectClicked => {
                if let Some(active) = &self.active {
                    let _ = sender.output(VpnRowOutput::Disconnect(active.object_path.clone()));
                }
            }
            VpnRowInput::ActiveUpdated(active) => self.active = active,
            VpnRowInput::IdUpdated(id) => self.id = id,
        }
    }
}

impl VpnRow {
    fn is_activated(&self) -> bool {
        self.active
            .as_ref()
            .is_some_and(|a| a.state == NMActiveConnectionState::Activated)
    }

    fn is_activating(&self) -> bool {
        self.active
            .as_ref()
            .is_some_and(|a| a.state == NMActiveConnectionState::Activating)
    }

    fn visible_stack_child(&self) -> &'static str {
        if !self.hovered {
            return "status";
        }
        match &self.active {
            None => "connect",
            Some(active) if active.state == NMActiveConnectionState::Activated => "disconnect",
            _ => "status",
        }
    }

    fn status_classes(&self) -> Vec<&'static str> {
        let mut classes = vec!["badge-subtle", "network-connection-status"];
        if self.is_activating() {
            classes.push("warning");
        }
        classes
    }

    fn status_label(&self) -> String {
        if self.is_activating() {
            t!("dropdown-network-connecting")
        } else {
            String::new()
        }
    }
}
