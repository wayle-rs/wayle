use std::{process::Stdio, time::Duration};

use relm4::ComponentSender;
use tokio::{process::Command, time::timeout};
use tokio_util::sync::CancellationToken;
use tracing::warn;
use wayle_config::schemas::modules::CustomModuleDefinition;

use super::super::{CustomModule, messages::CustomCmd};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) fn run_definition_command(
    sender: &ComponentSender<CustomModule>,
    definition: &CustomModuleDefinition,
    cancel_token: CancellationToken,
) {
    let Some(command) = definition.command.clone() else {
        return;
    };

    run_command_async(sender, &definition.id, command, cancel_token);
}

/// Runs a command asynchronously with timeout and single-flight cancellation.
///
/// If `cancel_token` is triggered, the command is cancelled.
/// Reset the token before calling to cancel any in-flight command.
pub(crate) fn run_command_async(
    sender: &ComponentSender<CustomModule>,
    module_id: &str,
    command: String,
    cancel_token: CancellationToken,
) {
    let module_id = module_id.to_string();
    sender.oneshot_command(async move {
        let outcome = tokio::select! {
            biased;
            () = cancel_token.cancelled() => ExecOutcome::Cancelled,
            result = timeout(COMMAND_TIMEOUT, run_command(&command)) => match result {
                Ok(Ok(output)) => ExecOutcome::Output(output),
                Ok(Err(error)) => ExecOutcome::Failed(error),
                Err(_) => ExecOutcome::TimedOut,
            },
        };

        map_exec_outcome(&module_id, outcome)
    });
}

enum ExecOutcome {
    Output(String),
    Cancelled,
    TimedOut,
    Failed(std::io::Error),
}

fn map_exec_outcome(module_id: &str, outcome: ExecOutcome) -> CustomCmd {
    match outcome {
        ExecOutcome::Output(output) => CustomCmd::CommandOutput(output),
        ExecOutcome::Cancelled => CustomCmd::CommandCancelled,
        ExecOutcome::TimedOut => {
            warn!(
                module_id = %module_id,
                timeout_secs = COMMAND_TIMEOUT.as_secs(),
                "command timed out"
            );
            CustomCmd::CommandCancelled
        }
        ExecOutcome::Failed(error) => {
            warn!(module_id = %module_id, error = %error, "command execution failed");
            CustomCmd::CommandCancelled
        }
    }
}

async fn run_command(command: &str) -> Result<String, std::io::Error> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .output()
        .await?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_exec_outcome_output() {
        let cmd = map_exec_outcome("test", ExecOutcome::Output(String::from("ok")));
        assert!(matches!(cmd, CustomCmd::CommandOutput(output) if output == "ok"));
    }

    #[test]
    fn map_exec_outcome_cancelled() {
        let cmd = map_exec_outcome("test", ExecOutcome::Cancelled);
        assert!(matches!(cmd, CustomCmd::CommandCancelled));
    }

    #[test]
    fn map_exec_outcome_timeout() {
        let cmd = map_exec_outcome("test", ExecOutcome::TimedOut);
        assert!(matches!(cmd, CustomCmd::CommandCancelled));
    }

    #[test]
    fn map_exec_outcome_failed() {
        let error = std::io::Error::other("boom");
        let cmd = map_exec_outcome("test", ExecOutcome::Failed(error));
        assert!(matches!(cmd, CustomCmd::CommandCancelled));
    }
}
