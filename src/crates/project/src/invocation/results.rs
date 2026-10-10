//! A returned answer or sealed Git result is not a claim that a Task is complete.
use super::*;
use agency_proto::{Dispatch, Proposal};
use store::{EffectIntent, InboxEntry};

/// Preflight before a local Git operation, not result admission. The transaction
/// below rechecks every authority, saved-byte and lifecycle guard after sealing.
pub fn sealing_input(
    store: &Store,
    actor: &TrustedActor,
    project: &str,
    id: &str,
    inbox: &Record,
    now_ms: u64,
) -> store::Result<(repo::changeset::Output, repo::changeset::Seal, String)> {
    let (root, call) = invocation(store, project, id)?;
    let scoped = super::lifecycle::reducer(actor, &root)?;
    if !current_authorization(store, &reference(&root), now_ms)? {
        return Err(reject(
            "OWNER_STALE",
            "write authorization ended before sealing",
            "inspect_original_invocation",
        ));
    }
    let write = call
        .preview
        .write
        .as_ref()
        .ok_or_else(|| invalid("write preview required"))?;
    let proposal: Proposal = participant::decode(inbox)?;
    if proposal.schema != repo::changeset::OUTPUT_SCHEMA
        || proposal.header.owner != call.spec.document.owner
        || proposal.header.spec_digest != call.spec.digest
        || proposal.header.bundle_digest != call.spec.document.bundle.digest
        || proposal.header.binding != call.spec.document.binding
        || inbox.materials.len() != 1
        || store.read_material(&scoped, &inbox.materials[0])? != proposal.output
        || agency_proto::hash(&proposal.output) != proposal.content_digest
    {
        return Err(reject(
            "PROPOSAL_MISMATCH",
            "write proposal differs from its saved authorization",
            "inspect_original_proposal",
        ));
    }
    let output: repo::changeset::Output = serde_json::from_slice(&proposal.output)?;
    let pending = &write.lease.pending;
    if output.change_set_id != pending.change_set_id
        || output.lease.lease_id != pending.lease.lease_id
        || output.lease.generation != pending.lease.generation
        || output.base_commit_sha != pending.baseline_commit
    {
        return Err(reject(
            "PROPOSAL_MISMATCH",
            "write proposal widens the frozen boundary",
            "inspect_original_proposal",
        ));
    }
    let current = repo::changeset::get_change_set(store, &pending.repo_id, &pending.change_set_id)?;
    if current.version != pending.version
        || current.lease.state != repo::changeset::LeaseState::Active
        || current.lease.holder != super::write::holder(&call)
    {
        return Err(reject(
            "LEASE_NOT_CURRENT",
            "write lease is no longer active",
            "inspect_change_set",
        ));
    }
    let seal = repo::changeset::Seal {
        association_key: proposal.header.proposal_id,
        change_set_id: output.change_set_id.clone(),
        change_set_version: pending.version,
        lease: Some(output.lease.clone()),
        base_commit_sha: output.base_commit_sha.clone(),
        result_tree_sha: String::new(),
        result_commit_sha: None,
        parent_revision_id: output.parent_revision_id.clone(),
        producer_ref: super::write::holder(&call),
    };
    Ok((output, seal, pending.repo_id.clone()))
}

pub fn admit_result(
    store: &mut Store,
    actor: &TrustedActor,
    project: &str,
    id: &str,
    inbox: &Record,
    now_ms: u64,
) -> store::Result<Value> {
    admit(store, actor, project, id, inbox, None, now_ms)
}

/// The control adapter has run the native Git tool outside the Store transaction.
/// All Proposal guards are still checked here, including its immutable saved bytes.
pub fn admit_sealed_result(
    store: &mut Store,
    actor: &TrustedActor,
    project: &str,
    id: &str,
    inbox: &Record,
    seal: &repo::changeset::Seal,
    now_ms: u64,
) -> store::Result<Value> {
    admit(store, actor, project, id, inbox, Some((seal, None)), now_ms)
}

/// The adapter has mechanically compared the baseline and result trees. An
/// unchanged turn is an accepted result, not a code version or a publication.
pub fn admit_unchanged_result(
    store: &mut Store,
    actor: &TrustedActor,
    project: &str,
    id: &str,
    inbox: &Record,
    readback: (&repo::changeset::Seal, &str),
    now_ms: u64,
) -> store::Result<Value> {
    admit(
        store,
        actor,
        project,
        id,
        inbox,
        Some((readback.0, Some(readback.1))),
        now_ms,
    )
}

