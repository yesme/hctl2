use foundation::{bytes_sha256, canonical_json_sha256};
use serde_json::{Value, json};
use store::{
    ActorSource, Command, EffectIntent, ObjectKey, Record, RecordData, Reference, Scope, Store,
    TrustedActor, Version,
};

use crate::*;

pub fn owner(
    actor: &TrustedActor,
    scopes: impl IntoIterator<Item = Scope>,
) -> Result<TrustedActor> {
    if actor.0.source != ActorSource::DirectClient
        || !actor.0.permission_scope.contains(&Scope::Control)
    {
        return Err(reject(
            "PERMISSION_DENIED",
            "Task commands require the authenticated control owner",
            "request_authorization",
        ));
    }
    let mut actor = actor.0.clone();
    for scope in scopes {
        if !actor.permission_scope.contains(&scope) {
            actor.permission_scope.push(scope);
        }
    }
    Ok(TrustedActor(actor))
}

pub fn command_key(store: &Store, idem: &str) -> ObjectKey {
    key(
        Scope::Control,
        "task_command",
        &bytes_sha256(format!("{}:{idem}", store.control_id()).as_bytes()),
    )
}

pub fn replay(store: &Store, input: &Input) -> Result<Option<Plan>> {
    let Some(record) = store.get(&command_key(store, &input.key))? else {
        return Ok(None);
    };
    let plan: Plan = decode(&record)?;
    if canonical_json_sha256(&serde_json::to_value(input)?)?
        != canonical_json_sha256(&serde_json::to_value(&plan.input)?)?
    {
        return Err(reject(
            "IDEMPOTENCY_CONFLICT",
            "key has different Task input",
            "use_original_command",
        ));
    }
    Ok(Some(plan))
}

impl Plan {
    fn check(&mut self, record: &Record) {
        if !self.checks.iter().any(|c| c.key == record.key) {
            self.checks.push(Check {
                key: record.key.clone(),
                version: Some(record.version),
            });
        }
    }
    fn put<T: serde::Serialize>(&mut self, store: &Store, key: ObjectKey, data: &T) -> Result<()> {
        let old = store.get(&key)?;
        self.checks.push(Check {
            key: key.clone(),
            version: old.as_ref().map(|r| r.version),
        });
        self.records
            .push(value_record(key, old.map_or(1, |r| r.version + 1), data)?);
        Ok(())
    }
    pub(crate) fn put_task(
        &mut self,
        store: &Store,
        task: &Task,
        source_ref: &Reference,
    ) -> Result<()> {
        let scope = Scope::Project(task.project_id.clone());
        self.put(store, key(scope.clone(), "task_state", &task.id), task)?;
        let identity = key(scope, "task", &task.id);
        let old = store.get(&identity)?;
        self.checks.push(Check {
            key: identity.clone(),
            version: old.as_ref().map(|r| r.version),
        });
        let data = RecordData::Task {
            entity: task.entity.clone(),
            source: source_ref.clone(),
        };
        self.records.push(Record {
            key: identity,
            version: old.map_or(1, |r| r.version + 1),
            revision_digest: canonical_json_sha256(&serde_json::to_value(&data)?)?,
            data,
            sources: vec![],
            materials: vec![],
        });
        self.task_key = Some(key(
            Scope::Project(task.project_id.clone()),
            "task_state",
            &task.id,
        ));
        self.result = json!({"project_id":task.project_id,"task_id":task.id});
        Ok(())
    }
}

fn project(store: &Store, plan: &mut Plan, id: &str, expected: Option<i64>) -> Result<String> {
    let r = required(store, &key(Scope::Project(id.into()), "project", id))?;
    if expected.is_some_and(|v| v != r.version) {
        return Err(stale());
    }
    let RecordData::Project {
        repo_id, archived, ..
    } = &r.data
    else {
        return Err(stale());
    };
    if *archived {
        return Err(reject(
            "PROJECT_ARCHIVED",
            "Project is read-only",
            "restore_project",
        ));
    }
    repo::require_active(store, repo_id)?;
    plan.check(&r);
    Ok(repo_id.clone())
}

