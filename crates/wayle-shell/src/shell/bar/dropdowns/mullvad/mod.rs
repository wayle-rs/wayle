mod city_item;
mod country_item;
mod current_connection;
mod factory;
mod helpers;
mod messages;
mod methods;
mod relay_item;
mod watchers;

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_mullvad::{ConnectionStatus, MullvadService, NetworkTarget};
use wayle_widgets::{WatcherToken, prelude::*};

pub(super) use self::factory::Factory;
use self::{
    country_item::{CountryItem, CountryItemInput, CountryItemOutput},
    current_connection::{CurrentConnection, CurrentConnectionOutput},
    messages::{MullvadDropdownCmd, MullvadDropdownInit, MullvadDropdownMsg},
};
use crate::{i18n::t, shell::bar::dropdowns::scaled_dimension};

const BASE_WIDTH: f32 = 382.0;
const BASE_HEIGHT: f32 = 512.0;

pub(crate) struct MullvadDropdown {
    mullvad: Option<std::sync::Arc<MullvadService>>,
    scaled_width: i32,
    scaled_height: i32,
    ready: bool,
    status: ConnectionStatus,
    current: Controller<CurrentConnection>,
    countries: FactoryVecDeque<CountryItem>,
    state_watcher: WatcherToken,
}

#[relm4::component(pub(crate))]
impl Component for MullvadDropdown {
    type Init = MullvadDropdownInit;
    type Input = MullvadDropdownMsg;
    type Output = ();
    type CommandOutput = MullvadDropdownCmd;

