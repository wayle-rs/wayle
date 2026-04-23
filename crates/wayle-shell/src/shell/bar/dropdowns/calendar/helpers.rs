use chrono::{DateTime, Datelike, Local, Weekday};

use crate::i18n::t;

pub(super) fn is_12h_format(format_str: &str) -> bool {
    format_str.contains("%I") || format_str.contains("%p")
}

pub(super) fn hours_text(now: &DateTime<Local>, use_12h: bool) -> String {
    if use_12h {
        now.format("%I").to_string()
    } else {
        now.format("%H").to_string()
    }
}

pub(super) fn minutes_text(now: &DateTime<Local>) -> String {
    now.format("%M").to_string()
}

pub(super) fn seconds_text(now: &DateTime<Local>) -> String {
    now.format("%S").to_string()
}

pub(super) fn ampm_text(now: &DateTime<Local>) -> String {
    now.format("%p").to_string()
}

pub(super) fn format_date_rest(months: &[String; 12], now: &DateTime<Local>) -> String {
    let month_idx = now.month0() as usize;
    t!(
        "cal-clock-date-rest",
        month = months[month_idx].clone(),
        day = now.day().to_string(),
        year = now.year().to_string()
    )
}

pub(super) fn day_names_array() -> [String; 7] {
    [
        t!("cal-day-sunday"),
        t!("cal-day-monday"),
        t!("cal-day-tuesday"),
        t!("cal-day-wednesday"),
        t!("cal-day-thursday"),
        t!("cal-day-friday"),
        t!("cal-day-saturday"),
    ]
}

pub(super) fn weekdays_array(week_start: Weekday) -> [String; 7] {
    // Base order is Sunday-first (matches num_days_from_sunday indexing).
    let base = [
        t!("cal-weekday-sun"),
        t!("cal-weekday-mon"),
        t!("cal-weekday-tue"),
        t!("cal-weekday-wed"),
        t!("cal-weekday-thu"),
        t!("cal-weekday-fri"),
        t!("cal-weekday-sat"),
    ];
    // Rotate left by the start day's Sunday-offset so week_start lands at col 0.
    let rot = week_start.num_days_from_sunday() as usize;
    std::array::from_fn(|i| base[(rot + i) % 7].clone())
}

pub(super) fn months_array() -> [String; 12] {
    [
        t!("cal-month-january"),
        t!("cal-month-february"),
        t!("cal-month-march"),
        t!("cal-month-april"),
        t!("cal-month-may"),
        t!("cal-month-june"),
        t!("cal-month-july"),
        t!("cal-month-august"),
        t!("cal-month-september"),
        t!("cal-month-october"),
        t!("cal-month-november"),
        t!("cal-month-december"),
    ]
}
