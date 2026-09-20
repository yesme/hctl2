use crate::*;
use foundation::{bytes_sha256, canonical_json_sha256};
use serde_json::{Value, json};
use store::{
    Actor, ActorSource, Command, EffectState, Expected, Readback, Record, RecordData, Reference,
    Scope, Store, TrustedActor, Version,
};

fn observer(authority: Reference, scopes: Vec<Scope>) -> TrustedActor {
    TrustedActor(Actor {
        principal: "task-source-reconciler".into(),
        source: ActorSource::InternalReducer,
        permission_scope: scopes,
        authority: Some(authority),
    })
}

fn command(
    actor: &TrustedActor,
    target: &Record,
    operation: &str,
    input: Value,
) -> Result<Command> {
    let digest = Command::digest_input(operation, &input)?;
    Ok(Command {
        command_id: format!("{operation}:{}:{}:{digest}", target.key.id, target.version),
        idempotency_key: format!("{operation}:{}:{}:{digest}", target.key.id, target.version),
        actor: actor.0.clone(),
        target: target.key.clone(),
        expected: if target.version == 1 {
            Expected::Absent
        } else {
            Expected::Exact(Version::State(target.version - 1))
        },
        binding: actor.0.authority.clone().unwrap(),
        input_digest: digest,
        operation: operation.into(),
        input,
    })
}

/// A full read or an explicit failure; an unavailable board never becomes an empty success.
pub fn observe(store: &mut Store, src: &Source, mut snap: Snapshot) -> Result<()> {
    let (sr, current) = source(store, &src.repo_id, &src.id)?;
    if !current.active || snap.source != reference(&sr) {
        return Err(stale());
    }
    let k = key(Scope::Repo(src.repo_id.clone()), "task_snapshot", &src.id);
    let old = store.get(&k)?;
    if !snap.complete {
        // Preserve last observations on failures. Consumers still see the incomplete flag.
        if let Some(r) = &old {
            snap.cards = decode::<Snapshot>(r)?.cards;
        }
    } else if let Some(r) = &old {
        for mut c in decode::<Snapshot>(r)?.cards {
            if !snap.cards.iter().any(|new| new.entity == c.entity) {
                c.tombstone = true;
                snap.cards.push(c);
            }
        }
    }
    let record = value_record(k, old.map_or(1, |r| r.version + 1), &snap)?;
    let mut changes = vec![record.clone()];
    let mut scopes = vec![Scope::Repo(src.repo_id.clone())];
    for (r, mut t) in tasks(store)?
        .into_iter()
        .filter(|(_, t)| t.source_id == src.id)
    {
        let found = t
            .entity
            .as_ref()
            .and_then(|e| snap.cards.iter().find(|c| &c.entity == e));
        t.needs_attention =
            !snap.complete || snap.error.is_some() || found.is_none_or(|c| c.tombstone);
        if let Some(expected) = t
            .revision
            .as_ref()
            .and_then(|r| r.backend_projection_digest.as_ref())
        {
            t.pending_contract = match found {
                Some(c) if !c.tombstone && contract_projection(c)? != *expected => {
                    Some(reference(&record))
                }
                _ => None,
            };
            t.needs_attention |= t.pending_contract.is_some();
        }
        t.snapshot = Some(reference(&record));
        t.state_version += 1;
        // Task's contract/title/lifecycle are governance, not overwritten by card fields.
        scopes.push(r.key.scope.clone());
        changes.push(value_record(r.key, r.version + 1, &t)?);
    }
    if snap.complete && snap.error.is_none() {
        for r in store.list("task_source_reference")? {
            let approved: SourceReference = decode(&r)?;
            if approved.source.key != sr.key {
                continue;
            }
            let Some(group) = approved.group.as_ref() else {
                continue;
            };
            if !snap.stable_groups.contains(group) {
                continue;
            }
            let pr = required(
                store,
                &key(
                    Scope::Project(approved.project_id.clone()),
                    "project",
                    &approved.project_id,
                ),
            )?;
            if !matches!(
                pr.data,
                RecordData::Project {
                    archived: false,
                    ..
                }
            ) {
                continue;
            }
            let existing = tasks(store)?;
            for c in snap
                .cards
                .iter()
                .filter(|c| !c.tombstone && c.groups.contains(group))
            {
                if existing.iter().any(|(_, t)| {
                    t.project_id == approved.project_id && t.entity.as_ref() == Some(&c.entity)
                }) {
                    continue;
                }
                if crate::planning::pending_creation(store, &approved.project_id, &src.id, c)?
                    .is_some()
                {
                    continue;
                }
                let mut t = crate::planning::new_task(
                    store,
                    &approved.project_id,
                    src,
                    &canonical_json_sha256(&serde_json::to_value(&c.entity)?)?,
                    &c.title,
                );
                t.entity = Some(c.entity.clone());
                t.number = Some(c.number);
                t.snapshot = Some(reference(&record));
                t.state_version = 1;
                let scope = Scope::Project(t.project_id.clone());
                scopes.push(scope.clone());
                changes.push(value_record(
                    key(scope.clone(), "task_state", &t.id),
                    1,
                    &t,
                )?);
                let data = RecordData::Task {
                    entity: t.entity.clone(),
                    source: reference(&sr),
                };
                changes.push(Record {
                    key: key(scope, "task", &t.id),
                    version: 1,
                    revision_digest: canonical_json_sha256(&serde_json::to_value(&data)?)?,
                    data,
                    sources: vec![reference(&r)],
                    materials: vec![],
                });
            }
        }
    }
    let actor = observer(reference(&sr), scopes);
    let cmd = command(
        &actor,
        &record,
        "task.observe",
        serde_json::to_value(&snap)?,
    )?;
    store.submit(store.generation(), &actor, &cmd, None, |tx| {
        for r in &changes {
            tx.put(r)?;
        }
        Ok(json!({"snapshot":reference(&record)}))
    })?;
    Ok(())
}

