use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::git::{Git, args};
use crate::repository::{Repository, WorktreeEntry, resolve_commit};
use crate::site_lock::SiteLock;
use crate::{ToolError, ToolOutput, observed_at_unix_ms};

const BRANCH_PREFIX: &str = "hctl2/changeset/";
const BASELINE_REF_PREFIX: &str = "refs/hctl2/changesets/";

pub(crate) fn materialize(
    repository_path: PathBuf,
    root: PathBuf,
    change_set_ref: String,
    baseline: String,
) -> Result<ToolOutput, ToolError> {
    validate_change_set_ref(&change_set_ref)?;
    let git = Git::discover()?;
    let repository = Repository::open(&git, &repository_path)?;
    let baseline = resolve_commit(
        &git,
        &repository.anchor,
        &baseline,
        "HCTL2_TOOL_BASELINE_NOT_FOUND",
    )?;
    let branch = branch_ref(&change_set_ref);
    let baseline_ref = baseline_ref(&change_set_ref);
    let recorded_baseline = read_optional_commit(&git, &repository.anchor, &baseline_ref)?;
    if let Some(recorded) = &recorded_baseline
        && recorded != &baseline
    {
        return Err(ToolError::new(
            "HCTL2_TOOL_BASELINE_MISMATCH",
            format!("ChangeSet {change_set_ref} was materialized from {recorded}, not {baseline}"),
        ));
    }

    let existing = repository
        .worktrees(&git)?
        .into_iter()
        .find(|entry| entry.branch.as_deref() == Some(branch.as_str()));
    if existing.is_some() && recorded_baseline.is_some() {
        let verification = verify_state(&git, &repository, &change_set_ref)?;
        if !verification.valid {
            return Err(ToolError::new(
                "HCTL2_TOOL_WORKTREE_STATE_INVALID",
                format!(
                    "existing ChangeSet worktree failed verification: {}",
                    verification.failure_summary()
                ),
            ));
        }
        return Ok(verification.output(&git, "already_materialized", None));
    }

    let mut site_lock = Some(SiteLock::acquire(
        &repository.common_dir,
        "worktree_materialize",
        Some(&change_set_ref),
    )?);
    let root = prepare_root(&root, &repository.common_dir)?;
    let target = root.join(&change_set_ref);

    // Re-read under the lock so two callers cannot both create the branch.
    let recorded_baseline = read_optional_commit(&git, &repository.anchor, &baseline_ref)?;
    if let Some(recorded) = &recorded_baseline
        && recorded != &baseline
    {
        return Err(ToolError::new(
            "HCTL2_TOOL_BASELINE_MISMATCH",
            format!("ChangeSet {change_set_ref} was materialized from {recorded}, not {baseline}"),
        ));
    }
    if let Some(entry) = repository
        .worktrees(&git)?
        .into_iter()
        .find(|entry| entry.branch.as_deref() == Some(branch.as_str()))
    {
        recover_missing_baseline_marker(
            &git,
            &repository,
            &entry,
            &target,
            &baseline_ref,
            &baseline,
            recorded_baseline.as_deref(),
        )?;
        let verification = verify_state(&git, &repository, &change_set_ref)?;
        if !verification.valid {
            return Err(ToolError::new(
                "HCTL2_TOOL_WORKTREE_STATE_INVALID",
                verification.failure_summary(),
            ));
        }
        return Ok(verification.output(&git, "already_materialized", site_lock.take()));
    }

    if target.exists() {
        return Err(ToolError::new(
            "HCTL2_TOOL_WORKTREE_PATH_OCCUPIED",
            format!(
                "materialization path exists but is not the ChangeSet worktree: {}",
                target.display()
            ),
        ));
    }

    let existing_branch = read_optional_commit(&git, &repository.anchor, &branch)?;
    if let Some(commit) = &existing_branch
        && commit != &baseline
    {
        return Err(ToolError::new(
            "HCTL2_TOOL_WORKTREE_STATE_INVALID",
            format!("reserved branch {branch} points to {commit}, expected {baseline}"),
        ));
    }
    let mut arguments = vec![OsString::from("worktree"), OsString::from("add")];
    if existing_branch.is_none() {
        arguments.extend([
            OsString::from("-b"),
            OsString::from(short_branch(&change_set_ref)),
        ]);
    }
    arguments.push(target.clone().into_os_string());
    arguments.push(OsString::from(
        existing_branch
            .as_ref()
            .map_or(baseline.as_str(), |_| branch.as_str()),
    ));
    git.checked(
        &repository.anchor,
        &arguments,
        "HCTL2_TOOL_WORKTREE_MATERIALIZE_FAILED",
        "materialize worktree",
    )?;

    if recorded_baseline.is_none() {
        create_baseline_marker(&git, &repository, &baseline_ref, &baseline)?;
    }
    let verification = verify_state(&git, &repository, &change_set_ref)?;
    if !verification.valid {
        return Err(ToolError::new(
            "HCTL2_TOOL_WORKTREE_VERIFICATION_FAILED",
            verification.failure_summary(),
        ));
    }
    Ok(verification.output(&git, "created", site_lock.take()))
}

