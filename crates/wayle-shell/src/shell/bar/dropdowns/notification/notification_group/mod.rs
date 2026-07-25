pub(crate) mod messages;
mod methods;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_config::schemas::modules::notification::IconSource;
use wayle_notification::core::notification::Notification;

use self::messages::{NotificationGroupInit, NotificationGroupInput, NotificationGroupOutput};
use super::notification_item::{NotificationItem, messages::NotificationItemInput};
use crate::i18n::t;

pub(crate) struct NotificationGroup {
    pub(crate) app_name: Option<String>,

    expanded: bool,
    count: usize,
    preview: String,
    overflow_count: usize,
    total_count: usize,

    icon_source: IconSource,
    group_icon: Option<String>,

    items: FactoryVecDeque<NotificationItem>,
    notifications: Vec<Arc<Notification>>,
}

#[relm4::factory(pub(crate))]
impl FactoryComponent for NotificationGroup {
    type Init = NotificationGroupInit;
    type Input = NotificationGroupInput;
    type Output = NotificationGroupOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "notification-dropdown-group",
            set_orientation: gtk::Orientation::Vertical,

            #[name = "header"]
            gtk::Box {
                add_css_class: "notification-dropdown-group-header",
                set_cursor_from_name: Some("pointer"),

                gtk::Box {
                    add_css_class: "notification-dropdown-group-icon",
                    set_valign: gtk::Align::Center,

                    gtk::Image {
                        add_css_class: "notification-dropdown-group-icon-img",
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        #[watch]
                        set_icon_name: self.group_icon.as_deref(),
                    },
                },

                gtk::Box {
                    add_css_class: "notification-dropdown-group-info",
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,
                    #[watch]
                    set_valign: if self.expanded {
                        gtk::Align::Center
                    } else {
                        gtk::Align::Start
                    },

                    gtk::Box {
                        gtk::Label {
                            add_css_class: "notification-dropdown-group-name",
                            set_halign: gtk::Align::Start,
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            #[watch]
                            set_label: self.app_name.as_deref().unwrap_or(
                                &t!("notification-dropdown-unknown-app")
                            ),
                        },

                        gtk::Label {
                            add_css_class: "notification-dropdown-group-count",
                            #[watch]
                            set_label: &format!("({})", self.count),
                            #[watch]
                            set_visible: self.count > 1,
                        },
                    },

                    gtk::Label {
                        add_css_class: "notification-dropdown-group-preview",
                        set_halign: gtk::Align::Start,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        #[watch]
                        set_label: &self.preview,
                        #[watch]
                        set_visible: !self.expanded,
                    },
                },

                gtk::Box {
                    add_css_class: "notification-dropdown-group-actions",

                    #[name = "clear_btn"]
                    gtk::Button {
                        add_css_class: "notification-dropdown-group-clear",
                        set_css_classes: &["btn", "btn-ghost", "btn-sm", "notification-dropdown-group-clear"],
                        set_label: &t!("notification-dropdown-group-clear"),
                        set_valign: gtk::Align::Center,
                        set_cursor_from_name: Some("pointer"),
                    },

                    gtk::Box {
                        add_css_class: "notification-dropdown-group-chevron",
                        set_valign: gtk::Align::Center,

                        gtk::Image {
                            add_css_class: "notification-dropdown-group-chevron-icon",
                            #[watch]
                            set_icon_name: Some(
                                if self.expanded {
                                    "ld-chevron-up-symbolic"
                                } else {
                                    "ld-chevron-down-symbolic"
                                }
                            ),
                        },
                    },
                },
            },

            gtk::Box {
                add_css_class: "notification-dropdown-group-items",
                set_orientation: gtk::Orientation::Vertical,
                #[watch]
                set_visible: self.expanded,

                #[local_ref]
                items_widget -> gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                },

                #[name = "more_label"]
                gtk::Label {
                    add_css_class: "notification-dropdown-group-more",
                    set_cursor_from_name: Some("pointer"),
                    #[watch]
                    set_label: &t!(
                        "notification-dropdown-group-more",
                        count = self.overflow_count.to_string()
                    ),
                    #[watch]
                    set_visible: self.overflow_count > 0,
                },
            },
        }
    }

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        // Items dismiss their own notification directly (the reactive list update then drops
        // them), so they emit no output and need no forwarding.
        let items = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();

        let group_icon = Self::resolve_group_icon(init.icon_source, &init.notifications);

        let mut model = Self {
            app_name: init.app_name,
            expanded: true,
            count: 0,
            preview: String::new(),
            overflow_count: 0,
            total_count: 0,
            icon_source: init.icon_source,
            group_icon,
            items,
            notifications: Vec::new(),
        };

        model.reconcile_items(init.notifications);
        model
    }

    fn init_widgets(
        &mut self,
        _index: &Self::Index,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let items_widget = self.items.widget();
        let widgets = view_output!();

        let header_sender = sender.input_sender().clone();
        let header_gesture = gtk::GestureClick::new();

        header_gesture.connect_released(move |gesture, _, _, _| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            header_sender.emit(NotificationGroupInput::ToggleExpanded);
        });

        widgets.header.add_controller(header_gesture);

        let clear_sender = sender.input_sender().clone();

        widgets.clear_btn.connect_clicked(move |_| {
            clear_sender.emit(NotificationGroupInput::ClearGroup);
        });

        let more_sender = sender.input_sender().clone();
        let more_gesture = gtk::GestureClick::new();

        more_gesture.connect_released(move |gesture, _, _, _| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            more_sender.emit(NotificationGroupInput::ShowAll);
        });

        widgets.more_label.add_controller(more_gesture);

        widgets
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            NotificationGroupInput::ToggleExpanded => {
                self.expanded = !self.expanded;
                if !self.expanded {
                    self.reset_to_default_cap();
                }
            }

            NotificationGroupInput::ShowAll => {
                self.show_all_items();
            }

            NotificationGroupInput::UpdateNotifications(notifications) => {
                self.group_icon = Self::resolve_group_icon(self.icon_source, &notifications);
                self.reconcile_items(notifications);
            }

            NotificationGroupInput::RefreshTime => {
                for idx in 0..self.items.len() {
                    self.items.send(idx, NotificationItemInput::RefreshTime);
                }
            }

            NotificationGroupInput::ClearGroup => {
                sender
                    .output(NotificationGroupOutput::ClearRequested(
                        self.notifications.clone(),
                    ))
                    .ok();
            }
        }
    }
}
