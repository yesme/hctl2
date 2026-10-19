//! Run-owned workflow commands use the common owner preview/submit boundary.
use project::{Result, invalid, reject};
use run::{Input, Plan, admit, list, prepare, show};
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;
use store::{Store, TrustedActor};
use tokio::sync::Mutex;

type Shared = Arc<Mutex<Option<Store>>>;

fn not_ready() -> project::StoreError {
    reject("STORE_NOT_READY", "store not ready", "check_status")
}

fn input(operation: &str, payload: &Value) -> Result<Input> {
    let input: Input = serde_json::from_value(payload.clone())?;
    if operation != format!("workflow.{}", input.action.kind()) {
        return Err(invalid("Workflow operation and action disagree"));
    }
    Ok(input)
}

pub(crate) fn query(
    shared: &Shared,
    actor: &TrustedActor,
    kind: &str,
    payload: &Value,
) -> Result<Value> {
    let project_id = payload["project_id"]
        .as_str()
        .ok_or_else(|| invalid("project_id required"))?;
    let slot = shared.blocking_lock();
    let store = slot.as_ref().ok_or_else(not_ready)?;
    if kind == "workflow.list" {
        let task = match payload.get("task_id") {
            None | Some(Value::Null) => None,
            Some(value) => Some(
                value
                    .as_str()
                    .ok_or_else(|| invalid("task_id must be a string"))?,
            ),
        };
        return list(store, actor, project_id, task);
    }
    if kind != "workflow.show" {
        return Err(invalid(format!("unknown Workflow query {kind}")));
    }
    let workflow_id = payload["workflow_id"]
        .as_str()
        .ok_or_else(|| invalid("workflow_id required"))?;
    let revision = match payload.get("revision") {
        // No revision names the current pointer; a named revision is read exactly.
        None | Some(Value::Null) => None,
        Some(value) => Some(
            value
                .as_i64()
                .ok_or_else(|| invalid("revision must be an integer"))?,
        ),
    };
    show(store, actor, project_id, workflow_id, revision)
}

pub(crate) fn preview(
    shared: &Shared,
    services: &crate::services::Supervisor,
    root: &Path,
    actor: &TrustedActor,
    operation: &str,
    payload: &Value,
) -> Result<Value> {
    let input = input(operation, payload)?;
    let slot = shared.blocking_lock();
    let store = slot.as_ref().ok_or_else(not_ready)?;
    Ok(serde_json::to_value(prepare(
        store,
        input,
        actor,
        &validator(root, services),
    )?)?)
}

pub(crate) fn submit(
    shared: &Shared,
    services: &crate::services::Supervisor,
    root: &Path,
    actor: &TrustedActor,
    request: &proto::SubmitRequest,
    details: &Value,
) -> Result<Value> {
    let input = input(
        &request.operation,
        &serde_json::from_slice(&request.payload)?,
    )?;
    if request.command_id != format!("workflow:{}", input.key)
        || request.idempotency_key != input.key
    {
        return Err(invalid("Workflow envelope key mismatch"));
    }
    let plan: Plan = serde_json::from_value(details.clone())?;
    if input != plan.input {
        return Err(invalid("Workflow preview input differs"));
    }
    let mut slot = shared.blocking_lock();
    let store = slot.as_mut().ok_or_else(not_ready)?;
    admit(store, actor, plan, &validator(root, services))
}

fn validator(root: &Path, services: &crate::services::Supervisor) -> run::NativeDagu {
    let binary = std::env::var_os("HCTL2_DAGU_BIN")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            services
                .packaged_paths()
                .map(|(install, _)| install.join("libexec/hctl2/dagu"))
        })
        .unwrap_or_else(|| root.join("missing-packaged-dagu"));
    run::NativeDagu {
        binary,
        scratch: root.join("tmp/workflow-validate"),
    }
}
