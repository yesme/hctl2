//! Native transaction fragments; the owner module admits authorization and outbox together.
use crate::{Binding, decode, frozen, invalid, key, port_error, reference, reject, value};
use agency_proto::{ExecutionSpec, OwnerKind, Profession, Sealed, context::Bundle};
use serde_json::{Value, json};
use store::{Command, EffectIntent, Record, Scope, Store};

pub struct DispatchPlan {
    pub record: Record,
    pub effect: EffectIntent,
    pub dependencies: Vec<Record>,
}

/// `owner` may be the not-yet-admitted record in the owning module's transaction.
/// This function grants nothing and performs no provider I/O.
pub fn plan(
    store: &Store,
    id: &str,
    owner: &Record,
    spec: &Sealed<ExecutionSpec>,
    bundle: &Sealed<Bundle>,
) -> store::Result<DispatchPlan> {
    spec.verify().map_err(port_error)?;
    spec.document.validate().map_err(port_error)?;
    bundle.verify().map_err(port_error)?;
    bundle.document.validate_delivery().map_err(port_error)?;
    if id.trim().is_empty()
        || owner.key.scope != Scope::Project(spec.document.owner.project.clone())
        || owner.key.id != spec.document.owner.id
        // The owning reducer validates invocation_version / attempt_generation.
        // A generic record's state version is not an execution generation.
        || bundle.document.consumer != spec.document.owner
        || bundle.document.id != spec.document.bundle.id
        || bundle.digest != spec.document.bundle.digest
        || bundle.document.manifest != spec.document.manifest
        || bundle.document.permission_digest != spec.document.permission_digest
        || bundle.document.budget != spec.document.budget
    {
        return Err(reject(
            "OWNER_MISMATCH",
            "owner, Spec or delivered Bundle differs",
            "rebuild_preview",
        ));
    }
    let binding = store
        .get(&key(
            Scope::Control,
            "agency_binding",
            &spec.document.binding.id,
        ))?
        .ok_or_else(|| invalid("binding is not accepted"))?;
    if frozen(&binding) != spec.document.binding {
        return Err(reject(
            "BINDING_MISMATCH",
            "binding revision differs",
            "rebuild_preview",
        ));
    }
    let b: Binding = decode(&binding)?;
    if !b.catalog.professions.contains(&spec.document.profession) {
        return Err(invalid("accepted Profession required"));
    }
    let accepted = store
        .get(&key(
            Scope::Control,
            "profession",
            &format!(
                "{}:{}:{}",
                b.id,
                spec.document.profession.reference.id,
                spec.document.profession.reference.digest
            ),
        ))?
        .ok_or_else(|| invalid("explicit Profession acceptance required"))?;
    let profession: Profession = decode(&accepted)?;
    if profession != spec.document.profession || !accepted.sources.contains(&reference(&binding)) {
        return Err(invalid("accepted Profession or Agency source differs"));
    }
    let k = key(owner.key.scope.clone(), "dispatch_intent", id);
    // Formal Room Invocation dispatches use their owner ID as the only key.
    // Test/legacy owners and Run Attempts retain the port's caller-supplied key.
    if spec.document.owner.kind == OwnerKind::RoomInvocation && owner.key.kind == "room_invocation"
    {
        if id != owner.key.id {
            return Err(reject(
                "DISPATCH_EXISTS",
                "Room Invocation dispatch key is its owner ID",
                "inspect_original_dispatch",
            ));
        }
        if let Some(previous) = store.get(&k)? {
            let input: Value = decode(&previous)?;
            let original: Sealed<ExecutionSpec> = serde_json::from_value(input["spec"].clone())?;
            if original != *spec {
                return Err(reject(
                    "DISPATCH_EXISTS",
                    "dispatch key already freezes another Spec",
                    "inspect_original_dispatch",
                ));
            }
        }
    }
    let mut record = value(
        k,
        1,
        &json!({"owner":reference(owner),"spec":spec,"bundle":bundle,"dispatch":null}),
    )?;
    record.sources = vec![reference(owner), reference(&binding)];
    let input = serde_json::to_value(&record)?;
    let effect = EffectIntent {
        intent_id: format!("prepare:{id}"),
        owner: reference(owner),
        binding: reference(&binding),
        operation: "agency.prepare".into(),
        target: id.into(),
        conflict_scope: format!("dispatch:{id}"),
        permission_scope: owner.key.scope.clone(),
        input_digest: Command::digest_input("agency.prepare", &input)?,
        input,
        idempotency_key: spec.document.idempotency_key.clone(),
    };
    Ok(DispatchPlan {
        record,
        effect,
        dependencies: vec![binding, accepted],
    })
}
