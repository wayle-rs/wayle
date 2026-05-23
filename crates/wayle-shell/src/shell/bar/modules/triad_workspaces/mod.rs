//! Triad workspace switcher bar module.

mod button;
mod factory;
mod filtering;
mod helpers;
mod messages;
mod methods;
mod styling;
mod watchers;

use std::{rc::Rc, sync::Arc, time::Duration};

use gtk::prelude::*;
use relm4::{factory::FactoryVecDeque, prelude::*};
use tokio_util::sync::CancellationToken;
use wayle_config::ConfigService;
use wayle_triad::TriadService;
use wayle_widgets::{prelude::BarSettings, utils::force_window_resize};

use self::button::{TriadWorkspaceButton, TriadWorkspaceButtonOutput};
pub(crate) use self::{
    factory::Factory,
    messages::{TriadWorkspacesCmd, TriadWorkspacesInit, TriadWorkspacesMsg},
};
use crate::shell::bar::dropdowns::DropdownRegistry;

pub(super) const BLINK_INTERVAL: Duration = Duration::from_millis(500);

pub(crate) struct TriadWorkspaces {
    pub(super) triad: Arc<TriadService>,
    pub(super) config: Arc<ConfigService>,
    pub(super) settings: BarSettings,
    pub(super) dropdowns: Rc<DropdownRegistry>,
    pub(super) css_provider: gtk::CssProvider,
    pub(super) buttons: FactoryVecDeque<TriadWorkspaceButton>,
    pub(super) blink_on: bool,
    pub(super) blink_token: Option<CancellationToken>,
    pub(super) urgent_present: bool,
}

#[relm4::component(pub(crate))]
impl Component for TriadWorkspaces {
    type Init = TriadWorkspacesInit;
    type Input = TriadWorkspacesMsg;
    type Output = ();
    type CommandOutput = TriadWorkspacesCmd;

    view! {
        gtk::Box {
            add_css_class: "workspaces",
            add_css_class: "triad",
            #[watch]
            set_orientation: model.orientation(),
            #[watch]
            set_hexpand: model.is_vertical(),
            #[watch]
            set_vexpand: !model.is_vertical(),
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = init.config.config();
        let workspaces_config = &config.modules.triad_workspaces;
        let theme_provider = config.styling.theme_provider.clone();
        let bar_scale = config.bar.scale.clone();

        watchers::spawn_watchers(
            &sender,
            workspaces_config,
            init.triad.clone(),
            theme_provider,
            bar_scale,
            &init.settings,
        );

        let css_provider = gtk::CssProvider::new();
        gtk::style_context_add_provider_for_display(
            &root.display(),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER + 1,
        );

        let buttons = FactoryVecDeque::builder().launch(root.clone()).forward(
            sender.input_sender(),
            |output| match output {
                TriadWorkspaceButtonOutput::LeftClick(id) => TriadWorkspacesMsg::LeftClick(id),
                TriadWorkspaceButtonOutput::MiddleClick(id) => TriadWorkspacesMsg::MiddleClick(id),
                TriadWorkspaceButtonOutput::RightClick(id) => TriadWorkspacesMsg::RightClick(id),
                TriadWorkspaceButtonOutput::ScrollUp => TriadWorkspacesMsg::ScrollUp,
                TriadWorkspaceButtonOutput::ScrollDown => TriadWorkspacesMsg::ScrollDown,
            },
        );

        let mut model = Self {
            triad: init.triad,
            config: init.config,
            settings: init.settings,
            dropdowns: init.dropdowns,
            css_provider,
            buttons,
            blink_on: false,
            blink_token: None,
            urgent_present: false,
        };
        styling::apply_styling(&model.css_provider, &model.config, &model.settings);
        model.rebuild_buttons();
        model.sync_blink(&sender);

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let ws_config = &self.config.config().modules.triad_workspaces;

        match msg {
            TriadWorkspacesMsg::LeftClick(id) => {
                self.dispatch_click_action(ws_config.left_click.get(), id);
            }
            TriadWorkspacesMsg::MiddleClick(id) => {
                self.dispatch_click_action(ws_config.middle_click.get(), id);
            }
            TriadWorkspacesMsg::RightClick(id) => {
                self.dispatch_click_action(ws_config.right_click.get(), id);
            }
            TriadWorkspacesMsg::ScrollUp => {
                self.dispatch_scroll_action(ws_config.scroll_up.get());
            }
            TriadWorkspacesMsg::ScrollDown => {
                self.dispatch_scroll_action(ws_config.scroll_down.get());
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: TriadWorkspacesCmd,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            TriadWorkspacesCmd::WorkspacesChanged => {
                self.rebuild_buttons();
                self.sync_blink(&sender);
                force_window_resize(root);
            }
            TriadWorkspacesCmd::ConfigChanged => {
                styling::apply_styling(&self.css_provider, &self.config, &self.settings);
                self.rebuild_buttons();
                self.sync_blink(&sender);
                force_window_resize(root);
            }
            TriadWorkspacesCmd::BlinkTick => {
                self.blink_on = !self.blink_on;
                self.rebuild_buttons();
            }
        }
    }
}

impl Drop for TriadWorkspaces {
    fn drop(&mut self) {
        gtk::style_context_remove_provider_for_display(
            &self.buttons.widget().display(),
            &self.css_provider,
        );
    }
}
