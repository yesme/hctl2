//! Integration intents and Integration Receipts (第 6 包，集成一半).
//!
//! control persists the intent first; `hctl2-tool` (local targets) or the platform adapter
//! (remote targets) executes outside the Store transaction; only a readback that shows the
//! merge commit and the target head writes the one Receipt. Execution never runs in here.
//!
//! The source of every intent is an admitted ChangeSet Revision. Its admission belongs to the
//! other half of 第 6 包; this module only reads the record kind [`REVISION_KIND`].
use std::collections::BTreeMap;

use foundation::canonical_json_sha256;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use store::{
    Actor, ActorSource, Command, EffectIntent, EffectState, Expected, ObjectKey,
    Readback as EffectReadback, Record, RecordData, Reference, Scope, Store, TrustedActor, Version,
};

use crate::{Lifecycle, Platform, Registration, Result, reject, require_active};

/// Record kind written by ChangeSet Revision admission (`changeset.rs`, the other half of
/// 第 6 包) and read here. The record body is that module's `revision_body`.
pub const REVISION_KIND: &str = "changeset_revision";
pub const INTENT_KIND: &str = "integration_intent";
pub const RECEIPT_KIND: &str = "integration_receipt";

/// The admitted version an intent integrates, as the Repo module records it. Identity is the
/// five fields of `spec/repo.md` §ChangeSet 与 Git 事实; no commit object belongs to it.
/// The integration packages or finds a commit for `(base_commit_sha, result_tree_sha)` itself.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdmittedRevision {
    pub change_set_revision_id: String,
    pub change_set_id: String,
    #[serde(default)]
    pub parent_revision_id: Option<String>,
    pub base_commit_sha: String,
    pub result_tree_sha: String,
    pub producer_ref: Value,
    #[serde(default)]
    pub review_subject_digest: String,
}

pub fn revision_key(repo_id: &str, id: &str) -> ObjectKey {
    ObjectKey {
        scope: Scope::Repo(repo_id.into()),
        kind: REVISION_KIND.into(),
        id: id.into(),
    }
}

/// Read one admitted revision of this Repo.
pub fn revision(store: &Store, repo_id: &str, id: &str) -> Result<AdmittedRevision> {
    let record = store.get(&revision_key(repo_id, id))?.ok_or_else(|| {
        reject(
            "REVISION_NOT_ADMITTED",
            "ChangeSet Revision is not an admitted version of this Repo",
            "admit_revision_first",
        )
    })?;
    let RecordData::Value { value } = &record.data else {
        return Err(reject(
            "REVISION_NOT_ADMITTED",
            "revision record malformed",
            "inspect_repo",
        ));
    };
    let revision: AdmittedRevision = serde_json::from_value(value.clone())?;
    if revision.change_set_revision_id != id {
        return Err(reject(
            "REVISION_NOT_ADMITTED",
            "revision record identity disagrees with its key",
            "inspect_repo",
        ));
    }
    Ok(revision)
}

/// 测试缝：在 ChangeSet Revision 的准入（第 6 包另一半，#384）合入之前，让集成的用例有可引用的版本。
/// 写的是同一种记录；不是领域准入，不经 Invocation、租约或封存；control 不把它暴露成命令。
/// 另一半合入后，用例改走 `changeset::admit`，这个函数删除。
pub fn admit_revision_seam(
    store: &mut Store,
    actor: &TrustedActor,
    repo_id: &str,
    revision: &AdmittedRevision,
) -> Result<()> {
    let registration = require_active(store, repo_id)?;
    let actor = scoped(actor, &registration.repo_id)?;
    let key = revision_key(repo_id, &revision.change_set_revision_id);
    let value = serde_json::to_value(revision)?;
    let operation = "repo.revision_seam";
    let cmd = Command {
        command_id: format!("{operation}:{repo_id}:{}", key.id),
        idempotency_key: format!("{operation}:{repo_id}:{}", key.id),
        actor: actor.0.clone(),
        target: key.clone(),
        expected: Expected::Absent,
        binding: crate::binding(repo_id),
        input_digest: Command::digest_input(operation, &value)?,
        operation: operation.into(),
        input: value.clone(),
    };
    store.submit(store.generation(), &actor, &cmd, None, |tx| {
        tx.put(&Record {
            key: key.clone(),
            version: 1,
            revision_digest: canonical_json_sha256(&value)?,
            data: RecordData::Value {
                value: value.clone(),
            },
            sources: Vec::new(),
            materials: Vec::new(),
        })?;
        Ok(json!({"change_set_revision_id": key.id}))
    })?;
    Ok(())
}

/// Record kind written by 发布评审 (the other half of 第 6 包): which platform commit and review
/// request a ChangeSet Revision was published as. Read here to find what to ask the platform
/// to merge. Keyed by the revision id in the Repo scope.
pub const PLATFORM_BINDING_KIND: &str = "changeset_platform_binding";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReviewRequestRef {
    /// The review request's number on the platform.
    pub index: u64,
    /// The platform-side commit of the published revision; merges pin it as the source head.
    pub platform_commit_sha: String,
}

