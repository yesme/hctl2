//! Review publishing (第 6 包 · 验收第 4 条、第 9 条发布半边): a persisted intent with two
//! execution stages — push the exact admitted revision to the policy's branch, then create or
//! update the review request on the platform and record which commit and request the
//! revision maps to. Nothing here touches Git or a platform; control executes and reads back.
//!
//! The intent is enqueued in the same Store transaction as the revision's admission, with the
//! actor envelope of the human command that authorized the write (`spec/repo.md` §发布评审).
//! One ChangeSet has at most one in-flight publish (the Store's conflict scope); a newer
//! revision admitted meanwhile replaces what the next attempt publishes.
use foundation::{bytes_sha256, canonical_json_sha256};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use store::{
    Actor, ActorSource, Command, CommandTransaction, EffectIntent, EffectState, Expected,
    ObjectKey, Readback as EffectReadback, Record, RecordData, Reference, Scope, Store,
    TrustedActor, Version,
};

use crate::changeset::{ChangeSetRevision, ProducerRef};
use crate::integration::{PLATFORM_BINDING_KIND, platform_binding_key};
use crate::{Result, reject};

pub const POLICY_KIND: &str = "review_publish_policy";
pub const INTENT_KIND: &str = "review_publish_intent";

/// The review publishing policy a write-type dispatch freezes (`spec/project.md` §Room
/// Invocation). Everything the publish may do is fixed here; the proposal cannot widen it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub repo_id: String,
    /// The platform binding version the policy was frozen against.
    pub binding_version: u64,
    /// Branch the revision is pushed to; `{change_set}` stands for the ChangeSet id.
    pub branch_rule: String,
    /// The branch the review request asks to merge into.
    pub target_branch: String,
    /// `false`: the first revision creates the request, later revisions are refused.
    pub allow_update: bool,
    /// Where the request's description comes from: `none` (the audit association only).
    pub description_source: String,
    /// Open: the intent waits for a human `review publish`; closed: control publishes.
    pub requires_human_confirmation: bool,
    /// What of the audit association goes into the request: `minimal`.
    pub audit_scope: String,
}

/// A policy frozen into the Store, referenced by id, version and digest from Execution Specs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenPolicy {
    pub policy_id: String,
    pub version: i64,
    pub digest: String,
    pub policy: Policy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    /// The policy wants a human to confirm; nothing is queued yet.
    PendingHuman,
    Pending,
    Unknown,
    Published,
    Failed,
}

/// The revision an attempt publishes and the commit it is packaged as, frozen before the
/// push leaves so a retry pushes the same object.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub change_set_revision_id: String,
    pub base_commit_sha: String,
    pub result_tree_sha: String,
    pub commit_sha: Option<String>,
}

/// Stage 1: the exact commit on the policy's branch, confirmed by reading the remote back.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PushStage {
    /// Set before a push leaves; only a refusal that proves no write takes it back.
    pub dispatched: bool,
    pub confirmed_commit: Option<String>,
    pub confirmed_at_unix_ms: Option<u64>,
}

/// Stage 2: the review request that carries the branch, confirmed by reading it back.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewStage {
    pub dispatched: bool,
    pub index: Option<u64>,
    pub confirmed_commit: Option<String>,
    pub confirmed_at_unix_ms: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attention {
    pub code: String,
    pub message: String,
    pub recovery_action: String,
    pub details: Value,
}

/// The persisted publish intent of one ChangeSet under one frozen policy.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Intent {
    pub intent_id: String,
    pub repo_id: String,
    pub change_set_id: String,
    pub policy: FrozenPolicy,
    pub branch: String,
    /// The human command whose authorization this publish continues, and its actor.
    pub authorized_by: ProducerRef,
    pub authorizing_actor: Actor,
    pub state: State,
    /// Publish rounds: one per revision that reached the platform or is on its way.
    pub round: u64,
    /// The platform binding version the ChangeSet was opened under; frozen here so a rebind
    /// cannot carry an old intent to a new endpoint.
    #[serde(default)]
    pub binding_version: u64,
    pub target: Target,
    /// True once the worker took this round for execution: from then on the target is
    /// immutable and a newer revision waits in `queued` until the round settles.
    #[serde(default)]
    pub started: bool,
    /// The newest admitted revision waiting for the current round to settle.
    #[serde(default)]
    pub queued: Option<Target>,
    pub push: PushStage,
    pub review: ReviewStage,
    pub attempts: u64,
    pub attention: Option<Attention>,
    pub failure: Option<Attention>,
    pub version: i64,
}

