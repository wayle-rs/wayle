use std::{
    process::{ExitStatus, Stdio},
    time::Duration,
};

use relm4::ComponentSender;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};
use wayle_config::schemas::modules::{CustomModuleDefinition, RestartPolicy};

use super::super::{CustomModule, messages::CustomCmd};

/// Spawns a long-running child process and restarts it according to the
/// configured [`RestartPolicy`].
///
/// On each exit the policy is checked: `Never` stops unconditionally,
/// `OnFailure` stops on clean exits, and `OnExit` always restarts.
/// Before restarting, the loop waits the user-configured
/// `restart-interval-ms`.
///
/// The loop exits when the `token` is cancelled, the component shuts
/// down, or the restart policy denies a restart.
pub(crate) fn spawn_command_watcher(
    sender: &ComponentSender<CustomModule>,
    definition: &CustomModuleDefinition,
    token: CancellationToken,
) {
    let Some(command) = definition.command.clone() else {
        return;
    };

    let module_id = definition.id.clone();
    let policy = definition.restart_policy;
    let delay_ms = definition.restart_interval_ms.value();

    sender.command(move |out, shutdown| async move {
        loop {
            if token.is_cancelled() {
                debug!(module_id = %module_id, "watch command stopped (config changed)");
                return;
            }

            let run_outcome = run_child(&command, &out, &shutdown, &token, &module_id).await;

            let RunOutcome::Exited(exit_status) = run_outcome else {
                return;
            };

            debug!(
                module_id = %module_id,
                exit_code = ?exit_status.code(),
                "watch command exited"
            );

            if !should_restart(policy, exit_status) {
                debug!(
                    module_id = %module_id,
                    policy = ?policy,
                    "watch command will not restart due to restart-policy"
                );
                return;
            }

            debug!(
                module_id = %module_id,
                policy = ?policy,
                delay_ms,
                "restarting watch command after exit"
            );

            let delay_expired = tokio::select! {
                () = shutdown.clone().wait() => false,
                () = token.cancelled() => false,
                () = tokio::time::sleep(Duration::from_millis(delay_ms)) => true,
            };

            if !delay_expired {
                return;
            }
        }
    });
}

enum RunOutcome {
    Stop,
    Exited(ExitStatus),
}

#[allow(clippy::cognitive_complexity)]
async fn run_child(
    command: &str,
    out: &relm4::Sender<CustomCmd>,
    shutdown: &relm4::ShutdownReceiver,
    token: &CancellationToken,
    module_id: &str,
) -> RunOutcome {
    let mut child = match spawn_child(command) {
        Ok(child) => child,
        Err(error) => {
            warn!(module_id = %module_id, error = %error, "failed to spawn watch command");
            return RunOutcome::Stop;
        }
    };

    debug!(module_id = %module_id, "watch command started");

    let Some(stdout) = child.stdout.take() else {
        warn!(module_id = %module_id, "watch command started without stdout");
        terminate_child(&mut child).await;
        return RunOutcome::Stop;
    };

    let stopped = read_stdout_until_stop(stdout, out, shutdown, token, module_id).await;

    if stopped {
        terminate_child(&mut child).await;
        return RunOutcome::Stop;
    }

    match child.wait().await {
        Ok(status) => RunOutcome::Exited(status),
        Err(error) => {
            warn!(
                module_id = %module_id,
                error = %error,
                "failed waiting for watch command exit"
            );
            RunOutcome::Stop
        }
    }
}

fn spawn_child(command: &str) -> Result<tokio::process::Child, std::io::Error> {
    Command::new("/bin/sh")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
}

/// Reads stdout lines until the process exits or a stop signal is received.
///
/// Returns `true` if stopped by shutdown/cancellation, `false` if the process
/// exited on its own.
async fn read_stdout_until_stop(
    stdout: tokio::process::ChildStdout,
    out: &relm4::Sender<CustomCmd>,
    shutdown: &relm4::ShutdownReceiver,
    token: &CancellationToken,
    module_id: &str,
) -> bool {
    let mut reader = BufReader::new(stdout).lines();

    tokio::select! {
        () = shutdown.clone().wait() => {
            debug!(module_id = %module_id, "watch command stopped (shutdown)");
            true
        }
        () = token.cancelled() => {
            debug!(module_id = %module_id, "watch command stopped (config changed)");
            true
        }
        () = async {
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = out.send(CustomCmd::WatchOutput(line));
            }
        } => {
            false
        }
    }
}

fn should_restart(policy: RestartPolicy, exit_status: ExitStatus) -> bool {
    match policy {
        RestartPolicy::Never => false,
        RestartPolicy::OnExit => true,
        RestartPolicy::OnFailure => !exit_status.success(),
    }
}

async fn terminate_child(child: &mut tokio::process::Child) {
    let _ = child.kill().await;
    let _ = child.wait().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_restart_never_returns_false() {
        let status = exit_status(1);
        assert!(!should_restart(RestartPolicy::Never, status));
    }

    #[test]
    fn should_restart_on_exit_returns_true_for_success_and_failure() {
        assert!(should_restart(RestartPolicy::OnExit, exit_status(0)));
        assert!(should_restart(RestartPolicy::OnExit, exit_status(1)));
    }

    #[test]
    fn should_restart_on_failure_returns_true_only_for_failures() {
        assert!(!should_restart(RestartPolicy::OnFailure, exit_status(0)));
        assert!(should_restart(RestartPolicy::OnFailure, exit_status(1)));
    }

    fn exit_status(code: i32) -> ExitStatus {
        std::process::Command::new("/bin/sh")
            .arg("-c")
            .arg(format!("exit {code}"))
            .status()
            .expect("failed to run shell to produce exit status")
    }
}
