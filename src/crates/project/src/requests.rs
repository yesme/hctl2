use crate::*;
use foundation::{bytes_sha256, canonical_json_sha256};
use serde_json::{Value, json};
use store::{
    Command, EffectIntent, EffectState, Expected, Readback, Record, Reference, Scope, Store,
    TrustedActor, Version,
};

pub fn requests(store: &Store, project: &str) -> Result<Vec<(Record, Request)>> {
    store
        .list("request")?
        .into_iter()
        .filter(|r| r.key.scope == Scope::Project(project.into()))
        .map(|r| Ok((r.clone(), decode(&r)?)))
        .collect()
}
fn request(store: &Store, project: &str, id: &str) -> Result<(Record, Request)> {
    let r = required(store, &key(Scope::Project(project.into()), "request", id))?;
    let value = decode(&r)?;
    Ok((r, value))
}
fn blocker_key(project: &str, root: &str) -> store::ObjectKey {
    key(
        Scope::Project(project.into()),
        "task_request_blocker",
        &bytes_sha256(root.as_bytes()),
    )
}
pub fn may_answer(
    store: &Store,
    project: &str,
    spec: &RequestSpec,
    actor: &TrustedActor,
) -> Result<bool> {
    Ok(match &spec.target {
        Target::Human { principal } => principal == &actor.0.principal,
        Target::Role { role } => definition(store, project)?
            .role_members
            .get(role)
            .is_some_and(|m| m.contains(&actor.0.principal)),
    })
}
fn check(plan: &mut Plan, r: &Record) {
    plan.checks.push(chat::Check {
        key: r.key.clone(),
        version: Some(r.version),
    });
}

pub(crate) fn prepare_request(
    store: &Store,
    _actor: &TrustedActor,
    plan: &mut Plan,
    spec: &RequestSpec,
    now: u64,
) -> Result<()> {
    let scope = Scope::Project(plan.project_id.clone());
    if spec.question.trim().is_empty()
        || spec.blocking_scope.trim().is_empty()
        || spec.dedup_root.trim().is_empty()
        || spec.owner.key.scope != scope
        || spec.owner.key.kind != "task_state"
        || spec.permissions != json!({"action":"task.adopt"})
        || spec.deadline.is_some_and(|d| d <= now)
        || spec.input_schema != "hctl2.task.Adoption.v1"
    {
        return Err(invalid(
            "precise current Task blocker, question, scope and future deadline required",
        ));
    }
    match &spec.target {
        Target::Human { principal } if principal.trim().is_empty() => {
            return Err(invalid("human target required"));
        }
        Target::Role { role } if !definition(store, &plan.project_id)?.roles.contains(role) => {
            return Err(invalid("undeclared role"));
        }
        _ => {}
    }
    let (source, task) = task::task(store, &plan.project_id, &spec.owner.key.id)?;
    if reference(&source) != spec.owner
        || task.state_version != spec.owner_state_version
        || task.archived
        || task.lifecycle != "open"
    {
        return Err(stale());
    }
    if let Some(revision) = &spec.affected_revision {
        // Arbitrary material or another Task is not this owner's affected revision.
        let expected = task.revision.as_ref().map(|r| {
            key(
                scope.clone(),
                "task_revision",
                &format!("{}:{}", task.id, r.number),
            )
        });
        if expected.as_ref() != Some(&revision.key) {
            return Err(invalid("affected revision is not this Task's revision"));
        }
        chat::at(store, revision)?;
    }
    check(plan, &source);
    let waiting_key = blocker_key(&plan.project_id, &spec.dedup_root);
    // This source adapter has one contract input per exact Task state. Multiple
    // independent roots for that same input would make every answer unfulfillable.
    if task::request_blockers(store, &plan.project_id, &source.key.id)?
        .iter()
        .any(|(r, b)| {
            r.key != waiting_key && b.owner == spec.owner && b.waiting && b.outcome.is_none()
        })
    {
        return Err(reject(
            "REQUEST_BUSY",
            "the same Task contract input already has a Request",
            "inspect_original_request",
        ));
    }
    let active: Vec<_> = requests(store, &plan.project_id)?
        .into_iter()
        .filter(|(_, r)| r.state == RequestState::Open && r.spec.dedup_root == spec.dedup_root)
        .collect();
    for (r, old) in &active {
        check(plan, r);
        if old.spec == *spec {
            plan.result = json!({"request_id":r.key.id,"version":r.version,"state":"open","deduplicated":true});
            return Ok(());
        }
    }
    let id = format!(
        "request-{}",
        bytes_sha256(format!("{}:{}", plan.project_id, plan.input.key).as_bytes())
    );
    for (r, mut old) in active {
        old.state = RequestState::Superseded;
        old.superseded_by = Some(id.clone());
        plan.records.push(value_record(r.key, r.version + 1, &old)?);
    }
    let waiting = store.get(&waiting_key)?;
    if let Some(r) = &waiting {
        check(plan, r);
    } else {
        plan.checks.push(chat::Check {
            key: waiting_key.clone(),
            version: None,
        });
    }
    // A resolution in delivery is not silently replaced by a new pending question.
    if waiting
        .as_ref()
        .map(decode::<Blocker>)
        .transpose()?
        .is_some_and(|b| b.delivery.is_some() && b.waiting)
    {
        return Err(reject(
            "REQUEST_DELIVERY_PENDING",
            "settle original delivery first",
            "resume_original_command",
        ));
    }
    let request = Request {
        question: spec.question.clone(),
        blockers: vec![spec.owner.clone()],
        spec: spec.clone(),
        state: RequestState::Open,
        created_at: now,
        superseded_by: None,
        solution_digest: None,
        effect_id: None,
        actor: None,
    };
    plan.records
        .push(value_record(key(scope, "request", &id), 1, &request)?);
    plan.records.push(value_record(
        waiting_key,
        waiting.map_or(1, |r| r.version + 1),
        &Blocker {
            owner: spec.owner.clone(),
            request_id: id.clone(),
            waiting: true,
            delivery: None,
            outcome: None,
        },
    )?);
    plan.result = json!({"request_id":id,"version":1,"state":"open","deduplicated":false});
    Ok(())
}

