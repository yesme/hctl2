//! Task transport, provider I/O and recovery; Store locks cover only short transactions.
#[cfg(test)]
mod completion_tests;
mod github;
mod provider;
use crate::services::Supervisor;
use serde_json::{Value, json};
use std::{path::Path, sync::Arc};
use store::{EffectState, Scope, Store, TrustedActor};
use task::{Action, Input, Plan, Result, Snapshot, Source, reject};
use tokio::sync::Mutex;
type Shared = Arc<Mutex<Option<Store>>>;

fn access<T>(shared: &Shared, f: impl FnOnce(&mut Store) -> Result<T>) -> Result<T> {
    let mut slot = shared.blocking_lock();
    f(slot
        .as_mut()
        .ok_or_else(|| reject("STORE_NOT_READY", "store not ready", "check_status"))?)
}
pub(super) fn input(operation: &str, payload: &Value) -> Result<Input> {
    let input: Input = serde_json::from_value(payload.clone())?;
    let action = serde_json::to_value(&input.action)?;
    if operation != format!("task.{}", action["kind"].as_str().unwrap_or("invalid")) {
        return Err(reject(
            "INVALID_INPUT",
            "operation and Task action disagree",
            "correct_input",
        ));
    }
    Ok(input)
}
fn read(
    shared: &Shared,
    services: &Supervisor,
    root: &Path,
    rid: &str,
    id: &str,
    action: Option<&Action>,
) -> Result<(Source, Snapshot)> {
    let (r, src, control, stamp, previous, entity) = access(shared, |s| {
        let (r, src) = task::source(s, rid, id)?;
        let entity = match action {
            Some(Action::Claim { entity_id, .. }) => Some(entity_id.clone()),
            Some(
                Action::Adopt {
                    project_id,
                    task_id,
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
                },
            ) => task::task(s, project_id, task_id)?
                .1
                .entity
                .map(|e| e.immutable_external_entity_id),
            _ => None,
        };
        let previous = task::latest(s, &src).ok().map(|(_, snap)| snap);
        Ok((
            r,
            src,
            s.control_id().to_owned(),
            s.read_stamp(),
            previous,
            entity,
        ))
    })?;
    if !src.active {
        return Err(reject(
            "SOURCE_DISABLED",
            "source disabled",
            "reconnect_source",
        ));
    }
    let result = provider::Client::connect(&src, root, &control, services).and_then(|c| {
        match (&previous, entity) {
            (Some(previous), Some(entity))
                if previous
                    .cards
                    .iter()
                    .any(|c| c.entity.immutable_external_entity_id == entity) =>
            {
                c.refresh_card(&src, previous, &entity)
            }
            _ => c.snapshot(&src, task::reference(&r)),
        }
    });
    match result {
        Ok(snap) => {
            access(shared, |s| {
                if s.read_stamp() != stamp {
                    return Err(task::stale());
                }
                task::observe(s, &src, snap.clone())
            })?;
            Ok((src, snap))
        }
        Err(e) => {
            let failed = Snapshot {
                source: task::reference(&r),
                observed_at: task::now(),
                complete: false,
                error: Some(e.code.into()),
                cards: vec![],
                stable_groups: vec![],
            };
            access(shared, |s| {
                if s.read_stamp() != stamp {
                    return Err(task::stale());
                }
                task::observe(s, &src, failed)
            })?;
            Err(e)
        }
    }
}

