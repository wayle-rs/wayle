mod methods;
mod watchers;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_config::{
    ConfigService,
    schemas::modules::notification::{IconSource, PopupCloseBehavior, UrgencyBarThreshold},
};
use wayle_notification::core::notification::Notification;

use super::{
    helpers::{
        ResolvedIcon, open_body_link, priority_bar_visible, relative_time, render_body,
        resolve_notification_icon,
    },
    templates::NotificationContentTemplate,
};
use crate::i18n::t;

/// Initialization data for a single popup card.
pub(crate) struct CardInit {
    pub(crate) notification: Arc<Notification>,
    pub(crate) config: Arc<ConfigService>,
    pub(crate) hover_pause: bool,
    pub(crate) close_behavior: PopupCloseBehavior,
    pub(crate) urgency_bar: UrgencyBarThreshold,
    pub(crate) icon_source: IconSource,
    pub(crate) shadow: bool,
}

/// Configuration change commands for popup cards.
#[derive(Debug)]
pub(crate) enum CardCmd {
    ConfigChanged {
        shadow: bool,
        urgency_bar: UrgencyBarThreshold,
    },
    /// The underlying notification's content/actions changed; re-render in place.
    NotificationChanged,
}

/// A single notification popup card.
pub(crate) struct NotificationPopupCard {
    notification: Arc<Notification>,
    hover_pause: bool,
    close_behavior: PopupCloseBehavior,
    resolved_icon: ResolvedIcon,
    app_label: String,
    time_label: String,
    icon_source: IconSource,
    urgency_bar: UrgencyBarThreshold,
    icon: Option<gtk::Image>,
    icon_container: Option<gtk::Box>,
    actions_box: Option<gtk::Box>,
    default_gesture: Option<gtk::GestureClick>,
}

#[relm4::component(pub(crate))]
impl Component for NotificationPopupCard {
    type Init = CardInit;
    type Input = ();
    type Output = ();
    type CommandOutput = CardCmd;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "notification-popup-card",
            set_orientation: gtk::Orientation::Vertical,

            gtk::Box {
                set_widget_name: "notification-content-row",
                add_css_class: "notification-popup-content",

                #[name = "icon_container"]
                gtk::Box {
                    add_css_class: "notification-popup-icon",
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Start,

                    #[name = "icon"]
                    gtk::Image {
                        add_css_class: "notification-popup-icon-img",
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                    },
                },

                #[template]
                NotificationContentTemplate {
                    #[template_child]
                    app_label {
                        #[watch]
                        set_label: &model.app_label,
                    },
                    #[template_child]
                    time_label {
                        set_label: &model.time_label,
                    },
                    #[template_child]
                    title {
                        #[watch]
                        set_label: &model.notification.view.get().content.summary,
                    },
                    #[template_child]
                    body {
                        #[watch]
                        set_label: &model
                            .notification
                            .view.get().content
                            .body
                            .map_or_else(String::new, |body| {
                                render_body(body.text(), body.is_markup())
                            }),
                        #[watch]
                        set_visible: model.notification.view.get().content.body.is_some(),
                        // Open <a href> links via the portal; stops GTK's crashing default handler.
                        connect_activate_link[notification = model.notification.clone()] => move |_, uri| {
                            open_body_link(&notification, uri)
                        },
                    },
                },

                gtk::Button {
                    set_widget_name: "notification-dismiss-btn",
                    add_css_class: "notification-popup-close",
                    set_icon_name: "window-close-symbolic",
                    set_valign: gtk::Align::Start,
                    set_cursor_from_name: Some("pointer"),
                    // An app that forbids manual dismissal (portal `persistent`) hides the close X.
                    set_visible: !model.notification.view.get().lifecycle.locked_open,
                    connect_clicked[
                        notification = model.notification.clone(),
                        close_behavior = model.close_behavior,
                    ] => move |_| {
                        match close_behavior {
                            PopupCloseBehavior::Dismiss => notification.dismiss_popup(),
                            PopupCloseBehavior::Remove => notification.dismiss(),
                        }
                    },
                },
            },

            #[name = "actions_box"]
            gtk::Box {
                add_css_class: "notification-popup-actions",
                set_orientation: gtk::Orientation::Vertical,
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let notif = &init.notification;

        let resolved_icon = resolve_notification_icon(init.icon_source, notif);

        let app_label = notif
            .view.get().origin
            .name
            .unwrap_or_else(|| t!("notification-popup-unknown-app"));

        let time_label = Self::format_time_label(relative_time(&notif.view.get().received));

        let mut model = Self {
            notification: init.notification,
            hover_pause: init.hover_pause,
            close_behavior: init.close_behavior,
            resolved_icon,
            app_label,
            time_label,
            icon_source: init.icon_source,
            urgency_bar: init.urgency_bar,
            icon: None,
            icon_container: None,
            actions_box: None,
            default_gesture: None,
        };

        let widgets = view_output!();

        model.apply_css_classes(&root, init.shadow, init.urgency_bar);
        model.apply_priority_class(&root);
        model.apply_icon(&widgets.icon, &widgets.icon_container);
        model.setup_action_buttons(&widgets.actions_box);
        let default_gesture = model.setup_default_action(&root);
        model.default_gesture = default_gesture;
        model.setup_hover_controller(&root);

        model.icon = Some(widgets.icon.clone());
        model.icon_container = Some(widgets.icon_container.clone());
        model.actions_box = Some(widgets.actions_box.clone());

        watchers::spawn(&sender, &init.config);
        watchers::spawn_notification(&sender, &model.notification);

        ComponentParts { model, widgets }
    }

    fn update_cmd(&mut self, msg: CardCmd, _sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            CardCmd::ConfigChanged {
                shadow,
                urgency_bar,
            } => {
                self.urgency_bar = urgency_bar;

                if shadow {
                    root.add_css_class("shadow");
                } else {
                    root.remove_css_class("shadow");
                }

                if priority_bar_visible(self.notification.view.get().classification.priority, urgency_bar)
                {
                    root.add_css_class("urgency-bar");
                } else {
                    root.remove_css_class("urgency-bar");
                }
            }
            CardCmd::NotificationChanged => {
                self.refresh_notification(root);
            }
        }
    }
}
