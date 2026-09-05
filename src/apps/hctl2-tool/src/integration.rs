//! Local integration uses Git plumbing only; it neither grants permission nor emits Receipts.
//!
//! A Git ref at `refs/hctl2/integrations/<key digest>/<input digest>` pins the prepared
//! commit before target CAS. This is a retry cache, not an intent ledger. Keeping the
//! commit reachable also prevents GC from changing a merge's identity during recovery.
//! P1 retains these refs, including on failure. P2 control owns explicit cleanup once
//! an intent is settled and no longer needs retries: successful results remain reachable
//! through the target; failed results need preservation before dropping their last ref.
//! Dropping the cache also drops this tool's key-to-result lookup. P1's packaging owner
//! documents this lifecycle; this command does not guess when a caller is done retrying.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use clap::ValueEnum;
use hctl2_foundation::canonical_json_sha256;
use serde_json::{Value, json};

use crate::git::{Git, args};
use crate::repository::{Repository, resolve_commit};
use crate::site_lock::SiteLock;
use crate::{ToolError, ToolOutput, observed_at_unix_ms};

#[derive(Debug, clap::Args)]
pub(crate) struct Arguments {
    /// Any directory in the local repository.
    #[arg(long)]
    repo: PathBuf,
    /// Full object ID of the candidate commit (not a moving ref).
    #[arg(long)]
    commit: String,
    /// Full object ID of the caller's baseline commit.
    #[arg(long)]
    base_commit_sha: String,
    /// Exact candidate tree approved by the caller; not the merged tree.
    #[arg(long)]
    result_tree_sha: String,
    /// Fully qualified local branch ref; only this target pointer is updated.
    #[arg(long)]
    target_ref: String,
    /// Full object ID expected at the target ref.
    #[arg(long)]
    expected_head: String,
    /// Caller-selected strategy; no automatic fallback or conflict resolution.
    #[arg(long, value_enum)]
    strategy: Strategy,
    /// Caller-supplied retry key, bound to all integration inputs in this repository.
    #[arg(long)]
    idempotency_key: String,
    /// Allow a checked-out target to diverge from its unchanged worktree/index.
    ///
    /// This flag is bound to the idempotency key. Changing it after a result is
    /// prepared returns HCTL2_TOOL_INTEGRATION_KEY_REUSED. If the target is then
    /// checked out, switch away from or detach its worktrees and retry with the
    /// original inputs.
    #[arg(long)]
    allow_checked_out_target: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum Strategy {
    FastForward,
    MergeCommit,
}

impl Strategy {
    const fn name(self) -> &'static str {
        match self {
            Self::FastForward => "fast_forward",
            Self::MergeCommit => "merge_commit",
        }
    }
}

struct Intent {
    commit: String,
    base: String,
    tree: String,
    expected: String,
    target: String,
    fields: Value,
    key_digest: String,
    input_digest: String,
    cache_prefix: String,
    cache_ref: String,
}

pub(crate) fn run(git: &Git, input: &Arguments) -> Result<ToolOutput, ToolError> {
    execute(git, input).map_err(|mut error| {
        // Also cover shared repository/lock/Git errors with a typed recovery action.
        if !error.details.is_object() {
            error.details = json!({});
        }
        error
            .details
            .as_object_mut()
            .expect("object initialized")
            .entry("recovery_action")
            .or_insert(json!("inspect_repository_then_retry_same_intent"));
        error
    })
}