pub(super) fn preview(
    shared: &Shared,
    services: &Supervisor,
    root: &Path,
    actor: &TrustedActor,
    operation: &str,
    payload: &Value,
) -> Result<Value> {
    let input = input(operation, payload)?;
    if let Some(p) = access(shared, |s| task::replay(s, &input))? {
        return Ok(serde_json::to_value(p)?);
    }
    if let Action::Connect { repo_id, .. } = &input.action {
        let reg = access(shared, |s| repo::require_active(s, repo_id))?;
        let observed = match reg.prepared.platform {
            repo::Platform::Github => crate::scm::github(&reg, services)?,
            repo::Platform::Local => {
                crate::scm::Hosted::connect(root, &reg.config.control_id, services)?
                    .repository(&reg, false)?
                    .ok_or_else(|| {
                        reject(
                            "PLATFORM_UNAVAILABLE",
                            "registered repository missing",
                            "inspect_repo",
                        )
                    })?
            }
            repo::Platform::None => {
                return Err(reject(
                    "SOURCE_UNAVAILABLE",
                    "local task server is not in P2.2",
                    "choose_platform_issues",
                ));
            }
        };
        access(shared, |s| repo::refresh_platform(s, repo_id, observed))?;
    }
    // Explicit source refresh is also an observation, never an implicit contract adoption.
    if let Some((rid, id)) = access(shared, |s| task::source_id_for_action(s, &input.action))? {
        read(shared, services, root, &rid, &id, Some(&input.action))?;
    }
    // 预览与准入同一套判定：操作声明的项目要落在 actor 的作用域里，project 头才读得到物料。
    let scoped;
    let actor = match task::project_id_for_action(&input.action) {
        Some(project) => {
            scoped = task::owner(actor, [Scope::Project(project.into())])?;
            &scoped
        }
        None => actor,
    };
    let mut plan = access(shared, |s| task::prepare_as(s, actor, input))?;
    if matches!(plan.input.action, Action::Connect { .. }) {
        let control = access(shared, |s| Ok(s.control_id().to_owned()))?;
        for record in plan
            .records
            .iter_mut()
            .filter(|r| r.key.kind == "task_source")
        {
            let mut src: Source = task::decode(record)?;
            src.capabilities.delete =
                provider::Client::connect(&src, root, &control, services)?.can_delete(&src)?;
            *record = task::value_record(record.key.clone(), record.version, &src)?;
        }
    }
    Ok(serde_json::to_value(plan)?)
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
    let payload = serde_json::from_slice(&request.payload)?;
    let input = input(&request.operation, &payload)?;
    let generation = access(shared, |s| Ok(s.generation()))?;
    if request.idempotency_key != input.key || request.command_id != format!("task:{}", input.key) {
        return Err(reject(
            "INVALID_INPUT",
            "envelope key mismatch",
            "use_original_command",
        ));
    }
    let plan: Plan = serde_json::from_value(details.clone())?;
    if serde_json::to_value(&plan.input)? != serde_json::to_value(&input)? {
        return Err(task::stale());
    }
    let replay = access(shared, |s| task::replay(s, &input))?.is_some();
    if !replay
        && let Some((rid, id)) = access(shared, |s| task::source_id_for_action(s, &input.action))?
    {
        read(shared, services, root, &rid, &id, Some(&input.action))?;
    }
    // Only actual writes serialize with restore/service maintenance. Polls and the
    // current read above do not hold this guard while traversing a board.
    let _guard = operations.blocking_lock();
    access(shared, |s| {
        if s.generation() == generation {
            Ok(())
        } else {
            Err(task::stale())
        }
    })?;
    let mut result = access(shared, |s| task::admit(s, actor, plan))?;
    if let Some(id) = result["effect_id"].as_str().map(str::to_owned) {
        if let Err(e) = drive(shared, services, root, actor, &id) {
            result["error"] =
                json!({"code":e.code,"message":e.message,"recovery_action":e.recovery_action});
        }
        result["effect_state"] = access(shared, |s| Ok(serde_json::to_value(s.effect(&id)?.1)?))?;
    }
    if let Some(project) = result["project_id"].as_str().map(str::to_owned) {
        result["provider_done"] = drive_completion_requests(shared, &project, actor)?;
    }
    if let (Some(project), Some(id)) = (result["project_id"].as_str(), result["task_id"].as_str()) {
        result["task"] = access(shared, |s| {
            let (r, t) = task::task(s, project, id)?;
            Ok(json!({"version":r.version,"data":t}))
        })?;
    }
    Ok(result)
}

