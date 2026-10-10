//! Lease changes join the owning Invocation transaction; this module does no provider I/O.
use super::*;
use store::CommandTransaction;

/// A preview is not a lease grant. `pending` becomes active only in `acquire_lease`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeasePlan {
    pub pending: ChangeSet,
    pub previous: Option<Reference>,
}

pub fn get_change_set(store: &Store, repo_id: &str, id: &str) -> Result<ChangeSet> {
    load_change_set(store, repo_id, id)?.ok_or_else(|| {
        reject(
            "CHANGESET_NOT_FOUND",
            "ChangeSet is not open",
            "inspect_change_set",
        )
    })
}

pub fn plan_lease(
    store: &Store,
    repo_id: &str,
    binding_version: u64,
    baseline_commit: &str,
    command_key: &str,
    existing: Option<&str>,
    holder: &ProducerRef,
) -> Result<LeasePlan> {
    git_sha(baseline_commit)?;
    nonempty(repo_id, "repo id")?;
    nonempty(command_key, "command key")?;
    valid_producer(holder)?;
    let id = existing.map_or_else(
        || {
            format!(
                "cs-{}",
                bytes_sha256(format!("{}:{repo_id}:{command_key}", store.control_id()).as_bytes())
            )
        },
        str::to_owned,
    );
    let previous = store.get(&change_set_key(repo_id, &id))?;
    if existing.is_some() && previous.is_none() {
        return Err(reject(
            "CHANGESET_NOT_FOUND",
            "ChangeSet is not in this Repo",
            "inspect_change_set",
        ));
    }
    let pending = next_lease(
        previous.as_ref(),
        &id,
        repo_id,
        binding_version,
        baseline_commit,
        holder,
    )?;
    Ok(LeasePlan {
        pending,
        previous: previous.as_ref().map(|r| Reference {
            key: r.key.clone(),
            version: Version::State(r.version),
        }),
    })
}

fn next_lease(
    previous: Option<&Record>,
    id: &str,
    repo_id: &str,
    binding_version: u64,
    baseline: &str,
    holder: &ProducerRef,
) -> Result<ChangeSet> {
    let (version, generation) = if let Some(record) = previous {
        let current: ChangeSet = decode(record)?;
        if current.lease.state != LeaseState::Revoked {
            return Err(reject(
                "WRITE_LEASE_BUSY",
                "old writer has not been proved stopped",
                "inspect_change_set",
            ));
        }
        if current.repo_id != repo_id
            || current.binding_version != binding_version
            || current.baseline_commit != baseline
        {
            return Err(reject(
                "CHANGESET_BOUNDARY_MISMATCH",
                "Repo, binding or baseline differs",
                "open_new_change_set",
            ));
        }
        (
            record.version.checked_add(1).ok_or_else(|| {
                reject(
                    "VERSION_OVERFLOW",
                    "ChangeSet version overflow",
                    "inspect_change_set",
                )
            })?,
            current.lease.generation.checked_add(1).ok_or_else(|| {
                reject(
                    "VERSION_OVERFLOW",
                    "lease generation overflow",
                    "inspect_change_set",
                )
            })?,
        )
    } else {
        (1, 1)
    };
    Ok(ChangeSet {
        change_set_id: id.into(),
        repo_id: repo_id.into(),
        binding_version,
        baseline_commit: baseline.into(),
        version,
        lease: WriteLease {
            lease_id: format!("lease-{id}-{generation}"),
            generation,
            state: LeaseState::Pending,
            holder: holder.clone(),
        },
    })
}

/// Compare again inside the authorization transaction, not just at preview time.
pub fn acquire_lease(tx: &mut CommandTransaction<'_>, plan: &LeasePlan) -> Result<ChangeSet> {
    let pending = &plan.pending;
    git_sha(&pending.baseline_commit)?;
    valid_producer(&pending.lease.holder)?;
    let key = change_set_key(&pending.repo_id, &pending.change_set_id);
    let current = tx.get(&key)?;
    let actual = current.as_ref().map(|r| Reference {
        key: r.key.clone(),
        version: Version::State(r.version),
    });
    if actual != plan.previous {
        return Err(reject(
            "VERSION_CONFLICT",
            "ChangeSet changed since preview",
            "preview_again",
        ));
    }
    let expected = next_lease(
        current.as_ref(),
        &pending.change_set_id,
        &pending.repo_id,
        pending.binding_version,
        &pending.baseline_commit,
        &pending.lease.holder,
    )?;
    if expected != *pending {
        return Err(reject(
            "LEASE_PREVIEW_MISMATCH",
            "pending lease differs from derived grant",
            "preview_again",
        ));
    }
    let mut active = pending.clone();
    active.lease.state = LeaseState::Active;
    tx.put(&value_record(key, active.version, &active)?)?;
    Ok(active)
}

