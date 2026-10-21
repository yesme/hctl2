//! Store-backed `Run–Engine Binding` driver enforcing `spec/run.md` and `CT-RUN #8, #9, #11, #18`.

use std::collections::BTreeMap;

use foundation::{bytes_sha256, canonical_json_sha256};
use serde::{Deserialize, Serialize};
use serde_json::json;
use store::{
    ActorSource, Command, EffectIntent, Expected, ObjectKey, Readback, Record, RecordData,
    Reference, Result, Scope, Store, TrustedActor, Version,
};

use crate::adapter::{DaguAdapter, reject};
use crate::model::{
    AuthorizedCheckpointCompletion, BindingState, CheckpointObservation, CompleteResolution,
    ControlledRun, DirectOperationKind, DivergenceRecord, EngineRunReadback,
    HumanTaskCompletionRequest, RunEngineBinding, RunState, StopOutcome,
};

const ENTITY_RUN: &str = "workflow_engine_run";
const ENTITY_BINDING: &str = "run_engine_binding";
const ENTITY_OBSERVATION: &str = "checkpoint_observation";

fn project_scope(project_id: &str) -> Scope {
    Scope::Project(project_id.to_owned())
}

fn check_permission(actor: &TrustedActor, scope: &Scope) -> Result<TrustedActor> {
    if actor.0.permission_scope.contains(scope) {
        Ok(TrustedActor(actor.0.clone()))
    } else if actor.0.source == ActorSource::DirectClient
        && actor.0.permission_scope.contains(&Scope::Control)
    {
        let mut scoped = actor.0.clone();
        scoped.permission_scope.push(scope.clone());
        Ok(TrustedActor(scoped))
    } else {
        Err(reject(
            "PERMISSION_DENIED",
            "scope not authorized",
            "request_authorization",
        ))
    }
}

fn object_key(scope: Scope, kind: &str, id: &str) -> ObjectKey {
    ObjectKey {
        scope,
        kind: kind.to_owned(),
        id: id.to_owned(),
    }
}

fn value_record<T: Serialize>(key: ObjectKey, version: i64, value: &T) -> Result<Record> {
    let val = serde_json::to_value(value).map_err(|e| {
        reject(
            "SERIALIZE_ERROR",
            format!("cannot serialize record: {e}"),
            "inspect_record",
        )
    })?;
    let revision_digest = canonical_json_sha256(&val)?;
    Ok(Record {
        key,
        version,
        revision_digest,
        data: RecordData::Value { value: val },
        sources: Vec::new(),
        materials: Vec::new(),
    })
}

fn decode_record<T: for<'de> Deserialize<'de>>(record: &Record) -> Result<T> {
    let RecordData::Value { value } = &record.data else {
        return Err(reject(
            "DECODE_ERROR",
            format!("record {} is not a Value payload", record.key.id),
            "inspect_record",
        ));
    };
    serde_json::from_value(value.clone()).map_err(|e| {
        reject(
            "DECODE_ERROR",
            format!("cannot decode record {}: {e}", record.key.id),
            "inspect_record",
        )
    })
}

fn module_binding() -> Reference {
    Reference {
        key: object_key(Scope::Control, "module", "run"),
        version: Version::State(1),
    }
}

fn next_command_version(store: &Store, target: &ObjectKey) -> Result<(Expected, i64)> {
    match store.get(target)? {
        None => Ok((Expected::Absent, 1)),
        Some(existing) => Ok((
            Expected::Exact(Version::State(existing.version)),
            existing.version + 1,
        )),
    }
}

/// Parameters for binding and starting a compiled workflow deployment on Dagu.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartRunParams {
    pub command_key: String,
    pub project_id: String,
    pub run_id: String,
    pub binding_id: String,
    pub workflow_id: String,
    pub revision: i64,
    pub port_provider_binding: String,
    pub enqueue: bool,
    pub now_ms: u64,
    pub progress_timeout_ms: u64,
}

/// Parameters for completing a waiting checkpoint through control command + outbox.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompleteCheckpointParams {
    pub command_key: String,
    pub project_id: String,
    pub binding_id: String,
    pub step_id: String,
    pub obligation_id: String,
    pub attempt_generation: i64,
    pub verdict: String,
    pub receipt_id: String,
    pub ack_unknown: bool,
    pub now_ms: u64,
}

/// Parameters for polling Dagu and reconciling a `RunEngineBinding`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PollReconcileParams {
    pub command_key: String,
    pub project_id: String,
    pub binding_id: String,
    pub now_ms: u64,
    pub from_cache: bool,
    pub incoming_event_seq: Option<u64>,
    pub incoming_cursor: Option<u64>,
}

