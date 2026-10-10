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
/// Invocations dispatched for it, the newest admitted version of their ChangeSet, its review
/// publication, the integration of that version, and the Completion Receipt. Read-only; the
/// human view of `task show` turns it into goal, state, blocking, harness, evidence and the
/// next command (CT-PRODUCT: answerable in ten seconds).
fn progress(s: &Store, t: &task::Task) -> Result<Value> {
    let mut invocations = Vec::new();
    let mut write_sets: Vec<(String, String)> = Vec::new();
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
        if let Some(w) = &call.preview.write {
            write_sets.push((
                w.lease.pending.repo_id.clone(),
                w.lease.pending.change_set_id.clone(),
            ));
        }
        let profession = &call.spec.document.profession;
        invocations.push(json!({
            "invocation_id": id, "state": state, "reason": reason,
            "harness": profession.harness.id, "model": profession.model, "write": write,
        }));
    }
    // The newest admitted version across this Task's ChangeSets, and what followed it.
    let mut revision = Value::Null;
    let mut review = Value::Null;
    let mut integration = Value::Null;
    if let Some((repo_id, change_set_id)) = write_sets.last() {
        // Revisions are listed in id order, not time order. The newest one is the one the
        // ChangeSet's publish intent is on (or waiting behind it); without publishing, a
        // single admitted version is the answer and several are not guessed at.
        let intent_id = repo::review::intent_id(s.control_id(), repo_id, change_set_id);
        let intent = repo::review::get(s, repo_id, &intent_id).ok();
        let revisions = repo::changeset::list_revisions(s, change_set_id)?;
        let current = intent
            .as_ref()
            .map(|i| {
                i.queued
                    .as_ref()
                    .unwrap_or(&i.target)
                    .change_set_revision_id
                    .clone()
            })
            .or_else(|| {
                (revisions.len() == 1).then(|| revisions[0].change_set_revision_id.clone())
            });
        if let Some(latest) = current.and_then(|id| {
            revisions
                .into_iter()
                .find(|r| r.change_set_revision_id == id)
        }) {
            if let Some(intent) = &intent {
                let index =
                    repo::integration::review_request(s, repo_id, &latest.change_set_revision_id)?
                        .map(|r| r.index);
                review = json!({"intent_id": intent.intent_id, "state": intent.state,
                    "branch": intent.branch, "target_branch": intent.policy.policy.target_branch,
                    "review_request": index});
            }
            if let Some(merge) = repo::integration::list(s, repo_id)?
                .into_iter()
                .rev()
                .find(|i| i.preview.source.change_set_revision_id == latest.change_set_revision_id)
            {
                integration = json!({"intent_id": merge.intent_id, "state": merge.state,
                    "receipt_id": merge.receipt_id});
            }
            revision = json!({"repo_id": repo_id, "change_set_id": change_set_id,
                "change_set_revision_id": latest.change_set_revision_id,
                "base_commit_sha": latest.base_commit_sha, "result_tree_sha": latest.result_tree_sha});
        }
    }
    let completion = s
        .list(task::COMPLETION_RECEIPT_KIND)?
        .into_iter()
        .filter_map(|r| task::decode::<task::CompletionReceipt>(&r).ok())
        .filter(|c| c.task_id == t.id && c.project_id == t.project_id)
        .max_by_key(|c| c.lifecycle_version)
        .map(|c| json!({"receipt_id": c.receipt_id, "lifecycle_version": c.lifecycle_version}));
    Ok(json!({
        "invocations": invocations, "revision": revision, "review": review,
        "integration": integration, "completion": completion,
    }))
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