pub fn stale() -> StoreError {
    reject(
        "VERSION_CONFLICT",
        "preview input changed",
        "preview_current_version",
    )
}

fn connected(
    store: &Store,
    plan: &mut Plan,
    project_id: &str,
    repo_id: &str,
    id: &str,
) -> Result<(Record, Source)> {
    let pair = approved_source(store, plan, project_id, repo_id, id)?;
    if !pair.1.active {
        return Err(reject(
            "SOURCE_DISABLED",
            "Task home is disabled",
            "reconnect_source",
        ));
    }
    Ok(pair)
}

fn approved_source(
    store: &Store,
    plan: &mut Plan,
    project_id: &str,
    repo_id: &str,
    id: &str,
) -> Result<(Record, Source)> {
    let r = required(
        store,
        &key(
            Scope::Project(project_id.into()),
            "task_source_reference",
            id,
        ),
    )?;
    let approved: SourceReference = decode(&r)?;
    let (sr, source) = source(store, repo_id, id)?;
    if approved.approved_scope != source.board_scope_stable_id || approved.source.key != sr.key {
        return Err(reject(
            "SOURCE_SCOPE",
            "source reference does not authorize this board",
            "attach_source",
        ));
    }
    plan.check(&r);
    plan.check(&sr);
    Ok((sr, source))
}

pub fn latest(store: &Store, source: &Source) -> Result<(Record, Snapshot)> {
    let r = required(
        store,
        &key(
            Scope::Repo(source.repo_id.clone()),
            "task_snapshot",
            &source.id,
        ),
    )?;
    let snap: Snapshot = decode(&r)?;
    if !source.active
        || !snap.complete
        || snap.error.is_some()
        || now().saturating_sub(snap.observed_at) > 60
    {
        return Err(reject(
            "READBACK_REQUIRED",
            "source unavailable, incomplete or older than 60 seconds",
            "refresh_source",
        ));
    }
    Ok((r, snap))
}

pub(crate) fn card<'a>(snapshot: &'a Snapshot, entity_id: &str) -> Result<&'a Card> {
    snapshot
        .cards
        .iter()
        .find(|c| c.entity.immutable_external_entity_id == entity_id && !c.tombstone)
        .ok_or_else(|| {
            reject(
                "CARD_UNAVAILABLE",
                "original card not observed",
                "refresh_source",
            )
        })
}

fn check_contract(
    store: &Store,
    plan: &mut Plan,
    project_id: &str,
    task: Option<&Task>,
    adoption: &Adoption,
) -> Result<()> {
    let c = &adoption.contract;
    if c.scope.trim().is_empty()
        || c.expected_outcome.trim().is_empty()
        || c.acceptance.is_empty()
        || c.acceptance.iter().any(|a| a.text.trim().is_empty())
    {
        return Err(reject(
            "INVALID_CONTRACT",
            "scope, outcome and graded acceptance items required",
            "edit_proposal",
        ));
    }
    match &adoption.origin {
        ContractOrigin::Local {
            reference: origin,
            proposal_digest,
        } => {
            if origin.key.scope != Scope::Project(project_id.into())
                || !["project", "room", "memo", "artifact", "request"]
                    .contains(&origin.key.kind.as_str())
            {
                return Err(reject(
                    "PROJECT_MISMATCH",
                    "proposal source belongs to another Project or is not a supported source",
                    "choose_same_project",
                ));
            }
            let r = required(store, &origin.key)?;
            if origin.version != Version::State(r.version)
                && origin.version != Version::Revision(r.revision_digest.clone())
            {
                return Err(stale());
            }
            if *proposal_digest != canonical_json_sha256(&serde_json::to_value(c)?)? {
                return Err(reject(
                    "PROPOSAL_DIGEST",
                    "proposal digest does not match contract",
                    "edit_proposal",
                ));
            }
            plan.check(&r);
        }
        ContractOrigin::Backend {
            snapshot,
            state_version,
        } => {
            let task = task.ok_or_else(|| {
                reject(
                    "INVALID_CONTRACT",
                    "new card has no backend Snapshot yet",
                    "use_local_proposal",
                )
            })?;
            if task.snapshot.as_ref() != Some(snapshot) || task.state_version != *state_version {
                return Err(stale());
            }
            let (_, src) = source(store, &task.repo_id, &task.source_id)?;
            let (r, snap) = latest(store, &src)?;
            if reference(&r) != *snapshot {
                return Err(stale());
            }
            card(
                &snap,
                &task
                    .entity
                    .as_ref()
                    .ok_or_else(stale)?
                    .immutable_external_entity_id,
            )?;
            plan.check(&r);
        }
    }
    plan.adoption = Some(adoption.clone());
    Ok(())
}

