use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use store::{EffectIntent, ProjectSettings, Record, Reference};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub name: String,
    pub goal: String,
    pub scope: String,
    pub roles: Vec<String>,
    /// Explicit human assignments. Room participants are not human actors.
    pub role_members: BTreeMap<String, Vec<String>>,
    pub defaults: Value,
    pub settings: ProjectSettings,
}

/// Immutable selection, independent of Matrix membership and the external binding.
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
    pub reference: Reference,
    /// None is the explicit unknown grade; required unknown is not dispatchable.
    pub digest: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub key: String,
    pub action: Action,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Create {
        repo_id: String,
        definition: Definition,
    },
    Update {
        project_id: String,
        version: i64,
        definition: Definition,
    },
    Archive {
        project_id: String,
        version: i64,
    },
    Restore {
        project_id: String,
        version: i64,
    },
    Members {
        project_id: String,
        project_version: i64,
        rooms: Vec<Reference>,
        users: Vec<String>,
        invite: bool,
    },
    Select {
        project_id: String,
        project_version: i64,
        /// Existing Room or the exact topic_id for the following create command.
        room_id: String,
        topic_command_key: Option<String>,
        roster_version: Option<i64>,
        selections: Vec<Selection>,
    },
    CreateRequest {
        project_id: String,
        project_version: i64,
        request: RequestSpec,
    },
    ResolveRequest {
        project_id: String,
        request_id: String,
        version: i64,
        adoption: task::Adoption,
    },
    CancelRequest {
        project_id: String,
        request_id: String,
        version: i64,
    },
    Resume {
        project_id: String,
        effect_id: String,
    },
}
impl Action {
    pub fn project_id(&self) -> Option<&str> {
        match self {
            Self::Create { .. } => None,
            Self::Update { project_id, .. }
            | Self::Archive { project_id, .. }
            | Self::Restore { project_id, .. }
            | Self::Select { project_id, .. }
            | Self::Members { project_id, .. }
            | Self::CreateRequest { project_id, .. }
            | Self::ResolveRequest { project_id, .. }
            | Self::CancelRequest { project_id, .. }
            | Self::Resume { project_id, .. } => Some(project_id),
        }
    }
}

/// The only current source action is Task contract adoption. Run/Invocation source
/// builders belong to P2.3/P2.5; arbitrary JSON is not an executable answer schema.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestSpec {
    pub question: String,
    pub target: Target,
    pub owner: Reference,
    pub affected_revision: Option<Reference>,
    pub blocking_scope: String,
    pub owner_state_version: i64,
    pub dedup_root: String,
    pub permissions: Value,
    pub deadline: Option<u64>,
    pub deadline_action: DeadlineAction,
    pub action: RequiredAction,
    pub input_schema: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Target {
    Human { principal: String },
    Role { role: String },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequiredAction {
    TaskAdopt,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeadlineAction {
    FailWaiting,
    CancelWaiting,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestState {
    Open,
    Resolved,
    Superseded,
    Cancelled,
    Expired,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    /// Chat's RequestSource reads these exact fields for Topic creation.
    pub question: String,
    pub blockers: Vec<Reference>,
    pub spec: RequestSpec,
    pub state: RequestState,
    pub created_at: u64,
    pub superseded_by: Option<String>,
    pub solution_digest: Option<String>,
    pub effect_id: Option<String>,
    pub actor: Option<store::Actor>,
}

/// Source-side waiting state: Request ID and exact blocker, not a lifecycle copy.
pub use task::RequestBlocker as Blocker;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plan {
    pub input: Input,
    pub project_id: String,
    pub checks: Vec<chat::Check>,
    pub records: Vec<Record>,
    pub effects: Vec<EffectIntent>,
    pub task_plan: Option<task::Plan>,
    pub result: Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveBlocker {
    pub reference: Reference,
    pub kind: String,
    pub state: String,
}