fn admit(
    store: &mut Store,
    actor: &TrustedActor,
    project: &str,
    id: &str,
    inbox: &Record,
    readback: Option<(&repo::changeset::Seal, Option<&str>)>,
    now_ms: u64,
) -> store::Result<Value> {
    let (seal, baseline_tree_sha) = match readback {
        Some((seal, baseline_tree)) => (Some(seal), baseline_tree),
        None => (None, None),
    };
    let (root, invocation) = invocation(store, project, id)?;
    if invocation.authorization.write && seal.is_none() {
        return Err(reject(
            "CHANGESET_RESULT_REQUIRED",
            "write result requires Git sealing and atomic revision admission",
            "inspect_original_proposal",
        ));
    }
    let mut actor = super::lifecycle::reducer(actor, &root)?;
    if let Some(write) = &invocation.preview.write {
        actor
            .0
            .permission_scope
            .push(Scope::Repo(write.lease.pending.repo_id.clone()));
    }
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
    let mut input = json!({"proposal":reference(inbox),"digest":proposal.content_digest,"owner":spec.document.owner});
    if let Some(seal) = seal {
        input["seal"] = serde_json::to_value(seal)?;
    }
    if let Some(tree) = baseline_tree_sha {
        input["unchanged_baseline_tree"] = json!(tree);
    }
    let publication = if seal.is_some() {
        super::write::publication(store, &invocation)?
    } else {
        None
    };
    if let Some(publication) = &publication {
        input["publication"] = json!({
            "policy": publication.policy,
            "authorizing_actor": publication.authorizing_actor,
        });
    }
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
        || if invocation.authorization.write {
            proposal.schema != repo::changeset::OUTPUT_SCHEMA
        } else {
            !matches!(
                proposal.schema.as_str(),
                "adapter.stdout.v1" | "claude.turn.v1" | "codex.turn.v1"
            ) || seal.is_some()
        }
        || proposal.outputs.len() != 1
        || inbox.materials.len() != 1
    {
        return Err(reject(
            "PROPOSAL_MISMATCH",
            "result is outside the frozen invocation",
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
            "answer must be UTF-8",
            "inspect_original_proposal",
        )
    })?;
    if let Some(seal) = seal {
        let declared: repo::changeset::Output = serde_json::from_slice(&proposal.output)?;
        if baseline_tree_sha.is_some_and(|tree| tree != seal.result_tree_sha)
            || matches!(
                declared.location,
                repo::changeset::OutputLocation::NoChanges { .. }
            ) && baseline_tree_sha.is_none()
        {
            return Err(reject(
                "NO_CHANGES_MISMATCH",
                "unchanged result requires matching native baseline and result trees",
                "submit_actual_changeset_output",
            ));
        }
        let write = invocation
            .preview
            .write
            .as_ref()
            .ok_or_else(|| invalid("write preview missing"))?;
        if declared.change_set_id != write.lease.pending.change_set_id
            || seal.change_set_id != declared.change_set_id
            || seal.lease.as_ref() != Some(&declared.lease)
            || declared.lease.lease_id != write.lease.pending.lease.lease_id
            || declared.lease.generation != write.lease.pending.lease.generation
            || spec.document.write_lease.as_ref() != Some(&super::write::lease_reference(write)?)
            || seal.base_commit_sha != declared.base_commit_sha
            || spec.document.base.as_deref() != Some(declared.base_commit_sha.as_str())
            || seal.parent_revision_id != declared.parent_revision_id
            || seal.producer_ref != super::write::holder(&invocation)
            || seal.association_key != h.proposal_id
            || !spec.document.permissions.iter().any(|p| p == "git.write")
        {
            return Err(reject(
                "PROPOSAL_MISMATCH",
                "ChangeSet output widens the frozen write boundary",
                "inspect_original_proposal",
            ));
        }
        if let repo::changeset::OutputLocation::Commit { commit_sha, .. } = &declared.location
            && seal.result_commit_sha.as_ref() != Some(commit_sha)
        {
            return Err(reject(
                "PROPOSAL_MISMATCH",
                "sealed commit differs from the proposed exact commit",
                "inspect_original_proposal",
            ));
        }
    }
    let (room_binding, room) = chat::room(store, project, &invocation.preview.input.room_id)?;
    let external = room
        .matrix_room_id
        .as_ref()
        .ok_or_else(|| invalid("Room binding is not ready"))?;
    let effect_id = format!("projection:{id}:{}", h.proposal_id);
    // What the Room shows. A read-only answer is the answer. A write result's output is the
    // machine ChangeSet record; the Room gets one human line about what was admitted and
    // what happens next instead (CT-PRODUCT: quiet on success, readable at a glance).
    let projection_effect = |body: &str| -> store::Result<EffectIntent> {
        let projection = json!({"room":room,"body":body,"proposal":reference(inbox),"evidence":proposal.evidence});
        Ok(EffectIntent {
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
        })
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
            reason: Some(
                if baseline_tree_sha.is_some() {
                    "unchanged Git tree verified; not Task acceptance"
                } else if seal.is_some() {
                    "Git version admitted; not Task acceptance"
                } else {
                    "read-only answer admitted; not Task acceptance"
                }
                .into(),
            ),
        },
    )?;
    let event = InboxEntry {
        binding: reference(&dispatch),
        message_key: h.producer_sequence.to_string(),
        digest: foundation::canonical_json_sha256(&serde_json::to_value(&proposal)?)?,
    };
    let write_cleanup = if seal.is_some() {
        Some(super::lifecycle::stop_effect(store, &root, &invocation)?)
    } else {
        None
    };
    let control_id = store.control_id().to_owned();
    store.submit(store.generation(), &actor, &command, Some(&event), |tx| {
        for expected in [reference(&root), reference(&state), reference(inbox), reference(&dispatch), reference(&room_binding)] {
            if tx.get(&expected.key)?.as_ref().map(reference) != Some(expected) { return Err(stale()); }
        }
        let revision = match seal {
            Some(seal) => {
                let write = invocation.preview.write.as_ref().ok_or_else(|| invalid("write preview missing"))?;
                if baseline_tree_sha.is_some() {
                    repo::changeset::validate_in_transaction(tx, &actor, &write.lease.pending.repo_id, seal, repo::changeset::OwnerGate::Active)?;
                    None
                } else {
                    Some(repo::changeset::admit_in_transaction(tx, &actor, &write.lease.pending.repo_id, seal, repo::changeset::OwnerGate::Active)?)
                }
            }
            None => None,
        };
        let publication_intent = match (&revision, &publication) {
            (Some(revision), Some(publication)) => {
                let write = invocation.preview.write.as_ref().ok_or_else(|| invalid("write preview missing"))?;
                if tx.get(&write.policy_record.key)?.as_ref().map(reference) != Some(write.policy_record.clone()) {
                    return Err(stale());
                }
                Some(repo::review::enqueue(tx, &control_id, revision, &write.lease.pending.repo_id, write.lease.pending.binding_version, publication, now_ms)?)
            }
            _ => None,
        };
        let mut admitted = admitted.clone();
        if baseline_tree_sha.is_some()
            && let RecordData::Value { value } = &mut admitted.data
        {
            value["no_changes"] = json!(true);
            admitted.revision_digest = canonical_json_sha256(value)?;
        }
        if let Some(revision) = &revision
            && let RecordData::Value { value } = &mut admitted.data
        {
            value["change_set_revision"] = serde_json::to_value(revision)?;
            admitted.revision_digest = canonical_json_sha256(value)?;
        }
        tx.put(&admitted)?;
        tx.put(&completed)?;
        if let Some(cleanup) = &write_cleanup {
            // Result admission does not prove the writer stopped. Retain exclusion
            // until the original dispatch supplies physical stop evidence.
            super::write::revoke(tx, &invocation, false)?;
            let mut revoked = invocation.clone();
            revoked.authorization.valid = false;
            let mut record = value_record(root.key.clone(), root.version.checked_add(1).ok_or_else(|| invalid("owner version overflow"))?, &revoked)?;
            record.sources = root.sources.clone();
            tx.put(&record)?;
            tx.enqueue_effect(cleanup)?;
        }
        let room_body = match (&revision, seal) {
            (Some(revision), _) => write_summary(revision, publication_intent.as_ref()),
            (None, Some(seal)) => format!(
                "Write turn returned no changes: the worktree matches baseline {}; no new version, nothing published.",
                short(&seal.base_commit_sha)
            ),
            (None, None) => body.to_owned(),
        };
        tx.enqueue_effect(&projection_effect(&room_body)?)?;
        let mut result = json!({"invocation_id":id,"state":"completed","result":reference(&admitted),"projection_effect":effect_id});
        if baseline_tree_sha.is_some() { result["no_changes"] = json!(true); }
        if let Some(revision) = revision { result["change_set_revision"] = serde_json::to_value(revision)?; }
        if let Some(intent) = publication_intent { result["review_publish_intent"] = serde_json::to_value(intent)?; }
        Ok(result)
    })
}