pub(crate) fn prepare_resolution(
    store: &Store,
    actor: &TrustedActor,
    plan: &mut Plan,
    id: &str,
    version: i64,
    adoption: &task::Adoption,
    now: u64,
) -> Result<()> {
    let (r, mut request) = request(store, &plan.project_id, id)?;
    if r.version != version
        || request.state != RequestState::Open
        || request.spec.deadline.is_some_and(|d| d <= now)
    {
        return Err(stale());
    }
    if !may_answer(store, &plan.project_id, &request.spec, actor)? {
        return Err(reject(
            "PERMISSION_DENIED",
            "actor is not the Request target",
            "request_authorization",
        ));
    }
    let source = required(store, &request.spec.owner.key)?;
    let source_task: task::Task = decode(&source)?;
    if reference(&source) != request.spec.owner
        || source_task.state_version != request.spec.owner_state_version
    {
        return Err(stale());
    }
    let waiting_record = required(
        store,
        &blocker_key(&plan.project_id, &request.spec.dedup_root),
    )?;
    let mut waiting: Blocker = decode(&waiting_record)?;
    if waiting.request_id != id
        || !waiting.waiting
        || waiting.delivery.is_some()
        || waiting.owner != request.spec.owner
    {
        return Err(stale());
    }
    let p = project(store, &plan.project_id)?;
    // The source's existing typed operation validates its schema and all source refs.
    let mut task_plan = task::prepare_request_adoption(
        store,
        task::Input {
            key: plan.input.key.clone(),
            action: task::Action::Adopt {
                project_id: plan.project_id.clone(),
                project_version: p.version,
                task_id: source.key.id.clone(),
                version: source.version,
                adoption: adoption.clone(),
            },
        },
        &waiting_record,
    )?;
    let effect_id = format!(
        "request:{}",
        bytes_sha256(format!("{}:{}", plan.project_id, plan.input.key).as_bytes())
    );
    request.state = RequestState::Resolved;
    request.solution_digest = Some(canonical_json_sha256(&serde_json::to_value(adoption)?)?);
    request.effect_id = Some(effect_id.clone());
    request.actor = Some(actor.0.clone());
    waiting.delivery = Some(effect_id.clone());
    check(plan, &r);
    check(plan, &source);
    check(plan, &waiting_record);
    plan.checks
        .extend(task_plan.checks.iter().map(|c| chat::Check {
            key: c.key.clone(),
            version: c.version,
        }));
    // The source can cite this answer's Open Request. Its exact historical origin
    // stays frozen, while its transaction precondition expects our Resolved version.
    for c in &mut task_plan.checks {
        if c.key == r.key {
            c.version = Some(r.version + 1);
        }
    }
    let resolved = value_record(r.key, r.version + 1, &request)?;
    let effect = EffectIntent {
        intent_id: effect_id.clone(),
        owner: reference(&resolved),
        binding: Reference {
            key: key(Scope::Control, "module", "task"),
            version: Version::State(1),
        },
        operation: "request.task_adopt".into(),
        target: source.key.id.clone(),
        conflict_scope: format!("request:{}:{}", plan.project_id, request.spec.dedup_root),
        permission_scope: source.key.scope,
        input: json!({"task_plan":task_plan,"request_id":id}),
        input_digest: String::new(),
        idempotency_key: effect_id.clone(),
    };
    let mut effect = effect;
    effect.input_digest = Command::digest_input(&effect.operation, &effect.input)?;
    plan.records.push(resolved);
    plan.records.push(value_record(
        waiting_record.key,
        waiting_record.version + 1,
        &waiting,
    )?);
    plan.effects.push(effect);
    plan.task_plan = Some(task_plan);
    plan.result = json!({"request_id":id,"version":version+1,"state":"resolved","delivery":"pending","effect_id":effect_id});
    Ok(())
}

