//! An explicit human command, never an Invocation borrowing another producer's lease.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanInput {
    pub key: String,
    pub repo_id: String,
    pub change_set_id: Option<String>,
    pub base_commit_sha: String,
    pub parent_revision_id: Option<String>,
    pub location: OutputLocation,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanPlan {
    pub input: HumanInput,
    pub repo: Reference,
    pub change_set: ChangeSet,
    pub previous: Option<Reference>,
}

fn owner(actor: &TrustedActor, repo: &str) -> Result<TrustedActor> {
    if actor.0.source != ActorSource::DirectClient
        || !actor.0.permission_scope.contains(&Scope::Control)
    {
        return Err(reject(
            "PERMISSION_DENIED",
            "human sealing requires the control owner",
            "request_authorization",
        ));
    }
    let mut scoped = actor.0.clone();
    scoped.permission_scope.push(Scope::Repo(repo.into()));
    Ok(TrustedActor(scoped))
}

fn receipt_key(repo: &str, command_key: &str) -> ObjectKey {
    ObjectKey {
        scope: Scope::Repo(repo.into()),
        kind: "human_seal".into(),
        id: format!("changeset:{command_key}"),
    }
}

/// A retry uses the original observation without requiring a surviving input directory.
pub fn human_receipt(
    store: &Store,
    actor: &TrustedActor,
    input: &HumanInput,
) -> Result<Option<Value>> {
    let _ = owner(actor, &input.repo_id)?;
    let Some(record) = store.get(&receipt_key(&input.repo_id, &input.key))? else {
        return Ok(None);
    };
    let value: Value = decode(&record)?;
    let old: HumanPlan = serde_json::from_value(value["plan"].clone())?;
    if old.input != *input {
        return Err(reject(
            "IDEMPOTENCY_CONFLICT",
            "human seal key has another input",
            "use_new_command_key",
        ));
    }
    Ok(Some(value))
}

pub fn prepare_human(store: &Store, actor: &TrustedActor, input: HumanInput) -> Result<HumanPlan> {
    let _ = owner(actor, &input.repo_id)?;
    if matches!(input.location, OutputLocation::NoChanges { .. }) {
        return Err(reject(
            "INVALID_INPUT",
            "no_changes is an execution result, not a human code-version seal",
            "use_commit_or_worktree",
        ));
    }
    if input.key.trim().is_empty() || input.key.trim() != input.key {
        return Err(reject(
            "INVALID_INPUT",
            "exact nonempty command key required",
            "correct_input",
        ));
    }
    git_sha(&input.base_commit_sha)?;
    if let Some(receipt) = human_receipt(store, actor, &input)? {
        return Ok(serde_json::from_value(receipt["plan"].clone())?);
    }
    let _ = crate::require_active(store, &input.repo_id)?;
    let repo = store
        .get(&crate::key(&input.repo_id))?
        .ok_or_else(|| reject("REPO_NOT_FOUND", "Repo is not registered", "register_repo"))?;
    let RecordData::Repo {
        platform_binding, ..
    } = &repo.data
    else {
        return Err(reject("INVALID_INPUT", "Repo required", "correct_input"));
    };
    let binding_version = match platform_binding.as_ref().map(|r| &r.version) {
        Some(Version::State(version)) => u64::try_from(*version)
            .map_err(|_| reject("INVALID_INPUT", "binding version", "inspect_repo"))?,
        None => 0,
        _ => {
            return Err(reject(
                "INVALID_INPUT",
                "binding state version required",
                "inspect_repo",
            ));
        }
    };
    let id = input.change_set_id.clone().unwrap_or_else(|| {
        format!(
            "cs-{}",
            bytes_sha256(
                format!("{}:{}:{}", store.control_id(), input.repo_id, input.key).as_bytes()
            )
        )
    });
    let previous = store.get(&change_set_key(&input.repo_id, &id))?;
    if input.change_set_id.is_some() && previous.is_none() {
        return Err(reject(
            "CHANGESET_NOT_FOUND",
            "ChangeSet is not in this Repo",
            "inspect_change_set",
        ));
    }
    let set = match previous.as_ref() {
        Some(record) => decode(record)?,
        None => ChangeSet {
            change_set_id: id.clone(),
            repo_id: input.repo_id.clone(),
            binding_version,
            baseline_commit: input.base_commit_sha.clone(),
            version: 1,
            lease: WriteLease {
                lease_id: format!("lease-{id}-0"),
                generation: 0,
                state: LeaseState::Revoked,
                holder: ProducerRef::HumanCommand {
                    command_id: format!("changeset:{}", input.key),
                },
            },
        },
    };
    Ok(HumanPlan {
        input,
        repo: Reference {
            key: repo.key,
            version: Version::State(repo.version),
        },
        change_set: set,
        previous: previous.as_ref().map(|r| Reference {
            key: r.key.clone(),
            version: Version::State(r.version),
        }),
    })
}

