//! ChangeSet Revision is the admitted Git snapshot Claude's integration consumes.
//! Commit packaging is not part of the revision. Git writes stay outside this transaction.
use foundation::{bytes_sha256, canonical_json_sha256};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use store::{
    Command, Expected, ObjectKey, Record, RecordData, Reference, Scope, Store, TrustedActor,
    Version,
};

use crate::{Result, reject};

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
    pub holder: String,
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

/// Tool readback handed to admission. `result_commit_sha` is packaging only.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seal {
    pub association_key: String,
    pub change_set_id: String,
    pub lease_id: String,
    pub lease_generation: u64,
    pub base_commit_sha: String,
    pub result_tree_sha: String,
    pub result_commit_sha: Option<String>,
    pub parent_revision_id: Option<String>,
    pub producer_ref: ProducerRef,
    pub owner: OwnerGate,
}

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
    holder: &str,
) -> Result<ChangeSet> {
    git_sha(baseline_commit)?;
    nonempty(repo_id, "repo id")?;
    nonempty(key, "change set key")?;
    nonempty(holder, "lease holder")?;
    let change_set_id = format!(
        "cs-{}",
        bytes_sha256(format!("{}:{repo_id}:{key}", store.control_id()).as_bytes())
    );
    if let Some(existing) = load_change_set(store, repo_id, &change_set_id)? {
        if existing.repo_id != repo_id
            || existing.baseline_commit != baseline_commit
            || existing.binding_version != binding_version
            || existing.lease.holder != holder
        {
            return Err(reject(
                "IDEMPOTENCY_CONFLICT",
                "change set key already opened with different input",
                "use_original_change_set",
            ));
        }
        return Ok(existing);
    }
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
            holder: holder.into(),
        },
    };
    let record = value_record(change_set_key(repo_id, &change_set_id), 1, &set)?;
    let input = serde_json::to_value(&set)?;
    let command = command(
        actor,
        record.key.clone(),
        &format!("open-{key}"),
        key,
        Expected::Absent,
        "changeset.open",
        input,
    )?;
    store.submit(store.generation(), actor, &command, None, |tx| {
        tx.put(&record)?;
        Ok(serde_json::to_value(&set)?)
    })?;
    Ok(set)
}

pub fn admit(store: &mut Store, actor: &TrustedActor, seal: Seal) -> Result<ChangeSetRevision> {
    let set = load_change_set(store, "", &seal.change_set_id)?.ok_or_else(|| {
        reject(
            "CHANGESET_NOT_FOUND",
            "ChangeSet is not open",
            "open_change_set",
        )
    })?;
    // `load` above scanned by id. Re-read with the repo from that record.
    let set = load_change_set(store, &set.repo_id, &seal.change_set_id)?.expect("just loaded");
    if seal.owner != OwnerGate::Active {
        return Err(reject(
            "CHANGESET_OWNER_CLOSED",
            "owner was cancelled or superseded before admission",
            "do_not_publish",
        ));
    }
    if set.lease.state != LeaseState::Active
        || set.lease.lease_id != seal.lease_id
        || set.lease.generation != seal.lease_generation
    {
        return Err(reject(
            "LEASE_NOT_CURRENT",
            "write lease is not the current active lease",
            "refresh_lease",
        ));
    }
    git_sha(&seal.base_commit_sha)?;
    git_sha(&seal.result_tree_sha)?;
    if let Some(commit) = &seal.result_commit_sha {
        git_sha(commit)?;
    }
    if let Some(parent) = &seal.parent_revision_id {
        let previous = get_revision(store, parent)?;
        if previous.change_set_id != set.change_set_id {
            return Err(reject(
                "CHANGESET_PARENT_MISMATCH",
                "parent revision belongs to another ChangeSet",
                "use_parent_from_this_change_set",
            ));
        }
    }
    let identity = ReviewIdentity {
        change_set_revision_id: String::new(),
        change_set_id: set.change_set_id.clone(),
        parent_revision_id: seal.parent_revision_id.clone(),
        base_commit_sha: seal.base_commit_sha.clone(),
        result_tree_sha: seal.result_tree_sha.clone(),
    };
    let bare = review_subject_digest(&ReviewIdentity {
        change_set_revision_id: String::new(),
        ..identity.clone()
    })?;
    let change_set_revision_id = format!("csr-{bare}");
    let identity = ReviewIdentity {
        change_set_revision_id: change_set_revision_id.clone(),
        ..identity
    };
    let review_subject_digest = review_subject_digest(&identity)?;
    if let Some(existing) = find_revision(store, &change_set_revision_id)? {
        return Ok(existing);
    }
    let mut revision = ChangeSetRevision {
        change_set_revision_id,
        change_set_id: set.change_set_id.clone(),
        parent_revision_id: seal.parent_revision_id.clone(),
        base_commit_sha: seal.base_commit_sha.clone(),
        result_tree_sha: seal.result_tree_sha.clone(),
        producer_ref: seal.producer_ref.clone(),
        review_subject_digest,
        revision_digest: String::new(),
    };
    revision.revision_digest =
        canonical_json_sha256(&serde_json::to_value(revision_body(&revision))?)?;
    let record = value_record(
        revision_key(&set.repo_id, &revision.change_set_revision_id),
        1,
        &revision,
    )?;
    let input = serde_json::to_value(&seal)?;
    let command = command(
        actor,
        change_set_key(&set.repo_id, &set.change_set_id),
        &format!("admit-{}", seal.association_key),
        &seal.association_key,
        Expected::Exact(Version::State(set.version)),
        "changeset.admit",
        input,
    )?;
    store.submit(store.generation(), actor, &command, None, |tx| {
        tx.put(&record)?;
        Ok(serde_json::to_value(&revision)?)
    })?;
    Ok(revision)
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
    if value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(reject(
            "GIT_SHA_INVALID",
            "Git object id must be 40 hex characters",
            "pass_readback_sha",
        ))
    }
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
