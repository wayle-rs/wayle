use chrono::{Datelike, Duration, NaiveDate, Weekday};

const GRID_CELLS: u32 = 42;

/// Single cell in the calendar grid.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct DayCell {
    pub date: NaiveDate,
    pub is_current_month: bool,
    pub is_today: bool,
    pub is_weekend: bool,
    pub is_selected: bool,
}

/// Builds a 42-cell grid (6 rows x 7 columns) for the given month.
///
/// Always returns exactly 42 cells to keep the grid height stable across
/// month navigation. Otherwise gtk annoyingly auto hides hides popovers
/// when their dimensions change.
pub fn build_month_grid(
    month: NaiveDate,
    today: NaiveDate,
    selected: Option<NaiveDate>,
    first_weekday: Weekday,
) -> Vec<DayCell> {
    let first_of_month = month.with_day(1).unwrap_or(month);
    let target_month = first_of_month.month();

    // Days between the configured first weekday and the weekday the month
    // starts on, i.e. how many leading cells from the previous month to show.
    let first_offset = first_weekday.num_days_from_sunday();
    let leading_days = (first_of_month.weekday().num_days_from_sunday() + 7 - first_offset) % 7;
    let grid_start = first_of_month - Duration::days(i64::from(leading_days));

    (0..GRID_CELLS)
        .map(|day_index| {
            let days_from_grid_start = Duration::days(i64::from(day_index));
            let date = grid_start + days_from_grid_start;
            DayCell {
                date,
                is_current_month: date.month() == target_month,
                is_today: date == today,
                is_weekend: matches!(date.weekday(), Weekday::Sun | Weekday::Sat),
                is_selected: selected == Some(date),
            }
        })
        .collect()
}

