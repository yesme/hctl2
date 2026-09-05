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
    git: &Git,
    repository_path: PathBuf,
    root: PathBuf,
    change_set_ref: String,
    baseline: String,
) -> Result<ToolOutput, ToolError> {
    let repository = Repository::open(git, &repository_path)?;
    let site_lock = SiteLock::acquire(
        &repository.common_dir,
        "worktree_materialize",
        Some(&change_set_ref),
    )?;
    let baseline = resolve_commit(
        git,
        &repository.anchor,
        &baseline,
        "HCTL2_TOOL_BASELINE_NOT_FOUND",
    )?;
    let marker = baseline_ref(&change_set_ref);
    let recorded_baseline = read_optional_commit(git, &repository.anchor, &marker)?;
    if let Some(recorded) = &recorded_baseline
        && recorded != &baseline
    {
        return Err(ToolError::not_established(
            "HCTL2_TOOL_BASELINE_MISMATCH",
            "ChangeSet has a different recorded baseline",
        )
        .with_details(json!({
            "change_set_ref": change_set_ref, "recorded_baseline": recorded,
            "requested_baseline": baseline,
        })));
    }

    let worktrees = repository.worktrees(git)?;
    let branch = branch_ref(&change_set_ref);
    let existing = worktrees
        .iter()
        .any(|entry| entry.branch.as_deref() == Some(branch.as_str()));
    if !existing {
        let root = prepare_root(&root, &repository, &worktrees)?;
        let target = root.join(&change_set_ref);
        check_target(&target)?;
        let existing_branch = read_optional_commit(git, &repository.anchor, &branch)?;
        if existing_branch
            .as_ref()
            .is_some_and(|commit| commit != &baseline)
        {
            return Err(ToolError::not_established(
                "HCTL2_TOOL_WORKTREE_STATE_INVALID",
                "reserved branch is not at the requested baseline",
            )
            .with_details(
                json!({ "branch": branch, "branch_head": existing_branch, "baseline": baseline }),
            ));
        }

        let mut arguments = args(&["worktree", "add"]);
        if existing_branch.is_none() {
            arguments.extend(args(&["-b", &short_branch(&change_set_ref)]));
        }
        arguments.push(target.clone().into_os_string());
        // Git attaches HEAD only for the short branch name, not refs/heads/… .
        arguments.push(OsString::from(if existing_branch.is_some() {
            short_branch(&change_set_ref)
        } else {
            baseline.clone()
        }));
        git.checked(
            &repository.anchor,
            &arguments,
            "HCTL2_TOOL_WORKTREE_MATERIALIZE_FAILED",
            "materialize worktree",
        )
        .map_err(|error| {
            error.with_details(json!({
                "occupied_path": target, "baseline_marker": marker,
                "recovery_action": "inspect_worktree_then_retry_same_changeset",
            }))
        })?;
    }

    // The marker is a disposable cache. Validate the caller's baseline against
    // the actual checkout before creating/rebuilding it, including after commits.
    let verification = verify_state(git, &repository, &change_set_ref, Some(baseline.clone()))?;
    if !verification.valid() {
        return Err(ToolError::not_established(
            "HCTL2_TOOL_WORKTREE_VERIFICATION_FAILED", "ChangeSet worktree failed readback",
        ).with_details(json!({
            "change_set_ref": change_set_ref, "worktree": verification.worktree.as_ref().map(WorktreeEntry::to_json),
            "verification": verification.facts(), "baseline_marker": marker,
            "marker_previously_present": recorded_baseline.is_some(),
            "recovery_action": "inspect_worktree_then_retry_same_changeset",
        })));
    }
    if recorded_baseline.is_none() {
        create_baseline_marker(git, &repository, &marker, &baseline)?;
    }
    // Read the stored marker as well; successful output is never an input echo.
    let verification = verify_state(
        git,
        &repository,
        &change_set_ref,
        read_optional_commit(git, &repository.anchor, &marker)?,
    )?;
    Ok(verification.output(
        git,
        if existing {
            "already_materialized"
        } else {
            "created"
        },
        Some(&site_lock),
    ))
}

