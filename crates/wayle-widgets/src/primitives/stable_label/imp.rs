use std::cell::{Cell, OnceCell};

use gtk4::{glib, pango, prelude::*, subclass::prelude::*};

#[derive(Default)]
pub struct StableLabelImp {
    label: OnceCell<gtk4::Label>,
    /// When false, behaves like a plain centered label.
    stabilize: Cell<bool>,
    /// Minimum digits to reserve for each number's integer part, regardless of
    /// the value shown. `0` disables the reservation.
    min_digits: Cell<u32>,
    /// When true, only reserve/normalize digit groups that are part of a
    /// colon-separated time (e.g. the hour and minute in `9:30`); standalone
    /// numbers such as a date stay at their natural width. Used by the clocks so
    /// date parts (which change at most once a day) get no reserved space.
    time_only: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for StableLabelImp {
    const NAME: &'static str = "WayleStableLabel";
    type Type = super::StableLabel;
    type ParentType = gtk4::Widget;
    // NOTE: deliberately no layout manager. `gtk_widget_measure()` delegates
    // straight to a layout manager when one is set and never calls our `measure`
    // vfunc, so a manager here would silently disable stabilization. We size the
    // single child by hand instead.
}

impl ObjectImpl for StableLabelImp {
    fn constructed(&self) {
        self.parent_constructed();
        self.stabilize.set(true);

        // Fill the (possibly widened) allocation; text position is controlled by
        // xalign/yalign so it centers by default and right-anchors when reserving.
        let label = gtk4::Label::builder()
            .halign(gtk4::Align::Fill)
            .valign(gtk4::Align::Fill)
            .justify(gtk4::Justification::Center)
            .xalign(0.5)
            .build();
        label.set_parent(&*self.obj());
        let _ = self.label.set(label);
    }

    fn dispose(&self) {
        if let Some(label) = self.label.get() {
            label.unparent();
        }
    }
}

impl WidgetImpl for StableLabelImp {
    fn request_mode(&self) -> gtk4::SizeRequestMode {
        self.label
            .get()
            .map_or(gtk4::SizeRequestMode::ConstantSize, |l| l.request_mode())
    }

    fn measure(&self, orientation: gtk4::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
        let Some(label) = self.label.get() else {
            return (0, 0, -1, -1);
        };

        let (mut min, mut nat, min_baseline, nat_baseline) = label.measure(orientation, for_size);

        // Recomputed on every measure so it stays correct across runtime
        // font/theme/scale changes (those queue a resize, re-invoking measure()).
        if orientation == gtk4::Orientation::Horizontal && self.stabilize.get() {
            let extra = stabilized_extra_px(label, self.min_digits.get(), self.time_only.get());
            min += extra;
            nat += extra;
        }

        (min, nat, min_baseline, nat_baseline)
    }

    fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
        if let Some(label) = self.label.get() {
            label.allocate(width, height, baseline, None);
        }
    }
}

impl StableLabelImp {
    pub(super) fn set_text(&self, text: &str) {
        if let Some(label) = self.label.get()
            && label.text() != text
        {
            // set_label already queues a resize; measure() recomputes the width.
            label.set_label(text);
        }
    }

    pub(super) fn set_stabilize(&self, on: bool) {
        if self.stabilize.get() != on {
            self.stabilize.set(on);
            self.obj().queue_resize();
        }
    }

    pub(super) fn set_min_digits(&self, n: u32) {
        if self.min_digits.get() != n {
            self.min_digits.set(n);
            self.obj().queue_resize();
        }
    }

    pub(super) fn set_time_only(&self, on: bool) {
        if self.time_only.get() != on {
            self.time_only.set(on);
            self.obj().queue_resize();
        }
    }

    pub(super) fn set_ellipsize(&self, mode: pango::EllipsizeMode) {
        if let Some(label) = self.label.get() {
            label.set_ellipsize(mode);
        }
    }

    pub(super) fn set_max_width_chars(&self, n: i32) {
        if let Some(label) = self.label.get() {
            label.set_max_width_chars(n);
        }
    }

    pub(super) fn add_css_class(&self, class: &str) {
        if let Some(label) = self.label.get() {
            label.add_css_class(class);
        }
    }
}

