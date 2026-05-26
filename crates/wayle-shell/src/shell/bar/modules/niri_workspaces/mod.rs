//! Niri workspace switcher bar module.

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
use wayle_niri::NiriService;
use wayle_widgets::{prelude::BarSettings, utils::force_window_resize};

use self::button::{NiriWorkspaceButton, NiriWorkspaceButtonOutput};
pub(crate) use self::{
    factory::Factory,
    messages::{NiriWorkspacesCmd, NiriWorkspacesInit, NiriWorkspacesMsg},
};
use crate::shell::{bar::dropdowns::DropdownRegistry, helpers::COMPONENT_CSS_PRIORITY};

pub(super) const BLINK_INTERVAL: Duration = Duration::from_millis(500);

pub(crate) struct NiriWorkspaces {
    pub(super) niri: Arc<NiriService>,
    pub(super) config: Arc<ConfigService>,
    pub(super) settings: BarSettings,
    pub(super) dropdowns: Rc<DropdownRegistry>,
    pub(super) css_provider: gtk::CssProvider,
    pub(super) buttons: FactoryVecDeque<NiriWorkspaceButton>,
    pub(super) blink_on: bool,
    pub(super) blink_token: Option<CancellationToken>,
    pub(super) urgent_present: bool,
}

#[relm4::component(pub(crate))]
impl Component for NiriWorkspaces {
    type Init = NiriWorkspacesInit;
    type Input = NiriWorkspacesMsg;
    type Output = ();
    type CommandOutput = NiriWorkspacesCmd;

    view! {
        gtk::Box {
            add_css_class: "workspaces",
            add_css_class: "niri",
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
        let workspaces_config = &config.modules.niri_workspaces;
        let theme_provider = config.styling.theme_provider.clone();
        let bar_scale = config.bar.scale.clone();

        watchers::spawn_watchers(
            &sender,
            workspaces_config,
            init.niri.clone(),
            theme_provider,
            bar_scale,
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
                NiriWorkspaceButtonOutput::LeftClick(id) => NiriWorkspacesMsg::LeftClick(id),
                NiriWorkspaceButtonOutput::MiddleClick(id) => NiriWorkspacesMsg::MiddleClick(id),
                NiriWorkspaceButtonOutput::RightClick(id) => NiriWorkspacesMsg::RightClick(id),
                NiriWorkspaceButtonOutput::ScrollUp => NiriWorkspacesMsg::ScrollUp,
                NiriWorkspaceButtonOutput::ScrollDown => NiriWorkspacesMsg::ScrollDown,
            },
        );

        let mut model = Self {
            niri: init.niri,
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
        let ws_config = &self.config.config().modules.niri_workspaces;

        match msg {
            NiriWorkspacesMsg::LeftClick(id) => {
                self.dispatch_click_action(ws_config.left_click.get(), id);
            }
            NiriWorkspacesMsg::MiddleClick(id) => {
                self.dispatch_click_action(ws_config.middle_click.get(), id);
            }
            NiriWorkspacesMsg::RightClick(id) => {
                self.dispatch_click_action(ws_config.right_click.get(), id);
            }
            NiriWorkspacesMsg::ScrollUp => {
                self.dispatch_scroll_action(ws_config.scroll_up.get());
            }
            NiriWorkspacesMsg::ScrollDown => {
                self.dispatch_scroll_action(ws_config.scroll_down.get());
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: NiriWorkspacesCmd,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            NiriWorkspacesCmd::WorkspacesChanged => {
                self.rebuild_buttons();
                self.sync_blink(&sender);
                force_window_resize(root);
            }
            NiriWorkspacesCmd::ConfigChanged => {
                styling::apply_styling(&self.css_provider, &self.config, &self.settings);
                self.rebuild_buttons();
                self.sync_blink(&sender);
                force_window_resize(root);
            }
            NiriWorkspacesCmd::BlinkTick => {
                self.blink_on = !self.blink_on;
                self.rebuild_buttons();
            }
        }
    }
}

impl Drop for NiriWorkspaces {
    fn drop(&mut self) {
        gtk::style_context_remove_provider_for_display(
            &self.buttons.widget().display(),
            &self.css_provider,
        );
    }
}
