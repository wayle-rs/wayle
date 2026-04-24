use std::sync::Arc;

use relm4::ComponentSender;
use wayle_config::schemas::modules::AuroraConfig;

pub(super) fn spawn_watchers(
    sender: &ComponentSender<super::AuroraModule>,
    config: &AuroraConfig,
    _weather: &Arc<wayle_weather::WeatherService>,
) {
    // Use the configured refresh interval (seconds) – for simplicity we fetch once at startup.
    let out = (*sender).command_sender().clone();
    if let Ok(resp) = reqwest::blocking::get("https://services.swpc.noaa.gov/json/ovation_aurora_latest.json") {
        if let Ok(text) = resp.text() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(coords) = json.get("coordinates").and_then(|c| c.as_array()) {
                    let max_val = coords
                        .iter()
                        .filter_map(|arr| arr.get(2))
                        .filter_map(|v| v.as_u64())
                        .max()
                        .unwrap_or(0);
                    let label = format!("{}", max_val);
                    let _ = out.send(super::messages::AuroraCmd::UpdateLabel(label));
                }
            }
        }
    }
}
