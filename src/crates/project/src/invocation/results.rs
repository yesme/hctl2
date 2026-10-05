//! Admit the returned read-only answer, not a claim that a Task is complete.
use super::*;
use agency_proto::{Dispatch, Proposal};
use store::{EffectIntent, InboxEntry};

pub fn admit_result(
    store: &mut Store,
    actor: &TrustedActor,
    project: &str,
    id: &str,
    inbox: &Record,
    now_ms: u64,
) -> store::Result<Value> {
    let (root, invocation) = invocation(store, project, id)?;
    let actor = super::lifecycle::reducer(actor, &root)?;
    let proposal: Proposal = participant::decode(inbox)?;
    let dispatch = required(
        store,
        &inbox
            .sources
            .first()
            .ok_or_else(|| invalid("proposal dispatch missing"))?
            .key,
    )?;
    let d: Dispatch = participant::decode(&dispatch)?;
    let h = &proposal.header;
    let spec = &invocation.spec;
    let target = key(
        root.key.scope.clone(),
        "invocation_result",
        &format!("{id}:{}", h.proposal_id),
    );
    let input = json!({"proposal":reference(inbox),"digest":proposal.content_digest,"owner":spec.document.owner});
    let command = Command {
        command_id: format!("admit:{id}:{}", h.proposal_id),
        idempotency_key: format!("admit:{id}:{}", h.proposal_id),
        actor: actor.0.clone(),
        target: target.clone(),
        expected: Expected::Absent,
        binding: reference(&dispatch),
        operation: "invocation.result".into(),
        input_digest: Command::digest_input("invocation.result", &input)?,
        input,
    };
    if store.get(&target)?.is_some() {
        return store.submit(store.generation(), &actor, &command, None, |_| Err(stale()));
    }
    if !current_authorization(store, &reference(&root), now_ms)? {
        return Err(reject(
            "OWNER_STALE",
            "late result retained for audit only",
            "inspect_original_invocation",
        ));
    }
    let (state, lifecycle) = lifecycle(store, project, id)?;
    if !matches!(lifecycle.state, State::Running | State::WaitingInput)
        || h.owner != spec.document.owner
        || d.owner != h.owner
        || h.dispatch != d.reference
        || h.spec_digest != spec.digest
        || d.spec_digest != spec.digest
        || h.bundle_digest != spec.document.bundle.digest
        || d.bundle_digest != h.bundle_digest
        || h.binding != spec.document.binding
        || d.binding != h.binding
        || h.producer_sequence == 0
        || h.idempotency_key.trim().is_empty()
        || spec.document.delivery_scope != vec![invocation.preview.input.room_id.clone()]
        || !spec
            .document
            .permissions
            .iter()
            .any(|p| p == "context.read")
        || !matches!(
            proposal.schema.as_str(),
            "adapter.stdout.v1" | "claude.turn.v1"
        )
        || proposal.outputs.len() != 1
        || inbox.materials.len() != 1
    {
        return Err(reject(
            "PROPOSAL_MISMATCH",
            "result is outside the frozen read-only invocation",
            "inspect_original_proposal",
        ));
    }
    let output = &proposal.outputs[0];
    if output.owner != h.owner
        || output.dispatch != h.dispatch
        || output.authorization.id != d.reference
        || output.authorization.revision != "1"
        || output.authorization.digest != spec.digest
        || output.schema != proposal.schema
        || output.content_digest != proposal.content_digest
        || output.candidate.digest != proposal.content_digest
        || agency_proto::hash(&proposal.output) != proposal.content_digest
        || store.read_material(&actor, &inbox.materials[0])? != proposal.output
        || proposal.evidence == agency_proto::EvidenceLevel::Unmediated
            && !d.capabilities.tool_execution_unmediated
    {
        return Err(reject(
            "PROPOSAL_MISMATCH",
            "output authority, evidence or preserved bytes differ",
            "inspect_original_proposal",
        ));
    }
    let body = std::str::from_utf8(&proposal.output).map_err(|_| {
        reject(
            "PROPOSAL_MISMATCH",
            "read-only answer must be UTF-8",
            "inspect_original_proposal",
        )
    })?;
    let (room_binding, room) = chat::room(store, project, &invocation.preview.input.room_id)?;
    let external = room
        .matrix_room_id
        .as_ref()
        .ok_or_else(|| invalid("Room binding is not ready"))?;
    let effect_id = format!("projection:{id}:{}", h.proposal_id);
    let projection =
        json!({"room":room,"body":body,"proposal":reference(inbox),"evidence":proposal.evidence});
    let effect = EffectIntent {
        intent_id: effect_id.clone(),
        owner: reference(&root),
        binding: reference(&room_binding),
        operation: "invocation.project".into(),
        target: external.clone(),
        conflict_scope: effect_id.clone(),
        permission_scope: root.key.scope.clone(),
        input_digest: Command::digest_input("invocation.project", &projection)?,
        input: projection,
        idempotency_key: effect_id.clone(),
    };
    let mut admitted = value_record(
        target,
        1,
        &json!({"proposal":reference(inbox),"content_digest":proposal.content_digest,"evidence":proposal.evidence}),
    )?;
    admitted.materials = inbox.materials.clone();
    admitted.sources = vec![reference(&root), reference(inbox), reference(&dispatch)];
    let completed = value_record(
        state.key.clone(),
        state
            .version
            .checked_add(1)
            .ok_or_else(|| invalid("Invocation state version overflow"))?,
        &Lifecycle {
            state: State::Completed,
            reason: Some("read-only answer admitted; not Task acceptance".into()),
        },
    )?;
    let event = InboxEntry {
        binding: reference(&dispatch),
        message_key: h.producer_sequence.to_string(),
        digest: foundation::canonical_json_sha256(&serde_json::to_value(&proposal)?)?,
    };
    store.submit(store.generation(), &actor, &command, Some(&event), |tx| {
        for expected in [reference(&root), reference(&state), reference(inbox), reference(&dispatch), reference(&room_binding)] {
            if tx.get(&expected.key)?.as_ref().map(reference) != Some(expected) { return Err(stale()); }
        }
        tx.put(&admitted)?;
        tx.put(&completed)?;
        tx.enqueue_effect(&effect)?;
        Ok(json!({"invocation_id":id,"state":"completed","result":reference(&admitted),"projection_effect":effect_id}))
    })
}
