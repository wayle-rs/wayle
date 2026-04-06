//! Top-level settings window. Owns the page stack, sidebar, and CSS provider.

use std::sync::Arc;

use gtk4::{
    CssProvider, STYLE_PROVIDER_PRIORITY_USER, gdk::Display, prelude::*,
    style_context_add_provider_for_display,
};
use relm4::prelude::*;
use tracing::warn;
use wayle_config::ConfigService;
use wayle_i18n::t;
use wayle_styling::{STATIC_CSS, theme_css};

use crate::{
    pages::{
        bar::{button::BarButtonPage, dropdown::BarDropdownPage, general::BarGeneralPage},
        general::GeneralPage,
    },
    sidebar::{NavChild, NavItem, NavSection, Sidebar, SidebarInit, SidebarOutput},
    watchers,
};

#[allow(dead_code)]
pub struct SettingsApp {
    config_service: Arc<ConfigService>,
    css_provider: CssProvider,
    stack: gtk4::Stack,
    general_page: Controller<GeneralPage>,
    bar_general_page: Controller<BarGeneralPage>,
    bar_button_page: Controller<BarButtonPage>,
    bar_dropdown_page: Controller<BarDropdownPage>,
    sidebar: Controller<Sidebar>,
}

#[derive(Debug)]
pub enum SettingsAppMsg {
    ReloadCss,
    DevCssRecompiled(String),
    PageSelected(&'static str),
}

#[derive(Debug)]
pub enum SettingsAppCmd {
    CssReloadNeeded,
}

#[relm4::component(pub)]
impl Component for SettingsApp {
    type Init = Arc<ConfigService>;
    type Input = SettingsAppMsg;
    type Output = ();
    type CommandOutput = SettingsAppCmd;

    view! {
        gtk4::Window {
            set_title: Some(&t("settings-title")),
            add_css_class: "settings-window",
            set_default_size: (900, 650),

            #[name = "layout"]
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,

                #[name = "content"]
                gtk4::Box {
                    set_hexpand: true,
                    set_orientation: gtk4::Orientation::Vertical,
                },
            },
        }
    }

    fn init(
        config_service: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();

        if let Err(err) = wayle_icons::IconRegistry::new().and_then(|registry| registry.init()) {
            warn!(error = %err, "icon registry init failed");
        }

        let css_provider = load_css(&config_service);

        watchers::spawn_palette_watcher(&sender);
        watchers::spawn_theme_watcher(&sender, &config_service);
        watchers::spawn_scss_dev_watcher(&sender, &config_service);

        let sidebar = Sidebar::builder()
            .launch(SidebarInit {
                sections: build_nav_sections(),
            })
            .forward(sender.input_sender(), |output| match output {
                SidebarOutput::PageSelected(id) => SettingsAppMsg::PageSelected(id),
            });

        let general_page = GeneralPage::builder()
            .launch(Arc::clone(&config_service))
            .detach();

        let bar_general_page = BarGeneralPage::builder()
            .launch(Arc::clone(&config_service))
            .detach();

        let stack = gtk4::Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        stack.set_hexpand(true);
        stack.set_vexpand(true);

        let bar_button_page = BarButtonPage::builder()
            .launch(Arc::clone(&config_service))
            .detach();

        let bar_dropdown_page = BarDropdownPage::builder()
            .launch(Arc::clone(&config_service))
            .detach();

        stack.add_named(general_page.widget(), Some("general"));
        stack.add_named(bar_general_page.widget(), Some("bar-general"));
        stack.add_named(bar_button_page.widget(), Some("bar-button"));
        stack.add_named(bar_dropdown_page.widget(), Some("bar-dropdown"));

        widgets.layout.prepend(sidebar.widget());
        widgets.content.append(&stack);

        let model = Self {
            config_service,
            css_provider,
            stack: stack.clone(),
            general_page,
            bar_general_page,
            bar_button_page,
            bar_dropdown_page,
            sidebar,
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            SettingsAppMsg::ReloadCss => {
                let css = build_css(&self.config_service);
                self.css_provider.load_from_string(&css);
            }

            SettingsAppMsg::DevCssRecompiled(css) => {
                self.css_provider.load_from_string(&css);
            }

            SettingsAppMsg::PageSelected(id) => {
                self.stack.set_visible_child_name(id);
            }
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

fn build_nav_sections() -> Vec<NavSection> {
    vec![NavSection {
        i18n_key: "settings-nav-appearance",
        items: vec![
            NavItem {
                id: "general",
                i18n_key: "settings-nav-general",
                icon: "ld-palette-symbolic",
                children: vec![],
            },
            NavItem {
                id: "bar",
                i18n_key: "settings-nav-bar",
                icon: "ld-layout-dashboard-symbolic",
                children: vec![
                    NavChild {
                        id: "bar-general",
                        i18n_key: "settings-nav-bar-general",
                    },
                    NavChild {
                        id: "bar-button",
                        i18n_key: "settings-nav-bar-button",
                    },
                    NavChild {
                        id: "bar-dropdown",
                        i18n_key: "settings-nav-bar-dropdown",
                    },
                ],
            },
        ],
    }]
}

fn load_css(config_service: &ConfigService) -> CssProvider {
    let Some(display) = Display::default() else {
        warn!("no display available, skipping CSS load");
        return CssProvider::new();
    };

    let provider = CssProvider::new();
    let css = build_css(config_service);

    provider.load_from_string(&css);
    style_context_add_provider_for_display(&display, &provider, STYLE_PROVIDER_PRIORITY_USER);

    provider
}

fn build_css(config_service: &ConfigService) -> String {
    let config = config_service.config();
    let palette = config.styling.palette();
    let theme = theme_css(&palette, &config.general, &config.bar, &config.styling);

    format!("{STATIC_CSS}\n{theme}")
}
