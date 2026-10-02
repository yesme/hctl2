use super::*;
use crate::task::{Write, query_task, write};

#[derive(Subcommand)]
pub(super) enum ProjectCommand {
    Create(Write),
    Update(Write),
    Archive(Write),
    Restore(Write),
    Select(Write),
    Members(Write),
    Resume(Write),
    List,
    Show { project_id: String },
    Overview { project_id: String },
    Pending { project_id: String },
    Attention { project_id: String },
    ArchiveBlockers { project_id: String },
    Roster { project_id: String, room_id: String },
}
#[derive(Subcommand)]
pub(super) enum RequestCommand {
    Create(Write),
    Resolve(Write),
    Cancel(Write),
    List {
        project_id: String,
    },
    Show {
        project_id: String,
        request_id: String,
    },
}
pub(super) async fn project(
    command: ProjectCommand,
    root: &Path,
    as_json: bool,
) -> Result<(), String> {
    report(execute_project(command, root, as_json).await, as_json)
}
fn report(result: Result<(), String>, as_json: bool) -> Result<(), String> {
    if let Err(e) = &result {
        print_out(
            as_json,
            json!({"error":{"code":"PROJECT_COMMAND_FAILED","message":e,"recovery_action":"inspect_error_and_retry"}}),
        );
    }
    result
}
async fn execute_project(
    command: ProjectCommand,
    root: &Path,
    as_json: bool,
) -> Result<(), String> {
    let (kind, w) = match command {
        ProjectCommand::List => return query_task(root, as_json, "project.list", json!({})).await,
        ProjectCommand::Show { project_id } => {
            return query_task(
                root,
                as_json,
                "project.show",
                json!({"project_id":project_id}),
            )
            .await;
        }
        ProjectCommand::Overview { project_id } => {
            return query_task(
                root,
                as_json,
                "project.overview",
                json!({"project_id":project_id}),
            )
            .await;
        }
        ProjectCommand::Pending { project_id } => {
            return query_task(
                root,
                as_json,
                "project.pending",
                json!({"project_id":project_id}),
            )
            .await;
        }
        ProjectCommand::Attention { project_id } => {
            return query_task(
                root,
                as_json,
                "project.attention",
                json!({"project_id":project_id}),
            )
            .await;
        }
        ProjectCommand::ArchiveBlockers { project_id } => {
            return query_task(
                root,
                as_json,
                "project.archive_blockers",
                json!({"project_id":project_id}),
            )
            .await;
        }
        ProjectCommand::Roster {
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
        ProjectCommand::Create(w) => ("create", w),
        ProjectCommand::Update(w) => ("update", w),
        ProjectCommand::Archive(w) => ("archive", w),
        ProjectCommand::Restore(w) => ("restore", w),
        ProjectCommand::Select(w) => ("select", w),
        ProjectCommand::Resume(w) => ("resume", w),
        ProjectCommand::Members(w) => ("members", w),
    };
    write(root, as_json, "project", kind, w).await
}
pub(super) async fn request(
    command: RequestCommand,
    root: &Path,
    as_json: bool,
) -> Result<(), String> {
    report(execute_request(command, root, as_json).await, as_json)
}
async fn execute_request(
    command: RequestCommand,
    root: &Path,
    as_json: bool,
) -> Result<(), String> {
    let (kind, w) = match command {
        RequestCommand::List { project_id } => {
            return query_task(
                root,
                as_json,
                "request.list",
                json!({"project_id":project_id}),
            )
            .await;
        }
        RequestCommand::Show {
            project_id,
            request_id,
        } => {
            return query_task(
                root,
                as_json,
                "request.show",
                json!({"project_id":project_id,"request_id":request_id}),
            )
            .await;
        }
        RequestCommand::Create(w) => ("create_request", w),
        RequestCommand::Resolve(w) => ("resolve_request", w),
        RequestCommand::Cancel(w) => ("cancel_request", w),
    };
    write(root, as_json, "project", kind, w).await
}
