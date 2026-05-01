//! Top-level settings window. Owns the page stack, sidebar, and CSS provider.

mod css;
mod helpers;
mod methods;
mod reset;
pub(crate) mod sourceview_scheme;
mod watchers;
mod window;

use std::{collections::HashMap, sync::Arc};

use relm4::{
    gtk::{CssProvider, prelude::*},
    prelude::*,
};
use tracing::warn;
use wayle_config::ConfigService;
use wayle_i18n::t;
use wayle_icons::IconRegistry;
use wayle_widgets::primitives::confirm_modal::{
    ConfirmModal, ConfirmModalConfig, ConfirmModalMsg, ConfirmStyle, ModalIcon,
};

use self::{
    css::{build_css, load_css},
    helpers::build_nav_and_factories,
    reset::{build_confirm_modal, perform_reset_all},
    sourceview_scheme::update_wayle_scheme,
    window::{build_content_overlay, setup_paned_clamp},
};
use crate::{
    pages::{nav::PageFactory, page::SettingsPage},
    sidebar::Sidebar,
};

const DEFAULT_SIDEBAR_WIDTH: i32 = 220;
const DEFAULT_WINDOW_WIDTH: i32 = 900;
const DEFAULT_WINDOW_HEIGHT: i32 = 650;
const STACK_TRANSITION_DURATION_MS: u32 = 100;
const INITIAL_PAGE_ID: &str = "bar-general";

pub(crate) struct SettingsApp {
    pub(super) config_service: Arc<ConfigService>,
    pub(super) css_provider: CssProvider,
    pub(super) stack: gtk::Stack,
    pub(super) factories: HashMap<&'static str, PageFactory>,
    pub(super) current_page: Option<(&'static str, Controller<SettingsPage>)>,
    _sidebar: Controller<Sidebar>,
    pub(super) confirm_modal: Controller<ConfirmModal>,
}

#[derive(Debug)]
pub(crate) enum SettingsAppMsg {
    ReloadCss,
    DevCssRecompiled(String),
    PageSelected(&'static str),
    ConfirmResetAll,
    ExecuteResetAll,
    Noop,
}

#[derive(Debug)]
pub(crate) enum SettingsAppCmd {
    CssReloadNeeded,
}

#[relm4::component(pub(crate))]
impl Component for SettingsApp {
    type Init = Arc<ConfigService>;
    type Input = SettingsAppMsg;
    type Output = ();
    type CommandOutput = SettingsAppCmd;

    view! {
        gtk::Window {
            set_title: Some(&t("settings-title")),
            add_css_class: "settings-window",
            set_default_size: (DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT),

            #[name = "paned"]
            gtk::Paned {
                set_orientation: gtk::Orientation::Horizontal,
                set_position: DEFAULT_SIDEBAR_WIDTH,
                set_shrink_start_child: false,
                set_shrink_end_child: false,
                set_wide_handle: true,
            },
        }
    }

    fn init(
        config_service: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();

        if let Err(err) = IconRegistry::new().and_then(|registry| registry.init()) {
            warn!(error = %err, "icon registry init failed");
        }

        let css_provider = load_css(&config_service);

        watchers::spawn_palette_watcher(&sender);
        watchers::spawn_theme_watcher(&sender, &config_service);
        watchers::spawn_scss_dev_watcher(&sender, &config_service);

        let config = config_service.config();
        update_wayle_scheme(&config.styling.palette);

        let (sidebar, stack, factories) = build_nav_and_factories(config, &sender);

        widgets.paned.set_start_child(Some(sidebar.widget()));
        widgets
            .paned
            .set_end_child(Some(&build_content_overlay(&stack, &root)));

        setup_paned_clamp(&widgets.paned, config);
        let confirm_modal = build_confirm_modal(&sender);

        let mut model = Self {
            config_service,
            css_provider,
            stack,
            factories,
            current_page: None,
            _sidebar: sidebar,
            confirm_modal,
        };

        model.show_page(INITIAL_PAGE_ID);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            SettingsAppMsg::ReloadCss => {
                let config = self.config_service.config();
                let css = build_css(&self.config_service);
                self.css_provider.load_from_string(&css);
                update_wayle_scheme(&config.styling.palette);
            }

            SettingsAppMsg::DevCssRecompiled(css) => {
                self.css_provider.load_from_string(&css);
            }

            SettingsAppMsg::PageSelected(id) => {
                self.show_page(id);
            }

            SettingsAppMsg::ConfirmResetAll => {
                self.confirm_modal
                    .emit(ConfirmModalMsg::Show(ConfirmModalConfig {
                        title: t("settings-reset-all-title"),
                        description: Some(t("settings-reset-all-description")),
                        icon: ModalIcon::Warning,
                        confirm_label: t("settings-reset-all-confirm"),
                        confirm_style: ConfirmStyle::Danger,
                        cancel_label: Some(t("settings-reset-all-cancel")),
                    }));
            }

            SettingsAppMsg::ExecuteResetAll => {
                perform_reset_all(&self.config_service);
            }

            SettingsAppMsg::Noop => {}
        }
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            SettingsAppCmd::CssReloadNeeded => {
                sender.input(SettingsAppMsg::ReloadCss);
            }
        }
    }
}
