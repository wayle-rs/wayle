use std::{cell::Cell, rc::Rc};

use gtk::prelude::*;
use relm4::gtk;

use super::{ICON_EYE, ICON_EYE_OFF, PasswordForm};

impl PasswordForm {
    pub(super) fn build_password_entry() -> gtk::Entry {
        let entry = gtk::Entry::new();
        entry.set_icon_from_icon_name(gtk::EntryIconPosition::Secondary, Some(ICON_EYE_OFF));
        entry.set_icon_activatable(gtk::EntryIconPosition::Secondary, true);
        entry.set_icon_sensitive(gtk::EntryIconPosition::Secondary, true);

        let revealed = Rc::new(Cell::new(false));
        let toggle_target = entry.clone();
        entry.connect_icon_press(move |_entry, position| {
            if position != gtk::EntryIconPosition::Secondary {
                return;
            }
            let new_state = !revealed.get();
            revealed.set(new_state);
            toggle_target.set_visibility(new_state);
            let icon = if new_state { ICON_EYE } else { ICON_EYE_OFF };
            toggle_target.set_icon_from_icon_name(gtk::EntryIconPosition::Secondary, Some(icon));
        });

        entry
    }

    pub(super) fn reset_entry(&mut self) {
        self.password_entry.set_text("");
        self.password_entry.set_visibility(false);
        self.password_entry
            .set_icon_from_icon_name(gtk::EntryIconPosition::Secondary, Some(ICON_EYE_OFF));
    }
}