/// Result of a `poll_and_reconcile` tick.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconcileOutcome {
    pub run: ControlledRun,
    pub binding: RunEngineBinding,
    pub observations: Vec<CheckpointObservation>,
    pub readback: Option<EngineRunReadback>,
}

/// Read a `ControlledRun` from `Store`.
///
/// # Errors
///
/// Returns `NOT_FOUND` if the run does not exist.
pub fn get_controlled_run(store: &Store, project_id: &str, run_id: &str) -> Result<ControlledRun> {
    let key = object_key(project_scope(project_id), ENTITY_RUN, run_id);
    let record = store.get(&key)?.ok_or_else(|| {
        reject(
            "NOT_FOUND",
            format!("controlled run {run_id} not found in project {project_id}"),
            "check_run_id",
        )
    })?;
    decode_record(&record)
}

/// Read a `RunEngineBinding` from `Store`.
///
/// # Errors
///
/// Returns `NOT_FOUND` if the binding does not exist.
pub fn get_binding(store: &Store, project_id: &str, binding_id: &str) -> Result<RunEngineBinding> {
    let key = object_key(project_scope(project_id), ENTITY_BINDING, binding_id);
    let record = store.get(&key)?.ok_or_else(|| {
        reject(
            "NOT_FOUND",
            format!("run_engine_binding {binding_id} not found in project {project_id}"),
            "check_binding_id",
        )
    })?;
    decode_record(&record)
}

/// Reject any attempt to advance a `ControlledRun` or synthesize a Verdict/Receipt from an engine
/// observation/callback without a control `Command` + `EffectIntent` (`CT-RUN #8`, 验收 1).
///
/// # Errors
///
/// Always returns `RUN_ADVANCE_REQUIRES_COMMAND` after verifying the `ControlledRun` in `Store` remains unmodified.
pub fn reject_engine_self_advance(
    store: &Store,
    project_id: &str,
    run_id: &str,
    step_id: &str,
) -> Result<()> {
    let _run = get_controlled_run(store, project_id, run_id)?;
    Err(reject(
        "RUN_ADVANCE_REQUIRES_COMMAND",
        format!(
            "engine observation on step {step_id} cannot advance Run {run_id} or synthesize Verdict/Receipt without a control Command and EffectIntent"
        ),
        "submit_control_command",
    ))
}

fn resolve_compiled_deployment(
    store: &Store,
    project_id: &str,
    workflow_id: &str,
    revision: i64,
) -> Result<(Reference, String)> {
    let pointer_key = object_key(
        project_scope(project_id),
        "workflow_deployment",
        &format!("{workflow_id}:{revision}"),
    );
    let pointer = store.get(&pointer_key)?.ok_or_else(|| {
        reject(
            "DEPLOYMENT_NOT_FOUND",
            format!("compiled deployment for {workflow_id}:{revision} not found"),
            "compile_workflow",
        )
    })?;
    let deployment_ref: Reference = decode_record(&pointer)?;
    let deployment_record = store.get(&deployment_ref.key)?.ok_or_else(|| {
        reject(
            "DEPLOYMENT_NOT_FOUND",
            format!("engine_deployment {} not found", deployment_ref.key.id),
            "compile_workflow",
        )
    })?;
    let RecordData::Value {
        value: deployment_val,
    } = &deployment_record.data
    else {
        return Err(reject(
            "INVALID_DEPLOYMENT",
            "engine_deployment is not a Value payload",
            "compile_workflow",
        ));
    };
    let yaml = deployment_val["compiled"]["yaml"]
        .as_str()
        .ok_or_else(|| {
            reject(
                "INVALID_DEPLOYMENT",
                "engine_deployment is missing compiled.yaml",
                "compile_workflow",
            )
        })?
        .to_owned();
    Ok((deployment_ref, yaml))
}

