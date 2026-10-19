//! Engine-independent immutable workflow and the restricted HCTL Profile.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use store::Reference;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workflow {
    pub schema: String,
    pub profile: String,
    #[serde(default)]
    pub context: String,
    #[serde(default)]
    pub readback: ReadbackPolicy,
    pub graph: Graph,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadbackPolicy {
    pub required: bool,
    pub different_worker_profile: bool,
}
impl Default for ReadbackPolicy {
    fn default() -> Self {
        Self {
            required: true,
            different_worker_profile: true,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    #[serde(default)]
    pub outputs: BTreeMap<String, OutputKind>,
    pub operation: Operation,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    External {
        criterion: Option<Criterion>,
        seats: Vec<Seat>,
        gate: Option<Gate>,
    },
    Fork {},
    Join {},
    Switch {
        cases: Vec<String>,
    },
    Loop {
        body: Graph,
        #[serde(default = "default_reworks")]
        max_reworks: u32,
        on_exhausted: Exhausted,
    },
    DynamicFork {
        template: Box<Node>,
        candidates: Vec<Candidate>,
        max_cardinality: u32,
        budget: u64,
        selection: Selection,
        permission_ceiling: Vec<String>,
    },
    TimerWait {
        seconds: u64,
    },
    Noop {},
    Transform {
        fields: BTreeMap<String, String>,
    },
}
/// delivery.md §运行默认值. Frozen by serde before hashing, shared with Manifest.
pub fn default_reworks() -> u32 {
    2
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Exhausted {
    Fail,
    Request,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Selection {
    All {},
    First { count: u32 },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub selection: Reference,
    pub responsibility: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputKind {
    ChangeSetRevision,
    ArtifactRevision,
    Verdict,
    Receipt,
    MechanicalFact,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Criterion {
    Mechanical {
        output: String,
        fact: MechanicalFact,
    },
    Gate {
        evidence: EvidenceLevel,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MechanicalFact {
    Ci { check: String },
    PrMerged {},
    PathDigest { path: String, digest: String },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceLevel {
    Transcript,
    Reported,
    Unmediated,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seat {
    pub id: String,
    pub required: bool,
    pub profession: String,
    pub evidence: EvidenceLevel,
    pub required_skills: Vec<Reference>,
    pub review_subject_node: String,
    /// A shared template is not a physical Worker Profile or delivered-content digest.
    pub template: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gate {
    pub quorum: u32,
    pub distinct_worker_profiles: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub artifact: Option<ArtifactEdge>,
    /// A switch edge names exactly one of the source switch cases.
    pub case: Option<String>,
    #[serde(default)]
    pub outcome: Outcome,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    #[default]
    Success,
    Failure,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactEdge {
    pub output: String,
    pub kind: OutputKind,
}
/// Exactly three lists; sources and discussion cannot be smuggled into readback material.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub decided: Vec<Decision>,
    pub unresolved: Vec<String>,
    pub out_of_scope: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    pub id: String,
    pub text: String,
    pub delivery_obligation: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SnapshotSource {
    Message { room_id: String, event: Reference },
    DirectText {},
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceMessage {
    pub room_id: String,
    pub event: Reference,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Production {
    Invocations { references: Vec<Reference> },
    None {},
    Missing {},
}
impl Default for Production {
    fn default() -> Self {
        Self::Missing {}
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub snapshot_source: SnapshotSource,
    pub source_chain: Vec<SourceMessage>,
    #[serde(default)]
    pub production: Production,
}