pub fn intent_key(repo_id: &str, intent_id: &str) -> ObjectKey {
    ObjectKey {
        scope: Scope::Repo(repo_id.into()),
        kind: INTENT_KIND.into(),
        id: intent_id.into(),
    }
}

pub fn policy_key(repo_id: &str, policy_id: &str) -> ObjectKey {
    ObjectKey {
        scope: Scope::Repo(repo_id.into()),
        kind: POLICY_KIND.into(),
        id: policy_id.into(),
    }
}

/// One intent per ChangeSet under one control plane: the association key recovery reads by.
pub fn intent_id(control_id: &str, repo_id: &str, change_set_id: &str) -> String {
    format!(
        "rp-{}",
        bytes_sha256(format!("{control_id}:{repo_id}:{change_set_id}").as_bytes())
    )
}

pub fn effect_id(intent_id: &str, round: u64) -> String {
    format!("review-publish:{intent_id}:{round}")
}

fn conflict_scope(repo_id: &str, change_set_id: &str) -> String {
    format!("review-publish:{repo_id}:{change_set_id}")
}

/// Freeze a policy. Same policy under the same id is idempotent; a different policy under
/// an existing id is refused — Execution Specs reference it by digest.
pub fn freeze_policy(
    store: &mut Store,
    actor: &TrustedActor,
    policy_id: &str,
    policy: Policy,
) -> Result<FrozenPolicy> {
    validate_policy(&policy)?;
    let digest = canonical_json_sha256(&serde_json::to_value(&policy)?)?;
    let frozen = FrozenPolicy {
        policy_id: policy_id.into(),
        version: 1,
        digest,
        policy,
    };
    let key = policy_key(&frozen.policy.repo_id, policy_id);
    if let Some(existing) = store.get(&key)? {
        let RecordData::Value { value } = &existing.data else {
            return Err(reject(
                "POLICY_RECORD",
                "policy record malformed",
                "inspect_repo",
            ));
        };
        let existing: FrozenPolicy = serde_json::from_value(value.clone())?;
        if existing.digest == frozen.digest {
            return Ok(existing);
        }
        return Err(reject(
            "POLICY_CONFLICT",
            "a different review publish policy is frozen under this id",
            "use_new_policy_id",
        ));
    }
    let value = serde_json::to_value(&frozen)?;
    let command_key = format!("review.policy:{}:{policy_id}", frozen.policy.repo_id);
    let scoped = scoped(actor, &frozen.policy.repo_id)?;
    let cmd = Command {
        command_id: command_key.clone(),
        idempotency_key: command_key,
        actor: scoped.0.clone(),
        target: key.clone(),
        expected: Expected::Absent,
        binding: crate::binding(&frozen.policy.repo_id),
        input_digest: Command::digest_input("review.policy", &value)?,
        operation: "review.policy".into(),
        input: value.clone(),
    };
    store.submit(store.generation(), &scoped, &cmd, None, |tx| {
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
        Ok(json!({"policy_id": policy_id}))
    })?;
    Ok(frozen)
}

pub fn policy(store: &Store, repo_id: &str, policy_id: &str) -> Result<FrozenPolicy> {
    let record = store.get(&policy_key(repo_id, policy_id))?.ok_or_else(|| {
        reject(
            "POLICY_NOT_FOUND",
            "review publish policy not frozen",
            "freeze_policy",
        )
    })?;
    let RecordData::Value { value } = &record.data else {
        return Err(reject(
            "POLICY_RECORD",
            "policy record malformed",
            "inspect_repo",
        ));
    };
    Ok(serde_json::from_value(value.clone())?)
}