/// Bind a compiled `EngineDeployment` to a `ControlledRun` and start/enqueue it on Dagu via `Store::submit` (`CT-RUN #8`).
///
/// # Errors
///
/// Returns a typed error if the deployment is missing, Store admission fails, or Dagu rejects start/enqueue.
pub async fn bind_and_start_run(
    store: &mut Store,
    actor: &TrustedActor,
    adapter: &DaguAdapter,
    params: &StartRunParams,
) -> Result<(ControlledRun, RunEngineBinding, EngineRunReadback)> {
    let scope = project_scope(&params.project_id);
    let actor = &check_permission(actor, &scope)?;
    let (deployment_ref, yaml_spec) = resolve_compiled_deployment(
        store,
        &params.project_id,
        &params.workflow_id,
        params.revision,
    )?;

    let dag_name = format!(
        "hctl_{}_{}_r{}",
        params.project_id, params.workflow_id, params.revision
    );
    let dag_run_id = format!("run_{}_{}", params.run_id, params.binding_id);
    let correlation_key = format!(
        "{}:{}:{}:{}",
        params.project_id, params.run_id, params.binding_id, dag_run_id
    );
    let expected_spec_digest = bytes_sha256(yaml_spec.trim().as_bytes());

    let run = ControlledRun {
        project_id: params.project_id.clone(),
        run_id: params.run_id.clone(),
        run_version: 1,
        workflow_id: params.workflow_id.clone(),
        workflow_revision: params.revision,
        deployment: deployment_ref.clone(),
        state: RunState::Running,
        needs_attention: false,
        cancel_intent_persisted: false,
        authority_revoked: false,
        completed_checkpoints: Vec::new(),
        verdicts: Vec::new(),
        receipts: Vec::new(),
    };
    let mut binding = RunEngineBinding {
        project_id: params.project_id.clone(),
        binding_id: params.binding_id.clone(),
        run_id: params.run_id.clone(),
        deployment: deployment_ref,
        port_provider_binding: params.port_provider_binding.clone(),
        dag_name: dag_name.clone(),
        dag_run_id: dag_run_id.clone(),
        correlation_key,
        expected_spec_digest,
        engine_binding_generation: 1,
        state: BindingState::Starting,
        last_observation_seq: 0,
        cursor: 0,
        last_progress_at_ms: params.now_ms,
        progress_timeout_ms: params.progress_timeout_ms.max(1),
        observed_step_waiting_counts: BTreeMap::new(),
        authorized_step_retries: BTreeMap::new(),
        authorized_completions: BTreeMap::new(),
        divergence: None,
    };

    let run_key = object_key(scope.clone(), ENTITY_RUN, &params.run_id);
    let binding_key = object_key(scope.clone(), ENTITY_BINDING, &params.binding_id);
    let (expected_binding, binding_ver) = next_command_version(store, &binding_key)?;
    let (_, run_ver) = next_command_version(store, &run_key)?;

    let effect_op = if params.enqueue {
        "dagu.enqueue"
    } else {
        "dagu.start"
    };
    let effect_input = json!({
        "dag_name": dag_name,
        "dag_run_id": dag_run_id,
        "binding_id": params.binding_id,
    });
    let effect_input_digest = Command::digest_input(effect_op, &effect_input)?;
    let intent_id = format!("intent:start:{}:{}", params.binding_id, params.command_key);
    let effect_intent = EffectIntent {
        intent_id: intent_id.clone(),
        owner: Reference {
            key: binding_key.clone(),
            version: Version::State(binding_ver),
        },
        binding: module_binding(),
        operation: effect_op.to_owned(),
        target: format!("{dag_name}/{dag_run_id}"),
        conflict_scope: format!("dagu:{dag_name}:{dag_run_id}"),
        permission_scope: scope.clone(),
        input: effect_input.clone(),
        input_digest: effect_input_digest.clone(),
        idempotency_key: params.command_key.clone(),
    };

    let cmd_input = serde_json::to_value(params).map_err(|e| {
        reject(
            "SERIALIZE_ERROR",
            format!("cannot serialize start command: {e}"),
            "inspect_command",
        )
    })?;
    let cmd = Command {
        command_id: format!("workflow.engine.start:{}", params.command_key),
        idempotency_key: params.command_key.clone(),
        actor: actor.0.clone(),
        target: binding_key.clone(),
        expected: expected_binding,
        binding: module_binding(),
        operation: "workflow.engine.start".to_owned(),
        input_digest: Command::digest_input("workflow.engine.start", &cmd_input)?,
        input: cmd_input,
    };

    store.submit(store.generation(), actor, &cmd, None, |tx| {
        tx.put(&value_record(run_key.clone(), run_ver, &run)?)?;
        tx.put(&value_record(binding_key.clone(), binding_ver, &binding)?)?;
        tx.enqueue_effect(&effect_intent)?;
        tx.confirm_effect(&intent_id, &Readback::Unknown)?;
        Ok(json!({"status": "starting"}))
    })?;

    let readback = if params.enqueue {
        adapter.enqueue(&dag_name, &yaml_spec, &dag_run_id).await?
    } else {
        adapter.start(&dag_name, &yaml_spec, &dag_run_id).await?
    };

    binding.state = BindingState::Bound;
    for step in &readback.steps {
        if step.status == 7 {
            binding.last_observation_seq += 1;
            binding.cursor += 1;
            binding
                .observed_step_waiting_counts
                .insert(step.step_name.clone(), step.done_count.max(1));
        }
    }

    let confirm_key = format!("{}:confirm", params.command_key);
    let confirm_input = json!({"binding_id": params.binding_id, "dag_run_id": dag_run_id});
    let confirm_cmd = Command {
        command_id: format!("workflow.engine.start.confirm:{confirm_key}"),
        idempotency_key: confirm_key,
        actor: actor.0.clone(),
        target: binding_key.clone(),
        expected: Expected::Exact(Version::State(binding_ver)),
        binding: module_binding(),
        operation: "workflow.engine.start.confirm".to_owned(),
        input_digest: Command::digest_input("workflow.engine.start.confirm", &confirm_input)?,
        input: confirm_input,
    };
    let readback_val = serde_json::to_value(&readback).unwrap_or_else(|_| json!({}));
    store.submit(store.generation(), actor, &confirm_cmd, None, |tx| {
        tx.put(&value_record(
            binding_key.clone(),
            binding_ver + 1,
            &binding,
        )?)?;
        tx.confirm_effect(
            &intent_id,
            &Readback::Confirmed {
                binding: module_binding(),
                target: format!("{dag_name}/{dag_run_id}"),
                input_digest: effect_input_digest.clone(),
                result: readback_val.clone(),
            },
        )?;
        Ok(json!({"status": "bound"}))
    })?;

    Ok((run, binding, readback))
}