/// The Room line for an admitted write: which version, from what, and what happens next.
fn write_summary(
    revision: &repo::changeset::ChangeSetRevision,
    intent: Option<&repo::review::Intent>,
) -> String {
    let next = match intent {
        Some(intent) if intent.state == repo::review::State::PendingHuman => format!(
            "Publishing for review waits for a human: hctl2 review publish {} {}",
            intent.repo_id, intent.intent_id
        ),
        Some(intent) => format!(
            "Publishing for review is queued: push to {} and open a review request, each read back.",
            intent.branch
        ),
        None => "No review publication was authorized for this write.".into(),
    };
    format!(
        "Write admitted as version {} of ChangeSet {} (baseline {}, result tree {}). {next}",
        short(&revision.change_set_revision_id),
        short(&revision.change_set_id),
        short(&revision.base_commit_sha),
        short(&revision.result_tree_sha),
    )
}

/// Identifiers in a Room line: the kind prefix and twelve characters are enough to find the
/// full one with `changeset show` / `review show`.
fn short(id: &str) -> String {
    match id.split_once('-') {
        Some((kind, rest)) if kind.len() <= 4 => {
            format!("{kind}-{}", rest.chars().take(12).collect::<String>())
        }
        _ => id.chars().take(12).collect(),
    }
}
