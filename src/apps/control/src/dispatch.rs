//! Room Invocation orchestration. Provider I/O is never inside a Store transaction.
mod context;
mod recovery;
use crate::services::Supervisor;
use participant::{decode, invalid, key, reference, reject};
use project::invocation::{self, State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{path::Path, sync::Arc};
use store::{ActorSource, Record, Scope, Store, TrustedActor};
use tokio::sync::Mutex;
type Shared = Arc<Mutex<Option<Store>>>;

fn access<T>(shared: &Shared, f: impl FnOnce(&mut Store) -> store::Result<T>) -> store::Result<T> {
    let mut slot = shared.blocking_lock();
    f(slot.as_mut().ok_or_else(|| invalid("store not ready"))?)
}
pub(super) fn now_ms() -> u64 {
    u64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
    )
    .unwrap_or(u64::MAX)
}
fn reducer(record: &Record) -> TrustedActor {
    TrustedActor(store::Actor {
        principal: format!("invocation-reducer:{}", record.key.id),
        source: ActorSource::InternalReducer,
        permission_scope: vec![Scope::Control, record.key.scope.clone()],
        authority: Some(store::Reference {
            key: record.key.clone(),
            version: store::Version::State(1),
        }),
    })
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    preview: invocation::Preview,
    assembly: ::context::Assembly,
}

pub(crate) fn preview(
    shared: &Shared,
    services: &Supervisor,
    root: &Path,
    actor: &TrustedActor,
    operation: &str,
    payload: &Value,
) -> store::Result<Value> {
    if operation != "invocation.start" {
        return Err(invalid("only invocation.start is exposed"));
    }
    let input: invocation::Input = serde_json::from_value(payload.clone())?;
    if let Some(plan) = access(shared, |s| frozen_plan(s, actor, &input))? {
        return Ok(serde_json::to_value(plan)?);
    }
    let preview = access(shared, |s| invocation::prepare(s, actor, input, now_ms()))?;
    let assembly = context::assemble(shared, services, root, actor, &preview)?;
    Ok(serde_json::to_value(Plan { preview, assembly })?)
}

pub(crate) fn submit(
    shared: &Shared,
    services: &Supervisor,
    root: &Path,
    actor: &TrustedActor,
    request: &proto::SubmitRequest,
    details: &Value,
) -> store::Result<Value> {
    let input: invocation::Input = serde_json::from_slice(&request.payload)?;
    if request.operation != "invocation.start"
        || request.idempotency_key != input.key
        || request.command_id != format!("invocation:{}", input.key)
    {
        return Err(invalid("Invocation envelope differs"));
    }
    let plan: Plan = serde_json::from_value(details.clone())?;
    if input != plan.preview.input {
        return Err(invalid("Invocation preview input differs"));
    }
    // A new authorization still requires a live, unencrypted Room. Replaying
    // an admitted command needs no new provider read or authorization.
    if access(shared, |s| frozen_plan(s, actor, &input))?.is_none() {
        let room = access(shared, |s| {
            Ok(chat::room(s, &input.project_id, &input.room_id)?.1)
        })?;
        crate::chat::invocation_window(services, root, &room)?;
    }
    access(shared, |s| {
        let actor = chat::owner(actor, &input.project_id)?;
        if frozen_plan(s, &actor, &input)?.is_some() {
            return invocation::start(s, &actor, &plan.preview, &plan.assembly, now_ms());
        }
        // Revalidate before saving Context, and again inside start's admission.
        let current = invocation::prepare(s, &actor, input.clone(), now_ms())?;
        if serde_json::to_value(current)? != serde_json::to_value(&plan.preview)? {
            return Err(reject(
                "VERSION_CONFLICT",
                "Invocation preview changed",
                "rebuild_preview",
            ));
        }
        context::verify_sources(s, &actor, &plan)?;
        ::context::save_assembly(
            s,
            &actor,
            &input.project_id,
            &format!("context:{}", plan.preview.consumer.id),
            &plan.assembly,
        )
        .map_err(participant::port_error)?;
        invocation::start(s, &actor, &plan.preview, &plan.assembly, now_ms())
    })
}

