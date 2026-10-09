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
    /// 平台上「归属 human」的账号：provider Done 只有映射到它才归一为完成请求。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub human_account: Option<String>,
    /// 绑定是否允许该供应端动作在预览无需临场选择时自动提交。
    #[serde(default)]
    pub auto_complete_provider_done: bool,
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
    /// 契约可事先声明这一项接受哪一种证据、最低证据通道。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<EvidenceRequirement>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRequirement {
    /// `integration_receipt` 表示按契约接受 Repo 模块的 Integration Receipt。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accept: Option<String>,
    /// 该项要求的最低证据通道：`unmediated` 或 `adapter_event`。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_channel: Option<String>,
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

/// Task-owned waiting state; Project owns the referenced Request lifecycle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestBlocker {
    pub owner: Reference,
    pub request_id: String,
    pub waiting: bool,
    pub delivery: Option<String>,
    pub outcome: Option<String>,
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
        /// 平台上归属该人的账号；缺省即不把任何 Done 归一到完成请求。
        #[serde(default, skip_serializing_if = "Option::is_none")]
        human_account: Option<String>,
        /// 是否允许该供应端动作在预览无需临场选择时自动提交。
        #[serde(default)]
        auto_complete_provider_done: bool,
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
    /// 完成 Task：先预览（逐项列出验收项、判定者、等级与证据），确认后同事务落 Receipt。
    Complete {
        project_id: String,
        task_id: String,
        version: i64,
        /// 空闲占用标记必须为空；完成会把它与 Receipt 一起在同一事务处理。
        lifecycle_version: i64,
        /// 精确 Task Revision（未采纳变化时，这就是「按当前 Revision 完成」的显式选择）。
        revision_number: i64,
        acceptance: Vec<ItemEvidence>,
    },
    /// 重开 Task：只接受有权 human actor，以预期生命周期版本把 完成/已取消 → 开放。
    Reopen {
        project_id: String,
        task_id: String,
        version: i64,
        lifecycle_version: i64,
        /// 存在未处理 drift 时必须显式给出要继续使用的当前 Revision。
        revision_number: Option<i64>,
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
    /// Withdraw an intent that has never been dispatched; the Task remains open.
    Withdraw {
        effect_id: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub key: String,
    pub action: Action,
}

/// Who judged one acceptance item. The receipt prints exactly this.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Judge {
    /// 工具箱直报（`hctl2-tool`）。
    Hctl2Tool,
    /// 供应端适配器直接观测到的变化（旁路证据）。
    Adapter { port: String },
    /// Gate 席位（第 9 包接）。
    Gate { seat: String },
    /// 有权的人。
    Human { actor: String },
}

/// One acceptance item's candidate evidence, submitted with 「完成 Task」.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemEvidence {
    /// 契约里验收项的序号（从 0 起）。
    pub item: usize,
    pub judge: Judge,
    /// 证据通道：`unmediated` | `adapter_event` | `narrated`。
    pub channel: String,
    /// Evidence / Verdict / Receipt 的精确引用。
    pub references: Vec<Reference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub producer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation: Option<i64>,
}

pub const COMPLETION_RECEIPT_KIND: &str = "task_completion_receipt";

pub const COMPLETION_REQUEST_KIND: &str = "task_completion_request";

/// 供应端 Done 归档成的「完成 Task」命令草稿。只有映射到归属 human 的账号、
/// 由非终态进入终态的**一次变化**才产生；control 用它走同一条预览与准入。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionRequest {
    pub request_id: String,
    pub task_id: String,
    pub project_id: String,
    pub repo_id: String,
    pub source_id: String,
    /// 绑定的精确版本（发出观测的那条源记录）。
    pub binding: Reference,
    /// 规范外部实体 ID。
    pub entity: String,
    pub provider_actor: String,
    pub stage_before: String,
    pub stage_after: String,
    pub remote_revision: String,
    /// 由上面这些规范字段组可重复计算，重复或迟到的投递落到同一条记录。
    pub idempotency_key: String,
    pub snapshot: Reference,
    pub observed_at: u64,
    /// `pending` | `accepted` | `declined`。
    pub state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<Value>,
}

/// Task Completion Receipt: immutable, written only by a successful 「完成 Task」.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionReceipt {
    pub receipt_id: String,
    pub task_id: String,
    pub project_id: String,
    pub command_id: String,
    pub idempotency_key: String,
    /// 这一次完成所依据的生命周期版本。
    pub lifecycle_version: i64,
    /// 精确 Task Revision。
    pub revision_number: i64,
    pub revision_digest: String,
    pub policy_digest: String,
    pub items: Vec<ItemOutcome>,
    pub completed_at: u64,
}

/// 逐项绑定：判定结果、校验等级、实际判定者、引用与摘要、来源。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemOutcome {
    pub item: usize,
    pub text: String,
    pub text_digest: String,
    pub grade: String,
    /// `passed` | `failed`。
    pub outcome: String,
    /// 实际采用的证据通道。
    pub validation_level: String,
    pub judge: Judge,
    pub references: Vec<Reference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub producer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_snapshot: Option<Reference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_head: Option<String>,
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