/// 供应端 Done 归档出的「完成 Task」请求：走与 Workbench/CLI 相同的预览与准入。
/// 只有绑定声明允许自动提交时才提交；被拒时保留外部 Done 与 HCTL 开放，把类型化
/// 结果记在请求上，等人处理。
fn drive_completion_requests(
    shared: &Shared,
    project: &str,
    actor: &TrustedActor,
) -> Result<Value> {
    let pending = access(shared, |s| {
        Ok(s.list(task::COMPLETION_REQUEST_KIND)?
            .into_iter()
            .filter(|r| r.key.scope == Scope::Project(project.into()))
            .filter_map(|r| {
                task::decode::<task::CompletionRequest>(&r)
                    .ok()
                    .map(|request| (r, request))
            })
            .filter(|(_, request)| request.state == "pending")
            .collect::<Vec<_>>())
    })?;
    let mut handled = vec![];
    for (_, request) in pending {
        let outcome = access(shared, |s| {
            let (_, src) = task::source(s, &request.repo_id, &request.source_id)?;
            if !src.auto_complete_provider_done {
                return Ok(json!({
                    "request_id": request.request_id,
                    "state": "pending",
                    "reason": "binding_does_not_auto_submit",
                }));
            }
            let (task_record, t) = task::task(s, &request.project_id, &request.task_id)?;
            // 与 Workbench/CLI 同源：由已认证的 owner 以自己的身份完成，供应端归属记在请求上。
            let actor = task::owner(actor, [Scope::Project(request.project_id.clone())])?;
            let evidence =
                task::provider_evidence(s, &actor, &t, &request.snapshot, &src.port_kind)?;
            let input = task::Input {
                key: format!("provider-done:{}", request.idempotency_key),
                action: Action::Complete {
                    project_id: request.project_id.clone(),
                    task_id: request.task_id.clone(),
                    version: task_record.version,
                    lifecycle_version: t.lifecycle_version,
                    revision_number: t.revision.as_ref().map(|r| r.number).unwrap_or(0),
                    acceptance: evidence,
                },
            };
            match task::prepare_as(s, &actor, input).and_then(|plan| task::admit(s, &actor, plan)) {
                Ok(value) => {
                    task::mark_completion_request(
                        s,
                        project,
                        &request.request_id,
                        "accepted",
                        None,
                    )?;
                    Ok(json!({
                        "request_id": request.request_id,
                        "state": "accepted",
                        "result": value,
                    }))
                }
                Err(error) => {
                    task::mark_completion_request(
                        s,
                        project,
                        &request.request_id,
                        "declined",
                        Some(json!({
                            "code": error.code,
                            "message": error.message,
                            "recovery_action": error.recovery_action,
                        })),
                    )?;
                    Ok(json!({
                        "request_id": request.request_id,
                        "state": "declined",
                        "error": {
                            "code": error.code,
                            "message": error.message,
                            "recovery_action": error.recovery_action,
                        },
                    }))
                }
            }
        })?;
        handled.push(outcome);
    }
    Ok(Value::Array(handled))
}

