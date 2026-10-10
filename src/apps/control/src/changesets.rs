//! Native Git observations. Sealing and object delivery never admit a revision.
use std::{ffi::OsString, path::Path, sync::Arc};

use participant::{decode, invalid, reject};
use repo::changeset::{OutputLocation, Seal};
use serde_json::{Value, json};
use store::{Scope, Store, TrustedActor};
use tokio::sync::Mutex;

type Shared = Arc<Mutex<Option<Store>>>;

fn access<T>(shared: &Shared, f: impl FnOnce(&mut Store) -> store::Result<T>) -> store::Result<T> {
    let mut lock = shared.blocking_lock();
    f(lock.as_mut().ok_or_else(|| invalid("store not ready"))?)
}

pub(crate) fn preview(
    shared: &Shared,
    actor: &TrustedActor,
    operation: &str,
    payload: &Value,
) -> store::Result<Value> {
    if operation != "changeset.seal" {
        return Err(invalid("unknown ChangeSet command"));
    }
    let input: repo::changeset::HumanInput = serde_json::from_value(payload.clone())?;
    if let Some(receipt) = access(shared, |s| repo::changeset::human_receipt(s, actor, &input))? {
        return Ok(receipt);
    }
    let plan = access(shared, |s| repo::changeset::prepare_human(s, actor, input))?;
    // The preview retains Git bytes but grants nothing. Confirmation accepts these
    // bytes, not whatever happens to be in the source worktree on the next request.
    let (seal, observation) = self::seal(&plan.input.location, plan.seal_input())?;
    Ok(json!({"plan":plan,"seal":seal,"observation":observation}))
}

pub(crate) fn submit(
    shared: &Shared,
    actor: &TrustedActor,
    request: &proto::SubmitRequest,
    details: &Value,
) -> store::Result<Value> {
    let input: repo::changeset::HumanInput = serde_json::from_slice(&request.payload)?;
    if request.operation != "changeset.seal"
        || request.idempotency_key != input.key
        || request.command_id != format!("changeset:{}", input.key)
    {
        return Err(invalid("human seal envelope differs"));
    }
    let plan: repo::changeset::HumanPlan = serde_json::from_value(details["plan"].clone())?;
    let frozen: Seal = serde_json::from_value(details["seal"].clone())?;
    if plan.input != input {
        return Err(invalid("human seal preview input differs"));
    }
    if let Some(receipt) = access(shared, |s| repo::changeset::human_receipt(s, actor, &input))? {
        let observation = &receipt["observation"];
        return access(shared, |s| {
            repo::changeset::admit_human(s, actor, &plan, &frozen, observation)
        });
    }
    let (seal, observation) = self::seal(&plan.input.location, plan.seal_input())?;
    if seal != frozen {
        return Err(reject(
            "VERSION_CONFLICT",
            "human seal readback differs from confirmed preview",
            "rebuild_preview",
        ));
    }
    let registration = access(shared, |s| repo::require_active(s, &input.repo_id))?;
    deliver_objects(&registration, &seal, &observation)?;
    access(shared, |s| {
        repo::changeset::admit_human(s, actor, &plan, &seal, &observation)
    })
}

fn native(arguments: Vec<OsString>) -> store::Result<Value> {
    let output = tool::run(arguments)
        .map_err(|error| reject("GIT_SEAL_FAILED", error.to_string(), "inspect_change_set"))?;
    let value: Value = serde_json::from_str(output.body())?;
    if output.exit_code() != 0 || !value["error"].is_null() {
        return Err(reject(
            "GIT_SEAL_FAILED",
            value["error"].to_string(),
            "inspect_change_set",
        ));
    }
    Ok(value)
}

fn exact_sha(value: &Value, name: &str) -> store::Result<String> {
    let sha = value[name]
        .as_str()
        .ok_or_else(|| invalid(format!("{name} missing")))?;
    if sha.len() != 40
        || !sha
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(invalid(format!("exact lowercase {name} required")));
    }
    Ok(sha.into())
}

