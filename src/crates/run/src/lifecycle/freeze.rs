use super::*;
use crate::{Deployment, Operation, compiler::CheckpointRole};
use agency_proto::Profession;
use chat::{decode, invalid, key, reference, reject, required};
use foundation::canonical_json_sha256;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use store::{Record, RecordData, Reference, Result, Scope, Store, TrustedActor, Version};

pub(super) fn track(checks: &mut Vec<chat::Check>, record: &Record) {
    if !checks.iter().any(|c| c.key == record.key) {
        checks.push(chat::Check {
            key: record.key.clone(),
            version: Some(record.version),
        });
    }
}
pub(super) fn exact(store: &Store, r: &Reference, checks: &mut Vec<chat::Check>) -> Result<Record> {
    r.validate()?;
    let record = required(store, &r.key)?;
    if reference(&record) != *r {
        return Err(chat::stale());
    }
    track(checks, &record);
    Ok(record)
}
pub(super) fn watched(
    store: &Store,
    k: &store::ObjectKey,
    checks: &mut Vec<chat::Check>,
) -> Result<Option<Record>> {
    let record = store.get(k)?;
    checks.push(chat::Check {
        key: k.clone(),
        version: record.as_ref().map(|r| r.version),
    });
    Ok(record)
}
pub(super) fn scope(project: &str) -> Scope {
    Scope::Project(project.into())
}
pub(super) fn run_key(project: &str, id: &str) -> store::ObjectKey {
    key(scope(project), "run", id)
}
pub(super) fn manifest_key(project: &str, id: &str) -> store::ObjectKey {
    key(scope(project), "run_manifest", id)
}
pub(super) fn error(code: &'static str, message: impl Into<String>) -> store::StoreError {
    reject(code, message, "preview_current_run")
}

