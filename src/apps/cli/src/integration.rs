use super::*;

#[derive(clap::Args)]
pub(super) struct Input {
    #[arg(long)]
    input: PathBuf,
    #[arg(long)]
    key: String,
}
#[derive(Subcommand)]
pub(super) enum IntegrationCommand {
    /// Freeze source, target, authorization form and protection; nothing is written.
    Preview(Input),
    /// Persist the previewed intent; execution and readback follow in the background.
    Submit {
        #[command(flatten)]
        input: Input,
        #[arg(long)]
        preview_token: String,
    },
    /// The intent, its attempts, and the Integration Receipt once readback confirmed it.
    Show {
        repo_id: String,
        intent_id: String,
    },
    List {
        repo_id: String,
    },
}

pub(super) async fn dispatch(
    command: IntegrationCommand,
    root: &Path,
    as_json: bool,
) -> Result<(), String> {
    let result = execute(command, root, as_json).await;
    if let Err(error) = &result {
        print_error(
            as_json,
            "INTEGRATION_COMMAND_FAILED",
            error,
            "inspect_error_and_retry",
        );
    }
    result
}
async fn execute(command: IntegrationCommand, root: &Path, as_json: bool) -> Result<(), String> {
    let (input, token) = match command {
        IntegrationCommand::Preview(input) => (input, None),
        IntegrationCommand::Submit {
            input,
            preview_token,
        } => (input, Some(preview_token)),
        IntegrationCommand::Show { repo_id, intent_id } => {
            return task::query_task(
                root,
                as_json,
                "integration.show",
                json!({"repo_id":repo_id,"intent_id":intent_id}),
            )
            .await;
        }
        IntegrationCommand::List { repo_id } => {
            return task::query_task(
                root,
                as_json,
                "integration.list",
                json!({"repo_id":repo_id}),
            )
            .await;
        }
    };
    let mut payload: Value = serde_json::from_slice(&std::fs::read(input.input).map_err(io)?)
        .map_err(|e| e.to_string())?;
    let object = payload.as_object_mut().ok_or("input must be an object")?;
    if object
        .get("key")
        .is_some_and(|v| v.as_str() != Some(&input.key))
    {
        return Err("input key differs".into());
    }
    object.insert("key".into(), json!(input.key));
    let payload = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
    let mut client = client(root).await?;
    let command_id = format!("integration:{}", input.key);
    if let Some(preview_token) = token {
        let result = client
            .submit(SubmitRequest {
                protocol: Some(Protocol {
                    version: PROTOCOL.into(),
                }),
                operation: "integration.submit".into(),
                payload,
                command_id,
                idempotency_key: input.key,
                preview_token,
            })
            .await
            .map_err(|e| e.to_string())?
            .into_inner();
        task::present(result.error, as_json);
        print_out(as_json, bytes_json(&result.result)?);
    } else {
        let result = client
            .preview(PreviewRequest {
                protocol: Some(Protocol {
                    version: PROTOCOL.into(),
                }),
                operation: "integration.submit".into(),
                payload,
                command_id,
            })
            .await
            .map_err(|e| e.to_string())?
            .into_inner();
        task::present(result.error, as_json);
        if !as_json {
            println!(
                "{}",
                crate::render::integration_preview(
                    &result.preview_token,
                    &bytes_json(&result.effect_summary)?
                )
            );
        } else {
            print_out(
                as_json,
                json!({"preview_token":result.preview_token,"effect_summary":bytes_json(&result.effect_summary)?}),
            );
        }
    }
    Ok(())
}
