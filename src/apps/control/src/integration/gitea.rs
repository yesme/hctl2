//! Hosted Gitea as an integration target: read the branch head and protection, ask the
//! platform to merge an existing review request, read the result back. Every call goes
//! through the packaged `tea api`; nothing here infers success from a response code alone.
use std::collections::BTreeMap;

use repo::integration::ProtectionSnapshot;
use repo::{Result, reject};
use serde_json::{Value, json};

use crate::scm::Hosted;

#[derive(Debug)]
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

/// Read the target branch's head and the protection that is in force for it, as the platform
/// reports them. Gitea keeps protection as named rules that may be globs, so the branch
/// record says whether it is protected and which rule applies; only that rule is fetched.
/// A protected branch whose rule cannot be read is an error, never "unprotected".
pub(crate) fn observe(hosted: &Hosted, full_name: &str, target_ref: &str) -> Result<Target> {
    let branch = branch(target_ref)?;
    let Some(record) = hosted.api("GET", &format!("repos/{full_name}/branches/{branch}"), None)?
    else {
        return Ok(Target {
            head: None,
            protection: snapshot(None),
        });
    };
    let head = record["commit"]["id"].as_str().map(str::to_owned);
    if record["protected"] != json!(true) {
        return Ok(Target {
            head,
            protection: snapshot(None),
        });
    }
    let rule_name = record["effective_branch_protection_name"]
        .as_str()
        .filter(|name| !name.is_empty())
        .ok_or_else(|| {
            reject(
                "PROTECTION_UNREAD",
                "the branch is protected but the platform names no rule for it",
                "inspect_platform_protection",
            )
        })?;
    let rule = hosted
        .api(
            "GET",
            &format!(
                "repos/{full_name}/branch_protections/{}",
                percent_encode(rule_name)
            ),
            None,
        )?
        .ok_or_else(|| {
            reject(
                "PROTECTION_UNREAD",
                format!("protection rule {rule_name:?} is in force but cannot be read"),
                "inspect_platform_protection",
            )
        })?;
    Ok(Target {
        head,
        protection: snapshot(Some(&rule)),
    })
}

/// A rule name as one path segment (`ma*` → `ma%2A`, `release/*` → `release%2F%2A`).
fn percent_encode(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for byte in name.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.~".contains(&byte) {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// Gitea's `BranchProtection` rule into the frozen snapshot shape. Every condition the rule
/// carries stays in the snapshot — the named slots for what the spec names, the rest under
/// `other` — so a change to any of them is seen at execution and readback. Only the rule's
/// timestamps are left out: they change without the conditions changing.
pub(crate) fn snapshot(protection: Option<&Value>) -> ProtectionSnapshot {
    let Some(protection) = protection else {
        return ProtectionSnapshot {
            other: BTreeMap::from([("protected".into(), json!(false))]),
            ..ProtectionSnapshot::default()
        };
    };
    let mut other = BTreeMap::from([("protected".into(), json!(true))]);
    if let Some(fields) = protection.as_object() {
        for (field, value) in fields {
            if !matches!(
                field.as_str(),
                "created_at"
                    | "updated_at"
                    | "enable_status_check"
                    | "status_check_contexts"
                    | "block_on_outdated_branch"
                    | "required_approvals"
            ) {
                other.insert(field.clone(), value.clone());
            }
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
        assert_eq!(frozen.other["branch_name"], json!("main"));
        assert!(!frozen.other.contains_key("created_at"));
        // A condition without a named slot still changes the snapshot.
        let mut bypass = protected.clone();
        bypass["block_admin_merge_override"] = json!(true);
        assert_ne!(snapshot(Some(&bypass)), frozen);
        assert_eq!(percent_encode("ma*"), "ma%2A");
        assert_eq!(percent_encode("release/*"), "release%2F%2A");
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