fn drive(
    shared: &Shared,
    services: &Supervisor,
    root: &Path,
    actor: &TrustedActor,
    id: &str,
) -> Result<()> {
    let (e, state) = access(shared, |s| s.effect(id))?;
    if state == EffectState::Rejected {
        return Err(reject(
            "PROVIDER_REJECTED",
            "original attempt was rejected by the platform",
            "refresh_and_preview",
        ));
    }
    if matches!(state, EffectState::Confirmed | EffectState::Cancelled) {
        return Ok(());
    }
    let rid = field(&e.input, "repo_id")?;
    let sid = field(&e.input, "source_id")?;
    let (src, control) = access(shared, |s| {
        Ok((task::source(s, rid, sid)?.1, s.control_id().to_owned()))
    })?;
    // Bootstrap/connect failure occurs before uncertainty, so the untouched intent stays Pending.
    let client = provider::Client::connect(&src, root, &control, services)?;
    let observed = client.effect_with_dispatch(
        &src,
        &e,
        state == EffectState::Pending,
        || {
            access(shared, |s| {
                if !task::begin(s, actor, id)?.1 {
                    return Err(task::stale());
                }
                Ok(())
            })
        },
        |evidence| access(shared, |s| task::reject_effect(s, id, evidence)),
    )?;
    access(shared, |s| {
        // The desired result can already exist before dispatch. Authorize the original
        // intent before confirming that readback, without sending a redundant write.
        task::begin(s, actor, id)?;
        task::confirm(s, id, &observed)
    })?;
    // Refresh every binding to this entity, including the other Project's projection.
    let (sr, previous, stamp) = access(shared, |s| {
        Ok((
            task::source(s, rid, sid)?.0,
            task::latest(s, &src).ok().map(|(_, snap)| snap),
            s.read_stamp(),
        ))
    })?;
    let snap = if let Some(previous) = previous {
        if previous.cards.iter().any(|c| c.entity == observed.entity) {
            client.refresh_card(
                &src,
                &previous,
                &observed.entity.immutable_external_entity_id,
            )?
        } else {
            // A newly created card is the only addition; no need to refetch other cards.
            let mut snap = previous;
            snap.source = task::reference(&sr);
            snap.cards.push(observed);
            snap.cards.sort_by(|a, b| {
                a.entity
                    .immutable_external_entity_id
                    .cmp(&b.entity.immutable_external_entity_id)
            });
            snap
        }
    } else {
        client.snapshot(&src, task::reference(&sr))?
    };
    access(shared, |s| {
        if s.read_stamp() == stamp {
            task::observe(s, &src, snap)
        } else {
            Ok(())
        }
    })?;
    Ok(())
}

fn field<'a>(v: &'a Value, k: &str) -> Result<&'a str> {
    v[k].as_str()
        .ok_or_else(|| reject("INVALID_INPUT", format!("{k} required"), "correct_input"))
}

pub(super) fn query(shared: &Shared, kind: &str, payload: &Value) -> Result<Value> {
    access(shared, |s| match kind {
        "task.list" => Ok(
            json!({"items":task::tasks(s)?.into_iter().filter(|(_,t)|payload["project_id"].as_str().is_none_or(|p|p==t.project_id)).map(|(r,t)|json!({"version":r.version,"data":t})).collect::<Vec<_>>()}),
        ),
        "task.show" => {
            let (r, t) = task::task(s, field(payload, "project_id")?, field(payload, "task_id")?)?;
            let mut shown = json!({"version":r.version,"data":t,"request_blockers":task::request_blockers(s,&t.project_id,&t.id)?.into_iter().map(|(_,b)|b).collect::<Vec<_>>()});
            shown["progress"] = progress(s, &t)?;
            Ok(shown)
        }
        "task.sources" => Ok(
            json!({"items":s.list("task_source")?,"references":s.list("task_source_reference")?,"defaults":s.list("task_default_source")?}),
        ),
        "task.board" => task::board(
            s,
            field(payload, "project_id")?,
            field(payload, "source_id")?,
        ),
        _ => Err(reject(
            "INVALID_INPUT",
            "unknown Task query",
            "correct_input",
        )),
    })
}

