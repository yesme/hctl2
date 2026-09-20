use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use store::{Actor, MaterialRef};

use crate::{Result, reject};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Local,
    External,
    Independent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Local,
    Github,
    None,
}

/// A path is an input to this operation, not a registered working-copy object.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalInput {
    pub machine: String,
    pub path: PathBuf,
    #[serde(default)]
    pub in_place: bool,
    #[serde(default)]
    pub extra_refs: Vec<String>,
    #[serde(default)]
    pub publish_governance: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Register {
    pub name: String,
    pub origin: Origin,
    /// Omitted only for a verified pure-local input, where Local is the default.
    pub platform: Option<Platform>,
    pub instance: Option<String>,
    pub platform_repo_id: Option<String>,
    /// Human-declared owner/name for GitHub; a name for a new hosted repository.
    pub platform_path: Option<String>,
    pub local: Option<LocalInput>,
    pub remote_evidence: Option<String>,
    /// Explicit selection; absence never silently binds the recommended source.
    pub default_source: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalSnapshot {
    pub path: PathBuf,
    pub remotes: BTreeMap<String, String>,
    pub refs: BTreeMap<String, String>,
    pub head_branch: String,
    pub governance_paths: Vec<String>,
}

/// Shared with the following task-source package, not a Project source reference.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceCandidate {
    pub id: String,
    pub provider: String,
    pub actual_source_and_scope: String,
    pub recommended: bool,
    pub create: bool,
    pub field_writeback: bool,
    pub can_claim: bool,
    pub available: bool,
}
impl SourceCandidate {
    pub fn can_default(&self) -> bool {
        self.available && self.create && self.field_writeback
    }
    pub fn label(&self) -> &'static str {
        if !self.available {
            "not_available"
        } else if self.can_default() {
            "can_default"
        } else {
            "cannot_default_can_claim"
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Prepared {
    pub request: Register,
    pub platform: Platform,
    pub local: Option<LocalSnapshot>,
    pub evidence_conflicts: Vec<String>,
    pub sources: Vec<SourceCandidate>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlatformObservation {
    pub instance: String,
    pub stable_id: String,
    pub full_name: String,
    pub clone_url: String,
    pub account_id: String,
    pub has_issues: bool,
    pub can_write_issues: bool,
    pub credential_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Lifecycle {
    Pending,
    Active,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Registration {
    pub repo_id: String,
    pub version: i64,
    pub lifecycle: Lifecycle,
    pub abandoned: bool,
    pub command_id: String,
    pub idempotency_key: String,
    pub actor: Actor,
    pub prepared: Prepared,
    pub config: MaterialRef,
    pub observed: Option<PlatformObservation>,
    pub delivered: bool,
    pub residual: Option<PlatformObservation>,
    /// Preserve the original target even if creation succeeded but its response was lost.
    pub residual_target: Option<String>,
}

pub fn source_candidates(platform: &Platform, scope: &str) -> Vec<SourceCandidate> {
    let (provider, available) = match platform {
        Platform::Github => ("github_issues", true),
        Platform::Local => ("gitea_issues", true),
        Platform::None => ("local_task_server", false),
    };
    vec![SourceCandidate {
        id: provider.into(),
        provider: provider.into(),
        actual_source_and_scope: scope.into(),
        recommended: true,
        create: available,
        field_writeback: available,
        can_claim: available,
        available,
    }]
}

pub fn validate_default(selected: Option<&str>, candidates: &[SourceCandidate]) -> Result<()> {
    if let Some(id) = selected {
        let candidate = candidates.iter().find(|c| c.id == id).ok_or_else(|| {
            reject(
                "SOURCE_UNAVAILABLE",
                "selected task source is not available",
                "select_source_later",
            )
        })?;
        if !candidate.can_default() {
            return Err(reject(
                "SOURCE_CAPABILITY_MISSING",
                "default source requires create and field writeback",
                "choose_capable_source",
            ));
        }
    }
    Ok(())
}

pub fn prepare(request: Register, local: Option<LocalSnapshot>) -> Result<Prepared> {
    if request.name.trim().is_empty() || request.local.is_some() != local.is_some() {
        return Err(reject(
            "INVALID_INPUT",
            "name and verified local input required",
            "correct_input",
        ));
    }
    let has_remote = local.as_ref().is_some_and(|s| !s.remotes.is_empty());
    if has_remote && request.origin == Origin::Local && request.platform != Some(Platform::None) {
        return Err(reject(
            "REMOTE_CHOICE_REQUIRED",
            "choose original external platform or independent work",
            "preview_explicit_choice",
        ));
    }
    let platform = match &request.platform {
        Some(p) => p.clone(),
        None if !has_remote && request.origin == Origin::Local && local.is_some() => {
            Platform::Local
        }
        None => {
            return Err(reject(
                "PLATFORM_CHOICE_REQUIRED",
                "declare platform or explicitly select none",
                "correct_input",
            ));
        }
    };
    if request.origin == Origin::External && platform == Platform::Local {
        return Err(reject(
            "EXTERNAL_MIRROR_REJECTED",
            "external Repo cannot bind to hosted platform; independent work needs a new registration",
            "declare_independent_work",
        ));
    }
    if request.origin == Origin::Independent && local.is_none() {
        return Err(reject(
            "LOCAL_INPUT_REQUIRED",
            "independent copy requires a verified local input",
            "supply_local_path",
        ));
    }
    if request.origin == Origin::Independent && platform != Platform::Local {
        return Err(reject(
            "INDEPENDENT_PLATFORM",
            "independent-work entry creates a new hosted repository; attach existing external/none with its declared origin",
            "choose_registration_route",
        ));
    }
    if request.local.as_ref().is_some_and(|l| l.in_place)
        && (request.origin != Origin::Independent
            || platform != Platform::Local
            || local.as_ref().is_some_and(|s| s.remotes.len() != 1))
    {
        return Err(reject(
            "IN_PLACE_SCOPE",
            "in-place switch requires independent work and exactly one remote",
            "use_independent_copy",
        ));
    }
    for url in request
        .remote_evidence
        .iter()
        .chain(local.iter().flat_map(|l| l.remotes.values()))
    {
        if url.chars().any(char::is_control)
            || url
                .strip_prefix("https://")
                .or_else(|| url.strip_prefix("http://"))
                .is_some_and(|s| s.split('/').next().unwrap_or("").contains('@'))
            || url.strip_prefix("ssh://").is_some_and(|s| {
                s.split('/')
                    .next()
                    .unwrap_or("")
                    .split_once('@')
                    .is_some_and(|(user, _)| user.contains(':'))
            })
        {
            return Err(reject(
                "REMOTE_CREDENTIALS",
                "remote evidence must not contain inline credentials or control characters",
                "remove_inline_credentials",
            ));
        }
    }
    let mut conflicts = Vec::new();
    match platform {
        Platform::Github => {
            for value in [
                &request.instance,
                &request.platform_repo_id,
                &request.platform_path,
            ] {
                if value.as_ref().is_none_or(|v| v.trim().is_empty()) {
                    return Err(reject(
                        "PLATFORM_ID_REQUIRED",
                        "declare service instance, stable repository ID and owner/name; URL is evidence only",
                        "declare_platform_identity",
                    ));
                }
            }
            let instance = request.instance.as_deref().unwrap();
            if !instance
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b".-".contains(&b))
                || instance.starts_with('-')
            {
                return Err(reject(
                    "INVALID_INPUT",
                    "instance must be a GitHub hostname",
                    "correct_input",
                ));
            }
            let path = request.platform_path.as_deref().unwrap();
            if path.split('/').count() != 2 || !path.split('/').all(valid_segment) {
                return Err(reject(
                    "INVALID_INPUT",
                    "platform path must be owner/name",
                    "correct_input",
                ));
            }
            let expected = format!("{instance}/{path}");
            let evidence = request
                .remote_evidence
                .iter()
                .chain(local.iter().flat_map(|l| l.remotes.values()));
            for url in evidence {
                let normalized = url
                    .strip_prefix("https://")
                    .or_else(|| url.strip_prefix("http://"))
                    .or_else(|| url.strip_prefix("ssh://git@"))
                    .map(str::to_owned)
                    .or_else(|| url.strip_prefix("git@").map(|s| s.replacen(':', "/", 1)));
                if normalized.as_deref().map(|s| s.trim_end_matches(".git"))
                    != Some(expected.as_str())
                {
                    conflicts.push(url.clone());
                }
            }
        }
        Platform::Local => {
            if let (Some(snapshot), Some(input)) = (&local, &request.local) {
                if snapshot.refs.keys().any(|r| {
                    r.starts_with("refs/heads/hctl2/") || r.starts_with("refs/tags/hctl2/")
                }) {
                    return Err(reject(
                        "PRIVATE_REF",
                        "private retention refs cannot be published",
                        "select_public_refs",
                    ));
                }
                if !snapshot.governance_paths.is_empty() && !input.publish_governance {
                    return Err(reject(
                        "GOVERNANCE_PUBLICATION_REQUIRED",
                        "selected history contains .memo/.hctl2 materials; publishing them requires explicit choice",
                        "preview_publication_scope",
                    ));
                }
            }
            if request.instance.is_some() || request.platform_repo_id.is_some() {
                return Err(reject(
                    "INVALID_INPUT",
                    "new hosted platform identity is declared after create/readback; do not substitute a URL",
                    "confirm_created_identity",
                ));
            }
            if !request.platform_path.as_deref().is_some_and(valid_segment) {
                return Err(reject(
                    "INVALID_INPUT",
                    "hosted platform needs a repository name",
                    "correct_input",
                ));
            }
        }
        Platform::None => {
            if request.instance.is_some()
                || request.platform_repo_id.is_some()
                || request.platform_path.is_some()
            {
                return Err(reject(
                    "INVALID_INPUT",
                    "no-platform choice cannot carry a platform binding",
                    "correct_input",
                ));
            }
        }
    }
    let sources = source_candidates(
        &platform,
        request.platform_path.as_deref().unwrap_or("unbound"),
    );
    validate_default(request.default_source.as_deref(), &sources)?;
    Ok(Prepared {
        request,
        platform,
        local,
        evidence_conflicts: conflicts,
        sources,
    })
}

pub fn valid_segment(s: &str) -> bool {
    !s.is_empty()
        && s != "."
        && s != ".."
        && !s.starts_with('-')
        && s.len() <= 100
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
}
