mod factory;
mod helpers;
mod messages;
mod methods;
mod watchers;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::prelude::*;
use wayle_bluetooth::BluetoothService;
use wayle_config::{ConfigProperty, ConfigService, schemas::styling::CssToken};
use wayle_core::DeferredService;
use wayle_widgets::{
    WatcherToken,
    prelude::{BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonOutput},
};

pub(crate) use self::{
    factory::Factory,
    messages::{BluetoothCmd, BluetoothInit, BluetoothMsg},
};
use crate::shell::bar::dropdowns::DropdownOpener;

pub(crate) struct BluetoothModule {
    bar_button: Controller<BarButton>,
    state_watcher: WatcherToken,
    adapter_watcher: WatcherToken,
    bluetooth: DeferredService<BluetoothService>,
    config: Arc<ConfigService>,
    opener: DropdownOpener,
}

#[relm4::component(pub(crate))]
impl Component for BluetoothModule {
    type Init = BluetoothInit;
    type Input = BluetoothMsg;
    type Output = ();
    type CommandOutput = BluetoothCmd;

    view! {
        gtk::Box {
            add_css_class: "bluetooth",

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
        let bt_config = &config.modules.bluetooth;

        let (initial_icon, initial_label) = Self::compute_display(bt_config, &init.bluetooth.get());

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: initial_icon,
                label: initial_label,
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: bt_config.icon_color.clone(),
                    label_color: bt_config.label_color.clone(),
                    icon_background: bt_config.icon_bg_color.clone(),
                    button_background: bt_config.button_bg_color.clone(),
                    border_color: bt_config.border_color.clone(),
                    auto_icon_color: CssToken::Blue,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: bt_config.label_max_length.clone(),
                    show_icon: bt_config.icon_show.clone(),
                    show_label: bt_config.label_show.clone(),
                    show_border: bt_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => BluetoothMsg::LeftClick,
                BarButtonOutput::RightClick => BluetoothMsg::RightClick,
                BarButtonOutput::MiddleClick => BluetoothMsg::MiddleClick,
                BarButtonOutput::ScrollUp => BluetoothMsg::ScrollUp,
                BarButtonOutput::ScrollDown => BluetoothMsg::ScrollDown,
            });

        watchers::spawn_service_watcher(&sender, &init.bluetooth);
        let adapter_watcher = WatcherToken::new();
        let state_watcher = WatcherToken::new();

        let opener = DropdownOpener::for_button(
            &init.dropdowns,
            &bar_button,
            bt_config.clone(),
        );

        let model = Self {
            bar_button,
            state_watcher,
            adapter_watcher,
            bluetooth: init.bluetooth,
            config: init.config,
            opener,
        };
        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let config = &self.config.config().modules.bluetooth;

        let action = match msg {
            BluetoothMsg::LeftClick => config.left_click.get(),
            BluetoothMsg::RightClick => config.right_click.get(),
            BluetoothMsg::MiddleClick => config.middle_click.get(),
            BluetoothMsg::ScrollUp => config.scroll_up.get(),
            BluetoothMsg::ScrollDown => config.scroll_down.get(),
        };

        self.opener.dispatch(&action);
    }

    fn update_cmd(&mut self, msg: BluetoothCmd, sender: ComponentSender<Self>, _root: &Self::Root) {
        let bt_config = &self.config.config().modules.bluetooth;

        match msg {
            BluetoothCmd::ServiceReady(bt) => {
                let state_token = self.state_watcher.reset();
                watchers::spawn_watchers(&sender, state_token, bt_config, &bt);
                watchers::spawn_adapter_watchers(&sender, self.adapter_watcher.reset(), &bt);

                self.update_display(bt_config, &Some(bt));
            }

            BluetoothCmd::StateChanged | BluetoothCmd::IconConfigChanged => {
                self.update_display(bt_config, &self.bluetooth.get());
            }

            BluetoothCmd::AdapterChanged => {
                let Some(bt) = self.bluetooth.get() else {
                    return;
                };

                watchers::spawn_adapter_watchers(&sender, self.adapter_watcher.reset(), &bt);
                self.update_display(bt_config, &Some(bt));
            }
        }
    }
}