/// Extra horizontal pixels to add so every ASCII-digit slot occupies the width of
/// the widest digit in the resolved font, plus per-number reservation so each
/// integer part holds at least `min_digits` digits. Returned as a *delta* from the
/// current text so it's independent of the label's margins/padding and GtkLabel's
/// own width rounding. `0` when there is nothing to stabilize.
fn stabilized_extra_px(label: &gtk4::Label, min_digits: u32, time_only: bool) -> i32 {
    let text = label.text();
    let text = text.as_str();

    // Only stabilize labels that actually contain a digit; a digit-free label
    // (icon text, "--", a keyboard layout) should never reserve digit slots.
    if text.is_empty() || !text.bytes().any(|b| b.is_ascii_digit()) {
        return 0;
    }

    // Copy the label's own render layout so measurements use the exact
    // CSS-resolved font/attributes it draws with.
    let layout = label.layout().copy();

    let mut widest = b'0';
    let mut widest_w = 0;
    for d in b'0'..=b'9' {
        layout.set_text(&(d as char).to_string());
        let w = layout.pixel_size().0;
        if w > widest_w {
            widest_w = w;
            widest = d;
        }
    }

    layout.set_text(text);
    let actual_w = layout.pixel_size().0;

    // Measure the whole "worst case" rendering in one pass: every digit widened
    // and each integer part padded to `min_digits`. Taking the delta from a single
    // in-context measurement avoids mixing isolated glyph advances with in-context
    // ones (which made a padded 1-digit value render slightly *wider* than 2 digits).
    let padded = padded_canonical(text, widest as char, min_digits, time_only);
    layout.set_text(&padded);
    let padded_w = layout.pixel_size().0;

    (padded_w - actual_w).max(0)
}

/// Builds the widest same-shape rendering of `text`: every ASCII digit replaced by
/// `widest`, and **each number's integer part** padded with `widest` up to
/// `min_digits` digits. Digits after a decimal point are widened but not padded
/// (those fields render at a fixed precision), and each number is padded
/// independently, so multi-field labels like `"{{ percent }}% {{ temp }}C"` stay
/// stable field-by-field rather than only in aggregate.
///
/// When `time_only` is set, only digit groups directly adjacent to a `:` (a clock's
/// hour/minute/second) are widened and padded; every other number keeps its exact
/// characters, so date parts get no reserved space.
fn padded_canonical(text: &str, widest: char, min_digits: u32, time_only: bool) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(chars.len() + 4);
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_ascii_digit() {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }

        // A digit group is a clock time component if it directly touches a ':'.
        let is_time =
            (start > 0 && chars[start - 1] == ':') || (i < chars.len() && chars[i] == ':');
        if time_only && !is_time {
            for &c in &chars[start..i] {
                out.push(c);
            }
            continue;
        }

        let run_len = (i - start) as u32;
        // A digit run immediately after "<digit>." is a fractional part; don't pad it.
        let is_fractional =
            start >= 2 && chars[start - 1] == '.' && chars[start - 2].is_ascii_digit();
        if !is_fractional {
            for _ in 0..min_digits.saturating_sub(run_len) {
                out.push(widest);
            }
        }
        for _ in 0..run_len {
            out.push(widest);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::padded_canonical;

    #[test]
    fn single_integer_pads_to_min() {
        assert_eq!(padded_canonical("5", '8', 2, false), "88");
        assert_eq!(padded_canonical("50", '8', 2, false), "88");
        assert_eq!(padded_canonical("100", '8', 2, false), "888");
    }

    #[test]
    fn fractional_part_is_widened_but_not_padded() {
        assert_eq!(padded_canonical("3.2", '8', 2, false), "88.8");
        assert_eq!(padded_canonical("512.0", '8', 3, false), "888.8");
        assert_eq!(padded_canonical("0.0", '8', 3, false), "888.8");
    }

    #[test]
    fn each_number_padded_independently() {
        assert_eq!(padded_canonical("5% 8C", '8', 2, false), "88% 88C");
        assert_eq!(padded_canonical("45% 55C", '8', 2, false), "88% 88C");
        assert_eq!(padded_canonical("5% 08C", '8', 2, false), "88% 88C");
    }

    #[test]
    fn non_digits_are_preserved() {
        assert_eq!(padded_canonical("", '8', 2, false), "");
        assert_eq!(padded_canonical("--", '8', 2, false), "--");
        assert_eq!(padded_canonical("KiB/s", '8', 2, false), "KiB/s");
        assert_eq!(padded_canonical("55°C", '8', 2, false), "88°C");
    }

    #[test]
    fn time_only_reserves_time_but_not_date() {
        // Hour padded to 2 digits and the time widened; the day stays natural.
        assert_eq!(padded_canonical("Jan 5 9:30 PM", '8', 2, true), "Jan 5 88:88 PM");
        assert_eq!(padded_canonical("Jan 15 12:30 PM", '8', 2, true), "Jan 15 88:88 PM");
        assert_eq!(padded_canonical("9:30 UTC", '8', 2, true), "88:88 UTC");
        // A year is not colon-adjacent, so it is left at its natural width.
        assert_eq!(padded_canonical("2024 9:05", '8', 2, true), "2024 88:88");
    }
}
