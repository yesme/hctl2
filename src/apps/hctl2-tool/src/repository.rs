use std::ffi::OsString;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::git::{Git, args};
use crate::{ToolError, ToolOutput, observed_at_unix_ms};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorktreeEntry {
    pub(crate) path: PathBuf,
    pub(crate) head: Option<String>,
    pub(crate) branch: Option<String>,
    pub(crate) detached: bool,
    pub(crate) bare: bool,
    pub(crate) locked: Option<String>,
    pub(crate) prunable: Option<String>,
}

#[derive(Debug)]
pub(crate) struct Repository {
    pub(crate) anchor: PathBuf,
    pub(crate) top_level: PathBuf,
    pub(crate) common_dir: PathBuf,
}

impl Repository {
    pub(crate) fn open(git: &Git, path: &Path) -> Result<Self, ToolError> {
        let anchor = path.canonicalize().map_err(|error| {
            ToolError::new(
                "HCTL2_TOOL_NOT_GIT_REPOSITORY",
                format!("cannot access repository path {}: {error}", path.display()),
            )
        })?;
        if !anchor.is_dir() {
            return Err(ToolError::new(
                "HCTL2_TOOL_NOT_GIT_REPOSITORY",
                format!("repository path is not a directory: {}", anchor.display()),
            ));
        }

        let bare = git.invoke(&anchor, &args(&["rev-parse", "--is-bare-repository"]))?;
        if !bare.success() {
            return Err(ToolError::new(
                "HCTL2_TOOL_NOT_GIT_REPOSITORY",
                format!(
                    "{} is not a Git repository: {}",
                    anchor.display(),
                    bare.stderr()
                ),
            ));
        }
        if bare.stdout_text()?.trim() == "true" {
            return Err(ToolError::new(
                "HCTL2_TOOL_BARE_REPOSITORY_UNSUPPORTED",
                format!(
                    "bare repositories cannot host a ChangeSet worktree: {}",
                    anchor.display()
                ),
            ));
        }

        let superproject = git.checked(
            &anchor,
            &args(&["rev-parse", "--show-superproject-working-tree"]),
            "HCTL2_TOOL_GIT_INSPECTION_FAILED",
            "read superproject",
        )?;
        let superproject = superproject.stdout_text()?;
        if !superproject.is_empty() {
            return Err(ToolError::new(
                "HCTL2_TOOL_SUBMODULE_UNSUPPORTED",
                format!(
                    "submodule worktrees are not Repo Instances: {} belongs to {}",
                    anchor.display(),
                    superproject
                ),
            ));
        }

        let top_level = absolute_path(
            git.checked(
                &anchor,
                &args(&["rev-parse", "--path-format=absolute", "--show-toplevel"]),
                "HCTL2_TOOL_GIT_INSPECTION_FAILED",
                "read worktree root",
            )?
            .stdout_text()?,
            "worktree root",
        )?;
        let common_dir = absolute_path(
            git.checked(
                &anchor,
                &args(&["rev-parse", "--path-format=absolute", "--git-common-dir"]),
                "HCTL2_TOOL_GIT_INSPECTION_FAILED",
                "read Git common directory",
            )?
            .stdout_text()?,
            "Git common directory",
        )?;

        Ok(Self {
            anchor: top_level.clone(),
            top_level,
            common_dir,
        })
    }

    pub(crate) fn head(&self, git: &Git) -> Result<Option<String>, ToolError> {
        let symbolic = git.invoke(&self.anchor, &args(&["symbolic-ref", "-q", "HEAD"]))?;
        if symbolic.success() {
            let reference = symbolic.stdout_text()?;
            let exists = git.invoke(
                &self.anchor,
                &args(&["show-ref", "--verify", "--quiet", &reference]),
            )?;
            if exists.code() == Some(1) {
                return Ok(None); // An unborn branch, not an unreadable commit.
            }
        }
        resolve_commit(git, &self.anchor, "HEAD", "HCTL2_TOOL_HEAD_MISSING").map(Some)
    }

    pub(crate) fn worktrees(&self, git: &Git) -> Result<Vec<WorktreeEntry>, ToolError> {
        let output = git.checked(
            &self.anchor,
            &args(&["worktree", "list", "--porcelain", "-z"]),
            "HCTL2_TOOL_GIT_INSPECTION_FAILED",
            "list worktrees",
        )?;
        parse_worktrees(output.stdout())
    }
}