pub(crate) fn new_task(
    store: &Store,
    project: &str,
    source: &Source,
    identity: &str,
    title: &str,
) -> Task {
    Task {
        id: bytes_sha256(format!("{}:{project}:{identity}", store.control_id()).as_bytes()),
        project_id: project.into(),
        source_id: source.id.clone(),
        repo_id: source.repo_id.clone(),
        entity: None,
        number: None,
        title: title.into(),
        lifecycle: "open".into(),
        lifecycle_version: 1,
        archived: false,
        revision: None,
        snapshot: None,
        state_version: 0,
        needs_attention: false,
        pending_contract: None,
        run_occupancy: None,
    }
}

fn add_effect(
    store: &Store,
    plan: &mut Plan,
    task: &Task,
    src: &Record,
    operation: &str,
    input: Value,
) -> Result<()> {
    let id = format!("task:{}", command_key(store, &plan.input.key).id);
    let input = json!({"project_id":task.project_id,"task_id":task.id,"source_id":task.source_id,"repo_id":task.repo_id,"write":input});
    let resource = if let Some(entity) = &task.entity {
        canonical_json_sha256(&serde_json::to_value(entity)?)?
    } else {
        id.clone()
    };
    plan.effects.push(EffectIntent {
        intent_id: id.clone(),
        owner: Reference {
            key: key(Scope::Project(task.project_id.clone()), "task", &task.id),
            version: Version::State(
                store
                    .get(&key(
                        Scope::Project(task.project_id.clone()),
                        "task",
                        &task.id,
                    ))?
                    .map_or(1, |r| r.version),
            ),
        },
        binding: reference(src),
        operation: operation.into(),
        target: resource.clone(),
        conflict_scope: format!("issue:{resource}"),
        permission_scope: Scope::Project(task.project_id.clone()),
        input_digest: Command::digest_input(operation, &input)?,
        input,
        idempotency_key: id.clone(),
    });
    plan.result["effect_id"] = json!(id);
    Ok(())
}

