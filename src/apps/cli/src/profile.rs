use super::*;
use crate::task::{Write, write};

#[derive(Subcommand)]
pub(super) enum ProfileCommand {
    /// Preview creation; pass the returned token to confirm the exact Profile.
    Create(Write),
}

pub(super) async fn dispatch(
    command: ProfileCommand,
    root: &Path,
    as_json: bool,
) -> Result<(), String> {
    let result = match command {
        ProfileCommand::Create(input) => write(root, as_json, "profile", "create", input).await,
    };
    if let Err(error) = &result {
        print_out(
            as_json,
            json!({"error":{"code":"PROFILE_COMMAND_FAILED", "message":error,
            "recovery_action":"inspect_error_and_retry"}}),
        );
    }
    result
}
