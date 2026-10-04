use crate::*;
use foundation::{bytes_sha256, canonical_json_sha256};
use serde_json::{Value, json};
use store::{
    Command, Expected, Record, RecordData, Reference, RoomState, Scope, Store, TrustedActor,
    Version,
};

pub fn project_id(store: &Store, command_key: &str) -> String {
    format!(
        "project-{}",
        bytes_sha256(format!("{}:{command_key}", store.control_id()).as_bytes())
    )
}
fn command_key(input: &Input) -> store::ObjectKey {
    key(
        Scope::Control,
        "project_command",
        &bytes_sha256(input.key.as_bytes()),
    )
}
pub fn replay(store: &Store, input: &Input) -> Result<Option<Plan>> {
    let Some(record) = store.get(&command_key(input))? else {
        return Ok(None);
    };
    let plan: Plan = decode(&record)?;
    if canonical_json_sha256(&serde_json::to_value(&plan.input)?)?
        != canonical_json_sha256(&serde_json::to_value(input)?)?
    {
        return Err(reject(
            "IDEMPOTENCY_CONFLICT",
            "Project key has different input",
            "use_original_command",
        ));
    }
    Ok(Some(plan))
}
fn checked(plan: &mut Plan, record: &Record) {
    plan.checks.push(chat::Check {
        key: record.key.clone(),
        version: Some(record.version),
    });
}
fn identity(record: &mut Record) -> Result<()> {
    let value = if record.key.kind == "project" {
        json!({"data":record.data,"sources":record.sources})
    } else {
        serde_json::to_value(&record.data)?
    };
    record.revision_digest = canonical_json_sha256(&value)?;
    Ok(())
}
fn validate_definition(value: &Definition) -> Result<()> {
    if value.name.trim().is_empty()
        || !value.defaults.is_object()
        || !value.settings.selection_policy.is_object()
        || value.roles.iter().any(|r| r.trim().is_empty())
        || value.role_members.keys().any(|r| !value.roles.contains(r))
        || value
            .role_members
            .values()
            .flatten()
            .any(|p| p.trim().is_empty())
    {
        return Err(invalid(
            "name, object policies and declared role assignments required",
        ));
    }
    Ok(())
}