pub fn platform_binding_key(repo_id: &str, revision_id: &str) -> ObjectKey {
    ObjectKey {
        scope: Scope::Repo(repo_id.into()),
        kind: PLATFORM_BINDING_KIND.into(),
        id: revision_id.into(),
    }
}

/// The review request a revision was published to, if 发布评审 recorded one.
pub fn review_request(
    store: &Store,
    repo_id: &str,
    revision_id: &str,
) -> Result<Option<ReviewRequestRef>> {
    let Some(record) = store.get(&platform_binding_key(repo_id, revision_id))? else {
        return Ok(None);
    };
    let RecordData::Value { value } = &record.data else {
        return Ok(None);
    };
    let index = value["review_request"]["index"].as_u64();
    let commit = value["platform_commit_sha"].as_str();
    Ok(match (index, commit) {
        (Some(index), Some(commit)) => Some(ReviewRequestRef {
            index,
            platform_commit_sha: commit.to_owned(),
        }),
        _ => None,
    })
}

/// 测试缝：发布评审落地前，让集成的用例有可引用的映射。另一半合入后改走它的写入，这个函数删除。
pub fn admit_platform_binding_seam(
    store: &mut Store,
    actor: &TrustedActor,
    repo_id: &str,
    revision_id: &str,
    review: &ReviewRequestRef,
) -> Result<()> {
    let registration = require_active(store, repo_id)?;
    let actor = scoped(actor, &registration.repo_id)?;
    let key = platform_binding_key(repo_id, revision_id);
    let value = json!({
        "change_set_revision_id": revision_id,
        "platform_commit_sha": review.platform_commit_sha,
        "review_request": {"index": review.index},
    });
    let operation = "repo.platform_binding_seam";
    let cmd = Command {
        command_id: format!("{operation}:{repo_id}:{revision_id}"),
        idempotency_key: format!("{operation}:{repo_id}:{revision_id}"),
        actor: actor.0.clone(),
        target: key.clone(),
        expected: Expected::Absent,
        binding: crate::binding(repo_id),
        input_digest: Command::digest_input(operation, &value)?,
        operation: operation.into(),
        input: value.clone(),
    };
    store.submit(store.generation(), &actor, &cmd, None, |tx| {
        tx.put(&Record {
            key: key.clone(),
            version: 1,
            revision_digest: canonical_json_sha256(&value)?,
            data: RecordData::Value {
                value: value.clone(),
            },
            sources: Vec::new(),
            materials: Vec::new(),
        })?;
        Ok(json!({}))
    })?;
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    /// A ref in the registered local repository; only for Repos with no platform.
    Local,
    /// A ref on the bound platform (hosted Gitea or GitHub).
    Platform,
}

/// The two authorization forms of `spec/repo.md` §集成.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    /// The intent freezes the target head seen at preview; any other head rejects, no retry.
    ExpectedHead,
    /// The intent freezes source, strategy and protection snapshot; the target may advance.
    AcceptAdvance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    FastForward,
    MergeCommit,
}

impl Strategy {
    pub const fn tool_name(self) -> &'static str {
        match self {
            Self::FastForward => "fast-forward",
            Self::MergeCommit => "merge-commit",
        }
    }
}

/// What the human submits. No Execution Spec, no preview, no observation of the target.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub key: String,
    pub repo_id: String,
    pub change_set_revision_id: String,
    pub target_kind: TargetKind,
    /// Fully qualified ref, for example `refs/heads/main`.
    pub target_ref: String,
    pub form: Form,
    pub strategy: Strategy,
}

/// Target protection as read at preview and frozen into the intent (`spec/repo.md` §目标保护快照).
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtectionSnapshot {
    pub requires_review_request: bool,
    pub required_checks: Vec<String>,
    pub strict_sync: bool,
    pub require_conversation_resolution: bool,
    pub required_approvals: u64,
    /// Anything else the platform reports, kept for the human to compare.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub other: BTreeMap<String, Value>,
}

/// Facts control read from the target before admission. Reading happens outside the Store.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    /// Provider target reference: the local repository path, or `instance/full_name`.
    pub provider_ref: String,
    pub head: Option<String>,
    pub protection: Option<ProtectionSnapshot>,
    /// Evidence that later executions resolve the same target, not a same-named replacement
    /// (`spec/repo.md` §集成: 同路径新 clone 不因此成为原目标). Local targets freeze the Git
    /// common directory's filesystem identity; platform targets freeze the platform's stable id.
    #[serde(default)]
    pub continuity: Option<Value>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub kind: TargetKind,
    pub provider_ref: String,
    pub target_ref: String,
    /// Binding version in force at preview (platform targets only).
    pub binding: Option<Reference>,
    /// Frozen continuity evidence; execution compares before writing.
    #[serde(default)]
    pub continuity: Option<Value>,
}

