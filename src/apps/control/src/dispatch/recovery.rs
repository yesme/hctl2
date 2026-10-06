//! Original-target cleanup and natively idempotent Matrix projection.
use super::*;
use agency_proto::{Dispatch, DispatchState, ExecutionSpec, Permission, Sealed};
use store::{Command, EffectIntent, EffectState, Expected, Readback};

pub(super) fn record(
    s: &mut Store,
    actor: &TrustedActor,
    record: &Record,
    operation: &str,
    effect: Option<(&EffectIntent, Value)>,
) -> store::Result<()> {
    let input = serde_json::to_value(record)?;
    let command = Command {
        command_id: format!("{operation}:{}", record.key.id),
        idempotency_key: format!("{operation}:{}", record.key.id),
        actor: actor.0.clone(),
        target: record.key.clone(),
        expected: Expected::Absent,
        binding: actor
            .0
            .authority
            .clone()
            .ok_or_else(|| invalid("reducer authority required"))?,
        operation: operation.into(),
        input_digest: Command::digest_input(operation, &input)?,
        input,
    };
    s.submit(s.generation(), actor, &command, None, |tx| {
        tx.put(record)?;
        if let Some((effect, result)) = &effect {
            tx.confirm_effect(
                &effect.intent_id,
                &Readback::Confirmed {
                    binding: effect.binding.clone(),
                    target: effect.target.clone(),
                    input_digest: effect.input_digest.clone(),
                    result: result.clone(),
                },
            )?;
        }
        Ok(json!({"record":record}))
    })?;
    Ok(())
}

pub(super) async fn stop(
    shared: &Shared,
    root: &Path,
    actor: &TrustedActor,
    intent: &Record,
) -> store::Result<()> {
    let (effect, state, generation) = {
        let lock = shared.lock().await;
        let s = lock.as_ref().ok_or_else(|| invalid("store not ready"))?;
        let id = format!("stop:{}", intent.key.id);
        if !s.has_effect(&id)? {
            return Ok(());
        }
        let (effect, state) = s.effect(&id)?;
        (effect, state, s.generation())
    };
    if state == EffectState::Confirmed {
        return Ok(());
    }
    let data: Value = decode(intent)?;
    let spec: Sealed<ExecutionSpec> = serde_json::from_value(data["spec"].clone())?;
    let client = crate::agency::paired_client(root, &spec.document.binding.id)?;
    // A cancelled prepare may never have produced a committed mapping.
    // Lookup the original key, never issue a new prepare on revoked authority.
    let dispatch: Dispatch = client
        .call(
            "lookup",
            &agency_proto::Lookup {
                idempotency_key: spec.document.idempotency_key.clone(),
            },
        )
        .await
        .map_err(participant::port_error)?;
    if dispatch.owner != spec.document.owner
        || dispatch.spec_digest != spec.digest
        || dispatch.binding != spec.document.binding
        || dispatch.bundle_digest != spec.document.bundle.digest
    {
        return Err(invalid("cleanup lookup differs from original dispatch"));
    }
    let _: Value = client
        .call(
            "fence",
            &agency_proto::Fence {
                writer_generation: u64::try_from(generation.0)
                    .map_err(|_| invalid("writer generation"))?,
            },
        )
        .await
        .map_err(participant::port_error)?;
    if state == EffectState::Pending {
        let mut lock = shared.lock().await;
        let s = lock.as_mut().ok_or_else(|| invalid("store not ready"))?;
        s.resume_pending_effect(generation, &effect.intent_id, true)?;
        s.begin_effect(generation, &effect.intent_id)?;
    }
    let observe = crate::agency::signed_ticket(
        root,
        actor,
        &dispatch,
        generation,
        vec![Permission::Observe],
        now_ms() + 30_000,
    )?;
    if state == EffectState::Pending {
        let stop = crate::agency::signed_ticket(
            root,
            actor,
            &dispatch,
            generation,
            vec![Permission::Stop],
            now_ms() + 30_000,
        )?;
        let _: Dispatch = client
            .call("stop", &stop)
            .await
            .map_err(participant::port_error)?;
    }
    // A Prepared dispatch cancelled before activation has no live execution.
    // Otherwise, the request alone is not proof of stopping or isolation.
    let mut after = 0;
    let trace = loop {
        let trace: agency_proto::Trace = client
            .call(
                "observe",
                &agency_proto::Observe {
                    ticket: observe.clone(),
                    after,
                },
            )
            .await
            .map_err(participant::port_error)?;
        if trace.dispatch.owner != dispatch.owner
            || trace.dispatch.reference != dispatch.reference
            || trace.dispatch.spec_digest != dispatch.spec_digest
            || trace.dispatch.binding != dispatch.binding
            || trace.dispatch.bundle_digest != dispatch.bundle_digest
        {
            return Err(invalid("stop readback differs"));
        }
        if trace.dispatch.state == DispatchState::Cancelled
            || trace
                .events
                .iter()
                .any(|e| e.kind == "stopped" || e.kind == "turn_stopped")
        {
            break trace;
        }
        if trace.complete || trace.cursor <= after {
            return Err(reject(
                "READBACK_REQUIRED",
                "stop requested; no stop report yet",
                "read_back_original_intent",
            ));
        }
        after = trace.cursor;
    };
    let mut lock = shared.lock().await;
    let s = lock.as_mut().ok_or_else(|| invalid("store not ready"))?;
    if s.generation() != generation {
        return Err(reject(
            "WRITER_STALE",
            "writer changed",
            "reconcile_control_writer",
        ));
    }
    let receipt = participant::value(
        key(intent.key.scope.clone(), "dispatch_cleanup", &intent.key.id),
        1,
        &json!({"dispatch":dispatch.reference,"stop_report":trace,"isolation_confirmed":false}),
    )?;
    record(
        s,
        actor,
        &receipt,
        "invocation.cleanup",
        Some((
            &effect,
            json!({"stop_report_received":true,"isolation_confirmed":false}),
        )),
    )
}

