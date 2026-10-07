use super::*;
use crate::task::{Write, query_task, write};

#[derive(Subcommand)]
pub(super) enum RoomCommand {
    List {
        #[arg(long)]
        project_id: Option<String>,
    },
    Show {
        project_id: String,
        room_id: String,
    },
    Hierarchy {
        project_id: String,
        room_id: String,
    },
    Timeline {
        project_id: String,
        room_id: String,
        #[arg(long)]
        cursor: Option<String>,
    },
    Event {
        project_id: String,
        room_id: String,
        event_id: String,
    },
    Sync {
        project_id: String,
        room_id: String,
        #[arg(long)]
        cursor: Option<String>,
    },
    ViewState {
        project_id: String,
        room_id: String,
        client_id: String,
    },
    SaveViewState {
        #[arg(long)]
        input: PathBuf,
    },
    Reparent {
        #[arg(long)]
        input: PathBuf,
    },
    /// Structural source selection. Does not create a Room or dispatch a Participant.
    Draft {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        key: Option<String>,
    },
    /// Query a frozen reference, including when the original event was redacted.
    Reference {
        #[arg(long)]
        input: PathBuf,
    },
    CreateTopic(Write),
    Close(Write),
    Rebind(Write),
    Send(Write),
    Freeze(Write),
    Resume(Write),
    #[command(subcommand)]
    Roster(RosterCommand),
}

#[derive(Subcommand)]
pub(super) enum RosterCommand {
    /// Show the roster's exact selections. Grants no dispatch right.
    Show { project_id: String, room_id: String },
    /// Submit the whole roster; keeping a candidate means resending it.
    Select(Write),
}

pub(super) async fn dispatch(
    command: RoomCommand,
    root: &Path,
    as_json: bool,
) -> Result<(), String> {
    let result = execute(command, root, as_json).await;
    if let Err(error) = &result {
        print_error(
            as_json,
            "ROOM_COMMAND_FAILED",
            error,
            "inspect_error_and_retry",
        );
    }
    result
}
async fn execute(command: RoomCommand, root: &Path, as_json: bool) -> Result<(), String> {
    let (kind, w) = match command {
        RoomCommand::List { project_id } => {
            return query_task(root, as_json, "room.list", json!({"project_id":project_id})).await;
        }
        RoomCommand::Show {
            project_id,
            room_id,
        } => {
            return query_task(
                root,
                as_json,
                "room.show",
                json!({"project_id":project_id,"room_id":room_id}),
            )
            .await;
        }
        RoomCommand::Hierarchy {
            project_id,
            room_id,
        } => {
            return query_task(
                root,
                as_json,
                "room.hierarchy",
                json!({"project_id":project_id,"room_id":room_id}),
            )
            .await;
        }
        RoomCommand::Timeline {
            project_id,
            room_id,
            cursor,
        } => {
            return query_task(
                root,
                as_json,
                "room.timeline",
                json!({"project_id":project_id,"room_id":room_id,"cursor":cursor}),
            )
            .await;
        }
        RoomCommand::Event {
            project_id,
            room_id,
            event_id,
        } => {
            return query_task(
                root,
                as_json,
                "room.event",
                json!({"project_id":project_id,"room_id":room_id,"event_id":event_id}),
            )
            .await;
        }
        RoomCommand::Sync {
            project_id,
            room_id,
            cursor,
        } => {
            return query_task(
                root,
                as_json,
                "room.sync",
                json!({"project_id":project_id,"room_id":room_id,"cursor":cursor}),
            )
            .await;
        }
        RoomCommand::ViewState {
            project_id,
            room_id,
            client_id,
        } => {
            return query_task(
                root,
                as_json,
                "room.view_state",
                json!({"project_id":project_id,"room_id":room_id,"client_id":client_id}),
            )
            .await;
        }
        RoomCommand::SaveViewState { input } => {
            return content_submit(root, as_json, "chat.view_state", input, String::new()).await;
        }
        RoomCommand::Reparent { input } => {
            return content_submit(root, as_json, "chat.reparent", input, String::new()).await;
        }
        RoomCommand::Draft { input, key } => {
            return content_submit(
                root,
                as_json,
                "chat.draft",
                input,
                key.unwrap_or_else(|| {
                    format!(
                        "draft-{}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos()
                    )
                }),
            )
            .await;
        }
        RoomCommand::Reference { input } => {
            return query_file(root, as_json, "room.reference", input).await;
        }
        RoomCommand::CreateTopic(w) => ("create_topic", w),
        RoomCommand::Close(w) => ("close", w),
        RoomCommand::Rebind(w) => ("rebind", w),
        RoomCommand::Send(w) => ("send", w),
        RoomCommand::Freeze(w) => ("freeze", w),
        RoomCommand::Resume(w) => ("resume", w),
        RoomCommand::Roster(command) => match command {
            RosterCommand::Show {
                project_id,
                room_id,
            } => {
                return query_task(
                    root,
                    as_json,
                    "project.roster",
                    json!({"project_id":project_id,"room_id":room_id}),
                )
                .await;
            }
            RosterCommand::Select(w) => {
                return write(root, as_json, "project", "select", w).await;
            }
        },
    };
    write(root, as_json, "room", kind, w).await
}

async fn content_submit(
    root: &Path,
    as_json: bool,
    operation: &str,
    input: PathBuf,
    key: String,
) -> Result<(), String> {
    let value: Value =
        serde_json::from_slice(&std::fs::read(input).map_err(io)?).map_err(|e| e.to_string())?;
    let r = client(root)
        .await?
        .submit(SubmitRequest {
            protocol: Some(Protocol {
                version: PROTOCOL.into(),
            }),
            operation: operation.into(),
            idempotency_key: key,
            payload: value.to_string().into_bytes(),
            ..Default::default()
        })
        .await
        .map_err(|e| e.to_string())?
        .into_inner();
    if let Some(e) = r.error {
        print_out(
            as_json,
            json!({"error":{"code":e.code,"message":e.message,"recovery_action":e.recovery_action}}),
        );
        std::process::exit(1);
    }
    print_out(as_json, bytes_json(&r.result)?);
    Ok(())
}
async fn query_file(root: &Path, as_json: bool, kind: &str, path: PathBuf) -> Result<(), String> {
    let payload =
        serde_json::from_slice(&std::fs::read(path).map_err(io)?).map_err(|e| e.to_string())?;
    query_task(root, as_json, kind, payload).await
}