fn execute(git: &Git, input: &Arguments) -> Result<ToolOutput, ToolError> {
    let repository = Repository::open(git, &input.repo)?;
    let intent = Intent::new(git, &repository.anchor, input)?;
    let _lock =
        SiteLock::acquire_for_intent(&repository.common_dir, "integrate", &intent.input_digest)?;
    let repo = &repository.anchor;

    // Preconditions deliberately run in task-book order, even for retries.
    require_commit(git, repo, &intent.commit, "HCTL2_TOOL_COMMIT_NOT_FOUND")?;
    let actual_tree = commit_tree(git, repo, &intent.commit)?;
    if actual_tree != intent.tree {
        return Err(rejected(
            "HCTL2_TOOL_INTEGRATION_TREE_MISMATCH",
            "candidate tree differs from result_tree_sha",
            "supply_matching_candidate_and_tree",
            json!({"expected_tree": intent.tree, "actual_tree": actual_tree}),
        ));
    }
    require_commit(git, repo, &intent.base, "HCTL2_TOOL_BASELINE_NOT_FOUND")?;
    require_commit(
        git,
        repo,
        &intent.expected,
        "HCTL2_TOOL_EXPECTED_HEAD_NOT_FOUND",
    )?;
    if !is_ancestor(git, repo, &intent.base, &intent.expected)? {
        return Err(rejected(
            "HCTL2_TOOL_INTEGRATION_BASELINE_NOT_ANCESTOR",
            "baseline is not an ancestor of the expected head",
            "reconcile_baseline_and_expected_head",
            intent.fields.clone(),
        ));
    }

    let prepared = intent.read_prepared(git, repo, input.strategy)?;
    let before = read_ref(git, repo, &input.target_ref).map_err(|error| {
        if let Some(new) = &prepared {
            unknown(&intent, new, None, error.to_string())
        } else {
            error
        }
    })?;
    if let Some(new) = &prepared {
        if contains_result(git, repo, &intent, new, before.as_deref())? {
            return success(
                git,
                &repository,
                &intent,
                new,
                &before,
                &before,
                "already_applied",
            );
        }
        if before.as_deref() != Some(&intent.expected) {
            return Err(unknown(
                &intent,
                new,
                before.as_deref(),
                "prepared result exists but target is neither the old head nor a descendant of the result",
            ));
        }
    }
    require_expected(&intent, before.as_deref())?;
    if intent.commit != intent.expected {
        require_available_target(git, &repository, input)?;
    }
    let reflog_message = format!("hctl2 integrate {}", &intent.key_digest[..12]);

    let new = match prepared {
        Some(new) => new,
        None => {
            let new = prepare_commit(git, repo, input, &intent)?;
            let zero = "0".repeat(new.len());
            git.checked(
                repo,
                &args(&[
                    "update-ref",
                    "--no-deref",
                    "-m",
                    &reflog_message,
                    "--create-reflog",
                    &intent.cache_ref,
                    &new,
                    &zero,
                ]),
                "HCTL2_TOOL_INTEGRATION_PREPARATION_FAILED",
                "pin prepared integration commit",
            )?;
            if read_ref(git, repo, &intent.cache_ref)?.as_deref() != Some(&new) {
                return Err(rejected(
                    "HCTL2_TOOL_INTEGRATION_CACHE_INVALID",
                    "prepared ref did not read back to the new commit",
                    "inspect_prepared_ref_then_retry_same_intent",
                    json!({"prepared_ref": intent.cache_ref, "new_head": new}),
                ));
            }
            new
        }
    };

    // Recheck immediately before CAS: non-tool Git writers do not take our site lock.
    let before = read_ref(git, repo, &input.target_ref)
        .map_err(|error| unknown(&intent, &new, None, error.to_string()))?;
    if contains_result(git, repo, &intent, &new, before.as_deref())? {
        return success(
            git,
            &repository,
            &intent,
            &new,
            &before,
            &before,
            "already_applied",
        );
    }
    require_expected(&intent, before.as_deref())?;
    require_available_target(git, &repository, input)?;
    let cas = git.invoke(
        repo,
        &args(&[
            "update-ref",
            "--no-deref",
            "-m",
            &reflog_message,
            &input.target_ref,
            &new,
            &intent.expected,
        ]),
    );
    // A process error/exit status is not evidence of the final ref value.
    let after = read_ref(git, repo, &input.target_ref)
        .map_err(|error| unknown(&intent, &new, None, error.to_string()))?;
    if contains_result(git, repo, &intent, &new, after.as_deref())? {
        return success(git, &repository, &intent, &new, &before, &after, "applied");
    }
    match cas {
        Ok(output)
            if output.code().is_some()
                && !output.success()
                && after.as_deref() == Some(&intent.expected) =>
        {
            Err(rejected(
                "HCTL2_TOOL_INTEGRATION_CAS_REJECTED",
                "Git rejected target compare-and-swap",
                "inspect_target_then_retry_same_intent_or_reconcile",
                json!({
                    "target_ref": input.target_ref, "before_head": before, "after_head": after,
                    "new_head": new, "prepared_ref": intent.cache_ref, "git_diagnostic": output.stderr(),
                }),
            ))
        }
        result => {
            // A head without the result could mean either a losing CAS or an applied
            // result later removed from history. Exit status cannot distinguish them.
            let mut error = unknown(
                &intent,
                &new,
                after.as_deref(),
                "CAS completion cannot be confirmed by target readback",
            );
            error.details["before_head"] = json!(before);
            error.details["git_diagnostic"] = json!(match result {
                Ok(output) => output.stderr(),
                Err(error) => error.to_string(),
            });
            Err(error)
        }
    }
}