/// Complete a `Waiting(7)` checkpoint via control command + outbox (`CT-RUN #8`, `spec/run.md` §从节点到结果 #5, 验收 4).
///
/// # Errors
///
/// Returns a typed error if the binding is not `Bound`, Store admission fails, or Dagu completion/readback fails.
pub async fn complete_checkpoint(
    store: &mut Store,
    actor: &TrustedActor,
    adapter: &DaguAdapter,
    params: &CompleteCheckpointParams,
) -> Result<(ControlledRun, RunEngineBinding, CompleteResolution)> {
    let scope = project_scope(&params.project_id);
    let actor = &check_permission(actor, &scope)?;

    let mut binding = get_binding(store, &params.project_id, &params.binding_id)?;
    if binding.state != BindingState::Bound {
        return Err(reject(
            "BINDING_NOT_ACTIVE",
            format!(
                "cannot complete checkpoint on binding {} in state {:?}",
                binding.binding_id, binding.state
            ),
            "reconcile_binding",
        ));
    }
    let mut run = get_controlled_run(store, &params.project_id, &binding.run_id)?;

    let authorized = AuthorizedCheckpointCompletion {
        step_id: params.step_id.clone(),
        obligation_id: params.obligation_id.clone(),
        attempt_generation: params.attempt_generation,
        verdict: params.verdict.clone(),
        receipt_id: params.receipt_id.clone(),
        persisted_at_ms: params.now_ms,
        engine_acknowledged: false,
    };
    binding
        .authorized_completions
        .insert(params.step_id.clone(), authorized);
    if !run.verdicts.contains(&params.verdict) {
        run.verdicts.push(params.verdict.clone());
    }
    if !run.receipts.contains(&params.receipt_id) {
        run.receipts.push(params.receipt_id.clone());
    }

    let run_key = object_key(scope.clone(), ENTITY_RUN, &run.run_id);
    let binding_key = object_key(scope.clone(), ENTITY_BINDING, &binding.binding_id);
    let (_, run_ver) = next_command_version(store, &run_key)?;
    let (expected_binding, binding_ver) = next_command_version(store, &binding_key)?;

    let effect_op = "dagu.complete_human_task";
    let effect_input = json!({
        "dag_name": binding.dag_name,
        "dag_run_id": binding.dag_run_id,
        "step_id": params.step_id,
        "obligation_id": params.obligation_id,
        "attempt_generation": params.attempt_generation,
        "verdict": params.verdict,
        "receipt_id": params.receipt_id,
    });
    let effect_input_digest = Command::digest_input(effect_op, &effect_input)?;
    let intent_id = format!(
        "intent:complete:{}:{}:{}",
        binding.binding_id, params.step_id, params.command_key
    );
    let conflict_scope = format!(
        "dagu:{}:{}:{}",
        binding.dag_name, binding.dag_run_id, params.step_id
    );
    let target_str = format!(
        "{}/{}/{}",
        binding.dag_name, binding.dag_run_id, params.step_id
    );
    let effect_intent = EffectIntent {
        intent_id: intent_id.clone(),
        owner: Reference {
            key: binding_key.clone(),
            version: Version::State(binding_ver),
        },
        binding: module_binding(),
        operation: effect_op.to_owned(),
        target: target_str.clone(),
        conflict_scope,
        permission_scope: scope.clone(),
        input: effect_input,
        input_digest: effect_input_digest.clone(),
        idempotency_key: params.command_key.clone(),
    };

    let cmd_input = serde_json::to_value(params).map_err(|e| {
        reject(
            "SERIALIZE_ERROR",
            format!("cannot serialize complete command: {e}"),
            "inspect_command",
        )
    })?;
    let cmd = Command {
        command_id: format!("workflow.engine.complete:{}", params.command_key),
        idempotency_key: params.command_key.clone(),
        actor: actor.0.clone(),
        target: binding_key.clone(),
        expected: expected_binding,
        binding: module_binding(),
        operation: "workflow.engine.complete".to_owned(),
        input_digest: Command::digest_input("workflow.engine.complete", &cmd_input)?,
        input: cmd_input,
    };

    store.submit(store.generation(), actor, &cmd, None, |tx| {
        tx.put(&value_record(run_key.clone(), run_ver, &run)?)?;
        tx.put(&value_record(binding_key.clone(), binding_ver, &binding)?)?;
        tx.enqueue_effect(&effect_intent)?;
        tx.confirm_effect(&intent_id, &Readback::Unknown)?;
        Ok(json!({"status": "intent_persisted"}))
    })?;

    let req = HumanTaskCompletionRequest {
        dag_name: binding.dag_name.clone(),
        dag_run_id: binding.dag_run_id.clone(),
        step_id: params.step_id.clone(),
        obligation_id: params.obligation_id.clone(),
        attempt_generation: params.attempt_generation,
        verdict: params.verdict.clone(),
        receipt_id: params.receipt_id.clone(),
        response_note: None,
    };

    let resolution = adapter
        .complete_human_task_with_readback(&req, params.ack_unknown)
        .await?;

    if let Some(entry) = binding.authorized_completions.get_mut(&params.step_id) {
        entry.engine_acknowledged = true;
    }
    binding.last_progress_at_ms = params.now_ms;
    if !run.completed_checkpoints.contains(&params.step_id) {
        run.completed_checkpoints.push(params.step_id.clone());
    }
    run.run_version = run_ver + 1;

    let confirm_key = format!("{}:confirm", params.command_key);
    let confirm_input = json!({
        "binding_id": binding.binding_id,
        "step_id": params.step_id,
        "resolution": resolution,
    });
    let confirm_cmd = Command {
        command_id: format!("workflow.engine.complete.confirm:{confirm_key}"),
        idempotency_key: confirm_key,
        actor: actor.0.clone(),
        target: binding_key.clone(),
        expected: Expected::Exact(Version::State(binding_ver)),
        binding: module_binding(),
        operation: "workflow.engine.complete.confirm".to_owned(),
        input_digest: Command::digest_input("workflow.engine.complete.confirm", &confirm_input)?,
        input: confirm_input.clone(),
    };
    store.submit(store.generation(), actor, &confirm_cmd, None, |tx| {
        tx.put(&value_record(run_key.clone(), run_ver + 1, &run)?)?;
        tx.put(&value_record(
            binding_key.clone(),
            binding_ver + 1,
            &binding,
        )?)?;
        tx.confirm_effect(
            &intent_id,
            &Readback::Confirmed {
                binding: module_binding(),
                target: target_str.clone(),
                input_digest: effect_input_digest.clone(),
                result: confirm_input.clone(),
            },
        )?;
        Ok(json!({"status": "completed"}))
    })?;

    Ok((run, binding, resolution))
}

