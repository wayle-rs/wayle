use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_mullvad::{ConnectionStatus, ErrorCause, RelayLocation};
use wayle_widgets::prelude::*;

use super::helpers;
use crate::i18n::t;

/// The pinned "selected relay" card at the top of the dropdown. Shows the
/// active relay when a tunnel is up, otherwise the selected relay that a connect
/// would use, inside its own elevated surface — swapping the status for a
/// connect/disconnect button on hover. The flag sits in a square whose
/// background color reflects the connection status.
pub(super) struct CurrentConnection {
    status: ConnectionStatus,
    selected: Option<RelayLocation>,
    hovered: bool,
}

#[derive(Debug)]
pub(super) enum CurrentConnectionInput {
    SetState(ConnectionStatus, Option<RelayLocation>),
    Hovered(bool),
    ActionClicked,
}

#[derive(Debug)]
pub(crate) enum CurrentConnectionOutput {
    ToggleRequested,
}

impl CurrentConnection {
    /// The relay to display: the active relay while connecting/connected, else
    /// the selected relay (the one a connect would use).
    fn relay(&self) -> Option<&RelayLocation> {
        self.status.relay().or(self.selected.as_ref())
    }

    fn flag(&self) -> String {
        self.relay()
            .map(|relay| relay.country_code.as_str())
            .filter(|code| !code.is_empty())
            .map_or_else(|| helpers::FLAG_FALLBACK.to_string(), helpers::flag_icon)
    }

    fn title(&self) -> String {
        match self.relay() {
            Some(relay) => match &relay.city {
                Some(city) if !city.is_empty() => format!("{city}, {}", relay.country),
                _ => relay.country.clone(),
            },
            // No relay to show: a transitional label, or "no relay selected"
            // when there is no geographic selection.
            None => match &self.status {
                ConnectionStatus::Connecting(_) => t!("dropdown-mullvad-connecting"),
                ConnectionStatus::Disconnecting => t!("dropdown-mullvad-disconnecting"),
                ConnectionStatus::Error(_) => t!("dropdown-mullvad-blocked"),
                ConnectionStatus::Connected(_)
                | ConnectionStatus::Disconnected
                | ConnectionStatus::LoggedOut
                | ConnectionStatus::Revoked => t!("dropdown-mullvad-not-connected"),
            },
        }
    }

    fn subtitle(&self) -> String {
        self.relay()
            .and_then(|relay| relay.hostname.clone())
            .unwrap_or_default()
    }

    fn has_subtitle(&self) -> bool {
        !self.subtitle().is_empty()
    }

    fn status_label(&self) -> String {
        match &self.status {
            ConnectionStatus::Connected(_) => t!("dropdown-mullvad-connected"),
            ConnectionStatus::Connecting(_) => t!("dropdown-mullvad-connecting"),
            ConnectionStatus::Disconnecting => t!("dropdown-mullvad-disconnecting"),
            ConnectionStatus::Disconnected => t!("dropdown-mullvad-disconnected"),
            ConnectionStatus::Error(cause) => error_label(*cause),
            // These render behind their own empty states, so the body is hidden;
            // fall back to a neutral label for exhaustiveness.
            ConnectionStatus::LoggedOut | ConnectionStatus::Revoked => {
                t!("dropdown-mullvad-disconnected")
            }
        }
    }

    fn action_label(&self) -> String {
        match &self.status {
            ConnectionStatus::Disconnected => t!("dropdown-mullvad-connect"),
            _ => t!("dropdown-mullvad-disconnect"),
        }
    }

