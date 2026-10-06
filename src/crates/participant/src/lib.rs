//! Control-side binding, frozen roster acceptance and dispatch records. No runtime dependency.
#![forbid(unsafe_code)]
pub mod dispatch;
pub mod profiles;
pub mod selection;
use agency_proto::{
    Catalog, Dispatch, ExecutionSpec, FrozenRef, PortError, Profession, Proposal, Sealed, Trace,
    hash,
};
use foundation::canonical_json_sha256;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use store::{
    Command, Expected, InboxEntry, ObjectKey, Record, RecordData, Reference, Scope, Store,
    StoreError, TrustedActor, Version,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub id: String,
    pub protocol: String,
    pub catalog: Catalog,
}
pub fn reject(
    code: &'static str,
    message: impl Into<String>,
    recovery: &'static str,
) -> StoreError {
    StoreError {
        code,
        message: message.into(),
        recovery_action: recovery,
    }
}
pub fn invalid(message: impl Into<String>) -> StoreError {
    reject("INVALID_INPUT", message, "correct_input")
}
pub fn key(scope: Scope, kind: &str, id: &str) -> ObjectKey {
    ObjectKey {
        scope,
        kind: kind.into(),
        id: id.into(),
    }
}
pub fn reference(r: &Record) -> Reference {
    Reference {
        key: r.key.clone(),
        version: Version::State(r.version),
    }
}
pub fn frozen(r: &Record) -> FrozenRef {
    FrozenRef {
        id: r.key.id.clone(),
        revision: r.version.to_string(),
        digest: r.revision_digest.clone(),
    }
}
pub fn port_error(e: PortError) -> StoreError {
    if e.code == "INVALID_INPUT" {
        return invalid(e.message);
    }
    let code = match e.code.as_str() {
        "AGENCY_UNREACHABLE" => "AGENCY_UNREACHABLE",
        "AGENCY_RESPONSE_UNKNOWN" => "AGENCY_RESPONSE_UNKNOWN",
        "CAPABILITY_MISSING" => "CAPABILITY_MISSING",
        "DEADLINE_EXPIRED" => "DEADLINE_EXPIRED",
        "TENANT_EXISTS" => "TENANT_EXISTS",
        "UNSAFE_ENDPOINT" => "UNSAFE_ENDPOINT",
        "PROFESSION_CHANGED" => "PROFESSION_CHANGED",
        "SKILL_DIGEST_MISMATCH" => "SKILL_DIGEST_MISMATCH",
        "SKILL_UNKNOWN" => "SKILL_UNKNOWN",
        "DIGEST_MISMATCH" => "DIGEST_MISMATCH",
        "BUNDLE_MISMATCH" => "BUNDLE_MISMATCH",
        "DISPATCH_NOT_FOUND" => "DISPATCH_NOT_FOUND",
        "IDEMPOTENCY_CONFLICT" => "IDEMPOTENCY_CONFLICT",
        "WRITER_STALE" => "WRITER_STALE",
        "PAIRING_REQUIRED" => "PAIRING_REQUIRED",
        _ => "AGENCY_ERROR",
    };
    reject(code, format!("{}: {}", e.code, e.message), "inspect_agency")
}
pub fn value<T: Serialize>(key: ObjectKey, version: i64, value: &T) -> store::Result<Record> {
    let value = serde_json::to_value(value)?;
    Ok(Record {
        key,
        version,
        revision_digest: canonical_json_sha256(&value)?,
        data: RecordData::Value { value },
        sources: vec![],
        materials: vec![],
    })
}
pub fn decode<T: serde::de::DeserializeOwned>(record: &Record) -> store::Result<T> {
    let RecordData::Value { value } = &record.data else {
        return Err(invalid("typed Participant record expected"));
    };
    Ok(serde_json::from_value(value.clone())?)
}
pub fn accept_binding(
    store: &mut Store,
    actor: &TrustedActor,
    command_key: &str,
    binding: Binding,
) -> store::Result<Record> {
    if binding.protocol != agency_proto::PROTOCOL || binding.id.is_empty() {
        return Err(invalid("compatible Agency binding required"));
    }
    let k = key(Scope::Control, "agency_binding", &binding.id);
    let old = store.get(&k)?;
    if let Some(old) = &old {
        let previous: Binding = decode(old)?;
        if agency_proto::canonical(&previous).map_err(port_error)?
            != agency_proto::canonical(&binding).map_err(port_error)?
        {
            return Err(reject(
                "BINDING_IMMUTABLE",
                "accept changed promises with hctl2 agency pair --binding-id <new-id> --key <new-key>",
                "hctl2 agency pair",
            ));
        }
    }
    let record = value(k, 1, &binding)?;
    write(
        store,
        actor,
        command_key,
        "agency.pair",
        &record,
        None,
        None,
    )?;
    // Retried command returns the accepted version, not a new version prepared above.
    store
        .get(&record.key)?
        .ok_or_else(|| invalid("binding was not stored"))
}
pub fn accept_profession(
    store: &mut Store,
    actor: &TrustedActor,
    command_key: &str,
    binding_id: &str,
    profession: &Profession,
) -> store::Result<Record> {
    let binding = store
        .get(&key(Scope::Control, "agency_binding", binding_id))?
        .ok_or_else(|| invalid("Agency binding required"))?;
    let b: Binding = decode(&binding)?;
    if !b.catalog.professions.contains(profession) {
        return Err(reject(
            "PROFESSION_CHANGED",
            "candidate is not in the accepted catalog; run hctl2 agency pair --binding-id <new-id> --agency-root <same-root> --key <new-key>, then accept under the new binding ID",
            "hctl2 agency pair",
        ));
    }
    let k = key(
        Scope::Control,
        "profession",
        &format!(
            "{binding_id}:{}:{}",
            profession.reference.id, profession.reference.digest
        ),
    );
    let mut record = value(k, 1, profession)?;
    record.sources = vec![reference(&binding)];
    write(
        store,
        actor,
        command_key,
        "profession.accept",
        &record,
        None,
        None,
    )?;
    store
        .get(&record.key)?
        .ok_or_else(|| invalid("Profession was not stored"))
}
fn write(
    store: &mut Store,
    actor: &TrustedActor,
    id: &str,
    operation: &str,
    record: &Record,
    old: Option<Reference>,
    inbox: Option<&InboxEntry>,
) -> store::Result<Value> {
    let input = serde_json::to_value(record)?;
    let command = Command {
        command_id: id.into(),
        idempotency_key: id.into(),
        actor: actor.0.clone(),
        target: record.key.clone(),
        expected: old.map_or(Expected::Absent, |r| Expected::Exact(r.version)),
        binding: Reference {
            key: record.key.clone(),
            version: Version::State(record.version),
        },
        input_digest: Command::digest_input(operation, &input)?,
        operation: operation.into(),
        input,
    };
    store.submit(store.generation(), actor, &command, inbox, |tx| {
        tx.put(record)?;
        if operation == "agency.pair" {
            tx.bind_secret(&reference(record), &record.key.id)?;
        }
        if operation == "dispatch.prepare" {
            let intent: Value = decode(record)?;
            let spec: Sealed<ExecutionSpec> = serde_json::from_value(intent["spec"].clone())?;
            tx.enqueue_effect(&store::EffectIntent {
                intent_id: format!("prepare:{}", record.key.id),
                owner: record.sources[0].clone(),
                binding: record.sources[1].clone(),
                operation: "agency.prepare".into(),
                target: record.key.id.clone(),
                conflict_scope: format!("dispatch:{}", record.key.id),
                permission_scope: record.key.scope.clone(),
                input: command.input.clone(),
                input_digest: Command::digest_input("agency.prepare", &command.input)?,
                idempotency_key: spec.document.idempotency_key,
            })?;
        }
        Ok(json!({"record":record}))
    })
}
/// Package 5 supplies the authorized owner record. The port cannot invent an Invocation.
pub fn prepare_dispatch(
    store: &mut Store,
    actor: &TrustedActor,
    id: &str,
    owner: &Reference,
    spec: &Sealed<ExecutionSpec>,
    bundle: &Sealed<agency_proto::context::Bundle>,
) -> store::Result<Record> {
    let owner_record = store
        .get(&owner.key)?
        .ok_or_else(|| invalid("persist authorized owner first"))?;
    if reference(&owner_record) != *owner {
        return Err(reject(
            "OWNER_MISMATCH",
            "dispatch must retain exact authorized owner",
            "rebuild_preview",
        ));
    }
    let record = dispatch::plan(store, id, &owner_record, spec, bundle)?.record;
    write(store, actor, id, "dispatch.prepare", &record, None, None)?;
    Ok(record)
}
pub fn record_dispatch(
    store: &mut Store,
    actor: &TrustedActor,
    intent: &Record,
    dispatch: &Dispatch,
) -> store::Result<Record> {
    // Readback after activation may have advanced the provider's state. The immutable
    // acceptance command always records the same prepared mapping, not that observation.
    let mut acceptance = dispatch.clone();
    acceptance.state = agency_proto::DispatchState::Prepared;
    let dispatch = &acceptance;
    let original: Value = decode(intent)?;
    let spec: Sealed<ExecutionSpec> = serde_json::from_value(original["spec"].clone())?;
    if dispatch.owner != spec.document.owner
        || dispatch.spec_digest != spec.digest
        || dispatch.bundle_digest != spec.document.bundle.digest
        || dispatch.binding != spec.document.binding
    {
        return Err(reject(
            "DISPATCH_MISMATCH",
            "Agency acceptance differs from frozen intent",
            "inspect_dispatch",
        ));
    }
    dispatch
        .capabilities
        .fulfills(&spec.document.required_capabilities)
        .map_err(port_error)?;
    let k = key(intent.key.scope.clone(), "dispatch", &dispatch.reference);
    let mut record = value(k, 1, dispatch)?;
    record.sources = intent.sources.clone();
    let input = serde_json::to_value(dispatch)?;
    let command = Command {
        command_id: format!("dispatch-map:{}", intent.key.id),
        idempotency_key: format!("dispatch-map:{}", intent.key.id),
        actor: actor.0.clone(),
        target: intent.key.clone(),
        expected: Expected::Exact(Version::State(intent.version)),
        binding: reference(intent),
        input_digest: Command::digest_input("dispatch.accept", &input)?,
        operation: "dispatch.accept".into(),
        input,
    };
    let mut mapped = original;
    mapped["dispatch"] = json!(dispatch.reference);
    let mut updated = value(intent.key.clone(), intent.version + 1, &mapped)?;
    updated.sources = intent.sources.clone();
    let prepare_id = format!("prepare:{}", intent.key.id);
    let effect = store.effect(&prepare_id)?.0;
    store.submit(store.generation(), actor, &command, None, |tx| {
        let owner = tx
            .get(&record.sources[0].key)?
            .ok_or_else(|| invalid("authorized owner missing"))?;
        if reference(&owner) != record.sources[0] {
            return Err(reject(
                "OWNER_STALE",
                "owner changed before mapping",
                "inspect_original_dispatch",
            ));
        }
        tx.confirm_effect(
            &prepare_id,
            &store::Readback::Confirmed {
                binding: effect.binding.clone(),
                target: effect.target.clone(),
                input_digest: effect.input_digest.clone(),
                result: serde_json::to_value(dispatch)?,
            },
        )?;
        tx.put(&record)?;
        tx.put(&updated)?;
        let input = json!({"dispatch":dispatch.reference});
        tx.enqueue_effect(&store::EffectIntent {
            intent_id: format!("activate:{}", intent.key.id),
            owner: record.sources[0].clone(),
            binding: record.sources[1].clone(),
            operation: "agency.activate".into(),
            target: dispatch.reference.clone(),
            conflict_scope: format!("dispatch:{}", intent.key.id),
            permission_scope: record.key.scope.clone(),
            input_digest: Command::digest_input("agency.activate", &input)?,
            input,
            idempotency_key: format!("activate:{}", intent.key.id),
        })?;
        Ok(json!({"dispatch":record}))
    })?;
    Ok(record)
}
/// Runtime observations remain observations. They never change an Invocation or Task.
pub fn record_observation(
    store: &mut Store,
    actor: &TrustedActor,
    dispatch: &Record,
    trace: &Trace,
) -> store::Result<()> {
    let d: Dispatch = decode(dispatch)?;
    if d.reference != trace.dispatch.reference
        || d.owner != trace.dispatch.owner
        || d.spec_digest != trace.dispatch.spec_digest
    {
        return Err(invalid("observation belongs to another dispatch"));
    }
    let trace_digest = hash(&agency_proto::canonical(trace).map_err(port_error)?);
    let k = key(
        dispatch.key.scope.clone(),
        "dispatch_observation",
        &format!("{}:{trace_digest}", d.reference),
    );
    let mut record = value(k, 1, trace)?;
    record.sources = vec![reference(dispatch)];
    write(
        store,
        actor,
        &format!(
            "observation:{}:{}",
            d.reference,
            hash(&agency_proto::canonical(trace).map_err(port_error)?)
        ),
        "dispatch.observe",
        &record,
        None,
        None,
    )?;
    Ok(())
}
pub fn record_unreachable(
    store: &mut Store,
    actor: &TrustedActor,
    dispatch: &Record,
    observed_ms: u64,
) -> store::Result<()> {
    let k = key(
        dispatch.key.scope.clone(),
        "dispatch_contact",
        &format!("{}:{observed_ms}", dispatch.key.id),
    );
    let mut record = value(
        k,
        1,
        &json!({"contact":"unreachable","observed_ms":observed_ms,"dispatch":dispatch.key.id}),
    )?;
    record.sources = vec![reference(dispatch)];
    write(
        store,
        actor,
        &format!("contact:{}:{observed_ms}", dispatch.key.id),
        "dispatch.contact",
        &record,
        None,
        None,
    )?;
    Ok(())
}
/// Byte preservation precedes acknowledgement; this is not domain result admission.
pub fn preserve_proposal(
    store: &mut Store,
    actor: &TrustedActor,
    dispatch: &Record,
    proposal: &Proposal,
) -> store::Result<()> {
    // A provider acknowledgement is mutable operational state, not part of the inbox
    // identity. Retrying after an acknowledgement must preserve the same command.
    let mut immutable = proposal.clone();
    immutable.preserved = false;
    let proposal = &immutable;
    let d: Dispatch = decode(dispatch)?;
    let h = &proposal.header;
    if h.dispatch != d.reference
        || h.owner != d.owner
        || h.binding != d.binding
        || h.spec_digest != d.spec_digest
        || h.bundle_digest != d.bundle_digest
        || hash(&proposal.output) != proposal.content_digest
    {
        return Err(reject(
            "PROPOSAL_MISMATCH",
            "result envelope or exact bytes differ",
            "inspect_original_proposal",
        ));
    }
    if proposal.evidence == agency_proto::EvidenceLevel::Unmediated
        && !d.capabilities.tool_execution_unmediated
    {
        return Err(reject(
            "EVIDENCE_SOURCE_INVALID",
            "Agency has no direct tool-report capability",
            "retain_as_untrusted",
        ));
    }
    if proposal.outputs.len() != 1 {
        return Err(invalid(
            "this port version accepts one exact output per proposal",
        ));
    }
    for output in &proposal.outputs {
        if output.owner != d.owner
            || output.dispatch != d.reference
            || output.authorization.id != d.reference
            || output.authorization.digest != d.spec_digest
            || output.content_digest != proposal.content_digest
            || output.schema != proposal.schema
            || output.candidate.digest != proposal.content_digest
        {
            return Err(reject(
                "PROPOSAL_MISMATCH",
                "output authority or content differs",
                "inspect_original_proposal",
            ));
        }
        output.candidate.validate().map_err(port_error)?;
        output.authorization.validate().map_err(port_error)?;
    }
    let material = store.save_material(
        store.generation(),
        actor,
        &dispatch.key.scope,
        &format!("proposal:{}", h.proposal_id),
        "output",
        &proposal.output,
    )?;
    let mut record = value(
        key(dispatch.key.scope.clone(), "proposal_inbox", &h.proposal_id),
        1,
        proposal,
    )?;
    record.sources = vec![reference(dispatch)];
    record.materials = vec![material.clone()];
    let input = serde_json::to_value(proposal)?;
    let command = Command {
        command_id: format!("proposal:{}", h.proposal_id),
        idempotency_key: format!("proposal:{}", h.proposal_id),
        actor: actor.0.clone(),
        target: record.key.clone(),
        expected: Expected::Absent,
        binding: reference(dispatch),
        input_digest: Command::digest_input("proposal.preserve", &input)?,
        operation: "proposal.preserve".into(),
        input,
    };
    let inbox = InboxEntry {
        binding: reference(dispatch),
        message_key: h.idempotency_key.clone(),
        digest: hash(&agency_proto::canonical(proposal).map_err(port_error)?),
    };
    store.submit(store.generation(), actor, &command, Some(&inbox), |tx| {
        tx.admit_material(&material)?;
        tx.put(&record)?;
        Ok(json!({"preserved":true}))
    })?;
    Ok(())
}