/// Formats the month navigation label using a locale-provided pattern.
///
/// Replaces `{month}` and `{year}` placeholders in `pattern`.
pub fn format_month_label(date: NaiveDate, months: &[String; 12], pattern: &str) -> String {
    let month_idx = date.month0() as usize;
    pattern
        .replace("{month}", &months[month_idx])
        .replace("{year}", &date.year().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    /// Builds a grid with the default Sunday week start, for tests that
    /// predate the configurable first weekday.
    fn grid_from(month: NaiveDate, today: NaiveDate, selected: Option<NaiveDate>) -> Vec<DayCell> {
        build_month_grid(month, today, selected, Weekday::Sun)
    }

    #[test]
    fn always_produces_42_cells() {
        let march = grid_from(date(2026, 3, 1), date(2026, 3, 5), None);
        assert_eq!(march.len(), 42);

        let august = grid_from(date(2026, 8, 1), date(2026, 3, 5), None);
        assert_eq!(august.len(), 42);

        let february = grid_from(date(2026, 2, 1), date(2026, 3, 5), None);
        assert_eq!(february.len(), 42);
    }

    #[test]
    fn march_2026_starts_on_sunday() {
        let grid = grid_from(date(2026, 3, 1), date(2026, 3, 5), None);
        assert_eq!(grid[0].date, date(2026, 3, 1));
        assert_eq!(grid[0].date.weekday(), Weekday::Sun);
    }

    #[test]
    fn march_2026_trailing_days_are_other_month() {
        let grid = grid_from(date(2026, 3, 1), date(2026, 3, 5), None);
        assert!(grid[30].is_current_month);
        assert_eq!(grid[30].date, date(2026, 3, 31));
        assert!(!grid[31].is_current_month);
        assert_eq!(grid[31].date, date(2026, 4, 1));
    }

    #[test]
    fn august_2026_starts_with_leading_days() {
        let grid = grid_from(date(2026, 8, 1), date(2026, 3, 5), None);
        assert_eq!(grid[0].date, date(2026, 7, 26));
        assert!(!grid[0].is_current_month);
    }

    #[test]
    fn today_is_highlighted() {
        let today = date(2026, 3, 5);
        let grid = grid_from(date(2026, 3, 1), today, None);
        let march_5 = grid.iter().find(|c| c.date == today).unwrap();
        assert!(march_5.is_today);
        assert!(march_5.is_current_month);
    }

    #[test]
    fn selected_day_is_marked() {
        let selected = date(2026, 3, 20);
        let grid = grid_from(date(2026, 3, 1), date(2026, 3, 5), Some(selected));
        let march_20 = grid.iter().find(|c| c.date == selected).unwrap();
        assert!(march_20.is_selected);
    }

    #[test]
    fn weekends_are_marked() {
        let grid = grid_from(date(2026, 3, 1), date(2026, 3, 5), None);
        assert!(grid[0].is_weekend);
        assert!(!grid[1].is_weekend);
        assert!(grid[6].is_weekend);
    }

    #[test]
    fn february_2026_has_28_current_month_days() {
        let grid = grid_from(date(2026, 2, 1), date(2026, 3, 5), None);
        assert_eq!(grid[0].date, date(2026, 2, 1));
        let feb_cells: Vec<_> = grid.iter().filter(|cell| cell.is_current_month).collect();
        assert_eq!(feb_cells.len(), 28);
    }

    #[test]
    fn today_in_different_month_not_highlighted() {
        let grid = grid_from(date(2026, 4, 1), date(2026, 3, 5), None);
        assert!(grid.iter().all(|c| !c.is_today));
    }

    #[test]
    fn any_day_in_month_selects_correct_month() {
        let from_mid = grid_from(date(2026, 3, 15), date(2026, 3, 5), None);
        let from_first = grid_from(date(2026, 3, 1), date(2026, 3, 5), None);
        assert_eq!(from_mid, from_first);
    }

    #[test]
    fn leap_year_february_has_29_days() {
        let grid = grid_from(date(2028, 2, 1), date(2028, 2, 15), None);
        let feb_cells: Vec<_> = grid.iter().filter(|cell| cell.is_current_month).collect();
        assert_eq!(feb_cells.len(), 29);
    }

    #[test]
    fn december_year_boundary() {
        let grid = grid_from(date(2026, 12, 1), date(2026, 12, 25), None);
        let dec_cells: Vec<_> = grid.iter().filter(|cell| cell.is_current_month).collect();
        assert_eq!(dec_cells.len(), 31);

        let last_dec = grid.iter().rfind(|cell| cell.is_current_month).unwrap();
        assert_eq!(last_dec.date, date(2026, 12, 31));

        let jan_trailing = grid
            .iter()
            .filter(|cell| !cell.is_current_month && cell.date.month() == 1)
            .count();
        assert!(jan_trailing > 0);
    }

    #[test]
    fn january_year_boundary() {
        let grid = grid_from(date(2027, 1, 1), date(2027, 1, 10), None);
        let jan_cells: Vec<_> = grid.iter().filter(|cell| cell.is_current_month).collect();
        assert_eq!(jan_cells.len(), 31);
    }

    #[test]
    fn first_column_is_always_sunday() {
        for month in 1..=12 {
            let grid = grid_from(date(2026, month, 1), date(2026, 1, 1), None);
            assert_eq!(
                grid[0].date.weekday(),
                Weekday::Sun,
                "month {month} col 0 not Sunday"
            );
            assert_eq!(
                grid[6].date.weekday(),
                Weekday::Sat,
                "month {month} col 6 not Saturday"
            );
        }
    }

    #[test]
    fn monday_start_reorders_first_column() {
        // March 2026 starts on a Sunday; with a Monday week start the grid's
        // first column is the preceding Monday and Sunday moves to column 6.
        let grid = build_month_grid(date(2026, 3, 1), date(2026, 3, 5), None, Weekday::Mon);
        assert_eq!(grid.len(), 42);
        assert_eq!(grid[0].date.weekday(), Weekday::Mon);
        assert_eq!(grid[0].date, date(2026, 2, 23));
        assert_eq!(grid[6].date.weekday(), Weekday::Sun);
        assert_eq!(grid[6].date, date(2026, 3, 1));
    }

    #[test]
    fn saturday_start_reorders_first_column() {
        let grid = build_month_grid(date(2026, 3, 1), date(2026, 3, 5), None, Weekday::Sat);
        assert_eq!(grid.len(), 42);
        assert_eq!(grid[0].date.weekday(), Weekday::Sat);
        assert_eq!(grid[0].date, date(2026, 2, 28));
        assert_eq!(grid[6].date.weekday(), Weekday::Fri);
    }

    #[test]
    fn monday_start_keeps_current_month_day_count() {
        // Reordering must not change which days belong to the month.
        let grid = build_month_grid(date(2026, 2, 1), date(2026, 3, 5), None, Weekday::Mon);
        let feb_cells = grid.iter().filter(|c| c.is_current_month).count();
        assert_eq!(feb_cells, 28);
    }

    #[test]
    fn weekend_marking_is_independent_of_week_start() {
        // Weekend stays Saturday/Sunday regardless of the chosen first day.
        let grid = build_month_grid(date(2026, 3, 1), date(2026, 3, 5), None, Weekday::Mon);
        for cell in &grid {
            let expected = matches!(cell.date.weekday(), Weekday::Sat | Weekday::Sun);
            assert_eq!(cell.is_weekend, expected);
        }
    }

    fn months() -> [String; 12] {
        [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ]
        .map(String::from)
    }

    const EN_PATTERN: &str = "{month} {year}";

    #[test]
    fn format_month_label_march_2026() {
        assert_eq!(
            format_month_label(date(2026, 3, 1), &months(), EN_PATTERN),
            "March 2026"
        );
    }

    #[test]
    fn format_month_label_mid_month_uses_correct_month() {
        assert_eq!(
            format_month_label(date(2026, 3, 15), &months(), EN_PATTERN),
            "March 2026"
        );
    }

    #[test]
    fn format_month_label_january_year_boundary() {
        assert_eq!(
            format_month_label(date(2027, 1, 1), &months(), EN_PATTERN),
            "January 2027"
        );
    }

    #[test]
    fn format_month_label_december() {
        assert_eq!(
            format_month_label(date(2026, 12, 1), &months(), EN_PATTERN),
            "December 2026"
        );
    }

    #[test]
    fn format_month_label_cjk_pattern() {
        assert_eq!(
            format_month_label(date(2026, 3, 1), &months(), "{year}年{month}"),
            "2026年March"
        );
    }
}