/// A response lost after admission retries the original frozen input, not a
/// new timeline window. Store still verifies the original command fingerprint.
fn frozen_plan(
    s: &Store,
    actor: &TrustedActor,
    input: &invocation::Input,
) -> store::Result<Option<Plan>> {
    let scoped = chat::owner(actor, &input.project_id)?;
    let id = invocation::invocation_id(s, &input.project_id, &input.key);
    if s.get(&invocation::owner_key(&input.project_id, &id))?
        .is_none()
    {
        return Ok(None);
    }
    let (_, call) = invocation::invocation(s, &input.project_id, &id)?;
    if call.preview.input != *input {
        return Err(reject(
            "IDEMPOTENCY_CONFLICT",
            "same invocation key with different input",
            "use_new_command_key",
        ));
    }
    let manifest = ::context::read_manifest(
        s,
        &scoped,
        &input.project_id,
        &call.spec.document.manifest.id,
    )
    .map_err(participant::port_error)?
    .ok_or_else(|| invalid("frozen Manifest missing"))?;
    let bundle =
        ::context::read_bundle(s, &scoped, &input.project_id, &call.spec.document.bundle.id)
            .map_err(participant::port_error)?
            .ok_or_else(|| invalid("frozen Bundle missing"))?;
    Ok(Some(Plan {
        preview: call.preview,
        assembly: ::context::Assembly { manifest, bundle },
    }))
}

pub(crate) fn show(shared: &Shared, actor: &TrustedActor, payload: &Value) -> store::Result<Value> {
    let p = payload["project_id"]
        .as_str()
        .ok_or_else(|| invalid("project_id required"))?;
    let id = payload["invocation_id"]
        .as_str()
        .ok_or_else(|| invalid("invocation_id required"))?;
    let _ = chat::owner(actor, p)?;
    access(shared, |s| {
        let (root, invocation) = invocation::invocation(s, p, id)?;
        let (state_record, state) = invocation::lifecycle(s, p, id)?;
        let intent = s.get(&key(root.key.scope.clone(), "dispatch_intent", id))?;
        let results: Vec<_> = s
            .list("invocation_result")?
            .into_iter()
            .filter(|r| {
                r.key.scope == root.key.scope
                    && r.sources.contains(&store::Reference {
                        key: root.key.clone(),
                        version: store::Version::State(1),
                    })
            })
            .collect();
        let actor = chat::owner(actor, p)?;
        let results = results.iter().map(|r| {
            let material = r.materials.first().ok_or_else(|| invalid("answer material missing"))?;
            Ok(json!({"record":r,"output":String::from_utf8(s.read_material(&actor, material)?).map_err(|_| invalid("answer is not UTF-8"))?}))
        }).collect::<store::Result<Vec<_>>>()?;
        let mut pending_effects = Vec::new();
        for id in s.pending_effects()? {
            let (effect, state) = s.effect(&id)?;
            if effect.owner.key == root.key {
                pending_effects.push(
                    json!({"id":effect.intent_id,"operation":effect.operation,"state":state}),
                );
            }
        }
        Ok(
            json!({"invocation":invocation,"owner":reference(&root),"state":state.state,"reason":state.reason,"state_version":state_record.version,"dispatch_intent":intent,"results":results,"pending_effects":pending_effects}),
        )
    })
}

/// One periodic worker owns business delivery. Query does not drive side effects.
pub(crate) async fn reconcile(shared: Shared, root: std::path::PathBuf, services: Arc<Supervisor>) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut previous_error = None;
    loop {
        interval.tick().await;
        let error = reconcile_once(&shared, &root, &services).await.err();
        let code = error.as_ref().map(|e| e.code);
        if code != previous_error {
            if let Some(error) = error {
                eprintln!(
                    "Invocation delivery: {} ({})",
                    error.code, error.recovery_action
                );
            }
            previous_error = code;
        }
    }
}

pub async fn reconcile_once(
    shared: &Shared,
    root: &Path,
    services: &Arc<Supervisor>,
) -> store::Result<()> {
    let roots = {
        let lock = shared.lock().await;
        let Some(store) = lock.as_ref() else {
            return Ok(());
        };
        store.list("room_invocation")?
    };
    let mut first_error = None;
    for owner in roots {
        if let Err(e) = drive(shared, root, services, &owner).await {
            first_error.get_or_insert(e);
        }
    }
    first_error.map_or(Ok(()), Err)
}

