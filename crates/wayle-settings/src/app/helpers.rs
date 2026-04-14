//! Nav/stack construction helpers used by `SettingsApp::init`.

use std::collections::HashMap;

use relm4::{
    gtk::{self, StackTransitionType, prelude::*},
    prelude::*,
};
use tracing::warn;
use wayle_config::Config;

use super::{STACK_TRANSITION_DURATION_MS, SettingsApp, SettingsAppMsg};
use crate::{
    pages::nav::{NavSectionLayout, PageFactory, layout},
    sidebar::{NavItem, NavSection, Sidebar, SidebarInit, SidebarOutput},
};

pub(super) fn build_nav_and_factories(
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
