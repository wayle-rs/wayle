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
                "updates dropdown cmd `{}` → stdout={:?} stderr={:?} count={}",
                command, trimmed, stderr.trim(), count
            );
            count
        }
        Err(e) => {
            tracing::warn!("updates dropdown check command failed: {e}");
            0
        }
    }
}

/// Spawn the update command in the user's terminal.
pub(super) fn spawn_update_in_terminal(update_command: &str) {
    let cmd = update_command.to_string();
    tokio::spawn(async move {
        // Try common terminals in order of preference
        let terminals = [
            ("kitty", vec!["-e", "sh", "-c"]),
            ("foot", vec!["-e", "sh", "-c"]),
            ("alacritty", vec!["-e", "sh", "-c"]),
            ("wezterm", vec!["start", "--", "sh", "-c"]),
            ("xterm", vec!["-e", "sh", "-c"]),
        ];

        for (term, args) in &terminals {
            if let Ok(path) = tokio::process::Command::new("which")
                .arg(term)
                .output()
                .await
            {
                if path.status.success() {
                    let mut full_args: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();
                    full_args.push(&cmd);
                    let _ = tokio::process::Command::new(term)
                        .args(&full_args)
                        .spawn();
                    return;
                }
            }
        }

        tracing::warn!("no terminal emulator found to run update command");
    });
}
