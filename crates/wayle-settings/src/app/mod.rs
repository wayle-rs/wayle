//! Top-level settings window. Owns the page stack, sidebar, and CSS provider.

mod css;
mod methods;
mod reset;
pub(crate) mod sourceview_scheme;
mod watchers;
mod window;

use std::{collections::HashMap, sync::Arc};

use relm4::{
    gtk::{CssProvider, StackTransitionType, prelude::*},
    prelude::*,
};
use tracing::warn;
use wayle_config::{Config, ConfigService};
use wayle_i18n::t;
use wayle_icons::IconRegistry;
use wayle_widgets::primitives::confirm_modal::{
    ConfirmModal, ConfirmModalConfig, ConfirmModalMsg, ConfirmStyle, ModalIcon,
};

use self::{
    css::{build_css, load_css},
    reset::{build_confirm_modal, perform_reset_all},
    sourceview_scheme::update_wayle_scheme,
    window::{build_content_overlay, setup_paned_clamp},
};
use crate::{
    pages::{
        nav::{NavSectionLayout, PageFactory, layout},
        page::SettingsPage,
    },
    sidebar::{NavItem, NavSection, Sidebar, SidebarInit, SidebarOutput},
};

const DEFAULT_SIDEBAR_WIDTH: i32 = 220;
const DEFAULT_WINDOW_WIDTH: i32 = 900;
const DEFAULT_WINDOW_HEIGHT: i32 = 650;
const STACK_TRANSITION_DURATION_MS: u32 = 100;
const INITIAL_PAGE_ID: &str = "general";

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

fn build_nav_and_factories(
    config: &Config,
    sender: &ComponentSender<SettingsApp>,
) -> (
    Controller<Sidebar>,
    gtk::Stack,
    HashMap<&'static str, PageFactory>,
) {
    let mut factories = HashMap::new();
    let mut nav_sections = Vec::new();

    for section in layout() {
        nav_sections.push(build_nav_section(config, section, &mut factories));
    }

    let sidebar = Sidebar::builder()
        .launch(SidebarInit {
            sections: nav_sections,
        })
        .forward(sender.input_sender(), |output| match output {
            SidebarOutput::PageSelected(id) => SettingsAppMsg::PageSelected(id),
            SidebarOutput::ResetAllRequested => SettingsAppMsg::ConfirmResetAll,
        });

    let stack = build_stack();

    (sidebar, stack, factories)
}

fn build_nav_section(
    config: &Config,
    section: NavSectionLayout,
    factories: &mut HashMap<&'static str, PageFactory>,
) -> NavSection {
    let mut items = Vec::with_capacity(section.factories.len());

    for factory in section.factories {
        let entry = factory(config);
        if factories.insert(entry.id, factory).is_some() {
            warn!(
                page_id = entry.id,
                "duplicate page id, replacing previous factory"
            );
        }

        items.push(NavItem {
            id: entry.id,
            i18n_key: entry.i18n_key,
            icon: entry.icon,
        });
    }

    NavSection {
        i18n_key: section.i18n_key,
        items,
    }
}

fn build_stack() -> gtk::Stack {
    let stack = gtk::Stack::new();
    stack.set_transition_type(StackTransitionType::Crossfade);
    stack.set_transition_duration(STACK_TRANSITION_DURATION_MS);
    stack.set_hexpand(true);
    stack.set_vexpand(true);
    stack
}
