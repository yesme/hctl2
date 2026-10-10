//! A confirmed discard is durable before physical cleanup. Lease state is untouched.
use super::*;
use store::{EffectIntent, Readback};

pub fn key(repo: &str, command_key: &str) -> ObjectKey {
    ObjectKey {
        scope: Scope::Repo(repo.into()),
        kind: "changeset_residual".into(),
        id: format!("changeset:discard:{command_key}"),
    }
}
fn owner(actor: &TrustedActor, repo: &str) -> Result<TrustedActor> {
    if actor.0.source != ActorSource::DirectClient
        || !actor.0.permission_scope.contains(&Scope::Control)
    {
        return Err(reject(
            "PERMISSION_DENIED",
            "residual cleanup requires the control owner",
            "request_authorization",
        ));
    }
    let mut actor = actor.0.clone();
    actor.permission_scope.push(Scope::Repo(repo.into()));
    Ok(TrustedActor(actor))
}
pub fn receipt(
    store: &Store,
    actor: &TrustedActor,
    repo: &str,
    command_key: &str,
    input: &Value,
) -> Result<Option<Value>> {
    let _ = owner(actor, repo)?;
    let Some(record) = store.get(&key(repo, command_key))? else {
        return Ok(None);
    };
    let value: Value = decode(&record)?;
    if value["plan"]["input"] != *input {
        return Err(reject(
            "IDEMPOTENCY_CONFLICT",
            "residual command key has another input",
            "use_new_command_key",
        ));
    }
    Ok(Some(value))
}
pub fn begin(
    store: &mut Store,
    actor: &TrustedActor,
    repo: &str,
    command_key: &str,
    plan: &Value,
    source: &Reference,
    repo_ref: &Reference,
) -> Result<Value> {
    if let Some(receipt) = receipt(store, actor, repo, command_key, &plan["input"])? {
        return Ok(receipt);
    }
    let actor = owner(actor, repo)?;
    let target = key(repo, command_key);
    let cmd = command(
        &actor,
        target.clone(),
        &target.id,
        &target.id,
        Expected::Absent,
        "changeset.discard",
        plan.clone(),
    )?;
    let effect = EffectIntent {
        intent_id: target.id.clone(),
        owner: Reference {
            key: target.clone(),
            version: Version::State(1),
        },
        binding: repo_ref.clone(),
        target: plan["worktree_path"]
            .as_str()
            .ok_or_else(|| reject("INVALID_INPUT", "worktree path missing", "rebuild_preview"))?
            .into(),
        conflict_scope: format!("discard:{repo}:{}", source.key.id),
        permission_scope: Scope::Repo(repo.into()),
        operation: "git.worktree.remove".into(),
        input: plan.clone(),
        input_digest: Command::digest_input("git.worktree.remove", plan)?,
        idempotency_key: target.id.clone(),
    };
    store.submit(store.generation(), &actor, &cmd, None, |tx| {
        if tx.get(&repo_ref.key)?.map(|r| Reference {
            key: r.key,
            version: Version::State(r.version),
        }) != Some(repo_ref.clone())
        {
            return Err(reject(
                "VERSION_CONFLICT",
                "residual Repo changed",
                "rebuild_preview",
            ));
        }
        if tx.get(&source.key)?.map(|r| Reference {
            key: r.key,
            version: Version::State(r.version),
        }) != Some(source.clone())
        {
            return Err(reject(
                "VERSION_CONFLICT",
                "residual source changed",
                "rebuild_preview",
            ));
        }
        tx.enqueue_effect(&effect)?;
        let value = json!({"status":"pending","plan":plan,"effect_id":effect.intent_id});
        tx.put(&value_record(target, 1, &value)?)?;
        Ok(value)
    })
}
pub fn finish(
    store: &mut Store,
    actor: &TrustedActor,
    repo: &str,
    command_key: &str,
    observation: &Value,
) -> Result<Value> {
    settle(store, actor, repo, command_key, observation, true)
}
pub fn refused(
    store: &mut Store,
    actor: &TrustedActor,
    repo: &str,
    command_key: &str,
    observation: &Value,
) -> Result<Value> {
    settle(store, actor, repo, command_key, observation, false)
}
fn settle(
    store: &mut Store,
    actor: &TrustedActor,
    repo: &str,
    command_key: &str,
    observation: &Value,
    established: bool,
) -> Result<Value> {
    let actor = owner(actor, repo)?;
    let target = key(repo, command_key);
    let old = store.get(&target)?.ok_or_else(|| {
        reject(
            "INVALID_INPUT",
            "discard intent missing",
            "inspect_change_set",
        )
    })?;
    let mut value: Value = decode(&old)?;
    if value["status"] == "discarded" || value["status"] == "rejected" {
        return Ok(value);
    }
    let (effect, _) = store.effect(&target.id)?;
    value["status"] = json!(if established { "discarded" } else { "rejected" });
    value["observation"] = observation.clone();
    let cmd = command(
        &actor,
        target.clone(),
        &format!("{}:confirm", target.id),
        &format!("{}:confirm", target.id),
        Expected::Exact(Version::State(old.version)),
        "changeset.discard_confirmed",
        json!({"effect_id":effect.intent_id}),
    )?;
    store.submit(store.generation(), &actor, &cmd, None, |tx| {
        let readback = if established {
            Readback::Confirmed {
                binding: effect.binding.clone(),
                target: effect.target.clone(),
                input_digest: effect.input_digest.clone(),
                result: observation.clone(),
            }
        } else {
            Readback::Rejected {
                binding: effect.binding.clone(),
                target: effect.target.clone(),
                input_digest: effect.input_digest.clone(),
                result: observation.clone(),
            }
        };
        tx.confirm_effect(&effect.intent_id, &readback)?;
        tx.put(&value_record(target, old.version + 1, &value)?)?;
        Ok(value)
    })
}
