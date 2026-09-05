//! Snapshot and salvage-remove isolated ChangeSet worktrees.
//!
//! Salvage proof is point-in-time: the site lock serializes toolbox commands,
//! not Harness writes. P1 callers must stop the writer; P2 lease revocation
//! does that. Snapshots record worktree content, not the Harness index staging
//! state.

use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::git::{Git, args};
use crate::repository::{Repository, WorktreeEntry, resolve_commit};
use crate::site_lock::SiteLock;
use crate::worktree::{baseline_ref, branch_ref, read_optional_commit};
use crate::{ToolError, ToolOutput, observed_at_unix_ms};

const IGNORED_RESIDUE_LIMIT: usize = 64;
const SNAPSHOT_MESSAGE: &str = "hctl2 archive snapshot";
const CAPTURES_WORKTREE_CONTENT: &str = "worktree_content";

pub(crate) fn snapshot(
    git: &Git,
    repository_path: PathBuf,
    change_set_ref: String,
) -> Result<ToolOutput, ToolError> {
    let repository = Repository::open(git, &repository_path)?;
    let site_lock = acquire_lock(&repository, "archive_snapshot", &change_set_ref)?;
    let worktree = require_worktree(git, &repository, &change_set_ref)?;
    let record = write_snapshot(git, &repository, &worktree, &change_set_ref)?;
    Ok(snapshot_output(
        git,
        &change_set_ref,
        &record,
        Some(&site_lock),
        "snapshotted",
    ))
}

pub(crate) fn remove(
    git: &Git,
    repository_path: PathBuf,
    change_set_ref: String,
    discard_unarchived: bool,
    confirm_discard: Option<String>,
    reject_ignored: bool,
) -> Result<ToolOutput, ToolError> {
    if discard_unarchived {
        if confirm_discard.is_none() {
            return Err(ToolError::new(
                "HCTL2_TOOL_INVALID_ARGUMENT",
                "--discard-unarchived requires --confirm-discard",
            ));
        }
    } else if confirm_discard.is_some() {
        return Err(ToolError::new(
            "HCTL2_TOOL_INVALID_ARGUMENT",
            "--confirm-discard is only valid with --discard-unarchived",
        ));
    }

    let repository = Repository::open(git, &repository_path)?;
    let site_lock = acquire_lock(&repository, "archive_remove", &change_set_ref)?;
    let worktree = require_worktree(git, &repository, &change_set_ref)?;
    if discard_unarchived {
        discard_worktree(
            git,
            &repository,
            &worktree,
            &change_set_ref,
            confirm_discard
                .as_deref()
                .expect("clap requires --confirm-discard with --discard-unarchived"),
            &site_lock,
        )
    } else {
        salvage_worktree(
            git,
            &repository,
            &worktree,
            &change_set_ref,
            reject_ignored,
            &site_lock,
        )
    }
}

struct SnapshotRecord {
    head_commit_sha: String,
    baseline_commit_sha: Option<String>,
    result_tree_sha: String,
    snapshot_commit_sha: String,
    snapshot_ref: String,
    reused: bool,
}

impl SnapshotRecord {
    fn to_json(&self) -> Value {
        json!({
            "head_commit_sha": self.head_commit_sha,
            "baseline_commit_sha": self.baseline_commit_sha,
            "result_tree_sha": self.result_tree_sha,
            "snapshot_commit_sha": self.snapshot_commit_sha,
            "snapshot_ref": self.snapshot_ref,
            "reused": self.reused,
            "captures": CAPTURES_WORKTREE_CONTENT,
        })
    }
}

struct IgnoredResidue {
    files: Vec<Value>,
    total: usize,
    truncated: bool,
}

struct Gitlink {
    path: String,
    commit: String,
    gitdir: Option<String>,
    in_gitmodules: bool,
}

