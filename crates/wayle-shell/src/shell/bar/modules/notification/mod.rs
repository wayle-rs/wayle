mod factory;
mod helpers;
mod messages;
mod methods;
mod watchers;

use std::{rc::Rc, sync::Arc};

use gtk::prelude::*;
use relm4::prelude::*;
use wayle_config::{ConfigProperty, ConfigService, schemas::styling::CssToken};
use wayle_widgets::prelude::{
    BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonInput, BarButtonOutput,
};

use self::helpers::{IconContext, format_label, select_icon};
pub(crate) use self::{
    factory::Factory,
    messages::{NotificationCmd, NotificationInit, NotificationMsg},
};
use crate::shell::bar::dropdowns::{self, DropdownRegistry};

pub(crate) struct NotificationModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    count: usize,
    dnd: bool,
    dropdowns: Rc<DropdownRegistry>,
}

#[relm4::component(pub(crate))]
impl Component for NotificationModule {
    type Init = NotificationInit;
    type Input = NotificationMsg;
    type Output = ();
    type CommandOutput = NotificationCmd;

    view! {
        gtk::Box {
            add_css_class: "notification",

            #[local_ref]
            bar_button -> gtk::MenuButton {},
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = init.config.config();
        let notification_config = &config.modules.notification;

        let initial_count = init.notification.notifications.get().len();
        let initial_dnd = init.notification.dnd.get();

        let initial_icon = select_icon(&IconContext {
            count: initial_count,
            dnd: initial_dnd,
            icon_name: &notification_config.icon_name.get(),
            icon_unread: &notification_config.icon_unread.get(),
            icon_dnd: &notification_config.icon_dnd.get(),
        });

        let initial_label = format_label(initial_count);

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: initial_icon,
                label: initial_label,
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: notification_config.icon_color.clone(),
                    label_color: notification_config.label_color.clone(),
                    icon_background: notification_config.icon_bg_color.clone(),
                    button_background: notification_config.button_bg_color.clone(),
                    border_color: notification_config.border_color.clone(),
                    auto_icon_color: CssToken::Green,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: notification_config.label_max_length.clone(),
                    show_icon: notification_config.icon_show.clone(),
                    show_label: notification_config.label_show.clone(),
                    show_border: notification_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => NotificationMsg::LeftClick,
                BarButtonOutput::RightClick => NotificationMsg::RightClick,
                BarButtonOutput::MiddleClick => NotificationMsg::MiddleClick,
                BarButtonOutput::ScrollUp => NotificationMsg::ScrollUp,
                BarButtonOutput::ScrollDown => NotificationMsg::ScrollDown,
            });

        watchers::spawn_watchers(&sender, notification_config, &init.notification);

        let model = Self {
            bar_button,
            config: init.config,
            count: initial_count,
            dnd: initial_dnd,
            dropdowns: init.dropdowns,
        };
        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let config = &self.config.config().modules.notification;

        let action = match msg {
            NotificationMsg::LeftClick => config.left_click.get(),
            NotificationMsg::RightClick => config.right_click.get(),
            NotificationMsg::MiddleClick => config.middle_click.get(),
            NotificationMsg::ScrollUp => config.scroll_up.get(),
            NotificationMsg::ScrollDown => config.scroll_down.get(),
        };

        dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
    }

    fn update_cmd(
        &mut self,
        msg: NotificationCmd,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        let notification_config = &self.config.config().modules.notification;

        match msg {
            NotificationCmd::NotificationsChanged(count) => {
                self.count = count;
                self.update_display(notification_config);
            }
            NotificationCmd::DndChanged(dnd) => {
                self.dnd = dnd;
                self.update_display(notification_config);
            }
            NotificationCmd::IconConfigChanged => {
                self.update_display(notification_config);
            }
            NotificationCmd::UpdateThresholdColors(colors) => {
                self.bar_button
                    .emit(BarButtonInput::SetThresholdColors(colors));
            }
        }
    }
}
