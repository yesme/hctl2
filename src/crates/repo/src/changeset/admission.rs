//! The same admission guards serve a human seal and a dispatch result transaction.
use super::*;
use store::CommandTransaction;

pub(super) fn revision_for(seal: &Seal) -> Result<ChangeSetRevision> {
    let identity = ReviewIdentity {
        change_set_revision_id: String::new(),
        change_set_id: seal.change_set_id.clone(),
        parent_revision_id: seal.parent_revision_id.clone(),
        base_commit_sha: seal.base_commit_sha.clone(),
        result_tree_sha: seal.result_tree_sha.clone(),
    };
    let identity = ReviewIdentity {
        change_set_revision_id: format!("csr-{}", review_subject_digest(&identity)?),
        ..identity
    };
    let mut revision = ChangeSetRevision {
        change_set_revision_id: identity.change_set_revision_id.clone(),
        change_set_id: identity.change_set_id.clone(),
        parent_revision_id: identity.parent_revision_id.clone(),
        base_commit_sha: identity.base_commit_sha.clone(),
        result_tree_sha: identity.result_tree_sha.clone(),
        producer_ref: seal.producer_ref.clone(),
        review_subject_digest: review_subject_digest(&identity)?,
        revision_digest: String::new(),
    };
    revision.revision_digest = canonical_json_sha256(&revision_body(&revision))?;
    Ok(revision)
}

/// Git has already been sealed and read back. This fragment grants no authorization:
/// callers check the semantic owner in this same transaction before calling it.
/// Publication and result admission can then reuse the transaction, without a nested
/// Store submission or a second implementation of the lease and version guards.
pub fn admit_in_transaction(
    tx: &mut CommandTransaction<'_>,
    actor: &TrustedActor,
    repo_id: &str,
    seal: &Seal,
    owner: OwnerGate,
) -> Result<ChangeSetRevision> {
    nonempty(&seal.association_key, "association key")?;
    let current: ChangeSet = decode(
        &tx.get(&change_set_key(repo_id, &seal.change_set_id))?
            .ok_or_else(|| {
                reject(
                    "CHANGESET_NOT_FOUND",
                    "ChangeSet is not open",
                    "open_change_set",
                )
            })?,
    )?;
    if current.version != seal.change_set_version || current.repo_id != repo_id {
        return Err(reject(
            "VERSION_CONFLICT",
            "ChangeSet changed before admission",
            "inspect_change_set",
        ));
    }
    valid_producer(&seal.producer_ref)?;
    match &seal.producer_ref {
        ProducerRef::Invocation { .. } => {
            if owner != OwnerGate::Active {
                return Err(reject(
                    "CHANGESET_OWNER_CLOSED",
                    "owner was cancelled or superseded before admission",
                    "do_not_publish",
                ));
            }
            let lease = seal.lease.as_ref().ok_or_else(|| {
                reject(
                    "LEASE_NOT_CURRENT",
                    "invocation requires its lease",
                    "refresh_lease",
                )
            })?;
            if current.lease.state != LeaseState::Active
                || current.lease.lease_id != lease.lease_id
                || current.lease.generation != lease.generation
            {
                return Err(reject(
                    "LEASE_NOT_CURRENT",
                    "write lease is not the current active lease",
                    "refresh_lease",
                ));
            }
            if current.lease.holder != seal.producer_ref {
                return Err(reject(
                    "LEASE_PRODUCER_MISMATCH",
                    "producer differs from the lease holder invocation or version",
                    "use_authorized_producer",
                ));
            }
        }
        ProducerRef::HumanCommand { .. } => {
            if actor.0.source != ActorSource::DirectClient
                || !actor.0.permission_scope.contains(&Scope::Control)
            {
                return Err(reject(
                    "PERMISSION_DENIED",
                    "human sealing requires a trusted authorized direct client",
                    "request_authorization",
                ));
            }
            if seal.lease.is_some() {
                return Err(reject(
                    "HUMAN_SEAL_LEASE",
                    "human sealing must not borrow an invocation lease",
                    "submit_human_seal_without_lease",
                ));
            }
        }
    }
    git_sha(&seal.base_commit_sha)?;
    git_sha(&seal.result_tree_sha)?;
    if let Some(commit) = &seal.result_commit_sha {
        git_sha(commit)?;
    }
    if let Some(parent) = &seal.parent_revision_id {
        let previous: ChangeSetRevision =
            decode(&tx.get(&revision_key(repo_id, parent))?.ok_or_else(|| {
                reject(
                    "CHANGESET_REVISION_NOT_FOUND",
                    "parent revision is not admitted in this Repo",
                    "use_parent_from_this_change_set",
                )
            })?)?;
        if previous.change_set_id != current.change_set_id {
            return Err(reject(
                "CHANGESET_PARENT_MISMATCH",
                "parent revision belongs to another ChangeSet",
                "use_parent_from_this_change_set",
            ));
        }
    }
    let revision = revision_for(seal)?;
    let key = revision_key(repo_id, &revision.change_set_revision_id);
    if let Some(existing) = tx.get(&key)? {
        return decode(&existing);
    }
    tx.put(&value_record(key, 1, &revision)?)?;
    Ok(revision)
}
