//! Explicit human recovery reuses P1 snapshots and human admission, not old leases.
use super::*;
use repo::changeset::{self as domain, HumanInput, HumanPlan, LeaseState};
use serde::{Deserialize, Serialize};
use store::Reference;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    key: String,
    repo_id: String,
    change_set_id: String,
    repo_path: std::path::PathBuf,
    #[serde(default)]
    target_change_set_id: Option<String>,
    #[serde(default)]
    parent_revision_id: Option<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    input: Input,
    operation: String,
    source: Reference,
    repo: Reference,
    worktree_path: std::path::PathBuf,
    anchor: std::path::PathBuf,
    git_common_dir: std::path::PathBuf,
    worktree_git_dir: std::path::PathBuf,
    source_observation: Value,
    git_status: String,
    discard_scope: String,
    human: Option<HumanPlan>,
    seal: Option<Seal>,
}
fn action(operation: &str) -> store::Result<&str> {
    match operation {
        "changeset.takeover" => Ok("takeover"),
        "changeset.adopt" => Ok("adopt"),
        "changeset.discard" => Ok("discard"),
        _ => Err(invalid("unknown residual command")),
    }
}
fn human_key(input: &Input, action: &str) -> String {
    format!("{action}:{}:{}", input.repo_id, input.key)
}
fn inspect(path: &Path) -> store::Result<Value> {
    native(vec![
        "repo".into(),
        "inspect".into(),
        "--path".into(),
        path.as_os_str().to_owned(),
    ])
}
fn worktree_git_dir(path: &Path) -> store::Result<std::path::PathBuf> {
    let git = repo::git::Git::discover()?;
    let output = repo::git::run(
        git.command(path).args(["rev-parse", "--absolute-git-dir"]),
        None,
    )?;
    if !output.status.success() {
        return Err(reject(
            "GIT_SEAL_FAILED",
            "worktree Git directory unreadable",
            "inspect_change_set",
        ));
    }
    Ok(String::from_utf8(output.stdout)
        .map_err(|_| invalid("worktree Git directory is not UTF-8"))?
        .trim()
        .into())
}
pub(super) fn git_status(path: &Path) -> store::Result<String> {
    let git = repo::git::Git::discover()?;
    let status = repo::git::run(
        git.command(path).args([
            "status",
            "--porcelain=v2",
            "--untracked-files=all",
            "--ignored",
        ]),
        None,
    )?;
    if !status.status.success() {
        return Err(reject(
            "GIT_SEAL_FAILED",
            "cannot read residual Git status",
            "inspect_change_set",
        ));
    }
    String::from_utf8(status.stdout).map_err(|_| invalid("Git status is not UTF-8"))
}
pub(super) fn preview(
    shared: &Shared,
    actor: &TrustedActor,
    operation: &str,
    payload: &Value,
) -> store::Result<Value> {
    let action = action(operation)?;
    if actor.0.source != store::ActorSource::DirectClient
        || !actor.0.permission_scope.contains(&Scope::Control)
    {
        return Err(reject(
            "PERMISSION_DENIED",
            "residual recovery requires the control owner",
            "request_authorization",
        ));
    }
    let input: Input = serde_json::from_value(payload.clone())?;
    if input.key.trim().is_empty()
        || input.key.trim() != input.key
        || !input.repo_path.is_absolute()
    {
        return Err(invalid("exact key and absolute worktree path required"));
    }
    if action != "adopt" && input.target_change_set_id.is_some()
        || action == "adopt" && input.target_change_set_id.as_deref() == Some(&input.change_set_id)
    {
        return Err(invalid(
            "only adoption can name a different target ChangeSet",
        ));
    }
    if action == "discard" {
        if input.parent_revision_id.is_some() {
            return Err(invalid("discard has no parent revision"));
        }
        if let Some(saved) = access(shared, |s| {
            domain::residual::receipt(
                s,
                actor,
                &input.repo_id,
                &input.key,
                &serde_json::to_value(&input)?,
            )
        })? {
            return Ok(saved["plan"].clone());
        }
    } else if let Some(receipt) = access(shared, |s| {
        let key = store::ObjectKey {
            scope: Scope::Repo(input.repo_id.clone()),
            kind: "human_seal".into(),
            id: format!("changeset:{}", human_key(&input, action)),
        };
        s.get(&key)?.map(|r| decode::<Value>(&r)).transpose()
    })? {
        if receipt["observation"]["residual"]["input"] != serde_json::to_value(&input)? {
            return Err(reject(
                "IDEMPOTENCY_CONFLICT",
                "residual command key has another input",
                "use_new_command_key",
            ));
        }
        return Ok(receipt["observation"]["residual"].clone());
    }
    let (source, source_plan) = access(shared, |s| {
        let set = domain::get_change_set(s, &input.repo_id, &input.change_set_id)?;
        if !matches!(set.lease.state, LeaseState::Revoking | LeaseState::Revoked) {
            return Err(reject(
                "RESIDUAL_NOT_REVOKED",
                "writer must first lose authorization",
                "cancel_original_invocation",
            ));
        }
        let plan = domain::prepare_human(
            s,
            actor,
            HumanInput {
                key: human_key(&input, action),
                repo_id: input.repo_id.clone(),
                change_set_id: Some(input.change_set_id.clone()),
                base_commit_sha: set.baseline_commit,
                parent_revision_id: None,
                location: OutputLocation::Worktree {
                    repo_path: input.repo_path.clone(),
                },
            },
        )?;
        Ok((
            plan.previous
                .clone()
                .ok_or_else(|| invalid("source reference missing"))?,
            plan,
        ))
    })?;
    let (source_seal, observation) = seal(&source_plan.input.location, source_plan.seal_input())?;
    let identity = inspect(&input.repo_path)?;
    let worktree_path: std::path::PathBuf = identity["repository_state"]["worktree_root"]
        .as_str()
        .ok_or_else(|| invalid("worktree root missing"))?
        .into();
    if input
        .repo_path
        .canonicalize()
        .map_err(|_| invalid("worktree inaccessible"))?
        != worktree_path
    {
        return Err(invalid("name the exact worktree root, not a subdirectory"));
    }
    let git_status = git_status(&worktree_path)?;
    let worktree_git_dir = worktree_git_dir(&worktree_path)?;
    let anchor: std::path::PathBuf = identity["common_directory_identity"]["worktrees"]
        .as_array()
        .ok_or_else(|| invalid("worktree list missing"))?
        .iter()
        .find(|w| {
            w["path"]
                .as_str()
                .is_some_and(|p| Path::new(p) != worktree_path)
                && w["bare"] != true
        })
        .and_then(|w| w["path"].as_str())
        .ok_or_else(|| invalid("P1 residual must have a surviving repository anchor"))?
        .into();
    let (human, target_seal) = if action == "discard" {
        (None, None)
    } else {
        let plan = access(shared, |s| {
            domain::prepare_human(
                s,
                actor,
                HumanInput {
                    key: human_key(&input, action),
                    repo_id: input.repo_id.clone(),
                    change_set_id: if action == "takeover" {
                        Some(input.change_set_id.clone())
                    } else {
                        input.target_change_set_id.clone()
                    },
                    base_commit_sha: source_seal.base_commit_sha.clone(),
                    parent_revision_id: input.parent_revision_id.clone(),
                    location: OutputLocation::Commit {
                        repo_path: input.repo_path.clone(),
                        commit_sha: source_seal
                            .result_commit_sha
                            .clone()
                            .ok_or_else(|| invalid("snapshot commit missing"))?,
                    },
                },
            )
        })?;
        let (sealed, _) = seal(&plan.input.location, plan.seal_input())?;
        (Some(plan), Some(sealed))
    };
    let git_common_dir = identity["common_directory_identity"]["git_common_dir"]
        .as_str()
        .ok_or_else(|| invalid("common directory missing"))?
        .into();
    Ok(serde_json::to_value(Plan {
        input,
        operation: operation.into(),
        source,
        repo: source_plan.repo,
        worktree_path,
        anchor,
        git_common_dir,
        worktree_git_dir,
        source_observation: observation,
        git_status,
        discard_scope: "entire confirmed worktree, including ignored files; Git snapshot excludes ignored files".into(),
        human,
        seal: target_seal,
    })?)
}
pub(super) fn submit(
    shared: &Shared,
    actor: &TrustedActor,
    request: &proto::SubmitRequest,
    details: &Value,
) -> store::Result<Value> {
    let action = action(&request.operation)?;
    let input: Input = serde_json::from_slice(&request.payload)?;
    let plan: Plan = serde_json::from_value(details.clone())?;
    if plan.input != input
        || plan.operation != request.operation
        || request.idempotency_key != input.key
        || request.command_id != format!("changeset:{action}:{}", input.key)
    {
        return Err(invalid("residual confirmation differs from preview"));
    }
    if action != "discard" {
        let human = plan
            .human
            .as_ref()
            .ok_or_else(|| invalid("human admission missing"))?;
        if let Some(receipt) = access(shared, |s| domain::human_receipt(s, actor, &human.input))? {
            return Ok(receipt);
        }
        let frozen = plan
            .seal
            .as_ref()
            .ok_or_else(|| invalid("frozen snapshot missing"))?;
        let (current, mut observation) = seal(&human.input.location, human.seal_input())?;
        if current != *frozen {
            return Err(reject(
                "VERSION_CONFLICT",
                "frozen residual bytes changed",
                "rebuild_preview",
            ));
        }
        observation["residual"] = details.clone();
        let registration = access(shared, |s| repo::require_active(s, &input.repo_id))?;
        deliver_objects(&registration, frozen, &observation)?;
        return access(shared, |s| {
            domain::admit_human_checked(
                s,
                actor,
                human,
                frozen,
                &observation,
                std::slice::from_ref(&plan.source),
            )
        });
    }
    let saved = access(shared, |s| {
        domain::residual::begin(
            s,
            actor,
            &input.repo_id,
            &input.key,
            details,
            &plan.source,
            &plan.repo,
        )
    })?;
    if saved["status"] == "discarded" {
        return Ok(saved);
    }
    if saved["status"] == "rejected" {
        return Err(reject(
            "RESIDUAL_DISCARD_REJECTED",
            "previous attempt proved no deletion",
            "preview_with_new_command_key",
        ));
    }
    let effect = saved["effect_id"]
        .as_str()
        .ok_or_else(|| invalid("discard effect missing"))?;
    let previously_sent = access(shared, |s| {
        let (_, state) = s.effect(effect)?;
        if state == store::EffectState::Pending {
            s.resume_pending_effect(s.generation(), effect, true)?;
            s.begin_effect(s.generation(), effect)?;
            return Ok(false);
        }
        Ok(true)
    })?;
    let inspected = inspect(&plan.anchor)?;
    if inspected["common_directory_identity"]["git_common_dir"]
        != plan.git_common_dir.to_string_lossy().as_ref()
    {
        let error = reject(
            "RESIDUAL_PATH_CHANGED",
            "repository anchor no longer names the confirmed repository",
            "inspect_change_set",
        );
        if !previously_sent {
            reject_discard(shared, actor, &input, &error)?;
        }
        return Err(error);
    }
    let trees = inspected["common_directory_identity"]["worktrees"]
        .as_array()
        .ok_or_else(|| invalid("worktree list missing"))?;
    let original = trees
        .iter()
        .find(|w| w["path"] == plan.worktree_path.to_string_lossy().as_ref());
    if original.is_none()
        && (plan.worktree_git_dir.exists()
            || trees.iter().any(|w| {
                w["branch"] == format!("refs/heads/hctl2/changeset/{}", input.change_set_id)
            }))
    {
        let error = reject(
            "RESIDUAL_PATH_CHANGED",
            "confirmed worktree was moved, not discarded",
            "inspect_change_set",
        );
        if !previously_sent {
            reject_discard(shared, actor, &input, &error)?;
        }
        return Err(error);
    }
    let observation = if original.is_none() && !plan.worktree_path.exists() {
        json!({"outcome":"established","operation":"removed_discarded","worktree_path":plan.worktree_path,"recovered_by_readback":true})
    } else {
        if previously_sent {
            return Err(reject(
                "RESULT_UNKNOWN",
                "original discard is unresolved and the confirmed directory remains; read back only",
                "inspect_with_hctl2_tool_then_retry_same_command",
            ));
        }
        if original.is_none_or(|w| {
            w["branch"] != format!("refs/heads/hctl2/changeset/{}", input.change_set_id)
                && !(w["detached"] == true
                    && plan
                        .worktree_path
                        .file_name()
                        .is_some_and(|name| name == input.change_set_id.as_str()))
        }) {
            let error = reject(
                "RESIDUAL_PATH_CHANGED",
                "directory is not the confirmed P1 worktree",
                "inspect_change_set",
            );
            reject_discard(shared, actor, &input, &error)?;
            return Err(error);
        }
        let result = native(vec![
            "archive".into(),
            "remove".into(),
            "--repo".into(),
            plan.anchor.as_os_str().to_owned(),
            "--change-set-ref".into(),
            input.change_set_id.clone().into(),
            "--discard-unarchived".into(),
            "--confirm-discard".into(),
            exact_sha(&plan.source_observation, "result_tree_sha")?.into(),
            "--expected-worktree".into(),
            plan.worktree_path.as_os_str().to_owned(),
        ]);
        match result {
            Ok(value) => value,
            Err(error) => {
                let refusal = serde_json::from_str::<Value>(&error.message).unwrap_or(Value::Null);
                if refusal["code"] == "HCTL2_TOOL_DISCARD_CONFIRMATION_MISMATCH"
                    || refusal["code"] == "HCTL2_TOOL_WORKTREE_PATH_CHANGED"
                {
                    access(shared, |s| {
                        domain::residual::refused(s, actor, &input.repo_id, &input.key, &refusal)
                    })?;
                }
                return Err(error);
            }
        }
    };
    access(shared, |s| {
        domain::residual::finish(s, actor, &input.repo_id, &input.key, &observation)
    })
}