pub fn prepare(
    store: &Store,
    input: Input,
    server: Option<chat::Server>,
    actor: &TrustedActor,
    now: u64,
) -> Result<Plan> {
    if input.key.trim().is_empty() {
        return Err(invalid("command key required"));
    }
    if let Some(plan) = replay(store, &input)? {
        return Ok(plan);
    }
    let id = input
        .action
        .project_id()
        .map(str::to_owned)
        .unwrap_or_else(|| project_id(store, &input.key));
    let scope = Scope::Project(id.clone());
    let mut plan = Plan {
        input: input.clone(),
        project_id: id.clone(),
        checks: vec![],
        records: vec![],
        effects: vec![],
        task_plan: None,
        result: json!({}),
    };
    let existing = if matches!(input.action, Action::Create { .. }) {
        None
    } else {
        let p = project(store, &id)?;
        checked(&mut plan, &p);
        if readonly(&p) && !matches!(input.action, Action::Restore { .. } | Action::Resume { .. }) {
            return Err(reject(
                "PROJECT_READ_ONLY",
                "Project is archived",
                "restore_project",
            ));
        }
        Some(p)
    };
    let owner = chat::owner(actor, &id)?;
    match &input.action {
        Action::Create {
            repo_id,
            definition,
        } => {
            validate_definition(definition)?;
            repo::require_active(store, repo_id)?;
            checked(&mut plan, &required(store, &repo::key(repo_id))?);
            let mut p = value_record(key(scope.clone(), "project", &id), 1, &json!({}))?;
            p.data = RecordData::Project {
                repo_id: repo_id.clone(),
                settings: definition.settings.clone(),
                archived: false,
            };
            let details = value_record(key(scope.clone(), "project_details", &id), 1, definition)?;
            p.sources = vec![reference(&details)];
            identity(&mut p)?;
            plan.checks.push(chat::Check {
                key: p.key.clone(),
                version: None,
            });
            let (rooms, effect) = chat::main_room(
                &p,
                server.ok_or_else(|| invalid("chat binding required"))?,
                definition.name.clone(),
                &input.key,
            )?;
            let main_id = rooms[0].key.id.clone();
            plan.records.push(p);
            plan.records.push(details);
            plan.records.extend(rooms);
            plan.records.push(value_record(
                key(scope.clone(), "room_created", &main_id),
                1,
                &json!({"created_at":now}),
            )?);
            plan.effects.push(effect);
            plan.result = json!({"project_id":id,"version":1,"main_room_id":main_id,"effect_id":plan.effects[0].intent_id,"state":"pending"});
        }
        Action::Update {
            version,
            definition,
            ..
        } => {
            validate_definition(definition)?;
            let mut p = existing.clone().ok_or_else(stale)?;
            if p.version != *version {
                return Err(stale());
            }
            let RecordData::Project { settings, .. } = &mut p.data else {
                return Err(invalid("Project required"));
            };
            *settings = definition.settings.clone();
            p.version += 1;
            let details = required(store, &key(scope.clone(), "project_details", &id))?;
            checked(&mut plan, &details);
            let next_details = value_record(details.key, details.version + 1, definition)?;
            p.sources = vec![reference(&next_details)];
            identity(&mut p)?;
            plan.records.push(next_details);
            plan.result = json!({"project_id":id,"version":p.version});
            plan.records.push(p);
        }
        Action::Archive { version, .. } | Action::Restore { version, .. } => {
            let restore = matches!(input.action, Action::Restore { .. });
            let mut p = existing.clone().ok_or_else(stale)?;
            if p.version != *version || readonly(&p) != restore {
                return Err(stale());
            }
            let blockers = archive_blockers(store, &id)?;
            if !blockers.is_empty() {
                return Err(reject(
                    "PROJECT_BUSY",
                    serde_json::to_string(&blockers)?,
                    "settle_project_blockers",
                ));
            }
            let archive_key = key(scope.clone(), "project_archive_rooms", &id);
            let saved = store.get(&archive_key)?;
            let saved_rooms: Vec<String> =
                saved.as_ref().map(decode).transpose()?.unwrap_or_default();
            let mut changed = vec![];
            // Ownership is the complete set. Matrix trees are neither complete nor unique.
            for mut room in store
                .list("room")?
                .into_iter()
                .filter(|r| r.key.scope == scope)
            {
                checked(&mut plan, &room);
                let RecordData::Room { state, .. } = &mut room.data else {
                    return Err(invalid("invalid Room identity"));
                };
                if (!restore && *state == RoomState::Active)
                    || (restore
                        && saved_rooms.contains(&room.key.id)
                        && *state == RoomState::ReadOnly)
                {
                    *state = if restore {
                        RoomState::Active
                    } else {
                        RoomState::ReadOnly
                    };
                    room.version += 1;
                    changed.push(room.key.id.clone());
                    identity(&mut room)?;
                    plan.records.push(room);
                }
            }
            plan.checks.push(chat::Check {
                key: archive_key.clone(),
                version: saved.as_ref().map(|r| r.version),
            });
            plan.records.push(value_record(
                archive_key,
                saved.map_or(1, |r| r.version + 1),
                &if restore { vec![] } else { changed.clone() },
            )?);
            let RecordData::Project { archived, .. } = &mut p.data else {
                return Err(invalid("Project required"));
            };
            *archived = !restore;
            p.version += 1;
            identity(&mut p)?;
            plan.result = json!({"project_id":id,"version":p.version,"archived":!restore,"rooms":changed,"blockers":blockers});
            plan.records.push(p);
        }
        Action::Select {
            project_version,
            room_id,
            topic_command_key,
            roster_version,
            selections,
            ..
        } => {
            if existing.as_ref().map(|p| p.version) != Some(*project_version) {
                return Err(stale());
            }
            let room = store.get(&key(scope.clone(), "room", room_id))?;
            if let Some(room) = room {
                if !matches!(
                    room.data,
                    RecordData::Room {
                        state: RoomState::Active,
                        ..
                    }
                ) {
                    return Err(invalid("active Room required"));
                }
                checked(&mut plan, &room);
                let mut next_room = room;
                next_room.version += 1;
                plan.records.push(next_room);
            } else if !topic_command_key
                .as_ref()
                .is_some_and(|k| chat::topic_id(&id, k) == *room_id)
            {
                return Err(invalid(
                    "selection needs existing Room or exact future Topic command",
                ));
            }
            let roster_key = key(scope.clone(), "room_roster", room_id);
            let roster = store.get(&roster_key)?;
            if roster.as_ref().map(|r| r.version) != *roster_version {
                return Err(stale());
            }
            plan.checks.push(chat::Check {
                key: roster_key.clone(),
                version: *roster_version,
            });
            let next = roster_version.map_or(1, |v| v + 1);
            let validated = participant::selection::validate_roster(
                store,
                existing.as_ref().ok_or_else(stale)?,
                selections,
            )?;
            for dependency in &validated.dependencies {
                checked(&mut plan, dependency);
            }
            let mut refs = vec![];
            for (index, s) in selections.iter().enumerate() {
                validate_selection(s, room_id)?;
                let selection_id = format!("{room_id}:{next}:{index}");
                let mut record =
                    value_record(key(scope.clone(), "room_selection", &selection_id), 1, s)?;
                record.sources = vec![
                    s.selected_item.clone(),
                    s.profession.clone(),
                    s.agency.clone(),
                ];
                refs.push(reference(&record));
                plan.records.push(record);
            }
            plan.records.push(value_record(roster_key, next, &refs)?);
            plan.result = json!({"room_id":room_id,"roster_version":next,"selections":refs,"candidate_validation":"accepted_catalog_and_project_policy","optional_skill_degradations":validated.optional_skill_degradations});
        }
        Action::Members {
            project_version,
            rooms,
            users,
            invite,
            ..
        } => {
            if existing.as_ref().map(|p| p.version) != Some(*project_version) {
                return Err(stale());
            }
            if rooms.is_empty() || users.is_empty() {
                return Err(invalid("explicit Rooms and Matrix user IDs required"));
            }
            for (index, target) in rooms.iter().enumerate() {
                if target.key.scope != scope
                    || target.key.kind != "room_binding"
                    || rooms[..index].contains(target)
                {
                    return Err(invalid("unique Room targets in this Project required"));
                }
                let (binding, room) = chat::room(store, &id, &target.key.id)?;
                if reference(&binding) != *target {
                    return Err(stale());
                }
                let identity = chat::writable(store, &room)?;
                checked(&mut plan, &binding);
                checked(&mut plan, &identity);
                let effect_id = format!(
                    "chat:{}",
                    bytes_sha256(format!("{id}:{}:{}", input.key, room.id).as_bytes())
                );
                let payload = json!({"room":room,"users":users,"invite":invite});
                let effect = store::EffectIntent {
                    intent_id: effect_id.clone(),
                    owner: reference(&identity),
                    binding: room.server.binding.clone(),
                    operation: "chat.members".into(),
                    target: room
                        .matrix_room_id
                        .clone()
                        .ok_or_else(|| invalid("bound Room required"))?,
                    conflict_scope: format!("chat:{id}:{}", room.id),
                    permission_scope: scope.clone(),
                    input_digest: Command::digest_input("chat.members", &payload)?,
                    input: payload,
                    idempotency_key: effect_id,
                };
                plan.effects.push(effect);
            }
            plan.result = json!({"project_id":id,"effect_ids":plan.effects.iter().map(|e|&e.intent_id).collect::<Vec<_>>(),"delivery":"pending"});
        }
        Action::CreateRequest {
            project_version,
            request,
            ..
        } => {
            if existing.as_ref().map(|p| p.version) != Some(*project_version) {
                return Err(stale());
            }
            prepare_request(store, &owner, &mut plan, request, now)?;
        }
        Action::ResolveRequest {
            request_id,
            version,
            adoption,
            ..
        } => {
            prepare_resolution(
                store, &owner, &mut plan, request_id, *version, adoption, now,
            )?;
        }
        Action::CancelRequest {
            request_id,
            version,
            ..
        } => {
            cancel_request(store, &mut plan, request_id, *version)?;
        }
        Action::Resume { effect_id, .. } => {
            let effect = store.effect(effect_id)?.0;
            if effect.permission_scope != scope
                || !(effect.operation.starts_with("chat.")
                    || effect.operation == "request.task_adopt")
            {
                return Err(invalid("effect is not this Project's effect"));
            }
            plan.result = json!({"effect_id":effect_id});
        }
    }
    Ok(plan)
}