    /// Classes for the flag square: the base class plus a status modifier that
    /// selects the background/foreground color (see the SCSS).
    fn icon_classes(&self) -> Vec<&'static str> {
        let modifier = match &self.status {
            ConnectionStatus::Connected(_) => "mullvad-connected",
            ConnectionStatus::Connecting(_) | ConnectionStatus::Disconnecting => {
                "mullvad-connecting"
            }
            ConnectionStatus::Disconnected
            | ConnectionStatus::LoggedOut
            | ConnectionStatus::Revoked => "mullvad-disconnected",
            ConnectionStatus::Error(_) => "mullvad-blocked",
        };
        vec!["network-connection-icon", modifier]
    }

    fn status_classes(&self) -> Vec<&'static str> {
        if matches!(self.status, ConnectionStatus::Error(_)) {
            vec!["network-connection-status", "error"]
        } else {
            vec!["network-connection-status"]
        }
    }
}

/// Translated label for a blocking/error cause.
fn error_label(cause: ErrorCause) -> String {
    match cause {
        ErrorCause::AuthFailed => t!("dropdown-mullvad-error-auth"),
        ErrorCause::Offline => t!("dropdown-mullvad-error-offline"),
        ErrorCause::Other => t!("dropdown-mullvad-blocked"),
    }
}

#[relm4::component(pub(super))]
impl Component for CurrentConnection {
    type Init = ();
    type Input = CurrentConnectionInput;
    type Output = CurrentConnectionOutput;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            add_css_class: "card",
            add_css_class: "network-connections-group",
            set_orientation: gtk::Orientation::Vertical,

            #[name = "card"]
            gtk::Box {
                add_css_class: "network-connection-card",

                gtk::Box {
                    #[watch]
                    set_css_classes: &model.icon_classes(),
                    set_hexpand: false,
                    gtk::Image {
                        #[watch]
                        set_icon_name: Some(model.flag().as_str()),
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
                        set_label: &model.title(),
                    },

                    gtk::Label {
                        add_css_class: "network-connection-detail",
                        set_xalign: 0.0,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_max_width_chars: 1,
                        #[watch]
                        set_visible: model.has_subtitle(),
                        #[watch]
                        set_label: &model.subtitle(),
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
                            #[watch]
                            set_css_classes: &model.status_classes(),
                            #[watch]
                            set_label: &model.status_label(),
                            set_valign: gtk::Align::Center,
                        },
                    },

                    add_named[Some("actions")] = &gtk::Box {
                        add_css_class: "network-connection-actions",
                        set_halign: gtk::Align::End,
                        set_valign: gtk::Align::Center,
                        #[template]
                        GhostButton {
                            add_css_class: "network-action-toggle",
                            #[template_child]
                            label {
                                #[watch]
                                set_label: &model.action_label(),
                            },
                            connect_clicked => CurrentConnectionInput::ActionClicked,
                        },
                    },

                    #[watch]
                    set_visible_child_name: if model.hovered { "actions" } else { "status" },
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            status: ConnectionStatus::default(),
            selected: None,
            hovered: false,
        };
        let widgets = view_output!();

        let hover = gtk::EventControllerMotion::new();
        let enter_sender = sender.input_sender().clone();
        hover.connect_enter(move |_, _, _| {
            enter_sender.emit(CurrentConnectionInput::Hovered(true));
        });
        let leave_sender = sender.input_sender().clone();
        hover.connect_leave(move |_| {
            leave_sender.emit(CurrentConnectionInput::Hovered(false));
        });
        widgets.card.add_controller(hover);

        // Self-heal: GTK does not reliably deliver a leave when the popover is
        // popped down with the pointer inside, which would otherwise leave the
        // action button showing on next open. Reset hover state on unmap.
        let unmap_sender = sender.input_sender().clone();
        widgets.card.connect_unmap(move |_| {
            unmap_sender.emit(CurrentConnectionInput::Hovered(false));
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            CurrentConnectionInput::SetState(status, selected) => {
                self.status = status;
                self.selected = selected;
            }
            CurrentConnectionInput::Hovered(hovered) => self.hovered = hovered,
            CurrentConnectionInput::ActionClicked => {
                let _ = sender.output(CurrentConnectionOutput::ToggleRequested);
            }
        }
    }
}
