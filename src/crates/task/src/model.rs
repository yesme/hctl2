use serde::{Deserialize, Serialize};
use serde_json::Value;
use store::{ExternalEntity, MaterialRef, ObjectKey, Record, Reference};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    pub create: bool,
    pub field_writeback: bool,
    pub conditional_write: bool,
    /// Database compare-and-update covers these fields, not the whole PATCH.
    pub conditional_fields: Vec<String>,
    pub placement: bool,
    pub delete: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub repo_id: String,
    pub port_kind: String,
    pub candidate: repo::SourceCandidate,
    pub platform: repo::PlatformObservation,
    pub capabilities: Capabilities,
    pub board_scope_stable_id: String,
    pub binding_revision: i64,
    pub active: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupKind {
    Milestone,
    Label,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub kind: GroupKind,
    pub anchor_stable_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceReference {
    pub project_id: String,
    pub source: Reference,
    pub approved_scope: String,
    pub group: Option<Group>,
}

/// Dependencies are observations inside a Snapshot, never separately writable Task objects.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Dependencies {
    pub parent: Option<Value>,
    pub children: Vec<Value>,
    pub blocked_by: Vec<Value>,
    pub blocking: Vec<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Card {
    pub entity: ExternalEntity,
    pub number: u64,
    pub title: String,
    pub body: String,
    pub stage: String,
    pub remote_revision: String,
    pub content_version: Option<i64>,
    pub groups: Vec<Group>,
    pub dependencies: Dependencies,
    pub comments: Vec<Value>,
    pub raw: Value,
    pub tombstone: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub source: Reference,
    pub observed_at: u64,
    pub complete: bool,
    pub error: Option<String>,
    pub cards: Vec<Card>,
    pub stable_groups: Vec<Group>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Grade {
    Mechanical,
    Gate,
    Human,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Acceptance {
    pub text: String,
    pub grade: Grade,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contract {
    pub scope: String,
    pub expected_outcome: String,
    pub acceptance: Vec<Acceptance>,
    pub roles: Vec<String>,
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ContractOrigin {
    Local {
        reference: Reference,
        proposal_digest: String,
    },
    Backend {
        snapshot: Reference,
        state_version: i64,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Adoption {
    pub contract: Contract,
    pub origin: ContractOrigin,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Revision {
    pub number: i64,
    pub material: MaterialRef,
    pub origin: ContractOrigin,
    pub proposal_digest: String,
    pub binding: Option<Reference>,
    pub policy_digest: String,
    pub backend_projection_digest: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub source_id: String,
    pub repo_id: String,
    pub entity: Option<ExternalEntity>,
    pub number: Option<u64>,
    pub title: String,
    pub lifecycle: String,
    pub lifecycle_version: i64,
    pub archived: bool,
    pub revision: Option<Revision>,
    pub snapshot: Option<Reference>,
    pub state_version: i64,
    pub needs_attention: bool,
    pub pending_contract: Option<Reference>,
    /// Reserved integration point for Run; Task cancellation never clears an occupied slot.
    pub run_occupancy: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fields {
    pub title: Option<String>,
    pub body: Option<String>,
    pub comment: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Connect {
        repo_id: String,
        candidate_id: String,
        consent: bool,
        make_default: bool,
    },
    SetActive {
        repo_id: String,
        source_id: String,
        active: bool,
        version: i64,
    },
    Attach {
        project_id: String,
        project_version: i64,
        source_id: String,
        approved_scope: String,
        group: Option<Group>,
        consent: bool,
    },
    Claim {
        project_id: String,
        project_version: i64,
        source_id: String,
        entity_id: String,
    },
    Create {
        project_id: String,
        project_version: i64,
        source_id: String,
        title: String,
        body: String,
        adoption: Option<Adoption>,
    },
    Adopt {
        project_id: String,
        project_version: i64,
        task_id: String,
        version: i64,
        adoption: Adoption,
    },
    Update {
        project_id: String,
        task_id: String,
        version: i64,
        state_version: i64,
        fields: Fields,
    },
    Move {
        project_id: String,
        task_id: String,
        version: i64,
        state_version: i64,
        stage: String,
        rank: Option<String>,
        relative_source_id: Option<String>,
    },
    Cancel {
        project_id: String,
        task_id: String,
        version: i64,
    },
    DeleteCard {
        project_id: String,
        task_id: String,
        version: i64,
        confirm_irreversible: bool,
        active_run_choices: Vec<Reference>,
    },
    Refresh {
        repo_id: String,
        source_id: String,
    },
    Resume {
        effect_id: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub key: String,
    pub action: Action,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Check {
    pub key: ObjectKey,
    pub version: Option<i64>,
}

/// Frozen preview. Transport never accepts this struct from a client.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plan {
    pub input: Input,
    pub checks: Vec<Check>,
    pub records: Vec<Record>,
    pub effects: Vec<store::EffectIntent>,
    pub cancel_effects: Vec<String>,
    pub result: Value,
    pub adoption: Option<Adoption>,
    pub task_key: Option<ObjectKey>,
}