pub(super) async fn projections(
    shared: &Shared,
    services: &Arc<Supervisor>,
    actor: &TrustedActor,
    root: &Path,
    invocation_id: &str,
) -> store::Result<()> {
    let ids = {
        let lock = shared.lock().await;
        lock.as_ref()
            .ok_or_else(|| invalid("store not ready"))?
            .pending_effects()?
    };
    for id in ids {
        let (effect, generation) = {
            let mut lock = shared.lock().await;
            let s = lock.as_mut().ok_or_else(|| invalid("store not ready"))?;
            let (effect, state) = s.effect(&id)?;
            if effect.operation != "invocation.project"
                || effect.owner.key.id != invocation_id
                || Some(&effect.owner.key) != actor.0.authority.as_ref().map(|a| &a.key)
            {
                continue;
            }
            if state == EffectState::Pending {
                s.resume_pending_effect(s.generation(), &id, true)?;
                s.begin_effect(s.generation(), &id)?;
            }
            (effect, s.generation())
        };
        let room: chat::Room = serde_json::from_value(effect.input["room"].clone())?;
        let body = effect.input["body"]
            .as_str()
            .ok_or_else(|| invalid("projection body missing"))?
            .to_owned();
        let txn = effect.idempotency_key.clone();
        let (services, root) = (Arc::clone(services), root.to_owned());
        let receipt = tokio::task::spawn_blocking(move || {
            crate::chat::project_invocation_result(&services, &root, &room, &txn, &body)
        })
        .await
        .map_err(|_| invalid("projection worker failed"))??;
        let mut lock = shared.lock().await;
        let s = lock.as_mut().ok_or_else(|| invalid("store not ready"))?;
        if s.generation() != generation {
            return Err(reject(
                "WRITER_STALE",
                "writer changed during projection",
                "reconcile_control_writer",
            ));
        }
        let record_data = participant::value(
            key(effect.owner.key.scope.clone(), "invocation_projection", &id),
            1,
            &receipt,
        )?;
        record(
            s,
            actor,
            &record_data,
            "invocation.projected",
            Some((&effect, receipt)),
        )?;
    }
    Ok(())
}
