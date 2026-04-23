use chrono::{NaiveDate, Weekday};

/// Localized strings for the calendar widget.
pub struct CalendarLabels {
    /// Abbreviated weekday names starting from the configured week start day.
    pub weekdays: [String; 7],
    /// Full month names starting from January.
    pub months: [String; 12],
    /// Label for the "go to today" button.
    pub today: String,
    /// Format pattern for the month navigation header.
    ///
    /// Receives `month` and `year` placeholders via Fluent-style substitution.
    /// The consumer is responsible for providing a localized pattern
    pub month_year: String,
}

/// Initialization data for the calendar widget.
pub struct CalendarInit {
    /// Date to highlight as "today".
    pub today: NaiveDate,
    /// Localized strings for weekdays, months, and navigation.
    pub labels: CalendarLabels,
    /// The weekday that appears in column 0 of the calendar grid.
    pub week_start: Weekday,
}

/// Calendar widget input messages.
#[derive(Debug)]
pub enum CalendarInput {
    /// Navigate to the previous month.
    PrevMonth,
    /// Navigate to the next month.
    NextMonth,
    /// Jump back to the current month.
    GoToToday,
    /// User clicked a day cell in the current month.
    DayClicked(NaiveDate),
    /// Midnight rollover — parent sends the new date.
    UpdateToday(NaiveDate),
    /// Week start day changed at runtime via config hot-reload.
    UpdateWeekStart(Weekday),
}

/// Calendar widget output messages.
#[derive(Debug)]
pub enum CalendarOutput {
    /// Emitted when a day is clicked.
    DaySelected(NaiveDate),
}
