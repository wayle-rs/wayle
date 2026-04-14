use std::{
    cell::{Cell, OnceCell, RefCell},
    sync::OnceLock,
    time::{Duration, Instant},
};

use glib::{Properties, subclass::Signal};
use gtk4::{gdk, glib, prelude::*, subclass::prelude::*};

const THROTTLE_INTERVAL: Duration = Duration::from_millis(100);
const GRACE_PERIOD: Duration = Duration::from_millis(150);

type LabelFormatter = Box<dyn Fn(f64) -> String>;

#[derive(Properties)]
#[properties(wrapper_type = super::DebouncedSlider)]
pub struct DebouncedSliderImp {
    #[property(get, set = Self::set_value_external, explicit_notify)]
    value: Cell<f64>,

    #[property(get, set)]
    range_min: Cell<f64>,

    #[property(get, set)]
    range_max: Cell<f64>,

    #[property(get, set = Self::set_show_label)]
    show_label: Cell<bool>,

    scale: OnceCell<gtk4::Scale>,
    label: OnceCell<gtk4::Label>,

    dragging: Cell<bool>,
    drag_ended_at: Cell<Option<Instant>>,
    last_committed_at: Cell<Option<Instant>>,
    trailing_source: Cell<Option<glib::SourceId>>,
    setting_programmatically: Cell<bool>,

    pub(super) formatter: RefCell<Option<LabelFormatter>>,
}

impl Default for DebouncedSliderImp {
    fn default() -> Self {
        Self {
            value: Cell::new(0.0),
            range_min: Cell::new(0.0),
            range_max: Cell::new(100.0),
            show_label: Cell::new(false),
            scale: OnceCell::new(),
            label: OnceCell::new(),
            dragging: Cell::new(false),
            drag_ended_at: Cell::new(None),
            last_committed_at: Cell::new(None),
            trailing_source: Cell::new(None),
            setting_programmatically: Cell::new(false),
            formatter: RefCell::new(None),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for DebouncedSliderImp {
    const NAME: &'static str = "WayleDebouncedSlider";
    type Type = super::DebouncedSlider;
    type ParentType = gtk4::Box;
}

#[glib::derived_properties]
impl ObjectImpl for DebouncedSliderImp {
    fn signals() -> &'static [Signal] {
        static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
        SIGNALS.get_or_init(|| {
            vec![
                Signal::builder("committed")
                    .param_types([f64::static_type()])
                    .build(),
            ]
        })
    }

    fn constructed(&self) {
        self.parent_constructed();
        self.build_widgets();
        self.connect_signals();
    }

    fn dispose(&self) {
        self.cancel_trailing();
    }
}

impl WidgetImpl for DebouncedSliderImp {}
impl BoxImpl for DebouncedSliderImp {}

impl DebouncedSliderImp {
    fn set_show_label(&self, visible: bool) {
        self.show_label.set(visible);
        if let Some(label) = self.label.get() {
            label.set_visible(visible);
        }
    }

    fn set_value_external(&self, value: f64) {
        if self.user_is_interacting() {
            return;
        }
        self.apply_value(value);
    }

    fn apply_value(&self, value: f64) {
        self.setting_programmatically.set(true);
        self.value.set(value);

        if let Some(scale) = self.scale.get() {
            scale.set_value(value);
        }
        self.update_label(value);

        self.setting_programmatically.set(false);
    }

    fn user_is_interacting(&self) -> bool {
        if self.dragging.get() {
            return true;
        }
        self.drag_ended_at
            .get()
            .is_some_and(|t| t.elapsed() < GRACE_PERIOD)
    }

    fn format_value(&self, value: f64) -> String {
        if let Ok(guard) = self.formatter.try_borrow()
            && let Some(ref fmt) = *guard
        {
            return fmt(value);
        }
        format!("{:.0}%", value)
    }

    fn update_label(&self, value: f64) {
        if let Some(label) = self.label.get() {
            label.set_label(&self.format_value(value));
        }
    }

    fn build_widgets(&self) {
        let min = self.range_min.get();
        let max = self.range_max.get();
        let val = self.value.get();

        let scale = gtk4::Scale::builder()
            .draw_value(false)
            .has_origin(true)
            .hexpand(true)
            .build();
        scale.set_cursor_from_name(Some("pointer"));
        scale.set_range(min, max);
        scale.set_value(val);

        let label = gtk4::Label::builder()
            .label(self.format_value(val))
            .width_chars(6)
            .xalign(1.0)
            .visible(self.show_label.get())
            .build();

        self.obj().append(&scale);
        self.obj().append(&label);

        let _ = self.scale.set(scale);
        let _ = self.label.set(label);
    }

    fn connect_signals(&self) {
        let Some(scale) = self.scale.get() else {
            return;
        };

        let weak = self.obj().downgrade();
        scale.connect_value_changed(move |scale| {
            let Some(obj) = weak.upgrade() else {
                return;
            };
            let imp = obj.imp();

            if imp.setting_programmatically.get() {
                return;
            }

            let value = scale.value();
            imp.value.set(value);
            imp.update_label(value);
            imp.throttle_commit(value);
        });

        let weak = self.obj().downgrade();
        let drag_controller = gtk4::EventControllerLegacy::new();
        drag_controller.connect_event(move |_, event| {
            let Some(obj) = weak.upgrade() else {
                return glib::Propagation::Proceed;
            };
            let imp = obj.imp();

            match event.event_type() {
                gdk::EventType::ButtonPress | gdk::EventType::TouchBegin => {
                    imp.dragging.set(true);
                    imp.drag_ended_at.set(None);
                }
                gdk::EventType::ButtonRelease
                | gdk::EventType::TouchEnd
                | gdk::EventType::TouchCancel => {
                    imp.dragging.set(false);
                    imp.drag_ended_at.set(Some(Instant::now()));
                    imp.cancel_trailing();
                    imp.commit(imp.value.get());
                }
                _ => {}
            }

            glib::Propagation::Proceed
        });
        scale.add_controller(drag_controller);
    }

    fn throttle_commit(&self, value: f64) {
        let now = Instant::now();
        let can_emit = self
            .last_committed_at
            .get()
            .is_none_or(|t| now.duration_since(t) >= THROTTLE_INTERVAL);

        if can_emit {
            self.cancel_trailing();
            self.commit(value);
        } else {
            self.schedule_trailing(value);
        }
    }

    fn commit(&self, value: f64) {
        self.last_committed_at.set(Some(Instant::now()));
        self.obj().emit_by_name::<()>("committed", &[&value]);
    }

    fn schedule_trailing(&self, value: f64) {
        self.cancel_trailing();

        let weak = self.obj().downgrade();
        let source_id = glib::timeout_add_local_once(THROTTLE_INTERVAL, move || {
            let Some(obj) = weak.upgrade() else { return };
            obj.imp().trailing_source.set(None);
            obj.imp().commit(value);
        });

        self.trailing_source.set(Some(source_id));
    }

    fn cancel_trailing(&self) {
        if let Some(source_id) = self.trailing_source.take() {
            source_id.remove();
        }
    }

    pub(super) fn set_range(&self, min: f64, max: f64) {
        self.range_min.set(min);
        self.range_max.set(max);

        if let Some(scale) = self.scale.get() {
            scale.set_range(min, max);
        }
    }

    pub(super) fn scale(&self) -> Option<gtk4::Scale> {
        self.scale.get().cloned()
    }

    pub(super) fn label_widget(&self) -> Option<gtk4::Label> {
        self.label.get().cloned()
    }
}