pub(crate) fn verify(
    repository_path: PathBuf,
    change_set_ref: String,
) -> Result<ToolOutput, ToolError> {
    validate_change_set_ref(&change_set_ref)?;
    let git = Git::discover()?;
    let repository = Repository::open(&git, &repository_path)?;
    let verification = verify_state(&git, &repository, &change_set_ref)?;
    let exit_code = if verification.valid { 0 } else { 3 };
    Ok(verification.output_with_code(&git, "verified", None, exit_code))
}

#[derive(Debug)]
struct Verification {
    change_set_ref: String,
    baseline: Option<String>,
    worktree: Option<WorktreeEntry>,
    path_exists: bool,
    branch_matches: bool,
    branch_head_matches: bool,
    baseline_is_ancestor: bool,
    tracked_changes: usize,
    untracked_changes: usize,
    valid: bool,
}

impl Verification {
    fn output(&self, git: &Git, action: &str, lock: Option<SiteLock>) -> ToolOutput {
        self.output_with_code(git, action, lock, 0)
    }

    fn output_with_code(
        &self,
        git: &Git,
        action: &str,
        lock: Option<SiteLock>,
        exit_code: u8,
    ) -> ToolOutput {
        let lock = lock.map(|lock| {
            json!({
                "path": lock.path(),
                "filesystem": lock.filesystem(),
            })
        });
        ToolOutput::json(
            json!({
                "schema": "hctl2.worktree.v1",
                "evidence_level": "toolbox_readback",
                "outcome": if self.valid { "established" } else { "not_established" },
                "observed_at_unix_ms": observed_at_unix_ms(),
                "operation": action,
                "git": {
                    "path": git.executable(),
                    "version": git.version(),
                },
                "change_set_ref": self.change_set_ref,
                "baseline_commit_sha": self.baseline,
                "site_lock": lock,
                "worktree": self.worktree.as_ref().map(WorktreeEntry::to_json),
                "verification": {
                    "path_exists": self.path_exists,
                    "branch_matches": self.branch_matches,
                    "branch_head_matches": self.branch_head_matches,
                    "baseline_is_ancestor": self.baseline_is_ancestor,
                    "tracked_changes": self.tracked_changes,
                    "untracked_changes": self.untracked_changes,
                    "clean": self.tracked_changes == 0 && self.untracked_changes == 0,
                },
            }),
            exit_code,
        )
    }

    fn failure_summary(&self) -> String {
        format!(
            "path_exists={}, branch_matches={}, branch_head_matches={}, baseline_is_ancestor={}",
            self.path_exists,
            self.branch_matches,
            self.branch_head_matches,
            self.baseline_is_ancestor
        )
    }
}

