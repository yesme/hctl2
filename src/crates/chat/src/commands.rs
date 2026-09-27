use crate::*;
use foundation::{bytes_sha256, canonical_json, canonical_json_sha256};
use serde_json::{Value, json};
use store::{
    ActorSource, Command, EffectIntent, Expected, Readback, Record, RecordData, Reference,
    RoomKind, RoomState, Scope, Store, TrustedActor, Version,
};

pub fn owner(actor: &TrustedActor, project: &str) -> Result<TrustedActor> {
    if actor.0.source != ActorSource::DirectClient
        || !actor.0.permission_scope.contains(&Scope::Control)
    {
        return Err(reject(
            "PERMISSION_DENIED",
            "Room commands require the control owner",
            "request_authorization",
        ));
    }
    let mut actor = actor.0.clone();
    actor.permission_scope.push(Scope::Project(project.into()));
    Ok(TrustedActor(actor))
}

fn command_key(input: &Input) -> store::ObjectKey {
    key(
        Scope::Project(input.action.project_id().into()),
        "room_command",
        &bytes_sha256(input.key.as_bytes()),
    )
}

/// Lets the roster owner (package 辛) prepare selections for this exact new Room.
pub fn topic_id(project: &str, command_key: &str) -> String {
    format!(
        "topic-{}",
        bytes_sha256(format!("{project}:{command_key}").as_bytes())
    )
}

pub fn replay(store: &Store, input: &Input) -> Result<Option<Plan>> {
    let Some(r) = store.get(&command_key(input))? else {
        return Ok(None);
    };
    let plan: Plan = decode(&r)?;
    if canonical_json_sha256(&serde_json::to_value(&plan.input)?)?
        != canonical_json_sha256(&serde_json::to_value(input)?)?
    {
        return Err(reject(
            "IDEMPOTENCY_CONFLICT",
            "Room command key has different input",
            "use_original_command",
        ));
    }
    Ok(Some(plan))
}

fn room_record(room: &Room, version: i64, kind: RoomKind, state: RoomState) -> Result<Record> {
    let data = RecordData::Room {
        room_kind: kind,
        state,
    };
    Ok(Record {
        key: key(Scope::Project(room.project_id.clone()), "room", &room.id),
        version,
        revision_digest: canonical_json_sha256(&serde_json::to_value(&data)?)?,
        data,
        sources: vec![],
        materials: vec![],
    })
}

fn effect(
    room: &Room,
    owner: Reference,
    input_key: &str,
    operation: &str,
    input: Value,
) -> Result<EffectIntent> {
    let id = format!(
        "chat:{}",
        bytes_sha256(format!("{}:{input_key}", room.project_id).as_bytes())
    );
    Ok(EffectIntent {
        intent_id: id.clone(),
        owner,
        binding: room.server.binding.clone(),
        operation: operation.into(),
        target: room
            .matrix_room_id
            .clone()
            .unwrap_or_else(|| room.id.clone()),
        conflict_scope: format!("chat:{}:{}", room.project_id, room.id),
        permission_scope: Scope::Project(room.project_id.clone()),
        input_digest: Command::digest_input(operation, &input)?,
        input,
        idempotency_key: id,
    })
}

/// P2.2 辛 admits these records and this effect in its Project-creation transaction.
/// This helper neither creates a Project nor dispatches network I/O.
pub fn main_room(
    project: &Record,
    server: Server,
    name: String,
    command_key: &str,
) -> Result<(Vec<Record>, EffectIntent)> {
    if !matches!(
        project.data,
        RecordData::Project {
            archived: false,
            ..
        }
    ) {
        return Err(invalid("active Project required"));
    }
    let room = Room {
        project_id: project.key.id.clone(),
        id: format!("main-{}", bytes_sha256(project.key.id.as_bytes())),
        name,
        server,
        matrix_room_id: None,
        participants: vec![],
        brief: None,
        origin: None,
    };
    let identity = room_record(&room, 1, RoomKind::Main, RoomState::Active)?;
    let binding = value_record(
        key(project.key.scope.clone(), "room_binding", &room.id),
        1,
        &room,
    )?;
    let effect = effect(
        &room,
        reference(&identity),
        command_key,
        "chat.create",
        json!({"room":room}),
    )?;
    Ok((vec![identity, binding], effect))
}