pub(crate) fn cancel_request(store: &Store, plan: &mut Plan, id: &str, version: i64) -> Result<()> {
    let (r, mut request) = request(store, &plan.project_id, id)?;
    if r.version != version || request.state != RequestState::Open {
        return Err(stale());
    }
    let waiting_record = required(
        store,
        &blocker_key(&plan.project_id, &request.spec.dedup_root),
    )?;
    let mut waiting: Blocker = decode(&waiting_record)?;
    if waiting.request_id != id || waiting.delivery.is_some() {
        return Err(stale());
    }
    request.state = RequestState::Cancelled;
    // Cancelling the question does not assert that its required input was supplied.
    waiting.outcome = Some("cancel_waiting".into());
    check(plan, &r);
    check(plan, &waiting_record);
    plan.records
        .push(value_record(r.key, r.version + 1, &request)?);
    plan.records.push(value_record(
        waiting_record.key,
        waiting_record.version + 1,
        &waiting,
    )?);
    plan.result = json!({"request_id":id,"state":"cancelled","source_waiting":true});
    Ok(())
}

/// A local module receiver confirms the unique delivery and applies the original
/// typed Task operation atomically. A lost response can never apply it twice.
pub fn deliver(store: &mut Store, actor: &TrustedActor, id: &str) -> Result<Value> {
    let (effect, state) = store.effect(id)?;
    if effect.operation != "request.task_adopt" {
        return Err(invalid("not a Request delivery"));
    }
    let Scope::Project(project_id) = effect.permission_scope.clone() else {
        return Err(invalid("Project scope required"));
    };
    let actor = chat::owner(actor, &project_id)?;
    let receipt_key = key(
        effect.permission_scope.clone(),
        "request_delivery_receipt",
        id,
    );
    if state == EffectState::Confirmed {
        return decode(&required(store, &receipt_key)?);
    }
    let effect = if state == EffectState::Pending {
        store.begin_effect(store.generation(), id)?
    } else {
        effect
    };
    let mut task_plan: task::Plan = serde_json::from_value(effect.input["task_plan"].clone())?;
    // Accepted authorization keeps its Project version and policies. A change in
    // defaults does not revoke it, but archiving still prevents delivery.
    let current_project = project(store, &project_id)?;
    if readonly(&current_project) {
        return Err(reject(
            "PROJECT_READ_ONLY",
            "Project is archived",
            "restore_project",
        ));
    }
    for c in &mut task_plan.checks {
        if c.key == current_project.key {
            c.version = Some(current_project.version);
        }
    }
    let request_id = effect.input["request_id"]
        .as_str()
        .ok_or_else(|| invalid("missing frozen Request"))?;
    let (r, request) = request(store, &project_id, request_id)?;
    let waiting_record = required(store, &blocker_key(&project_id, &request.spec.dedup_root))?;
    let mut waiting: Blocker = decode(&waiting_record)?;
    if reference(&r) != effect.owner
        || request.state != RequestState::Resolved
        || waiting.request_id != request_id
        || waiting.delivery.as_deref() != Some(id)
        || waiting.owner != request.spec.owner
    {
        return Err(stale());
    }
    waiting.waiting = false;
    waiting.outcome = Some("input_received".into());
    let result = json!({"request_id":request_id,"effect_id":id,"delivery":"confirmed","task":task_plan.result});
    let input = json!({"effect_id":id,"input_digest":effect.input_digest});
    let cmd = Command {
        command_id: format!("receipt:{id}"),
        idempotency_key: format!("receipt:{id}"),
        actor: actor.0.clone(),
        target: receipt_key.clone(),
        expected: Expected::Absent,
        binding: effect.binding.clone(),
        operation: "request.receive".into(),
        input_digest: Command::digest_input("request.receive", &input)?,
        input,
    };
    store.submit(store.generation(), &actor, &cmd, None, |tx| {
        if tx.get(&r.key)?.as_ref().map(reference) != Some(effect.owner.clone())
            || tx.get(&waiting_record.key)?.as_ref().map(|r| r.version)
                != Some(waiting_record.version)
        {
            return Err(stale());
        }
        task::apply(tx, &task_plan)?;
        tx.put(&value_record(
            waiting_record.key.clone(),
            waiting_record.version + 1,
            &waiting,
        )?)?;
        tx.put(&value_record(receipt_key.clone(), 1, &result)?)?;
        tx.confirm_effect(
            id,
            &Readback::Confirmed {
                binding: effect.binding.clone(),
                target: effect.target.clone(),
                input_digest: effect.input_digest.clone(),
                result: result.clone(),
            },
        )?;
        Ok(result.clone())
    })
}

