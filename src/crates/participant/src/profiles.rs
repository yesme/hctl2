//! Immutable Worker Profile revisions and their selection-only current pointer.
use crate::{decode, frozen, invalid, key, reference, reject, value};
use agency_proto::{Capabilities, FrozenRef};
use foundation::{bytes_sha256, canonical_json_sha256};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use store::{Command, Expected, Record, Reference, Scope, Store, TrustedActor, Version};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkerProfile {
    pub harness: FrozenRef,
    pub model: String,
    pub mode: String,
    pub permissions: Vec<String>,
    /// Requested environment properties, not Agency hosts, processes or directories.
    pub environment: Vec<String>,
    pub required_capabilities: Capabilities,
    /// Context bytes, not a token estimate or a currency budget.
    pub max_context_bytes: u64,
}
impl WorkerProfile {
    pub fn validate(&self) -> store::Result<()> {
        self.harness.validate().map_err(crate::port_error)?;
        if self.model.trim().is_empty()
            || self.mode != "read_only"
            || self.max_context_bytes == 0
            || self.max_context_bytes > agency_proto::MAX_DOCUMENT as u64
            || self.environment.iter().any(|v| v.trim().is_empty())
        {
            return Err(invalid(
                "read_only profile, model and bounded context bytes required",
            ));
        }
        validate_permissions(&self.permissions)
    }
}

