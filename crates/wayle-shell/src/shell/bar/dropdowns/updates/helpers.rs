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
/// Returns false if an update is already running.
pub(super) fn spawn_update_in_terminal(update_command: &str) -> bool {
    use std::sync::atomic::{AtomicBool, Ordering};
    static RUNNING: AtomicBool = AtomicBool::new(false);

    if RUNNING.swap(true, Ordering::SeqCst) {
        tracing::debug!("update already running, ignoring click");
        return false;
    }

    // Wrap command so terminal stays open after finish/failure
    let wrapped = format!(
        "{cmd}; echo ''; echo 'Press enter to close...'; read",
        cmd = update_command
    );

    let terminals = [
        ("kitty", vec!["-e", "sh", "-c"]),
        ("foot", vec!["-e", "sh", "-c"]),
        ("alacritty", vec!["-e", "sh", "-c"]),
        ("wezterm", vec!["start", "--", "sh", "-c"]),
        ("xterm", vec!["-e", "sh", "-c"]),
    ];

    tokio::spawn(async move {
        for (term, args) in &terminals {
            if let Ok(check) = tokio::process::Command::new("which")
                .arg(term)
                .output()
                .await
            {
                if check.status.success() {
                    let mut full_args: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();
                    full_args.push(&wrapped);

                    match tokio::process::Command::new(term)
                        .args(&full_args)
                        .spawn()
                    {
                        Ok(mut child) => {
                            // Wait for terminal to close before allowing another
                            let _ = child.wait().await;
                            RUNNING.store(false, Ordering::SeqCst);
                            return;
                        }
                        Err(e) => {
                            tracing::warn!("failed to spawn {term}: {e}");
                        }
                    }
                }
            }
        }

        tracing::warn!("no terminal emulator found to run update command");
        RUNNING.store(false, Ordering::SeqCst);
    });

    true
}