impl Intent {
    fn new(git: &Git, repo: &Path, input: &Arguments) -> Result<Self, ToolError> {
        let commit = full_oid(&input.commit)?;
        let base = full_oid(&input.base_commit_sha)?;
        let tree = full_oid(&input.result_tree_sha)?;
        let expected = full_oid(&input.expected_head)?;
        if input.idempotency_key.is_empty()
            || !input.target_ref.starts_with("refs/heads/")
            || !git
                .invoke(repo, &args(&["check-ref-format", &input.target_ref]))?
                .success()
        {
            return Err(rejected(
                "HCTL2_TOOL_INVALID_ARGUMENT",
                "integration needs a nonempty key and a fully qualified local branch ref",
                "supply_local_branch_ref_and_idempotency_key",
                json!({"target_ref": input.target_ref}),
            ));
        }
        let fields = json!({
            "schema": "hctl2.integration-input.v1", "commit": commit, "base_commit_sha": base,
            "result_tree_sha": tree, "target_ref": input.target_ref, "expected_head": expected,
            "strategy": input.strategy.name(), "idempotency_key": input.idempotency_key,
            "allow_checked_out_target": input.allow_checked_out_target,
        });
        let digest = |value: &Value| {
            canonical_json_sha256(value).map_err(|error| {
                ToolError::new("HCTL2_TOOL_SERIALIZATION_FAILED", error.to_string())
            })
        };
        let key_digest = digest(&json!(input.idempotency_key))?;
        let input_digest = digest(&fields)?;
        let cache_prefix = format!("refs/hctl2/integrations/{key_digest}/");
        let cache_ref = format!("{cache_prefix}{input_digest}");
        Ok(Self {
            commit,
            base,
            tree,
            expected,
            target: input.target_ref.clone(),
            fields,
            key_digest,
            input_digest,
            cache_prefix,
            cache_ref,
        })
    }

    fn read_prepared(
        &self,
        git: &Git,
        repo: &Path,
        strategy: Strategy,
    ) -> Result<Option<String>, ToolError> {
        let refs = list_refs(git, repo, &self.cache_prefix)?;
        if refs.is_empty() {
            return Ok(None);
        }
        if refs.len() != 1 || refs[0].0 != self.cache_ref {
            return Err(rejected(
                "HCTL2_TOOL_INTEGRATION_KEY_REUSED",
                "idempotency key is already bound to different inputs",
                "reuse_original_inputs_or_supply_new_key",
                json!({"prepared_prefix": self.cache_prefix}),
            ));
        }
        let (_, new, symbolic) = &refs[0];
        if !symbolic.is_empty() {
            return Err(cache_invalid(&self.cache_ref));
        }
        require_commit(git, repo, new, "HCTL2_TOOL_INTEGRATION_CACHE_INVALID")?;
        let valid = match strategy {
            Strategy::FastForward => new == &self.commit,
            Strategy::MergeCommit if self.commit == self.expected => new == &self.commit,
            Strategy::MergeCommit => {
                let parents = git
                    .checked(
                        repo,
                        &args(&["show", "-s", "--format=%P", new]),
                        "HCTL2_TOOL_INTEGRATION_CACHE_INVALID",
                        "read prepared commit parents",
                    )?
                    .stdout_text()?;
                let expected = format!("{} {}", self.expected, self.commit);
                parents == expected
            }
        };
        if !valid {
            return Err(cache_invalid(&self.cache_ref));
        }
        Ok(Some(new.clone()))
    }
}

