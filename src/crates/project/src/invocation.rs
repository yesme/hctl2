//! Room-owned authorization. State revisions never become dispatch generations.
mod lifecycle;
pub use lifecycle::*;
mod results;
pub use results::{admit_result, admit_sealed_result, sealing_input};
mod write;
pub use write::{WriteInput, WritePreview, confirm_write_stop, review_policy_input};

use crate::{
    decode, invalid, key, project, readonly, reference, reject, required, stale, value_record,
};
use agency_proto::{ExecutionSpec, FrozenRef, InputPolicy, Owner, OwnerKind, Profession, Sealed};
use context::{Assembly, Delivery};
use foundation::{bytes_sha256, canonical_json_sha256};
use participant::{frozen, port_error, profiles::WorkerProfile, selection::Selection};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use store::{
    Command, Expected, MaterialRef, Record, RecordData, Reference, RoomKind, RoomState, Scope,
    Store, TrustedActor, Version,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub key: String,
    pub project_id: String,
    pub room_id: String,
    #[serde(default)]
    pub task_id: Option<String>,
    /// Explicit admitted version for the review-comment source; never inferred from Task.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_change_set_revision: Option<Reference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub write: Option<WriteInput>,
    /// Exact selection ID or responsibility, never a display name.
    pub target: String,
    pub profile: Reference,
    pub request: String,
    pub budget: u64,
    pub deadline_ms: u64,
    pub retry_of: Option<Reference>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preview {
    pub input: Input,
    pub consumer: Owner,
    project: FrozenRef,
    room: Reference,
    selection: Record,
    profile: FrozenRef,
    configuration: WorkerProfile,
    profession: Profession,
    binding: FrozenRef,
    repo: FrozenRef,
    policy_digest: String,
    pub required_skills: Vec<FrozenRef>,
    optional_skill_degradations: Vec<participant::selection::SkillDegradation>,
    pub brief: Option<MaterialRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub write: Option<WritePreview>,
    publish_review_requires_confirmation: bool,
    checks: Vec<Reference>,
}

/// The root record is the semantic authorization; lifecycle has its own CAS version.
/// Revocation advances this root, invalidating every old owner reference atomically.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invocation {
    pub preview: Preview,
    pub spec: Sealed<ExecutionSpec>,
    pub authorization: Authorization,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Authorization {
    pub write: bool,
    pub valid: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorizing_actor: Option<store::Actor>,
}

pub fn invocation_id(store: &Store, project: &str, command_key: &str) -> String {
    format!(
        "invocation-{}",
        bytes_sha256(format!("{}:{project}:{command_key}", store.control_id()).as_bytes())
    )
}
pub fn owner_key(project: &str, id: &str) -> store::ObjectKey {
    key(Scope::Project(project.into()), "room_invocation", id)
}

/// Human entry only. Context assembly and all provider calls stay outside this function.
pub fn prepare(
    store: &Store,
    actor: &TrustedActor,
    input: Input,
    now_ms: u64,
) -> store::Result<Preview> {
    let scoped = chat::owner(actor, &input.project_id)?;
    if input.key.trim().is_empty()
        || input.key.trim() != input.key
        || input.target.trim().is_empty()
        || input.request.trim().is_empty()
        || input.deadline_ms <= now_ms
        || input.budget == 0
    {
        return Err(invalid(
            "key, exact target, request, positive budget and future deadline required",
        ));
    }
    let p = project(store, &input.project_id)?;
    if readonly(&p) {
        return Err(reject(
            "PROJECT_READ_ONLY",
            "Project is archived",
            "restore_project",
        ));
    }
    let RecordData::Project {
        repo_id, settings, ..
    } = &p.data
    else {
        return Err(invalid("Project required"));
    };
    repo::require_active(store, repo_id)?;
    let repo = required(store, &repo::key(repo_id))?;
    let scope = p.key.scope.clone();
    let room = required(store, &key(scope.clone(), "room", &input.room_id))?;
    let RecordData::Room {
        room_kind,
        state: RoomState::Active,
    } = &room.data
    else {
        return Err(reject(
            "ROOM_READ_ONLY",
            "active Room required",
            "choose_active_room",
        ));
    };
    let binding = required(store, &key(scope.clone(), "room_binding", &input.room_id))?;
    let chat: chat::Room = decode(&binding)?;
    if chat.project_id != input.project_id
        || chat.id != input.room_id
        || chat.matrix_room_id.is_none()
    {
        return Err(reject(
            "ROOM_NOT_READY",
            "exact Room binding is not ready",
            "inspect_room",
        ));
    }
    if *room_kind == RoomKind::Topic && chat.brief.is_none() {
        return Err(reject(
            "BRIEF_REQUIRED",
            "Topic requires its confirmed brief",
            "confirm_brief",
        ));
    }
    if let Some(brief) = &chat.brief {
        store.read_material(&scoped, brief)?;
    }
    let roster = required(store, &key(scope.clone(), "room_roster", &input.room_id))?;
    let selection = participant::selection::resolve_room_candidate(
        store,
        &input.project_id,
        &input.room_id,
        &input.target,
    )?;
    let selected: Selection = decode(&selection)?;
    let validated =
        participant::selection::validate_roster(store, &p, std::slice::from_ref(&selected))?;
    if !selected.worker_profiles.contains(&input.profile) {
        return Err(reject(
            "PROFILE_NOT_ALLOWED",
            "Profile is outside selected candidates",
            "select_allowed_profile",
        ));
    }
    let (profile_record, configuration) = participant::profiles::profile_at(store, &input.profile)?;
    if input.budget > configuration.max_context_bytes {
        return Err(reject(
            "BUDGET_EXCEEDED",
            "requested bytes exceed Profile",
            "narrow_budget",
        ));
    }
    if !configuration
        .permissions
        .iter()
        .any(|p| p == "context.read")
    {
        return Err(reject(
            "PERMISSION_DENIED",
            "Invocation needs context.read",
            "select_allowed_profile",
        ));
    }
    let accepted = required(store, &selected.profession.key)?;
    let profession: Profession = decode(&accepted)?;
    let agency = required(store, &selected.agency.key)?;
    let promises: participant::Binding = decode(&agency)?;
    let required_skills = selected
        .required_skills
        .iter()
        .map(|skill| {
            let matches: Vec<_> = promises
                .catalog
                .skills
                .iter()
                .filter(|claim| {
                    claim.reference.id == skill.reference.key.id
                        && skill.reference.version
                            == Version::Revision(claim.reference.digest.clone())
                })
                .collect();
            if matches.len() != 1 {
                return Err(invalid("unique required Skill revision required"));
            }
            Ok(matches[0].reference.clone())
        })
        .collect::<store::Result<Vec<_>>>()?;
    let mut checks = vec![
        reference(&p),
        reference(&repo),
        reference(&room),
        reference(&binding),
        reference(&roster),
        reference(&selection),
    ];
    if let Some(review) = &input.review_change_set_revision {
        if review.key.scope != Scope::Repo(repo_id.clone())
            || review.key.kind != "changeset_revision"
        {
            return Err(reject(
                "REVIEW_VERSION_MISMATCH",
                "review version is outside this Repo",
                "select_admitted_revision",
            ));
        }
        let revision = required(store, &review.key)?;
        if reference(&revision) != *review {
            return Err(stale());
        }
        let _ = repo::changeset::get_revision(store, &review.key.id)?;
        checks.push(review.clone());
    }
    checks.extend(validated.dependencies.iter().map(reference));
    if let Some(previous) = &input.retry_of {
        if previous.key.scope != scope || previous.key.kind != "room_invocation" {
            return Err(invalid("retry must name an Invocation in this Project"));
        }
        let old = required(store, &previous.key)?;
        let original: Invocation = decode(&old)?;
        let (_, state) = lifecycle(store, &input.project_id, &old.key.id)?;
        if reference(&old) != *previous
            || current_authorization(store, previous, now_ms)?
            || !state.state.terminal()
            || original.preview.input.room_id != input.room_id
        {
            return Err(reject(
                "RETRY_NOT_ALLOWED",
                "old authorization must be terminal and revoked in the same Room",
                "inspect_original_invocation",
            ));
        }
        checks.push(previous.clone());
    }
    let consumer = Owner {
        project: input.project_id.clone(),
        kind: OwnerKind::RoomInvocation,
        id: invocation_id(store, &input.project_id, &input.key),
        generation: 1,
    };
    if input
        .retry_of
        .as_ref()
        .is_some_and(|r| r.key.id == consumer.id)
    {
        return Err(invalid("retry requires a new command key"));
    }
    let write = write::prepare(store, &input, &consumer, &p, &repo, &configuration)?;
    if let Some(write) = &write {
        checks.push(write.policy_record.clone());
        if let Some(previous) = &write.lease.previous {
            checks.push(previous.clone());
        }
    }
    Ok(Preview {
        input,
        consumer,
        project: frozen(&p),
        room: reference(&room),
        selection,
        profile: frozen(&profile_record),
        configuration,
        profession,
        binding: frozen(&agency),
        repo: frozen(&repo),
        policy_digest: canonical_json_sha256(&settings.selection_policy)?,
        required_skills,
        optional_skill_degradations: validated.optional_skill_degradations,
        brief: chat.brief,
        write,
        publish_review_requires_confirmation: settings.publish_review_requires_confirmation,
        checks,
    })
}

fn sealed_ref(id: &str, digest: &str) -> FrozenRef {
    FrozenRef {
        id: id.into(),
        revision: digest.into(),
        digest: digest.into(),
    }
}
fn exact_entry(assembly: &Assembly, id: &str, digest: &str) -> bool {
    assembly.bundle.document.entries.iter().any(|e| {
        e.source.id == id && e.required && e.bytes_digest == digest
            && matches!(&e.delivery, Delivery::Inline { bytes } | Delivery::Pointer { bytes, .. } if agency_proto::hash(bytes) == digest)
    })
}

/// Derive the Spec; the caller cannot supply a wider permission or a different executor.
fn execution_spec(preview: &Preview, assembly: &Assembly) -> store::Result<Sealed<ExecutionSpec>> {
    assembly.manifest.verify().map_err(port_error)?;
    assembly.bundle.verify().map_err(port_error)?;
    assembly
        .bundle
        .document
        .validate_delivery()
        .map_err(port_error)?;
    let m = &assembly.manifest.document;
    let b = &assembly.bundle.document;
    if b.consumer != preview.consumer
        || b.budget != preview.input.budget
        || m.budget != b.budget
        || m.scope != format!("project {}", preview.input.project_id)
        || b.manifest != sealed_ref(&m.id, &assembly.manifest.digest)
        || m.permission_digest != b.permission_digest
        || m.required_skills != preview.required_skills
        || !exact_entry(
            assembly,
            &format!("invocation-request/{}", preview.consumer.id),
            &agency_proto::hash(preview.input.request.as_bytes()),
        )
        || if let Some(write) = &preview.write {
            !exact_entry(
                assembly,
                &format!("write-boundary/{}", preview.consumer.id),
                &agency_proto::hash(&write.context_bytes()?),
            )
        } else {
            false
        }
        || preview
            .required_skills
            .iter()
            .any(|s| !exact_entry(assembly, &s.id, &s.digest))
        || preview.brief.as_ref().is_some_and(|brief| {
            !b.entries
                .iter()
                .any(|e| e.required && e.bytes_digest == brief.byte_digest)
        })
    {
        return Err(reject(
            "CONTEXT_MISMATCH",
            "consumer, scope, request, required Skills, brief or policy differs",
            "rebuild_preview",
        ));
    }
    let spec = ExecutionSpec {
        owner: preview.consumer.clone(),
        project: preview.project.clone(),
        selection: frozen(&preview.selection),
        selection_policy_digest: preview.policy_digest.clone(),
        profession: preview.profession.clone(),
        profile: preview.profile.clone(),
        manifest: sealed_ref(&m.id, &assembly.manifest.digest),
        bundle: sealed_ref(&b.id, &assembly.bundle.digest),
        binding: preview.binding.clone(),
        required_capabilities: preview.configuration.required_capabilities.clone(),
        input_policy: InputPolicy::NoInput,
        permission_digest: b.permission_digest.clone(),
        permissions: preview.configuration.permissions.clone(),
        budget: preview.input.budget,
        deadline_ms: preview.input.deadline_ms,
        repo: Some(preview.repo.clone()),
        base: preview
            .write
            .as_ref()
            .map(|w| w.lease.pending.baseline_commit.clone()),
        delivery_scope: vec![preview.input.room_id.clone()],
        write_lease: preview
            .write
            .as_ref()
            .map(write::lease_reference)
            .transpose()?,
        review_publish_policy: preview
            .write
            .as_ref()
            .map(|w| w.review_publish_policy.clone()),
        idempotency_key: format!("prepare:{}", preview.consumer.id),
    };
    spec.validate().map_err(port_error)?;
    Sealed::new(spec).map_err(port_error)
}

/// Step 1: authorization, immutable Spec, pending lifecycle and prepare outbox commit together.
/// Context is already saved/admitted; saving it alone does not grant execution authority.
pub fn start(
    store: &mut Store,
    actor: &TrustedActor,
    preview: &Preview,
    assembly: &Assembly,
    now_ms: u64,
) -> store::Result<Value> {
    let mut scoped = chat::owner(actor, &preview.input.project_id)?;
    if let Some(write) = &preview.write {
        scoped
            .0
            .permission_scope
            .push(Scope::Repo(write.lease.pending.repo_id.clone()));
    }
    let spec = execution_spec(preview, assembly)?;
    let root_key = owner_key(&preview.input.project_id, &preview.consumer.id);
    let input = json!({"preview":preview,"spec":spec});
    let command = Command {
        command_id: format!("invocation-start:{}", preview.consumer.id),
        idempotency_key: preview.input.key.clone(),
        actor: scoped.0.clone(),
        target: root_key.clone(),
        expected: Expected::Absent,
        binding: preview.room.clone(),
        operation: "invocation.start".into(),
        input_digest: Command::digest_input("invocation.start", &input)?,
        input,
    };
    if store.get(&root_key)?.is_some() {
        // Store validates the original command fingerprint before returning its result.
        return store.submit(
            store.generation(),
            &scoped,
            &command,
            None,
            |_| Err(stale()),
        );
    }
    let current = prepare(store, actor, preview.input.clone(), now_ms)?;
    if serde_json::to_value(&current)? != serde_json::to_value(preview)? {
        return Err(stale());
    }
    let manifest = context::read_manifest(
        store,
        &scoped,
        &preview.input.project_id,
        &assembly.manifest.document.id,
    )
    .map_err(port_error)?;
    let bundle = context::read_bundle(
        store,
        &scoped,
        &preview.input.project_id,
        &assembly.bundle.document.id,
    )
    .map_err(port_error)?;
    if manifest.as_ref() != Some(&assembly.manifest) || bundle.as_ref() != Some(&assembly.bundle) {
        return Err(reject(
            "CONTEXT_NOT_ADMITTED",
            "exact Context materials must be admitted before dispatch",
            "save_context",
        ));
    }
    let manifest_record = context::manifest_record(
        store,
        &preview.input.project_id,
        &assembly.manifest.document.id,
    )
    .map_err(port_error)?
    .ok_or_else(stale)?;
    let bundle_record = context::bundle_record(
        store,
        &preview.input.project_id,
        &assembly.bundle.document.id,
    )
    .map_err(port_error)?
    .ok_or_else(stale)?;
    let mut root = value_record(
        root_key,
        1,
        &Invocation {
            preview: preview.clone(),
            spec: spec.clone(),
            authorization: Authorization {
                write: preview.write.is_some(),
                valid: true,
                authorizing_actor: preview.write.as_ref().map(|_| scoped.0.clone()),
            },
        },
    )?;
    root.sources = vec![
        preview.room.clone(),
        reference(&preview.selection),
        preview.input.profile.clone(),
        reference(&manifest_record),
        reference(&bundle_record),
    ];
    if let Some(previous) = &preview.input.retry_of {
        root.sources.push(previous.clone());
    }
    let dispatch =
        participant::dispatch::plan(store, &preview.consumer.id, &root, &spec, &assembly.bundle)?;
    let state = value_record(
        state_key(&preview.input.project_id, &preview.consumer.id),
        1,
        &Lifecycle {
            state: State::Pending,
            reason: None,
        },
    )?;
    let result = json!({"invocation_id":preview.consumer.id,"owner":reference(&root),"state":"pending","state_version":1,"prepare_effect":dispatch.effect.intent_id,"spec_digest":spec.digest});
    store.submit(store.generation(), &scoped, &command, None, |tx| {
        for expected in preview
            .checks
            .iter()
            .cloned()
            .chain(dispatch.dependencies.iter().map(reference))
            .chain([reference(&manifest_record), reference(&bundle_record)])
        {
            if tx.get(&expected.key)?.as_ref().map(reference) != Some(expected) {
                return Err(stale());
            }
        }
        if let Some(write) = &preview.write {
            repo::changeset::acquire_lease(tx, &write.lease)?;
        }
        tx.put(&root)?;
        tx.put(&state)?;
        tx.put(&dispatch.record)?;
        tx.enqueue_effect(&dispatch.effect)?;
        Ok(result)
    })
}

pub fn invocation(store: &Store, project: &str, id: &str) -> store::Result<(Record, Invocation)> {
    let root = required(store, &owner_key(project, id))?;
    let invocation = decode(&root)?;
    Ok((root, invocation))
}

/// Used before dispatch, input, tickets and result admission. Unreachable is not revocation.
pub fn current_authorization(store: &Store, owner: &Reference, now_ms: u64) -> store::Result<bool> {
    let Scope::Project(project_id) = &owner.key.scope else {
        return Err(invalid("Project-scoped Invocation required"));
    };
    if owner.key.kind != "room_invocation" {
        return Err(invalid("Invocation owner required"));
    }
    let Some(root) = store.get(&owner.key)? else {
        return Ok(false);
    };
    let value: Invocation = decode(&root)?;
    let (_, state) = lifecycle(store, project_id, &owner.key.id)?;
    Ok(reference(&root) == *owner
        && value.authorization.valid
        && !state.state.terminal()
        && value.spec.document.deadline_ms > now_ms
        && !readonly(&project(store, project_id)?))
}
