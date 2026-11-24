//! Shell command execution utilities.

use std::io;

use tokio::process::Command;
use tracing::warn;

/// Spawns a shell command, discarding stdout but logging failures.
fn spawn_quiet(cmd: &str) -> io::Result<()> {
    use std::process::Stdio;

    let child = Command::new("/bin/sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;

    let cmd = cmd.to_owned();
    tokio::spawn(async move {
        match child.wait_with_output().await {
            Ok(output) if !output.status.success() => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stderr = stderr.trim();

                if stderr.is_empty() {
                    warn!(cmd = %cmd, exit_code = ?output.status.code(), "command failed");
                } else {
                    warn!(
                        cmd = %cmd,
                        exit_code = ?output.status.code(),
                        stderr = %stderr,
                        "command failed"
                    );
                }
            }

            Err(err) => {
                warn!(cmd = %cmd, error = %err, "cannot wait on command");
            }

            Ok(_) => {}
        }
    });

    Ok(())
}

/// Runs a shell command if non-empty, logging failures.
pub fn run_if_set(cmd: &str) {
    if cmd.is_empty() {
        return;
    }

    if let Err(err) = spawn_quiet(cmd) {
        tracing::error!(error = %err, cmd = %cmd, "cannot spawn command");
    }
}
