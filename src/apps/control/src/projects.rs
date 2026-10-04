use crate::services::Supervisor;
use project::{Action, Input, Plan, Result, invalid, reject};
use serde_json::{Value, json};
use std::{path::Path, sync::Arc};
use store::{Scope, Store, TrustedActor};
use tokio::sync::Mutex;

type Shared = Arc<Mutex<Option<Store>>>;
fn access<T>(shared: &Shared, f: impl FnOnce(&mut Store) -> Result<T>) -> Result<T> {
    let mut slot = shared.blocking_lock();
    f(slot
        .as_mut()
        .ok_or_else(|| reject("STORE_NOT_READY", "store not ready", "check_status"))?)
}
fn input(operation: &str, payload: &Value) -> Result<Input> {
    let input: Input = serde_json::from_value(payload.clone())?;
    if operation
        != format!(
            "project.{}",
            payload["action"]["kind"].as_str().unwrap_or("")
        )
    {
        return Err(invalid("Project operation and action disagree"));
    }
    Ok(input)
}
pub(crate) fn preview(
    shared: &Shared,
    services: &Supervisor,
    actor: &TrustedActor,
    operation: &str,
    payload: &Value,
) -> Result<Value> {
    let input = input(operation, payload)?;
    if let Action::Members { users, .. } = &input.action {
        for user in users {
            ruma::UserId::parse(user).map_err(|_| invalid("invalid Matrix user ID"))?;
        }
    }
    if let Some(plan) = access(shared, |s| project::replay(s, &input))? {
        return Ok(serde_json::to_value(plan)?);
    }
    let server = if matches!(input.action, Action::Create { .. }) {
        Some(crate::chat::configured_server(services)?)
    } else {
        None
    };
    access(shared, |s| {
        Ok(serde_json::to_value(project::prepare(
            s,
            input,
            server,
            actor,
            task::now(),
        )?)?)
    })
}
pub(crate) fn submit(
    shared: &Shared,
    operations: &Mutex<()>,
    services: &Supervisor,
    root: &Path,
    actor: &TrustedActor,
    req: &proto::SubmitRequest,
    details: &Value,
) -> Result<Value> {
    let input = input(&req.operation, &serde_json::from_slice(&req.payload)?)?;
    if req.command_id != format!("project:{}", input.key) || req.idempotency_key != input.key {
        return Err(invalid("Project envelope key mismatch"));
    }
    let plan: Plan = serde_json::from_value(details.clone())?;
    if serde_json::to_value(&input)? != serde_json::to_value(&plan.input)? {
        return Err(project::stale());
    }
    let _operations = operations.blocking_lock();
    let mut result = access(shared, |s| project::admit(s, actor, plan))?;
    if let Some(id) = result["effect_id"].as_str().map(str::to_owned) {
        let operation = access(shared, |s| Ok(s.effect(&id)?.0.operation))?;
        let outcome = if operation == "request.task_adopt" {
            access(shared, |s| project::deliver(s, actor, &id))
        } else {
            crate::chat::drive(shared, services, root, actor, &id)
        };
        match outcome {
            Ok(receipt) => {
                result["receipt"] = receipt;
                result["delivery"] = json!("confirmed");
            }
            Err(e) => {
                result["delivery"] = json!("pending_or_unknown");
                result["error"] =
                    json!({"code":e.code,"message":e.message,"recovery_action":e.recovery_action});
            }
        }
    }
    if let Some(ids) = result["effect_ids"].as_array().cloned() {
        let mut outcomes = vec![];
        let mut failed = false;
        for id in ids {
            let id = id.as_str().ok_or_else(|| invalid("effect ID missing"))?;
            match crate::chat::drive(shared, services, root, actor, id) {
                Ok(receipt) => {
                    let partial = receipt["space_delivery"] == "partial";
                    failed |= partial;
                    outcomes.push(json!({"effect_id":id,"delivery":if partial { "partial" } else { "confirmed" },"receipt":receipt}))
                }
                Err(e) => {
                    failed = true;
                    outcomes.push(json!({"effect_id":id,"delivery":"pending_or_unknown","error":{"code":e.code,"message":e.message,"recovery_action":e.recovery_action}}));
                }
            }
        }
        result["rooms"] = json!(outcomes);
        result["delivery"] = json!(if failed { "partial" } else { "confirmed" });
        if failed {
            result["error"] = json!({"code":"ROOMS_PARTIAL","message":"not all Room actions confirmed","recovery_action":"resume_original_command"});
        }
    }
    Ok(result)
}

