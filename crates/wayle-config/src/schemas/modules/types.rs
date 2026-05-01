use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Time display format.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    wayle_derive::EnumVariants,
)]
#[serde(rename_all = "lowercase")]
pub enum TimeFormat {
    /// 12-hour format with AM/PM (e.g., "6:30 AM").
    #[default]
    #[serde(rename = "12h")]
    TwelveHour,
    /// 24-hour format (e.g., "06:30").
    #[serde(rename = "24h")]
    TwentyFourHour,
}