/// Where the Task's work stands, joined from the records that already exist: the Room
/// Invocations dispatched for it, every admitted version of their ChangeSets with that
/// version's own publication and integration state, and the Completion Receipts. Order is
/// the Store's event order (`first_sequence`), never id order. Read-only; the human view of
/// `task show` turns it into goal, state, blocking, harness, evidence and the next command
/// (CT-PRODUCT: answerable in ten seconds).
fn progress(s: &Store, t: &task::Task) -> Result<Value> {
    let order =
        |key: &store::ObjectKey| -> Result<i64> { Ok(s.first_sequence(key)?.unwrap_or(i64::MAX)) };
    let mut invocations = Vec::new();
    let mut sets: Vec<(i64, String, String)> = Vec::new();
    for record in s.list("room_invocation")? {
        if record.key.scope != Scope::Project(t.project_id.clone()) {
            continue;
        }
        let store::RecordData::Value { value } = &record.data else {
            continue;
        };
        let Ok(call) = serde_json::from_value::<project::invocation::Invocation>(value.clone())
        else {
            continue;
        };
        if call.preview.input.task_id.as_deref() != Some(t.id.as_str()) {
            continue;
        }
        let sequence = order(&record.key)?;
        let id = record.key.id.clone();
        let (state, reason) = match project::invocation::lifecycle(s, &t.project_id, &id) {
            Ok((_, lifecycle)) => (
                serde_json::to_value(lifecycle.state)?,
                json!(lifecycle.reason),
            ),
            Err(_) => (json!("unknown"), Value::Null),
        };
        let write = call.preview.write.as_ref().map(|w| {
            json!({"repo_id": w.lease.pending.repo_id, "change_set_id": w.lease.pending.change_set_id})
        });
        if let Some(w) = &call.preview.write
            && !sets
                .iter()
                .any(|(_, _, cs)| *cs == w.lease.pending.change_set_id)
        {
            sets.push((
                sequence,
                w.lease.pending.repo_id.clone(),
                w.lease.pending.change_set_id.clone(),
            ));
        }
        let profession = &call.spec.document.profession;
        invocations.push((
            sequence,
            json!({
                "invocation_id": id, "state": state, "reason": reason,
                "harness": profession.harness.id, "model": profession.model, "write": write,
            }),
        ));
    }
    invocations.sort_by_key(|(sequence, _)| *sequence);
    sets.sort_by_key(|(sequence, _, _)| *sequence);
    let revisions = revisions_of(s, &sets)?;
    // Completion Receipts stay as history after a reopen; only a completed lifecycle has a
    // current one.
    let mut receipts: Vec<task::CompletionReceipt> = s
        .list(task::COMPLETION_RECEIPT_KIND)?
        .into_iter()
        .filter_map(|r| task::decode::<task::CompletionReceipt>(&r).ok())
        .filter(|c| c.task_id == t.id && c.project_id == t.project_id)
        .collect();
    receipts.sort_by_key(|c| c.lifecycle_version);
    let current = (t.lifecycle == "completed")
        .then(|| receipts.last())
        .flatten()
        .map(|c| json!({"receipt_id": c.receipt_id, "lifecycle_version": c.lifecycle_version}));
    let history: Vec<_> = receipts.iter().map(|c| json!(c.receipt_id)).collect();
    Ok(json!({
        "invocations": invocations.into_iter().map(|(_, v)| v).collect::<Vec<_>>(),
        "revisions": revisions,
        "completion": {"current": current, "history": history},
    }))
}

/// Every admitted version of these ChangeSets in admission order across all of them (the
/// Store's event order), each with what happened to that exact version: its publication and
/// its latest integration attempt.
fn revisions_of(s: &Store, sets: &[(i64, String, String)]) -> Result<Vec<Value>> {
    let order =
        |key: &store::ObjectKey| -> Result<i64> { Ok(s.first_sequence(key)?.unwrap_or(i64::MAX)) };
    let mut revisions = Vec::new();
    for (_, repo_id, change_set_id) in sets {
        let intent_id = repo::review::intent_id(s.control_id(), repo_id, change_set_id);
        let intent = repo::review::get(s, repo_id, &intent_id).ok();
        let merges = repo::integration::list(s, repo_id)?;
        for revision in repo::changeset::list_revisions(s, change_set_id)? {
            let id = revision.change_set_revision_id.clone();
            let sequence = order(&repo::integration::revision_key(repo_id, &id))?;
            let mapping = repo::integration::review_request(s, repo_id, &id)?;
            let publication = publication_of(intent.as_ref(), &id, mapping.as_ref());
            let mut attempts = Vec::new();
            for merge in merges
                .iter()
                .filter(|m| m.preview.source.change_set_revision_id == id)
            {
                attempts.push((
                    order(&repo::integration::intent_key(repo_id, &merge.intent_id))?,
                    merge,
                ));
            }
            attempts.sort_by_key(|(sequence, _)| *sequence);
            let integration = attempts.last().map(|(_, merge)| {
                json!({"intent_id": merge.intent_id, "state": merge.state,
                    "receipt_id": merge.receipt_id,
                    "attention": merge.failure.as_ref().or(merge.attention.as_ref()).map(|a| &a.code)})
            });
            revisions.push((
                sequence,
                json!({
                    "repo_id": repo_id, "change_set_id": change_set_id,
                    "change_set_revision_id": id,
                    "base_commit_sha": revision.base_commit_sha,
                    "result_tree_sha": revision.result_tree_sha,
                    "publication": publication, "integration": integration,
                }),
            ));
        }
    }
    revisions.sort_by_key(|(sequence, _)| *sequence);
    Ok(revisions.into_iter().map(|(_, value)| value).collect())
}

