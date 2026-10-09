use super::*;
use crate::task::query_task;

#[derive(clap::Args)]
pub(super) struct Publish {
    /// JSON action: `{"project_id","project_version","memo_id","applicability","sources","supersedes","expires_at"}`.
    #[arg(long, value_name = "INPUT")]
    input: PathBuf,
    /// Text file holding the exact governance body; nothing else becomes a Memo.
    #[arg(long, value_name = "FILE")]
    body: PathBuf,
    #[arg(long)]
    key: String,
    #[arg(long)]
    preview_token: Option<String>,
}

#[derive(Subcommand)]
pub(super) enum MemoCommand {
    /// Preview a publication; the token confirms the exact same text.
    Publish(Publish),
    /// Read one published revision exactly; without `--revision`, the current pointer.
    Show {
        project_id: String,
        memo_id: String,
        #[arg(long)]
        revision: Option<i64>,
    },
    /// List this Project's Memos and the pointer manifest that filters them.
    List { project_id: String },
}

pub(super) async fn dispatch(
    command: MemoCommand,
    root: &Path,
    as_json: bool,
) -> Result<(), String> {
    let result = execute(command, root, as_json).await;
    if let Err(error) = &result {
        print_out(
            as_json,
            json!({"error":{"code":"MEMO_COMMAND_FAILED","message":error,"recovery_action":"inspect_error_and_retry"}}),
        );
    }
    result
}

async fn execute(command: MemoCommand, root: &Path, as_json: bool) -> Result<(), String> {
    match command {
        MemoCommand::Show {
            project_id,
            memo_id,
            revision,
        } => {
            query_task(
                root,
                as_json,
                "memo.show",
                json!({"project_id":project_id,"memo_id":memo_id,"revision":revision}),
            )
            .await
        }
        MemoCommand::List { project_id } => {
            query_task(root, as_json, "memo.list", json!({"project_id":project_id})).await
        }
        MemoCommand::Publish(publish) => write(root, as_json, publish).await,
    }
}

async fn write(root: &Path, as_json: bool, publish: Publish) -> Result<(), String> {
    let mut action: Value = serde_json::from_slice(&std::fs::read(&publish.input).map_err(io)?)
        .map_err(|e| e.to_string())?;
    let object = action.as_object_mut().ok_or("input must be an object")?;
    if object
        .get("kind")
        .is_some_and(|kind| kind.as_str() != Some("publish"))
    {
        return Err("action kind disagrees with subcommand".into());
    }
    // The body is a file, not a JSON string: the bytes the user wrote are the
    // bytes that become governance material.
    let body = std::fs::read(&publish.body).map_err(io)?;
    match object.get("body") {
        None | Some(Value::Null) => {}
        Some(claimed) if claimed.as_str().map(str::as_bytes) == Some(body.as_slice()) => {}
        Some(_) => return Err("input body differs from --body".into()),
    }
    object.insert("kind".into(), json!("publish"));
    object.insert(
        "body".into(),
        Value::String(String::from_utf8(body).map_err(|_| "body must be UTF-8 text")?),
    );
    let payload = json!({"key":publish.key,"action":action})
        .to_string()
        .into_bytes();
    let mut client = client(root).await?;
    let command_id = format!("memo:{}", publish.key);
    if let Some(preview_token) = publish.preview_token {
        let result = client
            .submit(SubmitRequest {
                protocol: Some(Protocol {
                    version: PROTOCOL.into(),
                }),
                operation: "memo.publish".into(),
                payload,
                command_id,
                idempotency_key: publish.key,
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
                operation: "memo.publish".into(),
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
