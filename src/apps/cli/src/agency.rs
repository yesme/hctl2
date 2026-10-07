use super::*;

#[derive(Subcommand)]
pub(super) enum AgencyCommand {
    /// Consume the independent local Agency and freeze its public catalog.
    Pair {
        #[arg(long)]
        binding_id: String,
        #[arg(long)]
        agency_root: Option<PathBuf>,
        #[arg(long)]
        key: String,
    },
    Bindings,
    Catalog {
        binding_id: String,
    },
    /// Accept one exact Profession from a previously accepted Agency catalog.
    Accept {
        #[arg(long)]
        binding_id: String,
        #[arg(long)]
        reference: PathBuf,
        #[arg(long)]
        key: String,
    },
}

pub(super) async fn dispatch(
    command: AgencyCommand,
    root: &Path,
    as_json: bool,
) -> Result<(), String> {
    let result = execute(command, root, as_json).await;
    if let Err(error) = &result {
        print_error(
            as_json,
            "AGENCY_COMMAND_FAILED",
            error,
            "inspect_error_and_retry",
        );
    }
    result
}
async fn execute(command: AgencyCommand, root: &Path, as_json: bool) -> Result<(), String> {
    let (operation, payload, key) = match command {
        AgencyCommand::Bindings => return query(root, as_json, "agency.bindings", json!({})).await,
        AgencyCommand::Catalog { binding_id } => {
            return query(
                root,
                as_json,
                "agency.catalog",
                json!({"binding_id":binding_id}),
            )
            .await;
        }
        AgencyCommand::Pair {
            binding_id,
            agency_root,
            key,
        } => (
            "agency.pair",
            json!({"binding_id":binding_id,"agency_root":agency_root}),
            key,
        ),
        AgencyCommand::Accept {
            binding_id,
            reference,
            key,
        } => {
            let reference: Value = serde_json::from_slice(&std::fs::read(reference).map_err(io)?)
                .map_err(|e| e.to_string())?;
            (
                "profession.accept",
                json!({"binding_id":binding_id,"profession":reference}),
                key,
            )
        }
    };
    keyed_submit(root, as_json, operation, payload, key).await
}
