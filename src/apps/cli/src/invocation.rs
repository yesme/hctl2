use super::*;

#[derive(clap::Args)]
pub(super) struct Input {
    #[arg(long)]
    input: PathBuf,
    #[arg(long)]
    key: String,
}
#[derive(Subcommand)]
pub(super) enum InvocationCommand {
    Preview(Input),
    Start {
        #[command(flatten)]
        input: Input,
        #[arg(long)]
        preview_token: String,
    },
    Show {
        project_id: String,
        invocation_id: String,
    },
}

pub(super) async fn dispatch(
    command: InvocationCommand,
    root: &Path,
    as_json: bool,
) -> Result<(), String> {
    let result = execute(command, root, as_json).await;
    if let Err(error) = &result {
        print_out(
            as_json,
            json!({"error":{"code":"INVOCATION_COMMAND_FAILED","message":error,"recovery_action":"inspect_error_and_retry"}}),
        );
    }
    result
}
async fn execute(command: InvocationCommand, root: &Path, as_json: bool) -> Result<(), String> {
    let (input, token) = match command {
        InvocationCommand::Preview(input) => (input, None),
        InvocationCommand::Start {
            input,
            preview_token,
        } => (input, Some(preview_token)),
        InvocationCommand::Show {
            project_id,
            invocation_id,
        } => {
            return task::query_task(
                root,
                as_json,
                "invocation.show",
                json!({"project_id":project_id,"invocation_id":invocation_id}),
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
    let command_id = format!("invocation:{}", input.key);
    if let Some(preview_token) = token {
        let result = client
            .submit(SubmitRequest {
                protocol: Some(Protocol {
                    version: PROTOCOL.into(),
                }),
                operation: "invocation.start".into(),
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
                operation: "invocation.start".into(),
                payload,
                command_id,
            })
            .await
            .map_err(|e| e.to_string())?
            .into_inner();
        task::present(result.error, as_json);
        print_out(
            as_json,
            json!({"preview_token":result.preview_token,"effect_summary":bytes_json(&result.effect_summary)?}),
        );
    }
    Ok(())
}
