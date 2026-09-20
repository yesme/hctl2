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
pub(super) enum TaskCommand {
    Connect(Write),
    SetActive(Write),
    Attach(Write),
    Claim(Write),
    Create(Write),
    Adopt(Write),
    Update(Write),
    Move(Write),
    Cancel(Write),
    DeleteCard(Write),
    Refresh(Write),
    Resume(Write),
    List {
        #[arg(long)]
        project_id: Option<String>,
    },
    Show {
        project_id: String,
        task_id: String,
    },
    Sources,
    Board {
        project_id: String,
        source_id: String,
    },
}

pub(super) async fn dispatch(
    command: TaskCommand,
    root: &Path,
    as_json: bool,
) -> Result<(), String> {
    let result = execute(command, root, as_json).await;
    if let Err(e) = &result {
        print_out(
            as_json,
            json!({"error":{"code":"TASK_COMMAND_FAILED","message":e,"recovery_action":"inspect_error_and_retry"}}),
        );
    }
    result
}
async fn execute(command: TaskCommand, root: &Path, as_json: bool) -> Result<(), String> {
    let (kind, w) = match command {
        TaskCommand::List { project_id } => {
            return query_task(root, as_json, "task.list", json!({"project_id":project_id})).await;
        }
        TaskCommand::Show {
            project_id,
            task_id,
        } => {
            return query_task(
                root,
                as_json,
                "task.show",
                json!({"project_id":project_id,"task_id":task_id}),
            )
            .await;
        }
        TaskCommand::Sources => return query_task(root, as_json, "task.sources", json!({})).await,
        TaskCommand::Board {
            project_id,
            source_id,
        } => {
            return query_task(
                root,
                as_json,
                "task.board",
                json!({"project_id":project_id,"source_id":source_id}),
            )
            .await;
        }
        TaskCommand::Connect(w) => ("connect", w),
        TaskCommand::SetActive(w) => ("set_active", w),
        TaskCommand::Attach(w) => ("attach", w),
        TaskCommand::Claim(w) => ("claim", w),
        TaskCommand::Create(w) => ("create", w),
        TaskCommand::Adopt(w) => ("adopt", w),
        TaskCommand::Update(w) => ("update", w),
        TaskCommand::Move(w) => ("move", w),
        TaskCommand::Cancel(w) => ("cancel", w),
        TaskCommand::DeleteCard(w) => ("delete_card", w),
        TaskCommand::Refresh(w) => ("refresh", w),
        TaskCommand::Resume(w) => ("resume", w),
    };
    let mut action: Value =
        serde_json::from_slice(&std::fs::read(w.input).map_err(io)?).map_err(|e| e.to_string())?;
    let object = action.as_object_mut().ok_or("input must be an object")?;
    if object.get("kind").is_some_and(|v| v.as_str() != Some(kind)) {
        return Err("action kind disagrees with subcommand".into());
    }
    object.insert("kind".into(), json!(kind));
    let payload = json!({"key":w.key,"action":action})
        .to_string()
        .into_bytes();
    let operation = format!("task.{kind}");
    let mut client = client(root).await?;
    if let Some(token) = w.preview_token {
        let r = client
            .submit(SubmitRequest {
                protocol: Some(Protocol {
                    version: PROTOCOL.into(),
                }),
                operation,
                payload,
                command_id: format!("task:{}", w.key),
                idempotency_key: w.key,
                preview_token: token,
            })
            .await
            .map_err(|e| e.to_string())?
            .into_inner();
        present(r.error, as_json);
        let value = bytes_json(&r.result)?;
        let failed = value.get("error").is_some();
        print_out(as_json, value);
        if failed {
            std::process::exit(1);
        }
    } else {
        let r = client
            .preview(PreviewRequest {
                protocol: Some(Protocol {
                    version: PROTOCOL.into(),
                }),
                operation,
                payload,
                command_id: invocation_id("preview"),
            })
            .await
            .map_err(|e| e.to_string())?
            .into_inner();
        present(r.error, as_json);
        print_out(
            as_json,
            json!({"preview_token":r.preview_token,"effect_summary":bytes_json(&r.effect_summary)?}),
        );
    }
    Ok(())
}
fn present(error: Option<proto::Error>, as_json: bool) {
    if let Some(e) = error {
        print_out(
            as_json,
            json!({"error":{"code":e.code,"message":e.message,"recovery_action":e.recovery_action}}),
        );
        std::process::exit(1);
    }
}
async fn query_task(root: &Path, as_json: bool, kind: &str, payload: Value) -> Result<(), String> {
    let r = client(root)
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
    present(r.error, as_json);
    print_out(as_json, bytes_json(&r.payload)?);
    Ok(())
}
