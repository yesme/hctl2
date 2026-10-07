use super::*;
use crate::task::{Write, write};

#[derive(Subcommand)]
pub(super) enum ProfileCommand {
    /// Preview creation; pass the returned token to confirm the exact Profile.
    Create(Write),
    /// Preview an update of the exact pointer version; the token confirms it.
    Update(Write),
    /// Show the exact revision the current pointer names. Grants nothing.
    Show { profile_id: String },
}

pub(super) async fn dispatch(
    command: ProfileCommand,
    root: &Path,
    as_json: bool,
) -> Result<(), String> {
    let result = match command {
        ProfileCommand::Create(input) => write(root, as_json, "profile", "create", input).await,
        ProfileCommand::Update(input) => write(root, as_json, "profile", "update", input).await,
        ProfileCommand::Show { profile_id } => {
            task::query_task(
                root,
                as_json,
                "profile.show",
                json!({"profile_id":profile_id}),
            )
            .await
        }
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