pub(crate) fn verify(
    git: &Git,
    repository_path: PathBuf,
    change_set_ref: String,
) -> Result<ToolOutput, ToolError> {
    let repository = Repository::open(git, &repository_path)?;
    let baseline = read_optional_commit(git, &repository.anchor, &baseline_ref(&change_set_ref))?;
    Ok(verify_state(git, &repository, &change_set_ref, baseline)?.output(git, "verified", None))
}

#[derive(Debug, Default)]
struct Verification {
    change_set_ref: String,
    baseline: Option<String>,
    worktree: Option<WorktreeEntry>,
    path_exists: bool,
    repository_matches: bool,
    branch_matches: bool,
    branch_head_matches: bool,
    baseline_is_ancestor: bool,
    status: Option<(usize, usize)>,
}

impl Verification {
    fn valid(&self) -> bool {
        self.path_exists
            && self.repository_matches
            && self.branch_matches
            && self.branch_head_matches
            && self.baseline_is_ancestor
    }

    fn facts(&self) -> serde_json::Value {
        json!({
            "path_exists": self.path_exists,
            "repository_matches": self.repository_matches,
            "branch_matches": self.branch_matches,
            "branch_head_matches": self.branch_head_matches,
            "baseline_is_ancestor": self.baseline_is_ancestor,
            "tracked_changes": self.status.map(|(tracked, _)| tracked),
            "untracked_changes": self.status.map(|(_, untracked)| untracked),
            "clean": self.status.map(|status| status == (0, 0)),
        })
    }

    fn output(&self, git: &Git, operation: &str, lock: Option<&SiteLock>) -> ToolOutput {
        ToolOutput::json(
            json!({
                "schema": "hctl2.worktree.v1",
                "evidence_level": "toolbox_readback",
                "outcome": if self.valid() { "established" } else { "not_established" },
                "observed_at_unix_ms": observed_at_unix_ms(),
                "operation": operation,
                "git": { "path": git.executable(), "version": git.version() },
                "change_set_ref": self.change_set_ref,
                "baseline_commit_sha": self.baseline,
                "site_lock": lock.map(|lock| json!({ "path": lock.path(), "filesystem": lock.filesystem() })),
                "worktree": self.worktree.as_ref().map(WorktreeEntry::to_json),
                "verification": self.facts(),
                "error": if self.valid() { serde_json::Value::Null } else { json!({
                    "code": "HCTL2_TOOL_WORKTREE_STATE_INVALID",
                    "recovery_action": "inspect_worktree_then_retry_same_changeset",
                }) },
            }),
            if self.valid() { 0 } else { 3 },
        )
    }
}

fn verify_state(
    git: &Git,
    repository: &Repository,
    change_set_ref: &str,
    baseline: Option<String>,
) -> Result<Verification, ToolError> {
    let branch = branch_ref(change_set_ref);
    let mut result = Verification {
        change_set_ref: change_set_ref.to_owned(),
        baseline,
        worktree: repository
            .worktrees(git)?
            .into_iter()
            .find(|entry| entry.branch.as_deref() == Some(branch.as_str())),
        ..Verification::default()
    };
    let Some(entry) = &mut result.worktree else {
        return Ok(result);
    };
    result.path_exists = entry.path.is_dir();
    if !result.path_exists {
        return Ok(result);
    }

    // A registered path can be replaced with another checkout or lose its .git
    // file. Git discovery alone could then silently inspect a parent repository.
    let actual = Repository::open(git, &entry.path)?;
    result.repository_matches = actual.common_dir == repository.common_dir
        && entry.path.canonicalize().ok().as_ref() == Some(&actual.top_level);
    if !result.repository_matches {
        return Ok(result);
    }
    let symbolic = git.invoke(&entry.path, &args(&["symbolic-ref", "-q", "HEAD"]))?;
    result.branch_matches = symbolic.success() && symbolic.stdout_text()? == branch;
    let head = resolve_commit(git, &entry.path, "HEAD", "HCTL2_TOOL_HEAD_MISSING")?;
    result.branch_head_matches =
        read_optional_commit(git, &repository.anchor, &branch)?.as_deref() == Some(head.as_str());
    if let Some(baseline) = &result.baseline {
        result.baseline_is_ancestor = is_ancestor(git, &repository.anchor, baseline, &head)?;
    }
    entry.head = Some(head);
    result.status = Some(status_counts(git, &entry.path)?);
    Ok(result)
}

