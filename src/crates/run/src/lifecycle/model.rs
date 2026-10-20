use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use store::Reference;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Starting,
    Running,
    Pausing,
    Paused,
    Cancelling,
    Completed,
    Failed,
    Cancelled,
    Replaced,
}
impl State {
    pub fn terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled | Self::Replaced
        )
    }
    pub fn transitional(self) -> bool {
        matches!(self, Self::Starting | Self::Pausing | Self::Cancelling)
    }
    pub fn allows(self, next: Self) -> bool {
        use State::*;
        matches!(
            (self, next),
            (Starting, Running | Failed | Cancelled | Replaced)
                | (
                    Running,
                    Pausing | Cancelling | Completed | Failed | Replaced
                )
                | (Pausing, Paused | Cancelling | Failed | Replaced)
                | (Paused, Running | Cancelling | Failed | Replaced)
                | (Cancelling, Cancelled | Failed | Replaced)
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExactContext {
    pub reference: Reference,
    pub digest: String,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationMode {
    #[default]
    Automatic,
    HumanPreview,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Integration {
    pub target_ref: String,
    pub target_kind: repo::integration::TargetKind,
    pub form: repo::integration::Form,
    pub strategy: repo::integration::Strategy,
    #[serde(default)]
    pub mode: IntegrationMode,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pick {
    pub checkpoint: String,
    pub seat: String,
    pub profession: Reference,
    pub agency: Reference,
    pub worker_profiles: Vec<Reference>,
    pub permissions: Vec<String>,
    pub max_context_bytes: u64,
    /// None leaves content unknown. Some commits this full immutable content for
    /// the Seat; future per-Attempt delivery must pass validate_delivery_contract.
    pub frozen_bundle: Option<Reference>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selected {
    pub checkpoint: String,
    pub requirement: crate::Seat,
    pub selection: Reference,
    pub profession: Reference,
    pub profession_digest: String,
    pub agency: Reference,
    pub worker_profiles: Vec<Reference>,
    pub profile_digests: Vec<String>,
    pub required_skills: Vec<participant::selection::Skill>,
    pub permissions: Vec<String>,
    pub max_context_bytes: u64,
    pub frozen_bundle: Option<Reference>,
    pub frozen_content_digest: Option<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Placement {
    pub destination: Option<Reference>,
    /// Only outward result delivery is mutable; never Agency location or authority.
    pub allowed_destinations: Vec<Reference>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub max_attempts: u64,
    pub context_bytes: u64,
    pub deadline_ms: u64,
    #[serde(default = "default_transition_timeout")]
    pub transition_timeout_ms: u64,
    #[serde(default)]
    pub incremental_review: bool,
    pub switch_on: Vec<String>,
    pub network_scope: Vec<String>,
    pub secret_scope: Vec<String>,
}
pub fn default_transition_timeout() -> u64 {
    15 * 60 * 1000
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Start {
    pub project_id: String,
    pub project_version: i64,
    pub run_id: String,
    pub workflow_id: String,
    pub workflow_revision: i64,
    /// A vector makes the 0..1 cardinality an explicit checked boundary.
    #[serde(default)]
    pub task_revisions: Vec<Reference>,
    pub repo: Reference,
    pub baseline: String,
    pub context: ExactContext,
    pub engine_binding: Reference,
    pub picks: Vec<Pick>,
    pub policy: Policy,
    pub integration: Integration,
    #[serde(default)]
    pub placement: Placement,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub project: Reference,
    pub selection_policy_digest: String,
    pub run_id: String,
    pub task_revision: Option<Reference>,
    pub workflow: Reference,
    pub approval: Reference,
    pub deployment: Reference,
    pub repo: Reference,
    pub baseline: String,
    pub context: ExactContext,
    pub engine_binding: Reference,
    pub selected: Vec<Selected>,
    pub frozen_rework_limits: BTreeMap<String, u32>,
    pub policy: Policy,
    pub integration: Integration,
    pub placement: Placement,
    pub readback_proof: serde_json::Value,
    pub authorized_by: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run {
    pub id: String,
    pub project_id: String,
    pub task_id: Option<String>,
    pub lifecycle: State,
    pub manifest: Reference,
    pub entered_at_ms: u64,
    pub transition_timeout_ms: u64,
    pub operation_epoch: u64,
    pub authorization_valid: bool,
    pub attention: Option<String>,
    pub terminal_requested: Option<State>,
    pub placement: Option<Reference>,
}
impl Run {
    pub fn timed_out(&self, now: u64) -> bool {
        self.lifecycle.transitional()
            && now.saturating_sub(self.entered_at_ms) >= self.transition_timeout_ms
    }
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
    Start {
        spec: Box<Start>,
    },
    Pause {
        project_id: String,
        run_id: String,
        version: i64,
    },
    Resume {
        project_id: String,
        run_id: String,
        version: i64,
    },
    Cancel {
        project_id: String,
        run_id: String,
        version: i64,
    },
    Replace {
        project_id: String,
        run_id: String,
        version: i64,
        spec: Box<Start>,
    },
    Place {
        project_id: String,
        run_id: String,
        version: i64,
        destination: Reference,
    },
}
impl Action {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Start { .. } => "start",
            Self::Pause { .. } => "pause",
            Self::Resume { .. } => "resume",
            Self::Cancel { .. } => "cancel",
            Self::Replace { .. } => "replace",
            Self::Place { .. } => "place",
        }
    }
    pub fn project(&self) -> &str {
        match self {
            Self::Start { spec } => &spec.project_id,
            Self::Pause { project_id, .. }
            | Self::Resume { project_id, .. }
            | Self::Cancel { project_id, .. }
            | Self::Replace { project_id, .. }
            | Self::Place { project_id, .. } => project_id,
        }
    }
    pub fn start(&self) -> Option<&Start> {
        match self {
            Self::Start { spec } | Self::Replace { spec, .. } => Some(spec),
            _ => None,
        }
    }
}
/// Child authorities are registered by package 4a BEFORE any dispatch. Unknown
/// write containment retains occupancy; a model/engine report cannot mark it safe.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Activity {
    pub run_id: String,
    pub manifest: Reference,
    pub owner: Reference,
    pub authorization_valid: bool,
    pub can_write: bool,
    pub isolation_proof: Option<Reference>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IsolationProof {
    pub manifest: Reference,
    pub owner: Reference,
    pub confined: bool,
    pub silent: bool,
}