pub fn validate_permissions(permissions: &[String]) -> store::Result<()> {
    for (index, permission) in permissions.iter().enumerate() {
        if !matches!(
            permission.as_str(),
            "context.read" | "git.read" | "terminal.observe"
        ) || permissions[..index].contains(permission)
        {
            return Err(reject(
                "PERMISSION_SCOPE_INVALID",
                "unique read-only permissions required; no command or integration authority",
                "narrow_permissions",
            ));
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProfileInput {
    pub key: String,
    pub action: ProfileAction,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProfileAction {
    Create {
        id: String,
        profile: WorkerProfile,
    },
    Update {
        id: String,
        version: i64,
        profile: WorkerProfile,
    },
}
impl ProfileAction {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Create { .. } => "create",
            Self::Update { .. } => "update",
        }
    }
    fn parts(&self) -> (&str, Option<i64>, &WorkerProfile) {
        match self {
            Self::Create { id, profile } => (id, None, profile),
            Self::Update {
                id,
                version,
                profile,
            } => (id, Some(*version), profile),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfilePlan {
    pub input: ProfileInput,
    pub previous: Option<Reference>,
    pub revision: Record,
    pub pointer: Record,
    pub result: Value,
}
fn command_key(input: &ProfileInput) -> store::ObjectKey {
    key(
        Scope::Control,
        "profile_command",
        &bytes_sha256(input.key.as_bytes()),
    )
}
fn access(actor: &TrustedActor) -> store::Result<()> {
    if !actor.0.permission_scope.contains(&Scope::Control) {
        return Err(reject(
            "PERMISSION_DENIED",
            "control definition permission required",
            "request_authorization",
        ));
    }
    Ok(())
}
fn replay(
    store: &Store,
    input: &ProfileInput,
    actor: &TrustedActor,
) -> store::Result<Option<ProfilePlan>> {
    access(actor)?;
    let Some(record) = store.get(&command_key(input))? else {
        return Ok(None);
    };
    let stored: Value = decode(&record)?;
    if stored["actor"] != serde_json::to_value(&actor.0)? {
        return Err(reject(
            "ACTOR_MISMATCH",
            "profile command belongs to another actor",
            "use_original_actor",
        ));
    }
    let plan: ProfilePlan = serde_json::from_value(stored["plan"].clone())?;
    if plan.input != *input {
        return Err(reject(
            "IDEMPOTENCY_CONFLICT",
            "profile key has different input",
            "use_original_command",
        ));
    }
    Ok(Some(plan))
}
pub fn prepare_profile(
    store: &Store,
    input: ProfileInput,
    actor: &TrustedActor,
) -> store::Result<ProfilePlan> {
    access(actor)?;
    if input.key.trim().is_empty() {
        return Err(invalid("profile command key required"));
    }
    if let Some(plan) = replay(store, &input, actor)? {
        return Ok(plan);
    }
    let (id, expected, profile) = input.action.parts();
    agency_proto::nonempty(id).map_err(crate::port_error)?;
    if id != id.trim() {
        return Err(invalid(
            "profile id must not have leading or trailing whitespace",
        ));
    }
    profile.validate()?;
    let pointer_key = key(Scope::Control, "worker_profile", id);
    let old = store.get(&pointer_key)?;
    if old.as_ref().map(|r| r.version) != expected {
        return Err(reject(
            "VERSION_CONFLICT",
            "profile pointer changed",
            "preview_again",
        ));
    }
    let digest = canonical_json_sha256(&serde_json::to_value(profile)?)?;
    let revision = value(
        key(Scope::Control, "worker_profile_revision", &digest),
        1,
        profile,
    )?;
    let next = expected
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| invalid("profile version exhausted"))?;
    let mut pointer = value(pointer_key, next, &reference(&revision))?;
    pointer.sources = vec![reference(&revision)];
    Ok(ProfilePlan {
        input: input.clone(),
        previous: old.as_ref().map(reference),
        result: json!({"profile_id":id,"version":pointer.version,"revision":reference(&revision),"frozen":frozen(&revision)}),
        revision,
        pointer,
    })
}
pub fn admit_profile(
    store: &mut Store,
    actor: &TrustedActor,
    plan: ProfilePlan,
) -> store::Result<Value> {
    if let Some(original) = replay(store, &plan.input, actor)? {
        return Ok(original.result);
    }
    let current = prepare_profile(store, plan.input.clone(), actor)?;
    if serde_json::to_value(&current)? != serde_json::to_value(&plan)? {
        return Err(reject(
            "VERSION_CONFLICT",
            "profile preview differs",
            "preview_again",
        ));
    }
    let input = serde_json::to_value(&plan.input)?;
    let command = Command {
        command_id: format!("profile:{}", plan.input.key),
        idempotency_key: plan.input.key.clone(),
        actor: actor.0.clone(),
        target: command_key(&plan.input),
        expected: Expected::Absent,
        binding: Reference {
            key: key(Scope::Control, "module", "participant"),
            version: Version::State(1),
        },
        input_digest: Command::digest_input("profile.command", &input)?,
        operation: "profile.command".into(),
        input,
    };
    store.submit(store.generation(), actor, &command, None, |tx| {
        if tx.get(&plan.pointer.key)?.as_ref().map(reference) != plan.previous {
            return Err(reject(
                "VERSION_CONFLICT",
                "profile pointer changed",
                "preview_again",
            ));
        }
        if let Some(existing) = tx.get(&plan.revision.key)? {
            if existing.revision_digest != plan.revision.revision_digest
                || serde_json::to_value(&existing.data)?
                    != serde_json::to_value(&plan.revision.data)?
            {
                return Err(invalid("immutable profile revision differs"));
            }
        } else {
            tx.put(&plan.revision)?;
        }
        tx.put(&plan.pointer)?;
        tx.put(&value(
            command.target.clone(),
            1,
            &json!({"actor":actor.0,"plan":plan}),
        )?)?;
        Ok(plan.result.clone())
    })
}

pub fn profile_at(store: &Store, target: &Reference) -> store::Result<(Record, WorkerProfile)> {
    if target.key.scope != Scope::Control || target.key.kind != "worker_profile_revision" {
        return Err(invalid("immutable Worker Profile revision required"));
    }
    let record = store.get(&target.key)?.ok_or_else(|| {
        reject(
            "PROFILE_NOT_FOUND",
            "profile revision missing",
            "create_profile",
        )
    })?;
    if reference(&record) != *target {
        return Err(reject(
            "VERSION_CONFLICT",
            "profile revision differs",
            "preview_again",
        ));
    }
    let profile: WorkerProfile = decode(&record)?;
    profile.validate()?;
    if canonical_json_sha256(&serde_json::to_value(&profile)?)? != record.revision_digest
        || record.key.id != record.revision_digest
    {
        return Err(reject(
            "DIGEST_MISMATCH",
            "profile content differs",
            "inspect_profile",
        ));
    }
    Ok((record, profile))
}

/// Read-only lookup of the pointer's exact revision. Reading shows a definition
/// and grants nothing; the trusted actor comes from the authenticated entry.
pub fn show_profile(store: &Store, actor: &TrustedActor, id: &str) -> store::Result<Value> {
    access(actor)?;
    agency_proto::nonempty(id).map_err(crate::port_error)?;
    let pointer = store
        .get(&key(Scope::Control, "worker_profile", id))?
        .ok_or_else(|| {
            reject(
                "PROFILE_NOT_FOUND",
                "profile pointer missing",
                "create_profile",
            )
        })?;
    let target: Reference = decode(&pointer)?;
    let (record, profile) = profile_at(store, &target)?;
    Ok(json!({
        "profile_id": id,
        "pointer": reference(&pointer),
        "revision": reference(&record),
        "profile": profile,
    }))
}
