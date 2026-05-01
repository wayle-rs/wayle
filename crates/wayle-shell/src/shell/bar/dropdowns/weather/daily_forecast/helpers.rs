use chrono::{Datelike, Local, NaiveDate, Weekday};
use wayle_weather::Temperature;

use crate::i18n::t;

pub(super) struct TempBarOffsets {
    pub left_pct: f32,
    pub width_pct: f32,
}

pub(super) fn temp_bar_offsets(
    day_low: f32,
    day_high: f32,
    range_min: f32,
    range_max: f32,
) -> TempBarOffsets {
    let range = range_max - range_min;
    if range <= 0.0 {
        return TempBarOffsets {
            left_pct: 0.0,
            width_pct: 100.0,
        };
    }

    let left_pct = ((day_low - range_min) / range * 100.0).clamp(0.0, 100.0);
    let right_pct = ((day_high - range_min) / range * 100.0).clamp(0.0, 100.0);
    let width_pct = (right_pct - left_pct).max(5.0);

    TempBarOffsets {
        left_pct,
        width_pct,
    }
}

pub(super) fn day_label(date: NaiveDate) -> String {
    match date.weekday() {
        Weekday::Sun => t!("cal-weekday-sun"),
        Weekday::Mon => t!("cal-weekday-mon"),
        Weekday::Tue => t!("cal-weekday-tue"),
        Weekday::Wed => t!("cal-weekday-wed"),
        Weekday::Thu => t!("cal-weekday-thu"),
        Weekday::Fri => t!("cal-weekday-fri"),
        Weekday::Sat => t!("cal-weekday-sat"),
    }
}

pub(super) fn is_today(date: NaiveDate) -> bool {
    date == Local::now().date_naive()
}

pub(super) fn temp_range(days: &[(Temperature, Temperature)]) -> (f32, f32) {
    let mut min = f32::MAX;
    let mut max = f32::MIN;

    for (low, high) in days {
        let low_c = low.celsius();
        let high_c = high.celsius();
        if low_c < min {
            min = low_c;
        }
        if high_c > max {
            max = high_c;
        }
    }

    if min == f32::MAX {
        return (0.0, 0.0);
    }

    (min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp_bar_offsets_full_range() {
        let offsets = temp_bar_offsets(10.0, 30.0, 10.0, 30.0);
        assert!((offsets.left_pct - 0.0).abs() < 0.1);
        assert!((offsets.width_pct - 100.0).abs() < 0.1);
    }

    #[test]
    fn temp_bar_offsets_partial_range() {
        let offsets = temp_bar_offsets(15.0, 25.0, 10.0, 30.0);
        assert!((offsets.left_pct - 25.0).abs() < 0.1);
        assert!((offsets.width_pct - 50.0).abs() < 0.1);
    }

    #[test]
    fn temp_bar_offsets_minimum_width() {
        let offsets = temp_bar_offsets(20.0, 20.0, 10.0, 30.0);
        assert!(offsets.width_pct >= 5.0);
    }

    #[test]
    fn temp_bar_offsets_zero_range() {
        let offsets = temp_bar_offsets(20.0, 20.0, 20.0, 20.0);
        assert!((offsets.left_pct - 0.0).abs() < 0.1);
        assert!((offsets.width_pct - 100.0).abs() < 0.1);
    }

    #[test]
    fn day_label_returns_abbreviation() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let label = day_label(date);
        assert!(!label.is_empty());
    }

    #[test]
    fn temp_range_multiple_days() {
        let days = vec![
            (
                Temperature::new(15.0).unwrap(),
                Temperature::new(25.0).unwrap(),
            ),
            (
                Temperature::new(10.0).unwrap(),
                Temperature::new(30.0).unwrap(),
            ),
        ];
        let (min, max) = temp_range(&days);
        assert!((min - 10.0).abs() < 0.1);
        assert!((max - 30.0).abs() < 0.1);
    }

    #[test]
    fn temp_range_empty() {
        let days: Vec<(Temperature, Temperature)> = vec![];
        let (min, max) = temp_range(&days);
        assert!((min - 0.0).abs() < 0.1);
        assert!((max - 0.0).abs() < 0.1);
    }
}