impl HumanPlan {
    /// Filled only with a native tool observation outside the admission transaction.
    pub fn seal_input(&self) -> Seal {
        Seal {
            association_key: self.input.key.clone(),
            change_set_id: self.change_set.change_set_id.clone(),
            change_set_version: self.change_set.version,
            lease: None,
            base_commit_sha: self.input.base_commit_sha.clone(),
            result_tree_sha: String::new(),
            result_commit_sha: None,
            parent_revision_id: self.input.parent_revision_id.clone(),
            producer_ref: ProducerRef::HumanCommand {
                command_id: format!("changeset:{}", self.input.key),
            },
        }
    }
}

pub fn admit_human(
    store: &mut Store,
    actor: &TrustedActor,
    plan: &HumanPlan,
    seal: &Seal,
    observation: &Value,
) -> Result<Value> {
    let actor = owner(actor, &plan.input.repo_id)?;
    let mut expected_seal = plan.seal_input();
    expected_seal.result_tree_sha = seal.result_tree_sha.clone();
    expected_seal.result_commit_sha = seal.result_commit_sha.clone();
    if *seal != expected_seal {
        return Err(reject(
            "PROPOSAL_MISMATCH",
            "human seal differs from the preview",
            "rebuild_preview",
        ));
    }
    if let OutputLocation::Commit { commit_sha, .. } = &plan.input.location {
        if seal.result_commit_sha.as_ref() != Some(commit_sha) {
            return Err(reject(
                "PROPOSAL_MISMATCH",
                "human seal differs from the declared commit",
                "rebuild_preview",
            ));
        }
    }
    let target = receipt_key(&plan.input.repo_id, &plan.input.key);
    if human_receipt(store, &actor, &plan.input)?.is_none()
        && prepare_human(store, &actor, plan.input.clone())? != *plan
    {
        return Err(reject(
            "VERSION_CONFLICT",
            "human preview no longer matches its source records",
            "rebuild_preview",
        ));
    }
    // Tool timestamps are observations, not the idempotency fingerprint.
    let input = json!({"plan":plan,"seal":seal});
    let cmd = command(
        &actor,
        target.clone(),
        &format!("changeset:{}", plan.input.key),
        &format!("human-seal:{}:{}", plan.input.repo_id, plan.input.key),
        Expected::Absent,
        "changeset.human",
        input,
    )?;
    store.submit(store.generation(), &actor, &cmd, None, |tx| {
        if tx.get(&plan.repo.key)?.map(|r| Reference {
            key: r.key,
            version: Version::State(r.version),
        }) != Some(plan.repo.clone())
        {
            return Err(reject(
                "VERSION_CONFLICT",
                "Repo changed before human admission",
                "rebuild_preview",
            ));
        }
        let key = change_set_key(&plan.input.repo_id, &plan.change_set.change_set_id);
        let current = tx.get(&key)?;
        if current.as_ref().map(|r| Reference {
            key: r.key.clone(),
            version: Version::State(r.version),
        }) != plan.previous
        {
            return Err(reject(
                "VERSION_CONFLICT",
                "ChangeSet changed before human admission",
                "rebuild_preview",
            ));
        }
        if current.is_none() {
            tx.put(&value_record(
                key,
                plan.change_set.version,
                &plan.change_set,
            )?)?;
        }
        let revision =
            admit_in_transaction(tx, &actor, &plan.input.repo_id, seal, OwnerGate::Active)?;
        let receipt =
            json!({"plan":plan,"seal":seal,"observation":observation,"revision":revision});
        tx.put(&value_record(target, 1, &receipt)?)?;
        Ok(receipt)
    })
}
