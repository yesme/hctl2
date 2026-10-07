//! Published Memo commands. The user previews the exact text, then confirms the
//! same text; only this command makes text a Memo.
use project::memo::{Input, Plan, admit, list, prepare, show};
use project::{Result, invalid, reject};
use serde_json::Value;
use std::sync::Arc;
use store::{Store, TrustedActor};
use tokio::sync::Mutex;

type Shared = Arc<Mutex<Option<Store>>>;

fn not_ready() -> project::StoreError {
    reject("STORE_NOT_READY", "store not ready", "check_status")
}

fn input(operation: &str, payload: &Value) -> Result<Input> {
    let input: Input = serde_json::from_value(payload.clone())?;
    if operation != format!("memo.{}", input.action.kind()) {
        return Err(invalid("Memo operation and action disagree"));
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
    if kind == "memo.list" {
        // The clock is the control's own; a caller cannot ask for a filter
        // computed at a time of its choosing.
        return list(store, actor, project_id, task::now());
    }
    if kind != "memo.show" {
        return Err(invalid(format!("unknown Memo query {kind}")));
    }
    let memo_id = payload["memo_id"]
        .as_str()
        .ok_or_else(|| invalid("memo_id required"))?;
    let revision = match payload.get("revision") {
        // No revision names the current pointer; a named revision is read exactly.
        None | Some(Value::Null) => None,
        Some(value) => Some(
            value
                .as_i64()
                .ok_or_else(|| invalid("revision must be an integer"))?,
        ),
    };
    show(store, actor, project_id, memo_id, revision)
}

pub(crate) fn preview(
    shared: &Shared,
    actor: &TrustedActor,
    operation: &str,
    payload: &Value,
) -> Result<Value> {
    let input = input(operation, payload)?;
    let slot = shared.blocking_lock();
    let store = slot.as_ref().ok_or_else(not_ready)?;
    Ok(serde_json::to_value(prepare(store, input, actor)?)?)
}

pub(crate) fn submit(
    shared: &Shared,
    actor: &TrustedActor,
    request: &proto::SubmitRequest,
    details: &Value,
) -> Result<Value> {
    let input = input(
        &request.operation,
        &serde_json::from_slice(&request.payload)?,
    )?;
    if request.command_id != format!("memo:{}", input.key) || request.idempotency_key != input.key {
        return Err(invalid("Memo envelope key mismatch"));
    }
    let plan: Plan = serde_json::from_value(details.clone())?;
    if input != plan.input {
        return Err(invalid("Memo preview input differs"));
    }
    let mut slot = shared.blocking_lock();
    let store = slot.as_mut().ok_or_else(not_ready)?;
    admit(store, actor, plan)
}