/// Everything the intent freezes. Changing any field after preview means a new preview.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preview {
    pub input: Input,
    pub repo_version: i64,
    pub platform: Platform,
    pub source: AdmittedRevision,
    pub target: Target,
    pub form: Form,
    pub strategy: Strategy,
    /// Frozen only under [`Form::ExpectedHead`].
    pub expected_head: Option<String>,
    /// The head seen at preview under either form; informational for accept-advance.
    pub observed_head: Option<String>,
    pub protection: Option<ProtectionSnapshot>,
    pub checks: Vec<String>,
    pub effects: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentState {
    /// Admitted, never attempted.
    Pending,
    /// Attempted at least once without a confirming readback.
    Unknown,
    Succeeded,
    Failed,
}

/// A retryable rejection left by the last attempt: the human must act, the intent stays.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attention {
    pub code: String,
    pub message: String,
    pub recovery_action: String,
    pub details: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Intent {
    pub intent_id: String,
    pub repo_id: String,
    pub version: i64,
    pub command_id: String,
    pub idempotency_key: String,
    pub actor: Actor,
    pub preview: Preview,
    pub state: IntentState,
    pub attempts: u64,
    pub attention: Option<Attention>,
    pub failure: Option<Attention>,
    pub receipt_id: Option<String>,
    /// The exact executor input of the attempt in flight. Recorded before the executor runs and
    /// reused verbatim on retry, so a lost confirmation never re-plans against a moved target.
    #[serde(default)]
    pub attempt: Option<AttemptInput>,
}

/// What one executor attempt was given. Frozen per attempt, not recomputed on retry.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptInput {
    pub number: u64,
    /// Executor-side retry key; a new plan gets a new key, a retry keeps it.
    pub idempotency_key: String,
    pub commit: String,
    pub expected_head: String,
    /// Set the moment a request that may write has left for a platform. From then on the
    /// attempt is only read back, never resent: a platform refusal that proves no write
    /// happened clears it; a lost response does not.
    #[serde(default)]
    pub dispatched: bool,
}

impl AttemptInput {
    /// The executor input, regardless of whether it has been dispatched yet.
    #[must_use]
    pub fn same_input(&self, other: &Self) -> bool {
        self.number == other.number
            && self.idempotency_key == other.idempotency_key
            && self.commit == other.commit
            && self.expected_head == other.expected_head
    }
}

/// The only proof of integration. Immutable; written in the same transaction as the
/// effect confirmation and the intent's terminal state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub receipt_id: String,
    pub intent_id: String,
    pub repo_id: String,
    pub source: AdmittedRevision,
    pub target: Target,
    pub form: Form,
    pub strategy: Strategy,
    pub target_head_before: Option<String>,
    /// The actual target head after integration; under accept-advance it may differ from preview.
    pub target_head_after: String,
    pub integrated_commit: String,
    /// Tree of the integrated commit when the readback channel exposes it; `None` records that
    /// the platform did not (Gitea's REST API does not return tree ids).
    pub integrated_tree: Option<String>,
    /// Channel level of the readback that signed this Receipt (`hctl2-tool` or adapter readback).
    pub evidence_level: String,
    pub readback: Value,
    pub observed_at_unix_ms: u64,
}

/// Outcome of one execution attempt, as read back by the executor.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case", deny_unknown_fields)]
pub enum Outcome {
    Succeeded {
        target_head_before: Option<String>,
        target_head_after: String,
        integrated_commit: String,
        integrated_tree: Option<String>,
        evidence_level: String,
        readback: Value,
        observed_at_unix_ms: u64,
    },
    /// Terminal: the frozen intent can never apply (head drift under expected-head, bad source…).
    Failed(Attention),
    /// Not terminal: the human can clear the cause and retry the same intent.
    Attention(Attention),
    /// Neither confirmed nor refuted; the conflict scope stays occupied.
    Unknown(Attention),
}

pub fn intent_key(repo_id: &str, intent_id: &str) -> ObjectKey {
    ObjectKey {
        scope: Scope::Repo(repo_id.into()),
        kind: INTENT_KIND.into(),
        id: intent_id.into(),
    }
}

pub fn receipt_key(repo_id: &str, receipt_id: &str) -> ObjectKey {
    ObjectKey {
        scope: Scope::Repo(repo_id.into()),
        kind: RECEIPT_KIND.into(),
        id: receipt_id.into(),
    }
}

pub fn intent_id(control_id: &str, repo_id: &str, key: &str) -> String {
    format!(
        "integration-{}",
        foundation::bytes_sha256(format!("{control_id}\0{repo_id}\0{key}").as_bytes())
    )
}

pub fn effect_id(intent_id: &str) -> String {
    format!("integration:{intent_id}")
}

