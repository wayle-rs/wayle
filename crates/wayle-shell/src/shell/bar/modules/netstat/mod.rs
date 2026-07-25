mod factory;
mod helpers;
mod messages;
mod watchers;

use std::{rc::Rc, sync::Arc};

use gtk::prelude::*;
use relm4::prelude::*;
use wayle_config::{ConfigProperty, ConfigService, schemas::styling::CssToken};
use wayle_widgets::prelude::{
    BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonInput, BarButtonOutput,
};

pub(crate) use self::{
    factory::Factory,
    messages::{NetstatCmd, NetstatInit, NetstatMsg},
};
use crate::shell::bar::dropdowns::{self, DropdownRegistry};

pub(crate) struct NetstatModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    dropdowns: Rc<DropdownRegistry>,
}

#[relm4::component(pub(crate))]
impl Component for NetstatModule {
    type Init = NetstatInit;
    type Input = NetstatMsg;
    type Output = ();
    type CommandOutput = NetstatCmd;

    view! {
        gtk::Box {
            add_css_class: "netstat",
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
        let netstat_config = &config.modules.netstat;

        let networks = init.sysinfo.network.get();
        let interface_config = netstat_config.interface.get();

        let initial_label = helpers::select_interface(&networks, &interface_config)
            .map(|n| helpers::format_label(&netstat_config.format.get(), n))
            .unwrap_or_else(|| String::from("--"));

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: netstat_config.icon_name.get().clone(),
                label: initial_label,
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: netstat_config.icon_color.clone(),
                    label_color: netstat_config.label_color.clone(),
                    icon_background: netstat_config.icon_bg_color.clone(),
                    button_background: netstat_config.button_bg_color.clone(),
                    border_color: netstat_config.border_color.clone(),
                    auto_icon_color: CssToken::Red,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: netstat_config.label_max_length.clone(),
                    show_icon: netstat_config.icon_show.clone(),
                    show_label: netstat_config.label_show.clone(),
                    show_border: netstat_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => NetstatMsg::LeftClick,
                BarButtonOutput::RightClick => NetstatMsg::RightClick,
                BarButtonOutput::MiddleClick => NetstatMsg::MiddleClick,
                BarButtonOutput::ScrollUp => NetstatMsg::ScrollUp,
                BarButtonOutput::ScrollDown => NetstatMsg::ScrollDown,
            });

        // Rates swing across orders of magnitude every second. Reserve the integer
        // part to 3 digits so idle<->active swings up to 999 don't shove neighbors;
        // only rare 4-integer-digit values (1000+, just before the unit switch) grow.
        bar_button.emit(BarButtonInput::SetLabelMinDigits(3));

        watchers::spawn_watchers(&sender, netstat_config, &init.sysinfo);

        let model = Self {
            bar_button,
            config: init.config,
            dropdowns: init.dropdowns,
        };
        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let netstat_config = &self.config.config().modules.netstat;

        let action = match msg {
            NetstatMsg::LeftClick => netstat_config.left_click.get(),
            NetstatMsg::RightClick => netstat_config.right_click.get(),
            NetstatMsg::MiddleClick => netstat_config.middle_click.get(),
            NetstatMsg::ScrollUp => netstat_config.scroll_up.get(),
            NetstatMsg::ScrollDown => netstat_config.scroll_down.get(),
        };

        dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
    }

    fn update_cmd(&mut self, msg: NetstatCmd, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            NetstatCmd::UpdateLabel(label) => {
                self.bar_button.emit(BarButtonInput::SetLabel(label));
            }
            NetstatCmd::UpdateIcon(icon) => {
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
            }
        }
    }
}
