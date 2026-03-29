//! Configurable color thresholds for bar modules.
//!
//! Dynamic color overrides based on a numeric metric (e.g., CPU %, battery level).

use std::borrow::Cow;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::color::ColorValue;

/// A threshold entry that maps a numeric value range to color overrides.
///
/// At least one of `above` or `below` must be set. When both are set,
/// both conditions must be satisfied (AND logic).
///
/// ## TOML Example
///
/// ```toml
/// [[modules.cpu.thresholds]]
/// above = 70
/// icon-color = "status-warning"
/// label-color = "status-warning"
///
/// [[modules.cpu.thresholds]]
/// above = 90
/// icon-color = "status-error"
/// label-color = "status-error"
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ThresholdEntry {
    /// Activate when metric value >= this threshold.
    #[serde(default)]
    pub above: Option<f64>,

    /// Activate when metric value <= this threshold.
    #[serde(default)]
    pub below: Option<f64>,

    /// Override icon color when threshold is active.
    #[serde(rename = "icon-color", default)]
    pub icon_color: Option<ColorValue>,

    /// Override label color when threshold is active.
    #[serde(rename = "label-color", default)]
    pub label_color: Option<ColorValue>,

    /// Override icon background color when threshold is active.
    #[serde(rename = "icon-bg-color", default)]
    pub icon_bg_color: Option<ColorValue>,

    /// Override button background color when threshold is active.
    #[serde(rename = "button-bg-color", default)]
    pub button_bg_color: Option<ColorValue>,

    /// Override border color when threshold is active.
    #[serde(rename = "border-color", default)]
    pub border_color: Option<ColorValue>,
}

impl ThresholdEntry {
    /// Returns true if `value` satisfies this entry's conditions.
    fn matches(&self, value: f64) -> bool {
        let above_ok = self.above.is_none_or(|t| value >= t);
        let below_ok = self.below.is_none_or(|t| value <= t);
        // At least one condition must be specified.
        (self.above.is_some() || self.below.is_some()) && above_ok && below_ok
    }
}

/// Resolved color overrides from threshold evaluation.
///
/// `None` means no override — use the module's configured color.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ThresholdColors {
    /// Icon color override.
    pub icon_color: Option<ColorValue>,
    /// Label color override.
    pub label_color: Option<ColorValue>,
    /// Icon background color override.
    pub icon_background: Option<ColorValue>,
    /// Button background color override.
    pub button_background: Option<ColorValue>,
    /// Border color override.
    pub border_color: Option<ColorValue>,
}

impl ThresholdColors {
    /// Returns `true` if no color overrides are set.
    pub fn is_empty(&self) -> bool {
        self.icon_color.is_none()
            && self.label_color.is_none()
            && self.icon_background.is_none()
            && self.button_background.is_none()
            && self.border_color.is_none()
    }

    /// Resolves a single color slot: threshold override if present, otherwise
    /// the config property value resolved for the current theme.
    pub fn resolve_or<'a>(
        override_color: &'a Option<ColorValue>,
        config_css: Cow<'static, str>,
    ) -> Cow<'a, str> {
        match override_color {
            Some(color) => color.to_css(),
            None => config_css,
        }
    }
}

