//! Candidate validation; Room ownership and writing remain in Project.
use crate::{Binding, decode, invalid, reference, reject};
use agency_proto::{EvidenceLevel, Profession};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use store::{Record, RecordData, Reference, Scope, Store, Version};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub room_id: String,
    pub selected_item: Reference,
    pub profession: Reference,
    pub profession_digest: String,
    pub agency: Reference,
    pub required_skills: Vec<Skill>,
    pub optional_skills: Vec<Skill>,
    pub worker_profiles: Vec<Reference>,
    pub responsibility: String,
    pub permission: Value,
    pub budget: Value,
    pub display_name: String,
    pub persona_tags: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Skill {
    /// Exact content digest in Revision; the accepted catalog retains provider revision.
    pub reference: Reference,
    /// Some means known from a matching unmediated readback; None is unknown, not missing.
    pub digest: Option<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SelectionPolicy {
    pub allowed_agencies: Option<Vec<String>>,
    pub allowed_professions: Option<Vec<String>>,
    pub permissions: Option<Vec<String>>,
    pub max_context_bytes: Option<u64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectionPermissions {
    pub allow: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectionBudget {
    pub max_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillDegradation {
    pub selection_index: usize,
    pub reference: Reference,
}
pub struct ValidatedRoster {
    pub dependencies: Vec<Record>,
    pub optional_skill_degradations: Vec<SkillDegradation>,
}

fn at(store: &Store, target: &Reference, kind: &str) -> store::Result<Record> {
    target.validate()?;
    if target.key.scope != Scope::Control
        || target.key.kind != kind
        || !matches!(target.version, Version::State(_))
    {
        return Err(invalid(format!(
            "{kind} with State version in control scope required"
        )));
    }
    let record = store.get(&target.key)?.ok_or_else(|| {
        reject(
            "CANDIDATE_NOT_ACCEPTED",
            "candidate reference is not accepted",
            "accept_candidate",
        )
    })?;
    if reference(&record) != *target {
        return Err(reject(
            "VERSION_CONFLICT",
            "candidate changed",
            "preview_again",
        ));
    }
    Ok(record)
}

/// Returns frozen dependencies so Project can recheck them in its admission transaction.
pub fn validate_roster(
    store: &Store,
    project: &Record,
    selections: &[Selection],
) -> store::Result<ValidatedRoster> {
    let RecordData::Project { settings, .. } = &project.data else {
        return Err(invalid("Project required"));
    };
    let policy: SelectionPolicy = serde_json::from_value(settings.selection_policy.clone())?;
    let mut dependencies = vec![];
    let mut optional_skill_degradations = vec![];
    for (selection_index, selected) in selections.iter().enumerate() {
        if selected.worker_profiles.is_empty() {
            return Err(invalid(
                "at least one exact Worker Profile candidate required",
            ));
        }
        if selected.selected_item != selected.profession {
            return Err(invalid(
                "selected item must be the exact accepted Profession",
            ));
        }
        let binding_record = at(store, &selected.agency, "agency_binding")?;
        let accepted = at(store, &selected.profession, "profession")?;
        let binding: Binding = decode(&binding_record)?;
        let profession: Profession = decode(&accepted)?;
        profession.reference.validate().map_err(crate::port_error)?;
        profession.harness.validate().map_err(crate::port_error)?;
        if !accepted.sources.contains(&selected.agency)
            || !binding.catalog.professions.contains(&profession)
            || profession.reference.digest != selected.profession_digest
        {
            return Err(reject(
                "PROFESSION_CHANGED",
                "Profession, binding or frozen terms differ",
                "accept_candidate",
            ));
        }
        if policy
            .allowed_agencies
            .as_ref()
            .is_some_and(|allowed| !allowed.contains(&binding.id))
            || policy
                .allowed_professions
                .as_ref()
                .is_some_and(|allowed| !allowed.contains(&profession.reference.id))
        {
            return Err(reject(
                "SELECTION_POLICY_DENIED",
                "Agency or Profession not allowed by Project",
                "select_allowed_candidate",
            ));
        }
        let permissions: SelectionPermissions =
            serde_json::from_value(selected.permission.clone())?;
        crate::profiles::validate_permissions(&permissions.allow)?;
        if policy
            .permissions
            .as_ref()
            .is_some_and(|allowed| permissions.allow.iter().any(|p| !allowed.contains(p)))
        {
            return Err(reject(
                "SELECTION_POLICY_DENIED",
                "selection expands Project permissions",
                "narrow_permissions",
            ));
        }
        let budget: SelectionBudget = serde_json::from_value(selected.budget.clone())?;
        if budget.max_bytes == 0
            || policy
                .max_context_bytes
                .is_some_and(|limit| budget.max_bytes > limit)
        {
            return Err(reject(
                "SELECTION_POLICY_DENIED",
                "selection exceeds Project budget",
                "narrow_budget",
            ));
        }
        for (index, target) in selected.worker_profiles.iter().enumerate() {
            if selected.worker_profiles[..index].contains(target) {
                return Err(invalid("duplicate profile candidate"));
            }
            let (record, profile) = crate::profiles::profile_at(store, target)?;
            if profile.harness != profession.harness || profile.model != profession.model {
                return Err(reject(
                    "PROFILE_MISMATCH",
                    "profile does not describe the selected Profession",
                    "select_matching_profile",
                ));
            }
            if profile
                .permissions
                .iter()
                .any(|p| !permissions.allow.contains(p))
                || profile.max_context_bytes > budget.max_bytes
            {
                return Err(reject(
                    "PROFILE_SCOPE_EXCEEDED",
                    "profile exceeds selection permissions or budget",
                    "narrow_profile",
                ));
            }
            profession
                .capabilities
                .fulfills(&profile.required_capabilities)
                .map_err(crate::port_error)?;
            dependencies.push(record);
        }
        let skills = selected
            .required_skills
            .iter()
            .chain(&selected.optional_skills)
            .collect::<Vec<_>>();
        for (index, skill) in skills.iter().enumerate() {
            skill.reference.validate()?;
            if skill.reference.key.scope != Scope::Control
                || skill.reference.key.kind != "skill"
                || skills[..index]
                    .iter()
                    .any(|s| s.reference.key.id == skill.reference.key.id)
            {
                return Err(invalid("unique exact Skill references required"));
            }
            let Version::Revision(digest) = &skill.reference.version else {
                return Err(invalid("Skill requires its exact content digest"));
            };
            let matching: Vec<_> = binding
                .catalog
                .skills
                .iter()
                .filter(|claim| {
                    claim.reference.id == skill.reference.key.id
                        && &claim.reference.digest == digest
                })
                .collect();
            if matching.len() > 1 {
                return Err(invalid("Skill catalog reference is ambiguous"));
            }
            let available = matching.first().copied();
            let required = index < selected.required_skills.len();
            if required && available.is_none() {
                return Err(reject(
                    "SKILL_MISSING",
                    "required Skill not available in accepted catalog",
                    "select_available_skill",
                ));
            }
            if !required && available.is_none() {
                optional_skill_degradations.push(SkillDegradation {
                    selection_index,
                    reference: skill.reference.clone(),
                });
            }
            if let Some(claim) = available {
                claim.reference.validate().map_err(crate::port_error)?;
                if let Some(report) = &claim.verification {
                    report.report.validate().map_err(crate::port_error)?;
                    if report.readback_digest != claim.reference.digest {
                        return Err(reject(
                            "SKILL_DIGEST_MISMATCH",
                            "catalog declaration and verification readback differ",
                            "verify_skill",
                        ));
                    }
                }
            }
            if let Some(known) = &skill.digest {
                if known != digest {
                    return Err(reject(
                        "SKILL_DIGEST_MISMATCH",
                        "Skill declaration and readback differ",
                        "verify_skill",
                    ));
                }
                let verified = available.and_then(|claim| claim.verification.as_ref());
                if !verified.is_some_and(|report| {
                    report.source == EvidenceLevel::Unmediated && &report.readback_digest == known
                }) {
                    return Err(reject(
                        "SKILL_UNKNOWN",
                        "cannot upgrade an unverified declaration to known",
                        "verify_skill",
                    ));
                }
            }
        }
        for claim in profession.skills.iter().filter(|claim| claim.required) {
            if !selected.required_skills.iter().any(|skill| {
                skill.reference.key.id == claim.reference.id
                    && skill.reference.version == Version::Revision(claim.reference.digest.clone())
            }) {
                return Err(reject(
                    "SKILL_MISSING",
                    "selection omitted Profession required Skill",
                    "select_required_skill",
                ));
            }
            if !binding
                .catalog
                .skills
                .iter()
                .any(|available| available.reference == claim.reference)
            {
                return Err(reject(
                    "SKILL_MISSING",
                    "accepted catalog lacks the Profession's exact required Skill revision",
                    "select_required_skill",
                ));
            }
        }
        dependencies.extend([binding_record, accepted]);
    }
    Ok(ValidatedRoster {
        dependencies,
        optional_skill_degradations,
    })
}

/// Exact selection IDs or responsibility tags only; display names are never routing keys.
pub fn resolve_room_candidate(
    store: &Store,
    project: &str,
    room: &str,
    target: &str,
) -> store::Result<Record> {
    let roster = store
        .get(&crate::key(
            Scope::Project(project.into()),
            "room_roster",
            room,
        ))?
        .ok_or_else(|| {
            reject(
                "CANDIDATE_NOT_FOUND",
                "Room has no roster",
                "select_room_candidate",
            )
        })?;
    let refs: Vec<Reference> = decode(&roster)?;
    let mut matches = vec![];
    for r in refs {
        if r.key.scope != Scope::Project(project.into()) || r.key.kind != "room_selection" {
            return Err(invalid("roster contains foreign selection"));
        }
        let record = store
            .get(&r.key)?
            .ok_or_else(|| invalid("selection missing"))?;
        if reference(&record) != r {
            return Err(invalid("selection changed"));
        }
        let selected: Selection = decode(&record)?;
        if selected.room_id != room {
            return Err(invalid("selection belongs to another Room"));
        }
        if record.key.id == target || selected.responsibility == target {
            matches.push(record);
        }
    }
    match matches.len() {
        1 => Ok(matches.remove(0)),
        0 => Err(reject(
            "CANDIDATE_NOT_FOUND",
            "no exact Room candidate",
            "select_room_candidate",
        )),
        _ => Err(reject(
            "CANDIDATE_AMBIGUOUS",
            "multiple candidates for this responsibility",
            "select_exact_candidate",
        )),
    }
}