fn is_ancestor(
    git: &Git,
    repository: &Path,
    baseline: &str,
    head: &str,
) -> Result<bool, ToolError> {
    let output = git.invoke(
        repository,
        &args(&["merge-base", "--is-ancestor", baseline, head]),
    )?;
    match output.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(ToolError::new(
            "HCTL2_TOOL_GIT_INSPECTION_FAILED",
            format!("could not compare baseline and HEAD: {}", output.stderr()),
        )),
    }
}

fn prepare_root(
    root: &Path,
    repository: &Repository,
    worktrees: &[WorktreeEntry],
) -> Result<PathBuf, ToolError> {
    // Validate before mkdir, then again after canonicalization.
    let ancestor = root
        .ancestors()
        .find(|path| path.exists() || path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let ancestor = if ancestor.as_os_str().is_empty() {
        Path::new(".")
    } else {
        ancestor
    };
    let ancestor = ancestor.canonicalize().map_err(root_error)?;
    check_root_location(&ancestor, repository, worktrees)?;
    fs::create_dir_all(root).map_err(root_error)?;
    let root = root.canonicalize().map_err(root_error)?;
    check_root_location(&root, repository, worktrees)?;
    let metadata = fs::metadata(&root).map_err(root_error)?;
    if !metadata.is_dir() || metadata.permissions().readonly() {
        return Err(ToolError::new(
            "HCTL2_TOOL_WORKTREE_ROOT_UNWRITABLE",
            format!(
                "worktree root is not a writable directory: {}",
                root.display()
            ),
        ));
    }
    Ok(root)
}

fn root_error(error: std::io::Error) -> ToolError {
    ToolError::new("HCTL2_TOOL_WORKTREE_ROOT_UNWRITABLE", error.to_string())
}

fn check_root_location(
    root: &Path,
    repository: &Repository,
    worktrees: &[WorktreeEntry],
) -> Result<(), ToolError> {
    let internal = repository
        .common_dir
        .join("hctl2")
        .canonicalize()
        .map_err(root_error)?;
    if root.starts_with(&repository.common_dir) || root.starts_with(internal) {
        return Err(ToolError::not_established(
            "HCTL2_TOOL_WORKTREE_ROOT_INTERNAL",
            "checkout root is inside Git's internal state directory",
        ));
    }
    for entry in worktrees {
        let path = entry
            .path
            .canonicalize()
            .unwrap_or_else(|_| entry.path.clone());
        if root.starts_with(&path) {
            return Err(ToolError::not_established(
                "HCTL2_TOOL_WORKTREE_ROOT_NESTED",
                "checkout root is inside an existing worktree",
            )
            .with_details(json!({ "root": root, "containing_worktree": path })));
        }
    }
    Ok(())
}

fn check_target(target: &Path) -> Result<(), ToolError> {
    match fs::symlink_metadata(target) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(root_error(error)),
        Ok(metadata) => {
            // Git natively accepts an empty directory. Keep it intact and let
            // worktree add use it; files, symlinks and nonempty directories stay.
            if metadata.is_dir() && fs::read_dir(target).map_err(root_error)?.next().is_none() {
                return Ok(());
            }
            Err(ToolError::not_established(
                "HCTL2_TOOL_WORKTREE_PATH_OCCUPIED",
                "materialization path is occupied; preserve its contents before retrying",
            )
            .with_details(json!({ "occupied_path": target,
                    "recovery_action": "preserve_occupied_path_then_retry" })))
        }
    }
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

pub(crate) fn validate_change_set_ref(value: &str) -> Result<(), ToolError> {
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
