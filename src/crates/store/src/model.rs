use foundation::canonical_json_sha256;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{MaterialRef, Result, StoreError};

/// The existing owner of a fact; shared definitions do not acquire a Project owner.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum Scope {
    Control,
    Repo(String),
    Project(String),
}

impl Scope {
    pub(crate) fn validate(&self) -> Result<()> {
        match self {
            Self::Control => Ok(()),
            Self::Repo(id) | Self::Project(id) => nonempty(id, "scope ID"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectKey {
    pub scope: Scope,
    pub kind: String,
    pub id: String,
}

impl ObjectKey {
    pub(crate) fn validate(&self) -> Result<()> {
        self.scope.validate()?;
        nonempty(&self.kind, "object kind")?;
        nonempty(&self.id, "object ID")?;
        match self.kind.as_str() {
            "repo" if self.scope != Scope::Repo(self.id.clone()) => {
                return Err(StoreError::invalid(
                    "Repo identity must retain its Repo scope",
                ));
            }
            "project" if self.scope != Scope::Project(self.id.clone()) => {
                return Err(StoreError::invalid("Project identity and scope must agree"));
            }
            "room" | "task" | "run" if !matches!(self.scope, Scope::Project(_)) => {
                return Err(StoreError::invalid("work must name its Project"));
            }
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn encoded(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Version {
    State(i64),
    Revision(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    pub key: ObjectKey,
    pub version: Version,
}

impl Reference {
    pub(crate) fn validate(&self) -> Result<()> {
        self.key.validate()?;
        match &self.version {
            Version::State(n) => safe_version(*n),
            Version::Revision(d) => digest(d),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorSource {
    DirectClient,
    ProviderEvent,
    InternalReducer,
}

/// Provenance is assigned by a trusted connection/adapter, never by an execution principal.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Actor {
    pub principal: String,
    pub source: ActorSource,
    pub permission_scope: Vec<Scope>,
    pub authority: Option<Reference>,
}

/// Constructed by the trusted caller after authenticating its connection or frozen rule.
/// Intentionally not Deserialize: do not build this value from a submitted actor field.
pub struct TrustedActor(pub Actor);

impl TrustedActor {
    pub(crate) fn permits(&self, scope: &Scope) -> Result<()> {
        if self.0.permission_scope.contains(scope) {
            Ok(())
        } else {
            Err(StoreError::new(
                "PERMISSION_DENIED",
                "scope not authorized",
                "request_authorization",
            ))
        }
    }
}

/// Six mandatory envelope groups; generation is checked separately on every write.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub command_id: String,
    pub idempotency_key: String,
    pub actor: Actor,
    pub target: ObjectKey,
    /// Absent means the target must not exist, not an omitted precondition.
    pub expected: Expected,
    pub binding: Reference,
    pub input_digest: String,
    pub operation: String,
    pub input: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Expected {
    Absent,
    Exact(Version),
}

impl Command {
    /// The domain operation name participates in the canonical input, not transport encoding.
    pub fn digest_input(operation: &str, input: &Value) -> Result<String> {
        Ok(canonical_json_sha256(
            &json!({"operation": operation, "input": input}),
        )?)
    }

    pub(crate) fn validate(&self, trusted: &TrustedActor) -> Result<String> {
        nonempty(&self.command_id, "command_id")?;
        nonempty(&self.idempotency_key, "idempotency_key")?;
        nonempty(&self.actor.principal, "actor principal")?;
        nonempty(&self.operation, "operation")?;
        self.target.validate()?;
        self.binding.validate()?;
        if self.actor != trusted.0 {
            return Err(StoreError::new(
                "ACTOR_MISMATCH",
                "actor differs from trusted provenance",
                "reauthenticate",
            ));
        }
        trusted.permits(&self.target.scope)?;
        if self.actor.source == ActorSource::InternalReducer && self.actor.authority.is_none() {
            return Err(StoreError::invalid(
                "internal reducer requires original authority",
            ));
        }
        if let Some(authority) = &self.actor.authority {
            authority.validate()?;
        }
        if Self::digest_input(&self.operation, &self.input)? != self.input_digest {
            return Err(StoreError::invalid("canonical input digest mismatch"));
        }
        Ok(canonical_json_sha256(&serde_json::to_value(self)?)?)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExternalEntity {
    pub provider: String,
    pub account_stable_id: String,
    pub external_entity_kind: String,
    pub immutable_external_entity_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomKind {
    Main,
    Topic,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomState {
    Active,
    ReadOnly,
    Archived,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub publish_review_requires_confirmation: bool,
    pub selection_policy: Value,
}

/// Payloads needed to reserve the P2.1 keys, not business command implementations.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RecordData {
    Repo {
        platform_binding: Option<Reference>,
    },
    Project {
        repo_id: String,
        settings: ProjectSettings,
        archived: bool,
    },
    Room {
        room_kind: RoomKind,
        state: RoomState,
    },
    Task {
        entity: Option<ExternalEntity>,
        source: Reference,
    },
    /// Other module payloads remain owned and validated by their reducer.
    Value {
        value: Value,
    },
}

/// An immutable event snapshot; the current-object table is entirely rebuildable.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    pub key: ObjectKey,
    pub version: i64,
    pub revision_digest: String,
    pub data: RecordData,
    pub sources: Vec<Reference>,
    pub materials: Vec<MaterialRef>,
}

impl Record {
    pub(crate) fn validate(&self) -> Result<()> {
        self.key.validate()?;
        safe_version(self.version)?;
        if self.version == 0 {
            return Err(StoreError::invalid("record version starts at one"));
        }
        digest(&self.revision_digest)?;
        match (&self.data, self.key.kind.as_str()) {
            (RecordData::Repo { platform_binding }, "repo") => {
                if let Some(binding) = platform_binding {
                    binding.validate()?;
                }
            }
            (RecordData::Room { .. }, "room") => {}
            (RecordData::Project { repo_id, .. }, "project") => nonempty(repo_id, "repo ID")?,
            (RecordData::Task { entity, source }, "task") => {
                source.validate()?;
                if let Some(e) = entity {
                    for s in [
                        &e.provider,
                        &e.account_stable_id,
                        &e.external_entity_kind,
                        &e.immutable_external_entity_id,
                    ] {
                        nonempty(s, "external entity field")?;
                    }
                }
            }
            (RecordData::Value { .. }, kind)
                if !["repo", "project", "room", "task"].contains(&kind) => {}
            _ => return Err(StoreError::invalid("record payload and kind disagree")),
        }
        for source in &self.sources {
            source.validate()?;
        }
        Ok(())
    }
}

pub(crate) fn nonempty(value: &str, name: &str) -> Result<()> {
    if value.trim().is_empty() {
        Err(StoreError::invalid(format!("missing {name}")))
    } else {
        Ok(())
    }
}

pub(crate) fn digest(value: &str) -> Result<()> {
    if value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        Ok(())
    } else {
        Err(StoreError::invalid("expected lowercase SHA-256"))
    }
}

pub(crate) fn safe_version(value: i64) -> Result<()> {
    if (0..=9_007_199_254_740_991).contains(&value) {
        Ok(())
    } else {
        Err(StoreError::invalid(
            "version exceeds canonical integer range",
        ))
    }
}
