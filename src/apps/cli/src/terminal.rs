use super::*;

#[derive(Subcommand)]
pub(super) enum TerminalCommand {
    /// Read what the Agency reports for the original dispatch; records nothing.
    Inspect {
        project_id: String,
        invocation_id: String,
        /// Observation cursor; defaults to the beginning.
        #[arg(long)]
        after: Option<u64>,
    },
    /// Replay stored observations for the original dispatch, without Agency I/O.
    Replay {
        project_id: String,
        invocation_id: String,
    },
    /// Managed input has no public entry; this reports the typed refusal.
    Attach {
        project_id: String,
        invocation_id: String,
    },
}

pub(super) async fn dispatch(
    command: TerminalCommand,
    root: &Path,
    as_json: bool,
) -> Result<(), String> {
    let result = execute(command, root, as_json).await;
    if let Err(error) = &result {
        print_error(
            as_json,
            "TERMINAL_COMMAND_FAILED",
            error,
            "inspect_error_and_retry",
        );
    }
    result
}
async fn execute(command: TerminalCommand, root: &Path, as_json: bool) -> Result<(), String> {
    let (kind, payload) = match command {
        TerminalCommand::Inspect {
            project_id,
            invocation_id,
            after,
        } => (
            "terminal.inspect",
            json!({"project_id":project_id,"invocation_id":invocation_id,"after":after}),
        ),
        TerminalCommand::Replay {
            project_id,
            invocation_id,
        } => (
            "terminal.replay",
            json!({"project_id":project_id,"invocation_id":invocation_id}),
        ),
        TerminalCommand::Attach {
            project_id,
            invocation_id,
        } => (
            "terminal.attach",
            json!({"project_id":project_id,"invocation_id":invocation_id}),
        ),
    };
    task::query_task(root, as_json, kind, payload).await
}