/// What happened to one exact version's review publication. The ChangeSet's publish intent
/// works one round at a time: its `target` is the version of the current round, `queued`
/// waits behind it, and a create-only policy records a refused newer version in attention.
/// A recorded mapping is the proof a version was published.
fn publication_of(
    intent: Option<&repo::review::Intent>,
    revision: &str,
    mapping: Option<&repo::integration::ReviewRequestRef>,
) -> Value {
    use repo::review::State;
    let Some(intent) = intent else {
        return json!({"state": "not_authorized"});
    };
    let base = |state: &str| {
        json!({"state": state, "intent_id": intent.intent_id, "repo_id": intent.repo_id,
            "branch": intent.branch, "target_branch": intent.policy.policy.target_branch,
            "review_request": mapping.map(|m| m.index)})
    };
    if mapping.is_some() {
        return base("published");
    }
    if intent
        .queued
        .as_ref()
        .map(|q| q.change_set_revision_id.as_str())
        == Some(revision)
    {
        return base("queued");
    }
    if intent.target.change_set_revision_id == revision {
        let mut shown = base(match intent.state {
            State::PendingHuman => "pending_human",
            State::Pending => "publishing",
            State::Unknown => "unknown",
            State::Failed => "failed",
            State::Published => "published",
        });
        shown["attention"] = json!(
            intent
                .failure
                .as_ref()
                .or(intent.attention.as_ref())
                .map(|a| &a.code)
        );
        return shown;
    }
    if intent.attention.as_ref().is_some_and(|a| {
        a.code == "UPDATE_NOT_ALLOWED" && a.details["unpublished_revision"] == revision
    }) {
        let mut shown = base("update_not_allowed");
        shown["attention"] = json!("UPDATE_NOT_ALLOWED");
        return shown;
    }
    base("superseded")
}

/// Periodic read-only reconciliation. No pending writes are resent by this loop.
pub(super) fn reconcile(
    shared: &Shared,
    services: &Supervisor,
    root: &Path,
    poller: &mut Poller,
) -> Result<()> {
    let sources = access(shared, |s| s.list("task_source"))?;
    for r in sources {
        let src: Source = task::decode(&r)?;
        if !src.active {
            continue;
        }
        poller.read(shared, services, root, &src)?;
    }
    Ok(())
}