fn prepare_commit(
    git: &Git,
    repo: &Path,
    input: &Arguments,
    intent: &Intent,
) -> Result<String, ToolError> {
    if input.strategy == Strategy::FastForward {
        if intent.expected != intent.base
            || !is_ancestor(git, repo, &intent.expected, &intent.commit)?
        {
            return Err(rejected(
                "HCTL2_TOOL_INTEGRATION_NOT_FAST_FORWARD",
                "fast-forward requires target head equal to baseline and an ancestral path to candidate",
                "choose_matching_inputs_and_explicit_strategy",
                intent.fields.clone(),
            ));
        }
        return Ok(intent.commit.clone());
    }
    if intent.commit == intent.expected {
        return Ok(intent.commit.clone());
    }
    let merge = git.invoke(
        repo,
        &args(&[
            "merge-tree",
            "--write-tree",
            "--name-only",
            "-z",
            "--messages",
            &intent.expected,
            &intent.commit,
        ]),
    )?;
    match merge.code() {
        Some(0) => {
            let tree = parse_merge_tree(merge.stdout())?;
            let message = format!("Integrate {} into {}", intent.commit, input.target_ref);
            let new = git
                .checked(
                    repo,
                    &args(&[
                        "commit-tree",
                        &tree,
                        "-p",
                        &intent.expected,
                        "-p",
                        &intent.commit,
                        "-m",
                        &message,
                    ]),
                    "HCTL2_TOOL_INTEGRATION_COMMIT_FAILED",
                    "create merge commit using configured Git identity",
                )?
                .stdout_text()?;
            full_oid(&new)
        }
        Some(1) => Err(rejected(
            "HCTL2_TOOL_INTEGRATION_CONFLICT",
            "Git reports merge conflicts; no refs were changed",
            "resolve_in_changeset_then_submit_new_intent",
            conflict_details(merge.stdout())?,
        )),
        _ => Err(ToolError::new(
            "HCTL2_TOOL_INTEGRATION_MERGE_FAILED",
            format!("merge-tree failed: {}", merge.stderr()),
        )),
    }
}

fn conflict_details(output: &[u8]) -> Result<Value, ToolError> {
    let invalid = || {
        ToolError::new(
            "HCTL2_TOOL_GIT_OUTPUT_INVALID",
            "invalid NUL-delimited merge conflict record",
        )
    };
    let mut fields = output.split(|byte| *byte == 0).skip(1);
    let mut paths = BTreeSet::new();
    for path in fields.by_ref() {
        if path.is_empty() {
            break;
        }
        paths.insert(path);
    }
    let path_record =
        |path: &[u8]| json!({"path": String::from_utf8_lossy(path), "path_bytes": path});
    let mut messages = Vec::new();
    loop {
        let Some(count) = fields.next() else {
            break;
        };
        if count.is_empty() {
            break;
        }
        let count: usize = std::str::from_utf8(count)
            .ok()
            .and_then(|count| count.parse().ok())
            .ok_or_else(invalid)?;
        let affected = fields.by_ref().take(count).collect::<Vec<_>>();
        if affected.len() != count {
            return Err(invalid());
        }
        let kind = fields.next().ok_or_else(invalid)?;
        let message = fields.next().ok_or_else(invalid)?;
        // Some directory-rename conflicts have no unmerged index entries at all.
        // Their affected paths only occur in Git's structured messages section.
        if kind.starts_with(b"CONFLICT") {
            paths.extend(affected.iter().copied());
        }
        messages.push(json!({
            "kind": String::from_utf8_lossy(kind), "message": String::from_utf8_lossy(message),
            "paths": affected.into_iter().map(path_record).collect::<Vec<_>>(),
        }));
    }
    Ok(
        json!({"conflict_paths": paths.into_iter().map(path_record).collect::<Vec<_>>(), "merge_messages": messages}),
    )
}

fn parse_merge_tree(output: &[u8]) -> Result<String, ToolError> {
    let tree = output.split(|byte| *byte == 0).next().unwrap_or_default();
    let tree = std::str::from_utf8(tree)
        .map_err(|_| ToolError::new("HCTL2_TOOL_GIT_OUTPUT_INVALID", "non-UTF-8 merge tree ID"))?;
    full_oid(tree.trim_end())
}