fn reject_discard(
    shared: &Shared,
    actor: &TrustedActor,
    input: &Input,
    error: &store::StoreError,
) -> store::Result<()> {
    access(shared, |s| {
        domain::residual::refused(
            s,
            actor,
            &input.repo_id,
            &input.key,
            &json!({"code":error.code,"message":error.message,"recovery_action":error.recovery_action}),
        )
    })?;
    Ok(())
}

pub(super) fn list(store: &Store, repo_id: &str, change_set_id: &str) -> store::Result<Vec<Value>> {
    let mut values = Vec::new();
    for record in store.list("changeset_residual")? {
        if record.key.scope != Scope::Repo(repo_id.into()) {
            continue;
        }
        let value: Value = decode(&record)?;
        if value["plan"]["input"]["change_set_id"] == change_set_id {
            values.push(value);
        }
    }
    for record in store.list("proposal_inbox")? {
        let proposal: agency_proto::Proposal = decode(&record)?;
        if proposal.schema != "hctl2.changeset-output.v1" {
            continue;
        }
        let Ok(output) = serde_json::from_slice::<domain::Output>(&proposal.output) else {
            continue;
        };
        if output.change_set_id == change_set_id {
            let result_key = store::ObjectKey {
                scope: record.key.scope.clone(),
                kind: "invocation_result".into(),
                id: format!(
                    "{}:{}",
                    proposal.header.owner.id, proposal.header.proposal_id
                ),
            };
            if store.get(&result_key)?.is_some() {
                continue;
            }
            let source = domain::get_change_set(store, repo_id, change_set_id)?;
            if !matches!(&source.lease.holder, domain::ProducerRef::Invocation{invocation_id, ..} if *invocation_id == proposal.header.owner.id)
                || source.lease.lease_id != output.lease.lease_id
            {
                continue;
            }
            values.push(json!({"proposal_id":proposal.header.proposal_id,"location":output.location,"lease":output.lease,"status":"preserved_proposal"}));
        }
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn human_recovery_command_identity_includes_repo_and_action() {
        let mut input: Input = serde_json::from_value(json!({
            "key":"same", "repo_id":"A", "change_set_id":"residual", "repo_path":"/fixture"
        }))
        .unwrap();
        let a = human_key(&input, "takeover");
        assert_ne!(a, human_key(&input, "adopt"));
        input.repo_id = "B".into();
        assert_ne!(a, human_key(&input, "takeover"));
    }
}
