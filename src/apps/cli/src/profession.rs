use super::*;

#[derive(Subcommand)]
pub(super) enum ProfessionCommand {
    /// List Professions accepted from paired Agency catalogs.
    List,
    /// Accept one exact Profession from a previously accepted Agency catalog.
    Accept {
        #[arg(long)]
        binding_id: String,
        /// JSON file holding the catalog's exact `FrozenRef`, never a display name.
        #[arg(long)]
        reference: PathBuf,
        #[arg(long)]
        key: String,
    },
}

pub(super) async fn dispatch(
    command: ProfessionCommand,
    root: &Path,
    as_json: bool,
) -> Result<(), String> {
    let result = execute(command, root, as_json).await;
    if let Err(error) = &result {
        print_out(
            as_json,
            json!({"error":{"code":"PROFESSION_COMMAND_FAILED","message":error,"recovery_action":"inspect_error_and_retry"}}),
        );
    }
    result
}
async fn execute(command: ProfessionCommand, root: &Path, as_json: bool) -> Result<(), String> {
    match command {
        ProfessionCommand::List => {
            task::query_task(root, as_json, "profession.list", json!({})).await
        }
        ProfessionCommand::Accept {
            binding_id,
            reference,
            key,
        } => {
            let reference: Value = serde_json::from_slice(&std::fs::read(reference).map_err(io)?)
                .map_err(|e| e.to_string())?;
            keyed_submit(
                root,
                as_json,
                "profession.accept",
                json!({"binding_id":binding_id,"profession":reference}),
                key,
            )
            .await
        }
    }
}