fn scoped(actor: &TrustedActor, repo_id: &str) -> Result<TrustedActor> {
    if !actor.0.permission_scope.contains(&Scope::Control) {
        return Err(reject(
            "PERMISSION_DENIED",
            "integration requires the control owner",
            "request_authorization",
        ));
    }
    let mut actor = actor.0.clone();
    if !actor
        .permission_scope
        .contains(&Scope::Repo(repo_id.into()))
    {
        actor.permission_scope.push(Scope::Repo(repo_id.into()));
    }
    Ok(TrustedActor(actor))
}

fn reducer(repo_id: &str, intent_id: &str) -> TrustedActor {
    TrustedActor(Actor {
        principal: format!("integration-reducer:{intent_id}"),
        source: ActorSource::InternalReducer,
        permission_scope: vec![Scope::Control, Scope::Repo(repo_id.into())],
        authority: Some(Reference {
            key: intent_key(repo_id, intent_id),
            version: Version::State(1),
        }),
    })
}

/// Freeze one preview from the human input, the active Repo and the observed target.
pub fn prepare(
    store: &Store,
    actor: &TrustedActor,
    input: Input,
    observation: Observation,
) -> Result<Preview> {
    scoped(actor, &input.repo_id)?;
    if input.key.trim().is_empty() {
        return Err(reject("INVALID_INPUT", "key required", "correct_input"));
    }
    if !input.target_ref.starts_with("refs/heads/") || input.target_ref.len() <= "refs/heads/".len()
    {
        return Err(reject(
            "INVALID_INPUT",
            "target_ref must be a fully qualified branch ref",
            "correct_input",
        ));
    }
    let registration = require_active(store, &input.repo_id)?;
    let source = revision(store, &input.repo_id, &input.change_set_revision_id)?;
    let (target, capabilities) = target_for(store, &registration, &input, &observation)?;
    let expected_head = match input.form {
        Form::ExpectedHead => {
            if input.target_kind == TargetKind::Platform
                && capabilities["expected_target_head"] != json!(true)
            {
                return Err(reject(
                    "EXPECTED_HEAD_UNSUPPORTED",
                    "this platform binding cannot guarantee the expected target head; choose accept_advance explicitly",
                    "choose_accept_advance_form",
                ));
            }
            Some(observation.head.clone().ok_or_else(|| {
                reject(
                    "TARGET_HEAD_UNKNOWN",
                    "target ref has no head to freeze",
                    "create_target_ref_or_choose_another",
                )
            })?)
        }
        Form::AcceptAdvance => None,
    };
    if input.target_kind == TargetKind::Platform && observation.protection.is_none() {
        return Err(reject(
            "PROTECTION_UNREAD",
            "platform target protection was not read; nothing to freeze",
            "retry_preview",
        ));
    }
    let mut checks = vec![
        "source commit present in the target repository with exactly result_tree_sha".into(),
        "base_commit_sha is an ancestor of the target head at execution".into(),
    ];
    let mut effects = Vec::new();
    match input.form {
        Form::ExpectedHead => {
            checks.push(
                "target head equals the frozen expected_head, otherwise reject without retry"
                    .into(),
            );
        }
        Form::AcceptAdvance => {
            checks.push("target may advance; the Receipt records the actual head".into());
        }
    }
    match input.target_kind {
        TargetKind::Local => {
            checks.push("target ref is not checked out by any worktree, otherwise wait for the human to switch away".into());
            effects.push(format!(
                "advance {} in {} by compare-and-swap ({:?})",
                input.target_ref, target.provider_ref, input.strategy
            ));
        }
        TargetKind::Platform => {
            checks.push("platform protection at execution matches the frozen snapshot".into());
            effects.push(format!(
                "ask the platform to merge into {} on {}",
                input.target_ref, target.provider_ref
            ));
        }
    }
    effects.push(
        "write one Integration Receipt only after readback shows the merge commit and target head"
            .into(),
    );
    Ok(Preview {
        repo_version: registration.version,
        platform: registration.prepared.platform.clone(),
        source,
        target,
        form: input.form,
        strategy: input.strategy,
        expected_head,
        observed_head: observation.head,
        protection: observation.protection,
        checks,
        effects,
        input,
    })
}

