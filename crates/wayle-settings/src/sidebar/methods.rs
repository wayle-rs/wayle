//! Navigation and section-toggle handlers for `Sidebar`.

use relm4::{gtk::prelude::*, prelude::*};

use super::{Sidebar, SidebarOutput};

impl Sidebar {
    pub(super) fn on_navigate(&mut self, id: &'static str, sender: &ComponentSender<Self>) {
        if let Some(prev) = self.nav_buttons.get(self.active_id) {
            prev.remove_css_class("active");
        }

        self.active_id = id;

        if let Some(next) = self.nav_buttons.get(id) {
            next.add_css_class("active");
        }

        let _ = sender.output(SidebarOutput::PageSelected(id));
    }

    pub(super) fn on_toggle_section(&mut self, section_key: &'static str) {
        let Some(items_box) = self.section_items.get(section_key) else {
            return;
        };
        let header = self.section_headers.get(section_key);

        if self.collapsed.contains(section_key) {
            self.collapsed.remove(section_key);
            items_box.set_visible(true);

            if let Some(header) = header {
                header.remove_css_class("collapsed");
            }

            return;
        }

        self.collapsed.insert(section_key);
        items_box.set_visible(false);

        if let Some(header) = header {
            header.add_css_class("collapsed");
        }
    }
}