impl Gitlink {
    fn to_json(&self) -> Value {
        json!({
            "path": self.path,
            "commit": self.commit,
            "gitdir": self.gitdir,
            "in_gitmodules": self.in_gitmodules,
        })
    }
}

struct StagedTree {
    tree_sha: String,
    gitlinks: Vec<Gitlink>,
}

struct TemporaryIndex {
    path: PathBuf,
}

impl TemporaryIndex {
    fn create(common_dir: &Path, kind: &str, change_set_ref: &str) -> Result<Self, ToolError> {
        let directory = common_dir.join("hctl2");
        fs::create_dir_all(&directory).map_err(|error| {
            ToolError::new(
                "HCTL2_TOOL_ARCHIVE_FAILED",
                format!("cannot create archive index directory: {error}"),
            )
        })?;
        let path = directory.join(format!("{kind}-{change_set_ref}.index"));
        let _ = fs::remove_file(&path);
        Ok(Self { path })
    }

    fn env(&self) -> [(&'static str, OsString); 1] {
        [("GIT_INDEX_FILE", self.path.clone().into_os_string())]
    }
}

impl Drop for TemporaryIndex {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn snapshot_output(
    git: &Git,
    change_set_ref: &str,
    record: &SnapshotRecord,
    lock: Option<&SiteLock>,
    operation: &str,
) -> ToolOutput {
    ToolOutput::json(
        json!({
            "schema": "hctl2.archive.v1",
            "evidence_level": "toolbox_readback",
            "outcome": "established",
            "observed_at_unix_ms": observed_at_unix_ms(),
            "operation": operation,
            "git": { "path": git.executable(), "version": git.version() },
            "change_set_ref": change_set_ref,
            "site_lock": lock.map(|lock| json!({
                "path": lock.path(),
                "filesystem": lock.filesystem(),
            })),
            "head_commit_sha": record.head_commit_sha,
            "baseline_commit_sha": record.baseline_commit_sha,
            "result_tree_sha": record.result_tree_sha,
            "snapshot_commit_sha": record.snapshot_commit_sha,
            "snapshot_ref": record.snapshot_ref,
            "reused": record.reused,
            "captures": CAPTURES_WORKTREE_CONTENT,
            "error": Value::Null,
        }),
        0,
    )
}

fn acquire_lock(
    repository: &Repository,
    operation: &str,
    change_set_ref: &str,
) -> Result<SiteLock, ToolError> {
    SiteLock::acquire(&repository.common_dir, operation, Some(change_set_ref)).map_err(|error| {
        if error.code() == "HCTL2_TOOL_SITE_BUSY" {
            error.with_recovery_action("retry_when_site_lock_free")
        } else {
            error
        }
    })
}

fn salvage_worktree(
    git: &Git,
    repository: &Repository,
    worktree: &WorktreeEntry,
    change_set_ref: &str,
    reject_ignored: bool,
    site_lock: &SiteLock,
) -> Result<ToolOutput, ToolError> {
    let snapshot = write_snapshot(git, repository, worktree, change_set_ref)?;
    let staged = prove_salvage(git, repository, worktree, change_set_ref, &snapshot)?;
    if !staged.gitlinks.is_empty() {
        return Err(nested_repository_error(
            git,
            worktree,
            &staged.gitlinks,
            Some(&snapshot),
        ));
    }
    let ignored = ignored_residue(git, &worktree.path)?;
    if reject_ignored && !ignored.files.is_empty() {
        return Err(ignored_residue_error(&ignored, &snapshot, git, worktree));
    }
    let summary = status_summary(git, &worktree.path);
    let path = worktree.path.clone();
    remove_worktree(git, repository, &path, change_set_ref)?;
    Ok(remove_output(
        git,
        change_set_ref,
        site_lock,
        &path,
        "removed_salvaged",
        Some(&snapshot),
        None,
        &[],
        &ignored,
        &summary,
    ))
}

fn discard_worktree(
    git: &Git,
    repository: &Repository,
    worktree: &WorktreeEntry,
    change_set_ref: &str,
    confirm_discard: &str,
    site_lock: &SiteLock,
) -> Result<ToolOutput, ToolError> {
    let index = TemporaryIndex::create(&repository.common_dir, "discard", change_set_ref)?;
    let staged = write_worktree_tree(git, &worktree.path, &index, "HCTL2_TOOL_ARCHIVE_FAILED")
        .map_err(|error| error.with_recovery_action("inspect_worktree_then_retry_archive"))?;
    drop(index);
    if confirm_discard != staged.tree_sha {
        return Err(ToolError::not_established(
            "HCTL2_TOOL_DISCARD_CONFIRMATION_MISMATCH",
            "discard confirmation must equal the current worktree tree sha",
        )
        .with_recovery_action("pass_matching_confirm_discard")
        .with_details(json!({
            "change_set_ref": change_set_ref,
            "confirm_discard": confirm_discard,
            "current_tree_sha": staged.tree_sha,
            "nested_repositories": staged
                .gitlinks
                .iter()
                .map(Gitlink::to_json)
                .collect::<Vec<_>>(),
            "status_summary": status_summary(git, &worktree.path),
        })));
    }
    let ignored = ignored_residue(git, &worktree.path)?;
    let summary = status_summary(git, &worktree.path);
    let path = worktree.path.clone();
    remove_worktree(git, repository, &path, change_set_ref)?;
    Ok(remove_output(
        git,
        change_set_ref,
        site_lock,
        &path,
        "removed_discarded",
        None,
        Some(staged.tree_sha.as_str()),
        &staged.gitlinks,
        &ignored,
        &summary,
    ))
}

#[allow(clippy::too_many_arguments)]
fn remove_output(
    git: &Git,
    change_set_ref: &str,
    site_lock: &SiteLock,
    path: &Path,
    operation: &str,
    snapshot: Option<&SnapshotRecord>,
    current_tree_sha: Option<&str>,
    gitlinks: &[Gitlink],
    ignored: &IgnoredResidue,
    summary: &str,
) -> ToolOutput {
    ToolOutput::json(
        json!({
            "schema": "hctl2.archive.v1",
            "evidence_level": "toolbox_readback",
            "outcome": "established",
            "observed_at_unix_ms": observed_at_unix_ms(),
            "operation": operation,
            "git": { "path": git.executable(), "version": git.version() },
            "change_set_ref": change_set_ref,
            "site_lock": { "path": site_lock.path(), "filesystem": site_lock.filesystem() },
            "worktree_path": path,
            "snapshot": snapshot.map(SnapshotRecord::to_json),
            "current_tree_sha": current_tree_sha,
            "nested_repositories": gitlinks.iter().map(Gitlink::to_json).collect::<Vec<_>>(),
            "status_summary": summary,
            "ignored_residue": ignored.files,
            "ignored_residue_total": ignored.total,
            "ignored_residue_truncated": ignored.truncated,
            "error": Value::Null,
        }),
        0,
    )
}

fn remove_worktree(
    git: &Git,
    repository: &Repository,
    path: &Path,
    change_set_ref: &str,
) -> Result<(), ToolError> {
    let mut arguments = args(&["worktree", "remove", "--force", "--"]);
    arguments.push(path.to_path_buf().into_os_string());
    git.checked(
        &repository.anchor,
        &arguments,
        "HCTL2_TOOL_WORKTREE_REMOVE_FAILED",
        "remove worktree",
    )
    .map_err(|error| {
        error
            .with_recovery_action("inspect_worktree_then_retry_remove")
            .with_details(json!({
                "worktree": path,
                "change_set_ref": change_set_ref,
            }))
    })?;
    Ok(())
}

fn require_worktree(
    git: &Git,
    repository: &Repository,
    change_set_ref: &str,
) -> Result<WorktreeEntry, ToolError> {
    let branch = branch_ref(change_set_ref);
    let mut worktrees = repository.worktrees(git)?;
    let matched = worktrees
        .iter()
        .position(|entry| entry.branch.as_deref() == Some(branch.as_str()))
        .or_else(|| {
            // Materialize places the checkout at <root>/<change_set_ref>. After the
            // Harness detaches HEAD, porcelain no longer carries the branch name.
            worktrees.iter().position(|entry| {
                entry.path.file_name().and_then(|name| name.to_str()) == Some(change_set_ref)
            })
        });
    let Some(entry) = matched.map(|index| worktrees.swap_remove(index)) else {
        return Err(ToolError::not_established(
            "HCTL2_TOOL_WORKTREE_NOT_FOUND",
            "no materialized worktree is attached to this ChangeSet",
        )
        .with_recovery_action("materialize_worktree_then_retry")
        .with_details(json!({ "change_set_ref": change_set_ref, "branch": branch })));
    };
    if !entry.path.is_dir() {
        return Err(ToolError::not_established(
            "HCTL2_TOOL_WORKTREE_STATE_INVALID",
            "registered worktree path is missing or incomplete",
        )
        .with_recovery_action("inspect_worktree_then_retry_same_changeset")
        .with_details(json!({ "change_set_ref": change_set_ref, "worktree": entry.path })));
    }
    let actual = Repository::open(git, &entry.path).map_err(|error| {
        error
            .with_recovery_action("inspect_worktree_then_retry_same_changeset")
            .with_details(json!({ "change_set_ref": change_set_ref, "worktree": entry.path }))
    })?;
    if actual.common_dir != repository.common_dir {
        return Err(ToolError::not_established(
            "HCTL2_TOOL_WORKTREE_STATE_INVALID",
            "worktree path no longer belongs to the source repository",
        )
        .with_recovery_action("inspect_worktree_then_retry_same_changeset")
        .with_details(json!({
            "change_set_ref": change_set_ref,
            "worktree": entry.path,
            "expected_common_dir": repository.common_dir,
            "actual_common_dir": actual.common_dir,
        })));
    }
    let symbolic = git.invoke(&entry.path, &args(&["symbolic-ref", "-q", "HEAD"]))?;
    if !symbolic.success() || symbolic.stdout_text()? != branch {
        return Err(ToolError::not_established(
            "HCTL2_TOOL_WORKTREE_BRANCH_MOVED",
            "ChangeSet branch is no longer checked out in the worktree",
        )
        .with_recovery_action("restore_changeset_branch_then_retry")
        .with_details(json!({
            "change_set_ref": change_set_ref,
            "expected_branch": branch,
            "worktree": entry.path,
        })));
    }
    Ok(entry)
}

fn write_snapshot(
    git: &Git,
    repository: &Repository,
    worktree: &WorktreeEntry,
    change_set_ref: &str,
) -> Result<SnapshotRecord, ToolError> {
    let head_commit_sha = resolve_commit(git, &worktree.path, "HEAD", "HCTL2_TOOL_HEAD_MISSING")?;
    let baseline_commit_sha =
        read_optional_commit(git, &repository.anchor, &baseline_ref(change_set_ref))?;
    let index = TemporaryIndex::create(&repository.common_dir, "archive", change_set_ref)?;
    let staged = write_worktree_tree(git, &worktree.path, &index, "HCTL2_TOOL_ARCHIVE_FAILED")
        .map_err(|error| error.with_recovery_action("inspect_worktree_then_retry_archive"))?;
    drop(index);
    let result_tree_sha = staged.tree_sha;
    let snapshot_ref = snapshot_ref(change_set_ref, &result_tree_sha);

    if let Some(existing) = read_optional_commit(git, &repository.anchor, &snapshot_ref)? {
        let existing_tree = commit_peel(git, &repository.anchor, &existing, "tree")?;
        if existing_tree != result_tree_sha {
            return Err(ToolError::new(
                "HCTL2_TOOL_ARCHIVE_FAILED",
                "snapshot ref does not point at the tree it is named for",
            )
            .with_recovery_action("inspect_worktree_then_retry_archive")
            .with_details(json!({
                "snapshot_ref": snapshot_ref,
                "expected_tree_sha": result_tree_sha,
                "actual_tree_sha": existing_tree,
            })));
        }
        let existing_parent = commit_parent(git, &repository.anchor, &existing)?;
        if existing_parent.as_deref() == Some(head_commit_sha.as_str()) {
            return Ok(SnapshotRecord {
                head_commit_sha,
                baseline_commit_sha,
                result_tree_sha,
                snapshot_commit_sha: existing,
                snapshot_ref,
                reused: true,
            });
        }
        let snapshot_commit_sha =
            create_snapshot_commit(git, repository, &result_tree_sha, &head_commit_sha)?;
        git.checked(
            &repository.anchor,
            &[
                OsString::from("update-ref"),
                OsString::from(&snapshot_ref),
                OsString::from(&snapshot_commit_sha),
                OsString::from(&existing),
            ],
            "HCTL2_TOOL_ARCHIVE_FAILED",
            "replace archive snapshot ref for a new parent",
        )?;
        return Ok(SnapshotRecord {
            head_commit_sha,
            baseline_commit_sha,
            result_tree_sha,
            snapshot_commit_sha,
            snapshot_ref,
            reused: false,
        });
    }

    let snapshot_commit_sha =
        create_snapshot_commit(git, repository, &result_tree_sha, &head_commit_sha)?;
    let zero = "0".repeat(snapshot_commit_sha.len());
    let publish = git.invoke(
        &repository.anchor,
        &[
            OsString::from("update-ref"),
            OsString::from(&snapshot_ref),
            OsString::from(&snapshot_commit_sha),
            OsString::from(zero),
        ],
    )?;
    if publish.success() {
        return Ok(SnapshotRecord {
            head_commit_sha,
            baseline_commit_sha,
            result_tree_sha,
            snapshot_commit_sha,
            snapshot_ref,
            reused: false,
        });
    }
    if let Some(existing) = read_optional_commit(git, &repository.anchor, &snapshot_ref)? {
        let existing_tree = commit_peel(git, &repository.anchor, &existing, "tree")?;
        let existing_parent = commit_parent(git, &repository.anchor, &existing)?;
        if existing_tree == result_tree_sha
            && existing_parent.as_deref() == Some(head_commit_sha.as_str())
        {
            return Ok(SnapshotRecord {
                head_commit_sha,
                baseline_commit_sha,
                result_tree_sha,
                snapshot_commit_sha: existing,
                snapshot_ref,
                reused: true,
            });
        }
    }
    Err(ToolError::new(
        "HCTL2_TOOL_ARCHIVE_FAILED",
        format!("publish archive snapshot ref failed: {}", publish.stderr()),
    )
    .with_recovery_action("inspect_worktree_then_retry_archive"))
}

fn prove_salvage(
    git: &Git,
    repository: &Repository,
    worktree: &WorktreeEntry,
    change_set_ref: &str,
    snapshot: &SnapshotRecord,
) -> Result<StagedTree, ToolError> {
    let index = TemporaryIndex::create(&repository.common_dir, "salvage", change_set_ref)?;
    let staged = write_worktree_tree(git, &worktree.path, &index, "HCTL2_TOOL_SALVAGE_UNPROVEN")
        .map_err(|error| {
            salvage_unproven(
                error.message(),
                snapshot,
                &[],
                &status_summary(git, &worktree.path),
            )
        })?;
    drop(index);
    if staged.tree_sha != snapshot.result_tree_sha {
        return Err(salvage_unproven(
            "worktree contents changed after the salvage snapshot",
            snapshot,
            &differing_paths(
                git,
                &repository.anchor,
                &snapshot.result_tree_sha,
                &staged.tree_sha,
            ),
            &status_summary(git, &worktree.path),
        ));
    }
    git.checked(
        &repository.anchor,
        &args(&[
            "rev-parse",
            "--verify",
            "--quiet",
            "--end-of-options",
            &format!("{}^{{commit}}", snapshot.snapshot_commit_sha),
        ]),
        "HCTL2_TOOL_SALVAGE_UNPROVEN",
        "confirm snapshot commit is a reachable Git object",
    )
    .map_err(|error| {
        salvage_unproven(
            error.message(),
            snapshot,
            &[],
            &status_summary(git, &worktree.path),
        )
    })?;
    Ok(staged)
}

fn write_worktree_tree(
    git: &Git,
    worktree: &Path,
    index: &TemporaryIndex,
    code: &'static str,
) -> Result<StagedTree, ToolError> {
    let env = index.env();
    git.checked_with_env(
        worktree,
        &args(&["read-tree", "HEAD"]),
        &env,
        code,
        "seed archive index from HEAD",
    )?;
    git.checked_with_env(
        worktree,
        &args(&["add", "--all", "--", "."]),
        &env,
        code,
        "stage worktree contents into a temporary index",
    )?;
    let tree = git
        .checked_with_env(
            worktree,
            &args(&["write-tree"]),
            &env,
            code,
            "write archive tree",
        )?
        .stdout_text()?;
    if !is_object_id(&tree) {
        return Err(ToolError::new(
            "HCTL2_TOOL_GIT_OUTPUT_INVALID",
            format!("write-tree returned an invalid tree: {tree}"),
        ));
    }
    let staged = git.checked_with_env(
        worktree,
        &args(&["ls-files", "--stage", "-z"]),
        &env,
        code,
        "list staged gitlinks",
    )?;
    Ok(StagedTree {
        tree_sha: tree,
        gitlinks: gitlinks_from_stage(git, worktree, staged.stdout())?,
    })
}

fn gitlinks_from_stage(
    git: &Git,
    worktree: &Path,
    stdout: &[u8],
) -> Result<Vec<Gitlink>, ToolError> {
    let submodules = submodule_paths(git, worktree);
    let mut gitlinks = Vec::new();
    for record in stdout.split(|byte| *byte == 0) {
        if record.is_empty() {
            continue;
        }
        let text = String::from_utf8(record.to_vec()).map_err(|_| {
            ToolError::new("HCTL2_TOOL_GIT_OUTPUT_INVALID", "staged path is not UTF-8")
        })?;
        let Some((meta, path)) = text.split_once('\t') else {
            continue;
        };
        let mut fields = meta.split(' ');
        let Some(mode) = fields.next() else { continue };
        let Some(commit) = fields.next() else {
            continue;
        };
        if mode != "160000" {
            continue;
        }
        // A gitlink in HEAD is not a nested repository until the path itself is
        // a Git directory. Uninitialized submodule checkouts are empty and have
        // nothing unique to lose; rev-parse in that directory would report the
        // parent worktree's gitdir and mislead the caller.
        if !nested_repository_on_disk(git, worktree, path) {
            continue;
        }
        let gitdir = git
            .invoke(
                &worktree.join(path),
                &args(&["rev-parse", "--path-format=absolute", "--git-dir"]),
            )
            .ok()
            .and_then(|output| output.success().then_some(output))
            .and_then(|output| output.stdout_text().ok());
        gitlinks.push(Gitlink {
            path: path.to_owned(),
            commit: commit.to_owned(),
            gitdir,
            in_gitmodules: submodules.contains(path),
        });
    }
    Ok(gitlinks)
}

fn nested_repository_on_disk(git: &Git, worktree: &Path, relative: &str) -> bool {
    let nested = worktree.join(relative);
    if nested.join(".git").exists() {
        return true;
    }
    let Ok(absolute) = nested.canonicalize() else {
        return false;
    };
    git.invoke(
        &nested,
        &args(&["rev-parse", "--path-format=absolute", "--show-toplevel"]),
    )
    .ok()
    .filter(crate::git::GitOutput::success)
    .and_then(|output| output.stdout_text().ok())
    .is_some_and(|toplevel| Path::new(&toplevel) == absolute.as_path())
}

fn submodule_paths(git: &Git, worktree: &Path) -> HashSet<String> {
    let output = git.invoke(
        worktree,
        &args(&[
            "config",
            "-f",
            ".gitmodules",
            "--get-regexp",
            r"^submodule\..*\.path$",
        ]),
    );
    let Ok(output) = output else {
        return HashSet::new();
    };
    if !output.success() {
        return HashSet::new();
    }
    let Ok(text) = output.stdout_text() else {
        return HashSet::new();
    };
    text.lines()
        .filter_map(|line| line.split_once(' ').map(|(_, path)| path.to_owned()))
        .collect()
}

fn ignored_residue(git: &Git, worktree: &Path) -> Result<IgnoredResidue, ToolError> {
    let output = git.checked(
        worktree,
        &args(&["ls-files", "-z", "-o", "-i", "--exclude-standard", "--"]),
        "HCTL2_TOOL_GIT_INSPECTION_FAILED",
        "list ignored files",
    )?;
    let mut files = Vec::new();
    let mut total = 0;
    for path in output.stdout().split(|byte| *byte == 0) {
        if path.is_empty() {
            continue;
        }
        total += 1;
        if files.len() == IGNORED_RESIDUE_LIMIT {
            continue;
        }
        let relative = String::from_utf8(path.to_vec()).map_err(|_| {
            ToolError::new("HCTL2_TOOL_GIT_OUTPUT_INVALID", "ignored path is not UTF-8")
        })?;
        let size = fs::metadata(worktree.join(&relative))
            .ok()
            .map(|metadata| metadata.len());
        files.push(json!({ "path": relative, "size": size }));
    }
    Ok(IgnoredResidue {
        files,
        total,
        truncated: total > IGNORED_RESIDUE_LIMIT,
    })
}

fn ignored_residue_error(
    ignored: &IgnoredResidue,
    snapshot: &SnapshotRecord,
    git: &Git,
    worktree: &WorktreeEntry,
) -> ToolError {
    ToolError::not_established(
        "HCTL2_TOOL_IGNORED_RESIDUE_PRESENT",
        "ignored files are present and --reject-ignored was set",
    )
    .with_recovery_action("clear_ignored_or_omit_reject_ignored")
    .with_details(json!({
        "worktree": worktree.path,
        "ignored_residue": ignored.files,
        "ignored_residue_total": ignored.total,
        "ignored_residue_truncated": ignored.truncated,
        "snapshot": snapshot.to_json(),
        "status_summary": status_summary(git, &worktree.path),
    }))
}

fn nested_repository_error(
    git: &Git,
    worktree: &WorktreeEntry,
    gitlinks: &[Gitlink],
    snapshot: Option<&SnapshotRecord>,
) -> ToolError {
    ToolError::not_established(
        "HCTL2_TOOL_NESTED_REPOSITORY_PRESENT",
        "nested repository or submodule has no copy in this repository",
    )
    .with_recovery_action("relocate_or_publish_nested_repository_or_discard")
    .with_details(json!({
        "worktree": worktree.path,
        "nested_repositories": gitlinks.iter().map(Gitlink::to_json).collect::<Vec<_>>(),
        "snapshot": snapshot.map(SnapshotRecord::to_json),
        "status_summary": status_summary(git, &worktree.path),
    }))
}

fn salvage_unproven(
    message: &str,
    snapshot: &SnapshotRecord,
    paths: &[String],
    status_summary: &str,
) -> ToolError {
    ToolError::not_established("HCTL2_TOOL_SALVAGE_UNPROVEN", message)
        .with_recovery_action("archive_snapshot_then_retry_remove")
        .with_details(json!({
            "snapshot_tree_sha": snapshot.result_tree_sha,
            "snapshot_commit_sha": snapshot.snapshot_commit_sha,
            "paths": paths,
            "status_summary": status_summary,
        }))
}

fn status_summary(git: &Git, worktree: &Path) -> String {
    git.invoke(
        worktree,
        &args(&["status", "--porcelain=v2", "-z", "--untracked-files=all"]),
    )
    .ok()
    .and_then(|output| output.stdout_text().ok())
    .unwrap_or_default()
    .chars()
    .take(2_000)
    .collect()
}

fn differing_paths(git: &Git, repository: &Path, left: &str, right: &str) -> Vec<String> {
    git.invoke(
        repository,
        &args(&[
            "diff-tree",
            "-z",
            "--no-commit-id",
            "--name-only",
            "-r",
            left,
            right,
        ]),
    )
    .ok()
    .map_or_else(Vec::new, |output| {
        output
            .stdout()
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
            .take(IGNORED_RESIDUE_LIMIT)
            .map(|path| String::from_utf8_lossy(path).into_owned())
            .collect()
    })
}

fn create_snapshot_commit(
    git: &Git,
    repository: &Repository,
    tree: &str,
    parent: &str,
) -> Result<String, ToolError> {
    let identity = commit_identity_env();
    let mut commit_arguments = args(&["commit-tree", tree, "-p", parent]);
    commit_arguments.extend(args(&["-m", SNAPSHOT_MESSAGE]));
    git.checked_with_env(
        &repository.anchor,
        &commit_arguments,
        &identity,
        "HCTL2_TOOL_ARCHIVE_FAILED",
        "create archive snapshot commit",
    )?
    .stdout_text()
}

fn commit_peel(
    git: &Git,
    repository: &Path,
    commit: &str,
    peel: &str,
) -> Result<String, ToolError> {
    git.checked(
        repository,
        &args(&[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{commit}^{{{peel}}}"),
        ]),
        "HCTL2_TOOL_GIT_INSPECTION_FAILED",
        "read snapshot object",
    )?
    .stdout_text()
}

fn commit_parent(git: &Git, repository: &Path, commit: &str) -> Result<Option<String>, ToolError> {
    let output = git.invoke(
        repository,
        &args(&[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{commit}^"),
        ]),
    )?;
    if output.success() {
        Ok(Some(output.stdout_text()?))
    } else if output.code() == Some(128) {
        Ok(None)
    } else {
        Err(ToolError::new(
            "HCTL2_TOOL_GIT_INSPECTION_FAILED",
            format!("could not read snapshot parent: {}", output.stderr()),
        ))
    }
}

fn commit_identity_env() -> [(&'static str, OsString); 4] {
    [
        ("GIT_AUTHOR_NAME", OsString::from("hctl2-tool")),
        ("GIT_AUTHOR_EMAIL", OsString::from("hctl2-tool@invalid")),
        ("GIT_COMMITTER_NAME", OsString::from("hctl2-tool")),
        ("GIT_COMMITTER_EMAIL", OsString::from("hctl2-tool@invalid")),
    ]
}

fn snapshot_ref(change_set_ref: &str, tree_sha: &str) -> String {
    format!("refs/hctl2/changesets/{change_set_ref}/trees/{tree_sha}")
}

fn is_object_id(value: &str) -> bool {
    (value.len() == 40 || value.len() == 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::{is_object_id, snapshot_ref};

    #[test]
    fn snapshot_refs_are_content_addressed() {
        assert_eq!(
            snapshot_ref("CS-1", "abc"),
            "refs/hctl2/changesets/CS-1/trees/abc"
        );
        assert!(is_object_id("0123456789abcdef0123456789abcdef01234567"));
        assert!(!is_object_id("NOT-AN-ID"));
    }
}