/// Cancel a `ControlledRun` using two-phase intent persistence + authority revocation before stopping Dagu
/// (`spec/run.md` §从节点到结果 第 6 条).
///
/// # Errors
///
/// Returns a typed error if the binding or run does not exist, Store admission fails, or Dagu stop fails.
pub async fn cancel_run(
    store: &mut Store,
    actor: &TrustedActor,
    adapter: &DaguAdapter,
    command_key: &str,
    project_id: &str,
    binding_id: &str,
    now_ms: u64,
) -> Result<(ControlledRun, RunEngineBinding, StopOutcome)> {
    let scope = project_scope(project_id);
    let actor = &check_permission(actor, &scope)?;

    let mut binding = get_binding(store, project_id, binding_id)?;
    let mut run = get_controlled_run(store, project_id, &binding.run_id)?;

    run.state = RunState::Cancelling;
    run.cancel_intent_persisted = true;
    run.authority_revoked = true;

    let run_key = object_key(scope.clone(), ENTITY_RUN, &run.run_id);
    let binding_key = object_key(scope.clone(), ENTITY_BINDING, &binding.binding_id);
    let (_, run_ver) = next_command_version(store, &run_key)?;
    let (expected_binding, binding_ver) = next_command_version(store, &binding_key)?;

    let effect_op = "dagu.stop";
    let effect_input = json!({
        "dag_name": binding.dag_name,
        "dag_run_id": binding.dag_run_id,
        "binding_id": binding.binding_id,
    });
    let effect_input_digest = Command::digest_input(effect_op, &effect_input)?;
    let intent_id = format!("intent:stop:{}:{command_key}", binding.binding_id);
    let target_str = format!("{}/{}", binding.dag_name, binding.dag_run_id);
    let effect_intent = EffectIntent {
        intent_id: intent_id.clone(),
        owner: Reference {
            key: binding_key.clone(),
            version: Version::State(binding_ver),
        },
        binding: module_binding(),
        operation: effect_op.to_owned(),
        target: target_str.clone(),
        conflict_scope: format!("dagu:stop:{}:{}", binding.dag_name, binding.dag_run_id),
        permission_scope: scope.clone(),
        input: effect_input.clone(),
        input_digest: effect_input_digest.clone(),
        idempotency_key: command_key.to_owned(),
    };

    let cmd = Command {
        command_id: format!("workflow.engine.cancel:{command_key}"),
        idempotency_key: command_key.to_owned(),
        actor: actor.0.clone(),
        target: binding_key.clone(),
        expected: expected_binding,
        binding: module_binding(),
        operation: "workflow.engine.cancel".to_owned(),
        input_digest: Command::digest_input("workflow.engine.cancel", &effect_input)?,
        input: effect_input,
    };

    store.submit(store.generation(), actor, &cmd, None, |tx| {
        tx.put(&value_record(run_key.clone(), run_ver, &run)?)?;
        tx.put(&value_record(binding_key.clone(), binding_ver, &binding)?)?;
        tx.enqueue_effect(&effect_intent)?;
        tx.confirm_effect(&intent_id, &Readback::Unknown)?;
        Ok(json!({"status": "cancelling"}))
    })?;

    let stop_outcome = adapter.stop(&binding.dag_name, &binding.dag_run_id).await?;

    run.state = RunState::Cancelled;
    run.run_version = run_ver + 1;
    binding.state = BindingState::Closed;
    binding.last_progress_at_ms = now_ms;

    let confirm_key = format!("{command_key}:confirm");
    let confirm_input = json!({
        "binding_id": binding.binding_id,
        "stop_outcome": stop_outcome,
    });
    let confirm_cmd = Command {
        command_id: format!("workflow.engine.cancel.confirm:{confirm_key}"),
        idempotency_key: confirm_key,
        actor: actor.0.clone(),
        target: binding_key.clone(),
        expected: Expected::Exact(Version::State(binding_ver)),
        binding: module_binding(),
        operation: "workflow.engine.cancel.confirm".to_owned(),
        input_digest: Command::digest_input("workflow.engine.cancel.confirm", &confirm_input)?,
        input: confirm_input.clone(),
    };
    store.submit(store.generation(), actor, &confirm_cmd, None, |tx| {
        tx.put(&value_record(run_key.clone(), run_ver + 1, &run)?)?;
        tx.put(&value_record(
            binding_key.clone(),
            binding_ver + 1,
            &binding,
        )?)?;
        tx.confirm_effect(
            &intent_id,
            &Readback::Confirmed {
                binding: module_binding(),
                target: target_str.clone(),
                input_digest: effect_input_digest.clone(),
                result: confirm_input.clone(),
            },
        )?;
        Ok(json!({"status": "cancelled"}))
    })?;

    Ok((run, binding, stop_outcome))
}