/// The target plus, for platform targets, the binding's declared capabilities.
fn target_for(
    store: &Store,
    registration: &Registration,
    input: &Input,
    observation: &Observation,
) -> Result<(Target, Value)> {
    match (&registration.prepared.platform, input.target_kind) {
        (Platform::None, TargetKind::Local) => {
            if observation.continuity.is_none() {
                return Err(reject(
                    "TARGET_CONTINUITY_UNREAD",
                    "local target identity was not read; nothing to freeze",
                    "retry_preview",
                ));
            }
            Ok((
                Target {
                    kind: TargetKind::Local,
                    provider_ref: observation.provider_ref.clone(),
                    target_ref: input.target_ref.clone(),
                    binding: None,
                    continuity: observation.continuity.clone(),
                },
                Value::Null,
            ))
        }
        (Platform::None, TargetKind::Platform) => Err(reject(
            "PLATFORM_NOT_BOUND",
            "this Repo has no platform; only local targets exist",
            "choose_local_target",
        )),
        (_, TargetKind::Local) => Err(reject(
            "LOCAL_TARGET_NOT_ALLOWED",
            "a platform-bound Repo integrates on the platform, not a local ref",
            "choose_platform_target",
        )),
        (_, TargetKind::Platform) => {
            if registration.lifecycle != Lifecycle::Active || registration.observed.is_none() {
                return Err(reject(
                    "REPO_PENDING",
                    "platform binding not confirmed",
                    "confirm_repo",
                ));
            }
            let binding = crate::binding(&registration.repo_id);
            let record = store.get(&binding.key)?.ok_or_else(|| {
                reject(
                    "REPO_PENDING",
                    "platform binding record missing",
                    "confirm_repo",
                )
            })?;
            let RecordData::Value { value } = &record.data else {
                return Err(reject(
                    "REPO_PENDING",
                    "platform binding malformed",
                    "inspect_repo",
                ));
            };
            let capabilities = value["capabilities"].clone();
            // Declared capabilities gate the command; a missing one rejects instead of guessing.
            for required in ["remote_merge", "protection_readback"] {
                if capabilities[required] != json!(true) {
                    return Err(reject(
                        "CAPABILITY_MISSING",
                        format!(
                            "platform binding does not declare {required}; verify the platform before integrating there"
                        ),
                        "verify_platform_capability",
                    ));
                }
            }
            Ok((
                Target {
                    kind: TargetKind::Platform,
                    provider_ref: observation.provider_ref.clone(),
                    target_ref: input.target_ref.clone(),
                    binding: Some(Reference {
                        key: binding.key,
                        version: Version::State(record.version),
                    }),
                    continuity: observation.continuity.clone(),
                },
                capabilities,
            ))
        }
    }
}

fn conflict_scope(target: &Target) -> String {
    format!(
        "integration:{:?}:{}:{}",
        target.kind, target.provider_ref, target.target_ref
    )
}

fn effect(intent: &Intent) -> Result<EffectIntent> {
    let input = serde_json::to_value(&intent.preview)?;
    let operation = format!("integration.{:?}", intent.preview.target.kind).to_lowercase();
    Ok(EffectIntent {
        intent_id: effect_id(&intent.intent_id),
        owner: Reference {
            key: intent_key(&intent.repo_id, &intent.intent_id),
            version: Version::State(1),
        },
        binding: crate::binding(&intent.repo_id),
        target: format!(
            "{}:{}",
            intent.preview.target.provider_ref, intent.preview.target.target_ref
        ),
        conflict_scope: conflict_scope(&intent.preview.target),
        permission_scope: Scope::Repo(intent.repo_id.clone()),
        input_digest: Command::digest_input(&operation, &input)?,
        operation,
        input,
        idempotency_key: effect_id(&intent.intent_id),
    })
}

fn record(intent: &Intent) -> Result<Record> {
    let value = serde_json::to_value(intent)?;
    Ok(Record {
        key: intent_key(&intent.repo_id, &intent.intent_id),
        version: intent.version,
        revision_digest: canonical_json_sha256(&value)?,
        data: RecordData::Value { value },
        sources: vec![Reference {
            key: revision_key(
                &intent.repo_id,
                &intent.preview.source.change_set_revision_id,
            ),
            version: Version::State(1),
        }],
        materials: Vec::new(),
    })
}

pub fn get(store: &Store, repo_id: &str, intent_id: &str) -> Result<Intent> {
    let record = store.get(&intent_key(repo_id, intent_id))?.ok_or_else(|| {
        reject(
            "INTENT_NOT_FOUND",
            "integration intent not found",
            "list_integrations",
        )
    })?;
    let RecordData::Value { value } = &record.data else {
        return Err(reject(
            "INTENT_NOT_FOUND",
            "intent record malformed",
            "inspect_repo",
        ));
    };
    Ok(serde_json::from_value(value.clone())?)
}

pub fn list(store: &Store, repo_id: &str) -> Result<Vec<Intent>> {
    let mut intents = Vec::new();
    for record in store.list(INTENT_KIND)? {
        if record.key.scope != Scope::Repo(repo_id.into()) {
            continue;
        }
        if let RecordData::Value { value } = &record.data {
            intents.push(serde_json::from_value(value.clone())?);
        }
    }
    Ok(intents)
}

/// Every intent that still owes an external result, across all Repos.
pub fn open(store: &Store) -> Result<Vec<Intent>> {
    let mut intents = Vec::new();
    for record in store.list(INTENT_KIND)? {
        if let RecordData::Value { value } = &record.data {
            let intent: Intent = serde_json::from_value(value.clone())?;
            if matches!(intent.state, IntentState::Pending | IntentState::Unknown) {
                intents.push(intent);
            }
        }
    }
    Ok(intents)
}