fn validate_selection(s: &Selection, room_id: &str) -> Result<()> {
    if s.room_id != room_id
        || s.responsibility.trim().is_empty()
        || s.display_name.trim().is_empty()
        || s.worker_profiles.is_empty()
        || !s.permission.is_object()
        || !s.budget.is_object()
    {
        return Err(invalid(
            "complete, independently scoped Room selection required",
        ));
    }
    s.selected_item.validate()?;
    s.profession.validate()?;
    s.agency.validate()?;
    for p in &s.worker_profiles {
        p.validate()?;
    }
    for digest in std::iter::once(&s.profession_digest).chain(
        s.required_skills
            .iter()
            .chain(&s.optional_skills)
            .filter_map(|s| s.digest.as_ref()),
    ) {
        if digest.len() != 64
            || !digest
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err(invalid("exact lowercase SHA-256 required"));
        }
    }
    for skill in s.required_skills.iter().chain(&s.optional_skills) {
        skill.reference.validate()?;
    }
    Ok(())
}

pub fn admit(store: &mut Store, actor: &TrustedActor, mut plan: Plan) -> Result<Value> {
    let actor = chat::owner(actor, &plan.project_id)?;
    if let Some(previous) = replay(store, &plan.input)? {
        return Ok(previous.result);
    }
    if let Action::ResolveRequest { request_id, .. } = &plan.input.action {
        let r = required(
            store,
            &key(
                Scope::Project(plan.project_id.clone()),
                "request",
                request_id,
            ),
        )?;
        if decode::<Request>(&r)?
            .spec
            .deadline
            .is_some_and(|d| d <= task::now())
        {
            return Err(stale());
        }
    }
    // Membership and pending effects can change without changing Project.version.
    if matches!(
        plan.input.action,
        Action::Archive { .. }
            | Action::Restore { .. }
            | Action::CreateRequest { .. }
            | Action::ResolveRequest { .. }
    ) {
        let current = prepare(store, plan.input.clone(), None, &actor, task::now())?;
        if serde_json::to_value(&current.checks)? != serde_json::to_value(&plan.checks)?
            || current.result != plan.result
        {
            return Err(stale());
        }
    }
    if let Some(task) = &mut plan.task_plan {
        task::materialize(store, &actor, task)?;
        let effect = plan.effects.first_mut().ok_or_else(stale)?;
        effect.input = json!({"task_plan":task,"request_id":plan.result["request_id"]});
        effect.input_digest = Command::digest_input(&effect.operation, &effect.input)?;
        // Sealed contract remains admitted even if delivery has to wait for restart.
        for record in &task.records {
            for material in &record.materials {
                plan.records
                    .iter_mut()
                    .find(|r| r.key.kind == "request")
                    .ok_or_else(stale)?
                    .materials
                    .push(material.clone());
            }
        }
    }
    let input = serde_json::to_value(&plan.input)?;
    let command = Command {
        command_id: format!("project:{}", plan.input.key),
        idempotency_key: plan.input.key.clone(),
        actor: actor.0.clone(),
        target: command_key(&plan.input),
        expected: Expected::Absent,
        binding: Reference {
            key: key(Scope::Control, "module", "project"),
            version: Version::State(1),
        },
        input_digest: Command::digest_input("project.command", &input)?,
        operation: "project.command".into(),
        input,
    };
    store.submit(store.generation(), &actor, &command, None, |tx| {
        for c in &plan.checks {
            if tx.get(&c.key)?.as_ref().map(|r| r.version) != c.version {
                return Err(stale());
            }
        }
        for r in &plan.records {
            for m in &r.materials {
                tx.admit_material(m)?;
            }
            tx.put(r)?;
        }
        for effect in &plan.effects {
            tx.enqueue_effect(effect)?;
        }
        tx.put(&value_record(command.target.clone(), 1, &plan)?)?;
        Ok(plan.result.clone())
    })
}
