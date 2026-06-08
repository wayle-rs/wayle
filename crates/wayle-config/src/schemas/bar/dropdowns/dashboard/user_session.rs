use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One action the dashboard session actions
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum SessionAction {
    /// Lock the session
    #[serde(rename = "lock")]
    Lock,
    /// Logout of the current session
    #[serde(rename = "log-out")]
    Logout,
    /// Reboot the machine
    #[serde(rename = "reboot")]
    Reboot,
    /// Power off the machine
    #[serde(rename = "power-off")]
    PowerOff,
}
