use super::{freeze::*, *};
use chat::{decode, invalid, key, reference, required, value_record};
use foundation::{bytes_sha256, canonical_json, canonical_json_sha256};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use store::{
    ActorSource, Command, EffectIntent, EffectState, Expected, ObjectKey, Record, Reference,
    Result, Scope, Store, TrustedActor, Version,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub input: Input,
    pub checks: Vec<chat::Check>,
    pub records: Vec<Record>,
    pub effects: Vec<EffectIntent>,
    pub cancel_effects: Vec<String>,
    pub result: Value,
}
fn command_key(input: &Input) -> ObjectKey {
    key(
        scope(input.action.project()),
        "run_command",
        &bytes_sha256(input.key.as_bytes()),
    )
}
pub fn replay(store: &Store, actor: &TrustedActor, input: &Input) -> Result<Option<Plan>> {
    let Some(record) = store.get(&command_key(input))? else {
        return Ok(None);
    };
    let data: Value = decode(&record)?;
    let plan: Plan = serde_json::from_value(data["plan"].clone())?;
    if data["actor"] != serde_json::to_value(&actor.0)?
        || serde_json::to_value(&plan.input)? != serde_json::to_value(input)?
    {
        return Err(error(
            "IDEMPOTENCY_CONFLICT",
            "Run key binds original actor and input",
        ));
    }
    Ok(Some(plan))
}
pub fn load(store: &Store, project: &str, id: &str) -> Result<(Record, Run)> {
    let record = required(store, &run_key(project, id))?;
    let run: Run = decode(&record)?;
    if run.project_id != project || run.id != id {
        return Err(invalid("Run identity disagrees with record"));
    }
    Ok((record, run))
}
pub fn manifest(
    store: &Store,
    actor: &TrustedActor,
    project: &str,
    id: &str,
) -> Result<(Record, Manifest)> {
    let record = required(store, &manifest_key(project, id))?;
    let m: Manifest = decode(&record)?;
    let [material] = record.materials.as_slice() else {
        return Err(invalid("Run Manifest needs its admitted exact body"));
    };
    let bytes = store.read_material(actor, material)?;
    if canonical_json_sha256(&serde_json::from_slice(&bytes)?)? != record.revision_digest
        || bytes != canonical_json(&serde_json::to_value(&m)?)?
    {
        return Err(error(
            "DIGEST_MISMATCH",
            "Run Manifest differs from admitted material",
        ));
    }
    Ok((record, m))
}
fn occupancy(record: &Record) -> Result<Option<Value>> {
    Ok(decode::<task::Task>(record)?.run_occupancy)
}
fn occupied(id: &str, manifest: &Reference, state: &str) -> Value {
    json!({"run_id":id,"manifest":manifest,"state":state})
}
pub(super) fn effect(
    m: &Manifest,
    run: &Record,
    operation: &str,
    epoch: u64,
) -> Result<EffectIntent> {
    let operation = format!("run.engine.{operation}");
    let id = format!("{}:{}:{}:{epoch}", m.project.key.id, m.run_id, operation);
    let input = json!({"project_id":m.project.key.id,"run_id":m.run_id,"manifest":reference(&value_record(manifest_key(&m.project.key.id,&m.run_id),1,m)?),"epoch":epoch});
    Ok(EffectIntent {
        intent_id: id.clone(),
        owner: reference(run),
        binding: m.engine_binding.clone(),
        operation: operation.clone(),
        target: m.run_id.clone(),
        conflict_scope: format!("{operation}:{}:{}", m.project.key.id, m.run_id),
        permission_scope: run.key.scope.clone(),
        input_digest: Command::digest_input(&operation, &input)?,
        input,
        idempotency_key: id,
    })
}
fn add_start(
    store: &Store,
    actor: &TrustedActor,
    spec: &Start,
    plan: &mut Plan,
    replacing: Option<(&str, Option<&str>)>,
) -> Result<()> {
    let frozen = freeze(store, actor, spec, &mut plan.checks)?;
    if watched(
        store,
        &run_key(&spec.project_id, &spec.run_id),
        &mut plan.checks,
    )?
    .is_some()
        || watched(
            store,
            &manifest_key(&spec.project_id, &spec.run_id),
            &mut plan.checks,
        )?
        .is_some()
    {
        return Err(error("RUN_EXISTS", "Run identity is never reused"));
    }
    if let Some((_, old_task)) = replacing
        && old_task != frozen.task_id.as_deref()
    {
        return Err(invalid("replacement must retain the bound Task identity"));
    }
    let m = &frozen.manifest;
    let manifest = value_record(manifest_key(&spec.project_id, &spec.run_id), 1, m)?;
    let run = value_record(
        run_key(&spec.project_id, &spec.run_id),
        1,
        &Run {
            id: spec.run_id.clone(),
            project_id: spec.project_id.clone(),
            task_id: frozen.task_id,
            lifecycle: State::Starting,
            manifest: reference(&manifest),
            entered_at_ms: 0,
            transition_timeout_ms: spec.policy.transition_timeout_ms,
            operation_epoch: 1,
            authorization_valid: true,
            attention: None,
            terminal_requested: None,
            placement: None,
        },
    )?;
    if let Some(record) = frozen.task_state {
        if replacing.is_none() && occupancy(&record)?.is_some() {
            return Err(error(
                "RUN_ACTIVE",
                "Task has an active or completion_pending Run",
            ));
        }
        let next = task::plan_run_occupancy(
            &record,
            replacing.map(|(id, _)| id),
            Some(occupied(&spec.run_id, &reference(&manifest), "active")),
        )?;
        plan.records.push(next);
    }
    for selection in &m.selected {
        if watched(store, &selection.selection.key, &mut plan.checks)?.is_some() {
            return Err(invalid("Run selection already exists"));
        }
        let mut selected = value_record(selection.selection.key.clone(), 1, selection)?;
        selected.sources = vec![reference(&manifest)];
        plan.records.push(selected);
    }
    plan.effects.push(effect(m, &run, "start", 1)?);
    plan.records.extend([manifest, run]);
    plan.result = json!({"project_id":spec.project_id,"run_id":spec.run_id,"version":1,"lifecycle":"starting","manifest":m,"integration":m.integration,"blockers_requiring_confirmation":frozen.blockers,"warnings":frozen.warnings});
    Ok(())
}
/// All activity authorities and unsent effects are revoked together. A missing
/// activity registration or unproved write containment is NOT an empty success.
pub(super) fn revoke(store: &Store, project: &str, id: &str, plan: &mut Plan) -> Result<bool> {
    let (_, run) = load(store, project, id)?;
    let mut safe = true;
    let mut owners = vec![run_key(project, id)];
    let mut activities = vec![];
    for record in store.list("run_activity")? {
        if record.key.scope != scope(project) {
            continue;
        }
        let mut a: Activity = decode(&record)?;
        if a.run_id != id {
            continue;
        }
        track(&mut plan.checks, &record);
        if a.manifest != run.manifest {
            return Err(invalid("activity belongs to another Run Manifest"));
        }
        owners.push(a.owner.key.clone());
        if a.can_write {
            let proven = match &a.isolation_proof {
                None => false,
                Some(proof) => {
                    if proof.key.scope != scope(project) || proof.key.kind != "run_isolation" {
                        return Err(invalid("control-verified Run isolation reference required"));
                    }
                    let record = exact(store, proof, &mut plan.checks)?;
                    let p: IsolationProof = decode(&record)?;
                    p.manifest == a.manifest && p.owner == a.owner && (p.confined || p.silent)
                }
            };
            safe &= proven;
        }
        if a.authorization_valid {
            a.authorization_valid = false;
            activities.push(value_record(record.key, record.version + 1, &a)?);
        }
    }
    for attempt in store.list("run_attempt")? {
        if attempt.key.scope != scope(project) {
            continue;
        }
        let value: Value = decode(&attempt)?;
        if value["run_id"] == id && !owners.contains(&attempt.key) {
            safe = false;
            track(&mut plan.checks, &attempt);
        }
    }
    for id in store.pending_effects()? {
        let (intent, state) = store.effect(&id)?;
        if owners.contains(&intent.owner.key) {
            if state == EffectState::Pending {
                plan.cancel_effects.push(id);
            } else if state == EffectState::Unknown && !intent.operation.starts_with("run.engine.")
            {
                safe = false;
            }
        }
    }
    plan.records.extend(activities);
    Ok(safe)
}
pub fn prepare(store: &Store, actor: &TrustedActor, input: Input) -> Result<Plan> {
    let scoped = chat::owner(actor, input.action.project())?;
    if input.key.trim().is_empty() {
        return Err(invalid("Run command key required"));
    }
    if let Some(plan) = replay(store, actor, &input)? {
        return Ok(plan);
    };
    let mut plan = Plan {
        input: input.clone(),
        checks: vec![],
        records: vec![],
        effects: vec![],
        cancel_effects: vec![],
        result: Value::Null,
    };
    match &input.action {
        Action::Start { spec } => add_start(store, &scoped, spec, &mut plan, None)?,
        action => {
            let (project, id, version) = match action {
                Action::Pause {
                    project_id,
                    run_id,
                    version,
                }
                | Action::Resume {
                    project_id,
                    run_id,
                    version,
                }
                | Action::Cancel {
                    project_id,
                    run_id,
                    version,
                }
                | Action::Replace {
                    project_id,
                    run_id,
                    version,
                    ..
                }
                | Action::Place {
                    project_id,
                    run_id,
                    version,
                    ..
                } => (project_id, run_id, *version),
                Action::Start { .. } => unreachable!(),
            };
            let (record, mut run) = load(store, project, id)?;
            if record.version != version {
                return Err(chat::stale());
            }
            track(&mut plan.checks, &record);
            let (manifest_record, m) = manifest(store, &scoped, project, id)?;
            track(&mut plan.checks, &manifest_record);
            if run.manifest != reference(&manifest_record) {
                return Err(invalid("Run Manifest reference mismatch"));
            }
            let old_state = run.lifecycle;
            let mut next = None;
            let mut op = None;
            match action {
                Action::Pause { .. } => {
                    if old_state != State::Running || !run.authorization_valid {
                        return Err(error(
                            "INVALID_TRANSITION",
                            "only an authorized running Run can pause",
                        ));
                    }
                    next = Some(State::Pausing);
                    op = Some("pause");
                }
                Action::Resume { .. } => {
                    if old_state != State::Paused || !run.authorization_valid {
                        return Err(error(
                            "INVALID_TRANSITION",
                            "only an authorized paused Run can resume",
                        ));
                    }
                    next = Some(State::Running);
                    op = Some("resume");
                }
                Action::Cancel { .. } => {
                    if old_state.terminal() || old_state == State::Cancelling {
                        return Err(error(
                            "INVALID_TRANSITION",
                            "Run is terminal or already cancelling",
                        ));
                    }
                    let safe = revoke(store, project, id, &mut plan)?;
                    run.authorization_valid = false;
                    run.terminal_requested = Some(State::Cancelled);
                    if old_state == State::Starting {
                        if safe {
                            next = Some(State::Cancelled)
                        } else {
                            run.attention = Some("cancellation waits for verified isolation".into())
                        }
                    } else {
                        next = Some(State::Cancelling);
                    }
                    op = Some("stop");
                }
                Action::Replace { spec, .. } => {
                    if spec.project_id != *project || spec.run_id == *id {
                        return Err(invalid("replacement needs a new Run in the same Project"));
                    }
                    if !revoke(store, project, id, &mut plan)? {
                        return Err(error(
                            "RUN_ISOLATION_UNPROVEN",
                            "old writes are not proved quiet or confined; cancel and inspect isolation",
                        ));
                    }
                    next = Some(State::Replaced);
                    run.authorization_valid = false;
                    run.terminal_requested = Some(State::Replaced);
                    op = Some("stop");
                    add_start(
                        store,
                        &scoped,
                        spec,
                        &mut plan,
                        Some((id, run.task_id.as_deref())),
                    )?;
                }
                Action::Place { destination, .. } => {
                    if run.lifecycle.terminal()
                        || !run.authorization_valid
                        || !m.placement.allowed_destinations.contains(destination)
                    {
                        return Err(error(
                            "PLACEMENT_DENIED",
                            "destination is outside the Manifest's mutable delivery boundary",
                        ));
                    }
                    let audit = value_record(
                        key(
                            scope(project),
                            "run_placement",
                            &format!("{id}:{}", version + 1),
                        ),
                        1,
                        &json!({"manifest":run.manifest,"before":run.placement,"after":destination,"rule":m.placement.allowed_destinations,"actor":actor.0,"run_version":version+1}),
                    )?;
                    plan.records.push(audit);
                    run.placement = Some(destination.clone());
                }
                Action::Start { .. } => unreachable!(),
            }
            if let Some(next) = next {
                if !old_state.allows(next) {
                    return Err(error(
                        "INVALID_TRANSITION",
                        format!("{old_state:?} -> {next:?} is not allowed"),
                    ));
                }
                run.lifecycle = next;
                run.entered_at_ms = 0;
                run.attention = None;
            }
            if op.is_some() {
                run.operation_epoch = run
                    .operation_epoch
                    .checked_add(1)
                    .ok_or_else(|| invalid("Run epoch exhausted"))?;
            }
            if run.lifecycle == State::Cancelled {
                release_slot(store, project, &run, &mut plan, None)?;
            }
            let next_record = value_record(
                record.key,
                version
                    .checked_add(1)
                    .ok_or_else(|| invalid("Run version exhausted"))?,
                &run,
            )?;
            if let Some(op) = op {
                plan.effects
                    .push(effect(&m, &next_record, op, run.operation_epoch)?);
            }
            plan.records.insert(0, next_record);
            if !matches!(action, Action::Replace { .. }) {
                plan.result = json!({"project_id":project,"run_id":id,"version":version+1,"lifecycle":run.lifecycle,"authorization_valid":run.authorization_valid,"attention":run.attention,"manifest":run.manifest});
            }
        }
    }
    Ok(plan)
}
pub(super) fn release_slot(
    store: &Store,
    project: &str,
    run: &Run,
    plan: &mut Plan,
    state: Option<&str>,
) -> Result<()> {
    if let Some(id) = &run.task_id {
        let (record, _) = task::task(store, project, id)?;
        track(&mut plan.checks, &record);
        if occupancy(&record)?.as_ref().is_none_or(|v| {
            v["manifest"] != serde_json::to_value(&run.manifest).unwrap_or(Value::Null)
        }) {
            return Err(error(
                "RUN_OCCUPANCY_MISMATCH",
                "Task occupancy belongs to another Manifest",
            ));
        }
        plan.records.push(task::plan_run_occupancy(
            &record,
            Some(&run.id),
            state.map(|state| occupied(&run.id, &run.manifest, state)),
        )?);
    }
    Ok(())
}
/// Admission rechecks immutable inputs and all versions. A transaction failure
/// after revocation rolls back revocation, new Run, occupancy and every outbox.
pub fn admit(store: &mut Store, actor: &TrustedActor, plan: Plan, now_ms: u64) -> Result<Value> {
    let scoped = chat::owner(actor, plan.input.action.project())?;
    if let Some(old) = replay(store, actor, &plan.input)? {
        return Ok(old.result);
    };
    if serde_json::to_value(prepare(store, actor, plan.input.clone())?)?
        != serde_json::to_value(&plan)?
    {
        return Err(chat::stale());
    }
    if plan
        .input
        .action
        .start()
        .is_some_and(|s| s.policy.deadline_ms <= now_ms)
    {
        return Err(error(
            "DEADLINE_EXPIRED",
            "Run deadline passed while confirming",
        ));
    }
    let mut records = plan.records.clone();
    let mut saved = vec![];
    for record in &mut records {
        if record.key.kind == "run_manifest" {
            let bytes = canonical_json(&serde_json::to_value(decode::<Manifest>(record)?)?)?;
            let material = store.save_material(
                store.generation(),
                &scoped,
                &record.key.scope,
                &plan.input.key,
                "run-manifest",
                &bytes,
            )?;
            record.materials.push(material.clone());
            saved.push(material);
        }
        if record.key.kind == "run" {
            let mut run: Run = decode(record)?;
            if run.entered_at_ms == 0 {
                run.entered_at_ms = now_ms;
                *record = value_record(record.key.clone(), record.version, &run)?;
            }
        }
    }
    let input = serde_json::to_value(&plan.input)?;
    let operation = format!("run.{}", plan.input.action.kind());
    let command = Command {
        command_id: format!("run:{}", plan.input.key),
        idempotency_key: plan.input.key.clone(),
        actor: scoped.0.clone(),
        target: command_key(&plan.input),
        expected: Expected::Absent,
        binding: Reference {
            key: key(Scope::Control, "module", "run"),
            version: Version::State(1),
        },
        input_digest: Command::digest_input(&operation, &input)?,
        operation,
        input,
    };
    store.submit(store.generation(), &scoped, &command, None, |tx| {
        for check in &plan.checks {
            if tx.get(&check.key)?.as_ref().map(|r| r.version) != check.version {
                return Err(chat::stale());
            }
        }
        for material in &saved {
            tx.admit_material(material)?;
        }
        for record in &records {
            tx.put(record)?;
        }
        for id in &plan.cancel_effects {
            tx.cancel_pending_effect(id)?;
        }
        for effect in &plan.effects {
            tx.enqueue_effect(effect)?;
        }
        tx.put(&value_record(
            command.target.clone(),
            1,
            &json!({"actor":actor.0,"plan":plan}),
        )?)?;
        Ok(plan.result.clone())
    })
}
pub fn show(
    store: &Store,
    actor: &TrustedActor,
    project: &str,
    id: &str,
    now_ms: u64,
) -> Result<Value> {
    let scoped = chat::owner(actor, project)?;
    let (record, run) = load(store, project, id)?;
    let (_, m) = manifest(store, &scoped, project, id)?;
    Ok(
        json!({"reference":reference(&record),"run":run,"manifest":m,"needs_attention":run.attention.is_some()||run.timed_out(now_ms),"transition_timed_out":run.timed_out(now_ms)}),
    )
}
pub fn list(store: &Store, actor: &TrustedActor, project: &str, now_ms: u64) -> Result<Value> {
    chat::owner(actor, project)?;
    let mut runs = vec![];
    for record in store.list("run")? {
        if record.key.scope == scope(project) {
            let run: Run = decode(&record)?;
            runs.push(json!({"reference":reference(&record),"run":run,"needs_attention":run.attention.is_some()||run.timed_out(now_ms)}));
        }
    }
    Ok(json!({"runs":runs}))
}
pub(super) fn reducer(actor: &TrustedActor, run: &Run) -> Result<TrustedActor> {
    if actor.0.source != ActorSource::InternalReducer
        || actor.0.authority.as_ref() != Some(&run.manifest)
        || !actor.0.permission_scope.contains(&scope(&run.project_id))
    {
        return Err(error(
            "PERMISSION_DENIED",
            "original Run Manifest reducer authority required",
        ));
    }
    Ok(TrustedActor(actor.0.clone()))
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionProof {
    pub manifest: Reference,
    pub engine_binding: Reference,
    pub all_required_satisfied: Option<bool>,
    pub all_attempts_terminal_or_revoked: Option<bool>,
    pub no_unresolved_output_effects: Option<bool>,
    pub references_match: Option<bool>,
}
/// Package 5b writes the control-verified completion proof; there is no public
/// command for a model or engine to assert these predicates. Failure revokes
/// authority first and retains occupancy if containment is unknown.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finish {
    pub key: String,
    pub project_id: String,
    pub run_id: String,
    pub version: i64,
    pub outcome: State,
    pub proof: Option<Reference>,
}
pub fn finish(
    store: &mut Store,
    actor: &TrustedActor,
    request: Finish,
    now_ms: u64,
) -> Result<Value> {
    if request.key.trim().is_empty() {
        return Err(invalid("reducer command key required"));
    }
    let project = request.project_id.as_str();
    let id = request.run_id.as_str();
    let version = request.version;
    let outcome = request.outcome;
    let proof = request.proof.as_ref();
    let input = serde_json::to_value(&request)?;
    let result_key = key(
        scope(project),
        "run_finish_command",
        &bytes_sha256(request.key.as_bytes()),
    );
    if let Some(record) = store.get(&result_key)? {
        let previous: Value = decode(&record)?;
        if previous["actor"] != serde_json::to_value(&actor.0)? || previous["input"] != input {
            return Err(error(
                "IDEMPOTENCY_CONFLICT",
                "finish key binds original actor and predicates",
            ));
        }
        return Ok(previous["result"].clone());
    }
    let (record, mut run) = load(store, project, id)?;
    let scoped = reducer(actor, &run)?;
    if record.version != version {
        return Err(chat::stale());
    }
    if !matches!(outcome, State::Failed | State::Completed) || !run.lifecycle.allows(outcome) {
        return Err(error(
            "INVALID_TRANSITION",
            "reducer finish is not allowed from this lifecycle",
        ));
    }
    let (_, m) = manifest(store, &scoped, project, id)?;
    let mut plan = Plan {
        input: Input {
            key: format!("finish:{id}:{version}"),
            action: Action::Cancel {
                project_id: project.into(),
                run_id: id.into(),
                version,
            },
        },
        checks: vec![],
        records: vec![],
        effects: vec![],
        cancel_effects: vec![],
        result: Value::Null,
    };
    if outcome == State::Completed {
        let proof = proof.ok_or_else(|| {
            error(
                "RUN_COMPLETION_UNPROVEN",
                "all four control completion predicates must be known",
            )
        })?;
        if proof.key.scope != scope(project)
            || proof.key.kind != "run_completion"
            || !run.authorization_valid
        {
            return Err(error(
                "RUN_COMPLETION_UNPROVEN",
                "control proof for this authorized Run required",
            ));
        }
        let r = exact(store, proof, &mut plan.checks)?;
        let p: CompletionProof = decode(&r)?;
        let binding = watched(
            store,
            &key(scope(project), "run_engine_binding", id),
            &mut plan.checks,
        )?
        .ok_or_else(|| error("RUN_COMPLETION_UNPROVEN", "Run engine binding missing"))?;
        if p.manifest != run.manifest
            || p.engine_binding != reference(&binding)
            || [
                p.all_required_satisfied,
                p.all_attempts_terminal_or_revoked,
                p.no_unresolved_output_effects,
                p.references_match,
            ]
            .iter()
            .any(|v| *v != Some(true))
        {
            return Err(error(
                "RUN_COMPLETION_UNPROVEN",
                "unknown or mismatching completion predicate",
            ));
        }
        let mut owners = vec![record.key.clone()];
        for r in store.list("run_activity")? {
            if r.key.scope != scope(project) {
                continue;
            }
            let a: Activity = decode(&r)?;
            if a.run_id == id {
                track(&mut plan.checks, &r);
                if a.authorization_valid {
                    return Err(error(
                        "RUN_COMPLETION_UNPROVEN",
                        "Attempt authority still active",
                    ));
                }
                owners.push(a.owner.key);
            }
        }
        for id in store.pending_effects()? {
            let (e, _) = store.effect(&id)?;
            if owners.contains(&e.owner.key) {
                return Err(error(
                    "RUN_COMPLETION_UNPROVEN",
                    "Run-owned effect remains unresolved",
                ));
            }
        }
        run.lifecycle = State::Completed;
        run.authorization_valid = false;
        run.entered_at_ms = now_ms;
        release_slot(store, project, &run, &mut plan, Some("completion_pending"))?;
    } else {
        let safe = revoke(store, project, id, &mut plan)?;
        run.authorization_valid = false;
        run.terminal_requested = Some(State::Failed);
        run.operation_epoch += 1;
        if safe {
            run.lifecycle = State::Failed;
            run.entered_at_ms = now_ms;
            release_slot(store, project, &run, &mut plan, None)?;
        } else {
            run.attention = Some("failure pending verified write isolation".into());
        }
        let next = value_record(record.key.clone(), version + 1, &run)?;
        plan.effects
            .push(effect(&m, &next, "stop", run.operation_epoch)?);
    }
    let result = json!({"run_id":id,"lifecycle":run.lifecycle,"authorization_valid":run.authorization_valid,"attention":run.attention});
    let command = Command {
        command_id: format!("run-finish:{}", request.key),
        idempotency_key: format!("run-finish:{}", request.key),
        actor: scoped.0.clone(),
        target: record.key.clone(),
        expected: Expected::Exact(Version::State(version)),
        binding: run.manifest.clone(),
        operation: "run.finish".into(),
        input_digest: Command::digest_input("run.finish", &input)?,
        input: input.clone(),
    };
    store.submit(store.generation(), &scoped, &command, None, |tx| {
        for check in &plan.checks {
            if tx.get(&check.key)?.as_ref().map(|r| r.version) != check.version {
                return Err(chat::stale());
            }
        }
        tx.put(&value_record(record.key.clone(), version + 1, &run)?)?;
        for record in &plan.records {
            tx.put(record)?;
        }
        for id in &plan.cancel_effects {
            tx.cancel_pending_effect(id)?;
        }
        for effect in &plan.effects {
            tx.enqueue_effect(effect)?;
        }
        tx.put(&value_record(
            result_key.clone(),
            1,
            &json!({"actor":actor.0,"input":input,"result":result}),
        )?)?;
        Ok(result.clone())
    })
}
/// Package 4a must register each owner in the same transaction that authorizes
/// its dispatch. Revoked/paused Runs cannot gain a new child authority.
pub fn authorize_activity_in(
    tx: &mut store::CommandTransaction<'_>,
    actor: &TrustedActor,
    activity: &Activity,
    spec: &agency_proto::Sealed<agency_proto::ExecutionSpec>,
    bundle: &agency_proto::Sealed<agency_proto::context::Bundle>,
) -> Result<()> {
    let Scope::Project(project) = &activity.manifest.key.scope else {
        return Err(invalid("Run Manifest Project scope required"));
    };
    if activity.manifest.key.kind != "run_manifest" || activity.manifest.key.id != activity.run_id {
        return Err(invalid("Run Manifest identity required"));
    }
    let record = tx.get(&run_key(project, &activity.run_id))?;
    let record = record.ok_or_else(|| invalid("Run missing"))?;
    let run: Run = decode(&record)?;
    reducer(actor, &run)?;
    if run.lifecycle != State::Running
        || !run.authorization_valid
        || activity.manifest != run.manifest
        || !activity.authorization_valid
        || activity.owner.key.scope != record.key.scope
        || activity.owner.key.kind != "run_attempt"
    {
        return Err(error("RUN_REVOKED", "Run cannot authorize this Attempt"));
    }
    let owner = tx
        .get(&activity.owner.key)?
        .ok_or_else(|| invalid("Attempt must be persisted before authorization"))?;
    if reference(&owner) != activity.owner {
        return Err(chat::stale());
    }
    let value: Value = decode(&owner)?;
    if value["spec"] != serde_json::to_value(spec)? {
        return Err(invalid("Attempt record does not freeze this Spec"));
    }
    spec.verify().map_err(|e| invalid(e.to_string()))?;
    spec.document
        .validate()
        .map_err(|e| invalid(e.to_string()))?;
    let manifest_record = tx
        .get(&run.manifest.key)?
        .ok_or_else(|| invalid("Run Manifest missing"))?;
    let manifest: Manifest = decode(&manifest_record)?;
    let selected = manifest
        .selected
        .iter()
        .find(|s| s.selection.key.id == spec.document.selection.id)
        .ok_or_else(|| invalid("Attempt selection is not in this Run"))?;
    let selected_record = tx
        .get(&selected.selection.key)?
        .ok_or_else(|| invalid("Run selection missing"))?;
    if spec.document.owner.kind != agency_proto::OwnerKind::RunAttempt
        || spec.document.owner.project != run.project_id
        || spec.document.owner.id != activity.owner.key.id
        || spec.document.selection != participant::frozen(&selected_record)
        || spec.document.profession.reference.digest != selected.profession_digest
        || spec.document.bundle.digest != bundle.digest
        || bundle.document.consumer != spec.document.owner
        || spec.document.manifest != bundle.document.manifest
        || spec.document.binding.id != selected.agency.key.id
        || activity.can_write != spec.document.write_lease.is_some()
        || spec.document.selection_policy_digest != manifest.selection_policy_digest
        || spec
            .document
            .permissions
            .iter()
            .any(|p| !selected.permissions.contains(p))
        || spec.document.budget > selected.max_context_bytes
        || !selected
            .worker_profiles
            .iter()
            .zip(&selected.profile_digests)
            .any(|(r, d)| {
                spec.document.profile.id == r.key.id && spec.document.profile.digest == *d
            })
    {
        return Err(invalid(
            "Attempt Spec exceeds its frozen Run selection or write boundary",
        ));
    }
    validate_delivery_contract(&manifest, selected, bundle)?;
    let k = key(record.key.scope, "run_activity", &activity.owner.key.id);
    if tx.get(&k)?.is_some() {
        return Err(invalid("Attempt authority already registered"));
    }
    tx.put(&value_record(k, 1, activity)?)
}
