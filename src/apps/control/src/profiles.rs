//! Worker Profile commands use the same preview token and Store transaction as
//! other control definitions. This module never talks to Agency.
use participant::{
    invalid,
    profiles::{ProfileInput, ProfilePlan, admit_profile, prepare_profile},
    reject,
};
use serde_json::Value;
use std::sync::Arc;
use store::{Store, TrustedActor};
use tokio::sync::Mutex;

type Shared = Arc<Mutex<Option<Store>>>;

fn input(operation: &str, payload: &Value) -> store::Result<ProfileInput> {
    let input: ProfileInput = serde_json::from_value(payload.clone())?;
    if operation != "profile.create" || input.action.kind() != "create" {
        return Err(invalid("Profile operation and action disagree"));
    }
    Ok(input)
}

pub(crate) fn preview(
    shared: &Shared,
    actor: &TrustedActor,
    operation: &str,
    payload: &Value,
) -> store::Result<Value> {
    let input = input(operation, payload)?;
    let slot = shared.blocking_lock();
    let store = slot
        .as_ref()
        .ok_or_else(|| reject("STORE_NOT_READY", "store not ready", "check_status"))?;
    Ok(serde_json::to_value(prepare_profile(store, input, actor)?)?)
}

pub(crate) fn submit(
    shared: &Shared,
    actor: &TrustedActor,
    request: &proto::SubmitRequest,
    details: &Value,
) -> store::Result<Value> {
    let input = input(
        &request.operation,
        &serde_json::from_slice(&request.payload)?,
    )?;
    if request.command_id != format!("profile:{}", input.key)
        || request.idempotency_key != input.key
    {
        return Err(invalid("Profile envelope key mismatch"));
    }
    let plan: ProfilePlan = serde_json::from_value(details.clone())?;
    if input != plan.input {
        return Err(invalid("Profile preview input differs"));
    }
    let mut slot = shared.blocking_lock();
    let store = slot
        .as_mut()
        .ok_or_else(|| reject("STORE_NOT_READY", "store not ready", "check_status"))?;
    admit_profile(store, actor, plan)
}
