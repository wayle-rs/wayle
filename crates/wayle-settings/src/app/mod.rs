//! Top-level settings window. Owns the page stack, sidebar, and CSS provider.

mod css;
mod watchers;

use std::sync::Arc;

use gtk4::{CssProvider, StackTransitionType, prelude::*};
use relm4::prelude::*;
use tracing::{info, warn};
use wayle_config::{Config, ConfigService};
use wayle_i18n::t;
use wayle_icons::IconRegistry;
use wayle_widgets::primitives::confirm_modal::{
    ConfirmModal, ConfirmModalConfig, ConfirmModalMsg, ConfirmModalOutput, ConfirmStyle, ModalIcon,
};

use self::css::{build_css, load_css};
use crate::{
    editors::toml_editor::update_wayle_scheme,
    pages::{
        bar, general, modules, nav::LeafEntry, notifications, osd, page::SettingsPage, styling,
        wallpaper,
    },
    sidebar::{NavItem, NavSection, Sidebar, SidebarInit, SidebarOutput},
};

const DEFAULT_SIDEBAR_WIDTH: i32 = 220;
const MAX_SIDEBAR_REM: f64 = 25.0;
const BASE_PX_PER_REM: f64 = 16.0;

#[allow(dead_code)]
pub struct SettingsApp {
    config_service: Arc<ConfigService>,
    css_provider: CssProvider,
    stack: gtk4::Stack,
    pages: Vec<Controller<SettingsPage>>,
    sidebar: Controller<Sidebar>,
    confirm_modal: Controller<ConfirmModal>,
}

#[derive(Debug)]
pub enum SettingsAppMsg {
    ReloadCss,
    DevCssRecompiled(String),
    PageSelected(&'static str),
    ConfirmResetAll,
    ExecuteResetAll,
    #[allow(dead_code)]
    Noop,
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

            #[name = "paned"]
            gtk4::Paned {
                set_orientation: gtk4::Orientation::Horizontal,
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
        let (sidebar, stack, pages) = build_pages(config, &sender);

        widgets.paned.set_start_child(Some(sidebar.widget()));
        widgets
            .paned
            .set_end_child(Some(&build_content_overlay(&stack, &root)));

        setup_paned_clamp(&widgets.paned, config);
        let confirm_modal = build_confirm_modal(&sender);

        let model = Self {
            config_service,
            css_provider,
            stack,
            pages,
            sidebar,
            confirm_modal,
        };

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
                self.stack.set_visible_child_name(id);
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

fn build_pages(
    config: &Config,
    sender: &ComponentSender<SettingsApp>,
) -> (
    Controller<Sidebar>,
    gtk4::Stack,
    Vec<Controller<SettingsPage>>,
) {
    let theme_entry = styling::entry(config);
    let wallpaper_entry = wallpaper::entry(config);
    let bar_entries = bar::entries(config);
    let notifications_entry = notifications::entry(config);
    let osd_entry = osd::entry(config);
    let general_entry = general::entry(config);

    let module_entries = modules::all_entries(config);

    let nav_sections = vec![
        NavSection {
            i18n_key: "settings-nav-system",
            items: vec![leaf_nav(&general_entry)],
        },
        NavSection {
            i18n_key: "settings-nav-appearance",
            items: vec![leaf_nav(&theme_entry), leaf_nav(&wallpaper_entry)],
        },
        NavSection {
            i18n_key: "settings-nav-bar-section",
            items: bar_entries.iter().map(leaf_nav).collect(),
        },
        NavSection {
            i18n_key: "settings-nav-overlays",
            items: vec![leaf_nav(&notifications_entry), leaf_nav(&osd_entry)],
        },
        NavSection {
            i18n_key: "settings-nav-modules",
            items: module_entries.iter().map(leaf_nav).collect(),
        },
    ];

    let sidebar = Sidebar::builder()
        .launch(SidebarInit {
            sections: nav_sections,
        })
        .forward(sender.input_sender(), |output| match output {
            SidebarOutput::PageSelected(id) => SettingsAppMsg::PageSelected(id),
            SidebarOutput::ResetAllRequested => SettingsAppMsg::ConfirmResetAll,
        });

    let stack = gtk4::Stack::new();
    stack.set_transition_type(StackTransitionType::Crossfade);
    stack.set_hexpand(true);
    stack.set_vexpand(true);

    let mut pages = Vec::new();

    for entry in [
        general_entry,
        theme_entry,
        wallpaper_entry,
        notifications_entry,
        osd_entry,
    ] {
        let page = SettingsPage::builder().launch(entry.spec).detach();
        stack.add_named(page.widget(), Some(entry.id));
        pages.push(page);
    }

    for entry in bar_entries {
        let page = SettingsPage::builder().launch(entry.spec).detach();
        stack.add_named(page.widget(), Some(entry.id));
        pages.push(page);
    }

    for entry in module_entries {
        let page = SettingsPage::builder().launch(entry.spec).detach();
        stack.add_named(page.widget(), Some(entry.id));
        pages.push(page);
    }

    (sidebar, stack, pages)
}

fn leaf_nav(entry: &LeafEntry) -> NavItem {
    NavItem {
        id: entry.id,
        i18n_key: entry.i18n_key,
        icon: entry.icon,
    }
}

fn build_content_overlay(stack: &gtk4::Stack, window: &gtk4::Window) -> gtk4::Overlay {
    let close_button = gtk4::Button::from_icon_name("ld-x-symbolic");
    close_button.add_css_class("settings-close");
    close_button.set_cursor_from_name(Some("pointer"));
    close_button.set_valign(gtk4::Align::Start);
    close_button.set_halign(gtk4::Align::End);
    close_button.set_tooltip_text(Some(&t("settings-close")));

    let window_ref = window.clone();
    close_button.connect_clicked(move |_| window_ref.close());

    let overlay = gtk4::Overlay::new();
    overlay.set_child(Some(stack));
    overlay.add_overlay(&close_button);

    overlay
}

fn setup_paned_clamp(paned: &gtk4::Paned, config: &Config) {
    let scale_property = config.styling.scale.clone();

    paned.connect_position_notify(move |paned| {
        let scale = scale_property.get().value() as f64;
        let max_width = (MAX_SIDEBAR_REM * BASE_PX_PER_REM * scale).round() as i32;

        if paned.position() > max_width {
            paned.set_position(max_width);
        }
    });
}

fn build_confirm_modal(sender: &ComponentSender<SettingsApp>) -> Controller<ConfirmModal> {
    ConfirmModal::builder()
        .launch(())
        .forward(sender.input_sender(), |output| match output {
            ConfirmModalOutput::Confirmed => SettingsAppMsg::ExecuteResetAll,
            ConfirmModalOutput::Cancelled => SettingsAppMsg::Noop,
        })
}

fn perform_reset_all(config_service: &ConfigService) {
    match config_service.reset_all_runtime() {
        Ok(()) => info!("all runtime overrides cleared"),
        Err(err) => warn!(error = %err, "reset-all partially failed"),
    }
}