fn validate_policy(policy: &Policy) -> Result<()> {
    for (value, label) in [
        (&policy.repo_id, "repo_id"),
        (&policy.branch_rule, "branch_rule"),
        (&policy.target_branch, "target_branch"),
    ] {
        if value.trim().is_empty() {
            return Err(reject(
                "INVALID_INPUT",
                format!("{label} is required"),
                "correct_input",
            ));
        }
    }
    if policy.branch_rule.starts_with("refs/")
        || policy.branch_rule.contains("..")
        || policy.target_branch.starts_with("refs/")
    {
        return Err(reject(
            "INVALID_INPUT",
            "branches are named without refs/ and without ..",
            "correct_input",
        ));
    }
    if policy.description_source != "none" {
        return Err(reject(
            "INVALID_INPUT",
            "description_source: only `none` is implemented",
            "correct_input",
        ));
    }
    if policy.audit_scope != "minimal" {
        return Err(reject(
            "INVALID_INPUT",
            "audit_scope: only `minimal` is implemented",
            "correct_input",
        ));
    }
    Ok(())
}

/// The branch a ChangeSet publishes to under a policy.
pub fn branch_for(policy: &Policy, change_set_id: &str) -> String {
    policy.branch_rule.replace("{change_set}", change_set_id)
}

/// What admission hands over to be published: the policy and the revision just admitted.
#[derive(Clone, Debug)]
pub struct Publication {
    pub policy: FrozenPolicy,
    pub authorizing_actor: Actor,
}

/// Enqueue the publish of `revision` inside the admission transaction.
///
/// No intent yet: one is written, queued unless the policy wants a human first. A round
/// the worker has not taken yet is superseded: its effect is withdrawn as never sent and a
/// new round starts for this revision (a human's release of the old round does not carry
/// over). A round already executing keeps its target and its evidence; the newer revision
/// waits in `queued` and becomes the next round when this one settles. After a settled
/// round a new one is queued when the policy allows updates, otherwise the intent records
/// the refusal and the revision stays unpublished. Returns the intent as written.
pub fn enqueue(
    tx: &mut CommandTransaction<'_>,
    control_id: &str,
    revision: &ChangeSetRevision,
    repo_id: &str,
    binding_version: u64,
    publication: &Publication,
    now_ms: u64,
) -> Result<Intent> {
    if publication.policy.policy.repo_id != repo_id {
        return Err(reject(
            "POLICY_REPO_MISMATCH",
            "the publish policy is frozen for another Repo",
            "use_policy_of_this_repo",
        ));
    }
    if publication.policy.policy.binding_version != binding_version {
        return Err(reject(
            "POLICY_BINDING_MISMATCH",
            "the publish policy was frozen against another platform binding version than the ChangeSet",
            "authorize_a_new_dispatch_under_the_current_binding",
        ));
    }
    let id = intent_id(control_id, repo_id, &revision.change_set_id);
    let key = intent_key(repo_id, &id);
    let target = Target {
        change_set_revision_id: revision.change_set_revision_id.clone(),
        base_commit_sha: revision.base_commit_sha.clone(),
        result_tree_sha: revision.result_tree_sha.clone(),
        commit_sha: None,
    };
    let existing = match tx.get(&key)? {
        Some(record) => {
            let RecordData::Value { value } = &record.data else {
                return Err(reject(
                    "INTENT_RECORD",
                    "intent record malformed",
                    "inspect_repo",
                ));
            };
            Some(serde_json::from_value::<Intent>(value.clone())?)
        }
        None => None,
    };
    let intent = match existing {
        None => {
            let policy = &publication.policy;
            let state = if policy.policy.requires_human_confirmation {
                State::PendingHuman
            } else {
                State::Pending
            };
            let intent = Intent {
                intent_id: id.clone(),
                repo_id: repo_id.into(),
                change_set_id: revision.change_set_id.clone(),
                policy: policy.clone(),
                branch: branch_for(&policy.policy, &revision.change_set_id),
                authorized_by: revision.producer_ref.clone(),
                authorizing_actor: publication.authorizing_actor.clone(),
                state,
                round: 1,
                binding_version,
                target,
                started: false,
                queued: None,
                push: PushStage::default(),
                review: ReviewStage::default(),
                attempts: 0,
                attention: None,
                failure: None,
                version: 1,
            };
            if state == State::Pending {
                tx.enqueue_effect(&effect(&intent)?)?;
            }
            intent
        }
        Some(mut intent) => {
            if intent.policy.digest != publication.policy.digest {
                return Err(reject(
                    "POLICY_CHANGED",
                    "this ChangeSet already publishes under another frozen policy",
                    "use_original_policy",
                ));
            }
            if intent.target.change_set_revision_id == revision.change_set_revision_id
                || intent
                    .queued
                    .as_ref()
                    .is_some_and(|q| q.change_set_revision_id == revision.change_set_revision_id)
            {
                return Ok(intent);
            }
            match intent.state {
                // Not taken yet: nothing was sent for this round, so it is withdrawn and the
                // newer revision gets its own round — and its own human confirmation.
                State::PendingHuman | State::Pending if !intent.started => {
                    if intent.state == State::Pending {
                        tx.cancel_pending_effect(&effect_id(&intent.intent_id, intent.round))?;
                    }
                    next_round(&mut intent, target);
                    if intent.state == State::Pending {
                        tx.enqueue_effect(&effect(&intent)?)?;
                    }
                }
                // Executing: the round keeps its target and evidence; the newer revision
                // waits its turn.
                State::Pending | State::Unknown | State::PendingHuman => {
                    intent.queued = Some(target);
                }
                State::Published | State::Failed if intent.policy.policy.allow_update => {
                    next_round(&mut intent, target);
                    if intent.state == State::Pending {
                        tx.enqueue_effect(&effect(&intent)?)?;
                    }
                }
                State::Published | State::Failed => {
                    intent.attention = Some(update_not_allowed(
                        &intent,
                        &revision.change_set_revision_id,
                        now_ms,
                    ));
                }
            }
            intent.version += 1;
            intent
        }
    };
    tx.put(&record(&intent)?)?;
    Ok(intent)
}