async fn drive(
    shared: &Shared,
    root: &Path,
    services: &Arc<Supervisor>,
    owner: &Record,
) -> store::Result<()> {
    let Scope::Project(p) = &owner.key.scope else {
        return Err(invalid("Project owner required"));
    };
    let id = &owner.key.id;
    let actor = reducer(owner);
    let (call, state, mut intent) = {
        let lock = shared.lock().await;
        let s = lock.as_ref().ok_or_else(|| invalid("store not ready"))?;
        (
            invocation::invocation(s, p, id)?.1,
            invocation::lifecycle(s, p, id)?.1,
            s.get(&key(owner.key.scope.clone(), "dispatch_intent", id))?
                .ok_or_else(|| invalid("dispatch intent missing"))?,
        )
    };
    if state.state == State::Completed {
        // Projection recovery does not depend on a live Agency. Later results
        // are still preserved for audit, never admitted into a completed call.
        recovery::projections(shared, services, &actor, root, id).await?;
        let data: Value = decode(&intent)?;
        if let Some(dispatch_id) = data["dispatch"].as_str() {
            let dispatch = {
                let lock = shared.lock().await;
                lock.as_ref()
                    .ok_or_else(|| invalid("store not ready"))?
                    .get(&key(owner.key.scope.clone(), "dispatch", dispatch_id))?
                    .ok_or_else(|| invalid("dispatch mapping missing"))?
            };
            preserve(shared, root, &actor, owner, &dispatch).await?;
        }
        return Ok(());
    }
    if !state.state.terminal() {
        let mut lock = shared.lock().await;
        let s = lock.as_mut().ok_or_else(|| invalid("store not ready"))?;
        let mut rejected = false;
        for effect in [format!("prepare:{id}"), format!("activate:{id}")] {
            if s.has_effect(&effect)? && s.effect(&effect)?.1 == store::EffectState::Rejected {
                rejected = true;
            }
        }
        if rejected {
            let (record, _) = invocation::lifecycle(s, p, id)?;
            invocation::end(
                s,
                &actor,
                invocation::End {
                    key: format!("rejected:{id}"),
                    project_id: p.clone(),
                    invocation_id: id.clone(),
                    state_version: record.version,
                    outcome: State::Failed,
                    reason: "Agency rejected the original operation before its effect".into(),
                },
            )?;
        }
    }
    if !state.state.terminal() && call.spec.document.deadline_ms <= now_ms() {
        let mut lock = shared.lock().await;
        let s = lock.as_mut().ok_or_else(|| invalid("store not ready"))?;
        let (record, state) = invocation::lifecycle(s, p, id)?;
        if !state.state.terminal() {
            invocation::end(
                s,
                &actor,
                invocation::End {
                    key: format!("deadline:{id}"),
                    project_id: p.clone(),
                    invocation_id: id.clone(),
                    state_version: record.version,
                    outcome: State::Failed,
                    reason: "deadline expired".into(),
                },
            )?;
        }
    }
    let authorized = {
        let lock = shared.lock().await;
        invocation::current_authorization(
            lock.as_ref().ok_or_else(|| invalid("store not ready"))?,
            &reference(owner),
            now_ms(),
        )?
    };
    let mut data: Value = decode(&intent)?;
    if data["dispatch"].is_null() && authorized {
        crate::agency::deliver_prepare(shared, root, &actor, &intent).await?;
        let lock = shared.lock().await;
        intent = lock
            .as_ref()
            .ok_or_else(|| invalid("store not ready"))?
            .get(&intent.key)?
            .ok_or_else(|| invalid("mapped intent missing"))?;
        data = decode(&intent)?;
    }
    let stop_error = if authorized {
        None
    } else {
        recovery::stop(shared, root, &actor, &intent).await.err()
    };
    if let Some(dispatch_id) = data["dispatch"].as_str() {
        let dispatch = {
            let lock = shared.lock().await;
            lock.as_ref()
                .ok_or_else(|| invalid("store not ready"))?
                .get(&key(owner.key.scope.clone(), "dispatch", dispatch_id))?
                .ok_or_else(|| invalid("dispatch mapping missing"))?
        };
        if authorized && state.state == State::Pending {
            crate::agency::deliver_activation(shared, root, &actor, id, &dispatch).await?;
            let mut lock = shared.lock().await;
            invocation::record_started(
                lock.as_mut().ok_or_else(|| invalid("store not ready"))?,
                &actor,
                p,
                id,
                now_ms(),
            )?;
        }
        let trace = match crate::agency::observe_dispatch(shared, root, &actor, &dispatch, 0).await
        {
            Ok(trace) => trace,
            Err(error) if error.code == "DISPATCH_NOT_FOUND" && authorized => {
                let mut lock = shared.lock().await;
                let s = lock.as_mut().ok_or_else(|| invalid("store not ready"))?;
                let (record, current) = invocation::lifecycle(s, p, id)?;
                if !current.state.terminal() {
                    invocation::end(
                        s,
                        &actor,
                        invocation::End {
                            key: format!("lost:{id}"),
                            project_id: p.clone(),
                            invocation_id: id.clone(),
                            state_version: record.version,
                            outcome: State::Lost,
                            reason: "Agency cannot identify the committed dispatch".into(),
                        },
                    )?;
                }
                return Err(error);
            }
            Err(error) => return Err(error),
        };
        preserve(shared, root, &actor, owner, &dispatch).await?;
        if trace.dispatch.state == agency_proto::DispatchState::ResultReturned && authorized {
            let mut lock = shared.lock().await;
            let s = lock.as_mut().ok_or_else(|| invalid("store not ready"))?;
            for inbox in s.list("proposal_inbox")?.into_iter().filter(|r| {
                r.key.scope == owner.key.scope && r.sources.contains(&reference(&dispatch))
            }) {
                match invocation::admit_result(s, &actor, p, id, &inbox, now_ms()) {
                    Ok(_) => {}
                    Err(e) if matches!(e.code, "OWNER_STALE" | "PROPOSAL_MISMATCH") => {
                        first_result_error(s, &actor, &inbox, &e)?;
                    }
                    Err(e) => return Err(e),
                }
            }
        } else if trace.dispatch.state == agency_proto::DispatchState::CannotFulfill && authorized {
            let mut lock = shared.lock().await;
            let s = lock.as_mut().ok_or_else(|| invalid("store not ready"))?;
            let (record, current) = invocation::lifecycle(s, p, id)?;
            if !current.state.terminal() {
                invocation::end(
                    s,
                    &actor,
                    invocation::End {
                        key: format!("cannot-fulfill:{id}"),
                        project_id: p.clone(),
                        invocation_id: id.clone(),
                        state_version: record.version,
                        outcome: State::Failed,
                        reason: "Agency cannot fulfill this dispatch".into(),
                    },
                )?;
            }
        }
    }
    recovery::projections(shared, services, &actor, root, id).await?;
    stop_error.map_or(Ok(()), Err)
}