pub(crate) fn query(
    shared: &Shared,
    services: &Supervisor,
    actor: &TrustedActor,
    kind: &str,
    payload: &Value,
) -> Result<Value> {
    if kind == "project.attention" {
        let id = payload["project_id"]
            .as_str()
            .ok_or_else(|| invalid("project_id required"))?;
        let (rooms, stamp) = access(shared, |s| {
            Ok((
                s.list("room_binding")?
                    .iter()
                    .filter(|r| r.key.scope == Scope::Project(id.into()))
                    .map(chat::decode::<chat::Room>)
                    .collect::<Result<Vec<_>>>()?,
                s.read_stamp(),
            ))
        })?;
        let mut last = std::collections::BTreeMap::new();
        let mut unread = vec![];
        for room in rooms {
            if !matches!(room.origin, Some(chat::Origin::Request { .. })) {
                continue;
            }
            match crate::chat::last_activity(services, &room) {
                Ok(value) => {
                    last.insert(room.id, value);
                }
                Err(e) => {
                    unread.push(json!({"room_id":room.id,"error":e.code}));
                    last.insert(room.id, None);
                }
            }
        }
        return access(shared, |s| {
            if s.read_stamp() != stamp {
                return Err(project::stale());
            }
            Ok(
                json!({"items":project::attention(s,id,task::now(),&last)?,"unread":unread,"adds_pending_items":false}),
            )
        });
    }
    access(shared, |s| {
        if kind == "project.list" {
            return Ok(json!({"projects":s.list("project")?}));
        }
        let project = payload["project_id"].as_str();
        if kind == "pending" && project.is_none() {
            let mut items = vec![];
            for p in s.list("project")? {
                items.extend(
                    project::pending(s, &p.key.id, actor)?["items"]
                        .as_array()
                        .cloned()
                        .unwrap_or_default(),
                );
            }
            return Ok(
                json!({"items":items,"implemented_sources":["request","contract_adoption"]}),
            );
        }
        let id = project.ok_or_else(|| invalid("project_id required"))?;
        match kind {
            "project.show" | "project.overview" | "overview" => project::overview(s, id, actor),
            "project.pending" | "pending" => project::pending(s, id, actor),
            "project.archive_blockers" => Ok(json!({"blockers":project::archive_blockers(s,id)?})),
            "request.list" => {
                let readonly = project::readonly(&project::project(s, id)?);
                Ok(
                    json!({"requests":project::requests(s,id)?.into_iter().map(|(r,q)|json!({"reference":project::reference(&r),"request":q,"readonly":readonly})).collect::<Vec<_>>()}),
                )
            }
            "request.show" => {
                let request_id = payload["request_id"]
                    .as_str()
                    .ok_or_else(|| invalid("request_id required"))?;
                let r = project::required(
                    s,
                    &project::key(Scope::Project(id.into()), "request", request_id),
                )?;
                Ok(json!({"record":r,"readonly":project::readonly(&project::project(s,id)?)}))
            }
            "project.roster" => {
                let room = payload["room_id"]
                    .as_str()
                    .ok_or_else(|| invalid("room_id required"))?;
                let (_, room) = chat::room(s, id, room)?;
                let selections = room
                    .participants
                    .iter()
                    .map(|r| chat::at(s, r))
                    .collect::<Result<Vec<_>>>()?;
                Ok(
                    json!({"participants":room.participants,"selections":selections,"candidate_validation":"deferred_to_p23"}),
                )
            }
            _ => Err(invalid("unknown Project query")),
        }
    })
}

/// Restart recovery uses the same durable effects, never a second signal channel.
pub(crate) fn reconcile(shared: &Shared, actor: &TrustedActor) -> Result<()> {
    access(shared, |s| {
        for p in s.list("project")? {
            project::expire(s, actor, &p.key.id, task::now())?;
        }
        for id in s.pending_effects()? {
            if s.effect(&id)?.0.operation == "request.task_adopt" {
                // One stale receiver must not prevent unrelated Requests from progressing.
                let _ = project::deliver(s, actor, &id);
            }
        }
        Ok(())
    })
}
