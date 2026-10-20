//! Trusted control adapter seam. Only committed effects cross this seam; raw
//! engine progress is deliberately not a lifecycle/complete command.
use super::{admission::*, freeze::*, *};
use chat::{decode, invalid, key, value_record};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use store::{
    Actor, ActorSource, Command, EffectIntent, EffectState, Expected, Readback, Result, Store,
    TrustedActor, Version,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Observation {
    Unknown {},
    /// A fresh read of THIS correlation proves the original action did not apply.
    NotApplied {},
    Applied {
        execution_id: String,
        engine_generation: u64,
    },
    Rejected {
        reason: String,
    },
}
pub trait Port {
    fn readback(&self, effect: &EffectIntent) -> Result<Observation>;
    fn execute(&self, effect: &EffectIntent) -> Result<Observation>;
}
pub fn actor_for(run: &Run) -> TrustedActor {
    TrustedActor(Actor {
        principal: format!("run-reducer:{}", run.id),
        source: ActorSource::InternalReducer,
        permission_scope: vec![scope(&run.project_id)],
        authority: Some(run.manifest.clone()),
    })
}
/// Native outbox recovery: Unknown is read first; never turn it into a blind retry.
/// Package 2 supplies the real Port. Tests supply a stateful fake, no Dagu service.
pub fn drive(
    store: &mut Store,
    actor: &TrustedActor,
    id: &str,
    port: &dyn Port,
    now_ms: u64,
) -> Result<Value> {
    let (effect, state) = store.effect(id)?;
    let project = effect.input["project_id"]
        .as_str()
        .ok_or_else(|| invalid("Run effect has no Project"))?;
    let (_, run) = load(store, project, &effect.target)?;
    reducer(actor, &run)?;
    let (_, m) = manifest(store, actor, project, &run.id)?;
    if effect.binding != m.engine_binding
        || effect.owner.key != run_key(project, &run.id)
        || effect.permission_scope != scope(project)
        || effect.input["manifest"] != serde_json::to_value(&run.manifest)?
        || !matches!(
            effect.operation.as_str(),
            "run.engine.start" | "run.engine.pause" | "run.engine.resume" | "run.engine.stop"
        )
    {
        return Err(invalid(
            "effect does not belong to the frozen Run engine boundary",
        ));
    }
    if matches!(
        state,
        EffectState::Confirmed | EffectState::Rejected | EffectState::Cancelled
    ) {
        return Ok(json!({"effect_id":id,"state":state}));
    }
    let epoch = effect.input["epoch"]
        .as_u64()
        .ok_or_else(|| invalid("Run effect needs its operation epoch"))?;
    let authorized = epoch == run.operation_epoch
        && (run.authorization_valid || effect.operation == "run.engine.stop");
    let observed = if state == EffectState::Pending {
        store.resume_pending_effect(store.generation(), id, authorized)?;
        let effect = store.begin_effect(store.generation(), id)?;
        port.execute(&effect)?
    } else {
        match port.readback(&effect)? {
            Observation::NotApplied {} if authorized => port.execute(&effect)?,
            Observation::NotApplied {} => Observation::Rejected {
                reason: "original authorization withdrawn before action applied".into(),
            },
            value => value,
        }
    };
    record_readback(store, actor, &effect, observed, now_ms)
}
pub fn record_readback(
    store: &mut Store,
    actor: &TrustedActor,
    effect: &EffectIntent,
    observed: Observation,
    now_ms: u64,
) -> Result<Value> {
    let project = effect.input["project_id"]
        .as_str()
        .ok_or_else(|| invalid("Project required"))?;
    let (record, mut run) = load(store, project, &effect.target)?;
    let before = serde_json::to_value(&run)?;
    let scoped = reducer(actor, &run)?;
    let (_, m) = manifest(store, &scoped, project, &run.id)?;
    let (original, state) = store.effect(&effect.intent_id)?;
    if serde_json::to_value(original)? != serde_json::to_value(effect)?
        || !matches!(
            state,
            EffectState::Unknown | EffectState::Confirmed | EffectState::Rejected
        )
        || effect.binding != m.engine_binding
        || effect.owner.key != run_key(project, &run.id)
        || effect.permission_scope != scope(project)
        || !matches!(
            effect.operation.as_str(),
            "run.engine.start" | "run.engine.pause" | "run.engine.resume" | "run.engine.stop"
        )
        || effect.input["manifest"] != serde_json::to_value(&run.manifest)?
    {
        return Err(invalid("original attempted Run effect required"));
    }
    let result = json!({"effect_id":effect.intent_id,"observation":observed});
    let readback = match &observed {
        Observation::Unknown {} | Observation::NotApplied {} => Readback::Unknown,
        Observation::Applied {
            execution_id,
            engine_generation,
        } => {
            if execution_id.is_empty() || *engine_generation == 0 {
                return Err(invalid(
                    "exact engine execution and independently sourced binding generation required",
                ));
            }
            Readback::Confirmed {
                binding: effect.binding.clone(),
                target: effect.target.clone(),
                input_digest: effect.input_digest.clone(),
                result: result.clone(),
            }
        }
        Observation::Rejected { .. } => Readback::Rejected {
            binding: effect.binding.clone(),
            target: effect.target.clone(),
            input_digest: effect.input_digest.clone(),
            result: result.clone(),
        },
    };
    let epoch = effect.input["epoch"]
        .as_u64()
        .ok_or_else(|| invalid("Run operation epoch required"))?;
    let applicable = epoch == run.operation_epoch && !run.lifecycle.terminal();
    let divergent = epoch != run.operation_epoch
        || (effect.operation != "run.engine.stop"
            && (!run.authorization_valid || run.lifecycle.terminal()));
    let mut plan = Plan {
        input: Input {
            key: effect.intent_id.clone(),
            action: Action::Cancel {
                project_id: project.into(),
                run_id: run.id.clone(),
                version: record.version,
            },
        },
        checks: vec![],
        records: vec![],
        effects: vec![],
        cancel_effects: vec![],
        result: result.clone(),
    };
    let mut next = None;
    if applicable {
        match (&observed, effect.operation.as_str(), run.lifecycle) {
            (Observation::Applied { .. }, "run.engine.start", State::Starting)
                if run.authorization_valid =>
            {
                next = Some(State::Running)
            }
            (Observation::Applied { .. }, "run.engine.pause", State::Pausing)
                if run.authorization_valid =>
            {
                next = Some(State::Paused)
            }
            (Observation::Applied { .. }, "run.engine.stop", _)
                if run.terminal_requested.is_some() =>
            {
                if revoke(store, project, &run.id, &mut plan)? {
                    next = run.terminal_requested;
                } else {
                    run.attention = Some(
                        "runtime stopped; write containment or external outcome still unproved"
                            .into(),
                    );
                }
            }
            (Observation::Rejected { reason }, _, _) => {
                run.attention = Some(format!("engine rejected action: {reason}"))
            }
            _ => {}
        }
    }
    if let Some(next) = next {
        if !run.lifecycle.allows(next) {
            return Err(error(
                "INVALID_TRANSITION",
                "engine receipt cannot cause this transition",
            ));
        }
        run.lifecycle = next;
        run.entered_at_ms = now_ms;
        run.attention = None;
        if next.terminal() {
            run.authorization_valid = false;
            release_slot(store, project, &run, &mut plan, None)?;
        }
    }
    let command = Command {
        command_id: format!(
            "run-readback:{}:{}:{}",
            effect.intent_id,
            record.version,
            foundation::canonical_json_sha256(&result)?
        ),
        idempotency_key: format!(
            "run-readback:{}:{}:{}",
            effect.intent_id,
            record.version,
            foundation::canonical_json_sha256(&result)?
        ),
        actor: scoped.0.clone(),
        target: record.key.clone(),
        expected: Expected::Exact(Version::State(record.version)),
        binding: run.manifest.clone(),
        operation: "run.engine_readback".into(),
        input_digest: Command::digest_input("run.engine_readback", &result)?,
        input: result.clone(),
    };
    store.submit(store.generation(),&scoped,&command,None,|tx| {
        for check in &plan.checks {if tx.get(&check.key)?.as_ref().map(|r|r.version)!=check.version {return Err(chat::stale());}}
        tx.confirm_effect(&effect.intent_id,&readback)?;
        for record in &plan.records {tx.put(record)?;}
        if let Observation::Applied{execution_id,engine_generation}=&observed {
            let k=key(scope(project),"run_engine_binding",&run.id);
            let old=tx.get(&k)?;
            tx.put(&value_record(k,old.map_or(1,|r|r.version+1),&json!({"manifest":run.manifest,"deployment":m.deployment,"binding":m.engine_binding,"execution_id":execution_id,"engine_binding_generation":engine_generation,"divergent":divergent,"state":if divergent {"divergent"}else if effect.operation=="run.engine.stop" {"closed"}else{"bound"}}))?)?;
        }
        if serde_json::to_value(&run)?!=before {tx.put(&value_record(record.key.clone(),record.version+1,&run)?)?;}
        Ok(result.clone())
    })
}
/// Wall clock marks attention only. It cannot cancel, replace, release occupancy,
/// retry effects or manufacture any engine observation.
pub fn mark_timeouts(store: &mut Store, now_ms: u64) -> Result<usize> {
    let mut marked = 0;
    for record in store.list("run")? {
        let mut run: Run = decode(&record)?;
        if !run.timed_out(now_ms)
            || run.attention.as_deref()
                == Some("transition timeout: confirm cancellation or replacement")
        {
            continue;
        }
        let actor = actor_for(&run);
        let input = json!({"run_id":run.id,"epoch":run.operation_epoch,"state":run.lifecycle});
        let command = Command {
            command_id: format!("run-timeout:{}:{}", run.id, run.operation_epoch),
            idempotency_key: format!(
                "run-timeout:{}:{}:{}",
                run.project_id, run.id, run.operation_epoch
            ),
            actor: actor.0.clone(),
            target: record.key.clone(),
            expected: Expected::Exact(Version::State(record.version)),
            binding: run.manifest.clone(),
            operation: "run.timeout".into(),
            input_digest: Command::digest_input("run.timeout", &input)?,
            input,
        };
        run.attention = Some("transition timeout: confirm cancellation or replacement".into());
        store.submit(store.generation(), &actor, &command, None, |tx| {
            tx.put(&value_record(record.key.clone(), record.version + 1, &run)?)?;
            Ok(json!({"attention":true}))
        })?;
        marked += 1;
    }
    Ok(marked)
}

/// Re-evaluate a previously requested stop when package 4a admits a new physical
/// isolation proof. The engine stop receipt alone never proves Git containment.
pub fn settle_revocation(
    store: &mut Store,
    actor: &TrustedActor,
    project: &str,
    id: &str,
    now_ms: u64,
) -> Result<bool> {
    let (record, mut run) = load(store, project, id)?;
    let scoped = reducer(actor, &run)?;
    let Some(next) = run.terminal_requested else {
        return Ok(false);
    };
    if run.lifecycle.terminal() {
        return Ok(false);
    }
    let (_, m) = manifest(store, &scoped, project, id)?;
    let stop = effect(&m, &record, "stop", run.operation_epoch)?;
    if !store.has_effect(&stop.intent_id)?
        || store.effect(&stop.intent_id)?.1 != EffectState::Confirmed
    {
        return Ok(false);
    }
    let mut plan = Plan {
        input: Input {
            key: format!("settle-{id}"),
            action: Action::Cancel {
                project_id: project.into(),
                run_id: id.into(),
                version: record.version,
            },
        },
        checks: vec![],
        records: vec![],
        effects: vec![],
        cancel_effects: vec![],
        result: Value::Null,
    };
    if !revoke(store, project, id, &mut plan)? {
        return Ok(false);
    }
    if !run.lifecycle.allows(next) {
        return Err(error(
            "INVALID_TRANSITION",
            "requested terminal transition is illegal",
        ));
    }
    run.lifecycle = next;
    run.authorization_valid = false;
    run.attention = None;
    run.entered_at_ms = now_ms;
    release_slot(store, project, &run, &mut plan, None)?;
    let input = json!({"run_id":id,"outcome":next,"version":record.version});
    let command = Command {
        command_id: format!("run-settle:{id}:{}", record.version),
        idempotency_key: format!("run-settle:{project}:{id}:{}", record.version),
        actor: scoped.0.clone(),
        target: record.key.clone(),
        expected: Expected::Exact(Version::State(record.version)),
        binding: run.manifest.clone(),
        operation: "run.settle".into(),
        input_digest: Command::digest_input("run.settle", &input)?,
        input,
    };
    store.submit(store.generation(), &scoped, &command, None, |tx| {
        for check in &plan.checks {
            if tx.get(&check.key)?.as_ref().map(|r| r.version) != check.version {
                return Err(chat::stale());
            }
        }
        tx.put(&value_record(record.key.clone(), record.version + 1, &run)?)?;
        for record in &plan.records {
            tx.put(record)?;
        }
        Ok(json!({"settled":true}))
    })?;
    Ok(true)
}
