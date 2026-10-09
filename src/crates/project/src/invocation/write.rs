//! Write boundaries are derived from the human request and frozen Repo/Project records.
//! Policy persistence and Git/provider I/O remain outside the domain preview.
use super::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WriteInput {
    pub change_set_id: Option<String>,
    pub baseline_commit: String,
    pub target_branch: String,
    pub allow_update: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WritePreview {
    pub lease: repo::changeset::LeasePlan,
    pub review_publish_policy: FrozenRef,
    pub policy_record: Reference,
    /// Explicitly a publish-for-review authorization, never target integration.
    pub authorization: String,
}

impl WritePreview {
    /// Required Context bytes let the executor see the exact boundary, not just a
    /// lease id whose ChangeSet it would otherwise have to guess.
    pub fn context_bytes(&self) -> store::Result<Vec<u8>> {
        foundation::canonical_json(&serde_json::to_value(self)?).map_err(Into::into)
    }
}

/// Canonical input for `repo::review::freeze_policy`; the Repo owns that record's type.
/// The control adapter persists it before asking the pure domain for its preview.
pub fn review_policy_input(
    store: &Store,
    input: &Input,
) -> store::Result<Option<(String, repo::review::Policy)>> {
    let p = project(store, &input.project_id)?;
    let RecordData::Project {
        repo_id, settings, ..
    } = &p.data
    else {
        return Err(invalid("Project required"));
    };
    let repo = required(store, &repo::key(repo_id))?;
    let (_, configuration) = participant::profiles::profile_at(store, &input.profile)?;
    policy_input(
        input,
        &p,
        &repo,
        &configuration,
        invocation_id(store, &input.project_id, &input.key),
        settings.publish_review_requires_confirmation,
    )
}

fn policy_input(
    input: &Input,
    p: &Record,
    repo: &Record,
    profile: &WorkerProfile,
    id: String,
    confirmation: bool,
) -> store::Result<Option<(String, repo::review::Policy)>> {
    if profile.mode == "read_only" {
        if input.write.is_some() {
            return Err(reject(
                "WRITE_BOUNDARY_NOT_ALLOWED",
                "read-only Profile cannot authorize a write",
                "select_write_profile",
            ));
        }
        return Ok(None);
    }
    let write = input.write.as_ref().ok_or_else(|| {
        reject(
            "WRITE_BOUNDARY_REQUIRED",
            "write Profile needs a baseline and publication target",
            "specify_write_boundary",
        )
    })?;
    let RecordData::Project { repo_id, .. } = &p.data else {
        return Err(invalid("Project required"));
    };
    let RecordData::Repo {
        platform_binding: Some(binding),
        ..
    } = &repo.data
    else {
        return Err(reject(
            "PUBLICATION_UNAVAILABLE",
            "write preview requires a platform publication target in this slice",
            "bind_repo_platform",
        ));
    };
    let Version::State(version) = binding.version else {
        return Err(invalid("platform binding version required"));
    };
    let binding_version =
        u64::try_from(version).map_err(|_| invalid("positive binding version required"))?;
    if binding_version == 0
        || write.target_branch.trim().is_empty()
        || write.target_branch.trim() != write.target_branch
    {
        return Err(invalid(
            "positive binding version and exact target branch required",
        ));
    }
    let policy = repo::review::Policy {
        repo_id: repo_id.clone(),
        binding_version,
        branch_rule: "hctl2/{change_set}".into(),
        target_branch: write.target_branch.clone(),
        allow_update: write.allow_update,
        description_source: "none".into(),
        requires_human_confirmation: confirmation,
        audit_scope: "minimal".into(),
    };
    let digest = canonical_json_sha256(&serde_json::to_value(&policy)?)?;
    Ok(Some((format!("dispatch-review:{id}:{digest}"), policy)))
}

pub(super) fn prepare(
    store: &Store,
    input: &Input,
    consumer: &Owner,
    p: &Record,
    repo: &Record,
    profile: &WorkerProfile,
) -> store::Result<Option<WritePreview>> {
    let RecordData::Project {
        repo_id, settings, ..
    } = &p.data
    else {
        return Err(invalid("Project required"));
    };
    let Some((policy_id, policy)) = policy_input(
        input,
        p,
        repo,
        profile,
        consumer.id.clone(),
        settings.publish_review_requires_confirmation,
    )?
    else {
        return Ok(None);
    };
    let write = input
        .write
        .as_ref()
        .ok_or_else(|| invalid("write input required"))?;
    let policy_record = required(
        store,
        &key(
            Scope::Repo(repo_id.clone()),
            "review_publish_policy",
            &policy_id,
        ),
    )?;
    let frozen_policy = repo::review::policy(store, repo_id, &policy_id)?;
    let digest = canonical_json_sha256(&serde_json::to_value(&policy)?)?;
    if frozen_policy.policy_id != policy_id
        || frozen_policy.version != policy_record.version
        || frozen_policy.digest != digest
        || frozen_policy.policy != policy
    {
        return Err(reject(
            "POLICY_MISMATCH",
            "frozen publication policy differs from preview",
            "rebuild_preview",
        ));
    }
    let lease = repo::changeset::plan_lease(
        store,
        repo_id,
        policy.binding_version,
        &write.baseline_commit,
        &input.key,
        write.change_set_id.as_deref(),
        &repo::changeset::ProducerRef::Invocation {
            invocation_id: consumer.id.clone(),
            invocation_version: consumer.generation,
        },
    )?;
    Ok(Some(WritePreview {
        lease,
        review_publish_policy: FrozenRef {
            id: policy_id,
            revision: policy_record.version.to_string(),
            digest,
        },
        policy_record: reference(&policy_record),
        authorization: "publish_for_review_not_integration".into(),
    }))
}

pub(super) fn lease_reference(preview: &WritePreview) -> store::Result<FrozenRef> {
    let mut active = preview.lease.pending.lease.clone();
    active.state = repo::changeset::LeaseState::Active;
    Ok(FrozenRef {
        id: active.lease_id.clone(),
        revision: active.generation.to_string(),
        digest: canonical_json_sha256(&serde_json::to_value(active)?)?,
    })
}

pub(super) fn holder(invocation: &Invocation) -> repo::changeset::ProducerRef {
    repo::changeset::ProducerRef::Invocation {
        invocation_id: invocation.spec.document.owner.id.clone(),
        invocation_version: invocation.spec.document.owner.generation,
    }
}

/// Continue only the original human's frozen publication authorization. The
/// result producer supplies neither a replacement policy nor a releasing actor.
pub(super) fn publication(
    store: &Store,
    invocation: &Invocation,
) -> store::Result<Option<repo::review::Publication>> {
    let Some(write) = &invocation.preview.write else {
        return Ok(None);
    };
    let expected = &write.review_publish_policy;
    let policy = repo::review::policy(store, &write.lease.pending.repo_id, &expected.id)?;
    let record = required(store, &write.policy_record.key)?;
    if reference(&record) != write.policy_record
        || policy.version.to_string() != expected.revision
        || policy.digest != expected.digest
        || policy.policy.repo_id != write.lease.pending.repo_id
        || policy.policy.binding_version != write.lease.pending.binding_version
        || invocation.spec.document.review_publish_policy.as_ref() != Some(expected)
    {
        return Err(reject(
            "POLICY_MISMATCH",
            "publication must use the exact Spec policy and platform binding",
            "inspect_original_invocation",
        ));
    }
    let actor = invocation
        .authorization
        .authorizing_actor
        .clone()
        .ok_or_else(|| invalid("original write authorizing actor missing"))?;
    if actor.source != store::ActorSource::DirectClient {
        return Err(reject(
            "PERMISSION_DENIED",
            "publication requires the original direct human authorization",
            "inspect_original_invocation",
        ));
    }
    Ok(Some(repo::review::Publication {
        policy,
        authorizing_actor: actor,
    }))
}

pub(super) fn revoke(
    tx: &mut store::CommandTransaction<'_>,
    invocation: &Invocation,
    never_dispatched: bool,
) -> store::Result<()> {
    if let Some(write) = &invocation.preview.write {
        let set = &write.lease.pending;
        let lease = repo::changeset::LeaseRef {
            lease_id: set.lease.lease_id.clone(),
            generation: set.lease.generation,
        };
        repo::changeset::revoke_lease(
            tx,
            &set.repo_id,
            &set.change_set_id,
            &lease,
            &holder(invocation),
        )?;
        if never_dispatched {
            repo::changeset::complete_revocation(
                tx,
                &set.repo_id,
                &set.change_set_id,
                &lease,
                &holder(invocation),
                None,
            )?;
        }
    }
    Ok(())
}

/// Called in the same transaction that stores the verified original-dispatch stop report.
pub fn confirm_write_stop(
    tx: &mut store::CommandTransaction<'_>,
    invocation: &Invocation,
    proof: &Reference,
) -> store::Result<()> {
    if let Some(write) = &invocation.preview.write {
        let set = &write.lease.pending;
        repo::changeset::complete_revocation(
            tx,
            &set.repo_id,
            &set.change_set_id,
            &repo::changeset::LeaseRef {
                lease_id: set.lease.lease_id.clone(),
                generation: set.lease.generation,
            },
            &holder(invocation),
            Some(proof),
        )?;
    }
    Ok(())
}