#[derive(Default)]
pub(super) struct Poller {
    entries: std::collections::BTreeMap<String, PollEntry>,
}
struct PollEntry {
    generation: store::WriterGeneration,
    binding: store::Reference,
    client: provider::Client,
    full_at: u64,
}
impl Poller {
    fn read(
        &mut self,
        shared: &Shared,
        services: &Supervisor,
        root: &Path,
        src: &Source,
    ) -> Result<()> {
        let (src, binding, control, stamp, previous) = access(shared, |s| {
            let (r, current) = task::source(s, &src.repo_id, &src.id)?;
            let previous = task::latest(s, &current).ok().map(|(_, snap)| snap);
            Ok((
                current,
                task::reference(&r),
                s.control_id().to_owned(),
                s.read_stamp(),
                previous,
            ))
        })?;
        if !src.active {
            return Ok(());
        }
        let src = &src;
        let id = format!("{}:{}", src.repo_id, src.id);
        if self
            .entries
            .get(&id)
            .is_some_and(|e| e.generation != stamp.0 || e.binding != binding)
        {
            self.entries.remove(&id);
        }
        let result: Result<Snapshot> = (|| {
            if !self.entries.contains_key(&id) {
                self.entries.insert(
                    id.clone(),
                    PollEntry {
                        generation: stamp.0,
                        binding: binding.clone(),
                        client: provider::Client::connect(src, root, &control, services)?,
                        full_at: 0,
                    },
                );
            }
            let entry = self.entries.get_mut(&id).unwrap();
            let full = task::now().saturating_sub(entry.full_at) >= 900 || previous.is_none();
            let snap = entry.client.snapshot_since(
                src,
                binding.clone(),
                if full { None } else { previous.as_ref() },
            )?;
            if full {
                entry.full_at = task::now();
            }
            Ok(snap)
        })();
        let snap = match result {
            Ok(snap) => snap,
            Err(e) => {
                // A restored Gitea credential must be picked up on the next read;
                // rate-limit feedback on GitHub must instead survive between polls.
                if src.candidate.provider == "gitea_issues" {
                    self.entries.remove(&id);
                }
                Snapshot {
                    source: binding,
                    observed_at: task::now(),
                    complete: false,
                    error: Some(e.code.into()),
                    cards: vec![],
                    stable_groups: vec![],
                }
            }
        };
        access(shared, |s| {
            if s.read_stamp() != stamp {
                return Ok(());
            } // Another writer/read won; discard stale network results.
            task::observe(s, src, snap)
        })
    }
}

#[cfg(test)]
mod progress_tests {
    use super::*;

