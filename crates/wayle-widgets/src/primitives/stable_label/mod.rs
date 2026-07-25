//! A label that reserves width for the widest digit in each digit slot, so
//! non-monospace fonts don't shift neighboring bar items as digits change.
//!
//! Digit *width* jitter (same digit count, e.g. `1` vs `8`) is eliminated by
//! measuring the text with every digit replaced by the widest digit. Digit
//! *count* jitter can additionally be reduced via [`StableLabel::set_min_digits`],
//! which reserves width for a minimum number of digit slots.
#![allow(missing_docs)]

mod imp;

use gtk4::{glib, pango, subclass::prelude::*};

glib::wrapper! {
    pub struct StableLabel(ObjectSubclass<imp::StableLabelImp>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for StableLabel {
    fn default() -> Self {
        glib::Object::builder().build()
    }
}

impl StableLabel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the displayed text and re-measures the reserved width.
    pub fn set_text(&self, text: &str) {
        self.imp().set_text(text);
    }

    /// Enables or disables width stabilization. When disabled the widget behaves
    /// like a plain centered label.
    pub fn set_stabilize(&self, on: bool) {
        self.imp().set_stabilize(on);
    }

    /// Reserves width so each number's integer part holds at least `n` digits,
    /// regardless of the value shown. `0` disables the reservation.
    pub fn set_min_digits(&self, n: u32) {
        self.imp().set_min_digits(n);
    }

    /// When enabled, only stabilizes digit groups that are part of a colon-separated
    /// time (a clock's hour/minute/second); standalone numbers such as a date keep
    /// their natural width and get no reserved space.
    pub fn set_time_only(&self, on: bool) {
        self.imp().set_time_only(on);
    }

    /// Forwards to the inner label so `.bar-button-label` styling attaches to the
    /// node that is actually rendered and measured.
    pub fn add_css_class(&self, class: &str) {
        self.imp().add_css_class(class);
    }

    pub fn set_ellipsize(&self, mode: pango::EllipsizeMode) {
        self.imp().set_ellipsize(mode);
    }

    pub fn set_max_width_chars(&self, n: i32) {
        self.imp().set_max_width_chars(n);
    }
}