pub fn prepare(store: &Store, input: Input, source_texts: Vec<SourceText>) -> Result<Plan> {
    if input.key.trim().is_empty() {
        return Err(invalid("command key is required"));
    }
    if let Some(plan) = replay(store, &input)? {
        return Ok(plan);
    }
    let project = input.action.project_id().to_owned();
    let mut plan = Plan {
        input: input.clone(),
        checks: vec![],
        records: vec![],
        effects: vec![],
        source_texts,
        result: json!({}),
    };
    let p = required(
        store,
        &key(Scope::Project(project.clone()), "project", &project),
    )?;
    if !matches!(
        p.data,
        RecordData::Project {
            archived: false,
            ..
        }
    ) {
        return Err(reject(
            "PROJECT_READ_ONLY",
            "Project is archived",
            "restore_project",
        ));
    }
    plan.checks.push(Check {
        key: p.key.clone(),
        version: Some(p.version),
    });
    match input.action {
        Action::CreateTopic {
            project_version,
            name,
            origin,
            brief,
            participants,
            roster_confirmed,
            ..
        } => {
            if project_version != p.version {
                return Err(stale());
            }
            if name.trim().is_empty()
                || brief.context_and_goal.trim().is_empty()
                || !roster_confirmed
            {
                return Err(invalid(
                    "Topic name, confirmed brief and independent roster confirmation required",
                ));
            }
            plan.checks.extend(origin_checks(store, &project, &origin)?);
            validate_sources(&project, &origin, &plan.source_texts)?;
            let resolved: Vec<_> = plan.source_texts.iter().map(|s| s.source.clone()).collect();
            if brief.sources != resolved {
                return Err(invalid(
                    "confirmed brief sources differ from verified sources",
                ));
            }
            let id = topic_id(&project, &input.key);
            // Selection itself is 辛; existing selections from another Room are not inherited.
            for participant in &participants {
                if participant.key.scope != Scope::Project(project.clone())
                    || participant.key.kind != "room_selection"
                {
                    return Err(invalid("Room roster needs exact Project selection records"));
                }
                let selection: Value = decode(&at(store, participant)?)?;
                if selection["room_id"] != id {
                    return Err(invalid("selection belongs to another Room"));
                }
            }
            let (_, main) = main_binding(store, &project)?;
            let room = Room {
                project_id: project.clone(),
                id: id.clone(),
                name,
                server: main.server,
                matrix_room_id: None,
                participants,
                brief: None,
                origin: Some(origin),
            };
            let identity = room_record(&room, 1, RoomKind::Topic, RoomState::Active)?;
            let binding = value_record(key(p.key.scope.clone(), "room_binding", &id), 1, &room)?;
            plan.effects.push(effect(
                &room,
                reference(&identity),
                &input.key,
                "chat.create",
                json!({"room":room}),
            )?);
            plan.records.extend([identity, binding]);
            plan.result = json!({"room_id":id,"project_id":project,"state":"pending","effect_id":plan.effects[0].intent_id});
        }
        Action::Close {
            room_id, version, ..
        } => {
            let (r, room) = room(store, &project, &room_id)?;
            if r.version != version {
                return Err(stale());
            }
            let mut identity = writable(store, &room)?;
            if !matches!(
                identity.data,
                RecordData::Room {
                    room_kind: RoomKind::Topic,
                    ..
                }
            ) {
                return Err(invalid("only a Topic Room may be closed"));
            }
            plan.checks.push(Check {
                key: r.key.clone(),
                version: Some(version),
            });
            identity.version += 1;
            identity.data = RecordData::Room {
                room_kind: RoomKind::Topic,
                state: RoomState::Archived,
            };
            identity.revision_digest =
                canonical_json_sha256(&serde_json::to_value(&identity.data)?)?;
            plan.records
                .extend([identity, value_record(r.key, version + 1, &room)?]);
            plan.result = json!({"room_id":room_id,"state":"archived"});
        }
        Action::Rebind {
            room_id,
            version,
            matrix_room_id,
            ..
        } => {
            let (r, mut room) = room(store, &project, &room_id)?;
            if r.version != version {
                return Err(stale());
            }
            writable(store, &room)?;
            if !matrix_room_id.starts_with('!') {
                return Err(invalid("Matrix room ID required"));
            }
            unique_binding(store, &room.id, &matrix_room_id)?;
            room.matrix_room_id = Some(matrix_room_id);
            plan.checks.push(Check {
                key: r.key.clone(),
                version: Some(version),
            });
            plan.records.push(value_record(r.key, version + 1, &room)?);
            plan.result = json!({"room_id":room_id,"binding_version":version+1});
        }
        Action::Send {
            room_id,
            version,
            body,
            ..
        } => {
            let (r, room) = room(store, &project, &room_id)?;
            if r.version != version {
                return Err(stale());
            }
            writable(store, &room)?;
            if room.matrix_room_id.is_none() || body.is_empty() {
                return Err(invalid("bound Room and nonempty message required"));
            }
            plan.checks.push(Check {
                key: r.key,
                version: Some(version),
            });
            plan.effects.push(effect(
                &room,
                Reference {
                    key: key(p.key.scope.clone(), "room", &room_id),
                    version: Version::State(1),
                },
                &input.key,
                "chat.send",
                json!({"room":room,"body":body}),
            )?);
            plan.result =
                json!({"room_id":room_id,"effect_id":plan.effects[0].intent_id,"state":"pending"});
        }
        Action::Freeze {
            room_id,
            version,
            event_id,
            ..
        } => {
            let (r, _) = room(store, &project, &room_id)?;
            if r.version != version {
                return Err(stale());
            }
            if plan.source_texts.len() != 1
                || !matches!(&plan.source_texts[0].source,
                Source::Message { binding, event_id: id, .. } if *binding == reference(&r) && *id == event_id)
            {
                return Err(invalid("freeze requires the exact read-back event"));
            }
            plan.checks.push(Check {
                key: r.key,
                version: Some(version),
            });
            plan.result = json!({"reference_id":canonical_json_sha256(&serde_json::to_value(&plan.source_texts[0].source)?)?,"source":plan.source_texts[0].source});
        }
        Action::Resume { effect_id, .. } => {
            let (e, _) = store.effect(&effect_id)?;
            if e.permission_scope != p.key.scope || !e.operation.starts_with("chat.") {
                return Err(invalid("effect belongs elsewhere"));
            }
            plan.result = json!({"effect_id":effect_id});
        }
    }
    Ok(plan)
}

