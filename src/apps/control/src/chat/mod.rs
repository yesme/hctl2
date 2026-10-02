mod matrix;
pub use matrix::Client as MatrixClient;
mod config;
mod drafting;
mod queries;
mod runtime;
pub(super) mod tree;
pub(crate) use config::private_write as write_private_config;
pub(super) use drafting::draft;
#[cfg(test)]
use drafting::select;
pub(super) use queries::{query, set_view_state};
pub(super) use runtime::{reconcile, serve};
mod inbox;
#[cfg(test)]
mod tests;

use crate::services::Supervisor;
use chat::{
    Action, DraftInput, Input, Origin, Plan, Result, Room, Selection, Source, SourceText, invalid,
    reject,
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use store::{
    Actor, ActorSource, Command, Expected, Reference, Scope, Store, TrustedActor, Version,
};
use tokio::sync::Mutex;
type Shared = Arc<Mutex<Option<Store>>>;

fn access<T>(shared: &Shared, f: impl FnOnce(&mut Store) -> Result<T>) -> Result<T> {
    let mut slot = shared.blocking_lock();
    f(slot
        .as_mut()
        .ok_or_else(|| reject("STORE_NOT_READY", "store not ready", "check_status"))?)
}
fn client(services: &Supervisor) -> Result<matrix::Client> {
    let (server, token) = config::load(services)?;
    matrix::Client::new(server, token)
}

/// Project creation (package 辛) freezes this connection when admitting its main Room.
pub fn configured_server(services: &Supervisor) -> Result<chat::Server> {
    Ok(config::load(services)?.0)
}
fn observations(root: &Path) -> inbox::Inbox {
    inbox::Inbox {
        root: root.into(),
        token: String::new(),
    }
}
fn bound(room: &Room) -> Result<&str> {
    room.matrix_room_id.as_deref().ok_or_else(|| {
        reject(
            "ROOM_PENDING",
            "Room creation not confirmed",
            "resume_original_command",
        )
    })
}
fn guard(client: &matrix::Client, root: &Path, room: &Room, external: &str) -> Result<()> {
    if room.server.binding != client.server.binding || room.server.url != client.server.url {
        return Err(reject(
            "CHAT_BINDING_CHANGED",
            "chat deployment differs from frozen binding",
            "check_chat_binding",
        ));
    }
    let result = client.guard(external);
    observations(root).health(&room.id, result.as_ref().err().map(|e| e.code))?;
    result
}
fn input(operation: &str, payload: &Value) -> Result<Input> {
    let input: Input = serde_json::from_value(payload.clone())?;
    if operation != format!("room.{}", payload["action"]["kind"].as_str().unwrap_or("")) {
        return Err(invalid("Room operation and action disagree"));
    }
    Ok(input)
}

fn resolve(
    shared: &Shared,
    services: &Supervisor,
    root: &Path,
    action: &Action,
) -> Result<Vec<SourceText>> {
    match action {
        Action::CreateTopic {
            project_id,
            origin,
            brief,
            ..
        } => {
            access(shared, |s| chat::origin_checks(s, project_id, origin))?;
            match origin.as_ref() {
                Origin::Request { request, blockers } => {
                    let mut texts = access(shared, |s| chat::request_texts(s, request, blockers))?;
                    for source in &brief.sources {
                        if matches!(source, Source::Message { .. }) {
                            texts
                                .push(resolve_message(shared, services, root, project_id, source)?);
                        }
                    }
                    // Preserve the confirmed source order, including optional Messages.
                    brief
                        .sources
                        .iter()
                        .map(|s| {
                            texts
                                .iter()
                                .find(|t| t.source == *s)
                                .cloned()
                                .ok_or_else(|| invalid("unverified Request source"))
                        })
                        .collect()
                }
                Origin::Room { room_id, .. } => {
                    let (r, room) = access(shared, |s| chat::room(s, project_id, room_id))?;
                    let client = client(services)?;
                    let external = bound(&room)?;
                    guard(&client, root, &room, external)?;
                    brief
                        .sources
                        .iter()
                        .map(|source| {
                            let Source::Message {
                                binding, event_id, ..
                            } = source
                            else {
                                return Err(invalid("main Room source must be a Message"));
                            };
                            if *binding != chat::reference(&r) {
                                return Err(chat::stale());
                            }
                            let source_text = matrix::source_text(
                                binding.clone(),
                                &client.event(external, event_id)?,
                            )?;
                            if source_text.source != *source {
                                return Err(reject(
                                    "SOURCE_CHANGED",
                                    "Message differs from previewed content",
                                    "preview_again",
                                ));
                            }
                            Ok(source_text)
                        })
                        .collect()
                }
            }
        }
        Action::Freeze {
            project_id,
            room_id,
            version,
            event_id,
        } => {
            let (r, room) = access(shared, |s| chat::room(s, project_id, room_id))?;
            if r.version != *version {
                return Err(chat::stale());
            }
            let client = client(services)?;
            guard(&client, root, &room, bound(&room)?)?;
            Ok(vec![matrix::source_text(
                chat::reference(&r),
                &client.event(bound(&room)?, event_id)?,
            )?])
        }
        Action::Rebind {
            project_id,
            room_id,
            matrix_room_id,
            ..
        } => {
            let (_, room) = access(shared, |s| chat::room(s, project_id, room_id))?;
            let client = client(services)?;
            if client.server.binding != room.server.binding {
                return Err(chat::stale());
            }
            client.guard(matrix_room_id)?;
            Ok(vec![])
        }
        Action::Send {
            project_id,
            room_id,
            ..
        } => {
            let (_, room) = access(shared, |s| chat::room(s, project_id, room_id))?;
            let client = client(services)?;
            guard(&client, root, &room, bound(&room)?)?;
            if let Action::Send {
                thread_root: Some(root),
                ..
            } = action
            {
                client.thread_root(bound(&room)?, root)?;
            }
            Ok(vec![])
        }
        Action::Close { .. } | Action::Resume { .. } => Ok(vec![]),
    }
}

fn resolve_message(
    shared: &Shared,
    services: &Supervisor,
    root: &Path,
    project: &str,
    source: &Source,
) -> Result<SourceText> {
    let Source::Message {
        binding, event_id, ..
    } = source
    else {
        return Err(invalid("Message required"));
    };
    if binding.key.scope != Scope::Project(project.into()) || binding.key.kind != "room_binding" {
        return Err(invalid("Message belongs to another Project"));
    }
    let (r, room) = access(shared, |s| chat::room(s, project, &binding.key.id))?;
    if chat::reference(&r) != *binding {
        return Err(chat::stale());
    }
    let client = client(services)?;
    guard(&client, root, &room, bound(&room)?)?;
    let text = matrix::source_text(binding.clone(), &client.event(bound(&room)?, event_id)?)?;
    if text.source != *source {
        return Err(reject(
            "SOURCE_CHANGED",
            "Message content differs",
            "preview_again",
        ));
    }
    Ok(text)
}

pub(super) fn preview(
    shared: &Shared,
    services: &Supervisor,
    root: &Path,
    operation: &str,
    payload: &Value,
) -> Result<Value> {
    let input = input(operation, payload)?;
    if let Some(plan) = access(shared, |s| chat::replay(s, &input))? {
        return Ok(serde_json::to_value(plan)?);
    }
    let stamp = access(shared, |s| Ok(s.read_stamp()))?;
    let sources = resolve(shared, services, root, &input.action)?;
    access(shared, |s| {
        if s.read_stamp() != stamp {
            return Err(chat::stale());
        }
        Ok(serde_json::to_value(chat::prepare(s, input, sources)?)?)
    })
}

pub(super) fn submit(
    shared: &Shared,
    operations: &Mutex<()>,
    services: &Supervisor,
    root: &Path,
    actor: &TrustedActor,
    request: &proto::SubmitRequest,
    details: &Value,
) -> Result<Value> {
    let input = input(
        &request.operation,
        &serde_json::from_slice(&request.payload)?,
    )?;
    if request.command_id != format!("room:{}", input.key) || request.idempotency_key != input.key {
        return Err(invalid("Room command envelope key mismatch"));
    }
    let plan: Plan = serde_json::from_value(details.clone())?;
    if serde_json::to_value(&input)? != serde_json::to_value(&plan.input)? {
        return Err(chat::stale());
    }
    let replay = access(shared, |s| chat::replay(s, &input))?.is_some();
    let stamp = access(shared, |s| Ok(s.read_stamp()))?;
    if !replay {
        let sources = resolve(shared, services, root, &input.action)?;
        if serde_json::to_value(&sources)? != serde_json::to_value(&plan.source_texts)? {
            return Err(chat::stale());
        }
    }
    let _operation = operations.blocking_lock();
    let mut result = access(shared, |s| {
        if s.read_stamp() != stamp {
            return Err(chat::stale());
        }
        chat::admit(s, actor, plan)
    })?;
    if let Action::Rebind { room_id, .. } = &input.action {
        observations(root).health(room_id, None)?;
    }
    if let Some(id) = result["effect_id"].as_str().map(str::to_owned) {
        match drive(shared, services, root, actor, &id) {
            Ok(receipt) => {
                result["state"] = json!("confirmed");
                result["receipt"] = receipt;
            }
            Err(e) => {
                result["state"] = json!(if e.code == "EFFECT_CANCELLED" {
                    "cancelled"
                } else {
                    "pending_or_unknown"
                });
                result["error"] =
                    json!({"code":e.code,"message":e.message,"recovery_action":e.recovery_action});
            }
        }
    }
    Ok(result)
}

pub(crate) fn drive(
    shared: &Shared,
    services: &Supervisor,
    root: &Path,
    actor: &TrustedActor,
    id: &str,
) -> Result<Value> {
    drive_using(shared, root, actor, id, || client(services))
}

fn drive_using(
    shared: &Shared,
    root: &Path,
    actor: &TrustedActor,
    id: &str,
    connect: impl FnOnce() -> Result<matrix::Client>,
) -> Result<Value> {
    let (effect, state) = access(shared, |s| s.effect(id))?;
    let room: Room = serde_json::from_value(effect.input["room"].clone())?;
    if state == store::EffectState::Confirmed {
        return access(shared, |s| {
            chat::decode::<Value>(&chat::required(
                s,
                &chat::key(effect.permission_scope, "room_effect_receipt", id),
            )?)
        });
    }
    if state == store::EffectState::Cancelled {
        return Err(reject(
            "EFFECT_CANCELLED",
            "unsent Room creation was cancelled",
            "inspect_room",
        ));
    }
    let client = connect()?;
    if room.server.binding != client.server.binding || room.server.url != client.server.url {
        return Err(chat::stale());
    }
    let pending = state == store::EffectState::Pending;
    if state == store::EffectState::Unknown && effect.operation == "chat.create" {
        let active = access(shared, |s| {
            let identity = chat::required(s, &effect.owner.key)?;
            Ok(matches!(
                identity.data,
                store::RecordData::Room {
                    state: store::RoomState::Active,
                    ..
                }
            ))
        })?;
        if !active {
            let receipt = client
                .lookup_creation(&room, &effect.idempotency_key, false, true)?
                .ok_or_else(|| {
                    reject(
                        "RESULT_UNKNOWN",
                        "closed Room creation has no confirmed native result",
                        "read_back_original_intent",
                    )
                })?;
            if let Some(parent) = effect.input.get("parent") {
                let expected = json!({"command":effect.idempotency_key,"parent_room_id":parent["id"],"project_id":room.project_id});
                let external = receipt["matrix_room_id"]
                    .as_str()
                    .ok_or_else(|| invalid("child room missing"))?;
                match client.state(external, "io.hctl2.topic_creation") {
                    Ok(marker) if marker == expected => (),
                    Ok(_) => return Err(invalid("Topic creation marker differs")),
                    Err(e) if e.code == "CHAT_NOT_FOUND" => {
                        return Err(reject(
                            "RESULT_UNKNOWN",
                            "closed Topic creation is not fully confirmed",
                            "read_back_original_intent",
                        ));
                    }
                    Err(e) => return Err(e),
                }
            }
            return access(shared, |s| chat::confirm(s, actor, id, receipt));
        }
    }
    // Check current local authority before first dispatch. Unknown writes retain the original
    // target and key; readback never retargets a newer Room binding.
    if pending {
        access(shared, |s| {
            let (_, current) = chat::room(s, &room.project_id, &room.id)?;
            chat::writable(s, &current)?;
            let project = chat::required(
                s,
                &chat::key(effect.permission_scope.clone(), "project", &room.project_id),
            )?;
            chat::active_project(s, &room.project_id, project.version)?;
            if current.matrix_room_id != room.matrix_room_id
                || current.server.binding != room.server.binding
            {
                return Err(chat::stale());
            }
            s.resume_pending_effect(s.generation(), id, true)?;
            Ok(())
        })?;
    }
    let receipt = match effect.operation.as_str() {
        "chat.carrier" => {
            guard(&client, root, &room, bound(&room)?)?;
            access(shared, |s| {
                let (_, current) = chat::room(s, &room.project_id, &room.id)?;
                chat::writable(s, &current)?;
                let p = chat::required(
                    s,
                    &chat::key(effect.permission_scope.clone(), "project", &room.project_id),
                )?;
                chat::active_project(s, &room.project_id, p.version)?;
                if current.matrix_room_id != room.matrix_room_id
                    || current.server.binding != room.server.binding
                {
                    return Err(chat::stale());
                }
                if pending {
                    s.begin_effect(s.generation(), id)?;
                }
                Ok(())
            })?;
            json!({"space_id":client.ensure_carrier(&room)?})
        }
        "chat.create" => {
            let parent: Option<Room> = effect
                .input
                .get("parent")
                .map(|v| serde_json::from_value(v.clone()))
                .transpose()?;
            if let Some(parent) = &parent {
                guard(&client, root, parent, bound(parent)?)?;
            }
            let mark = || {
                access(shared, |s| {
                    let (_, current) = chat::room(s, &room.project_id, &room.id)?;
                    chat::writable(s, &current)?;
                    let p = chat::required(
                        s,
                        &chat::key(effect.permission_scope.clone(), "project", &room.project_id),
                    )?;
                    chat::active_project(s, &room.project_id, p.version)?;
                    if current
                        .matrix_room_id
                        .is_some_and(|r| room.matrix_room_id.as_ref() != Some(&r))
                    {
                        return Err(chat::stale());
                    }
                    if let Some(parent) = &parent {
                        let (_, current_parent) = chat::room(s, &room.project_id, &parent.id)?;
                        if current_parent.matrix_room_id != parent.matrix_room_id
                            || current_parent.server.binding != parent.server.binding
                        {
                            return Err(chat::stale());
                        }
                    }
                    if s.effect(id)?.1 == store::EffectState::Pending {
                        s.begin_effect(s.generation(), id)?;
                    }
                    Ok(())
                })
            };
            let receipt =
                client.create_with_dispatch(&room, &effect.idempotency_key, pending, mark)?;
            if let Some(parent) = &parent {
                mark()?;
                let child = receipt["matrix_room_id"]
                    .as_str()
                    .ok_or_else(|| invalid("child room missing"))?;
                let completion = json!({"command":effect.idempotency_key,"parent_room_id":parent.id,"project_id":room.project_id});
                match client.state(child, "io.hctl2.topic_creation") {
                    Ok(value) if value == completion => (),
                    Ok(_) => return Err(invalid("Topic creation marker differs")),
                    Err(e) if e.code == "CHAT_NOT_FOUND" => {
                        let space = client.ensure_carrier(parent)?;
                        client.attach(&space, child, true)?;
                        client.put_state(child, "io.hctl2.topic_creation", "", completion)?;
                    }
                    Err(e) => return Err(e),
                }
            }
            receipt
        }
        "chat.send" => {
            guard(&client, root, &room, bound(&room)?)?;
            let body = effect.input["body"]
                .as_str()
                .ok_or_else(|| invalid("send body missing"))?;
            access(shared, |s| {
                let (_, current) = chat::room(s, &room.project_id, &room.id)?;
                chat::writable(s, &current)?;
                let project = chat::required(
                    s,
                    &chat::key(effect.permission_scope.clone(), "project", &room.project_id),
                )?;
                chat::active_project(s, &room.project_id, project.version)?;
                if current.matrix_room_id != room.matrix_room_id {
                    return Err(chat::stale());
                }
                Ok(())
            })?;
            if pending {
                access(shared, |s| {
                    s.begin_effect(s.generation(), id)?;
                    Ok(())
                })?;
            }
            client.send_thread(
                bound(&room)?,
                &effect.idempotency_key,
                body,
                effect.input["thread_root"].as_str(),
            )?
        }
        "chat.members" => {
            guard(&client, root, &room, bound(&room)?)?;
            if pending {
                access(shared, |s| {
                    s.begin_effect(s.generation(), id)?;
                    Ok(())
                })?;
            }
            let users: Vec<String> = serde_json::from_value(effect.input["users"].clone())?;
            client.members(
                bound(&room)?,
                &users,
                effect.input["invite"]
                    .as_bool()
                    .ok_or_else(|| invalid("membership action missing"))?,
                pending,
            )?
        }
        _ => return Err(invalid("not a chat effect")),
    };
    access(shared, |s| {
        // A verified alias may predate this sender attempt. Record delivery before confirmation.
        if s.effect(id)?.1 == store::EffectState::Pending {
            s.begin_effect(s.generation(), id)?;
        }
        chat::confirm(s, actor, id, receipt)
    })
}

pub(crate) fn last_activity(services: &Supervisor, room: &Room) -> Result<Option<u64>> {
    let client = client(services)?;
    if client.server.binding != room.server.binding {
        return Err(chat::stale());
    }
    client.last_activity(bound(room)?)
}
