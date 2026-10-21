use super::*;
use crate::task::query_task;
#[derive(clap::Args)]
pub(super) struct Write {
    /// Exact JSON action. Command kind is supplied by the subcommand.
    #[arg(long)]
    input: PathBuf,
    #[arg(long)]
    key: String,
    #[arg(long)]
    preview_token: Option<String>,
}
#[derive(Subcommand)]
pub(super) enum EngineCommand {
    Start(Write),
    Poll(Write),
    Complete(Write),
    Cancel(Write),
    SelfAdvance(Write),
    DirectOp(Write),
    Show {
        project_id: String,
        binding_id: String,
    },
}

#[derive(Subcommand)]
pub(super) enum WorkflowCommand {
    Register(Write),
    Compile(Write),
    Approve(Write),
    Engine {
        #[command(subcommand)]
        command: EngineCommand,
    },
    List {
        project_id: String,
        /// Recover every saved version associated with this Task, including history.
        #[arg(long = "task")]
        task_id: Option<String>,
    },
    Show {
        project_id: String,
        workflow_id: String,
        #[arg(long)]
        revision: Option<i64>,
    },
}
pub(super) async fn dispatch(
    command: WorkflowCommand,
    root: &Path,
    as_json: bool,
) -> Result<(), String> {
    let result = execute(command, root, as_json).await;
    if let Err(error) = &result {
        print_out(
            as_json,
            json!({"error":{"code":"WORKFLOW_COMMAND_FAILED","message":error,"recovery_action":"inspect_error_and_retry"}}),
        );
    }
    result
}
async fn execute(command: WorkflowCommand, root: &Path, as_json: bool) -> Result<(), String> {
    let (kind, write) = match command {
        WorkflowCommand::List {
            project_id,
            task_id,
        } => {
            return query_task(
                root,
                as_json,
                "workflow.list",
                json!({"project_id":project_id,"task_id":task_id}),
            )
            .await;
        }
        WorkflowCommand::Show {
            project_id,
            workflow_id,
            revision,
        } => {
            return query_task(
                root,
                as_json,
                "workflow.show",
                json!({"project_id":project_id,"workflow_id":workflow_id,"revision":revision}),
            )
            .await;
        }
        WorkflowCommand::Engine { command } => match command {
            EngineCommand::Show {
                project_id,
                binding_id,
            } => {
                return query_task(
                    root,
                    as_json,
                    "workflow.engine.show",
                    json!({"project_id":project_id,"binding_id":binding_id}),
                )
                .await;
            }
            EngineCommand::Start(write) => ("engine.start", write),
            EngineCommand::Poll(write) => ("engine.poll", write),
            EngineCommand::Complete(write) => ("engine.complete", write),
            EngineCommand::Cancel(write) => ("engine.cancel", write),
            EngineCommand::SelfAdvance(write) => ("engine.self_advance", write),
            EngineCommand::DirectOp(write) => ("engine.direct_op", write),
        },
        WorkflowCommand::Register(write) => ("register", write),
        WorkflowCommand::Compile(write) => ("compile", write),
        WorkflowCommand::Approve(write) => ("approve", write),
    };
    let mut action: Value = serde_json::from_slice(&std::fs::read(&write.input).map_err(io)?)
        .map_err(|e| e.to_string())?;
    let object = action.as_object_mut().ok_or("input must be an object")?;
    if !kind.starts_with("engine.") {
        if object.get("kind").is_some_and(|v| v.as_str() != Some(kind)) {
            return Err("action kind disagrees with subcommand".into());
        }
        object.insert("kind".into(), json!(kind));
    }
    let payload = json!({"key":write.key,"action":action})
        .to_string()
        .into_bytes();
    let operation = format!("workflow.{kind}");
    let command_id = format!("workflow:{}", write.key);
    let mut client = client(root).await?;
    if let Some(preview_token) = write.preview_token {
        let result = client
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
        task::present(result.error, as_json);
        print_out(as_json, bytes_json(&result.result)?);
    } else {
        let result = client
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
        task::present(result.error, as_json);
        print_out(
            as_json,
            json!({"preview_token":result.preview_token,"effect_summary":bytes_json(&result.effect_summary)?}),
        );
    }
    Ok(())
}