pub(crate) fn seal(location: &OutputLocation, mut seal: Seal) -> store::Result<(Seal, Value)> {
    let (path, commit) = match location {
        OutputLocation::Commit {
            repo_path,
            commit_sha,
        } => (repo_path, Some(commit_sha)),
        OutputLocation::Worktree { repo_path } | OutputLocation::NoChanges { repo_path } => {
            (repo_path, None)
        }
    };
    let mut args: Vec<OsString> = ["repo", "seal", "--path"].map(Into::into).into();
    args.extend([
        path.as_os_str().to_owned(),
        "--change-set-ref".into(),
        seal.change_set_id.clone().into(),
        "--baseline".into(),
        seal.base_commit_sha.clone().into(),
        "--key".into(),
        seal.association_key.clone().into(),
    ]);
    if let Some(commit) = commit {
        args.extend(["--commit".into(), commit.into()]);
    }
    let observation = native(args)?;
    if observation["schema"] != "hctl2.git-seal.v1"
        || observation["outcome"] != "established"
        || observation["base_commit_sha"] != seal.base_commit_sha
        || observation["change_set_id"] != seal.change_set_id
    {
        return Err(invalid(
            "Git seal readback differs from the declared boundary",
        ));
    }
    seal.result_tree_sha = exact_sha(&observation, "result_tree_sha")?;
    seal.result_commit_sha = Some(exact_sha(&observation, "result_commit_sha")?);
    let baseline_tree = exact_sha(&observation, "base_tree_sha")?;
    if matches!(location, OutputLocation::NoChanges { .. }) && baseline_tree != seal.result_tree_sha
    {
        return Err(reject(
            "NO_CHANGES_MISMATCH",
            "the reported unchanged worktree differs from the baseline tree",
            "submit_actual_changeset_output",
        ));
    }
    if commit.is_some_and(|commit| seal.result_commit_sha.as_ref() != Some(commit)) {
        return Err(invalid("Git seal differs from the exact proposed commit"));
    }
    Ok((seal, observation))
}

/// Move immutable objects into the already registered local delivery copy, when one
/// exists. Native fetch does not move the user's HEAD, branches or worktree files.
pub(crate) fn deliver_objects(
    registration: &repo::Registration,
    seal: &Seal,
    observation: &Value,
) -> store::Result<()> {
    let Some(destination) = registration.prepared.local.as_ref() else {
        // A remote-only registration still keeps the source retention ref. The
        // publishing worker reports its existing missing-local-delivery condition.
        return Ok(());
    };
    let source = Path::new(
        observation["repo_path"]
            .as_str()
            .ok_or_else(|| invalid("Git source missing"))?,
    );
    let commit = seal
        .result_commit_sha
        .as_deref()
        .ok_or_else(|| invalid("sealed commit missing"))?;
    let git = repo::git::Git::discover()?;
    let fetched = repo::git::run(
        git.command(&destination.path)
            .args(["fetch", "--no-tags", "--no-write-fetch-head", "--"])
            .arg(source)
            .arg(commit),
        None,
    )?;
    if !fetched.status.success() {
        return Err(reject(
            "GIT_DELIVERY_FAILED",
            "cannot retain sealed objects in the registered delivery copy",
            "inspect_change_set",
        ));
    }
    let (copy, _) = self::seal(
        &OutputLocation::Commit {
            repo_path: destination.path.clone(),
            commit_sha: commit.into(),
        },
        seal.clone(),
    )?;
    if copy != *seal {
        return Err(invalid(
            "delivery copy differs from the sealed Git identity",
        ));
    }
    Ok(())
}