/// Open the next round for `target`: stages reset (a confirmed push stays as the lease for
/// the next push), the state follows the policy's human gate.
fn next_round(intent: &mut Intent, target: Target) {
    intent.target = target;
    intent.queued = None;
    intent.started = false;
    intent.push = PushStage {
        dispatched: false,
        confirmed_commit: intent.push.confirmed_commit.clone(),
        confirmed_at_unix_ms: None,
    };
    intent.review.dispatched = false;
    intent.review.confirmed_commit = None;
    intent.review.confirmed_at_unix_ms = None;
    intent.attention = None;
    intent.failure = None;
    intent.round += 1;
    intent.state = if intent.policy.policy.requires_human_confirmation {
        State::PendingHuman
    } else {
        State::Pending
    };
}

fn update_not_allowed(intent: &Intent, unpublished: &str, now_ms: u64) -> Attention {
    Attention {
        code: "UPDATE_NOT_ALLOWED".into(),
        message: "the frozen policy allows creating the review request, not updating it; the newer revision is admitted but not published".into(),
        recovery_action: "authorize_a_new_dispatch_with_updates_allowed".into(),
        details: json!({"unpublished_revision": unpublished, "published_revision": intent.target.change_set_revision_id, "at_unix_ms": now_ms}),
    }
}

/// The round this call is about; every persisted step of the worker names it so a round
/// superseded underneath it is refused instead of mixed up.
fn expect_round(intent: &Intent, revision_id: &str, round: u64) -> Result<()> {
    if intent.round != round || intent.target.change_set_revision_id != revision_id {
        return Err(reject(
            "FROZEN_INPUT_CHANGED",
            "the intent moved to another revision or round since this step was planned",
            "read_intent_again",
        ));
    }
    Ok(())
}

