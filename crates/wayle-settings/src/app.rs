//! Top-level settings window. Owns the page stack, sidebar, and CSS provider.

use std::sync::Arc;

use gtk4::{
    CssProvider, STYLE_PROVIDER_PRIORITY_USER, StackTransitionType, gdk::Display, prelude::*,
    style_context_add_provider_for_display,
};
use relm4::prelude::*;
use tracing::warn;
use wayle_config::ConfigService;
use wayle_i18n::t;
use wayle_icons::IconRegistry;
use wayle_styling::{STATIC_CSS, theme_css};

use crate::{
    pages::{bar, general, page::SettingsPage, test_controls},
    sidebar::{NavChild, NavItem, NavSection, Sidebar, SidebarInit, SidebarOutput},
    watchers,
};

#[allow(dead_code)]
pub struct SettingsApp {
    config_service: Arc<ConfigService>,
    css_provider: CssProvider,
    stack: gtk4::Stack,
    pages: Vec<Controller<SettingsPage>>,
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

        if let Err(err) = IconRegistry::new().and_then(|registry| registry.init()) {
            warn!(error = %err, "icon registry init failed");
        }

        let css_provider = load_css(&config_service);

        watchers::spawn_palette_watcher(&sender);
        watchers::spawn_theme_watcher(&sender, &config_service);
        watchers::spawn_scss_dev_watcher(&sender, &config_service);

        let config = config_service.config();
        let general_entry = general::entry(config);
        let bar_entry = bar::entry(config);
        let test_entry = test_controls::entry(config);

        let nav_sections = vec![NavSection {
            i18n_key: "settings-nav-appearance",
            items: vec![
                NavItem {
                    id: general_entry.id,
                    i18n_key: general_entry.i18n_key,
                    icon: general_entry.icon,
                    children: vec![],
                },
                NavItem {
                    id: test_entry.id,
                    i18n_key: test_entry.i18n_key,
                    icon: test_entry.icon,
                    children: vec![],
                },
                NavItem {
                    id: bar_entry.id,
                    i18n_key: bar_entry.i18n_key,
                    icon: bar_entry.icon,
                    children: bar_entry
                        .children
                        .iter()
                        .map(|child| NavChild {
                            id: child.id,
                            i18n_key: child.i18n_key,
                        })
                        .collect(),
                },
            ],
        }];

        let sidebar = Sidebar::builder()
            .launch(SidebarInit {
                sections: nav_sections,
            })
            .forward(sender.input_sender(), |output| match output {
                SidebarOutput::PageSelected(id) => SettingsAppMsg::PageSelected(id),
            });

        let stack = gtk4::Stack::new();
        stack.set_transition_type(StackTransitionType::Crossfade);
        stack.set_hexpand(true);
        stack.set_vexpand(true);

        let mut pages = Vec::new();

        let leaf_page = SettingsPage::builder().launch(general_entry.spec).detach();
        stack.add_named(leaf_page.widget(), Some(general_entry.id));
        pages.push(leaf_page);

        let test_page = SettingsPage::builder().launch(test_entry.spec).detach();
        stack.add_named(test_page.widget(), Some(test_entry.id));
        pages.push(test_page);

        for child in bar_entry.children {
            let child_page = SettingsPage::builder().launch(child.spec).detach();
            stack.add_named(child_page.widget(), Some(child.id));
            pages.push(child_page);
        }

        widgets.layout.prepend(sidebar.widget());
        widgets.content.append(&stack);

        let model = Self {
            config_service,
            css_provider,
            stack: stack.clone(),
            pages,
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