/// Evaluate threshold entries against a metric value.
///
/// Entries are checked in declaration order. The last matching entry wins
/// for each color slot, so place more specific thresholds after general ones:
///
/// ```toml
/// [[thresholds]]
/// above = 70           # matches 70-100
/// icon-color = "yellow"
///
/// [[thresholds]]
/// above = 90           # matches 90-100, overrides yellow
/// icon-color = "red"
/// ```
pub fn evaluate_thresholds(value: f64, entries: &[ThresholdEntry]) -> ThresholdColors {
    let mut result = ThresholdColors::default();

    for entry in entries {
        if entry.matches(value) {
            if entry.icon_color.is_some() {
                result.icon_color.clone_from(&entry.icon_color);
            }
            if entry.label_color.is_some() {
                result.label_color.clone_from(&entry.label_color);
            }
            if entry.icon_bg_color.is_some() {
                result.icon_background.clone_from(&entry.icon_bg_color);
            }
            if entry.button_bg_color.is_some() {
                result.button_background.clone_from(&entry.button_bg_color);
            }
            if entry.border_color.is_some() {
                result.border_color.clone_from(&entry.border_color);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::styling::CssToken;

    fn warning_entry(above: f64) -> ThresholdEntry {
        ThresholdEntry {
            above: Some(above),
            below: None,
            icon_color: Some(ColorValue::Token(CssToken::StatusWarning)),
            label_color: Some(ColorValue::Token(CssToken::StatusWarning)),
            icon_bg_color: None,
            button_bg_color: None,
            border_color: None,
        }
    }

    fn error_entry(above: f64) -> ThresholdEntry {
        ThresholdEntry {
            above: Some(above),
            below: None,
            icon_color: Some(ColorValue::Token(CssToken::StatusError)),
            label_color: Some(ColorValue::Token(CssToken::StatusError)),
            icon_bg_color: None,
            button_bg_color: None,
            border_color: None,
        }
    }

    fn below_entry(below: f64, color: ColorValue) -> ThresholdEntry {
        ThresholdEntry {
            above: None,
            below: Some(below),
            icon_color: Some(color),
            label_color: None,
            icon_bg_color: None,
            button_bg_color: None,
            border_color: None,
        }
    }

    #[test]
    fn no_thresholds_returns_empty() {
        let result = evaluate_thresholds(50.0, &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn below_threshold_returns_empty() {
        let entries = vec![warning_entry(70.0)];
        let result = evaluate_thresholds(50.0, &entries);
        assert!(result.is_empty());
    }

    #[test]
    fn at_threshold_matches() {
        let entries = vec![warning_entry(70.0)];
        let result = evaluate_thresholds(70.0, &entries);
        assert_eq!(
            result.icon_color,
            Some(ColorValue::Token(CssToken::StatusWarning))
        );
    }

    #[test]
    fn above_threshold_matches() {
        let entries = vec![warning_entry(70.0)];
        let result = evaluate_thresholds(85.0, &entries);
        assert_eq!(
            result.icon_color,
            Some(ColorValue::Token(CssToken::StatusWarning))
        );
    }

    #[test]
    fn higher_threshold_overrides_lower() {
        let entries = vec![warning_entry(70.0), error_entry(90.0)];
        let result = evaluate_thresholds(95.0, &entries);
        assert_eq!(
            result.icon_color,
            Some(ColorValue::Token(CssToken::StatusError))
        );
        assert_eq!(
            result.label_color,
            Some(ColorValue::Token(CssToken::StatusError))
        );
    }

    #[test]
    fn only_lower_threshold_matches_when_between() {
        let entries = vec![warning_entry(70.0), error_entry(90.0)];
        let result = evaluate_thresholds(80.0, &entries);
        assert_eq!(
            result.icon_color,
            Some(ColorValue::Token(CssToken::StatusWarning))
        );
    }

    #[test]
    fn below_condition_matches_low_value() {
        let entries = vec![below_entry(20.0, ColorValue::Token(CssToken::StatusError))];
        let result = evaluate_thresholds(15.0, &entries);
        assert_eq!(
            result.icon_color,
            Some(ColorValue::Token(CssToken::StatusError))
        );
    }

    #[test]
    fn below_condition_no_match_for_high_value() {
        let entries = vec![below_entry(20.0, ColorValue::Token(CssToken::StatusError))];
        let result = evaluate_thresholds(50.0, &entries);
        assert!(result.is_empty());
    }

    #[test]
    fn partial_override_preserves_unset_slots() {
        let entries = vec![ThresholdEntry {
            above: Some(70.0),
            below: None,
            icon_color: Some(ColorValue::Token(CssToken::StatusWarning)),
            label_color: None,
            icon_bg_color: None,
            button_bg_color: None,
            border_color: None,
        }];
        let result = evaluate_thresholds(80.0, &entries);
        assert!(result.icon_color.is_some());
        assert!(result.label_color.is_none());
        assert!(result.icon_background.is_none());
    }

    #[test]
    fn both_above_and_below_requires_both() {
        let entries = vec![ThresholdEntry {
            above: Some(30.0),
            below: Some(70.0),
            icon_color: Some(ColorValue::Token(CssToken::StatusSuccess)),
            label_color: None,
            icon_bg_color: None,
            button_bg_color: None,
            border_color: None,
        }];
        // Inside range
        assert!(evaluate_thresholds(50.0, &entries).icon_color.is_some());
        // Outside range (too high)
        assert!(evaluate_thresholds(80.0, &entries).icon_color.is_none());
        // Outside range (too low)
        assert!(evaluate_thresholds(20.0, &entries).icon_color.is_none());
    }

    #[test]
    fn no_conditions_never_matches() {
        let entries = vec![ThresholdEntry {
            above: None,
            below: None,
            icon_color: Some(ColorValue::Token(CssToken::StatusError)),
            label_color: None,
            icon_bg_color: None,
            button_bg_color: None,
            border_color: None,
        }];
        let result = evaluate_thresholds(50.0, &entries);
        assert!(result.is_empty());
    }
}
