mod methods;
mod watchers;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_config::{
    ConfigService,
    schemas::modules::notification::{IconSource, PopupCloseBehavior, UrgencyBarThreshold},
};
use wayle_notification::{NotificationService, core::notification::Notification};

use super::{
    helpers::{
        ResolvedIcon, relative_time, resolve_icon, sanitize_markup, urgency_bar_visible,
        urgency_css_class,
    },
    templates::NotificationContentTemplate,
};
use crate::i18n::t;

/// Initialization data for a single popup card.
pub(crate) struct CardInit {
    pub(crate) notification: Arc<Notification>,
    pub(crate) service: Arc<NotificationService>,
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
}

/// A single notification popup card.
pub(crate) struct NotificationPopupCard {
    notification: Arc<Notification>,
    service: Arc<NotificationService>,
    hover_pause: bool,
    close_behavior: PopupCloseBehavior,
    resolved_icon: ResolvedIcon,
    app_label: String,
    time_label: String,
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
            add_css_class: urgency_css_class(model.notification.urgency.get()),
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
                        set_label: &model.app_label,
                    },
                    #[template_child]
                    time_label {
                        set_label: &model.time_label,
                    },
                    #[template_child]
                    title {
                        set_label: &model.notification.summary.get(),
                    },
                    #[template_child]
                    body {
                        set_label: &model
                            .notification
                            .body
                            .get()
                            .as_deref()
                            .map_or_else(String::new, sanitize_markup),
                        set_visible: model.notification.body.get().is_some(),
                    },
                },

                gtk::Button {
                    set_widget_name: "notification-dismiss-btn",
                    add_css_class: "notification-popup-close",
                    set_icon_name: "window-close-symbolic",
                    set_valign: gtk::Align::Start,
                    set_cursor_from_name: Some("pointer"),
                    connect_clicked[
                        service = model.service.clone(),
                        notification = model.notification.clone(),
                        close_behavior = model.close_behavior,
                    ] => move |_| {
                        match close_behavior {
                            PopupCloseBehavior::Dismiss => service.dismiss_popup(notification.id),
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

        let prefer_color = init.config.config().general.prefer_color_icons.get();
        let resolved_icon = resolve_icon(
            init.icon_source,
            &notif.app_name.get(),
            &notif.app_icon.get(),
            &notif.image_path.get(),
            &notif.desktop_entry.get(),
            prefer_color,
        );

        let app_label = notif
            .app_name
            .get()
            .unwrap_or_else(|| t!("notification-popup-unknown-app"));

        let time_label = Self::format_time_label(relative_time(&notif.timestamp.get()));

        let model = Self {
            notification: init.notification,
            service: init.service,
            hover_pause: init.hover_pause,
            close_behavior: init.close_behavior,
            resolved_icon,
            app_label,
            time_label,
        };

        let widgets = view_output!();

        model.apply_css_classes(&root, init.shadow, init.urgency_bar);
        model.apply_icon(&widgets.icon, &widgets.icon_container);
        model.setup_action_buttons(&widgets.actions_box);
        model.setup_default_action(&root);
        model.setup_hover_controller(&root);

        watchers::spawn(&sender, &init.config);

        ComponentParts { model, widgets }
    }

    fn update_cmd(&mut self, msg: CardCmd, _sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            CardCmd::ConfigChanged {
                shadow,
                urgency_bar,
            } => {
                if shadow {
                    root.add_css_class("shadow");
                } else {
                    root.remove_css_class("shadow");
                }

                if urgency_bar_visible(self.notification.urgency.get(), urgency_bar) {
                    root.add_css_class("urgency-bar");
                } else {
                    root.remove_css_class("urgency-bar");
                }
            }
        }
    }
}