/// Deadlines are one CAS transition, not answers and not Task terminal commands.
pub fn expire(
    store: &mut Store,
    actor: &TrustedActor,
    project_id: &str,
    now: u64,
) -> Result<usize> {
    if readonly(&project(store, project_id)?) {
        return Ok(0);
    }
    let actor = chat::owner(actor, project_id)?;
    let mut count = 0;
    for (r, mut request) in requests(store, project_id)? {
        if request.state != RequestState::Open || !request.spec.deadline.is_some_and(|d| d <= now) {
            continue;
        }
        let b = required(store, &blocker_key(project_id, &request.spec.dedup_root))?;
        let mut waiting: Blocker = decode(&b)?;
        if waiting.request_id != r.key.id || waiting.delivery.is_some() {
            continue;
        }
        // A stale source leaves history intact; it must not starve other deadlines.
        if store.get(&request.spec.owner.key)?.as_ref().map(reference)
            != Some(request.spec.owner.clone())
        {
            continue;
        }
        request.state = RequestState::Expired;
        waiting.outcome = Some(
            match request.spec.deadline_action {
                DeadlineAction::FailWaiting => "fail_waiting",
                DeadlineAction::CancelWaiting => "cancel_waiting",
            }
            .into(),
        );
        let input = json!({"request":reference(&r),"deadline":request.spec.deadline});
        let cmd = Command {
            command_id: format!("expire:{}:{}", r.key.id, r.version),
            idempotency_key: format!("expire:{}:{}", r.key.id, r.version),
            actor: actor.0.clone(),
            target: r.key.clone(),
            expected: Expected::Exact(Version::State(r.version)),
            binding: Reference {
                key: key(Scope::Control, "module", "project"),
                version: Version::State(1),
            },
            operation: "request.expire".into(),
            input_digest: Command::digest_input("request.expire", &input)?,
            input,
        };
        store.submit(store.generation(), &actor, &cmd, None, |tx| {
            if tx.get(&b.key)?.as_ref().map(|r| r.version) != Some(b.version) {
                return Err(stale());
            }
            if tx.get(&request.spec.owner.key)?.as_ref().map(reference)
                != Some(request.spec.owner.clone())
            {
                return Err(stale());
            }
            tx.put(&value_record(r.key.clone(), r.version + 1, &request)?)?;
            tx.put(&value_record(b.key.clone(), b.version + 1, &waiting)?)?;
            Ok(json!({"state":"expired"}))
        })?;
        count += 1;
    }
    Ok(count)
}