fn verify_state(
    git: &Git,
    repository: &Repository,
    change_set_ref: &str,
) -> Result<Verification, ToolError> {
    let branch = branch_ref(change_set_ref);
    let baseline = read_optional_commit(git, &repository.anchor, &baseline_ref(change_set_ref))?;
    let worktree = repository
        .worktrees(git)?
        .into_iter()
        .find(|entry| entry.branch.as_deref() == Some(branch.as_str()));
    let Some(entry) = worktree else {
        return Ok(Verification {
            change_set_ref: change_set_ref.to_owned(),
            baseline,
            worktree: None,
            path_exists: false,
            branch_matches: false,
            branch_head_matches: false,
            baseline_is_ancestor: false,
            tracked_changes: 0,
            untracked_changes: 0,
            valid: false,
        });
    };
    let path_exists = entry.path.is_dir();
    if !path_exists {
        return Ok(Verification {
            change_set_ref: change_set_ref.to_owned(),
            baseline,
            worktree: Some(entry),
            path_exists,
            branch_matches: false,
            branch_head_matches: false,
            baseline_is_ancestor: false,
            tracked_changes: 0,
            untracked_changes: 0,
            valid: false,
        });
    }

    let symbolic = git.invoke(&entry.path, &args(&["symbolic-ref", "-q", "HEAD"]))?;
    let branch_matches = symbolic.success() && symbolic.stdout_text()? == branch;
    let head = resolve_commit(git, &entry.path, "HEAD", "HCTL2_TOOL_HEAD_MISSING")?;
    let branch_head = read_optional_commit(git, &repository.anchor, &branch)?;
    let branch_head_matches = branch_head.as_deref() == Some(head.as_str());
    let baseline_is_ancestor = if let Some(baseline) = &baseline {
        let output = git.invoke(
            &repository.anchor,
            &[
                OsString::from("merge-base"),
                OsString::from("--is-ancestor"),
                OsString::from(baseline),
                OsString::from(&head),
            ],
        )?;
        match output.code() {
            Some(0) => true,
            Some(1) => false,
            _ => {
                return Err(ToolError::new(
                    "HCTL2_TOOL_GIT_INSPECTION_FAILED",
                    format!("could not compare baseline and HEAD: {}", output.stderr()),
                ));
            }
        }
    } else {
        false
    };
    let (tracked_changes, untracked_changes) = status_counts(git, &entry.path)?;
    let valid = path_exists
        && branch_matches
        && branch_head_matches
        && baseline_is_ancestor
        && baseline.is_some();
    Ok(Verification {
        change_set_ref: change_set_ref.to_owned(),
        baseline,
        worktree: Some(entry),
        path_exists,
        branch_matches,
        branch_head_matches,
        baseline_is_ancestor,
        tracked_changes,
        untracked_changes,
        valid,
    })
}

fn prepare_root(root: &Path, common_dir: &Path) -> Result<PathBuf, ToolError> {
    fs::create_dir_all(root).map_err(|error| {
        ToolError::new(
            "HCTL2_TOOL_WORKTREE_ROOT_UNWRITABLE",
            format!("cannot create worktree root {}: {error}", root.display()),
        )
    })?;
    let metadata = fs::metadata(root).map_err(|error| {
        ToolError::new(
            "HCTL2_TOOL_WORKTREE_ROOT_UNWRITABLE",
            format!("cannot inspect worktree root {}: {error}", root.display()),
        )
    })?;
    if !metadata.is_dir() || metadata.permissions().readonly() {
        return Err(ToolError::new(
            "HCTL2_TOOL_WORKTREE_ROOT_UNWRITABLE",
            format!(
                "worktree root is not a writable directory: {}",
                root.display()
            ),
        ));
    }
    let root = root.canonicalize().map_err(|error| {
        ToolError::new(
            "HCTL2_TOOL_WORKTREE_ROOT_UNWRITABLE",
            format!(
                "cannot canonicalize worktree root {}: {error}",
                root.display()
            ),
        )
    })?;
    let internal = common_dir.join("hctl2").canonicalize().map_err(|error| {
        ToolError::new(
            "HCTL2_TOOL_SITE_LOCK_UNAVAILABLE",
            format!("cannot canonicalize internal state directory: {error}"),
        )
    })?;
    if root.starts_with(&internal) {
        return Err(ToolError::new(
            "HCTL2_TOOL_WORKTREE_ROOT_INTERNAL",
            format!(
                "worktree checkout cannot live under disposable state directory {}",
                internal.display()
            ),
        ));
    }
    Ok(root)
}