pub fn prepare(store: &Store, input: Input) -> Result<Plan> {
    if input.key.trim().is_empty() {
        return Err(reject(
            "INVALID_INPUT",
            "command key required",
            "correct_input",
        ));
    }
    if let Some(old) = replay(store, &input)? {
        return Ok(old);
    }
    let mut p = Plan {
        input: input.clone(),
        checks: vec![],
        records: vec![],
        effects: vec![],
        cancel_effects: vec![],
        result: json!({}),
        adoption: None,
        task_key: None,
    };
    match &input.action {
        Action::Connect {
            repo_id,
            candidate_id,
            consent,
            make_default,
        } => {
            if !consent {
                return Err(reject(
                    "CONSENT_REQUIRED",
                    "connecting source needs explicit consent",
                    "confirm_source",
                ));
            }
            let reg = repo::require_active(store, repo_id)?;
            let candidate = reg
                .sources
                .iter()
                .find(|c| c.id == *candidate_id && c.available && c.can_claim)
                .ok_or_else(|| {
                    reject(
                        "SOURCE_UNAVAILABLE",
                        "candidate is not available",
                        "inspect_repo_candidates",
                    )
                })?
                .clone();
            if *make_default && !candidate.can_default() {
                return Err(reject(
                    "SOURCE_READ_ONLY",
                    "default needs create and writeback",
                    "choose_capable_source",
                ));
            }
            let platform = reg.observed.ok_or_else(|| {
                reject(
                    "SOURCE_UNAVAILABLE",
                    "platform not confirmed",
                    "finish_registration",
                )
            })?;
            p.check(&required(store, &repo::key(repo_id))?);
            let id = bytes_sha256(
                format!(
                    "{repo_id}:{}:{}:{}:{}",
                    candidate.provider, platform.instance, platform.account_id, platform.stable_id
                )
                .as_bytes(),
            );
            let skey = key(Scope::Repo(repo_id.clone()), "task_source", &id);
            if let Some(existing) = store.get(&skey)? {
                let src: Source = decode(&existing)?;
                if !src.active {
                    return Err(reject(
                        "SOURCE_DISABLED",
                        "existing source disabled",
                        "reconnect_source",
                    ));
                }
                p.check(&existing);
            } else {
                let src = Source {
                    id: id.clone(),
                    repo_id: repo_id.clone(),
                    port_kind: "task_source".into(),
                    capabilities: Capabilities {
                        create: candidate.create,
                        field_writeback: candidate.field_writeback,
                        conditional_write: candidate.provider == "gitea_issues",
                        conditional_fields: if candidate.provider == "gitea_issues" {
                            vec!["body".into()]
                        } else {
                            vec![]
                        },
                        placement: false,
                        delete: candidate.field_writeback,
                    },
                    board_scope_stable_id: platform.stable_id.clone(),
                    platform,
                    candidate,
                    binding_revision: 1,
                    active: true,
                };
                p.put(store, skey, &src)?;
            }
            let default_key = key(Scope::Repo(repo_id.clone()), "task_default_source", repo_id);
            if *make_default
                || (reg.prepared.request.default_source.as_ref() == Some(candidate_id)
                    && store.get(&default_key)?.is_none())
            {
                if !reg
                    .sources
                    .iter()
                    .any(|c| c.id == *candidate_id && c.can_default())
                {
                    return Err(reject(
                        "SOURCE_READ_ONLY",
                        "confirmed default lost write capability",
                        "choose_capable_source",
                    ));
                }
                p.put(
                    store,
                    default_key,
                    &json!({"source_id":id,"candidate_id":candidate_id}),
                )?;
            }
            p.result = json!({"source_id":id,"repo_id":repo_id});
        }
        Action::SetActive {
            repo_id,
            source_id,
            active,
            version,
        } => {
            let (r, mut src) = source(store, repo_id, source_id)?;
            if r.version != *version {
                return Err(stale());
            }
            src.active = *active;
            p.put(store, r.key, &src)?;
            for (r, mut t) in tasks(store)?
                .into_iter()
                .filter(|(_, t)| t.source_id == *source_id)
            {
                t.needs_attention = true;
                p.put(store, r.key, &t)?;
            }
            p.result = json!({"source_id":source_id,"active":active});
        }
        Action::Attach {
            project_id,
            project_version,
            source_id,
            approved_scope,
            group,
            consent,
        } => {
            if !consent {
                return Err(reject(
                    "CONSENT_REQUIRED",
                    "Project source selection needs consent",
                    "confirm_source",
                ));
            }
            let rid = project(store, &mut p, project_id, Some(*project_version))?;
            let (sr, src) = source(store, &rid, source_id)?;
            if !src.active || *approved_scope != src.board_scope_stable_id {
                return Err(reject(
                    "SOURCE_SCOPE",
                    "unavailable or unauthorized board",
                    "choose_source_scope",
                ));
            }
            p.check(&sr);
            if let Some(group) = group {
                let (r, snap) = latest(store, &src)?;
                if !snap.stable_groups.contains(group) {
                    return Err(reject(
                        "GROUP_UNSTABLE",
                        "group anchor cannot be stably read",
                        "choose_native_group",
                    ));
                }
                p.check(&r);
            }
            let k = key(
                Scope::Project(project_id.clone()),
                "task_source_reference",
                source_id,
            );
            if let Some(old) = store.get(&k)? {
                let old: SourceReference = decode(&old)?;
                if old.group != *group {
                    return Err(reject(
                        "GROUP_IMMUTABLE",
                        "reconnecting does not replace approved group",
                        "use_original_group",
                    ));
                }
            }
            p.put(
                store,
                k,
                &SourceReference {
                    project_id: project_id.clone(),
                    source: reference(&sr),
                    approved_scope: approved_scope.clone(),
                    group: group.clone(),
                },
            )?;
            p.result = json!({"project_id":project_id,"source_id":source_id});
        }
        Action::Claim {
            project_id,
            project_version,
            source_id,
            entity_id,
        } => {
            let rid = project(store, &mut p, project_id, Some(*project_version))?;
            let (sr, src) = connected(store, &mut p, project_id, &rid, source_id)?;
            let (r, snap) = latest(store, &src)?;
            let c = card(&snap, entity_id)?;
            p.check(&r);
            if let Some((_, t)) = tasks(store)?
                .into_iter()
                .find(|(_, t)| t.project_id == *project_id && t.entity.as_ref() == Some(&c.entity))
            {
                p.result = json!({"project_id":project_id,"task_id":t.id});
            } else {
                if pending_creation(store, project_id, &src.id, c)?.is_some() {
                    return Err(reject(
                        "CREATION_PENDING",
                        "card belongs to an unresolved Task creation",
                        "resume_original_creation",
                    ));
                }
                let mut t = new_task(
                    store,
                    project_id,
                    &src,
                    &canonical_json_sha256(&serde_json::to_value(&c.entity)?)?,
                    &c.title,
                );
                t.entity = Some(c.entity.clone());
                t.number = Some(c.number);
                t.snapshot = Some(reference(&r));
                t.state_version = 1;
                p.put_task(store, &t, &reference(&sr))?;
            }
        }
        Action::Create {
            project_id,
            project_version,
            source_id,
            title,
            body,
            adoption,
        } => {
            let rid = project(store, &mut p, project_id, Some(*project_version))?;
            let (sr, src) = connected(store, &mut p, project_id, &rid, source_id)?;
            if !src.capabilities.create || title.trim().is_empty() {
                return Err(reject(
                    "CREATE_UNAVAILABLE",
                    "explicit writable source and title required",
                    "choose_capable_source",
                ));
            }
            if let Some(a) = adoption {
                check_contract(store, &mut p, project_id, None, a)?;
            }
            let mut t = new_task(
                store,
                project_id,
                &src,
                &format!("create:{}", input.key),
                title,
            );
            t.needs_attention = true;
            p.put_task(store, &t, &reference(&sr))?;
            let marker = format!(
                "<!-- hctl2:{}:{}:{} -->",
                store.control_id(),
                t.id,
                canonical_json_sha256(&json!({"title":title,"body":body}))?
            );
            add_effect(
                store,
                &mut p,
                &t,
                &sr,
                "task.create",
                json!({"title":title,"body":format!("{body}\n\n{marker}"),"marker":marker}),
            )?;
        }
        Action::Adopt {
            project_id,
            project_version,
            task_id,
            version,
            adoption,
        } => {
            project(store, &mut p, project_id, Some(*project_version))?;
            let (r, t) = task(store, project_id, task_id)?;
            if *version != r.version {
                return Err(stale());
            }
            check_contract(store, &mut p, project_id, Some(&t), adoption)?;
            let (_, src) = source(store, &t.repo_id, &t.source_id)?;
            let (sr, _) = approved_source(store, &mut p, project_id, &src.repo_id, &src.id)?;
            p.put_task(store, &t, &reference(&sr))?;
        }
        Action::Update {
            project_id,
            task_id,
            version,
            state_version,
            ..
        }
        | Action::Move {
            project_id,
            task_id,
            version,
            state_version,
            ..
        } => {
            let rid = project(store, &mut p, project_id, None)?;
            let (r, t) = task(store, project_id, task_id)?;
            if r.version != *version || t.state_version != *state_version {
                return Err(stale());
            }
            let (sr, src) = connected(store, &mut p, project_id, &rid, &t.source_id)?;
            if !src.capabilities.field_writeback {
                return Err(reject(
                    "SOURCE_READ_ONLY",
                    "source cannot write fields",
                    "choose_writable_binding",
                ));
            }
            let (snap_r, snap) = latest(store, &src)?;
            let c = card(
                &snap,
                &t.entity
                    .as_ref()
                    .ok_or_else(stale)?
                    .immutable_external_entity_id,
            )?;
            p.check(&r);
            p.check(&snap_r);
            let write = match &input.action {
                Action::Update { fields, .. } => {
                    if fields.title.as_ref().is_some_and(|s| s.trim().is_empty())
                        || (fields.title.is_none()
                            && fields.body.is_none()
                            && fields.comment.is_none())
                    {
                        return Err(reject(
                            "INVALID_INPUT",
                            "nonempty update required",
                            "correct_input",
                        ));
                    }
                    if fields.comment.is_some() && (fields.title.is_some() || fields.body.is_some())
                    {
                        return Err(reject(
                            "INVALID_INPUT",
                            "append comment and edit fields are separate effects",
                            "split_commands",
                        ));
                    }
                    json!({"title":fields.title,"body":fields.body,"comment":fields.comment.as_ref().map(|s|format!("{s}\n\n<!-- hctl2:{}:{}:{} -->",store.control_id(),t.id,command_key(store,&input.key).id))})
                }
                Action::Move {
                    stage,
                    rank,
                    relative_source_id,
                    ..
                } => {
                    if relative_source_id
                        .as_ref()
                        .is_some_and(|s| s != &t.source_id)
                    {
                        return Err(reject(
                            "CROSS_SOURCE_MOVE",
                            "relative move across sources is unsupported",
                            "move_within_source",
                        ));
                    }
                    if rank.is_some() || !["open", "closed"].contains(&stage.as_str()) {
                        return Err(reject(
                            "PLACEMENT_UNSUPPORTED",
                            "issues binding exposes state only, not native board ordering",
                            "use_provider_client",
                        ));
                    }
                    json!({"state":stage})
                }
                _ => unreachable!(),
            };
            p.result = json!({"project_id":project_id,"task_id":task_id});
            add_effect(
                store,
                &mut p,
                &t,
                &sr,
                "task.update",
                json!({"fields":write,"card":c}),
            )?;
        }
        Action::Cancel {
            project_id,
            task_id,
            version,
        } => {
            project(store, &mut p, project_id, None)?;
            let (r, mut t) = task(store, project_id, task_id)?;
            if *version != r.version {
                return Err(stale());
            }
            if t.run_occupancy.is_some() || !active_runs(store, &t)?.is_empty() {
                return Err(reject(
                    "RUN_ACTIVE",
                    "cancel does not stop Run",
                    "finish_run_first",
                ));
            }
            if t.lifecycle != "open" {
                return Err(reject("TASK_TERMINAL", "Task is not open", "inspect_task"));
            }
            for id in store.pending_effects()? {
                let (effect, state) = store.effect(&id)?;
                if effect.owner.key == key(Scope::Project(project_id.clone()), "task", task_id) {
                    if state == store::EffectState::Pending {
                        p.cancel_effects.push(id);
                    } else {
                        t.needs_attention = true;
                    }
                }
            }
            t.lifecycle = "cancelled".into();
            t.lifecycle_version += 1;
            t.archived = true;
            let (sr, _) = source(store, &t.repo_id, &t.source_id)?;
            p.put_task(store, &t, &reference(&sr))?;
        }
        Action::DeleteCard {
            project_id,
            task_id,
            version,
            confirm_irreversible,
            active_run_choices,
        } => {
            let rid = project(store, &mut p, project_id, None)?;
            let (r, t) = task(store, project_id, task_id)?;
            if r.version != *version {
                return Err(stale());
            }
            let (sr, src) = connected(store, &mut p, project_id, &rid, &t.source_id)?;
            if !src.capabilities.delete {
                return Err(reject(
                    "DELETE_UNSUPPORTED",
                    "binding cannot delete",
                    "use_provider_client",
                ));
            }
            let (snap_r, snap) = latest(store, &src)?;
            let c = card(
                &snap,
                &t.entity
                    .as_ref()
                    .ok_or_else(stale)?
                    .immutable_external_entity_id,
            )?;
            p.check(&snap_r);
            let mut affected = vec![];
            for (r, other) in tasks(store)?
                .into_iter()
                .filter(|(_, o)| o.entity == t.entity)
            {
                p.check(&r);
                let runs = active_runs(store, &other)?;
                for run in &runs {
                    p.check(run);
                }
                if *confirm_irreversible
                    && runs
                        .iter()
                        .any(|r| !active_run_choices.contains(&reference(r)))
                {
                    return Err(reject(
                        "RUN_CHOICE_REQUIRED",
                        "explicitly acknowledge each active Run; deletion will not stop it",
                        "review_affected_tasks",
                    ));
                }
                affected.push(json!({"project_id":other.project_id,"task_id":other.id,"lifecycle":other.lifecycle,"runs":runs,"occupancy":other.run_occupancy}));
                if *confirm_irreversible
                    && other.run_occupancy.is_some()
                    && active_runs(store, &other)?.is_empty()
                {
                    return Err(reject(
                        "RUN_CHOICE_REQUIRED",
                        "occupied Task lacks readable Run",
                        "inspect_run",
                    ));
                }
            }
            p.result = json!({"project_id":project_id,"task_id":task_id,"affected":affected,"irreversible":true,"other_controls":"not_enumerated","does_not_stop_runs":true});
            if *confirm_irreversible {
                add_effect(
                    store,
                    &mut p,
                    &t,
                    &sr,
                    "task.delete",
                    json!({"card":c,"affected":affected}),
                )?;
            }
        }
        Action::Refresh { repo_id, source_id } => {
            source(store, repo_id, source_id)?;
            p.result = json!({"repo_id":repo_id,"source_id":source_id});
        }
        Action::Resume { effect_id } => {
            let (e, state) = store.effect(effect_id)?;
            if !e.operation.starts_with("task.") {
                return Err(stale());
            }
            p.result = json!({"effect_id":effect_id,"state":state});
        }
    }
    Ok(p)
}

