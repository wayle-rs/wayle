//! Debounced slider with full frame-rate updates and commit-on-idle signal.
#![allow(missing_docs)]

mod imp;

use glib::subclass::types::ObjectSubclassIsExt;
use gtk4::glib;

glib::wrapper! {
    /// Slider combining a Scale and optional label that updates at full GTK frame rate.
    ///
    /// Emits `committed` signal after 100ms of inactivity for backend writes.
    /// Programmatic `set_value()` calls are ignored while the user is dragging
    /// or within a grace period after drag ends.
    pub struct DebouncedSlider(ObjectSubclass<imp::DebouncedSliderImp>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl DebouncedSlider {
    pub fn new(initial_value: f64) -> Self {
        glib::Object::builder()
            .property("value", initial_value)
            .build()
    }

    pub fn with_label(initial_value: f64) -> Self {
        glib::Object::builder()
            .property("value", initial_value)
            .property("show-label", true)
            .build()
    }

    /// Updates the slider's minimum and maximum bounds.
    pub fn set_range(&self, min: f64, max: f64) {
        self.imp().set_range(min, max);
    }

    /// Sets a custom label formatter. Defaults to `"{value:.0}%"`.
    pub fn set_formatter(&self, formatter: impl Fn(f64) -> String + 'static) {
        if let Ok(mut guard) = self.imp().formatter.try_borrow_mut() {
            *guard = Some(Box::new(formatter));
        }
    }

    /// The internal Scale widget, for adding CSS classes.
    pub fn scale(&self) -> Option<gtk4::Scale> {
        self.imp().scale()
    }

    /// The internal Label widget, for adding CSS classes.
    pub fn label_widget(&self) -> Option<gtk4::Label> {
        self.imp().label_widget()
    }
}