fn effect(intent: &Intent) -> Result<EffectIntent> {
    let input = json!({
        "intent_id": intent.intent_id,
        "change_set_id": intent.change_set_id,
        "round": intent.round,
        "policy_digest": intent.policy.digest,
        "branch": intent.branch,
    });
    let operation = "review.publish";
    Ok(EffectIntent {
        intent_id: effect_id(&intent.intent_id, intent.round),
        owner: Reference {
            key: intent_key(&intent.repo_id, &intent.intent_id),
            version: Version::State(1),
        },
        binding: crate::binding(&intent.repo_id),
        target: format!("{}:{}", intent.repo_id, intent.branch),
        conflict_scope: conflict_scope(&intent.repo_id, &intent.change_set_id),
        permission_scope: Scope::Repo(intent.repo_id.clone()),
        input_digest: Command::digest_input(operation, &input)?,
        operation: operation.into(),
        input,
        idempotency_key: effect_id(&intent.intent_id, intent.round),
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
            key: crate::integration::revision_key(
                &intent.repo_id,
                &intent.target.change_set_revision_id,
            ),
            version: Version::State(1),
        }],
        materials: Vec::new(),
    })
}

pub fn get(store: &Store, repo_id: &str, intent_id: &str) -> Result<Intent> {
    let record = store.get(&intent_key(repo_id, intent_id))?.ok_or_else(|| {
        reject(
            "REVIEW_INTENT_NOT_FOUND",
            "review publish intent not found",
            "list_reviews",
        )
    })?;
    let RecordData::Value { value } = &record.data else {
        return Err(reject(
            "INTENT_RECORD",
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

/// Every intent that still owes the platform something, across all Repos.
pub fn open(store: &Store) -> Result<Vec<Intent>> {
    let mut intents = Vec::new();
    for record in store.list(INTENT_KIND)? {
        if let RecordData::Value { value } = &record.data {
            let intent: Intent = serde_json::from_value(value.clone())?;
            if matches!(intent.state, State::Pending | State::Unknown) {
                intents.push(intent);
            }
        }
    }
    Ok(intents)
}

fn scoped(actor: &TrustedActor, repo_id: &str) -> Result<TrustedActor> {
    let mut scoped = actor.0.clone();
    let scope = Scope::Repo(repo_id.into());
    if !scoped.permission_scope.contains(&scope) {
        scoped.permission_scope.push(scope);
    }
    Ok(TrustedActor(scoped))
}

/// The intent's own reducer identity for its bookkeeping commands.
fn reducer(repo_id: &str, intent_id: &str) -> TrustedActor {
    TrustedActor(Actor {
        principal: format!("review-publish:{intent_id}"),
        source: ActorSource::InternalReducer,
        permission_scope: vec![Scope::Control, Scope::Repo(repo_id.into())],
        authority: Some(Reference {
            key: intent_key(repo_id, intent_id),
            version: Version::State(1),
        }),
    })
}

/// A human lets a `pending_human` intent go: the effect is queued now, under that human's
/// actor. Idempotent once queued.
pub fn release(
    store: &mut Store,
    actor: &TrustedActor,
    command_id: &str,
    repo_id: &str,
    intent_id: &str,
    revision_id: &str,
    round: u64,
) -> Result<Intent> {
    let mut intent = get(store, repo_id, intent_id)?;
    // The human releases exactly what the preview showed; a newer revision or round since
    // then wants its own look.
    expect_round(&intent, revision_id, round)?;
    if intent.state != State::PendingHuman {
        return Ok(intent);
    }
    if actor.0.source != ActorSource::DirectClient {
        return Err(reject(
            "PERMISSION_DENIED",
            "a human confirms review publishing through a direct client",
            "request_authorization",
        ));
    }
    intent.state = State::Pending;
    intent.authorizing_actor = actor.0.clone();
    intent.version += 1;
    let input = json!({"intent_id": intent_id, "round": intent.round});
    let operation = "review.release";
    let scoped = scoped(actor, repo_id)?;
    let cmd = Command {
        command_id: command_id.into(),
        idempotency_key: format!("{operation}:{intent_id}:{}", intent.round),
        actor: scoped.0.clone(),
        target: intent_key(repo_id, intent_id),
        expected: Expected::Exact(Version::State(intent.version - 1)),
        binding: crate::binding(repo_id),
        input_digest: Command::digest_input(operation, &input)?,
        operation: operation.into(),
        input,
    };
    let effect = effect(&intent)?;
    store.submit(store.generation(), &scoped, &cmd, None, |tx| {
        tx.enqueue_effect(&effect)?;
        tx.put(&record(&intent)?)?;
        Ok(json!({"state": intent.state}))
    })?;
    get(store, repo_id, intent_id)
}

/// Take the effect for execution. Refuses settled intents and any whose frozen inputs no
/// longer match the queued effect.
pub fn begin(store: &mut Store, repo_id: &str, intent_id: &str) -> Result<(Intent, EffectState)> {
    let intent = get(store, repo_id, intent_id)?;
    if !matches!(intent.state, State::Pending | State::Unknown) {
        return Err(reject(
            "INTENT_TERMINAL",
            "review publish intent is not open",
            "inspect_intent",
        ));
    }
    let id = effect_id(intent_id, intent.round);
    let (effect_record, state) = store.effect(&id)?;
    if effect_record.owner.key != intent_key(repo_id, intent_id) {
        return Err(reject(
            "FROZEN_INPUT_CHANGED",
            "effect differs from the queued intent",
            "inspect_intent",
        ));
    }
    if state == EffectState::Pending {
        store.resume_pending_effect(store.generation(), &id, true)?;
        store.begin_effect(store.generation(), &id)?;
    }
    // From the first take the round's target is immutable; admission queues behind it.
    let intent = if intent.started {
        intent
    } else {
        let mut started = intent;
        started.started = true;
        let input = json!({"round": started.round});
        bump(
            store,
            repo_id,
            intent_id,
            started,
            "review.round_started",
            input,
        )?
    };
    Ok((intent, state))
}

fn bump(
    store: &mut Store,
    repo_id: &str,
    intent_id: &str,
    mut intent: Intent,
    operation: &str,
    input: Value,
) -> Result<Intent> {
    let actor = reducer(repo_id, intent_id);
    intent.version += 1;
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

/// Freeze the commit the push will carry. Idempotent for the same commit; another commit
/// while one is frozen and possibly pushed is refused.
pub fn freeze_commit(
    store: &mut Store,
    repo_id: &str,
    intent_id: &str,
    revision_id: &str,
    round: u64,
    commit_sha: &str,
) -> Result<Intent> {
    let mut intent = get(store, repo_id, intent_id)?;
    expect_round(&intent, revision_id, round)?;
    match intent.target.commit_sha.as_deref() {
        Some(existing) if existing == commit_sha => return Ok(intent),
        Some(_) if intent.push.dispatched => {
            return Err(reject(
                "PUSH_IN_FLIGHT",
                "another commit is frozen and may have been pushed; read it back first",
                "read_back_branch",
            ));
        }
        _ => {}
    }
    intent.target.commit_sha = Some(commit_sha.into());
    intent.attempts += 1;
    let input = json!({"commit_sha": commit_sha, "attempt": intent.attempts});
    bump(
        store,
        repo_id,
        intent_id,
        intent,
        "review.push_planned",
        input,
    )
}

/// Mark whether the push has left (`true` before sending, `false` after a refusal that
/// proves nothing was written).
pub fn mark_push_dispatched(
    store: &mut Store,
    repo_id: &str,
    intent_id: &str,
    revision_id: &str,
    round: u64,
    dispatched: bool,
) -> Result<Intent> {
    let mut intent = get(store, repo_id, intent_id)?;
    expect_round(&intent, revision_id, round)?;
    if intent.push.dispatched == dispatched {
        return Ok(intent);
    }
    intent.push.dispatched = dispatched;
    let input = json!({"dispatched": dispatched});
    bump(
        store,
        repo_id,
        intent_id,
        intent,
        "review.push_dispatched",
        input,
    )
}

/// Stage 1 confirmed: the branch reads back at the frozen commit.
pub fn confirm_push(
    store: &mut Store,
    repo_id: &str,
    intent_id: &str,
    revision_id: &str,
    round: u64,
    commit_sha: &str,
    now_ms: u64,
) -> Result<Intent> {
    let mut intent = get(store, repo_id, intent_id)?;
    expect_round(&intent, revision_id, round)?;
    if intent.target.commit_sha.as_deref() != Some(commit_sha) {
        return Err(reject(
            "FROZEN_INPUT_CHANGED",
            "the pushed commit is not the frozen one",
            "inspect_intent",
        ));
    }
    if intent.push.confirmed_commit.as_deref() == Some(commit_sha) {
        return Ok(intent);
    }
    intent.push.confirmed_commit = Some(commit_sha.into());
    intent.push.confirmed_at_unix_ms = Some(now_ms);
    intent.attention = None;
    let input = json!({"commit_sha": commit_sha});
    bump(
        store,
        repo_id,
        intent_id,
        intent,
        "review.push_confirmed",
        input,
    )
}

pub fn mark_review_dispatched(
    store: &mut Store,
    repo_id: &str,
    intent_id: &str,
    revision_id: &str,
    round: u64,
    dispatched: bool,
) -> Result<Intent> {
    let mut intent = get(store, repo_id, intent_id)?;
    expect_round(&intent, revision_id, round)?;
    if intent.review.dispatched == dispatched {
        return Ok(intent);
    }
    intent.review.dispatched = dispatched;
    let input = json!({"dispatched": dispatched});
    bump(
        store,
        repo_id,
        intent_id,
        intent,
        "review.request_dispatched",
        input,
    )
}

/// What one attempt read back after trying to finish the round.
#[derive(Clone, Debug)]
pub enum Outcome {
    /// Both stages confirmed: the request carries the frozen commit. Writes the mapping.
    Published {
        index: u64,
        platform_commit_sha: String,
        readback: Value,
    },
    /// Nothing more will be written for this round; the human decides what next.
    Failed(Attention),
    /// Waiting for the platform or the human; the same round retries.
    Attention(Attention),
    /// A request may have reached the platform; only readback decides.
    Unknown(Attention),
}

/// Record a round's readback. `Published` writes the ChangeSet–Platform Binding for the
/// published revision and confirms the effect in the same transaction.
pub fn confirm(
    store: &mut Store,
    repo_id: &str,
    intent_id: &str,
    revision_id: &str,
    round: u64,
    outcome: Outcome,
    now_ms: u64,
) -> Result<Intent> {
    let mut intent = get(store, repo_id, intent_id)?;
    expect_round(&intent, revision_id, round)?;
    if !matches!(intent.state, State::Pending | State::Unknown) {
        return Err(reject(
            "INTENT_TERMINAL",
            "review publish intent is not open",
            "inspect_intent",
        ));
    }
    let actor = reducer(repo_id, intent_id);
    let effect_key = effect_id(intent_id, intent.round);
    let (effect_record, _) = store.effect(&effect_key)?;
    let mut binding: Option<(ObjectKey, Value)> = None;
    let (operation, readback) = match outcome {
        Outcome::Published {
            index,
            platform_commit_sha,
            readback,
        } => {
            if intent.target.commit_sha.as_deref() != Some(platform_commit_sha.as_str()) {
                return Err(reject(
                    "FROZEN_INPUT_CHANGED",
                    "the request's commit is not the frozen one",
                    "inspect_intent",
                ));
            }
            intent.state = State::Published;
            intent.review.index = Some(index);
            intent.review.confirmed_commit = Some(platform_commit_sha.clone());
            intent.review.confirmed_at_unix_ms = Some(now_ms);
            intent.review.dispatched = false;
            intent.push.dispatched = false;
            intent.attention = None;
            intent.failure = None;
            // The shape `integration::review_request` reads: the platform commit at the top,
            // the request's number under `review_request`.
            binding = Some((
                platform_binding_key(repo_id, &intent.target.change_set_revision_id),
                json!({
                    "change_set_revision_id": intent.target.change_set_revision_id,
                    "change_set_id": intent.change_set_id,
                    "platform_commit_sha": platform_commit_sha,
                    "review_request": {"index": index, "base_branch": intent.policy.policy.target_branch, "head_branch": intent.branch},
                    "intent_id": intent_id,
                    "round": intent.round,
                    "recorded_at_unix_ms": now_ms,
                }),
            ));
            (
                "review.published",
                EffectReadback::Confirmed {
                    binding: effect_record.binding.clone(),
                    target: effect_record.target.clone(),
                    input_digest: effect_record.input_digest.clone(),
                    result: json!({"index": index, "platform_commit_sha": platform_commit_sha, "readback": readback}),
                },
            )
        }
        Outcome::Failed(reason) => {
            intent.state = State::Failed;
            intent.failure = Some(reason.clone());
            intent.attention = None;
            (
                "review.failed",
                EffectReadback::Rejected {
                    binding: effect_record.binding.clone(),
                    target: effect_record.target.clone(),
                    input_digest: effect_record.input_digest.clone(),
                    result: serde_json::to_value(&reason)?,
                },
            )
        }
        Outcome::Attention(reason) => {
            intent.state = State::Unknown;
            intent.attention = Some(reason);
            ("review.attention", EffectReadback::Unknown)
        }
        Outcome::Unknown(reason) => {
            intent.state = State::Unknown;
            intent.attention = Some(reason);
            ("review.unknown", EffectReadback::Unknown)
        }
    };
    // A settled round hands over to the revision that waited behind it, under the same
    // policy rules as any later admission.
    let mut next_effect = None;
    if matches!(intent.state, State::Published | State::Failed)
        && let Some(queued) = intent.queued.take()
    {
        if intent.policy.policy.allow_update {
            next_round(&mut intent, queued);
            if intent.state == State::Pending {
                next_effect = Some(effect(&intent)?);
            }
        } else {
            intent.attention = Some(update_not_allowed(
                &intent,
                &queued.change_set_revision_id,
                now_ms,
            ));
        }
    }
    intent.version += 1;
    let input = json!({"round": round, "state": intent.state, "attempt": intent.attempts});
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
        tx.confirm_effect(&effect_key, &readback)?;
        if let Some(next) = &next_effect {
            tx.enqueue_effect(next)?;
        }
        if let Some((key, value)) = &binding {
            // The mapping is evidence: written once per revision, never rewritten.
            if tx.get(key)?.is_none() {
                tx.put(&Record {
                    key: key.clone(),
                    version: 1,
                    revision_digest: canonical_json_sha256(value)?,
                    data: RecordData::Value {
                        value: value.clone(),
                    },
                    sources: vec![Reference {
                        key: intent_key(repo_id, intent_id),
                        version: Version::State(1),
                    }],
                    materials: Vec::new(),
                })?;
            }
        }
        tx.put(&record(&intent)?)?;
        Ok(json!({"state": intent.state}))
    })?;
    get(store, repo_id, intent_id)
}

/// The intent with its stages and every mapping this ChangeSet's revisions have.
pub fn show(store: &Store, repo_id: &str, intent_id: &str) -> Result<Value> {
    let intent = get(store, repo_id, intent_id)?;
    let mut mappings = Vec::new();
    for record in store.list(PLATFORM_BINDING_KIND)? {
        if record.key.scope != Scope::Repo(repo_id.into()) {
            continue;
        }
        if let RecordData::Value { value } = &record.data
            && value["change_set_id"] == json!(intent.change_set_id)
        {
            mappings.push(value.clone());
        }
    }
    mappings.sort_by_key(|m| m["round"].as_u64().unwrap_or_default());
    Ok(json!({
        "intent": intent,
        "stages": {
            "push": {"confirmed": intent.push.confirmed_commit.is_some(), "commit_sha": intent.push.confirmed_commit, "dispatched": intent.push.dispatched},
            "review_request": {"confirmed": intent.review.confirmed_commit.is_some(), "index": intent.review.index, "commit_sha": intent.review.confirmed_commit, "dispatched": intent.review.dispatched},
        },
        "mappings": mappings,
    }))
}
