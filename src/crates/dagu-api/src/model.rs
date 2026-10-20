//! Controlled workflow engine port models, `Run–Engine Binding`, observations, and divergence records.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use store::Reference;

/// Authoritative Run lifecycle states relevant to the workflow engine port (`spec/run.md` §写入约束).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
    Starting,
    Running,
    Pausing,
    Paused,
    Cancelling,
    Completed,
    Failed,
    Cancelled,
    Superseded,
}

/// Authoritative `Run–Engine Binding` lifecycle states (`spec/run.md` §写入约束 第三行).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BindingState {
    Starting,
    Bound,
    Closed,
    Diverged,
}

/// Direct modifications on Dagu UI/API or engine stalls that mark `Run–Engine Binding` as `Diverged`
/// without back-propagating Run commands, Verdicts, or Receipts (`CT-RUN #9`, `CT-RUN #18`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DirectOperationKind {
    Start,
    Stop,
    Retry,
    Reschedule,
    Approve,
    Reject,
    Edit,
    Rename,
    Delete,
    EngineProgressStalled,
}

/// Recorded divergence details on a `Run–Engine Binding`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DivergenceRecord {
    pub kind: DirectOperationKind,
    pub detail: String,
    pub observed_at_ms: u64,
}

/// Immutable `Run` governance record managed exclusively by control commands (`CT-RUN #8`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlledRun {
    pub project_id: String,
    pub run_id: String,
    pub run_version: i64,
    pub workflow_id: String,
    pub workflow_revision: i64,
    pub deployment: Reference,
    pub state: RunState,
    pub needs_attention: bool,
    pub cancel_intent_persisted: bool,
    pub authority_revoked: bool,
    pub completed_checkpoints: Vec<String>,
    pub verdicts: Vec<String>,
    pub receipts: Vec<String>,
}

/// Authorized checkpoint completion outbox persisted before driving Dagu (`spec/run.md` §从节点到结果 #5).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizedCheckpointCompletion {
    pub step_id: String,
    pub obligation_id: String,
    pub attempt_generation: i64,
    pub verdict: String,
    pub receipt_id: String,
    pub persisted_at_ms: u64,
    pub engine_acknowledged: bool,
}

/// `Run–Engine Binding` mapping a `Run` and `EngineDeployment` to a Dagu execution (`spec/run.md` §对象).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunEngineBinding {
    pub project_id: String,
    pub binding_id: String,
    pub run_id: String,
    pub deployment: Reference,
    pub port_provider_binding: String,
    pub dag_name: String,
    pub dag_run_id: String,
    pub correlation_key: String,
    pub expected_spec_digest: String,
    pub engine_binding_generation: i64,
    pub state: BindingState,
    pub last_observation_seq: u64,
    pub cursor: u64,
    pub last_progress_at_ms: u64,
    pub progress_timeout_ms: u64,
    pub observed_step_waiting_counts: BTreeMap<String, i64>,
    pub authorized_step_retries: BTreeMap<String, i64>,
    pub authorized_completions: BTreeMap<String, AuthorizedCheckpointCompletion>,
    pub divergence: Option<DivergenceRecord>,
}

/// Monotonic observation of an engine checkpoint entering `Waiting(7)` (`CT-RUN #11, #18`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointObservation {
    pub project_id: String,
    pub binding_id: String,
    pub run_id: String,
    pub dag_name: String,
    pub dag_run_id: String,
    pub step_name: String,
    pub observation_seq: u64,
    pub cursor: u64,
    pub status: i64,
    pub status_label: String,
    pub done_count: i64,
    pub retry_count: i64,
    pub human_task_prompt: Option<String>,
    pub from_cache: bool,
    pub late_event: bool,
    pub stale_cursor: bool,
    pub admissible_for_obligation: bool,
    pub rejection_reason: Option<String>,
    pub observed_at_ms: u64,
}

/// Readback of a single step from `GET /dag-runs/{name}/{dagRunId}` (`.dagRunDetails.nodes[]`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineStepReadback {
    pub step_name: String,
    pub status: i64,
    pub status_label: String,
    pub started_at: String,
    pub finished_at: String,
    pub retry_count: i64,
    pub done_count: i64,
    pub human_task_prompt: Option<String>,
    pub human_task_resume_pending: Option<bool>,
    pub approved_at: Option<String>,
    pub approved_by: Option<String>,
    pub rejected_at: Option<String>,
    pub rejected_by: Option<String>,
    pub rejection_reason: Option<String>,
    pub output_variables: BTreeMap<String, String>,
}

/// Readback of a DAG run and its surrounding DAG state in Dagu.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineRunReadback {
    pub dag_name: String,
    pub dag_run_id: String,
    pub status: i64,
    pub status_label: String,
    pub started_at: String,
    pub finished_at: String,
    pub queued_at: String,
    pub spec_digest: Option<String>,
    pub all_run_ids: Vec<String>,
    pub existing_dag_files: Vec<String>,
    pub steps: Vec<EngineStepReadback>,
}

/// Request to complete a `human.task` checkpoint in Dagu.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanTaskCompletionRequest {
    pub dag_name: String,
    pub dag_run_id: String,
    pub step_id: String,
    pub obligation_id: String,
    pub attempt_generation: i64,
    pub verdict: String,
    pub receipt_id: String,
    pub response_note: Option<String>,
}

impl HumanTaskCompletionRequest {
    /// Convert the governance fields into the 4 required Dagu `human.task` form values.
    #[must_use]
    pub fn form_values(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("obligation_id".to_owned(), self.obligation_id.clone()),
            (
                "attempt_generation".to_owned(),
                self.attempt_generation.to_string(),
            ),
            ("verdict".to_owned(), self.verdict.clone()),
            ("receipt_id".to_owned(), self.receipt_id.clone()),
        ])
    }

    /// Convert the governance fields into the typed JSON map expected by Dagu's
    /// `POST /dag-runs/{name}/{dagRunId}/human-tasks/{stepId}/complete` schema.
    #[must_use]
    pub fn to_human_task_input(&self) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        map.insert(
            "obligation_id".to_owned(),
            serde_json::Value::String(self.obligation_id.clone()),
        );
        map.insert(
            "attempt_generation".to_owned(),
            serde_json::Value::Number(self.attempt_generation.into()),
        );
        map.insert(
            "verdict".to_owned(),
            serde_json::Value::String(self.verdict.clone()),
        );
        map.insert(
            "receipt_id".to_owned(),
            serde_json::Value::String(self.receipt_id.clone()),
        );
        map
    }
}

/// Action taken after reading back a step whose prior `human.task` completion acknowledgement was unknown.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionRecoveryAction {
    SubmittedDirectly,
    ResubmittedAfterWaitingReadback,
    ResumedPendingAfterReadback,
    ConfirmedAdvancedWithoutResubmit,
}

/// Outcome of completing a `human.task` checkpoint (including unknown-ack readback-before-retry).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompleteResolution {
    pub dag_name: String,
    pub dag_run_id: String,
    pub step_id: String,
    pub readback_performed: bool,
    pub readback_status_before: Option<i64>,
    pub resubmitted: bool,
    pub action_taken: CompletionRecoveryAction,
}

/// Outcome of stopping a Dagu run (`docs/research/sdk/dagu.md` §2026-10-19 复核记录).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopOutcome {
    StoppedRunningProcess,
    ParkedWaitingNoProcess,
}
