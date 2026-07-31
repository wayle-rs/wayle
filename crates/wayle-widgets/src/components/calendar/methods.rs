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
        attach_weekday_headers(&self.grid, &self.weekdays, self.week_start);
        attach_day_cells(
            &self.grid,
            self.displayed_month,
            self.today,
            self.selected_day,
            self.week_start,
            sender,
        );
    }
}

fn clear_grid(grid: &gtk::Grid) {
    while let Some(child) = grid.first_child() {
        grid.remove(&child);
    }
}

fn attach_weekday_headers(grid: &gtk::Grid, weekdays: &[String; 7], week_start: Weekday) {
    // Compute which columns land on Saturday and Sunday for the chosen week_start.
    // num_days_from_monday: Mon=0, Tue=1, Wed=2, Thu=3, Fri=4, Sat=5, Sun=6
    let start = week_start.num_days_from_monday() as usize;
    let sat_col = (5usize + 7 - start) % 7;
    let sun_col = (6usize + 7 - start) % 7;

    for (col, weekday_name) in weekdays.iter().enumerate() {
        let label = gtk::Label::new(Some(weekday_name));
        label.add_css_class("cal-weekday");
        label.set_hexpand(true);

        if col == sat_col || col == sun_col {
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
    week_start: Weekday,
    sender: &ComponentSender<Calendar>,
) {
    let cells = build_month_grid(displayed_month, today, selected_day, week_start);

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