fn detect_direct_operation_divergence(
    run: &ControlledRun,
    binding: &RunEngineBinding,
    readback: &EngineRunReadback,
    now_ms: u64,
) -> Option<DivergenceRecord> {
    if readback.spec_digest.is_none() {
        let renamed_prefix = format!("{}_", binding.dag_name.to_ascii_lowercase());
        let has_renamed = readback
            .existing_dag_files
            .iter()
            .any(|file| file.to_ascii_lowercase().starts_with(&renamed_prefix));
        if has_renamed {
            return Some(DivergenceRecord {
                kind: DirectOperationKind::Rename,
                detail: format!(
                    "DAG {} was renamed directly in Dagu ({:?})",
                    binding.dag_name, readback.existing_dag_files
                ),
                observed_at_ms: now_ms,
            });
        }
        return Some(DivergenceRecord {
            kind: DirectOperationKind::Delete,
            detail: format!("DAG {} was deleted directly in Dagu", binding.dag_name),
            observed_at_ms: now_ms,
        });
    }

    if let Some(digest) = &readback.spec_digest
        && *digest != binding.expected_spec_digest
    {
        return Some(DivergenceRecord {
            kind: DirectOperationKind::Edit,
            detail: format!(
                "DAG {} spec digest changed from {} to {}",
                binding.dag_name, binding.expected_spec_digest, digest
            ),
            observed_at_ms: now_ms,
        });
    }

    for run_id in &readback.all_run_ids {
        if run_id != &binding.dag_run_id {
            if run_id.starts_with("reschedule") {
                return Some(DivergenceRecord {
                    kind: DirectOperationKind::Reschedule,
                    detail: format!(
                        "DAG {} was rescheduled directly in Dagu to run {run_id}",
                        binding.dag_name
                    ),
                    observed_at_ms: now_ms,
                });
            }
            return Some(DivergenceRecord {
                kind: DirectOperationKind::Start,
                detail: format!(
                    "unauthorized DAG run {run_id} was started directly in Dagu on {}",
                    binding.dag_name
                ),
                observed_at_ms: now_ms,
            });
        }
    }

    if readback.status == 3 && !(run.cancel_intent_persisted && run.authority_revoked) {
        return Some(DivergenceRecord {
            kind: DirectOperationKind::Stop,
            detail: format!(
                "DAG run {}/{} was stopped directly in Dagu without control cancel intent and authority revocation",
                binding.dag_name, binding.dag_run_id
            ),
            observed_at_ms: now_ms,
        });
    }

    for step in &readback.steps {
        if step.status == 8 || step.rejected_at.is_some() || step.rejection_reason.is_some() {
            return Some(DivergenceRecord {
                kind: DirectOperationKind::Reject,
                detail: format!(
                    "step {} was rejected directly in Dagu ({:?})",
                    step.step_name, step.rejection_reason
                ),
                observed_at_ms: now_ms,
            });
        }
        let authorized_retry = binding
            .authorized_step_retries
            .get(&step.step_name)
            .copied()
            .unwrap_or(0);
        if step.status == 2
            || step.retry_count > authorized_retry
            || step.done_count > 1 + authorized_retry
        {
            return Some(DivergenceRecord {
                kind: DirectOperationKind::Retry,
                detail: format!(
                    "step {} was retried directly in Dagu (status={}, retry_count={}, done_count={}, authorized={authorized_retry})",
                    step.step_name, step.status, step.retry_count, step.done_count
                ),
                observed_at_ms: now_ms,
            });
        }
        if step.approved_at.is_some() {
            return Some(DivergenceRecord {
                kind: DirectOperationKind::Approve,
                detail: format!(
                    "step {} was approved directly in Dagu at {:?}",
                    step.step_name, step.approved_at
                ),
                observed_at_ms: now_ms,
            });
        }
        if step.status == 4 && !binding.authorized_completions.contains_key(&step.step_name) {
            return Some(DivergenceRecord {
                kind: DirectOperationKind::Approve,
                detail: format!(
                    "step {} succeeded in Dagu without an authorized HCTL checkpoint completion",
                    step.step_name
                ),
                observed_at_ms: now_ms,
            });
        }
    }

    None
}