/// Recheck the original authorization before dispatch, including after control restart.
pub fn begin(
    store: &mut Store,
    actor: &TrustedActor,
    id: &str,
) -> Result<(store::EffectIntent, bool)> {
    let (e, state) = store.effect(id)?;
    owner(actor, [e.permission_scope.clone()])?;
    if !e.operation.starts_with("task.") {
        return Err(stale());
    }
    if state == EffectState::Confirmed {
        return Ok((e, false));
    }
    let (_, t) = task(
        store,
        e.input["project_id"].as_str().ok_or_else(stale)?,
        e.input["task_id"].as_str().ok_or_else(stale)?,
    )?;
    let (sr, src) = source(store, &t.repo_id, &t.source_id)?;
    let pr = required(
        store,
        &key(
            Scope::Project(t.project_id.clone()),
            "project",
            &t.project_id,
        ),
    )?;
    if state == EffectState::Pending {
        if e.operation == "task.delete"
            && e.input["write"]["affected"] != crate::planning::deletion_consequences(store, &t)?
        {
            return Err(reject(
                "VERSION_CONFLICT",
                "delete consequences changed before dispatch",
                "cancel_intent_then_preview_again",
            ));
        }
        if !src.active
            || reference(&sr) != e.binding
            || !matches!(
                pr.data,
                RecordData::Project {
                    archived: false,
                    ..
                }
            )
            || t.lifecycle != "open"
        {
            return Err(reject(
                "PERMISSION_DENIED",
                "original write authorization no longer applies",
                "inspect_original_intent",
            ));
        }
        store.resume_pending_effect(store.generation(), id, true)?;
        store.begin_effect(store.generation(), id)?;
        return Ok((e, true));
    }
    Ok((e, false))
}