fn commit_tree(git: &Git, repo: &Path, commit: &str) -> Result<String, ToolError> {
    git.checked(
        repo,
        &args(&["rev-parse", "--verify", &format!("{commit}^{{tree}}")]),
        "HCTL2_TOOL_GIT_INSPECTION_FAILED",
        "read commit tree",
    )?
    .stdout_text()
}

fn require_commit(git: &Git, repo: &Path, oid: &str, code: &'static str) -> Result<(), ToolError> {
    // resolve_commit intentionally accepts tags for the inspection/worktree commands;
    // integration's frozen SHA fields must identify commits themselves, without peeling.
    if resolve_commit(git, repo, oid, code)? != oid {
        return Err(rejected(
            "HCTL2_TOOL_INVALID_COMMIT_ID",
            "object ID identifies a tag, not a commit",
            "supply_full_commit_object_id",
            json!({"object_id": oid}),
        ));
    }
    Ok(())
}

fn is_ancestor(git: &Git, repo: &Path, base: &str, head: &str) -> Result<bool, ToolError> {
    let output = git.invoke(repo, &args(&["merge-base", "--is-ancestor", base, head]))?;
    match output.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(ToolError::new(
            "HCTL2_TOOL_GIT_INSPECTION_FAILED",
            format!("could not inspect ancestry: {}", output.stderr()),
        )),
    }
}

fn contains_result(
    git: &Git,
    repo: &Path,
    intent: &Intent,
    new: &str,
    actual: Option<&str>,
) -> Result<bool, ToolError> {
    match actual {
        None => Ok(false),
        Some(head) if head == new => Ok(true),
        Some(head) => is_ancestor(git, repo, new, head)
            .map_err(|error| unknown(intent, new, actual, error.to_string())),
    }
}

fn checked_out_worktrees(
    git: &Git,
    repository: &Repository,
    target: &str,
) -> Result<Vec<PathBuf>, ToolError> {
    Ok(repository
        .worktrees(git)?
        .into_iter()
        .filter(|worktree| worktree.branch.as_deref() == Some(target))
        .map(|worktree| worktree.path)
        .collect())
}

fn require_available_target(
    git: &Git,
    repository: &Repository,
    input: &Arguments,
) -> Result<(), ToolError> {
    let paths = checked_out_worktrees(git, repository, &input.target_ref)?;
    if !paths.is_empty() && !input.allow_checked_out_target {
        return Err(rejected(
            "HCTL2_TOOL_INTEGRATION_TARGET_CHECKED_OUT",
            "target branch is checked out; advancing only its ref would leave a stale index and worktree",
            "switch_or_detach_target_worktrees_then_retry_same_intent",
            json!({"target_ref": input.target_ref, "worktree_paths": paths}),
        ));
    }
    Ok(())
}

// refname / raw object ID / symbolic target, read in one Git command (no peel race).
fn list_refs(
    git: &Git,
    repo: &Path,
    pattern: &str,
) -> Result<Vec<(String, String, String)>, ToolError> {
    let output = git
        .checked(
            repo,
            &args(&[
                "for-each-ref",
                "--format=%(refname)%00%(objectname)%00%(symref)",
                pattern,
            ]),
            "HCTL2_TOOL_GIT_INSPECTION_FAILED",
            "read integration refs",
        )?
        .stdout_text()?;
    output
        .lines()
        .map(|line| {
            let parts = line.split('\0').collect::<Vec<_>>();
            if parts.len() != 3 {
                return Err(ToolError::new(
                    "HCTL2_TOOL_GIT_OUTPUT_INVALID",
                    "invalid ref record",
                ));
            }
            Ok((
                parts[0].to_owned(),
                full_oid(parts[1])?,
                parts[2].to_owned(),
            ))
        })
        .collect()
}

fn read_ref(git: &Git, repo: &Path, reference: &str) -> Result<Option<String>, ToolError> {
    for (name, oid, symbolic) in list_refs(git, repo, reference)? {
        if name == reference {
            if !symbolic.is_empty() {
                return Err(rejected(
                    "HCTL2_TOOL_INTEGRATION_SYMBOLIC_REF",
                    "integration requires a direct ref",
                    "supply_direct_target_ref",
                    json!({"ref": reference, "symbolic_target": symbolic}),
                ));
            }
            return Ok(Some(oid));
        }
    }
    Ok(None)
}

