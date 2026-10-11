use super::*;

#[derive(Subcommand)]
pub(super) enum ChangeSetCommand {
    /// Retain exact Git bytes, then confirm their admission as an independent human command.
    Seal {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        key: String,
        #[arg(long)]
        preview_token: Option<String>,
    },
    /// Preview and confirm a human seal of a revoked writer's residual in this ChangeSet.
    Takeover {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        key: String,
        #[arg(long)]
        preview_token: Option<String>,
    },
    /// Preview and confirm adoption of preserved bytes into a different ChangeSet.
    Adopt {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        key: String,
        #[arg(long)]
        preview_token: Option<String>,
    },
    /// Preview the exact residual tree before explicitly discarding its P1 worktree.
    Discard {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        key: String,
        #[arg(long)]
        preview_token: Option<String>,
    },
    /// Show a ChangeSet, its lease and admitted immutable versions.
    Show {
        repo_id: String,
        change_set_id: String,
    },
    /// Read the exact base-to-tree diff of one admitted revision.
    Diff {
        repo_id: String,
        change_set_id: String,
        revision_id: String,
    },
}

pub(super) async fn dispatch(
    command: ChangeSetCommand,
    root: &Path,
    as_json: bool,
) -> Result<(), String> {
    let result = execute(command, root, as_json).await;
    if let Err(error) = &result {
        print_out(
            as_json,
            json!({"error":{"code":"CHANGESET_COMMAND_FAILED","message":error,"recovery_action":"inspect_error_and_retry"}}),
        );
    }
    result
}

async fn execute(command: ChangeSetCommand, root: &Path, as_json: bool) -> Result<(), String> {
    let operation = match &command {
        ChangeSetCommand::Takeover { .. } => "changeset.takeover",
        ChangeSetCommand::Adopt { .. } => "changeset.adopt",
        ChangeSetCommand::Discard { .. } => "changeset.discard",
        _ => "changeset.seal",
    };
    match command {
        ChangeSetCommand::Seal {
            input,
            key,
            preview_token,
        }
        | ChangeSetCommand::Takeover {
            input,
            key,
            preview_token,
        }
        | ChangeSetCommand::Adopt {
            input,
            key,
            preview_token,
        }
        | ChangeSetCommand::Discard {
            input,
            key,
            preview_token,
        } => {
            let mut payload: Value = serde_json::from_slice(&std::fs::read(input).map_err(io)?)
                .map_err(|e| e.to_string())?;
            let object = payload.as_object_mut().ok_or("input must be an object")?;
            if object.get("key").is_some_and(|v| v.as_str() != Some(&key)) {
                return Err("input key differs".into());
            }
            object.insert("key".into(), json!(key));
            let payload = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
            let mut client = client(root).await?;
            let command_id = if operation == "changeset.seal" {
                format!("changeset:{key}")
            } else {
                format!("{operation}:{key}").replacen("changeset.", "changeset:", 1)
            };
            if let Some(preview_token) = preview_token {
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
        ChangeSetCommand::Show {
            repo_id,
            change_set_id,
        } => {
            task::query_task(
                root,
                as_json,
                "changeset.show",
                json!({"repo_id":repo_id,"change_set_id":change_set_id}),
            )
            .await
        }
        ChangeSetCommand::Diff {
            repo_id,
            change_set_id,
            revision_id,
        } => {
            task::query_task(
                root,
                as_json,
                "changeset.diff",
                json!({"repo_id":repo_id,"change_set_id":change_set_id,"revision_id":revision_id}),
            )
            .await
        }
    }
}
