mod factory;
mod helpers;
mod messages;
mod watchers;

use chrono::{Datelike, Local, NaiveDate};
use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_widgets::{
    components::calendar::{Calendar, CalendarInit, CalendarInput, CalendarLabels},
    prelude::*,
};

pub(super) use self::factory::Factory;
use self::{
    helpers::{day_names_array, format_date_rest, months_array, weekdays_array},
    messages::{CalendarDropdownCmd, CalendarDropdownInit},
};
use chrono::Weekday;

use crate::{i18n::t, shell::bar::dropdowns::scaled_dimension};

const BASE_WIDTH: f32 = 340.0;

pub(crate) struct CalendarDropdown {
    calendar: Controller<Calendar>,
    scaled_width: i32,
    use_12h: bool,
    show_seconds: bool,
    week_start: Weekday,
    last_today: NaiveDate,

    day_names: [String; 7],
    months: [String; 12],

    hours: String,
    minutes: String,
    seconds: String,
    ampm: String,
    day_name: String,
    date_rest: String,
}

#[relm4::component(pub(crate))]
impl Component for CalendarDropdown {
    type Init = CalendarDropdownInit;
    type Input = ();
    type Output = ();
    type CommandOutput = CalendarDropdownCmd;

    view! {
        #[root]
        gtk::Popover {
            set_css_classes: &["dropdown", "calendar-dropdown"],
            set_has_arrow: false,
            #[watch]
            set_width_request: model.scaled_width,

            #[template]
            Dropdown {

                #[template]
                DropdownHeader {
                    #[template_child]
                    icon {
                        set_visible: true,
                        set_icon_name: Some("tb-calendar-time-symbolic"),
                    },
                    #[template_child]
                    label {
                        set_label: &t!("dropdown-calendar-title"),
                    },
                },

                #[template]
                DropdownContent {

                    gtk::Box {
                        add_css_class: "clock-hero",
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Box {
                            set_halign: gtk::Align::Center,
                            #[watch]
                            set_css_classes: match (model.show_seconds, model.use_12h) {
                                (true, true) => &["clock-time-row", "show-seconds", "use-12h"],
                                (true, false) => &["clock-time-row", "show-seconds"],
                                (false, true) => &["clock-time-row", "use-12h"],
                                (false, false) => &["clock-time-row"],
                            },

                            gtk::Label {
                                add_css_class: "clock-time",
                                #[watch]
                                set_label: &model.hours,
                            },

                            gtk::Label {
                                set_css_classes: &["clock-time", "clock-separator"],
                                set_label: ":",
                            },

                            gtk::Label {
                                add_css_class: "clock-time",
                                #[watch]
                                set_label: &model.minutes,
                            },

                            gtk::Label {
                                set_css_classes: &["clock-time", "clock-separator"],
                                set_label: ":",
                                #[watch]
                                set_visible: model.show_seconds,
                            },

                            gtk::Label {
                                add_css_class: "clock-time",
                                #[watch]
                                set_label: &model.seconds,
                                #[watch]
                                set_visible: model.show_seconds,
                            },

                            gtk::Label {
                                add_css_class: "clock-ampm",
                                #[watch]
                                set_label: &model.ampm,
                                #[watch]
                                set_visible: model.use_12h,
                            },
                        },

                        gtk::Box {
                            add_css_class: "clock-date",
                            set_halign: gtk::Align::Center,

                            gtk::Label {
                                add_css_class: "clock-date-day",
                                #[watch]
                                set_label: &model.day_name,
                            },

                            gtk::Label {
                                #[watch]
                                set_label: &model.date_rest,
                            },
                        },
                    },

                    #[local_ref]
                    calendar_widget -> gtk::Box {},
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let now = Local::now();
        let today = now.date_naive();
        let clock_config = &init.config.config().modules.clock;
        let format_str = clock_config.format.get();
        let use_12h = helpers::is_12h_format(&format_str);
        let show_seconds = clock_config.dropdown_show_seconds.get();
        let week_start =
            watchers::week_start_to_weekday(clock_config.calendar_weekday_start.get());

        let months = months_array();

        let calendar = Calendar::builder()
            .launch(CalendarInit {
                today,
                week_start,
                labels: CalendarLabels {
                    today: t!("cal-today"),
                    weekdays: weekdays_array(week_start),
                    months: months.clone(),
                    month_year: t!("cal-month-year", month = "{month}", year = "{year}"),
                },
            })
            .detach();

        let scale = init.config.config().styling.scale.get().value();

        watchers::spawn(&sender, &init.config);

        let day_names = day_names_array();
        // day_names is always Sunday-indexed, so index with num_days_from_sunday.
        let weekday_idx = now.weekday().num_days_from_sunday() as usize;

        let model = Self {
            calendar,
            scaled_width: scaled_dimension(BASE_WIDTH, scale),
            use_12h,
            show_seconds,
            week_start,
            last_today: today,
            hours: helpers::hours_text(&now, use_12h),
            minutes: helpers::minutes_text(&now),
            seconds: helpers::seconds_text(&now),
            ampm: helpers::ampm_text(&now),
            day_name: day_names[weekday_idx].clone(),
            date_rest: format_date_rest(&months, &now),
            day_names,
            months,
        };

        let calendar_widget = model.calendar.widget();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            CalendarDropdownCmd::ScaleChanged(scale) => {
                self.scaled_width = scaled_dimension(BASE_WIDTH, scale);
            }

            CalendarDropdownCmd::TimeTick => {
                let now = Local::now();

                self.hours = helpers::hours_text(&now, self.use_12h);
                self.minutes = helpers::minutes_text(&now);
                self.seconds = helpers::seconds_text(&now);
                self.ampm = helpers::ampm_text(&now);

                let weekday_idx = now.weekday().num_days_from_sunday() as usize;
                self.day_name = self.day_names[weekday_idx].clone();
                self.date_rest = format_date_rest(&self.months, &now);

                let new_today = now.date_naive();
                if new_today != self.last_today {
                    self.last_today = new_today;
                    self.calendar.emit(CalendarInput::UpdateToday(new_today));
                }
            }

            CalendarDropdownCmd::FormatChanged(use_12h) => {
                let now = Local::now();
                self.use_12h = use_12h;
                self.hours = helpers::hours_text(&now, use_12h);
            }

            CalendarDropdownCmd::ShowSecondsChanged(show) => {
                self.show_seconds = show;
            }

            CalendarDropdownCmd::WeekStartChanged(week_start) => {
                self.week_start = week_start;
                self.calendar.emit(CalendarInput::UpdateWeekStart(week_start));
            }
        }
    }
}