/// Revoking does not prove loss of write authority; it still prevents a new grant.
pub fn revoke_lease(
    tx: &mut CommandTransaction<'_>,
    repo_id: &str,
    change_set_id: &str,
    lease: &LeaseRef,
    holder: &ProducerRef,
) -> Result<ChangeSet> {
    let key = change_set_key(repo_id, change_set_id);
    let record = tx.get(&key)?.ok_or_else(|| {
        reject(
            "CHANGESET_NOT_FOUND",
            "ChangeSet is not open",
            "inspect_change_set",
        )
    })?;
    let mut set: ChangeSet = decode(&record)?;
    if set.lease.lease_id != lease.lease_id
        || set.lease.generation != lease.generation
        || set.lease.holder != *holder
    {
        return Err(reject(
            "LEASE_NOT_CURRENT",
            "lease or producer differs",
            "inspect_change_set",
        ));
    }
    if set.lease.state == LeaseState::Active {
        set.version = record.version.checked_add(1).ok_or_else(|| {
            reject(
                "VERSION_OVERFLOW",
                "ChangeSet version overflow",
                "inspect_change_set",
            )
        })?;
        set.lease.state = LeaseState::Revoking;
        tx.put(&value_record(key, set.version, &set)?)?;
    }
    Ok(set)
}

/// Complete revocation only with a stored report of physical exit, or an unsent prepare
/// cancelled in this transaction. A turn result, a stop request or loss is not such proof.
pub fn complete_revocation(
    tx: &mut CommandTransaction<'_>,
    repo_id: &str,
    change_set_id: &str,
    lease: &LeaseRef,
    holder: &ProducerRef,
    proof: Option<&Reference>,
) -> Result<ChangeSet> {
    let key = change_set_key(repo_id, change_set_id);
    let record = tx.get(&key)?.ok_or_else(|| {
        reject(
            "CHANGESET_NOT_FOUND",
            "ChangeSet is not open",
            "inspect_change_set",
        )
    })?;
    let mut set: ChangeSet = decode(&record)?;
    if set.lease.lease_id != lease.lease_id
        || set.lease.generation != lease.generation
        || set.lease.holder != *holder
    {
        return Err(reject(
            "LEASE_NOT_CURRENT",
            "lease or producer differs",
            "inspect_change_set",
        ));
    }
    if set.lease.state == LeaseState::Revoked {
        return Ok(set);
    }
    if set.lease.state != LeaseState::Revoking {
        return Err(reject(
            "LEASE_NOT_REVOKING",
            "revocation was not requested",
            "cancel_original_invocation",
        ));
    }
    let ProducerRef::Invocation {
        invocation_id,
        invocation_version,
    } = holder
    else {
        return Err(reject(
            "STOP_PROOF_REQUIRED",
            "invocation stop proof required",
            "inspect_change_set",
        ));
    };
    let stopped = if let Some(proof) = proof {
        if proof.key.kind != "dispatch_cleanup"
            || proof.key.id != *invocation_id
            || !matches!(&proof.key.scope, Scope::Project(_))
        {
            false
        } else {
            let report = tx.get(&proof.key)?.ok_or_else(|| {
                reject(
                    "STOP_PROOF_REQUIRED",
                    "stop report is not admitted",
                    "inspect_original_dispatch",
                )
            })?;
            let body: Value = decode(&report)?;
            let root_key = ObjectKey {
                scope: proof.key.scope.clone(),
                kind: "room_invocation".into(),
                id: invocation_id.clone(),
            };
            let root = tx.get(&root_key)?.ok_or_else(|| {
                reject(
                    "STOP_PROOF_REQUIRED",
                    "original invocation is missing",
                    "inspect_original_dispatch",
                )
            })?;
            let root: Value = decode(&root)?;
            let mut active_lease = set.lease.clone();
            active_lease.state = LeaseState::Active;
            let spec_lease = &root["spec"]["document"]["write_lease"];
            let generation = lease.generation.to_string();
            report.version
                == match proof.version {
                    Version::State(v) => v,
                    _ => 0,
                }
                && body["stop_report"]["dispatch"]["owner"]["id"] == *invocation_id
                && body["stop_report"]["dispatch"]["owner"]["generation"] == *invocation_version
                && body["stop_report"]["dispatch"]["spec_digest"] == root["spec"]["digest"]
                && spec_lease["id"] == lease.lease_id
                && spec_lease["revision"].as_str() == Some(generation.as_str())
                && spec_lease["digest"]
                    == canonical_json_sha256(&serde_json::to_value(active_lease)?)?
                && body["stop_report"]["events"]
                    .as_array()
                    .is_some_and(|events| {
                        events.iter().any(|event| {
                            event["kind"] == "stopped"
                                && event["source"] == "adapter_event"
                                && ((body["stop_report"]["dispatch"]["state"] == "cancelled"
                                    && event["payload"]["never_started"] == true)
                                    || (event["payload"]
                                        .as_object()
                                        .is_some_and(|payload| payload.contains_key("exit_code"))
                                        && event["payload"]["terminal_result_present"]
                                            .is_boolean()))
                        })
                    })
        }
    } else {
        let (prepare, state) = tx.effect(&format!("prepare:{invocation_id}"))?;
        prepare.owner.key.id == *invocation_id
            && prepare.owner.key.kind == "room_invocation"
            && prepare.operation == "agency.prepare"
            && prepare.owner.version
                == Version::State(i64::try_from(*invocation_version).unwrap_or(0))
            && state == store::EffectState::Cancelled
    };
    if !stopped {
        return Err(reject(
            "STOP_PROOF_REQUIRED",
            "old writer has not been proved stopped",
            "inspect_original_dispatch",
        ));
    }
    set.version = record.version.checked_add(1).ok_or_else(|| {
        reject(
            "VERSION_OVERFLOW",
            "ChangeSet version overflow",
            "inspect_change_set",
        )
    })?;
    set.lease.state = LeaseState::Revoked;
    tx.put(&value_record(key, set.version, &set)?)?;
    Ok(set)
}