pub(crate) fn inspect(
    git: &Git,
    path: PathBuf,
    requested_ref: Option<String>,
) -> Result<ToolOutput, ToolError> {
    let repository = Repository::open(git, &path)?;
    let head = repository.head(git)?;
    let worktrees = repository.worktrees(git)?;
    let requested_ref = requested_ref
        .map(|name| {
            let value = resolve_commit(git, &repository.anchor, &name, "HCTL2_TOOL_REF_NOT_FOUND")?;
            Ok(json!({ "name": name, "commit_sha": value }))
        })
        .transpose()?;
    let mut stable_identity = stable_repo_identity(git, &repository, head.as_deref())?;
    stable_identity["source"] = json!({
        "worktree_root": repository.top_level,
        "commit_sha": head,
    });
    let remotes = remotes(git, &repository)?;

    Ok(ToolOutput::json(
        json!({
            "schema": "hctl2.repository-inspection.v1",
            "evidence_level": "toolbox_readback",
            "outcome": "established",
            "observed_at_unix_ms": observed_at_unix_ms(),
            "git": {
                "path": git.executable(),
                "version": git.version(),
            },
            "common_directory_identity": {
                "git_common_dir": repository.common_dir,
                "worktrees": worktrees.iter().map(WorktreeEntry::to_json).collect::<Vec<_>>(),
            },
            "stable_repo_identity": stable_identity,
            "repository_state": {
                "worktree_root": repository.top_level,
                "head_commit_sha": head,
                "requested_ref": requested_ref,
            },
            "auxiliary_evidence": {
                "remotes": remotes,
            },
        }),
        0,
    ))
}

pub(crate) fn resolve_commit(
    git: &Git,
    repository: &Path,
    revision: &str,
    code: &'static str,
) -> Result<String, ToolError> {
    if revision.is_empty() || revision.starts_with('-') || revision.contains(['\0', '\n', '\r']) {
        return Err(ToolError::new(code, "Git revision is empty or unsafe"));
    }
    let expression = format!("{revision}^{{commit}}");
    let arguments = vec![
        OsString::from("rev-parse"),
        OsString::from("--verify"),
        OsString::from("--end-of-options"),
        OsString::from(expression),
    ];
    let output = git.invoke(repository, &arguments)?;
    if !output.success() {
        return Err(ToolError::not_established(
            code,
            format!(
                "Git commit {revision:?} does not exist: {}",
                output.stderr()
            ),
        ));
    }
    let value = output.stdout_text()?;
    if is_object_id(&value) {
        Ok(value)
    } else {
        Err(ToolError::new(
            "HCTL2_TOOL_GIT_OUTPUT_INVALID",
            format!("git returned an invalid object ID for {revision:?}: {value}"),
        ))
    }
}

impl WorktreeEntry {
    pub(crate) fn to_json(&self) -> Value {
        json!({
            "path": self.path,
            "head_commit_sha": self.head,
            "branch": self.branch,
            "detached": self.detached,
            "bare": self.bare,
            "locked": self.locked,
            "prunable": self.prunable,
        })
    }
}

fn stable_repo_identity(
    git: &Git,
    repository: &Repository,
    head: Option<&str>,
) -> Result<Value, ToolError> {
    let relative = ".hctl2/repo.toml";
    let path = repository.top_level.join(relative);
    let tracked = git.invoke(
        &repository.anchor,
        &args(&["ls-files", "--error-unmatch", "--", relative]),
    )?;
    let tree = head
        .map(|head| {
            git.checked(
                &repository.anchor,
                &args(&["ls-tree", "-z", head, "--", relative]),
                "HCTL2_TOOL_REPO_IDENTITY_UNREADABLE",
                "read committed Repo identity",
            )
        })
        .transpose()?;
    if let Some(tree) = tree.filter(|tree| !tree.stdout().is_empty()) {
        let entry = std::str::from_utf8(tree.stdout().strip_suffix(&[0]).unwrap_or(tree.stdout()))
            .map_err(|_| {
                ToolError::new(
                    "HCTL2_TOOL_REPO_IDENTITY_UNREADABLE",
                    "Repo identity tree entry is not UTF-8",
                )
            })?;
        let (metadata, _) = entry.split_once('\t').ok_or_else(|| {
            ToolError::new(
                "HCTL2_TOOL_GIT_OUTPUT_INVALID",
                "Repo identity tree entry has no path separator",
            )
        })?;
        let mut fields = metadata.split_whitespace();
        let mode = fields.next().unwrap_or_default();
        let kind = fields.next().unwrap_or_default();
        let object_id = fields.next().unwrap_or_default();
        if !matches!(mode, "100644" | "100755") || kind != "blob" {
            return Err(ToolError::new(
                "HCTL2_TOOL_REPO_IDENTITY_INVALID",
                format!("{relative} must be a regular tracked file, found mode {mode} {kind}"),
            ));
        }
        let blob = git.checked(
            &repository.anchor,
            &[
                OsString::from("cat-file"),
                OsString::from("blob"),
                OsString::from(object_id),
            ],
            "HCTL2_TOOL_REPO_IDENTITY_UNREADABLE",
            "read Repo identity blob",
        )?;
        let content = std::str::from_utf8(blob.stdout()).map_err(|_| {
            ToolError::new(
                "HCTL2_TOOL_REPO_IDENTITY_UNREADABLE",
                "Repo identity blob is not UTF-8",
            )
        })?;
        Ok(json!({
            "status": "committed",
            "path": path,
            "blob_oid": object_id,
            "content": content,
        }))
    } else if tracked.success() {
        Ok(json!({
            "status": "tracked_uncommitted",
            "path": path,
        }))
    } else if path.exists() {
        Ok(json!({
            "status": "untracked",
            "path": path,
        }))
    } else {
        Ok(json!({
            "status": "missing",
            "path": path,
        }))
    }
}