pub fn receipt(store: &Store, repo_id: &str, receipt_id: &str) -> Result<Receipt> {
    let record = store
        .get(&receipt_key(repo_id, receipt_id))?
        .ok_or_else(|| {
            reject(
                "RECEIPT_NOT_FOUND",
                "Integration Receipt not found",
                "list_integrations",
            )
        })?;
    let RecordData::Value { value } = &record.data else {
        return Err(reject(
            "RECEIPT_NOT_FOUND",
            "receipt record malformed",
            "inspect_repo",
        ));
    };
    Ok(serde_json::from_value(value.clone())?)
}

/// Admit the frozen preview: persist the intent and its effect in one transaction.
///
/// Same key with the same preview returns the existing intent; another pending or unknown
/// intent on the same target rejects with `TARGET_BUSY` (the Store's conflict scope).
pub fn submit(
    store: &mut Store,
    actor: &TrustedActor,
    command_id: &str,
    preview: Preview,
) -> Result<Intent> {
    let actor = scoped(actor, &preview.input.repo_id)?;
    let registration = require_active(store, &preview.input.repo_id)?;
    if registration.version != preview.repo_version {
        return Err(reject(
            "VERSION_CONFLICT",
            "Repo changed since the preview",
            "rebuild_preview",
        ));
    }
    // The source must still be the admitted version the preview froze.
    let current = revision(
        store,
        &preview.input.repo_id,
        &preview.input.change_set_revision_id,
    )?;
    if current != preview.source {
        return Err(reject(
            "VERSION_CONFLICT",
            "source revision changed since the preview",
            "rebuild_preview",
        ));
    }
    let id = intent_id(
        store.control_id(),
        &preview.input.repo_id,
        &preview.input.key,
    );
    if let Some(existing) = store.get(&intent_key(&preview.input.repo_id, &id))? {
        let RecordData::Value { value } = &existing.data else {
            return Err(reject(
                "INTENT_NOT_FOUND",
                "intent record malformed",
                "inspect_repo",
            ));
        };
        let existing: Intent = serde_json::from_value(value.clone())?;
        if existing.preview != preview {
            return Err(reject(
                "IDEMPOTENCY_CONFLICT",
                "same integration key with a different preview",
                "use_new_command_key",
            ));
        }
        return Ok(existing);
    }
    let intent = Intent {
        intent_id: id.clone(),
        repo_id: preview.input.repo_id.clone(),
        version: 1,
        command_id: command_id.into(),
        idempotency_key: preview.input.key.clone(),
        actor: actor.0.clone(),
        preview,
        state: IntentState::Pending,
        attempts: 0,
        attention: None,
        failure: None,
        receipt_id: None,
        attempt: None,
    };
    let input = serde_json::to_value(&intent.preview)?;
    let operation = "integration.submit";
    let cmd = Command {
        command_id: command_id.into(),
        idempotency_key: intent.idempotency_key.clone(),
        actor: actor.0.clone(),
        target: intent_key(&intent.repo_id, &id),
        expected: Expected::Absent,
        binding: crate::binding(&intent.repo_id),
        input_digest: Command::digest_input(operation, &input)?,
        operation: operation.into(),
        input,
    };
    let effect = effect(&intent)?;
    let result = store.submit(store.generation(), &actor, &cmd, None, |tx| {
        tx.put(&record(&intent)?)?;
        tx.enqueue_effect(&effect)?;
        Ok(json!({"intent_id": id}))
    });
    match result {
        Ok(_) => get(store, &intent.repo_id, &id),
        Err(error) if error.code == "EFFECT_CONFLICT" => {
            let mut holder = None;
            for pending in store.pending_effects()? {
                let (other, _) = store.effect(&pending)?;
                if other.conflict_scope == effect.conflict_scope {
                    holder = Some(other.owner.key.id);
                }
            }
            Err(reject(
                "TARGET_BUSY",
                format!(
                    "target already has an unresolved integration intent{}; it must reach a terminal state first",
                    holder.map(|h| format!(" ({h})")).unwrap_or_default()
                ),
                "inspect_original_intent",
            ))
        }
        Err(error) => Err(error),
    }
}

/// Re-check the original authorization and move a never-attempted effect into delivery.
/// Returns the effect state before this call: `Pending` means this attempt is the first.
pub fn begin(store: &mut Store, repo_id: &str, intent_id: &str) -> Result<(Intent, EffectState)> {
    let intent = get(store, repo_id, intent_id)?;
    if !matches!(intent.state, IntentState::Pending | IntentState::Unknown) {
        return Err(reject(
            "INTENT_TERMINAL",
            "integration intent already settled",
            "inspect_intent",
        ));
    }
    let registration = require_active(store, repo_id)?;
    if registration.version != intent.preview.repo_version {
        return Err(reject(
            "FROZEN_INPUT_CHANGED",
            "Repo changed since admission",
            "rebuild_preview",
        ));
    }
    let id = effect_id(intent_id);
    let (effect_record, state) = store.effect(&id)?;
    let still_authorized = effect_record.owner.key == intent_key(repo_id, intent_id)
        && effect_record.input == serde_json::to_value(&intent.preview)?;
    if !still_authorized {
        return Err(reject(
            "FROZEN_INPUT_CHANGED",
            "effect differs from the admitted intent",
            "inspect_intent",
        ));
    }
    if state == EffectState::Pending {
        store.resume_pending_effect(store.generation(), &id, true)?;
        store.begin_effect(store.generation(), &id)?;
    }
    Ok((intent, state))
}

