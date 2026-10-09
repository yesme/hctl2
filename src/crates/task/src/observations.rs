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
    let previous: Option<Snapshot> = old.as_ref().map(decode).transpose()?;
    if !snap.complete {
        // Preserve last observations on failures. Consumers still see the incomplete flag.
        if let Some(previous) = &previous {
            snap.cards = previous.cards.clone();
            snap.stable_groups = previous.stable_groups.clone();
        }
    } else if let Some(r) = &old {
        for mut c in decode::<Snapshot>(r)?.cards {
            if !snap.cards.iter().any(|new| new.entity == c.entity) {
                c.tombstone = true;
                snap.cards.push(c);
            }
        }
    }
    let unchanged = match &previous {
        Some(previous) => {
            previous.source == snap.source && content_digest(previous)? == content_digest(&snap)?
        }
        None => false,
    };
    let record = if unchanged {
        old.clone().unwrap()
    } else {
        value_record(k, old.as_ref().map_or(1, |r| r.version + 1), &snap)?
    };
    let mut changes = if unchanged {
        vec![]
    } else {
        vec![record.clone()]
    };
    let mut scopes = vec![Scope::Repo(src.repo_id.clone())];
    for (r, mut t) in tasks(store)?
        .into_iter()
        .filter(|(_, t)| t.source_id == src.id && t.repo_id == src.repo_id)
    {
        let found = t
            .entity
            .as_ref()
            .and_then(|e| snap.cards.iter().find(|c| &c.entity == e));
        let old_card = previous.as_ref().and_then(|previous| {
            t.entity
                .as_ref()
                .and_then(|e| previous.cards.iter().find(|c| &c.entity == e))
        });
        let same_card = t.snapshot.is_some()
            && previous.as_ref().is_some_and(|previous| {
                previous.source == snap.source
                    && previous.complete == snap.complete
                    && previous.error == snap.error
            })
            && serde_json::to_value(old_card)? == serde_json::to_value(found)?;
        let old_attention = t.needs_attention;
        let old_pending = t.pending_contract.clone();
        t.needs_attention =
            !snap.complete || snap.error.is_some() || found.is_none_or(|c| c.tombstone);
        if let Some(expected) = t
            .revision
            .as_ref()
            .and_then(|r| r.backend_projection_digest.as_ref())
        {
            t.pending_contract = match found {
                Some(c) if !c.tombstone && contract_projection(store, c)? != *expected => {
                    if same_card {
                        old_pending.clone().or_else(|| Some(reference(&record)))
                    } else {
                        Some(reference(&record))
                    }
                }
                _ => None,
            };
            t.needs_attention |= t.pending_contract.is_some();
        }
        if same_card && t.needs_attention == old_attention && t.pending_contract == old_pending {
            continue;
        }
        // 供应端 Done：绑定声明了归属 human 的账号，且卡片由它从非终态推进到终态，
        // 归档一次「完成 Task」请求。HCTL 服务账号写回、未知 actor、只见当前 Done、
        // 重复或迟到的投递都不产生新请求（同一幂等键落到同一条记录）。
        if let (Some(card), Some(account)) = (found, src.human_account.as_deref())
            && !card.tombstone
            && terminal_stage(&card.stage)
            && old_card.is_some_and(|c| !terminal_stage(&c.stage))
            && closer_of(card).as_deref() == Some(account)
            && account != src.platform.account_id
        {
            let tuple = json!([
                current.binding_revision,
                card.entity.immutable_external_entity_id,
                t.id,
                account,
                old_card.map(|c| c.stage.clone()).unwrap_or_default(),
                card.stage,
                card.remote_revision,
            ]);
            let idempotency_key =
                foundation::bytes_sha256(foundation::canonical_json(&tuple)?.as_slice());
            let request_key = key(
                Scope::Project(t.project_id.clone()),
                COMPLETION_REQUEST_KIND,
                &idempotency_key,
            );
            if store.get(&request_key)?.is_none() {
                let request = CompletionRequest {
                    request_id: idempotency_key.clone(),
                    task_id: t.id.clone(),
                    project_id: t.project_id.clone(),
                    repo_id: src.repo_id.clone(),
                    source_id: src.id.clone(),
                    binding: reference(&sr),
                    entity: card.entity.immutable_external_entity_id.clone(),
                    provider_actor: account.to_owned(),
                    stage_before: old_card.map(|c| c.stage.clone()).unwrap_or_default(),
                    stage_after: card.stage.clone(),
                    remote_revision: card.remote_revision.clone(),
                    idempotency_key: idempotency_key.clone(),
                    snapshot: reference(&record),
                    observed_at: snap.observed_at,
                    state: "pending".into(),
                    last_error: None,
                };
                scopes.push(request_key.scope.clone());
                changes.push(value_record(request_key, 1, &request)?);
            }
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
    if changes.is_empty() {
        return Ok(());
    }
    // New approved mappings may claim cards even when the source content is unchanged.
    // Use a fresh snapshot record as that transaction's compare-and-swap target.
    let record = if unchanged {
        let mut record = record;
        record.version += 1;
        changes.push(record.clone());
        record
    } else {
        record
    };
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
            || (t.lifecycle != "open" && e.operation != "task.delete")
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
    t.needs_attention = observed.tombstone || t.pending_contract.is_some();
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
        let own_comments = if e.input["write"]["fields"]["comment"].is_string() {
            observed.comments.clone()
        } else {
            vec![]
        };
        tx.put(&value_record(
            key(e.permission_scope.clone(), "task_effect_result", effect_id),
            1,
            &json!({"outcome":"confirmed","card":observed,"own_comments":own_comments}),
        )?)?;
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

/// Adapter-only evidence: a failed read or absence is not a rejection receipt.
pub fn reject_effect(store: &mut Store, id: &str, evidence: Value) -> Result<()> {
    let (e, _) = store.effect(id)?;
    let record = value_record(
        key(e.permission_scope.clone(), "task_effect_result", id),
        1,
        &evidence,
    )?;
    let actor = observer(e.owner.clone(), vec![e.permission_scope.clone()]);
    let cmd = command(&actor, &record, "task.rejected", evidence.clone())?;
    store.submit(store.generation(), &actor, &cmd, None, |tx| {
        tx.confirm_effect(
            id,
            &Readback::Rejected {
                binding: e.binding.clone(),
                target: e.target.clone(),
                input_digest: e.input_digest.clone(),
                result: evidence.clone(),
            },
        )?;
        tx.put(&record)?;
        Ok(json!({"rejected":id}))
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

/// Action 声明的项目作用域；不带项目的动作返回 `None`。
pub fn project_id_for_action(action: &Action) -> Option<&str> {
    match action {
        Action::Attach { project_id, .. }
        | Action::Claim { project_id, .. }
        | Action::Create { project_id, .. }
        | Action::Adopt { project_id, .. }
        | Action::Update { project_id, .. }
        | Action::Move { project_id, .. }
        | Action::Cancel { project_id, .. }
        | Action::Complete { project_id, .. }
        | Action::Reopen { project_id, .. }
        | Action::DeleteCard { project_id, .. } => Some(project_id.as_str()),
        _ => None,
    }
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

/// 归档请求的状态推进（`accepted` / `declined`）；重复或迟到的观测只更新快照。
pub fn mark_completion_request(
    store: &mut Store,
    project: &str,
    request_id: &str,
    state: &str,
    error: Option<Value>,
) -> Result<()> {
    let k = key(
        Scope::Project(project.into()),
        COMPLETION_REQUEST_KIND,
        request_id,
    );
    let record = required(store, &k)?;
    let mut request: CompletionRequest = decode(&record)?;
    request.state = state.into();
    request.last_error = error;
    let actor = observer(
        Reference {
            key: k.clone(),
            version: Version::State(record.version),
        },
        vec![k.scope.clone()],
    );
    let value = serde_json::to_value(&request)?;
    let cmd = store::Command {
        command_id: format!("task-completion-request:{}", k.id),
        idempotency_key: format!("task-completion-request:{}", k.id),
        actor: actor.0.clone(),
        target: k.clone(),
        expected: store::Expected::Exact(Version::State(record.version)),
        binding: Reference {
            key: k.clone(),
            version: Version::State(record.version),
        },
        input_digest: store::Command::digest_input("task.completion_request", &value)?,
        operation: "task.completion_request".into(),
        input: value,
    };
    store.submit(store.generation(), &actor, &cmd, None, |tx| {
        tx.put(&value_record(k.clone(), record.version + 1, &request)?)?;
        Ok(json!({"request":k.id}))
    })?;
    Ok(())
}

/// 供应端把卡片推进到的终态名（issue 源只有 open/closed）。
fn terminal_stage(stage: &str) -> bool {
    matches!(stage, "closed" | "done" | "completed")
}

/// 卡片的关闭者：Gitea 报 `username`，GitHub 报 `login`。
fn closer_of(card: &Card) -> Option<String> {
    let closed = card.raw.get("closed_by")?;
    closed
        .get("login")
        .or_else(|| closed.get("username"))
        .and_then(Value::as_str)
        .map(str::to_owned)
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

pub fn contract_projection(store: &Store, card: &Card) -> Result<String> {
    // A marker alone is forgeable. Exclude only exact comments recorded in the
    // adapter's confirmed readback, including their native IDs and original contents.
    let mut own_comments = vec![];
    for record in store.list("task_effect_result")? {
        let value: Value = decode(&record)?;
        if value["outcome"] == "confirmed" {
            let observed: Card = serde_json::from_value(value["card"].clone())?;
            if observed.entity == card.entity {
                own_comments.extend(
                    value["own_comments"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .cloned(),
                );
            }
        }
    }
    let comments = card
        .comments
        .iter()
        .filter(|comment| !own_comments.contains(comment))
        .collect::<Vec<_>>();
    // Stages, health and relation changes do not create contract drift.
    Ok(canonical_json_sha256(
        &json!({"title":card.title,"body":card.body,"comments":comments}),
    )?)
}
