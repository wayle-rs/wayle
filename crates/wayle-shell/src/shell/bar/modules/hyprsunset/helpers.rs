use std::{env, io, path::PathBuf, str};

use serde_json::json;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    process::Command,
};
use tracing::debug;

use crate::i18n::t;

pub(crate) struct LabelContext<'a> {
    pub format: &'a str,
    pub temp: u32,
    pub gamma: u32,
    pub config_temp: u32,
    pub config_gamma: u32,
    pub enabled: bool,
}

pub(super) fn build_label(ctx: &LabelContext<'_>) -> String {
    let (status, temp, gamma) = if ctx.enabled {
        (
            t!("bar-hyprsunset-on"),
            ctx.temp.to_string(),
            ctx.gamma.to_string(),
        )
    } else {
        (
            t!("bar-hyprsunset-off"),
            String::from("--"),
            String::from("--"),
        )
    };

    let template_ctx = json!({
        "status": status,
        "temp": temp,
        "gamma": gamma,
        "config_temp": ctx.config_temp.to_string(),
        "config_gamma": ctx.config_gamma.to_string(),
    });
    crate::template::render(ctx.format, template_ctx).unwrap_or_default()
}

pub(super) fn select_icon(enabled: bool, icon_off: &str, icon_on: &str) -> String {
    if enabled { icon_on } else { icon_off }.to_string()
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct HyprsunsetState {
    pub temp: u32,
    pub gamma: u32,
}

pub(super) async fn query_state() -> Option<HyprsunsetState> {
    let socket_path = socket_path()?;

    let temp = query_value(&socket_path, "temperature").await?;
    let gamma = query_value(&socket_path, "gamma").await?;

    Some(HyprsunsetState { temp, gamma })
}

fn socket_path() -> Option<PathBuf> {
    let runtime_dir = env::var("XDG_RUNTIME_DIR").ok()?;
    let his = env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    Some(PathBuf::from(format!(
        "{runtime_dir}/hypr/{his}/.hyprsunset.sock"
    )))
}

async fn query_value(socket_path: &PathBuf, command: &str) -> Option<u32> {
    let response = send_socket_command(socket_path, command).await?;
    parse_numeric_response(&response, command)
}

async fn send_socket_command(socket_path: &PathBuf, command: &str) -> Option<String> {
    let mut stream = UnixStream::connect(socket_path).await.ok()?;

    stream.write_all(command.as_bytes()).await.ok()?;
    stream.shutdown().await.ok()?;

    let mut buf = [0u8; 32];
    let bytes_read = stream.read(&mut buf).await.ok()?;

    if bytes_read == 0 {
        return None;
    }

    str::from_utf8(&buf[..bytes_read]).ok().map(String::from)
}

fn parse_numeric_response(response: &str, command: &str) -> Option<u32> {
    match response.trim().parse::<f32>() {
        Ok(value) => Some(value.round() as u32),
        Err(err) => {
            debug!(error = %err, command, response = response.trim(), "float parse failed");
            None
        }
    }
}

pub(super) async fn start(overrides: Option<(u32, u32)>) -> io::Result<()> {
    start_command(overrides).spawn()?;
    Ok(())
}

fn start_command(overrides: Option<(u32, u32)>) -> Command {
    let mut command = Command::new("hyprsunset");

    if let Some((temperature, gamma)) = overrides {
        command
            .arg("-t")
            .arg(temperature.to_string())
            .arg("-g")
            .arg(gamma.to_string());
    }

    command
}

pub(super) async fn stop() -> io::Result<()> {
    Command::new("pkill")
        .arg("-x")
        .arg("hyprsunset")
        .output()
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx<'a>(format: &'a str, temp: u32, gamma: u32, enabled: bool) -> LabelContext<'a> {
        LabelContext {
            format,
            temp,
            gamma,
            config_temp: temp,
            config_gamma: gamma,
            enabled,
        }
    }

    #[test]
    fn build_label_with_temp_and_gamma() {
        assert_eq!(
            build_label(&ctx("{{ temp }}K {{ gamma }}%", 4500, 80, true)),
            "4500K 80%"
        );
    }

    #[test]
    fn build_label_temp_only() {
        assert_eq!(build_label(&ctx("{{ temp }}K", 5000, 100, true)), "5000K");
    }

    #[test]
    fn build_label_with_status_enabled() {
        assert_eq!(build_label(&ctx("{{ status }}", 4500, 100, true)), "On");
    }

    #[test]
    fn build_label_with_status_disabled() {
        assert_eq!(build_label(&ctx("{{ status }}", 4500, 100, false)), "Off");
    }

    #[test]
    fn build_label_temp_shows_dash_when_disabled() {
        assert_eq!(build_label(&ctx("{{ temp }}K", 4500, 100, false)), "--K");
    }

    #[test]
    fn build_label_gamma_shows_dash_when_disabled() {
        assert_eq!(build_label(&ctx("{{ gamma }}%", 4500, 80, false)), "--%");
    }

    #[test]
    fn select_icon_enabled() {
        assert_eq!(select_icon(true, "sun", "moon"), "moon");
    }

    #[test]
    fn select_icon_disabled() {
        assert_eq!(select_icon(false, "sun", "moon"), "sun");
    }

    #[test]
    fn start_command_with_overrides() {
        let command = start_command(Some((4500, 80)));
        let args: Vec<String> = command
            .as_std()
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        assert_eq!(args, ["-t", "4500", "-g", "80"]);
    }

    #[test]
    fn start_command_without_overrides() {
        let command = start_command(None);

        assert!(command.as_std().get_args().next().is_none());
    }

    #[test]
    fn build_label_with_config_temp_and_gamma() {
        let ctx = LabelContext {
            format: "{{ config_temp }}K ({{ config_gamma }}%)",
            temp: 4500,
            gamma: 80,
            config_temp: 5000,
            config_gamma: 100,
            enabled: true,
        };
        assert_eq!(build_label(&ctx), "5000K (100%)");
    }

    #[test]
    fn build_label_with_current_and_config() {
        let ctx = LabelContext {
            format: "{{ temp }}K -> {{ config_temp }}K",
            temp: 4000,
            gamma: 80,
            config_temp: 4500,
            config_gamma: 100,
            enabled: true,
        };
        assert_eq!(build_label(&ctx), "4000K -> 4500K");
    }
}