/// Poll Dagu and reconcile a `RunEngineBinding` (`CT-RUN #9, #11, #18`, 验收 2, 3, 6).
///
/// Direct Dagu operations or progress stalls transition `RunEngineBinding` to `Diverged` and set
/// `run.needs_attention = true`, while **never** back-propagating `ControlledRun.state`,
/// `completed_checkpoints`, `verdicts`, or `receipts`.
///
/// # Errors
///
/// Returns a typed error if the binding or run cannot be loaded or Store persistence fails.
pub async fn poll_and_reconcile(
    store: &mut Store,
    actor: &TrustedActor,
    adapter: &DaguAdapter,
    params: &PollReconcileParams,
) -> Result<ReconcileOutcome> {
    let scope = project_scope(&params.project_id);
    let actor = &check_permission(actor, &scope)?;

    let mut binding = get_binding(store, &params.project_id, &params.binding_id)?;
    let mut run = get_controlled_run(store, &params.project_id, &binding.run_id)?;
    let mut observations = Vec::new();

    let readback_result = adapter
        .readback_run(&binding.dag_name, &binding.dag_run_id)
        .await;

    let readback = match readback_result {
        Ok(rb) => Some(rb),
        Err(err) if err.code == "WORKFLOW_ENGINE_NOT_FOUND" => {
            let kind = if adapter.has_renamed_dag(&binding.dag_name).await {
                DirectOperationKind::Rename
            } else {
                DirectOperationKind::Delete
            };
            binding.state = BindingState::Diverged;
            binding.divergence = Some(DivergenceRecord {
                kind,
                detail: err.message,
                observed_at_ms: params.now_ms,
            });
            run.needs_attention = true;
            None
        }
        Err(err) => return Err(err),
    };

    if let Some(rb) = &readback {
        if let Some(divergence) =
            detect_direct_operation_divergence(&run, &binding, rb, params.now_ms)
        {
            binding.state = BindingState::Diverged;
            binding.divergence = Some(divergence);
            run.needs_attention = true;
        } else {
            let mut made_progress = false;
            for step in &rb.steps {
                if step.status == 7 {
                    binding.last_observation_seq += 1;
                    let obs_seq = binding.last_observation_seq;
                    let late_event = params.incoming_event_seq.is_some_and(|seq| seq < obs_seq);
                    let stale_cursor = params
                        .incoming_cursor
                        .is_some_and(|cur| cur < binding.cursor);
                    let admissible = !params.from_cache && !late_event && !stale_cursor;
                    let rejection_reason = if params.from_cache {
                        Some("cached_status_not_admissible".to_owned())
                    } else if late_event {
                        Some("late_event_seq_regression".to_owned())
                    } else if stale_cursor {
                        Some("stale_cursor_regression".to_owned())
                    } else {
                        None
                    };
                    if admissible {
                        binding.cursor += 1;
                        let prev_count = binding
                            .observed_step_waiting_counts
                            .insert(step.step_name.clone(), step.done_count.max(1));
                        if prev_count.is_none() {
                            made_progress = true;
                        }
                    }
                    let obs = CheckpointObservation {
                        project_id: binding.project_id.clone(),
                        binding_id: binding.binding_id.clone(),
                        run_id: binding.run_id.clone(),
                        dag_name: binding.dag_name.clone(),
                        dag_run_id: binding.dag_run_id.clone(),
                        step_name: step.step_name.clone(),
                        observation_seq: obs_seq,
                        cursor: binding.cursor,
                        status: step.status,
                        status_label: step.status_label.clone(),
                        done_count: step.done_count,
                        retry_count: step.retry_count,
                        human_task_prompt: step.human_task_prompt.clone(),
                        from_cache: params.from_cache,
                        late_event,
                        stale_cursor,
                        admissible_for_obligation: admissible,
                        rejection_reason,
                        observed_at_ms: params.now_ms,
                    };
                    observations.push(obs);
                }
            }
            if made_progress {
                binding.last_progress_at_ms = params.now_ms;
            } else if params.now_ms.saturating_sub(binding.last_progress_at_ms)
                > binding.progress_timeout_ms
            {
                binding.state = BindingState::Diverged;
                binding.divergence = Some(DivergenceRecord {
                    kind: DirectOperationKind::EngineProgressStalled,
                    detail: format!(
                        "engine progress stalled for {}ms (timeout={}ms)",
                        params.now_ms.saturating_sub(binding.last_progress_at_ms),
                        binding.progress_timeout_ms
                    ),
                    observed_at_ms: params.now_ms,
                });
                run.needs_attention = true;
            }
        }
    }

    let run_key = object_key(scope.clone(), ENTITY_RUN, &run.run_id);
    let binding_key = object_key(scope.clone(), ENTITY_BINDING, &binding.binding_id);
    let (_, run_ver) = next_command_version(store, &run_key)?;
    let (expected_binding, binding_ver) = next_command_version(store, &binding_key)?;

    let cmd_input = serde_json::to_value(params).map_err(|e| {
        reject(
            "SERIALIZE_ERROR",
            format!("cannot serialize poll command: {e}"),
            "inspect_command",
        )
    })?;
    let cmd = Command {
        command_id: format!("workflow.engine.poll:{}", params.command_key),
        idempotency_key: params.command_key.clone(),
        actor: actor.0.clone(),
        target: binding_key.clone(),
        expected: expected_binding,
        binding: module_binding(),
        operation: "workflow.engine.poll".to_owned(),
        input_digest: Command::digest_input("workflow.engine.poll", &cmd_input)?,
        input: cmd_input,
    };

    store.submit(store.generation(), actor, &cmd, None, |tx| {
        tx.put(&value_record(run_key.clone(), run_ver, &run)?)?;
        tx.put(&value_record(binding_key.clone(), binding_ver, &binding)?)?;
        for obs in &observations {
            let obs_key = object_key(
                scope.clone(),
                ENTITY_OBSERVATION,
                &format!("{}:{}", binding.binding_id, obs.observation_seq),
            );
            tx.put(&value_record(obs_key, 1, obs)?)?;
        }
        Ok(json!({
            "binding_state": binding.state,
            "run_state": run.state,
            "needs_attention": run.needs_attention,
        }))
    })?;

    Ok(ReconcileOutcome {
        run,
        binding,
        observations,
        readback,
    })
}
