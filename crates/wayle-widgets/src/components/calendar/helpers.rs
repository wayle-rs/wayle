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
///
/// `week_start` controls which day appears in column 0.
pub fn build_month_grid(
    month: NaiveDate,
    today: NaiveDate,
    selected: Option<NaiveDate>,
    week_start: Weekday,
) -> Vec<DayCell> {
    let first_of_month = month.with_day(1).unwrap_or(month);
    let target_month = first_of_month.month();

    // Number of leading filler days before the 1st: how many days from week_start
    // to first_of_month's weekday, counting forward (mod 7).
    let start_offset = week_start.num_days_from_monday();
    let dow_offset = first_of_month.weekday().num_days_from_monday();
    let leading_days = (dow_offset + 7 - start_offset) % 7;
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

    #[test]
    fn always_produces_42_cells() {
        let march = build_month_grid(date(2026, 3, 1), date(2026, 3, 5), None, Weekday::Sun);
        assert_eq!(march.len(), 42);

        let august = build_month_grid(date(2026, 8, 1), date(2026, 3, 5), None, Weekday::Sun);
        assert_eq!(august.len(), 42);

        let february = build_month_grid(date(2026, 2, 1), date(2026, 3, 5), None, Weekday::Sun);
        assert_eq!(february.len(), 42);
    }

    #[test]
    fn march_2026_starts_on_sunday() {
        let grid = build_month_grid(date(2026, 3, 1), date(2026, 3, 5), None, Weekday::Sun);
        assert_eq!(grid[0].date, date(2026, 3, 1));
        assert_eq!(grid[0].date.weekday(), Weekday::Sun);
    }

    #[test]
    fn march_2026_trailing_days_are_other_month() {
        let grid = build_month_grid(date(2026, 3, 1), date(2026, 3, 5), None, Weekday::Sun);
        assert!(grid[30].is_current_month);
        assert_eq!(grid[30].date, date(2026, 3, 31));
        assert!(!grid[31].is_current_month);
        assert_eq!(grid[31].date, date(2026, 4, 1));
    }

    #[test]
    fn august_2026_starts_with_leading_days() {
        let grid = build_month_grid(date(2026, 8, 1), date(2026, 3, 5), None, Weekday::Sun);
        assert_eq!(grid[0].date, date(2026, 7, 26));
        assert!(!grid[0].is_current_month);
    }

    #[test]
    fn today_is_highlighted() {
        let today = date(2026, 3, 5);
        let grid = build_month_grid(date(2026, 3, 1), today, None, Weekday::Sun);
        let march_5 = grid.iter().find(|c| c.date == today).unwrap();
        assert!(march_5.is_today);
        assert!(march_5.is_current_month);
    }

    #[test]
    fn selected_day_is_marked() {
        let selected = date(2026, 3, 20);
        let grid = build_month_grid(date(2026, 3, 1), date(2026, 3, 5), Some(selected), Weekday::Sun);
        let march_20 = grid.iter().find(|c| c.date == selected).unwrap();
        assert!(march_20.is_selected);
    }

    #[test]
    fn weekends_are_marked() {
        // is_weekend is date-based (Sat/Sun), independent of week start
        let grid = build_month_grid(date(2026, 3, 1), date(2026, 3, 5), None, Weekday::Sun);
        assert!(grid[0].is_weekend); // 2026-03-01 is a Sunday
        assert!(!grid[1].is_weekend); // Monday
        assert!(grid[6].is_weekend); // Saturday
    }

    #[test]
    fn february_2026_has_28_current_month_days() {
        let grid = build_month_grid(date(2026, 2, 1), date(2026, 3, 5), None, Weekday::Sun);
        assert_eq!(grid[0].date, date(2026, 2, 1));
        let feb_cells: Vec<_> = grid.iter().filter(|cell| cell.is_current_month).collect();
        assert_eq!(feb_cells.len(), 28);
    }

    #[test]
    fn today_in_different_month_not_highlighted() {
        let grid = build_month_grid(date(2026, 4, 1), date(2026, 3, 5), None, Weekday::Sun);
        assert!(grid.iter().all(|c| !c.is_today));
    }

    #[test]
    fn any_day_in_month_selects_correct_month() {
        let from_mid = build_month_grid(date(2026, 3, 15), date(2026, 3, 5), None, Weekday::Sun);
        let from_first = build_month_grid(date(2026, 3, 1), date(2026, 3, 5), None, Weekday::Sun);
        assert_eq!(from_mid, from_first);
    }

    #[test]
    fn leap_year_february_has_29_days() {
        let grid = build_month_grid(date(2028, 2, 1), date(2028, 2, 15), None, Weekday::Sun);
        let feb_cells: Vec<_> = grid.iter().filter(|cell| cell.is_current_month).collect();
        assert_eq!(feb_cells.len(), 29);
    }

    #[test]
    fn december_year_boundary() {
        let grid = build_month_grid(date(2026, 12, 1), date(2026, 12, 25), None, Weekday::Sun);
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
        let grid = build_month_grid(date(2027, 1, 1), date(2027, 1, 10), None, Weekday::Sun);
        let jan_cells: Vec<_> = grid.iter().filter(|cell| cell.is_current_month).collect();
        assert_eq!(jan_cells.len(), 31);
    }

    #[test]
    fn first_column_matches_week_start() {
        // (week_start, expected weekday at col 6)
        let cases: &[(Weekday, Weekday)] = &[
            (Weekday::Sun, Weekday::Sat),
            (Weekday::Mon, Weekday::Sun),
            (Weekday::Tue, Weekday::Mon),
            (Weekday::Wed, Weekday::Tue),
            (Weekday::Thu, Weekday::Wed),
            (Weekday::Fri, Weekday::Thu),
            (Weekday::Sat, Weekday::Fri),
        ];
        for &(start, expected_last_col) in cases {
            for month in 1..=12 {
                let grid =
                    build_month_grid(date(2026, month, 1), date(2026, 1, 1), None, start);
                assert_eq!(
                    grid[0].date.weekday(),
                    start,
                    "start={start:?} month={month}: col 0 wrong"
                );
                assert_eq!(
                    grid[6].date.weekday(),
                    expected_last_col,
                    "start={start:?} month={month}: col 6 wrong"
                );
            }
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