async fn preserve(
    shared: &Shared,
    root: &Path,
    actor: &TrustedActor,
    owner: &Record,
    dispatch: &Record,
) -> store::Result<()> {
    let report = crate::agency::preserve_results_report(shared, root, actor, dispatch).await?;
    if report.refused.is_empty() {
        return Ok(());
    }
    let mut lock = shared.lock().await;
    let s = lock.as_mut().ok_or_else(|| invalid("store not ready"))?;
    for (proposal_id, error) in report.refused {
        let refusal = participant::value(
            key(
                owner.key.scope.clone(),
                "proposal_preservation_refusal",
                &format!("{}:{proposal_id}:{}", owner.key.id, error.code),
            ),
            1,
            &json!({"proposal_id":proposal_id,"dispatch":reference(dispatch),"code":error.code,"recovery_action":error.recovery_action}),
        )?;
        recovery::record(s, actor, &refusal, "invocation.preservation_refusal", None)?;
    }
    Ok(())
}

fn first_result_error(
    s: &mut Store,
    actor: &TrustedActor,
    inbox: &Record,
    error: &store::StoreError,
) -> store::Result<()> {
    let record = participant::value(
        key(
            inbox.key.scope.clone(),
            "proposal_admission_refusal",
            &inbox.key.id,
        ),
        1,
        &json!({"proposal":reference(inbox),"code":error.code,"recovery_action":error.recovery_action}),
    )?;
    recovery::record(s, actor, &record, "invocation.result_refusal", None)
}
