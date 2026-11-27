use serde_json::json;
use std::sync::LazyLock;
use tokio::sync::Notify;

/// Signal for the bar module to re-poll update counts immediately.
pub(crate) static REFRESH_NOTIFY: LazyLock<Notify> = LazyLock::new(Notify::new);

/// Counts of available package updates.
#[derive(Debug, Clone, Default)]
pub(super) struct UpdateCounts {
    pub pacman: u32,
    pub aur: u32,
    pub flatpak: u32,
}

impl UpdateCounts {
    pub fn total(&self) -> u32 {
        self.pacman + self.aur + self.flatpak
    }
}

/// Run a shell command and parse the trimmed stdout as a u32 count.
pub(super) async fn run_count_command(command: &str) -> u32 {
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .await;

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let trimmed = stdout.trim();
            let count = trimmed.parse::<u32>().unwrap_or(0);
            tracing::debug!(
                "updates cmd `{}` → stdout={:?} stderr={:?} count={}",
                command, trimmed, stderr.trim(), count
            );
            count
        }
        Err(e) => {
            tracing::warn!("updates check command failed: {e}");
            0
        }
    }
}

/// Check for updates by running the configured commands.
pub(super) async fn check_updates(
    official_cmd: &str,
    aur_cmd: &str,
    flatpak_cmd: &str,
) -> UpdateCounts {
    let (pacman, aur, flatpak) = tokio::join!(
        run_count_command(official_cmd),
        run_count_command(aur_cmd),
        run_count_command(flatpak_cmd),
    );
    UpdateCounts { pacman, aur, flatpak }
}

/// Format the bar label using Jinja2 template syntax.
pub(super) fn format_label(format: &str, counts: &UpdateCounts) -> String {
    let ctx = json!({
        "pacman": counts.pacman,
        "aur": counts.aur,
        "flatpak": counts.flatpak,
        "total": counts.total(),
    });
    crate::template::render(format, ctx).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_label_total() {
        let counts = UpdateCounts { pacman: 145, aur: 24, flatpak: 3 };
        let result = format_label("{{ total }}", &counts);
        assert_eq!(result, "172");
    }

    #[test]
    fn format_label_breakdown() {
        let counts = UpdateCounts { pacman: 145, aur: 24, flatpak: 3 };
        let result = format_label("pac:{{ pacman }} aur:{{ aur }} fp:{{ flatpak }}", &counts);
        assert_eq!(result, "pac:145 aur:24 fp:3");
    }

    #[test]
    fn format_label_zero() {
        let counts = UpdateCounts::default();
        let result = format_label("{{ total }}", &counts);
        assert_eq!(result, "0");
    }

    #[test]
    fn update_counts_total() {
        let counts = UpdateCounts { pacman: 100, aur: 50, flatpak: 5 };
        assert_eq!(counts.total(), 155);
    }
}
