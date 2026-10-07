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
    /// List this Project's Invocations with their authorization and state versions.
    List {
        project_id: String,
    },
    /// Revoke the original authorization and queue its stop through `End`.
    Cancel {
        /// JSON file: `{"project_id","invocation_id","state_version","reason"}`.
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        key: String,
        #[arg(long)]
        preview_token: Option<String>,
    },
    /// Start a new Invocation naming the exact original as `retry_of`.
    Retry {
        #[command(flatten)]
        input: Input,
        /// JSON file holding the original's exact `owner` reference.
        #[arg(long)]
        retry_of: PathBuf,
        #[arg(long)]
        preview_token: Option<String>,
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
    let (operation, file, key, retry_of, token) = match command {
        InvocationCommand::Preview(input) => {
            ("invocation.start", input.input, input.key, None, None)
        }
        InvocationCommand::Start {
            input,
            preview_token,
        } => (
            "invocation.start",
            input.input,
            input.key,
            None,
            Some(preview_token),
        ),
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
        InvocationCommand::List { project_id } => {
            return task::query_task(
                root,
                as_json,
                "invocation.list",
                json!({"project_id":project_id}),
            )
            .await;
        }
        InvocationCommand::Cancel {
            input,
            key,
            preview_token,
        } => ("invocation.cancel", input, key, None, preview_token),
        InvocationCommand::Retry {
            input,
            retry_of,
            preview_token,
        } => (
            "invocation.start",
            input.input,
            input.key,
            Some(retry_of),
            preview_token,
        ),
    };
    let mut payload = read_json(&file)?;
    let object = payload.as_object_mut().ok_or("input must be an object")?;
    if object.get("key").is_some_and(|v| v.as_str() != Some(&key)) {
        return Err("input key differs".into());
    }
    object.insert("key".into(), json!(key));
    if let Some(retry_of) = retry_of {
        // The flag carries the original's exact owner reference; an input file left
        // over from the original call still says `retry_of: null`, which is no claim.
        let original = read_json(&retry_of)?;
        match object.get("retry_of") {
            None | Some(Value::Null) => {}
            Some(claimed) if *claimed == original => {}
            Some(_) => return Err("input retry_of differs from --retry-of".into()),
        }
        object.insert("retry_of".into(), original);
    }
    if operation == "invocation.cancel" {
        // A human cancels. Failure and loss stay reducer decisions under the
        // original authorization and are not client commands.
        match object.get("outcome") {
            None => {
                object.insert("outcome".into(), json!("cancelled"));
            }
            Some(outcome) if outcome.as_str() == Some("cancelled") => {}
            Some(_) => return Err("only a cancelled outcome is a client command".into()),
        }
    }
    let payload = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
    let mut client = client(root).await?;
    let command_id = format!("invocation:{}", key);
    if let Some(preview_token) = token {
        let result = client
            .submit(SubmitRequest {
                protocol: Some(Protocol {
                    version: PROTOCOL.into(),
                }),
                operation: operation.into(),
                payload,
                command_id,
                idempotency_key: key,
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
                operation: operation.into(),
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

fn read_json(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&std::fs::read(path).map_err(io)?).map_err(|e| e.to_string())
}