/// An unresolved creation reserves its own card even if periodic reads see it first.
pub(crate) fn pending_creation(
    store: &Store,
    project: &str,
    source: &str,
    card: &Card,
) -> Result<Option<String>> {
    for id in store.pending_effects()? {
        let (effect, _) = store.effect(&id)?;
        if effect.operation == "task.create"
            && effect.input["project_id"].as_str() == Some(project)
            && effect.binding.key.id == source
            && effect.input["write"]["marker"]
                .as_str()
                .is_some_and(|marker| card.body.contains(marker))
        {
            return Ok(effect.input["task_id"].as_str().map(str::to_owned));
        }
    }
    Ok(None)
}

pub(crate) fn deletion_consequences(store: &Store, task: &Task) -> Result<Value> {
    let mut affected = vec![];
    for (_, other) in tasks(store)?
        .into_iter()
        .filter(|(_, other)| other.entity == task.entity)
    {
        let runs = active_runs(store, &other)?;
        affected.push(json!({"project_id":other.project_id,"task_id":other.id,"lifecycle":other.lifecycle,"runs":runs,"occupancy":other.run_occupancy}));
    }
    Ok(Value::Array(affected))
}

/// P2.3 writes Run records. Until then unknown Run shapes are conservatively nonterminal.
pub fn active_runs(store: &Store, task: &Task) -> Result<Vec<Record>> {
    Ok(store
        .list("run")?
        .into_iter()
        .filter(|r| {
            if r.key.scope != Scope::Project(task.project_id.clone()) {
                return false;
            }
            let RecordData::Value { value } = &r.data else {
                return true;
            };
            let target = value.get("task_id").and_then(Value::as_str);
            target.is_none_or(|s| s == task.id)
                && !["completed", "cancelled", "failed", "replaced"]
                    .contains(&value["lifecycle"].as_str().unwrap_or("unknown"))
        })
        .collect())
}