pub(crate) fn query(
    shared: &Shared,
    actor: &TrustedActor,
    kind: &str,
    payload: &Value,
) -> store::Result<Value> {
    if actor.0.source != store::ActorSource::DirectClient
        || !actor.0.permission_scope.contains(&Scope::Control)
    {
        return Err(reject(
            "PERMISSION_DENIED",
            "ChangeSet query requires the control owner",
            "request_authorization",
        ));
    }
    let repo = payload["repo_id"]
        .as_str()
        .ok_or_else(|| invalid("repo_id required"))?;
    let id = payload["change_set_id"]
        .as_str()
        .ok_or_else(|| invalid("change_set_id required"))?;
    let (set, versions, registration, saved) = {
        let lock = shared.blocking_lock();
        let s = lock.as_ref().ok_or_else(|| invalid("store not ready"))?;
        let set = repo::changeset::get_change_set(s, repo, id)?;
        let versions = s
            .list("changeset_revision")?
            .into_iter()
            .filter(|r| r.key.scope == Scope::Repo(repo.into()))
            .map(|r| decode::<repo::changeset::ChangeSetRevision>(&r))
            .collect::<store::Result<Vec<_>>>()?
            .into_iter()
            .filter(|r| r.change_set_id == id)
            .collect::<Vec<_>>();
        let mut saved = Vec::new();
        for record in s
            .list("changeset_seal")?
            .into_iter()
            .chain(s.list("human_seal")?)
        {
            let value: Value = decode(&record)?;
            if value["seal"]["change_set_id"] == id {
                saved.push(value);
            }
        }
        (set, versions, repo::require_active(s, repo)?, saved)
    };
    if kind == "changeset.show" {
        return Ok(json!({"change_set":set,"revisions":versions,"saved_git_observations":saved}));
    }
    if kind != "changeset.diff" {
        return Err(invalid("unknown ChangeSet query"));
    }
    let revision_id = payload["revision_id"]
        .as_str()
        .ok_or_else(|| invalid("revision_id required"))?;
    let revision = versions
        .into_iter()
        .find(|r| r.change_set_revision_id == revision_id)
        .ok_or_else(|| {
            reject(
                "CHANGESET_REVISION_NOT_FOUND",
                "revision is outside this ChangeSet",
                "inspect_change_set",
            )
        })?;
    let source = saved
        .iter()
        .find(|s| {
            s["seal"]["base_commit_sha"] == revision.base_commit_sha
                && s["seal"]["result_tree_sha"] == revision.result_tree_sha
        })
        .and_then(|s| s["observation"]["repo_path"].as_str())
        .map(std::path::PathBuf::from);
    let path = registration
        .prepared
        .local
        .as_ref()
        .map(|s| &s.path)
        .or(source.as_ref())
        .ok_or_else(|| {
            reject(
                "LOCAL_GIT_DELIVERY_UNAVAILABLE",
                "no registered local copy for this exact diff",
                "inspect_change_set",
            )
        })?;
    native(vec![
        "repo".into(),
        "diff".into(),
        "--path".into(),
        path.as_os_str().to_owned(),
        "--base".into(),
        revision.base_commit_sha.into(),
        "--tree".into(),
        revision.result_tree_sha.into(),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_changes_claim_is_checked_against_the_actual_materialized_git_tree() {
        let root = std::env::temp_dir().join(format!(
            "hctl2-empty-seal-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(root.clone());
        let repository = root.join("repo");
        std::fs::create_dir(&repository).unwrap();
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .arg("-C")
                .arg(&repository)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .args(args)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8(output.stdout).unwrap().trim().to_owned()
        };
        git(&["init", "--initial-branch=main"]);
        git(&["config", "user.name", "Empty Seal Fixture"]);
        git(&["config", "user.email", "fixture@example.test"]);
        std::fs::write(repository.join("source.txt"), "baseline\n").unwrap();
        git(&["add", "source.txt"]);
        git(&["commit", "-m", "baseline"]);
        let baseline = git(&["rev-parse", "HEAD"]);
        let sites = root.join("sites");
        let arguments: Vec<OsString> = vec![
            "worktree".into(),
            "materialize".into(),
            "--repo".into(),
            repository.clone().into_os_string(),
            "--root".into(),
            sites.clone().into_os_string(),
            "--change-set-ref".into(),
            "cs-empty".into(),
            "--baseline".into(),
            baseline.clone().into(),
        ];
        let materialized = tool::run(arguments).unwrap();
        assert_eq!(materialized.exit_code(), 0, "{}", materialized.body());
        let input = Seal {
            association_key: "empty-first".into(),
            change_set_id: "cs-empty".into(),
            change_set_version: 1,
            lease: Some(repo::changeset::LeaseRef {
                lease_id: "lease-empty".into(),
                generation: 1,
            }),
            base_commit_sha: baseline,
            result_tree_sha: String::new(),
            result_commit_sha: None,
            parent_revision_id: None,
            producer_ref: repo::changeset::ProducerRef::Invocation {
                invocation_id: "fixture".into(),
                invocation_version: 1,
            },
        };
        let location = OutputLocation::NoChanges {
            repo_path: repository.clone(),
        };
        let (sealed, observation) = seal(&location, input.clone()).unwrap();
        assert_eq!(observation["base_tree_sha"], sealed.result_tree_sha);
        // Untracked edits also change the alternate-index snapshot. A model's
        // unchanged claim must not hide them, or reuse a prior proposal's seal.
        std::fs::write(sites.join("cs-empty/untracked.txt"), "actual edit\n").unwrap();
        let mut changed = input;
        changed.association_key = "empty-second".into();
        let error = seal(&location, changed.clone()).unwrap_err();
        assert_eq!(error.code, "NO_CHANGES_MISMATCH");
        assert_eq!(error.recovery_action, "submit_actual_changeset_output");
        let (sealed, observation) = seal(
            &OutputLocation::Worktree {
                repo_path: repository,
            },
            changed,
        )
        .unwrap();
        assert_ne!(observation["base_tree_sha"], sealed.result_tree_sha);
    }
}