pub(super) struct Frozen {
    pub manifest: Manifest,
    pub task_id: Option<String>,
    pub task_state: Option<Record>,
    pub blockers: Vec<Value>,
    pub warnings: Vec<String>,
}
pub(super) fn freeze(
    store: &Store,
    actor: &TrustedActor,
    s: &Start,
    checks: &mut Vec<chat::Check>,
) -> Result<Frozen> {
    if s.run_id.trim().is_empty()
        || s.run_id.len() > 160
        || s.run_id.contains(':')
        || s.task_revisions.len() > 1
    {
        return Err(invalid(
            "Run id without colon and at most one Task Revision required",
        ));
    }
    if s.baseline.len() != 40 || !s.baseline.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(invalid("exact baseline commit SHA required"));
    }
    if s.policy.max_attempts == 0
        || s.policy.context_bytes == 0
        || s.policy.deadline_ms == 0
        || s.policy.transition_timeout_ms == 0
    {
        return Err(invalid(
            "positive frozen budget, deadline and transition timeout required",
        ));
    }
    if !s.integration.target_ref.starts_with("refs/heads/")
        || s.integration.target_ref.len() == 11
        || s.integration.target_ref.contains(['\0', '\n', ' '])
    {
        return Err(invalid(
            "fully qualified integration branch target required",
        ));
    }
    let project = chat::active_project(store, &s.project_id, s.project_version)?;
    track(checks, &project);
    let RecordData::Project { repo_id, .. } = &project.data else {
        return Err(invalid("Project required"));
    };
    if s.repo.key != repo::key(repo_id) {
        return Err(invalid("Run Repo differs from Project Repo"));
    }
    exact(store, &s.repo, checks)?;
    repo::require_active(store, repo_id)?;
    if s.engine_binding.key.scope != Scope::Control
        || s.engine_binding.key.kind != "workflow_engine_binding"
    {
        return Err(invalid("accepted workflow_engine_binding required"));
    }
    exact(store, &s.engine_binding, checks)?;
    if s.context.reference.key.scope != scope(&s.project_id)
        || s.context.reference.key.kind != "context_manifest"
    {
        return Err(invalid("root Context Manifest belongs to another Project"));
    }
    exact(store, &s.context.reference, checks)?;
    let context = context::read_manifest(store, actor, &s.project_id, &s.context.reference.key.id)
        .map_err(|e| invalid(e.to_string()))?
        .ok_or_else(|| invalid("root Context Manifest missing"))?;
    if context.digest != s.context.digest {
        return Err(error(
            "DIGEST_MISMATCH",
            "root Context Manifest digest changed",
        ));
    }
    let (workflow, revision) = crate::at(
        store,
        actor,
        &s.project_id,
        &s.workflow_id,
        s.workflow_revision,
    )?;
    track(checks, &workflow);
    let approval =
        crate::require_approved(store, &s.project_id, &s.workflow_id, s.workflow_revision)?;
    track(checks, &approval);
    let approved: Value = decode(&approval)?;
    let approved_revision: Reference = serde_json::from_value(approved["revision"].clone())?;
    let deployment_ref: Reference = serde_json::from_value(approved["deployment"].clone())?;
    if approved_revision != reference(&workflow)
        || deployment_ref.key.scope != scope(&s.project_id)
        || deployment_ref.key.kind != "engine_deployment"
    {
        return Err(invalid(
            "approval does not bind this immutable Workflow and Deployment",
        ));
    }
    let deployment_record = exact(store, &deployment_ref, checks)?;
    let deployment: Deployment = decode(&deployment_record)?;
    if deployment.workflow != reference(&workflow)
        || deployment.workflow_digest != revision.workflow_digest
        || deployment.compiled != crate::compile(&revision.workflow)?
    {
        return Err(error(
            "DIGEST_MISMATCH",
            "approved compiler mapping and Workflow differ",
        ));
    }
    let mut task_state = None;
    let mut task_id = None;
    let mut blockers = vec![];
    if let Some(target) = s.task_revisions.first() {
        if target.key.scope != scope(&s.project_id) || target.key.kind != "task_revision" {
            return Err(invalid("Task Revision belongs to another Project"));
        }
        let revision_record = exact(store, target, checks)?;
        let r: task::Revision = decode(&revision_record)?;
        let (id, _) = target
            .key
            .id
            .rsplit_once(':')
            .ok_or_else(|| invalid("Task Revision identity required"))?;
        let (record, t) = task::task(store, &s.project_id, id)?;
        track(checks, &record);
        if t.project_id != s.project_id
            || t.repo_id != *repo_id
            || t.lifecycle != "open"
            || t.archived
        {
            return Err(error(
                "TASK_NOT_OPEN",
                "Run requires an open Task in its Project",
            ));
        }
        let current = t.revision.as_ref().ok_or_else(|| {
            error(
                "CONTRACT_REQUIRED",
                "Run-bound Task has no adopted contract",
            )
        })?;
        if t.pending_contract.is_some() {
            return Err(error(
                "CONTRACT_DIVERGED",
                "adopt the pending contract before starting Run",
            ));
        }
        if serde_json::to_value(current)? != serde_json::to_value(&r)?
            || target.key.id != format!("{}:{}", t.id, current.number)
        {
            return Err(error("VERSION_CONFLICT", "Task current Revision changed"));
        }
        let contract: task::Contract =
            serde_json::from_slice(&store.read_material(actor, &current.material)?)?;
        if contract.acceptance.is_empty()
            || contract.scope.trim().is_empty()
            || contract.expected_outcome.trim().is_empty()
        {
            return Err(error("CONTRACT_REQUIRED", "Task contract is incomplete"));
        }
        let (source_record, source) = task::source(store, &t.repo_id, &t.source_id)?;
        track(checks, &source_record);
        let (snap_record, snap) = task::latest(store, &source)?;
        track(checks, &snap_record);
        let entity = t
            .entity
            .as_ref()
            .ok_or_else(|| error("SOURCE_UNAVAILABLE", "Task has no bound card"))?;
        let card = snap
            .cards
            .iter()
            .find(|c| c.entity == *entity && !c.tombstone)
            .ok_or_else(|| {
                error(
                    "SOURCE_UNAVAILABLE",
                    "Task card missing from source readback",
                )
            })?;
        blockers = card
            .dependencies
            .blocked_by
            .iter()
            .filter(|b| !closed(b))
            .cloned()
            .collect();
        task_id = Some(t.id);
        task_state = Some(record);
    }
    let mut expected = BTreeMap::new();
    for c in deployment
        .compiled
        .checkpoints
        .iter()
        .filter(|c| c.role == CheckpointRole::Node)
    {
        if let Operation::External { seats, .. } = &c.node.operation {
            for seat in seats {
                expected.insert((c.id.clone(), seat.id.clone()), seat);
            }
        }
    }
    let mut selected = vec![];
    let mut seen = BTreeSet::new();
    for pick in &s.picks {
        let identity = (pick.checkpoint.clone(), pick.seat.clone());
        let seat = expected
            .get(&identity)
            .ok_or_else(|| invalid("pick is not a frozen Workflow Seat"))?;
        if !seen.insert(identity) || pick.max_context_bytes > s.policy.context_bytes {
            return Err(invalid("duplicate Seat or selection exceeds Run budget"));
        }
        let accepted = exact(store, &pick.profession, checks)?;
        let profession: Profession = decode(&accepted)?;
        if profession.reference.id != seat.profession
            || pick.profession.key.kind != "profession"
            || pick.profession.key.scope != Scope::Control
            || !evidence_matches(&profession.capabilities, &seat.evidence)
        {
            return Err(error(
                "SELECTION_REQUIREMENT_UNMET",
                "accepted Profession does not fulfill Seat requirements",
            ));
        }
        if pick.agency.key.scope != Scope::Control || pick.agency.key.kind != "agency_binding" {
            return Err(invalid("accepted Agency binding required"));
        }
        let agency = exact(store, &pick.agency, checks)?;
        let agency: participant::Binding = decode(&agency)?;
        let mut required_skills = seat.required_skills.clone();
        for claim in profession.skills.iter().filter(|c| c.required) {
            let reference = Reference {
                key: key(Scope::Control, "skill", &claim.reference.id),
                version: Version::Revision(claim.reference.digest.clone()),
            };
            if required_skills
                .iter()
                .any(|r| r.key == reference.key && *r != reference)
            {
                return Err(invalid(
                    "Seat and Profession require different versions of one Skill",
                ));
            }
            if !required_skills.contains(&reference) {
                required_skills.push(reference);
            }
        }
        let skills: Vec<_> = required_skills
            .iter()
            .map(|r| participant::selection::Skill {
                reference: r.clone(),
                digest: agency
                    .catalog
                    .skills
                    .iter()
                    .find(|c| {
                        c.reference.id == r.key.id
                            && r.version == Version::Revision(c.reference.digest.clone())
                    })
                    .and_then(|c| {
                        c.verification
                            .as_ref()
                            .filter(|v| {
                                v.source == agency_proto::EvidenceLevel::Unmediated
                                    && v.readback_digest == c.reference.digest
                            })
                            .map(|_| c.reference.digest.clone())
                    }),
            })
            .collect();
        let selection = participant::selection::Selection {
            room_id: String::new(),
            selected_item: pick.profession.clone(),
            profession: pick.profession.clone(),
            profession_digest: profession.reference.digest.clone(),
            agency: pick.agency.clone(),
            required_skills: skills.clone(),
            optional_skills: vec![],
            worker_profiles: pick.worker_profiles.clone(),
            responsibility: format!("{}/{}", pick.checkpoint, seat.id),
            permission: json!({"allow":pick.permissions}),
            budget: json!({"max_bytes":pick.max_context_bytes}),
            display_name: String::new(),
            persona_tags: vec![],
        };
        let validated = participant::selection::validate_roster(store, &project, &[selection])?;
        for r in &validated.dependencies {
            track(checks, r);
        }
        let profile_digests = pick
            .worker_profiles
            .iter()
            .map(|r| participant::profiles::profile_at(store, r).map(|(r, _)| r.revision_digest))
            .collect::<Result<Vec<_>>>()?;
        let content = pick
            .frozen_bundle
            .as_ref()
            .map(|r| -> Result<Option<String>> {
                if r.key.scope != scope(&s.project_id) || r.key.kind != "context_bundle" {
                    return Err(invalid("frozen Bundle belongs to another Project"));
                }
                exact(store, r, checks)?;
                let bundle = context::read_bundle(store, actor, &s.project_id, &r.key.id)
                    .map_err(|e| invalid(e.to_string()))?
                    .ok_or_else(|| invalid("Bundle missing"))?;
                if bundle.document.manifest.digest != s.context.digest {
                    return Err(invalid("Bundle differs from frozen root Manifest"));
                }
                delivered_content_digest(&bundle.document, &required_skills)
            })
            .transpose()?
            .flatten();
        selected.push(Selected {
            checkpoint: pick.checkpoint.clone(),
            requirement: (*seat).clone(),
            selection: Reference {
                key: key(
                    scope(&s.project_id),
                    "run_selection",
                    &format!("{}:{}:{}", s.run_id, pick.checkpoint, seat.id),
                ),
                version: Version::State(1),
            },
            profession: pick.profession.clone(),
            profession_digest: profession.reference.digest,
            agency: pick.agency.clone(),
            worker_profiles: pick.worker_profiles.clone(),
            profile_digests,
            required_skills: skills,
            permissions: pick.permissions.clone(),
            max_context_bytes: pick.max_context_bytes,
            frozen_bundle: pick.frozen_bundle.clone(),
            frozen_content_digest: content,
        });
    }
    if seen.len() != expected.len() {
        return Err(error(
            "SELECTION_REQUIRED",
            "every frozen Seat needs an independently selected accepted candidate",
        ));
    }
    let mut warnings = vec![];
    for c in &deployment.compiled.checkpoints {
        if let Operation::External {
            gate: Some(gate), ..
        } = &c.node.operation
        {
            let group: Vec<_> = selected
                .iter()
                .filter(|s| s.checkpoint == c.id && s.requirement.required)
                .collect();
            let actual: Vec<_> = group
                .iter()
                .map(|s| (s.requirement.id.clone(), s.profile_digests[0].clone()))
                .collect();
            if !(gate_tally::Diversity {
                distinct_profiles: gate.distinct_worker_profiles,
                avoid_author_profile: false,
            })
            .check(None, &actual)
            .is_empty()
            {
                return Err(error(
                    "SEAT_DIVERSITY_REQUIRED",
                    "initial selected Worker Profiles violate declared diversity",
                ));
            }
            let seats = group
                .iter()
                .map(|s| {
                    let candidates: Vec<_> = s.profile_digests.iter().map(String::as_str).collect();
                    let mut seat = gate_tally::Seat::new(
                        &s.requirement.id,
                        s.frozen_content_digest.as_deref().unwrap_or("unknown"),
                        &candidates,
                    );
                    seat.delivered = s.frozen_content_digest.clone();
                    seat
                })
                .collect();
            let tally = gate_tally::tally(&gate_tally::Gate {
                quorum: gate.quorum as usize,
                seats,
            })
            .map_err(|e| invalid(format!("Gate input: {e:?}")))?;
            if matches!(tally.outcome,gate_tally::Outcome::NoPass{upper_bound} if upper_bound<gate.quorum as usize)
            {
                return Err(error(
                    "QUORUM_UNREACHABLE",
                    "all legal candidate keys cannot reach the frozen quorum",
                ));
            }
            if group.iter().any(|s| s.frozen_content_digest.is_none()) {
                warnings.push(format!(
                    "{}: delivered content unknown; not assumed identical",
                    c.id
                ));
            }
        }
    }
    for destination in s
        .placement
        .destination
        .iter()
        .chain(&s.placement.allowed_destinations)
    {
        if destination.key.scope != scope(&s.project_id)
            || !matches!(destination.key.kind.as_str(), "room_binding" | "task")
        {
            return Err(invalid(
                "placement is only result delivery to a frozen Project Room or Task",
            ));
        }
        exact(store, destination, checks)?;
    }
    Ok(Frozen {
        manifest: Manifest {
            project: reference(&project),
            selection_policy_digest: match &project.data {
                RecordData::Project { settings, .. } => {
                    canonical_json_sha256(&settings.selection_policy)?
                }
                _ => unreachable!(),
            },
            run_id: s.run_id.clone(),
            task_revision: s.task_revisions.first().cloned(),
            workflow: reference(&workflow),
            approval: reference(&approval),
            deployment: deployment_ref,
            repo: s.repo.clone(),
            baseline: s.baseline.clone(),
            context: s.context.clone(),
            engine_binding: s.engine_binding.clone(),
            selected,
            frozen_rework_limits: deployment.compiled.frozen_rework_limits,
            policy: s.policy.clone(),
            integration: s.integration.clone(),
            placement: s.placement.clone(),
            readback_proof: approved["proof"].clone(),
            authorized_by: actor.0.principal.clone(),
        },
        task_id,
        task_state,
        blockers,
        warnings,
    })
}
fn closed(value: &Value) -> bool {
    [
        value.get("state"),
        value.get("issue_state"),
        value.pointer("/issue/state"),
    ]
    .into_iter()
    .flatten()
    .any(|s| matches!(s.as_str(), Some("closed" | "done" | "completed")))
        || value.get("is_closed").and_then(Value::as_bool) == Some(true)
}
fn evidence_matches(actual: &agency_proto::Capabilities, required: &crate::EvidenceLevel) -> bool {
    // Lower evidence is checked on each Proposal; cursor support is not an
    // evidence grade. Only the explicit direct-tool capability is decidable here.
    !matches!(required, crate::EvidenceLevel::Unmediated) || actual.tool_execution_unmediated
}
/// Same content key for startup and future voting. Consumer, metering, retention,
/// locator paths and delivery mode are excluded; unknown bytes stay unknown.
pub fn delivered_content_digest(
    bundle: &agency_proto::context::Bundle,
    skills: &[Reference],
) -> Result<Option<String>> {
    use agency_proto::context::Delivery;
    if bundle
        .entries
        .iter()
        .any(|e| matches!(e.delivery, Delivery::Recall { .. }))
    {
        return Ok(None);
    }
    let mut entries: Vec<_> = bundle
        .entries
        .iter()
        .map(|e| json!({"source":e.source,"bytes_digest":e.bytes_digest}))
        .collect();
    entries.sort_by_key(Value::to_string);
    let mut skills = skills.to_vec();
    skills.sort_by_key(|r| serde_json::to_string(r).unwrap_or_default());
    Ok(Some(canonical_json_sha256(
        &json!({"entries":entries,"required_skills":skills}),
    )?))
}
