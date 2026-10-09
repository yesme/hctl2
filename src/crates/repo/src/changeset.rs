//! ChangeSet Revision is the admitted Git snapshot Claude's integration consumes.
//! Commit packaging is not part of the revision. Git writes stay outside this transaction.
use foundation::{bytes_sha256, canonical_json_sha256};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use store::{
    ActorSource, Command, Expected, ObjectKey, Record, RecordData, Reference, Scope, Store,
    TrustedActor, Version,
};

use crate::{Result, reject};

mod admission;
mod human;
mod leases;
pub use admission::admit_in_transaction;
pub use human::{HumanInput, HumanPlan, admit_human, human_receipt, prepare_human};
pub use leases::{
    LeasePlan, acquire_lease, complete_revocation, get_change_set, plan_lease, revoke_lease,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseState {
    Pending,
    Active,
    Revoking,
    Revoked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WriteLease {
    pub lease_id: String,
    pub generation: u64,
    pub state: LeaseState,
    pub holder: ProducerRef,
}

/// The admitted write boundary. One ChangeSet has at most one active lease.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeSet {
    pub change_set_id: String,
    pub repo_id: String,
    pub binding_version: u64,
    pub baseline_commit: String,
    pub version: i64,
    pub lease: WriteLease,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProducerRef {
    Invocation {
        invocation_id: String,
        invocation_version: u64,
    },
    HumanCommand {
        command_id: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerGate {
    Active,
    Cancelled,
    Superseded,
}

/// Five fields decide review identity. `producer_ref` is stored and is not one of them.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewIdentity {
    pub change_set_revision_id: String,
    pub change_set_id: String,
    pub parent_revision_id: Option<String>,
    pub base_commit_sha: String,
    pub result_tree_sha: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeSetRevision {
    pub change_set_revision_id: String,
    pub change_set_id: String,
    pub parent_revision_id: Option<String>,
    pub base_commit_sha: String,
    pub result_tree_sha: String,
    pub producer_ref: ProducerRef,
    pub review_subject_digest: String,
    pub revision_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeaseRef {
    pub lease_id: String,
    pub generation: u64,
}

/// Frozen admission input, including the tool readback. Human commands have no lease.
/// Owner state is trusted caller context, not a tool observation or command input.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seal {
    pub association_key: String,
    pub change_set_id: String,
    pub change_set_version: i64,
    pub lease: Option<LeaseRef>,
    pub base_commit_sha: String,
    pub result_tree_sha: String,
    pub result_commit_sha: Option<String>,
    pub parent_revision_id: Option<String>,
    pub producer_ref: ProducerRef,
}

/// One proposed Git result. Its paths are operation inputs, not registered workspaces.
/// Control's field checks authorize nothing until native Git readback and admission.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub change_set_id: String,
    pub lease: LeaseRef,
    pub base_commit_sha: String,
    pub parent_revision_id: Option<String>,
    pub location: OutputLocation,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum OutputLocation {
    Commit {
        repo_path: std::path::PathBuf,
        commit_sha: String,
    },
    Worktree {
        repo_path: std::path::PathBuf,
    },
}

pub const OUTPUT_SCHEMA: &str = "hctl2.changeset-output.v1";

pub fn review_subject_digest(identity: &ReviewIdentity) -> Result<String> {
    Ok(canonical_json_sha256(&serde_json::to_value(identity)?)?)
}

pub fn open_change_set(
    store: &mut Store,
    actor: &TrustedActor,
    repo_id: &str,
    binding_version: u64,
    baseline_commit: &str,
    key: &str,
    holder: &ProducerRef,
) -> Result<ChangeSet> {
    git_sha(baseline_commit)?;
    nonempty(repo_id, "repo id")?;
    nonempty(key, "change set key")?;
    valid_producer(holder)?;
    let change_set_id = format!(
        "cs-{}",
        bytes_sha256(format!("{}:{repo_id}:{key}", store.control_id()).as_bytes())
    );
    let set = ChangeSet {
        change_set_id: change_set_id.clone(),
        repo_id: repo_id.into(),
        binding_version,
        baseline_commit: baseline_commit.into(),
        version: 1,
        lease: WriteLease {
            lease_id: format!("lease-{change_set_id}-1"),
            generation: 1,
            state: LeaseState::Active,
            holder: holder.clone(),
        },
    };
    let record = value_record(change_set_key(repo_id, &change_set_id), 1, &set)?;
    let input = serde_json::to_value(&set)?;
    let command_key = format!("changeset.open:{repo_id}:{key}");
    let command = command(
        actor,
        record.key.clone(),
        &command_key,
        &command_key,
        Expected::Absent,
        "changeset.open",
        input,
    )?;
    let result = store.submit(store.generation(), actor, &command, None, |tx| {
        tx.put(&record)?;
        Ok(serde_json::to_value(&set)?)
    })?;
    Ok(serde_json::from_value(result)?)
}

pub fn admit(
    store: &mut Store,
    actor: &TrustedActor,
    seal: Seal,
    owner: OwnerGate,
) -> Result<ChangeSetRevision> {
    admit_with_publication(store, actor, seal, owner, None, 0).map(|(revision, _)| revision)
}

/// [`admit`], and in the same transaction the publish intent the frozen review publishing
/// policy calls for (`spec/repo.md` §发布评审: persisted with the admission, under the actor
/// envelope of the human command that authorized the write). A failure anywhere leaves
/// neither the revision nor the intent.
pub fn admit_with_publication(
    store: &mut Store,
    actor: &TrustedActor,
    seal: Seal,
    owner: OwnerGate,
    publication: Option<&crate::review::Publication>,
    now_ms: u64,
) -> Result<(ChangeSetRevision, Option<crate::review::Intent>)> {
    nonempty(&seal.association_key, "association key")?;
    let control_id = store.control_id().to_owned();
    // Only the immutable Repo scope is discovered here. Mutable guards and duplicate
    // revision lookup run inside submit, after its persisted-command replay.
    let set = load_change_set(store, "", &seal.change_set_id)?.ok_or_else(|| {
        reject(
            "CHANGESET_NOT_FOUND",
            "ChangeSet is not open",
            "open_change_set",
        )
    })?;
    // The command's input is everything that decides what this admission does: the seal,
    // and — when a publish is requested — which frozen policy under whose authority. A
    // replay of the same key with another policy, another authorizer, or no publish at all
    // is then a different input and refused, not answered from the cache. Without a
    // publication the input is the seal alone, exactly as admissions recorded before
    // publishing existed, so those keep replaying.
    let input = match publication {
        None => serde_json::to_value(&seal)?,
        Some(publication) => json!({
            "seal": seal,
            "publication": {
                "policy_id": publication.policy.policy_id,
                "policy_version": publication.policy.version,
                "policy_digest": publication.policy.digest,
                "authorizing_actor": publication.authorizing_actor,
            },
        }),
    };
    let command_key = format!(
        "changeset.admit:{}:{}",
        set.change_set_id, seal.association_key
    );
    let command_id = match &seal.producer_ref {
        ProducerRef::HumanCommand { command_id } => command_id.as_str(),
        ProducerRef::Invocation { .. } => command_key.as_str(),
    };
    let command = command(
        actor,
        change_set_key(&set.repo_id, &set.change_set_id),
        command_id,
        &command_key,
        Expected::Exact(Version::State(seal.change_set_version)),
        "changeset.admit",
        input,
    )?;
    let result = store.submit(store.generation(), actor, &command, None, |tx| {
        let admitted = admit_in_transaction(tx, actor, &set.repo_id, &seal, owner)?;
        match publication {
            Some(publication) => {
                let intent = crate::review::enqueue(
                    tx,
                    &control_id,
                    &admitted,
                    &set.repo_id,
                    set.binding_version,
                    publication,
                    now_ms,
                )?;
                Ok(json!({"revision": admitted, "intent": intent}))
            }
            // The result shape admissions always had; replays of older ones decode it.
            None => Ok(serde_json::to_value(&admitted)?),
        }
    })?;
    // A result is either the bare revision (no publish) or `{revision, intent}`.
    if result.get("revision").is_some() {
        Ok((
            serde_json::from_value(result["revision"].clone())?,
            serde_json::from_value(result["intent"].clone())?,
        ))
    } else {
        Ok((serde_json::from_value(result)?, None))
    }
}

pub fn get_revision(store: &Store, revision_id: &str) -> Result<ChangeSetRevision> {
    find_revision(store, revision_id)?.ok_or_else(|| {
        reject(
            "CHANGESET_REVISION_NOT_FOUND",
            "ChangeSet Revision is not admitted",
            "admit_revision",
        )
    })
}

pub fn list_revisions(store: &Store, change_set_id: &str) -> Result<Vec<ChangeSetRevision>> {
    let mut rows = revisions(store)?
        .into_iter()
        .filter(|revision| revision.change_set_id == change_set_id)
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        left.change_set_revision_id
            .cmp(&right.change_set_revision_id)
    });
    Ok(rows)
}

fn revision_body(revision: &ChangeSetRevision) -> Value {
    json!({
        "change_set_revision_id": revision.change_set_revision_id,
        "change_set_id": revision.change_set_id,
        "parent_revision_id": revision.parent_revision_id,
        "base_commit_sha": revision.base_commit_sha,
        "result_tree_sha": revision.result_tree_sha,
        "producer_ref": revision.producer_ref,
        "review_subject_digest": revision.review_subject_digest,
    })
}

fn find_revision(store: &Store, revision_id: &str) -> Result<Option<ChangeSetRevision>> {
    Ok(revisions(store)?
        .into_iter()
        .find(|revision| revision.change_set_revision_id == revision_id))
}

fn revisions(store: &Store) -> Result<Vec<ChangeSetRevision>> {
    store
        .list("changeset_revision")?
        .into_iter()
        .map(|record| decode(&record))
        .collect()
}

fn load_change_set(store: &Store, repo_id: &str, change_set_id: &str) -> Result<Option<ChangeSet>> {
    if !repo_id.is_empty() {
        return store
            .get(&change_set_key(repo_id, change_set_id))?
            .map(|record| decode(&record))
            .transpose();
    }
    Ok(store.list("changeset")?.into_iter().find_map(|record| {
        let set: ChangeSet = decode(&record).ok()?;
        (set.change_set_id == change_set_id).then_some(set)
    }))
}

fn change_set_key(repo_id: &str, change_set_id: &str) -> ObjectKey {
    ObjectKey {
        scope: Scope::Repo(repo_id.into()),
        kind: "changeset".into(),
        id: change_set_id.into(),
    }
}

fn revision_key(repo_id: &str, revision_id: &str) -> ObjectKey {
    ObjectKey {
        scope: Scope::Repo(repo_id.into()),
        kind: "changeset_revision".into(),
        id: revision_id.into(),
    }
}

fn value_record<T: Serialize>(key: ObjectKey, version: i64, value: &T) -> Result<Record> {
    let value = serde_json::to_value(value)?;
    Ok(Record {
        key,
        version,
        revision_digest: canonical_json_sha256(&value)?,
        data: RecordData::Value { value },
        sources: Vec::new(),
        materials: Vec::new(),
    })
}

fn decode<T: for<'de> Deserialize<'de>>(record: &Record) -> Result<T> {
    match &record.data {
        RecordData::Value { value } => Ok(serde_json::from_value(value.clone())?),
        _ => Err(reject(
            "CHANGESET_RECORD",
            "stored record is not a ChangeSet payload",
            "inspect_store",
        )),
    }
}

fn command(
    actor: &TrustedActor,
    target: ObjectKey,
    command_id: &str,
    idem: &str,
    expected: Expected,
    operation: &str,
    input: Value,
) -> Result<Command> {
    Ok(Command {
        command_id: command_id.into(),
        idempotency_key: idem.into(),
        actor: actor.0.clone(),
        binding: Reference {
            key: target.clone(),
            version: Version::State(1),
        },
        target,
        expected,
        input_digest: Command::digest_input(operation, &input)?,
        operation: operation.into(),
        input,
    })
}

fn git_sha(value: &str) -> Result<()> {
    if value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(reject(
            "GIT_SHA_INVALID",
            "Git object id must be 40 lowercase hex characters",
            "pass_readback_sha",
        ))
    }
}

fn valid_producer(producer: &ProducerRef) -> Result<()> {
    match producer {
        ProducerRef::Invocation {
            invocation_id,
            invocation_version,
        } => {
            nonempty(invocation_id, "invocation id")?;
            if *invocation_version == 0 {
                return Err(reject(
                    "INVALID_INPUT",
                    "invocation version starts at one",
                    "use_authorized_producer",
                ));
            }
        }
        ProducerRef::HumanCommand { command_id } => nonempty(command_id, "human command id")?,
    }
    Ok(())
}

fn nonempty(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() {
        Err(reject(
            "INVALID_INPUT",
            format!("{label} is required"),
            "correct_input",
        ))
    } else {
        Ok(())
    }
}
