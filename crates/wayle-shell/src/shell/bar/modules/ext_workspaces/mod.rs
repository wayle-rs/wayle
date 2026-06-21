//! Generic Wayland workspace switcher bar module, backed by
//! `ext-workspace-v1` rather than a compositor-specific IPC. Works on any
//! compositor advertising `ext_workspace_manager_v1` (e.g. Sway).

mod button;
mod factory;
mod filtering;
mod helpers;
mod messages;
mod methods;
mod styling;
mod watchers;

use std::{rc::Rc, sync::Arc};

use gtk::prelude::*;
use relm4::{factory::FactoryVecDeque, prelude::*};
use wayle_config::ConfigService;
use wayle_ext_workspace::ExtWorkspaceService;
use wayle_widgets::{prelude::BarSettings, utils::force_window_resize};

use self::button::{ExtWorkspaceButton, ExtWorkspaceButtonOutput};
pub(crate) use self::{
    factory::Factory,
    messages::{ExtWorkspacesCmd, ExtWorkspacesInit, ExtWorkspacesMsg},
};
use crate::shell::{bar::dropdowns::DropdownRegistry, helpers::COMPONENT_CSS_PRIORITY};

pub(crate) struct ExtWorkspaces {
    pub(super) service: Arc<ExtWorkspaceService>,
    pub(super) config: Arc<ConfigService>,
    pub(super) settings: BarSettings,
    pub(super) dropdowns: Rc<DropdownRegistry>,
    pub(super) css_provider: gtk::CssProvider,
    pub(super) buttons: FactoryVecDeque<ExtWorkspaceButton>,
}

#[relm4::component(pub(crate))]
impl Component for ExtWorkspaces {
    type Init = ExtWorkspacesInit;
    type Input = ExtWorkspacesMsg;
    type Output = ();
    type CommandOutput = ExtWorkspacesCmd;

    view! {
        gtk::Box {
            add_css_class: "workspaces",
            add_css_class: "ext",
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
        let workspaces_config = &config.modules.ext_workspaces;
        let theme_provider = config.styling.theme_provider.clone();

        watchers::spawn_watchers(
            &sender,
            workspaces_config,
            &init.service,
            theme_provider,
            &init.settings,
        );

        let css_provider = gtk::CssProvider::new();
        gtk::style_context_add_provider_for_display(
            &root.display(),
            &css_provider,
            COMPONENT_CSS_PRIORITY,
        );

        let buttons = FactoryVecDeque::builder().launch(root.clone()).forward(
            sender.input_sender(),
            |output| match output {
                ExtWorkspaceButtonOutput::LeftClick(key) => ExtWorkspacesMsg::LeftClick(key),
                ExtWorkspaceButtonOutput::MiddleClick(key) => ExtWorkspacesMsg::MiddleClick(key),
                ExtWorkspaceButtonOutput::RightClick(key) => ExtWorkspacesMsg::RightClick(key),
                ExtWorkspaceButtonOutput::ScrollUp => ExtWorkspacesMsg::ScrollUp,
                ExtWorkspaceButtonOutput::ScrollDown => ExtWorkspacesMsg::ScrollDown,
            },
        );

        let mut model = Self {
            service: init.service,
            config: init.config,
            settings: init.settings,
            dropdowns: init.dropdowns,
            css_provider,
            buttons,
        };
        styling::apply_styling(&model.css_provider, &model.config, &model.settings);
        model.rebuild_buttons();

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let ws_config = &self.config.config().modules.ext_workspaces;

        match msg {
            ExtWorkspacesMsg::LeftClick(key) => {
                self.dispatch_click_action(ws_config.left_click.get(), key);
            }
            ExtWorkspacesMsg::MiddleClick(key) => {
                self.dispatch_click_action(ws_config.middle_click.get(), key);
            }
            ExtWorkspacesMsg::RightClick(key) => {
                self.dispatch_click_action(ws_config.right_click.get(), key);
            }
            ExtWorkspacesMsg::ScrollUp => {
                self.dispatch_scroll_action(ws_config.scroll_up.get());
            }
            ExtWorkspacesMsg::ScrollDown => {
                self.dispatch_scroll_action(ws_config.scroll_down.get());
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: ExtWorkspacesCmd,
        _sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            ExtWorkspacesCmd::WorkspacesChanged => {
                self.rebuild_buttons();
                force_window_resize(root);
            }
            ExtWorkspacesCmd::ConfigChanged => {
                styling::apply_styling(&self.css_provider, &self.config, &self.settings);
                self.rebuild_buttons();
                force_window_resize(root);
            }
        }
    }
}

impl Drop for ExtWorkspaces {
    fn drop(&mut self) {
        gtk::style_context_remove_provider_for_display(
            &self.buttons.widget().display(),
            &self.css_provider,
        );
    }
}