fn remotes(git: &Git, repository: &Repository) -> Result<Vec<Value>, ToolError> {
    let names = git
        .checked(
            &repository.anchor,
            &args(&["remote"]),
            "HCTL2_TOOL_GIT_INSPECTION_FAILED",
            "list remotes",
        )?
        .stdout_text()?;
    let mut values = Vec::new();
    for name in names.lines().filter(|line| !line.is_empty()) {
        let arguments = vec![
            OsString::from("remote"),
            OsString::from("get-url"),
            OsString::from("--all"),
            OsString::from("--"),
            OsString::from(name),
        ];
        let urls = git
            .checked(
                &repository.anchor,
                &arguments,
                "HCTL2_TOOL_GIT_INSPECTION_FAILED",
                "read remote URLs",
            )?
            .stdout_text()?;
        values.push(json!({
            "name": name,
            "urls": urls
                .lines()
                .filter(|line| !line.is_empty())
                .map(redact_remote_url)
                .collect::<Vec<_>>(),
        }));
    }
    Ok(values)
}

fn redact_remote_url(url: &str) -> String {
    if let Some((scheme, remainder)) = url.split_once("://") {
        let boundary = remainder.find(['/', '?', '#']).unwrap_or(remainder.len());
        let (authority, suffix) = remainder.split_at(boundary);
        if let Some((_, host)) = authority.rsplit_once('@') {
            return format!("{scheme}://***@{host}{suffix}");
        }
        return url.to_owned();
    }
    if let Some((_, host_and_path)) = url.split_once('@')
        && host_and_path.contains(':')
    {
        return format!("***@{host_and_path}");
    }
    url.to_owned()
}

fn parse_worktrees(bytes: &[u8]) -> Result<Vec<WorktreeEntry>, ToolError> {
    let mut entries = Vec::new();
    let mut current: Option<WorktreeEntry> = None;
    for field in bytes.split(|byte| *byte == 0) {
        if field.is_empty() {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            continue;
        }
        let text = std::str::from_utf8(field).map_err(|_| {
            ToolError::new(
                "HCTL2_TOOL_GIT_OUTPUT_INVALID",
                "git worktree list returned a non-UTF-8 path",
            )
        })?;
        let (key, value) = text.split_once(' ').unwrap_or((text, ""));
        if key == "worktree" {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            current = Some(WorktreeEntry {
                path: PathBuf::from(value),
                head: None,
                branch: None,
                detached: false,
                bare: false,
                locked: None,
                prunable: None,
            });
            continue;
        }
        let entry = current.as_mut().ok_or_else(|| {
            ToolError::new(
                "HCTL2_TOOL_GIT_OUTPUT_INVALID",
                format!("worktree field appeared before its path: {text}"),
            )
        })?;
        match key {
            "HEAD" => entry.head = Some(value.to_owned()),
            "branch" => entry.branch = Some(value.to_owned()),
            "detached" => entry.detached = true,
            "bare" => entry.bare = true,
            "locked" => entry.locked = Some(value.to_owned()),
            "prunable" => entry.prunable = Some(value.to_owned()),
            _ => {}
        }
    }
    if let Some(entry) = current {
        entries.push(entry);
    }
    Ok(entries)
}

fn absolute_path(value: String, label: &str) -> Result<PathBuf, ToolError> {
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(ToolError::new(
            "HCTL2_TOOL_GIT_OUTPUT_INVALID",
            format!("git returned a non-absolute {label}: {}", path.display()),
        ));
    }
    path.canonicalize().map_err(|error| {
        ToolError::new(
            "HCTL2_TOOL_GIT_OUTPUT_INVALID",
            format!("cannot canonicalize {label} {}: {error}", path.display()),
        )
    })
}

fn is_object_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::{parse_worktrees, redact_remote_url};

    #[test]
    fn parses_nul_delimited_worktree_records() {
        let entries = parse_worktrees(
            b"worktree /repo\0HEAD 0123456789012345678901234567890123456789\0branch refs/heads/main\0\0worktree /other\0HEAD abcdefabcdefabcdefabcdefabcdefabcdefabcd\0detached\0locked reason\0\0",
        )
        .expect("records must parse");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].branch.as_deref(), Some("refs/heads/main"));
        assert!(entries[1].detached);
        assert_eq!(entries[1].locked.as_deref(), Some("reason"));
    }

    #[test]
    fn redacts_remote_user_information() {
        assert_eq!(
            redact_remote_url("https://token@example.invalid"),
            "https://***@example.invalid"
        );
        assert_eq!(
            redact_remote_url("https://token@example.invalid/repo.git"),
            "https://***@example.invalid/repo.git"
        );
        assert_eq!(
            redact_remote_url("git@example.invalid:repo.git"),
            "***@example.invalid:repo.git"
        );
        assert_eq!(
            redact_remote_url("https://example.invalid/repo.git"),
            "https://example.invalid/repo.git"
        );
    }
}