/// Freeze the executor input of the next attempt before anything runs. Idempotent for the same
/// input; a different input while one is already recorded is refused.
pub fn record_attempt(
    store: &mut Store,
    repo_id: &str,
    intent_id: &str,
    attempt: AttemptInput,
) -> Result<Intent> {
    let mut intent = get(store, repo_id, intent_id)?;
    if !matches!(intent.state, IntentState::Pending | IntentState::Unknown) {
        return Err(reject(
            "INTENT_TERMINAL",
            "integration intent already settled",
            "inspect_intent",
        ));
    }
    match &intent.attempt {
        Some(existing) if existing.same_input(&attempt) => return Ok(intent),
        Some(_) => {
            return Err(reject(
                "ATTEMPT_IN_FLIGHT",
                "an attempt with other executor input is recorded; read it back before planning anew",
                "read_back_recorded_attempt",
            ));
        }
        None => {}
    }
    intent.attempt = Some(attempt);
    bump(store, repo_id, intent_id, intent, "integration.attempt")
}

/// Record whether the recorded attempt's request has left for the platform. `true` before a
/// request that may write is sent; `false` again only when the platform refused it before
/// writing. Idempotent; no attempt recorded is an error.
pub fn mark_attempt_dispatched(
    store: &mut Store,
    repo_id: &str,
    intent_id: &str,
    number: u64,
    dispatched: bool,
) -> Result<Intent> {
    let mut intent = get(store, repo_id, intent_id)?;
    let Some(attempt) = intent.attempt.as_mut() else {
        return Err(reject(
            "ATTEMPT_NOT_RECORDED",
            "no attempt is recorded for this intent",
            "record_attempt_first",
        ));
    };
    if attempt.number != number {
        return Err(reject(
            "ATTEMPT_MISMATCH",
            "the recorded attempt is not the one being dispatched",
            "read_back_recorded_attempt",
        ));
    }
    if attempt.dispatched == dispatched {
        return Ok(intent);
    }
    attempt.dispatched = dispatched;
    bump(
        store,
        repo_id,
        intent_id,
        intent,
        if dispatched {
            "integration.attempt_dispatched"
        } else {
            "integration.attempt_refused"
        },
    )
}

/// Drop a recorded attempt after the executor proved it wrote nothing, so the next attempt
/// may plan against the current target (accept-advance only).
pub fn clear_attempt(store: &mut Store, repo_id: &str, intent_id: &str) -> Result<Intent> {
    let mut intent = get(store, repo_id, intent_id)?;
    if intent.attempt.is_none() {
        return Ok(intent);
    }
    intent.attempt = None;
    bump(
        store,
        repo_id,
        intent_id,
        intent,
        "integration.attempt_cleared",
    )
}

fn bump(
    store: &mut Store,
    repo_id: &str,
    intent_id: &str,
    mut intent: Intent,
    operation: &str,
) -> Result<Intent> {
    let actor = reducer(repo_id, intent_id);
    intent.version += 1;
    let input = json!({"version": intent.version, "attempt": intent.attempt});
    let cmd = Command {
        command_id: format!("{operation}:{intent_id}:{}", intent.version),
        idempotency_key: format!("{operation}:{intent_id}:{}", intent.version),
        actor: actor.0.clone(),
        target: intent_key(repo_id, intent_id),
        expected: Expected::Exact(Version::State(intent.version - 1)),
        binding: crate::binding(repo_id),
        input_digest: Command::digest_input(operation, &input)?,
        operation: operation.into(),
        input,
    };
    store.submit(store.generation(), &actor, &cmd, None, |tx| {
        tx.put(&record(&intent)?)?;
        Ok(json!({"version": intent.version}))
    })?;
    get(store, repo_id, intent_id)
}

