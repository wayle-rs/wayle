mod factory;
mod helpers;
mod messages;
mod methods;
mod watchers;

use std::{rc::Rc, sync::Arc};

use gtk::prelude::*;
use relm4::prelude::*;
use wayle_config::{ConfigProperty, ConfigService, schemas::styling::CssToken};
use wayle_iwd::IwdService;
use wayle_widgets::{
    WatcherToken,
    prelude::{
        BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonInput,
        BarButtonOutput,
    },
};

pub(crate) use self::{
    factory::Factory,
    messages::{IwdCmd, IwdInit, IwdMsg},
};
use crate::shell::bar::dropdowns::{self, DropdownRegistry};

pub(crate) struct IwdModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    station_watcher: WatcherToken,
    iwd: Arc<IwdService>,
    dropdowns: Rc<DropdownRegistry>,
}

#[relm4::component(pub(crate))]
impl Component for IwdModule {
    type Init = IwdInit;
    type Input = IwdMsg;
    type Output = ();
    type CommandOutput = IwdCmd;

    view! {
        gtk::Box {
            add_css_class: "iwd",

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
        let iwd_config = &config.modules.iwd;

        let (initial_icon, initial_label) = Self::compute_display(iwd_config, &init.iwd);

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: initial_icon,
                label: initial_label,
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: iwd_config.icon_color.clone(),
                    label_color: iwd_config.label_color.clone(),
                    icon_background: iwd_config.icon_bg_color.clone(),
                    button_background: iwd_config.button_bg_color.clone(),
                    border_color: iwd_config.border_color.clone(),
                    auto_icon_color: CssToken::Accent,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: iwd_config.label_max_length.clone(),
                    show_icon: iwd_config.icon_show.clone(),
                    show_label: iwd_config.label_show.clone(),
                    show_border: iwd_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => IwdMsg::LeftClick,
                BarButtonOutput::RightClick => IwdMsg::RightClick,
                BarButtonOutput::MiddleClick => IwdMsg::MiddleClick,
                BarButtonOutput::ScrollUp => IwdMsg::ScrollUp,
                BarButtonOutput::ScrollDown => IwdMsg::ScrollDown,
            });

        watchers::spawn_watchers(&sender, iwd_config, &init.iwd);

        let mut station_watcher = WatcherToken::new();
        watchers::spawn_station_watchers(&sender, &init.iwd, station_watcher.reset());

        let model = Self {
            bar_button,
            config: init.config,
            station_watcher,
            iwd: init.iwd,
            dropdowns: init.dropdowns,
        };
        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let config = &self.config.config().modules.iwd;

        let action = match msg {
            IwdMsg::LeftClick => config.left_click.get(),
            IwdMsg::RightClick => config.right_click.get(),
            IwdMsg::MiddleClick => config.middle_click.get(),
            IwdMsg::ScrollUp => config.scroll_up.get(),
            IwdMsg::ScrollDown => config.scroll_down.get(),
        };

        dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
    }

    fn update_cmd(&mut self, msg: IwdCmd, sender: ComponentSender<Self>, _root: &Self::Root) {
        let iwd_config = &self.config.config().modules.iwd;

        match msg {
            IwdCmd::StateChanged | IwdCmd::IconConfigChanged => {
                let (icon, label) = Self::compute_display(iwd_config, &self.iwd);
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
                self.bar_button.emit(BarButtonInput::SetLabel(label));
            }
            IwdCmd::StationDeviceChanged => {
                let token = self.station_watcher.reset();
                watchers::spawn_station_watchers(&sender, &self.iwd, token);

                let (icon, label) = Self::compute_display(iwd_config, &self.iwd);
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
                self.bar_button.emit(BarButtonInput::SetLabel(label));
            }
        }
    }
}
