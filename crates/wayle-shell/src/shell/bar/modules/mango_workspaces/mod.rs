//! MangoWM tag switcher bar module.

mod button;
mod factory;
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
use wayle_mango::MangoService;
use wayle_widgets::{prelude::BarSettings, utils::force_window_resize};

use self::button::{MangoTagButton, MangoTagButtonOutput};
pub(crate) use self::{
    factory::Factory,
    messages::{MangoWorkspacesCmd, MangoWorkspacesInit, MangoWorkspacesMsg},
};
use crate::shell::{bar::dropdowns::DropdownRegistry, helpers::COMPONENT_CSS_PRIORITY};

pub(super) const BLINK_INTERVAL: Duration = Duration::from_millis(500);

pub(crate) struct MangoWorkspaces {
    pub(super) mango: Arc<MangoService>,
    pub(super) config: Arc<ConfigService>,
    pub(super) settings: BarSettings,
    pub(super) dropdowns: Rc<DropdownRegistry>,
    pub(super) css_provider: gtk::CssProvider,
    pub(super) buttons: FactoryVecDeque<MangoTagButton>,
    pub(super) blink_on: bool,
    pub(super) blink_token: Option<CancellationToken>,
    pub(super) urgent_present: bool,
}

#[relm4::component(pub(crate))]
impl Component for MangoWorkspaces {
    type Init = MangoWorkspacesInit;
    type Input = MangoWorkspacesMsg;
    type Output = ();
    type CommandOutput = MangoWorkspacesCmd;

    view! {
        gtk::Box {
            add_css_class: "workspaces",
            add_css_class: "mango",
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
        let tags_config = &config.modules.mango_workspaces;
        let theme_provider = config.styling.theme_provider.clone();
        let bar_scale = config.bar.scale.clone();

        watchers::spawn_watchers(
            &sender,
            tags_config,
            init.mango.clone(),
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
                MangoTagButtonOutput::LeftClick(index) => MangoWorkspacesMsg::LeftClick(index),
                MangoTagButtonOutput::MiddleClick(index) => MangoWorkspacesMsg::MiddleClick(index),
                MangoTagButtonOutput::RightClick(index) => MangoWorkspacesMsg::RightClick(index),
                MangoTagButtonOutput::ScrollUp => MangoWorkspacesMsg::ScrollUp,
                MangoTagButtonOutput::ScrollDown => MangoWorkspacesMsg::ScrollDown,
            },
        );

        let mut model = Self {
            mango: init.mango,
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
        model.rebuild_tags();
        model.sync_blink(&sender);

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let tags_config = &self.config.config().modules.mango_workspaces;

        match msg {
            MangoWorkspacesMsg::LeftClick(index) => {
                self.dispatch_click_action(tags_config.left_click.get(), index);
            }
            MangoWorkspacesMsg::MiddleClick(index) => {
                self.dispatch_click_action(tags_config.middle_click.get(), index);
            }
            MangoWorkspacesMsg::RightClick(index) => {
                self.dispatch_click_action(tags_config.right_click.get(), index);
            }
            MangoWorkspacesMsg::ScrollUp => {
                self.dispatch_scroll_action(tags_config.scroll_up.get());
            }
            MangoWorkspacesMsg::ScrollDown => {
                self.dispatch_scroll_action(tags_config.scroll_down.get());
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: MangoWorkspacesCmd,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            MangoWorkspacesCmd::TagsChanged => {
                self.rebuild_tags();
                self.sync_blink(&sender);
                force_window_resize(root);
            }
            MangoWorkspacesCmd::ConfigChanged => {
                styling::apply_styling(&self.css_provider, &self.config, &self.settings);
                self.rebuild_tags();
                self.sync_blink(&sender);
                force_window_resize(root);
            }
            MangoWorkspacesCmd::BlinkTick => {
                self.blink_on = !self.blink_on;
                self.rebuild_tags();
            }
        }
    }
}

impl Drop for MangoWorkspaces {
    fn drop(&mut self) {
        gtk::style_context_remove_provider_for_display(
            &self.buttons.widget().display(),
            &self.css_provider,
        );
    }
}