fn recover_missing_baseline_marker(
    git: &Git,
    repository: &Repository,
    entry: &WorktreeEntry,
    target: &Path,
    baseline_ref: &str,
    baseline: &str,
    recorded_baseline: Option<&str>,
) -> Result<(), ToolError> {
    if recorded_baseline.is_some() {
        return Ok(());
    }
    let actual_path = entry.path.canonicalize().map_err(|error| {
        ToolError::new(
            "HCTL2_TOOL_WORKTREE_STATE_INVALID",
            format!(
                "cannot access interrupted worktree {}: {error}",
                entry.path.display()
            ),
        )
    })?;
    let target_path = target.canonicalize().map_err(|error| {
        ToolError::new(
            "HCTL2_TOOL_WORKTREE_STATE_INVALID",
            format!(
                "cannot access expected worktree {}: {error}",
                target.display()
            ),
        )
    })?;
    let head = resolve_commit(git, &actual_path, "HEAD", "HCTL2_TOOL_HEAD_MISSING")?;
    if actual_path != target_path || head != baseline {
        return Err(ToolError::new(
            "HCTL2_TOOL_WORKTREE_STATE_INVALID",
            "reserved ChangeSet branch exists without its baseline marker",
        ));
    }
    create_baseline_marker(git, repository, baseline_ref, baseline)
}

fn create_baseline_marker(
    git: &Git,
    repository: &Repository,
    baseline_ref: &str,
    baseline: &str,
) -> Result<(), ToolError> {
    let zero = "0".repeat(baseline.len());
    git.checked(
        &repository.anchor,
        &[
            OsString::from("update-ref"),
            OsString::from(baseline_ref),
            OsString::from(baseline),
            OsString::from(zero),
        ],
        "HCTL2_TOOL_WORKTREE_STATE_INVALID",
        "record ChangeSet baseline",
    )?;
    Ok(())
}

fn read_optional_commit(
    git: &Git,
    repository: &Path,
    reference: &str,
) -> Result<Option<String>, ToolError> {
    let expression = format!("{reference}^{{commit}}");
    let output = git.invoke(
        repository,
        &[
            OsString::from("rev-parse"),
            OsString::from("--verify"),
            OsString::from("--quiet"),
            OsString::from("--end-of-options"),
            OsString::from(expression),
        ],
    )?;
    if output.success() {
        Ok(Some(output.stdout_text()?))
    } else if output.code() == Some(1) {
        Ok(None)
    } else {
        Err(ToolError::new(
            "HCTL2_TOOL_GIT_INSPECTION_FAILED",
            format!("could not read ref {reference}: {}", output.stderr()),
        ))
    }
}

fn status_counts(git: &Git, worktree: &Path) -> Result<(usize, usize), ToolError> {
    let output = git.checked(
        worktree,
        &args(&["status", "--porcelain=v2", "-z", "--untracked-files=all"]),
        "HCTL2_TOOL_GIT_INSPECTION_FAILED",
        "read worktree status",
    )?;
    let mut tracked = 0;
    let mut untracked = 0;
    let mut fields = output.stdout().split(|byte| *byte == 0);
    while let Some(field) = fields.next() {
        if field.starts_with(b"1 ") || field.starts_with(b"u ") {
            tracked += 1;
        } else if field.starts_with(b"2 ") {
            tracked += 1;
            let _ = fields.next();
        } else if field.starts_with(b"? ") {
            untracked += 1;
        }
    }
    Ok((tracked, untracked))
}

fn validate_change_set_ref(value: &str) -> Result<(), ToolError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        Err(ToolError::new(
            "HCTL2_TOOL_CHANGE_SET_REF_INVALID",
            "ChangeSet ref must be 1-128 ASCII letters, digits, '-' or '_'",
        ))
    } else {
        Ok(())
    }
}

fn short_branch(change_set_ref: &str) -> String {
    format!("{BRANCH_PREFIX}{change_set_ref}")
}

fn branch_ref(change_set_ref: &str) -> String {
    format!("refs/heads/{}", short_branch(change_set_ref))
}

fn baseline_ref(change_set_ref: &str) -> String {
    format!("{BASELINE_REF_PREFIX}{change_set_ref}/baseline")
}

#[cfg(test)]
mod tests {
    use super::validate_change_set_ref;

    #[test]
    fn change_set_refs_are_safe_in_paths_and_git_refs() {
        for valid in ["CS-123", "project_7", "a"] {
            assert!(validate_change_set_ref(valid).is_ok());
        }
        for invalid in ["", "../escape", "has/slash", "two words", "."] {
            assert!(validate_change_set_ref(invalid).is_err());
        }
    }
}