    /// Two ChangeSets of one Task: A dispatched first, B second, but B's version is admitted
    /// before A's. The list follows admission across both, so the newest is A1, not B1.
    #[test]
    fn versions_follow_admission_order_across_change_sets() {
        use repo::changeset::{self, LeaseRef, OwnerGate, ProducerRef, Seal};
        let dir = std::env::temp_dir().join(format!("hctl2-progress-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut store = Store::open(&dir).unwrap();
        let actor = TrustedActor(store::Actor {
            principal: "owner".into(),
            source: store::ActorSource::DirectClient,
            permission_scope: vec![Scope::Control, Scope::Repo("R".into())],
            authority: None,
        });
        let open = |store: &mut Store, key: &str, invocation: &str| {
            let holder = ProducerRef::Invocation {
                invocation_id: invocation.into(),
                invocation_version: 1,
            };
            let set =
                changeset::open_change_set(store, &actor, "R", 1, &"b".repeat(40), key, &holder)
                    .unwrap();
            (set, holder)
        };
        let admit =
            |store: &mut Store, set: &changeset::ChangeSet, holder: &ProducerRef, tree: &str| {
                changeset::admit(
                    store,
                    &actor,
                    Seal {
                        association_key: format!("seal-{tree}"),
                        change_set_id: set.change_set_id.clone(),
                        change_set_version: set.version,
                        lease: Some(LeaseRef {
                            lease_id: set.lease.lease_id.clone(),
                            generation: set.lease.generation,
                        }),
                        base_commit_sha: "b".repeat(40),
                        result_tree_sha: tree.repeat(40),
                        result_commit_sha: None,
                        parent_revision_id: None,
                        producer_ref: holder.clone(),
                    },
                    OwnerGate::Active,
                )
                .unwrap()
                .change_set_revision_id
            };
        let (a, a_holder) = open(&mut store, "cs-a", "inv-a");
        let (b, b_holder) = open(&mut store, "cs-b", "inv-b");
        let b1 = admit(&mut store, &b, &b_holder, "1");
        let a1 = admit(&mut store, &a, &a_holder, "2");
        // Dispatch order puts A's ChangeSet first.
        let sets = vec![
            (1, "R".to_owned(), a.change_set_id.clone()),
            (2, "R".to_owned(), b.change_set_id.clone()),
        ];
        let listed: Vec<_> = revisions_of(&store, &sets)
            .unwrap()
            .into_iter()
            .map(|r| r["change_set_revision_id"].as_str().unwrap().to_owned())
            .collect();
        assert_eq!(listed, vec![b1, a1]);
        // Without any publish intent each version says so, none is dropped.
        let states: Vec<_> = revisions_of(&store, &sets)
            .unwrap()
            .into_iter()
            .map(|r| r["publication"]["state"].clone())
            .collect();
        assert_eq!(
            states,
            vec![json!("not_authorized"), json!("not_authorized")]
        );
        drop(store);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A publish intent as the review domain stores it, for the shapes that matter here.
    fn intent(
        state: &str,
        target: &str,
        queued: Option<&str>,
        attention: Option<(&str, &str)>,
    ) -> repo::review::Intent {
        let revision = |id: &str| json!({"change_set_revision_id": id, "base_commit_sha": "b".repeat(40), "result_tree_sha": "c".repeat(40), "commit_sha": null});
        serde_json::from_value(json!({
            "intent_id": "pub", "repo_id": "R", "change_set_id": "cs-1",
            "policy": {"policy_id": "p", "version": 1, "digest": "d", "policy": {
                "repo_id": "R", "binding_version": 1, "branch_rule": "hctl2/{change_set}",
                "target_branch": "main", "allow_update": false, "description_source": "none",
                "requires_human_confirmation": false, "audit_scope": "minimal"}},
            "branch": "hctl2/cs-1",
            "authorized_by": {"kind": "invocation", "invocation_id": "inv", "invocation_version": 1},
            "authorizing_actor": {"principal": "owner", "source": "direct_client", "permission_scope": [{"kind": "control"}], "authority": null},
            "state": state, "round": 1, "binding_version": 1, "target": revision(target),
            "started": false, "queued": queued.map(revision),
            "push": {"dispatched": false, "confirmed_commit": null, "confirmed_at_unix_ms": null},
            "review": {"dispatched": false, "index": null, "confirmed_commit": null, "confirmed_at_unix_ms": null},
            "attempts": 0,
            "attention": attention.map(|(code, unpublished)| json!({"code": code, "message": "m", "recovery_action": "r", "details": {"unpublished_revision": unpublished}})),
            "failure": null, "version": 1
        }))
        .unwrap()
    }

    /// Each exact version gets its own publication state from the one-round-at-a-time intent.
    #[test]
    fn publication_state_is_per_exact_version() {
        let mapping = repo::integration::ReviewRequestRef {
            index: 7,
            platform_commit_sha: "c".repeat(40),
        };
        let state = |intent: Option<&repo::review::Intent>, rev: &str, mapped: bool| {
            publication_of(intent, rev, mapped.then_some(&mapping))["state"].clone()
        };
        // V1 in flight, V2 waiting behind it: V2 is queued, not V1's state.
        let in_flight = intent("pending", "csr-1", Some("csr-2"), None);
        assert_eq!(state(Some(&in_flight), "csr-1", false), "publishing");
        assert_eq!(state(Some(&in_flight), "csr-2", false), "queued");
        // Create-only: V1 published (mapped), V2 refused.
        let refused = intent(
            "published",
            "csr-1",
            None,
            Some(("UPDATE_NOT_ALLOWED", "csr-2")),
        );
        assert_eq!(state(Some(&refused), "csr-1", true), "published");
        assert_eq!(state(Some(&refused), "csr-2", false), "update_not_allowed");
        // An older version the intent moved past is superseded; a mapped one stays published.
        let moved = intent("pending", "csr-3", None, None);
        assert_eq!(state(Some(&moved), "csr-1", false), "superseded");
        assert_eq!(state(Some(&moved), "csr-1", true), "published");
        // Failed / unknown / human gate on the current round.
        assert_eq!(
            state(Some(&intent("failed", "csr-1", None, None)), "csr-1", false),
            "failed"
        );
        assert_eq!(
            state(
                Some(&intent("unknown", "csr-1", None, None)),
                "csr-1",
                false
            ),
            "unknown"
        );
        assert_eq!(
            state(
                Some(&intent("pending_human", "csr-1", None, None)),
                "csr-1",
                false
            ),
            "pending_human"
        );
        // No publish intent at all: every version says so; none is dropped.
        assert_eq!(state(None, "csr-1", false), "not_authorized");
        assert_eq!(state(None, "csr-2", false), "not_authorized");
    }
}