/// Exact adapter readback is committed with Task identity/projection changes.
pub fn confirm(store: &mut Store, effect_id: &str, observed: &Card) -> Result<()> {
    let (e, state) = store.effect(effect_id)?;
    if state == EffectState::Confirmed {
        return Ok(());
    }
    let (old, mut t) = task(
        store,
        e.input["project_id"].as_str().ok_or_else(stale)?,
        e.input["task_id"].as_str().ok_or_else(stale)?,
    )?;
    if t.entity.as_ref().is_some_and(|v| v != &observed.entity) {
        return Err(reject(
            "ENTITY_CHANGED",
            "readback followed another entity",
            "inspect_original_card",
        ));
    }
    t.entity = Some(observed.entity.clone());
    t.number = Some(observed.number);
    t.needs_attention = false;
    let r = value_record(old.key, old.version + 1, &t)?;
    let identity = required(
        store,
        &key(Scope::Project(t.project_id.clone()), "task", &t.id),
    )?;
    let data = RecordData::Task {
        entity: t.entity.clone(),
        source: e.binding.clone(),
    };
    let identity = Record {
        version: identity.version + 1,
        revision_digest: canonical_json_sha256(&serde_json::to_value(&data)?)?,
        data,
        ..identity
    };
    let actor = observer(e.owner.clone(), vec![e.permission_scope.clone()]);
    let mut cmd = command(
        &actor,
        &r,
        "task.confirm",
        json!({"effect":effect_id,"card":observed}),
    )?;
    cmd.idempotency_key = format!("task.confirm:{effect_id}");
    cmd.command_id = cmd.idempotency_key.clone();
    store.submit(store.generation(), &actor, &cmd, None, |tx| {
        tx.put(&r)?;
        tx.put(&identity)?;
        tx.confirm_effect(
            effect_id,
            &Readback::Confirmed {
                binding: e.binding.clone(),
                target: e.target.clone(),
                input_digest: e.input_digest.clone(),
                result: serde_json::to_value(observed)?,
            },
        )?;
        Ok(json!({"confirmed":effect_id}))
    })?;
    Ok(())
}

pub fn board(store: &Store, project_id: &str, source_id: &str) -> Result<Value> {
    let approved = required(
        store,
        &key(
            Scope::Project(project_id.into()),
            "task_source_reference",
            source_id,
        ),
    )?;
    let approved: SourceReference = decode(&approved)?;
    let source_record = required(store, &approved.source.key)?;
    let src: Source = decode(&source_record)?;
    let snapshot = store.get(&key(
        Scope::Repo(src.repo_id.clone()),
        "task_snapshot",
        &src.id,
    ))?;
    let all = tasks(store)?;
    let snapshot: Option<Snapshot> = snapshot.as_ref().map(decode).transpose()?;
    let cards = snapshot
        .as_ref()
        .map(|s| {
            s.cards
                .iter()
                .map(|c| {
                    let task = all
                        .iter()
                        .find(|(_, t)| {
                            t.project_id == project_id && t.entity.as_ref() == Some(&c.entity)
                        })
                        .map(|(_, t)| t);
                    json!({"card":c,"task":task,"claimed":task.is_some()})
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(
        json!({"source":src,"complete":src.active && snapshot.as_ref().is_some_and(|s|s.complete && s.error.is_none()),"snapshot":snapshot,"cards":cards}),
    )
}

pub fn source_id_for_action(store: &Store, action: &Action) -> Result<Option<(String, String)>> {
    Ok(match action {
        Action::Attach {
            project_id,
            source_id,
            ..
        }
        | Action::Claim {
            project_id,
            source_id,
            ..
        }
        | Action::Create {
            project_id,
            source_id,
            ..
        } => {
            let r = required(
                store,
                &key(Scope::Project(project_id.clone()), "project", project_id),
            )?;
            let RecordData::Project { repo_id, .. } = r.data else {
                return Err(stale());
            };
            Some((repo_id, source_id.clone()))
        }
        Action::Adopt {
            project_id,
            task_id,
            adoption:
                Adoption {
                    origin: ContractOrigin::Backend { .. },
                    ..
                },
            ..
        }
        | Action::Update {
            project_id,
            task_id,
            ..
        }
        | Action::Move {
            project_id,
            task_id,
            ..
        }
        | Action::DeleteCard {
            project_id,
            task_id,
            ..
        } => {
            let (_, t) = task(store, project_id, task_id)?;
            Some((t.repo_id, t.source_id))
        }
        Action::Refresh { repo_id, source_id } => Some((repo_id.clone(), source_id.clone())),
        _ => None,
    })
}

pub fn content_digest(snap: &Snapshot) -> Result<String> {
    let cards = snap
        .cards
        .iter()
        .filter(|c| !c.tombstone)
        .collect::<Vec<_>>();
    Ok(bytes_sha256(&foundation::canonical_json(
        &json!({"cards":cards,"groups":snap.stable_groups,"complete":snap.complete,"error":snap.error}),
    )?))
}

pub fn contract_projection(card: &Card) -> Result<String> {
    // Stages, health and relation changes do not create contract drift.
    Ok(canonical_json_sha256(
        &json!({"title":card.title,"body":card.body,"comments":card.comments}),
    )?)
}