pub fn main_binding(store: &Store, project: &str) -> Result<(Record, Room)> {
    let record = store
        .list("room")?
        .into_iter()
        .find(|r| {
            r.key.scope == Scope::Project(project.into())
                && matches!(
                    r.data,
                    RecordData::Room {
                        room_kind: RoomKind::Main,
                        ..
                    }
                )
        })
        .ok_or_else(|| {
            reject(
                "MAIN_ROOM_MISSING",
                "Project has no main Room",
                "inspect_project",
            )
        })?;
    room(store, project, &record.key.id)
}

pub fn writable(store: &Store, room: &Room) -> Result<Record> {
    let r = required(
        store,
        &key(Scope::Project(room.project_id.clone()), "room", &room.id),
    )?;
    if !matches!(
        r.data,
        RecordData::Room {
            state: RoomState::Active,
            ..
        }
    ) {
        return Err(reject(
            "ROOM_READ_ONLY",
            "Room is closed or read-only",
            "choose_active_room",
        ));
    }
    Ok(r)
}

pub fn unique_binding(store: &Store, room_id: &str, external: &str) -> Result<()> {
    for record in store.list("room_binding")? {
        let room: Room = decode(&record)?;
        if room.id != room_id && room.matrix_room_id.as_deref() == Some(external) {
            return Err(reject(
                "ROOM_ALREADY_BOUND",
                "Matrix room already represents another Room",
                "choose_another_room",
            ));
        }
    }
    Ok(())
}

