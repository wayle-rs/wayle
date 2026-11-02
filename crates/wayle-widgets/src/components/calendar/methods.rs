use chrono::{Datelike, Weekday};
use gtk::prelude::*;
use relm4::{ComponentSender, gtk};

use crate::components::calendar::{
    Calendar, CalendarInput,
    helpers::{DayCell, build_month_grid},
};

impl Calendar {
    pub(super) fn rebuild_grid(&self, sender: &ComponentSender<Self>) {
        clear_grid(&self.grid);
        attach_weekday_headers(&self.grid, &self.weekdays, self.first_weekday);
        attach_day_cells(
            &self.grid,
            self.displayed_month,
            self.today,
            self.selected_day,
            self.first_weekday,
            sender,
        );
    }
}

fn clear_grid(grid: &gtk::Grid) {
    while let Some(child) = grid.first_child() {
        grid.remove(&child);
    }
}

fn attach_weekday_headers(grid: &gtk::Grid, weekdays: &[String; 7], first_weekday: Weekday) {
    // `weekdays` is canonically ordered from Sunday (index 0). Rotate by the
    // first weekday's offset so the leftmost column matches the configured day.
    let first_offset = first_weekday.num_days_from_sunday() as usize;

    for col in 0..7 {
        let day_index = (first_offset + col) % 7;
        let label = gtk::Label::new(Some(&weekdays[day_index]));
        label.add_css_class("cal-weekday");
        label.set_hexpand(true);

        // Weekend is Sunday (0) or Saturday (6), independent of column position.
        let is_weekend = day_index == 0 || day_index == 6;
        if is_weekend {
            label.add_css_class("weekend");
        }

        grid.attach(&label, col as i32, 0, 1, 1);
    }
}

fn attach_day_cells(
    grid: &gtk::Grid,
    displayed_month: chrono::NaiveDate,
    today: chrono::NaiveDate,
    selected_day: Option<chrono::NaiveDate>,
    first_weekday: Weekday,
    sender: &ComponentSender<Calendar>,
) {
    let cells = build_month_grid(displayed_month, today, selected_day, first_weekday);

    for (idx, cell) in cells.iter().enumerate() {
        let col = (idx % 7) as i32;
        let row = (idx / 7) as i32 + 1;

        let day_label = create_day_label(cell);

        if cell.is_current_month {
            attach_click_handler(&day_label, cell.date, sender);
        }

        grid.attach(&day_label, col, row, 1, 1);
    }
}

fn create_day_label(cell: &DayCell) -> gtk::Label {
    let label = gtk::Label::new(Some(&cell.date.day().to_string()));
    label.add_css_class("cal-day");
    label.set_hexpand(true);
    apply_cell_classes(&label, cell);
    label
}

fn apply_cell_classes(label: &gtk::Label, cell: &DayCell) {
    if cell.is_today {
        label.add_css_class("today");
    }
    if cell.is_selected {
        label.add_css_class("selected");
    }
    if !cell.is_current_month {
        label.add_css_class("other");
    }
    if cell.is_weekend {
        label.add_css_class("weekend");
    }
}

fn attach_click_handler(
    label: &gtk::Label,
    date: chrono::NaiveDate,
    sender: &ComponentSender<Calendar>,
) {
    let click = gtk::GestureClick::new();
    let input_sender = sender.input_sender().clone();

    click.connect_released(move |gesture, _, _, _| {
        gesture.set_state(gtk::EventSequenceState::Claimed);
        input_sender.emit(CalendarInput::DayClicked(date));
    });

    label.add_controller(click);
    label.set_cursor_from_name(Some("pointer"));
}