/// Record one attempt's readback. Only `Succeeded` writes a Receipt; it, the effect
/// confirmation and the terminal state land in one transaction.
pub fn confirm(
    store: &mut Store,
    repo_id: &str,
    intent_id: &str,
    outcome: Outcome,
) -> Result<Intent> {
    let mut intent = get(store, repo_id, intent_id)?;
    if !matches!(intent.state, IntentState::Pending | IntentState::Unknown) {
        return Err(reject(
            "INTENT_TERMINAL",
            "integration intent already settled",
            "inspect_intent",
        ));
    }
    let actor = reducer(repo_id, intent_id);
    let effect = effect(&intent)?;
    let attempt = intent.attempts + 1;
    let (operation, readback, receipt) = match outcome {
        Outcome::Succeeded {
            target_head_before,
            target_head_after,
            integrated_commit,
            integrated_tree,
            evidence_level,
            readback,
            observed_at_unix_ms,
        } => {
            if intent.preview.strategy == Strategy::FastForward
                && integrated_tree.as_deref()
                    != Some(intent.preview.source.result_tree_sha.as_str())
            {
                return Err(reject(
                    "READBACK_MISMATCH",
                    "fast-forward readback tree differs from the admitted result tree",
                    "read_back_original_intent",
                ));
            }
            if let Some(expected) = &intent.preview.expected_head
                && target_head_before
                    .as_deref()
                    .is_some_and(|before| before != expected.as_str())
            {
                return Err(reject(
                    "READBACK_MISMATCH",
                    "expected-head intent reported a different head before integration",
                    "read_back_original_intent",
                ));
            }
            let receipt = Receipt {
                receipt_id: format!("receipt-{}", &intent.intent_id["integration-".len()..]),
                intent_id: intent.intent_id.clone(),
                repo_id: repo_id.into(),
                source: intent.preview.source.clone(),
                target: intent.preview.target.clone(),
                form: intent.preview.form,
                strategy: intent.preview.strategy,
                target_head_before,
                target_head_after: target_head_after.clone(),
                integrated_commit,
                integrated_tree,
                evidence_level,
                readback: readback.clone(),
                observed_at_unix_ms,
            };
            intent.state = IntentState::Succeeded;
            intent.attention = None;
            intent.attempt = None;
            intent.receipt_id = Some(receipt.receipt_id.clone());
            (
                "integration.succeeded",
                EffectReadback::Confirmed {
                    binding: effect.binding.clone(),
                    target: effect.target.clone(),
                    input_digest: effect.input_digest.clone(),
                    result: json!({"receipt_id": receipt.receipt_id, "target_head_after": target_head_after}),
                },
                Some(receipt),
            )
        }
        Outcome::Failed(reason) => {
            intent.state = IntentState::Failed;
            intent.attention = None;
            intent.attempt = None;
            intent.failure = Some(reason.clone());
            (
                "integration.failed",
                EffectReadback::Rejected {
                    binding: effect.binding.clone(),
                    target: effect.target.clone(),
                    input_digest: effect.input_digest.clone(),
                    result: serde_json::to_value(&reason)?,
                },
                None,
            )
        }
        Outcome::Attention(reason) => {
            intent.state = IntentState::Unknown;
            intent.attention = Some(reason);
            ("integration.attention", EffectReadback::Unknown, None)
        }
        Outcome::Unknown(reason) => {
            intent.state = IntentState::Unknown;
            intent.attention = Some(reason);
            ("integration.unknown", EffectReadback::Unknown, None)
        }
    };
    intent.attempts = attempt;
    intent.version += 1;
    let input = json!({"attempt": attempt, "readback": readback_summary(&readback)});
    let cmd = Command {
        command_id: format!("{operation}:{intent_id}:{attempt}"),
        idempotency_key: format!("{operation}:{intent_id}:{attempt}"),
        actor: actor.0.clone(),
        target: intent_key(repo_id, intent_id),
        expected: Expected::Exact(Version::State(intent.version - 1)),
        binding: crate::binding(repo_id),
        input_digest: Command::digest_input(operation, &input)?,
        operation: operation.into(),
        input,
    };
    let effect_key = effect_id(intent_id);
    store.submit(store.generation(), &actor, &cmd, None, |tx| {
        tx.confirm_effect(&effect_key, &readback)?;
        if let Some(receipt) = &receipt {
            let value = serde_json::to_value(receipt)?;
            tx.put(&Record {
                key: receipt_key(repo_id, &receipt.receipt_id),
                version: 1,
                revision_digest: canonical_json_sha256(&value)?,
                data: RecordData::Value { value },
                sources: vec![Reference {
                    key: intent_key(repo_id, intent_id),
                    version: Version::State(1),
                }],
                materials: Vec::new(),
            })?;
        }
        tx.put(&record(&intent)?)?;
        Ok(json!({"state": intent.state}))
    })?;
    get(store, repo_id, intent_id)
}

fn readback_summary(readback: &EffectReadback) -> Value {
    match readback {
        EffectReadback::Unknown => json!("unknown"),
        EffectReadback::Confirmed { result, .. } => json!({"confirmed": result}),
        EffectReadback::Rejected { result, .. } => json!({"rejected": result}),
    }
}

/// Human-facing view: the intent, its Receipt when any, and the effect's outbox state.
pub fn show(store: &Store, repo_id: &str, intent_id: &str) -> Result<Value> {
    let intent = get(store, repo_id, intent_id)?;
    let receipt = intent
        .receipt_id
        .as_deref()
        .map(|id| receipt(store, repo_id, id))
        .transpose()?;
    let effect_state = store
        .effect(&effect_id(intent_id))
        .map(|(_, state)| state)
        .ok();
    Ok(json!({
        "intent": intent,
        "receipt": receipt,
        "effect_state": effect_state,
    }))
}
