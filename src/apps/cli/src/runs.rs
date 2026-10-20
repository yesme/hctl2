use super::*;
#[derive(clap::Args)]
pub(super) struct Write {
    #[arg(long)]
    input: PathBuf,
    #[arg(long)]
    key: String,
    #[arg(long)]
    preview_token: Option<String>,
}
#[derive(Subcommand)]
pub(super) enum RunCommand {
    List {
        project_id: String,
    },
    Show {
        project_id: String,
        run_id: String,
    },
    Preview {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        key: String,
    },
    Start(Write),
    Pause(Write),
    Resume(Write),
    Cancel(Write),
    Replace(Write),
    Place(Write),
}
pub(super) async fn dispatch(
    command: RunCommand,
    root: &Path,
    as_json: bool,
) -> Result<(), String> {
    let result = execute(command, root, as_json).await;
    if let Err(error) = &result {
        print_out(
            as_json,
            json!({"error":{"code":"RUN_COMMAND_FAILED","message":error,"recovery_action":"inspect_error_and_retry"}}),
        );
    }
    result
}
async fn execute(command: RunCommand, root: &Path, as_json: bool) -> Result<(), String> {
    let (kind, write) = match command {
        RunCommand::List { project_id } => {
            return task::query_task(root, as_json, "run.list", json!({"project_id":project_id}))
                .await;
        }
        RunCommand::Show { project_id, run_id } => {
            return task::query_task(
                root,
                as_json,
                "run.show",
                json!({"project_id":project_id,"run_id":run_id}),
            )
            .await;
        }
        RunCommand::Preview { input, key } => (
            "start",
            Write {
                input,
                key,
                preview_token: None,
            },
        ),
        RunCommand::Start(w) => ("start", w),
        RunCommand::Pause(w) => ("pause", w),
        RunCommand::Resume(w) => ("resume", w),
        RunCommand::Cancel(w) => ("cancel", w),
        RunCommand::Replace(w) => ("replace", w),
        RunCommand::Place(w) => ("place", w),
    };
    let mut action: Value = serde_json::from_slice(&std::fs::read(&write.input).map_err(io)?)
        .map_err(|e| e.to_string())?;
    let object = action
        .as_object_mut()
        .ok_or("Run input must be an action object")?;
    if object.get("kind").is_some_and(|v| v.as_str() != Some(kind)) {
        return Err("Run action disagrees with subcommand".into());
    }
    object.insert("kind".into(), json!(kind));
    let payload = json!({"key":write.key,"action":action})
        .to_string()
        .into_bytes();
    let operation = format!("run.{kind}");
    let command_id = format!("run:{}", write.key);
    let mut client = client(root).await?;
    if let Some(preview_token) = write.preview_token {
        let r = client
            .submit(SubmitRequest {
                protocol: Some(Protocol {
                    version: PROTOCOL.into(),
                }),
                operation,
                payload,
                command_id,
                idempotency_key: write.key,
                preview_token,
            })
            .await
            .map_err(|e| e.to_string())?
            .into_inner();
        task::present(r.error, as_json);
        print_out(as_json, bytes_json(&r.result)?);
    } else {
        let r = client
            .preview(PreviewRequest {
                protocol: Some(Protocol {
                    version: PROTOCOL.into(),
                }),
                operation,
                payload,
                command_id,
            })
            .await
            .map_err(|e| e.to_string())?
            .into_inner();
        task::present(r.error, as_json);
        print_out(
            as_json,
            json!({"preview_token":r.preview_token,"effect_summary":bytes_json(&r.effect_summary)?}),
        );
    }
    Ok(())
}