fn full_oid(value: &str) -> Result<String, ToolError> {
    if matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(value.to_ascii_lowercase())
    } else {
        Err(rejected(
            "HCTL2_TOOL_INVALID_OBJECT_ID",
            "integration needs full SHA-1 or SHA-256 object IDs, not moving refs",
            "supply_full_object_ids",
            json!({"value": value}),
        ))
    }
}

fn require_expected(intent: &Intent, actual: Option<&str>) -> Result<(), ToolError> {
    if actual == Some(&intent.expected) {
        return Ok(());
    }
    let code = if actual.is_none() {
        "HCTL2_TOOL_INTEGRATION_TARGET_MISSING"
    } else {
        "HCTL2_TOOL_INTEGRATION_HEAD_DRIFT"
    };
    Err(rejected(
        code,
        "target ref does not equal expected head",
        "reconcile_target_before_new_intent",
        json!({"target_ref": intent.fields["target_ref"], "expected_head": intent.expected, "actual_head": actual}),
    ))
}

fn cache_invalid(reference: &str) -> ToolError {
    rejected(
        "HCTL2_TOOL_INTEGRATION_CACHE_INVALID",
        "prepared ref does not match the integration inputs",
        "inspect_prepared_ref_and_target_without_reapplying",
        json!({"prepared_ref": reference}),
    )
}

fn rejected(code: &'static str, message: &str, recovery: &str, mut details: Value) -> ToolError {
    details["recovery_action"] = json!(recovery);
    ToolError::not_established(code, message).with_details(details)
}

fn unknown(
    intent: &Intent,
    new: &str,
    actual: Option<&str>,
    reason: impl Into<String>,
) -> ToolError {
    ToolError::new("HCTL2_TOOL_INTEGRATION_RESULT_UNKNOWN", reason).with_details(json!({
        "target_ref": intent.fields["target_ref"], "expected_head": intent.expected,
        "new_head": new, "observed_head": actual, "prepared_ref": intent.cache_ref,
        "recovery_action": "read_target_with_same_intent_before_retry_or_cleanup",
    }))
}

fn success(
    git: &Git,
    repository: &Repository,
    intent: &Intent,
    new: &str,
    before: &Option<String>,
    after: &Option<String>,
    status: &str,
) -> Result<ToolOutput, ToolError> {
    // No checkout/reset: a checked-out target's index and files stay byte-for-byte intact.
    let paths = checked_out_worktrees(git, repository, &intent.target)
        .map_err(|error| unknown(intent, new, after.as_deref(), error.to_string()))?;
    Ok(ToolOutput::json(
        json!({
            "schema": "hctl2.integration.v1", "evidence_level": "toolbox_readback",
            "observed_at_unix_ms": observed_at_unix_ms(), "operation": "integrate",
            "git": {"path": git.executable(), "version": git.version()},
            "git_common_dir": repository.common_dir, "outcome": "established", "status": status,
            "input": intent.fields, "target_ref": intent.target,
            "before_head": before, "new_head": new, "after_head": after,
            "integrated_tree_sha": commit_tree(git, &repository.anchor, new)
                .map_err(|error| unknown(intent, new, after.as_deref(), error.to_string()))?,
            "prepared_ref": intent.cache_ref, "worktrees_updated": false,
            "checked_out_worktrees": paths,
            "warnings": if paths.is_empty() { json!([]) } else { json!([{
                "code": "HCTL2_TOOL_INTEGRATION_TARGET_CHECKED_OUT",
                "message": "Target is checked out. Worktree/index were not updated; inspect them before committing.",
                "worktree_paths": paths,
            }]) },
        }),
        0,
    ))
}

#[cfg(test)]
mod tests {
    use super::conflict_details;
    use serde_json::json;

    #[test]
    fn directory_conflict_paths_can_exist_only_in_structured_messages() {
        let details = conflict_details(
            b"tree\0\0\x31\0directory/file\0CONFLICT (directory rename suggested)\0detail\0",
        )
        .unwrap();
        assert_eq!(
            details["conflict_paths"],
            json!([{"path": "directory/file", "path_bytes": b"directory/file".as_slice()}])
        );
    }

    #[test]
    fn malformed_structured_conflict_message_is_unreadable() {
        assert!(conflict_details(b"tree\0\0\x32\0one-path\0").is_err());
    }
}