pub fn admit(store: &mut Store, actor: &TrustedActor, mut plan: Plan) -> Result<Value> {
    let actor = owner(actor, plan.input.action.project_id())?;
    if let Some(previous) = replay(store, &plan.input)? {
        return Ok(previous.result);
    }
    let scope = Scope::Project(plan.input.action.project_id().into());
    let mut materials = vec![];
    if let Action::CreateTopic { brief, .. } = &plan.input.action {
        let material = store.save_material(
            store.generation(),
            &actor,
            &scope,
            &plan.input.key,
            "brief",
            &canonical_json(&serde_json::to_value(brief)?)?,
        )?;
        for record in plan
            .records
            .iter_mut()
            .filter(|r| r.key.kind == "room_binding")
        {
            let mut room: Room = decode(record)?;
            room.brief = Some(material.clone());
            *record = value_record(record.key.clone(), record.version, &room)?;
            record.materials.push(material.clone());
        }
        materials.push(material);
    }
    for (i, source) in plan.source_texts.iter().enumerate() {
        let material = store.save_material(
            store.generation(),
            &actor,
            &scope,
            &plan.input.key,
            &format!("source-{i}"),
            source.body.as_bytes(),
        )?;
        let id = canonical_json_sha256(&serde_json::to_value(&source.source)?)?;
        let k = key(scope.clone(), "chat_source_reference", &id);
        if store.get(&k)?.is_none() {
            let mut record = value_record(k, 1, &source.source)?;
            record.materials.push(material.clone());
            plan.records.push(record);
            materials.push(material);
        }
    }
    let input = serde_json::to_value(&plan.input)?;
    let command = Command {
        command_id: format!("room:{}", plan.input.key),
        idempotency_key: plan.input.key.clone(),
        actor: actor.0.clone(),
        target: command_key(&plan.input),
        expected: Expected::Absent,
        binding: Reference {
            key: key(Scope::Control, "module", "chat"),
            version: Version::State(1),
        },
        input_digest: Command::digest_input("room.command", &input)?,
        operation: "room.command".into(),
        input,
    };
    store.submit(store.generation(), &actor, &command, None, |tx| {
        for check in &plan.checks {
            if tx.get(&check.key)?.as_ref().map(|r| r.version) != check.version {
                return Err(stale());
            }
        }
        for m in &materials {
            tx.admit_material(m)?;
        }
        for r in &plan.records {
            tx.put(r)?;
        }
        for e in &plan.effects {
            tx.enqueue_effect(e)?;
        }
        // Keep source references and admitted bytes, not a second authoritative message cache.
        let mut stored = plan.clone();
        stored.source_texts.clear();
        tx.put(&value_record(command.target.clone(), 1, &stored)?)?;
        Ok(plan.result.clone())
    })
}

pub fn confirm(store: &mut Store, actor: &TrustedActor, id: &str, result: Value) -> Result<Value> {
    let (effect, state) = store.effect(id)?;
    if state == store::EffectState::Confirmed {
        return Ok(result);
    }
    let room: Room = serde_json::from_value(effect.input["room"].clone())?;
    let actor = owner(actor, &room.project_id)?;
    let (binding, mut current) = crate::room(store, &room.project_id, &room.id)?;
    let readback = Readback::Confirmed {
        binding: effect.binding.clone(),
        target: effect.target.clone(),
        input_digest: effect.input_digest.clone(),
        result: result.clone(),
    };
    if effect.operation == "chat.create" {
        let external = result["matrix_room_id"]
            .as_str()
            .ok_or_else(|| invalid("room ID missing from readback"))?;
        unique_binding(store, &room.id, external)?;
        if current
            .matrix_room_id
            .as_ref()
            .is_some_and(|r| r != external)
        {
            return Err(stale());
        }
        current.matrix_room_id = Some(external.into());
    }
    let ck = key(binding.key.scope.clone(), "room_effect_receipt", id);
    let input = json!({"effect":id,"result":result});
    let command = Command {
        command_id: format!("confirm:{id}"),
        idempotency_key: format!("confirm:{id}"),
        actor: actor.0.clone(),
        target: ck.clone(),
        expected: Expected::Absent,
        binding: effect.binding,
        input_digest: Command::digest_input("chat.confirm", &input)?,
        operation: "chat.confirm".into(),
        input,
    };
    store.submit(store.generation(), &actor, &command, None, |tx| {
        tx.confirm_effect(id, &readback)?;
        if effect.operation == "chat.create" {
            let mut next = value_record(binding.key.clone(), binding.version + 1, &current)?;
            next.materials = binding.materials.clone();
            tx.put(&next)?;
        }
        tx.put(&value_record(ck, 1, &result)?)?;
        Ok(result.clone())
    })
}
