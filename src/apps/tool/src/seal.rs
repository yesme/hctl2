//! A reachable Git snapshot and readback, not admission or publication authority.
use crate::{
    ToolError, ToolOutput, archive,
    git::{Git, args},
    observed_at_unix_ms,
    repository::{Repository, resolve_commit},
    site_lock::SiteLock,
};
use serde_json::{Value, json};
use std::{ffi::OsString, path::PathBuf};

pub(crate) fn seal(
    git: &Git,
    path: PathBuf,
    change_set: String,
    baseline: String,
    commit: Option<String>,
    key: Option<String>,
) -> Result<ToolOutput, ToolError> {
    exact_sha(&baseline)?;
    if let Some(commit) = &commit {
        exact_sha(commit)?;
    }
    if let Some(key) = &key
        && (key.trim().is_empty() || key.trim() != key)
    {
        return Err(ToolError::new(
            "HCTL2_TOOL_INVALID_ARGUMENT",
            "exact nonempty seal key required",
        ));
    }
    let association = key.as_ref().map(|key| {
        format!(
            "refs/hctl2/changesets/{change_set}/seals/{baseline}/key-{}",
            foundation::bytes_sha256(key.as_bytes())
        )
    });
    let initial = Repository::open(git, &path)?;
    let previous = association
        .as_ref()
        .map(|reference| crate::worktree::read_optional_commit(git, &initial.anchor, reference))
        .transpose()?
        .flatten();
    if previous
        .as_ref()
        .zip(commit.as_ref())
        .is_some_and(|(old, requested)| old != requested)
    {
        return Err(ToolError::not_established(
            "HCTL2_TOOL_SEAL_CONFLICT",
            "seal association already retains another proposed commit",
        ));
    }
    // Reuse P1's alternate index and archive implementation for an uncommitted tree.
    // Its lock is released before we lock again for the immutable object readback.
    let reused = previous.is_some();
    let commit = if let Some(commit) = previous.or(commit) {
        commit
    } else {
        let snapshot = archive::snapshot(git, path.clone(), change_set.clone())?;
        let snapshot: Value = serde_json::from_str(snapshot.body())
            .map_err(|e| ToolError::new("HCTL2_TOOL_GIT_OUTPUT_INVALID", e.to_string()))?;
        if snapshot["baseline_commit_sha"] != baseline {
            return Err(ToolError::not_established(
                "HCTL2_TOOL_BASELINE_MISMATCH",
                "worktree baseline differs from the declared baseline",
            ));
        }
        snapshot["snapshot_commit_sha"]
            .as_str()
            .ok_or_else(|| {
                ToolError::new("HCTL2_TOOL_GIT_OUTPUT_INVALID", "snapshot commit missing")
            })?
            .to_owned()
    };
    let repository = Repository::open(git, &path)?;
    let _lock = SiteLock::acquire(&repository.common_dir, "changeset_seal", Some(&change_set))?;
    let base = resolve_commit(
        git,
        &repository.anchor,
        &baseline,
        "HCTL2_TOOL_BASELINE_MISSING",
    )?;
    let commit = resolve_commit(
        git,
        &repository.anchor,
        &commit,
        "HCTL2_TOOL_COMMIT_MISSING",
    )?;
    let ancestry = git.invoke(
        &repository.anchor,
        &args(&["merge-base", "--is-ancestor", &base, &commit]),
    )?;
    if !ancestry.success() {
        return Err(ToolError::not_established(
            "HCTL2_TOOL_BASELINE_MISMATCH",
            "result commit does not descend from the declared baseline",
        ));
    }
    let tree = git
        .checked(
            &repository.anchor,
            &args(&["rev-parse", "--verify", &format!("{commit}^{{tree}}")]),
            "HCTL2_TOOL_GIT_READ_FAILED",
            "read result tree",
        )?
        .stdout_text()?;
    exact_sha(&tree)?;
    let base_tree = git
        .checked(
            &repository.anchor,
            &args(&["rev-parse", "--verify", &format!("{base}^{{tree}}")]),
            "HCTL2_TOOL_GIT_READ_FAILED",
            "read baseline tree",
        )?
        .stdout_text()?;
    exact_sha(&base_tree)?;
    // A tree/base pair is the version identity; a retained ref protects the packaging
    // commit from collection. Repeated sealing does not need to move a user's branch.
    let retained = format!("refs/hctl2/changesets/{change_set}/seals/{base}/{tree}/{commit}");
    git.checked(
        &repository.anchor,
        &args(&["update-ref", &retained, &commit]),
        "HCTL2_TOOL_SEAL_FAILED",
        "retain sealed Git objects",
    )?;
    let readback = resolve_commit(git, &repository.anchor, &retained, "HCTL2_TOOL_SEAL_FAILED")?;
    if readback != commit {
        return Err(ToolError::not_established(
            "HCTL2_TOOL_SEAL_FAILED",
            "retained snapshot differs on readback",
        ));
    }
    if let Some(association) = &association {
        // Compare-and-swap prevents two retrying tool processes from moving a fixed
        // association. The repository's existing site lock serializes this write.
        if !reused {
            git.checked(
                &repository.anchor,
                &args(&["update-ref", association, &commit, &"0".repeat(40)]),
                "HCTL2_TOOL_SEAL_CONFLICT",
                "freeze seal association",
            )?;
        }
        if resolve_commit(
            git,
            &repository.anchor,
            association,
            "HCTL2_TOOL_SEAL_FAILED",
        )? != commit
        {
            return Err(ToolError::not_established(
                "HCTL2_TOOL_SEAL_CONFLICT",
                "seal association differs on readback",
            ));
        }
    }
    Ok(ToolOutput::json(
        json!({
            "schema":"hctl2.git-seal.v1", "evidence_level":"unmediated",
            "outcome":"established", "observed_at_unix_ms":observed_at_unix_ms(),
            "git":{"path":git.executable(),"version":git.version()},
            "repo_path":repository.anchor, "change_set_id":change_set,
            "base_commit_sha":base,"base_tree_sha":base_tree,"result_tree_sha":tree,"result_commit_sha":commit,
        "retained_ref":retained, "association_ref":association, "reused":reused,"error":Value::Null,
        }),
        0,
    ))
}

pub(crate) fn diff(
    git: &Git,
    path: PathBuf,
    base: String,
    tree: String,
) -> Result<ToolOutput, ToolError> {
    exact_sha(&base)?;
    exact_sha(&tree)?;
    let repository = Repository::open(git, &path)?;
    let output = git.checked(
        &repository.anchor,
        &[
            OsString::from("diff"),
            "--no-ext-diff".into(),
            "--no-textconv".into(),
            "--binary".into(),
            base.clone().into(),
            tree.clone().into(),
            "--".into(),
        ],
        "HCTL2_TOOL_DIFF_FAILED",
        "read exact base-to-tree diff",
    )?;
    Ok(ToolOutput::json(
        json!({"schema":"hctl2.git-diff.v1","evidence_level":"unmediated","outcome":"established","base_commit_sha":base,"result_tree_sha":tree,"diff":output.stdout_text()?,"error":Value::Null}),
        0,
    ))
}

fn exact_sha(sha: &str) -> Result<(), ToolError> {
    if sha.len() != 40
        || !sha
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(ToolError::new(
            "HCTL2_TOOL_INVALID_ARGUMENT",
            "expected exact lowercase SHA-1",
        ));
    }
    Ok(())
}
