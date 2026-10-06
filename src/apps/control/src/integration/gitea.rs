//! Hosted Gitea as an integration target: read the branch head and protection, ask the
//! platform to merge an existing review request, read the result back. Every call goes
//! through the packaged `tea api`; nothing here infers success from a response code alone.
use std::collections::BTreeMap;

use repo::integration::ProtectionSnapshot;
use repo::{Result, reject};
use serde_json::{Value, json};

use crate::scm::Hosted;

pub(crate) struct Target {
    pub head: Option<String>,
    pub protection: ProtectionSnapshot,
}

/// A branch name from a fully qualified ref.
fn branch(target_ref: &str) -> Result<&str> {
    target_ref.strip_prefix("refs/heads/").ok_or_else(|| {
        reject(
            "INVALID_INPUT",
            "platform targets are branches (refs/heads/...)",
            "correct_input",
        )
    })
}

/// Read the target branch's head and its protection rules as the platform reports them.
pub(crate) fn observe(hosted: &Hosted, full_name: &str, target_ref: &str) -> Result<Target> {
    let branch = branch(target_ref)?;
    let head = hosted
        .api("GET", &format!("repos/{full_name}/branches/{branch}"), None)?
        .and_then(|value| value["commit"]["id"].as_str().map(str::to_owned));
    let protection = hosted.api(
        "GET",
        &format!("repos/{full_name}/branch_protections/{branch}"),
        None,
    )?;
    Ok(Target {
        head,
        protection: snapshot(protection.as_ref()),
    })
}

/// Gitea's `BranchProtection` fields into the frozen snapshot shape. Fields the snapshot has
/// no slot for stay under `other`, so the comparison at execution still sees them.
pub(crate) fn snapshot(protection: Option<&Value>) -> ProtectionSnapshot {
    let Some(protection) = protection else {
        return ProtectionSnapshot {
            other: BTreeMap::from([("protected".into(), json!(false))]),
            ..ProtectionSnapshot::default()
        };
    };
    let mut other = BTreeMap::from([("protected".into(), json!(true))]);
    for field in [
        "enable_push",
        "enable_push_whitelist",
        "enable_merge_whitelist",
        "block_on_rejected_reviews",
        "block_on_official_review_requests",
        "dismiss_stale_approvals",
        "require_signed_commits",
        "enable_approvals_whitelist",
        "protected_file_patterns",
    ] {
        if !protection[field].is_null() {
            other.insert(field.into(), protection[field].clone());
        }
    }
    ProtectionSnapshot {
        // A protected branch that forbids direct pushes only changes through review requests.
        requires_review_request: protection["enable_push"] != json!(true),
        required_checks: if protection["enable_status_check"] == json!(true) {
            protection["status_check_contexts"]
                .as_array()
                .map(|contexts| {
                    contexts
                        .iter()
                        .filter_map(|c| c.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        },
        strict_sync: protection["block_on_outdated_branch"] == json!(true),
        // Gitea has no "all threads resolved" rule; it is recorded as absent, not as satisfied.
        require_conversation_resolution: false,
        required_approvals: protection["required_approvals"]
            .as_u64()
            .unwrap_or_default(),
        other,
    }
}

/// What the platform knows about one review request after a merge attempt or on readback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReviewRequest {
    pub index: u64,
    pub state: String,
    pub merged: bool,
    pub merge_commit_sha: Option<String>,
    pub head_sha: Option<String>,
    pub base_branch: Option<String>,
    pub raw: Value,
}

pub(crate) fn review_request(
    hosted: &Hosted,
    full_name: &str,
    index: u64,
) -> Result<Option<ReviewRequest>> {
    let Some(value) = hosted.api("GET", &format!("repos/{full_name}/pulls/{index}"), None)? else {
        return Ok(None);
    };
    Ok(Some(ReviewRequest {
        index,
        state: value["state"].as_str().unwrap_or_default().to_owned(),
        merged: value["merged"] == json!(true),
        merge_commit_sha: value["merge_commit_sha"].as_str().map(str::to_owned),
        head_sha: value["head"]["sha"].as_str().map(str::to_owned),
        base_branch: value["base"]["ref"].as_str().map(str::to_owned),
        raw: value,
    }))
}

/// Gitea merge style for a frozen strategy. `fast-forward-only` refuses anything that is not a
/// true fast-forward; `merge` always creates a merge commit.
pub(crate) fn merge_style(strategy: repo::integration::Strategy) -> &'static str {
    match strategy {
        repo::integration::Strategy::FastForward => "fast-forward-only",
        repo::integration::Strategy::MergeCommit => "merge",
    }
}

/// Ask the platform to merge the review request, pinning the source head it must merge.
/// Returns the platform's answer; the caller reads the request and the branch back before
/// treating anything as integrated.
pub(crate) fn request_merge(
    hosted: &Hosted,
    full_name: &str,
    index: u64,
    style: &str,
    head_commit_id: &str,
    message: &str,
) -> Result<Value> {
    let body = json!({
        "Do": style,
        "head_commit_id": head_commit_id,
        "merge_message_field": message,
        "delete_branch_after_merge": false,
    });
    Ok(hosted
        .api(
            "POST",
            &format!("repos/{full_name}/pulls/{index}/merge"),
            Some(body),
        )?
        .unwrap_or(Value::Null))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protection_snapshot_keeps_what_gitea_reports_and_marks_absent_rules_absent() {
        let protected = json!({
            "branch_name": "main", "enable_push": false, "enable_status_check": true,
            "status_check_contexts": ["canary", "lint"], "required_approvals": 1,
            "block_on_outdated_branch": true, "block_on_rejected_reviews": true,
            "require_signed_commits": false
        });
        let frozen = snapshot(Some(&protected));
        assert!(frozen.requires_review_request);
        assert_eq!(frozen.required_checks, vec!["canary", "lint"]);
        assert!(frozen.strict_sync);
        assert_eq!(frozen.required_approvals, 1);
        assert!(!frozen.require_conversation_resolution);
        assert_eq!(frozen.other["protected"], json!(true));
        assert_eq!(frozen.other["block_on_rejected_reviews"], json!(true));
        // Status checks declared but disabled are not required checks.
        let loose = json!({"enable_push": true, "enable_status_check": false, "status_check_contexts": ["x"]});
        let loose = snapshot(Some(&loose));
        assert!(!loose.requires_review_request);
        assert!(loose.required_checks.is_empty());
        let none = snapshot(None);
        assert_eq!(none.other["protected"], json!(false));
        assert!(!none.requires_review_request);
        assert_eq!(
            merge_style(repo::integration::Strategy::FastForward),
            "fast-forward-only"
        );
    }
}
