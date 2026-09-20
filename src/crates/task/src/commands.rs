use crate::planning::card;
use crate::*;
use foundation::{canonical_json, canonical_json_sha256};
use serde_json::{Value, json};
use store::{Command, Expected, Reference, Scope, Store, TrustedActor, Version};

pub fn admit(store: &mut Store, actor: &TrustedActor, mut plan: Plan) -> Result<Value> {
    let scopes = plan
        .records
        .iter()
        .map(|r| r.key.scope.clone())
        .chain(plan.effects.iter().map(|e| e.permission_scope.clone()))
        .collect::<Vec<_>>();
    let actor = owner(actor, scopes)?;
    let input = serde_json::to_value(&plan.input)?;
    let ck = command_key(store, &plan.input.key);
    if let Some(old) = replay(store, &plan.input)? {
        plan = old;
    } else {
        // Set membership is a precondition too: a newly claimed Task or new Run after
        // the deletion/cancellation preview must not escape the user's consequence list.
        if matches!(
            plan.input.action,
            Action::DeleteCard { .. } | Action::Cancel { .. }
        ) {
            let current = prepare(store, plan.input.clone())?;
            if serde_json::to_value(&current.checks)? != serde_json::to_value(&plan.checks)?
                || current.result != plan.result
            {
                return Err(stale());
            }
        }
        if let Some(adoption) = &plan.adoption {
            let task_key = plan.task_key.clone().ok_or_else(stale)?;
            let r = plan
                .records
                .iter_mut()
                .find(|r| r.key == task_key)
                .ok_or_else(stale)?;
            let mut t: Task = decode(r)?;
            let material = store.save_material(
                store.generation(),
                &actor,
                &task_key.scope,
                &plan.input.key,
                "contract",
                &canonical_json(&serde_json::to_value(&adoption.contract)?)?,
            )?;
            let revision = Revision {
                number: t.revision.as_ref().map_or(1, |r| r.number + 1),
                material: material.clone(),
                origin: adoption.origin.clone(),
                proposal_digest: canonical_json_sha256(&serde_json::to_value(&adoption.contract)?)?,
                binding: match &adoption.origin {
                    ContractOrigin::Local { .. } => None,
                    ContractOrigin::Backend { .. } => Some(Reference {
                        key: task_key.clone(),
                        version: Version::State(r.version - 1),
                    }),
                },
                policy_digest: canonical_json_sha256(
                    &json!({"content":"backend_authoritative","contract":"hctl_authoritative","lifecycle":"hctl_authoritative"}),
                )?,
                backend_projection_digest: if let ContractOrigin::Backend { snapshot, .. } =
                    &adoption.origin
                {
                    let snap: Snapshot = decode(&required(store, &snapshot.key)?)?;
                    Some(contract_projection(card(
                        &snap,
                        &t.entity
                            .as_ref()
                            .ok_or_else(stale)?
                            .immutable_external_entity_id,
                    )?)?)
                } else {
                    None
                },
            };
            t.revision = Some(revision.clone());
            t.pending_contract = None;
            *r = value_record(task_key.clone(), r.version, &t)?;
            let mut rev = value_record(
                key(
                    task_key.scope,
                    "task_revision",
                    &format!("{}:{}", t.id, revision.number),
                ),
                1,
                &revision,
            )?;
            rev.materials.push(material);
            plan.records.push(rev);
        }
    }
    let cmd = Command {
        command_id: format!("task:{}", plan.input.key),
        idempotency_key: plan.input.key.clone(),
        actor: actor.0.clone(),
        target: ck.clone(),
        expected: Expected::Absent,
        binding: Reference {
            key: key(Scope::Control, "module", "task"),
            version: Version::State(1),
        },
        input_digest: Command::digest_input("task.command", &input)?,
        operation: "task.command".into(),
        input,
    };
    store.submit(store.generation(), &actor, &cmd, None, |tx| {
        for c in &plan.checks {
            if tx.get(&c.key)?.as_ref().map(|r| r.version) != c.version {
                return Err(stale());
            }
        }
        if let Action::DeleteCard {
            confirm_irreversible: false,
            ..
        } = plan.input.action
        {
            return Err(reject(
                "DELETE_CONFIRMATION_REQUIRED",
                "review consequences then explicitly confirm",
                "confirm_delete",
            ));
        }
        for r in &plan.records {
            for m in &r.materials {
                tx.admit_material(m)?;
            }
            tx.put(r)?;
        }
        for e in &plan.effects {
            tx.enqueue_effect(e)?;
        }
        for id in &plan.cancel_effects {
            tx.cancel_pending_effect(id)?;
        }
        tx.put(&value_record(ck, 1, &plan)?)?;
        Ok(plan.result.clone())
    })
}
