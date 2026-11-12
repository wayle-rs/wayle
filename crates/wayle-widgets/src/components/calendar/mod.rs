/// Date math for building the calendar grid.
pub mod helpers;
/// Calendar component messages.
pub mod messages;
mod methods;

use chrono::{Datelike, Months, NaiveDate, Weekday};
use gtk::prelude::*;
use relm4::{gtk, prelude::*};

pub use self::messages::{CalendarInit, CalendarInput, CalendarLabels, CalendarOutput};
use crate::components::calendar::helpers::format_month_label;

/// Calendar grid with month navigation, today highlighting, and day selection.
pub struct Calendar {
    displayed_month: NaiveDate,
    selected_day: Option<NaiveDate>,
    today: NaiveDate,
    month_label: String,

    months: [String; 12],
    month_year_pattern: String,
    weekdays: [String; 7],
    week_start: Weekday,
    grid: gtk::Grid,
}

#[allow(missing_docs)]
#[relm4::component(pub)]
impl Component for Calendar {
    type Init = CalendarInit;
    type Input = CalendarInput;
    type Output = CalendarOutput;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            add_css_class: "cal-section",
            set_orientation: gtk::Orientation::Vertical,

            gtk::Box {
                add_css_class: "cal-header",

                gtk::Label {
                    add_css_class: "cal-month",
                    #[watch]
                    set_label: &model.month_label,
                },

                gtk::Box {
                    add_css_class: "cal-nav",
                    set_hexpand: true,
                    set_halign: gtk::Align::End,

                    #[name = "today_btn"]
                    gtk::Button {
                        set_css_classes: &["cal-nav-btn", "cal-today-btn"],
                        set_cursor_from_name: Some("pointer"),
                        connect_clicked => CalendarInput::GoToToday,
                    },

                    gtk::Button {
                        add_css_class: "cal-nav-btn",
                        set_icon_name: "ld-chevron-left-symbolic",
                        set_cursor_from_name: Some("pointer"),
                        connect_clicked => CalendarInput::PrevMonth,
                    },

                    gtk::Button {
                        add_css_class: "cal-nav-btn",
                        set_icon_name: "ld-chevron-right-symbolic",
                        set_cursor_from_name: Some("pointer"),
                        connect_clicked => CalendarInput::NextMonth,
                    },
                },
            },

            gtk::Box {
                add_css_class: "cal-grid-wrap",
                set_hexpand: true,

                #[local_ref]
                grid_widget -> gtk::Grid {
                    add_css_class: "cal-grid",
                    set_hexpand: true,
                    set_column_homogeneous: true,
                },
            },
        },
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let grid = gtk::Grid::new();
        let displayed_month = init.today.with_day(1).unwrap_or(init.today);

        let today_label = init.labels.today;
        let month_year_pattern = init.labels.month_year;

        let model = Self {
            displayed_month,
            selected_day: None,
            today: init.today,
            month_label: format_month_label(
                displayed_month,
                &init.labels.months,
                &month_year_pattern,
            ),
            months: init.labels.months,
            month_year_pattern,
            weekdays: init.labels.weekdays,
            week_start: init.week_start,
            grid: grid.clone(),
        };

        model.rebuild_grid(&sender);

        let grid_widget = &model.grid;
        let widgets = view_output!();
        widgets.today_btn.set_label(&today_label);
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            CalendarInput::PrevMonth => {
                let Some(prev) = self.displayed_month.checked_sub_months(Months::new(1)) else {
                    return;
                };
                self.displayed_month = prev;
                self.month_label = format_month_label(prev, &self.months, &self.month_year_pattern);
                self.rebuild_grid(&sender);
            }

            CalendarInput::NextMonth => {
                let Some(next) = self.displayed_month.checked_add_months(Months::new(1)) else {
                    return;
                };
                self.displayed_month = next;
                self.month_label = format_month_label(next, &self.months, &self.month_year_pattern);
                self.rebuild_grid(&sender);
            }

            CalendarInput::GoToToday => {
                let today_month = self.today.with_day(1).unwrap_or(self.today);
                if self.displayed_month != today_month {
                    self.displayed_month = today_month;
                    self.month_label =
                        format_month_label(today_month, &self.months, &self.month_year_pattern);
                    self.selected_day = None;
                    self.rebuild_grid(&sender);
                }
            }

            CalendarInput::DayClicked(date) => {
                if date == self.today {
                    self.selected_day = None;
                } else {
                    self.selected_day = Some(date);
                }
                self.rebuild_grid(&sender);
                let _ = sender.output(CalendarOutput::DaySelected(date));
            }

            CalendarInput::UpdateToday(new_today) => {
                if self.today != new_today {
                    self.today = new_today;
                    self.rebuild_grid(&sender);
                }
            }

            CalendarInput::UpdateWeekStart(week_start) => {
                if self.week_start != week_start {
                    self.week_start = week_start;
                    self.rebuild_grid(&sender);
                }
            }
        }
    }
}