    view! {
        #[root]
        gtk::Popover {
            set_css_classes: &["dropdown", "mullvad-dropdown"],
            set_has_arrow: false,
            #[watch]
            set_width_request: model.scaled_width,
            #[watch]
            set_height_request: model.scaled_height,

            #[template]
            Dropdown {
                set_overflow: gtk::Overflow::Hidden,

                #[template]
                DropdownHeader {
                    #[template_child]
                    icon {
                        set_visible: true,
                        set_icon_name: Some("network-vpn-symbolic"),
                    },
                    #[template_child]
                    label {
                        set_label: &t!("dropdown-mullvad-title"),
                    },
                },

                #[template]
                DropdownContent {
                    add_css_class: "mullvad-content",
                    set_vexpand: true,

                    #[name = "empty_unavailable"]
                    #[template]
                    EmptyState {
                        #[watch]
                        set_visible: !model.ready,
                        #[template_child]
                        icon {
                            set_icon_name: Some("network-vpn-disabled-symbolic"),
                        },
                        #[template_child]
                        title {
                            set_label: &t!("dropdown-mullvad-unavailable-title"),
                        },
                        #[template_child]
                        description {
                            set_label: &t!("dropdown-mullvad-unavailable-description"),
                        },
                    },

                    #[name = "empty_logged_out"]
                    #[template]
                    EmptyState {
                        #[watch]
                        set_visible: model.ready && model.logged_out(),
                        #[template_child]
                        icon {
                            set_icon_name: Some("network-vpn-disabled-symbolic"),
                        },
                        #[template_child]
                        title {
                            set_label: &t!("dropdown-mullvad-logged-out-title"),
                        },
                        #[template_child]
                        description {
                            set_label: &t!("dropdown-mullvad-logged-out-description"),
                        },
                    },

                    #[name = "empty_revoked"]
                    #[template]
                    EmptyState {
                        #[watch]
                        set_visible: model.ready && model.revoked(),
                        #[template_child]
                        icon {
                            set_icon_name: Some("network-vpn-disabled-symbolic"),
                        },
                        #[template_child]
                        title {
                            set_label: &t!("dropdown-mullvad-revoked-title"),
                        },
                        #[template_child]
                        description {
                            set_label: &t!("dropdown-mullvad-revoked-description"),
                        },
                    },

                    gtk::Box {
                        add_css_class: "mullvad-body",
                        set_orientation: gtk::Orientation::Vertical,
                        set_vexpand: true,
                        #[watch]
                        set_visible: model.ready && model.account_active(),

                        gtk::Label {
                            add_css_class: "section-label",
                            set_halign: gtk::Align::Start,
                            set_label: &t!("dropdown-mullvad-current"),
                        },

                        #[local_ref]
                        current_widget -> gtk::Box {},

                        gtk::Label {
                            add_css_class: "section-label",
                            set_halign: gtk::Align::Start,
                            set_label: &t!("dropdown-mullvad-available"),
                        },

                        #[template]
                        Card {
                            add_css_class: "mullvad-list",
                            set_overflow: gtk::Overflow::Hidden,
                            set_vexpand: true,

                            gtk::ScrolledWindow {
                                add_css_class: "mullvad-list-scroll",
                                set_vexpand: true,
                                set_hscrollbar_policy: gtk::PolicyType::Never,

                                #[local_ref]
                                countries_widget -> gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                },
                            },
                        },
                    },
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let current = CurrentConnection::builder()
            .launch(())
            .forward(sender.input_sender(), MullvadDropdownMsg::Current);

        let countries = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .forward(sender.input_sender(), MullvadDropdownMsg::Country);

        let scale = init.config.config().styling.scale.get().value();

        watchers::spawn_config_watcher(&sender, &init.config);
        watchers::spawn_service_watcher(&sender, &init.mullvad);

        let model = Self {
            mullvad: None,
            scaled_width: scaled_dimension(BASE_WIDTH, scale),
            scaled_height: scaled_dimension(BASE_HEIGHT, scale),
            ready: false,
            status: ConnectionStatus::default(),
            current,
            countries,
            state_watcher: WatcherToken::new(),
        };

        let current_widget = model.current.widget();
        let countries_widget = model.countries.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            MullvadDropdownMsg::Country(CountryItemOutput::Select(target)) => {
                self.select(&target);
            }
            MullvadDropdownMsg::Country(CountryItemOutput::Expanded(index)) => {
                let expanded = index.current_index();
                for i in 0..self.countries.len() {
                    if i != expanded {
                        self.countries.send(i, CountryItemInput::Collapse);
                    }
                }
            }
            MullvadDropdownMsg::Current(CurrentConnectionOutput::ToggleRequested) => {
                self.toggle();
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: MullvadDropdownCmd,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            MullvadDropdownCmd::ServiceReady(service) => {
                self.ready = true;
                self.status = service.mullvad.status.get();

                let token = self.state_watcher.reset();
                watchers::spawn_state_watchers(&sender, token, &service);

                self.mullvad = Some(service);
                // The state watchers above emit their current value on
                // subscribe, so RelaysChanged/TunnelChanged fire immediately and
                // populate the tree + card — no inline rebuild needed here (it
                // would rebuild the whole tree twice).
            }
            MullvadDropdownCmd::ScaleChanged(scale) => {
                self.scaled_width = scaled_dimension(BASE_WIDTH, scale);
                self.scaled_height = scaled_dimension(BASE_HEIGHT, scale);
            }
            MullvadDropdownCmd::RelaysChanged => {
                self.rebuild_countries();
            }
            MullvadDropdownCmd::TunnelChanged => {
                if let Some(service) = &self.mullvad {
                    self.status = service.mullvad.status.get();
                }
                self.push_status();
            }
        }
    }
}

impl MullvadDropdown {
    /// Whether no account is logged in (drives the logged-out empty state).
    fn logged_out(&self) -> bool {
        matches!(self.status, ConnectionStatus::LoggedOut)
    }

    /// Whether the account's device was revoked (drives the revoked empty state).
    fn revoked(&self) -> bool {
        matches!(self.status, ConnectionStatus::Revoked)
    }

    /// Whether an account is usable (logged in, not revoked) — drives the main
    /// body with the relay list.
    fn account_active(&self) -> bool {
        !self.logged_out() && !self.revoked()
    }

    /// Selects `target` as the relay location without connecting. The daemon
    /// persists it and — if a tunnel is already up — reconnects to it; while
    /// disconnected it just becomes the location a later connect will use. The
    /// current-connection card's action button is how the user actually
    /// connects. Non-blocking.
    fn select(&self, target: &NetworkTarget) {
        if let Some(service) = &self.mullvad {
            service.mullvad.select(target);
        }
    }

    /// Connect when disconnected, otherwise disconnect (non-blocking). A no-op
    /// without a logged-in account.
    fn toggle(&self) {
        let Some(service) = &self.mullvad else {
            return;
        };
        match service.mullvad.status.get() {
            ConnectionStatus::Disconnected => service.mullvad.connect(),
            ConnectionStatus::LoggedOut | ConnectionStatus::Revoked => {}
            _ => service.mullvad.disconnect(),
        }
    }
}
