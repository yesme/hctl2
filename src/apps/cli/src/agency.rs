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
        print_out(
            as_json,
            json!({"error":{"code":"AGENCY_COMMAND_FAILED","message":error,"recovery_action":"inspect_error_and_retry"}}),
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
    let response = client(root)
        .await?
        .submit(SubmitRequest {
            protocol: Some(Protocol {
                version: PROTOCOL.into(),
            }),
            operation: operation.into(),
            payload: payload.to_string().into_bytes(),
            command_id: format!("{operation}:{key}"),
            idempotency_key: key,
            preview_token: String::new(),
        })
        .await
        .map_err(|e| e.to_string())?
        .into_inner();
    if let Some(error) = response.error {
        print_out(
            as_json,
            json!({"error":{"code":error.code,"message":error.message,"recovery_action":error.recovery_action}}),
        );
        std::process::exit(1);
    }
    print_out(as_json, bytes_json(&response.result)?);
    Ok(())
}
