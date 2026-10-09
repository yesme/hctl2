use super::*;

#[derive(Subcommand)]
pub(super) enum RepoCommand {
    /// Without --preview-token, only inspect and preview. --key is stable across retries.
    Register {
        #[arg(long, group = "action", required_unless_present_any = ["resume", "confirm", "abandon"])]
        input: Option<PathBuf>,
        #[arg(long, group = "action")]
        resume: Option<String>,
        #[arg(long, group = "action")]
        confirm: Option<String>,
        #[arg(long, group = "action")]
        abandon: Option<String>,
        #[arg(long)]
        key: String,
        #[arg(long)]
        version: Option<i64>,
        #[arg(long)]
        platform_repo_id: Option<String>,
        #[arg(long)]
        preview_token: Option<String>,
    },
    List,
    Show {
        repo_id: String,
    },
    /// Ensure a local human has an ordinary account on the hosted platform and can
    /// collaborate on a registered Repo. Without --preview-token, only preview.
    Grant {
        #[arg(long)]
        repo_id: String,
        #[arg(long)]
        username: String,
        /// read or write. read is the least privilege that lets the human open issues.
        #[arg(long, default_value = "read")]
        permission: String,
        #[arg(long)]
        key: String,
        #[arg(long)]
        preview_token: Option<String>,
    },
}

pub(super) async fn dispatch(
    command: RepoCommand,
    root: &Path,
    as_json: bool,
) -> Result<(), String> {
    let result = execute(command, root, as_json).await;
    if let Err(error) = &result {
        // Observations and rejections remain machine-readable, including transport failures.
        print_error(
            as_json,
            "REPO_COMMAND_FAILED",
            error,
            "inspect_error_and_retry",
        );
    }
    result
}

async fn execute(command: RepoCommand, root: &Path, as_json: bool) -> Result<(), String> {
    match command {
        RepoCommand::List => repo_query(root, as_json, "repo.list", json!({})).await,
        RepoCommand::Show { repo_id } => {
            repo_query(root, as_json, "repo.show", json!({"repo_id":repo_id})).await
        }
        RepoCommand::Register {
            input,
            resume,
            confirm,
            abandon,
            key,
            version,
            platform_repo_id,
            preview_token,
        } => {
            let (operation, payload) = if let Some(input) = input {
                let request: Value = serde_json::from_slice(&std::fs::read(input).map_err(io)?)
                    .map_err(|e| e.to_string())?;
                (
                    "repo.register",
                    json!({"registration_key":key,"request":request}),
                )
            } else if let Some(id) = resume {
                ("repo.resume", json!({"repo_id":id}))
            } else if let Some(id) = confirm {
                (
                    "repo.confirm",
                    json!({"repo_id":id,"version":version,"platform_repo_id":platform_repo_id}),
                )
            } else {
                ("repo.abandon", json!({"repo_id":abandon,"version":version}))
            };
            write(root, as_json, operation, payload, key, preview_token).await
        }
        RepoCommand::Grant {
            repo_id,
            username,
            permission,
            key,
            preview_token,
        } => {
            write(
                root,
                as_json,
                "repo.grant",
                json!({"repo_id":repo_id,"username":username,"permission":permission}),
                key,
                preview_token,
            )
            .await
        }
    }
}

/// Every Repo write is the same command twice: without --preview-token it only previews,
/// with one it submits. No --yes shortcut, so the human reads effect_summary in between.
async fn write(
    root: &Path,
    as_json: bool,
    operation: &str,
    payload: Value,
    key: String,
    preview_token: Option<String>,
) -> Result<(), String> {
    let Some(token) = preview_token else {
        let response = client(root)
            .await?
            .preview(PreviewRequest {
                protocol: Some(Protocol {
                    version: PROTOCOL.into(),
                }),
                operation: operation.into(),
                payload: payload.to_string().into_bytes(),
                command_id: invocation_id("preview"),
            })
            .await
            .map_err(|e| e.to_string())?
            .into_inner();
        present_error(as_json, response.error);
        print_out(
            as_json,
            json!({"preview_token":response.preview_token,
            "dangerous":response.dangerous,"effect_summary":bytes_json(&response.effect_summary)?}),
        );
        return Ok(());
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
            preview_token: token,
        })
        .await
        .map_err(|e| e.to_string())?
        .into_inner();
    present_error(as_json, response.error);
    let result = bytes_json(&response.result)?;
    let failed = result.get("error").is_some();
    print_out(as_json, result);
    if failed {
        std::process::exit(1);
    }
    Ok(())
}

fn present_error(as_json: bool, error: Option<proto::Error>) {
    if let Some(error) = error {
        print_out(
            as_json,
            json!({"error":{"code":error.code,"message":error.message,"recovery_action":error.recovery_action}}),
        );
        // One JSON observation, preserving the control's stable error code and recovery action.
        std::process::exit(1);
    }
}

async fn repo_query(root: &Path, as_json: bool, kind: &str, payload: Value) -> Result<(), String> {
    let response = client(root)
        .await?
        .query(QueryRequest {
            protocol: Some(Protocol {
                version: PROTOCOL.into(),
            }),
            kind: kind.into(),
            payload: payload.to_string().into_bytes(),
        })
        .await
        .map_err(|e| e.to_string())?
        .into_inner();
    present_error(as_json, response.error);
    print_out(as_json, bytes_json(&response.payload)?);
    Ok(())
}
