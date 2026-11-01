mod messages;
mod watchers;

use std::{env, path::PathBuf, sync::Arc};

use gtk::{CssProvider, gdk::Display, prelude::*, style_context_add_provider_for_display};
use relm4::{gtk, prelude::*};
use wayle_config::ConfigService;
use wayle_widgets::prelude::IconButton;

pub(crate) use self::messages::UserSessionInit;
use self::messages::{UserSessionCmd, UserSessionInput};
use crate::{i18n::t, process, shell::helpers::COMPONENT_CSS_PRIORITY};

pub(crate) struct UserSessionSection {
    username: String,
    has_face: bool,
    face_path: PathBuf,
    face_css_provider: CssProvider,
    config: Arc<ConfigService>,
}

impl UserSessionSection {
    fn update_face_css(&self) {
        let css = if self.has_face {
            let path = self.face_path.display();
            format!(".user-avatar {{ background-image: url(\"file://{path}\"); }}")
        } else {
            String::from(".user-avatar { background-image: none; }")
        };
        self.face_css_provider.load_from_string(&css);
    }
}

#[relm4::component(pub(crate))]
impl Component for UserSessionSection {
    type Init = UserSessionInit;
    type Input = UserSessionInput;
    type Output = ();
    type CommandOutput = UserSessionCmd;

    view! {
        #[root]
        gtk::Box {
            set_css_classes: &["card", "dashboard-card"],
            set_orientation: gtk::Orientation::Vertical,

            #[name = "session_row"]
            gtk::Box {
                add_css_class: "dashboard-user-session",

                #[name = "user_info"]
                gtk::Box {
                    add_css_class: "user-info",
                    set_halign: gtk::Align::Start,

                    #[name = "avatar"]
                    gtk::Box {
                        add_css_class: "user-avatar",
                        set_halign: gtk::Align::Start,
                        set_valign: gtk::Align::Start,

                        gtk::Image {
                            set_icon_name: Some("ld-user-symbolic"),
                            set_hexpand: true,
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                            #[watch]
                            set_visible: !model.has_face,
                        },
                    },

                    #[name = "user_meta"]
                    gtk::Box {
                        set_valign: gtk::Align::Center,
                        set_halign: gtk::Align::Start,

                        #[name = "username_label"]
                        gtk::Label {
                            add_css_class: "user-name",
                            set_halign: gtk::Align::Start,
                            #[watch]
                            set_label: &model.username,
                        },
                    },
                },

                #[name = "session_actions"]
                gtk::Box {
                    add_css_class: "session-actions",
                    set_hexpand: true,
                    set_halign: gtk::Align::End,

                    #[template]
                    #[name = "lock_btn"]
                    IconButton {
                        add_css_class: "session-btn",
                        set_tooltip_text: Some(&t!("dropdown-dashboard-lock")),
                        connect_clicked => UserSessionInput::Lock,

                        gtk::Image {
                            set_icon_name: Some("ld-lock-symbolic"),
                        },
                    },

                    #[template]
                    #[name = "logout_btn"]
                    IconButton {
                        add_css_class: "session-btn",
                        set_tooltip_text: Some(&t!("dropdown-dashboard-logout")),
                        connect_clicked => UserSessionInput::Logout,

                        gtk::Image {
                            set_icon_name: Some("ld-log-out-symbolic"),
                        },
                    },

                    #[template]
                    #[name = "reboot_btn"]
                    IconButton {
                        add_css_class: "session-btn",
                        set_tooltip_text: Some(&t!("dropdown-dashboard-reboot")),
                        connect_clicked => UserSessionInput::Reboot,

                        gtk::Image {
                            set_icon_name: Some("ld-refresh-cw-symbolic"),
                        },
                    },

                    #[template]
                    #[name = "power_off_btn"]
                    IconButton {
                        set_css_classes: &["icon", "session-btn", "danger"],
                        set_tooltip_text: Some(&t!("dropdown-dashboard-power-off")),
                        connect_clicked => UserSessionInput::PowerOff,

                        gtk::Image {
                            set_icon_name: Some("ld-power-symbolic"),
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
        let face_path = env::var_os("HOME")
            .map(|home| PathBuf::from(home).join(".face"))
            .unwrap_or_default();

        let has_face = face_path.exists();

        let face_css_provider = CssProvider::new();

        #[allow(clippy::expect_used)]
        let display = Display::default().expect("display required for user session");
        style_context_add_provider_for_display(
            &display,
            &face_css_provider,
            COMPONENT_CSS_PRIORITY,
        );

        watchers::spawn_face_watcher(&sender, &face_path);

        let model = Self {
            username: init.username,
            has_face,
            face_path,
            face_css_provider,
            config: init.config,
        };

        model.update_face_css();

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let dashboard = &self.config.config().modules.dashboard;

        match msg {
            UserSessionInput::Lock => {
                process::run_if_set(&dashboard.dropdown_lock_command.get());
            }
            UserSessionInput::Logout => {
                process::run_if_set(&dashboard.dropdown_logout_command.get());
            }
            UserSessionInput::Reboot => {
                process::run_if_set(&dashboard.dropdown_reboot_command.get());
            }
            UserSessionInput::PowerOff => {
                process::run_if_set(&dashboard.dropdown_poweroff_command.get());
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            UserSessionCmd::FaceChanged(exists) => {
                self.has_face = exists;
                self.update_face_css();
            }
        }
    }
}
