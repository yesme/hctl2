//! GitHub as an integration target through the packaged `gh`: read the branch head and the
//! protection in force for it, merge an existing pull request with the source head pinned,
//! read the request back. The credential stays in `gh`'s own store; control never copies a
//! token, and the Git readback of the target uses whatever credential helper Git has (for a
//! private repository, `gh auth setup-git` is GitHub's own way to provide one).
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

use repo::git::run;
use repo::integration::{ProtectionSnapshot, Strategy};
use repo::{Result, reject};
use serde_json::{Value, json};

use super::target::{PlatformTarget, ReviewRequest, Target, branch};
use crate::services::Supervisor;

pub(crate) struct GitHub {
    gh: PathBuf,
    hostname: String,
}

impl GitHub {
    /// The packaged `gh` (or `HCTL2_GH`), talking to `hostname` with its own login.
    pub(crate) fn connect(services: &Supervisor, hostname: &str) -> Result<Self> {
        let requested = std::env::var_os("HCTL2_GH")
            .or_else(|| {
                services
                    .packaged_paths()
                    .map(|(install, _)| install.join("libexec/hctl2/gh").into_os_string())
            })
            .unwrap_or_else(|| "gh".into());
        let gh = foundation::git::resolve_executable(&requested)
            .ok_or_else(|| reject("PROVIDER_UNAVAILABLE", "gh is not available", "install_gh"))?;
        Ok(Self {
            gh,
            hostname: hostname.into(),
        })
    }

    #[cfg(test)]
    pub(crate) fn fixture(gh: PathBuf, hostname: &str) -> Self {
        Self {
            gh,
            hostname: hostname.into(),
        }
    }

    /// One REST call. `Ok(None)` is a 404 on a read; writes the platform refuses come back as
    /// `NATIVE_REJECTED` (4xx) or `NATIVE_CONFLICT` (409); anything else is `PLATFORM_UNAVAILABLE`.
    pub(crate) fn api(
        &self,
        method: &str,
        path: &str,
        body: Option<Value>,
    ) -> Result<Option<Value>> {
        let mut cmd = Command::new(&self.gh);
        cmd.env("GH_PROMPT_DISABLED", "1")
            .env("NO_COLOR", "1")
            .args(["api", "--hostname", &self.hostname, "--method", method]);
        if body.is_some() {
            cmd.args(["--input", "-"]);
        }
        cmd.arg(path);
        let output = run(&mut cmd, body.map(|v| v.to_string().into_bytes()))?;
        // gh reports the HTTP status on stderr as "(HTTP 405)"; the body stays on stdout.
        let stderr = String::from_utf8_lossy(&output.stderr);
        let status = stderr
            .split("(HTTP ")
            .nth(1)
            .and_then(|rest| rest.split(')').next())
            .and_then(|code| code.trim().parse::<u16>().ok());
        if output.status.success() {
            return Ok(Some(if output.stdout.is_empty() {
                Value::Null
            } else {
                serde_json::from_slice(&output.stdout)?
            }));
        }
        if method == "GET" && status == Some(404) {
            return Ok(None);
        }
        let detail = serde_json::from_slice::<Value>(&output.stdout)
            .ok()
            .and_then(|v| v["message"].as_str().map(str::to_owned))
            .unwrap_or_else(|| stderr.trim().to_owned());
        Err(reject(
            if method != "GET" && matches!(status, Some(401 | 403 | 404 | 405 | 422)) {
                "NATIVE_REJECTED"
            } else if method != "GET" && status == Some(409) {
                "NATIVE_CONFLICT"
            } else {
                "PLATFORM_UNAVAILABLE"
            },
            format!("gh api {method} {path} not confirmed (HTTP {status:?}): {detail}"),
            "read_back_original_intent",
        ))
    }
}

impl PlatformTarget for GitHub {
    fn observe(&self, full_name: &str, target_ref: &str) -> Result<Target> {
        observe(self, full_name, target_ref)
    }
    fn review_request(&self, full_name: &str, index: u64) -> Result<Option<ReviewRequest>> {
        pull_request(self, full_name, index)
    }
    fn check_strategy(&self, strategy: Strategy) -> Result<()> {
        merge_method(strategy).map(|_| ())
    }
    fn request_merge(
        &self,
        full_name: &str,
        index: u64,
        strategy: Strategy,
        head: &str,
        message: &str,
    ) -> Result<()> {
        request_merge(
            self,
            full_name,
            index,
            merge_method(strategy)?,
            head,
            message,
        )
        .map(|_| ())
    }
}

/// Branch head and the protection in force for it: classic branch protection, which the
/// branch record announces with `protected`, plus the rules repository rulesets apply to the
/// branch, which it does not. A branch announced as protected whose protection cannot be read
/// is an error, never "unprotected".
pub(crate) fn observe(github: &GitHub, full_name: &str, target_ref: &str) -> Result<Target> {
    let branch = branch(target_ref)?;
    let Some(record) = github.api("GET", &format!("repos/{full_name}/branches/{branch}"), None)?
    else {
        return Ok(Target {
            head: None,
            protection: snapshot(None, &[]),
        });
    };
    let head = record["commit"]["sha"].as_str().map(str::to_owned);
    let classic = github.api(
        "GET",
        &format!("repos/{full_name}/branches/{branch}/protection"),
        None,
    )?;
    if record["protected"] == json!(true) && classic.is_none() {
        return Err(reject(
            "PROTECTION_UNREAD",
            "the branch is protected but its protection cannot be read",
            "inspect_platform_protection",
        ));
    }
    let rules = github
        .api(
            "GET",
            &format!("repos/{full_name}/rules/branches/{branch}"),
            None,
        )?
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default();
    Ok(Target {
        head,
        protection: snapshot(classic.as_ref(), &rules),
    })
}

/// Classic `BranchProtection` and the ruleset rules in force into the frozen snapshot shape.
/// Everything that conditions a merge stays in the snapshot: the named slots for what the
/// spec names, the rest of the classic record under `other` (minus its URLs), and the rules
/// verbatim under `other.rules`, so any change is seen at execution and readback.
pub(crate) fn snapshot(classic: Option<&Value>, rules: &[Value]) -> ProtectionSnapshot {
    let mut other = BTreeMap::from([("protected".into(), json!(classic.is_some()))]);
    let mut contexts: Vec<String> = Vec::new();
    let mut requires_review_request = false;
    let mut strict_sync = false;
    let mut require_conversation_resolution = false;
    let mut required_approvals = 0;
    if let Some(classic) = classic {
        let reviews = &classic["required_pull_request_reviews"];
        let checks = &classic["required_status_checks"];
        contexts.extend(
            checks["contexts"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str().map(str::to_owned)),
        );
        contexts.extend(
            checks["checks"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|check| check["context"].as_str().map(str::to_owned)),
        );
        requires_review_request = !reviews.is_null();
        strict_sync = checks["strict"] == json!(true);
        require_conversation_resolution =
            classic["required_conversation_resolution"]["enabled"] == json!(true);
        required_approvals = reviews["required_approving_review_count"]
            .as_u64()
            .unwrap_or_default();
        if let Some(fields) = classic.as_object() {
            for (field, value) in fields {
                if matches!(
                    field.as_str(),
                    "url" | "required_status_checks" | "required_pull_request_reviews"
                ) {
                    continue;
                }
                other.insert(field.clone(), without_urls(value));
            }
        }
        other.insert(
            "required_pull_request_reviews".into(),
            without_urls(reviews),
        );
    }
    for rule in rules {
        match rule["type"].as_str() {
            Some("required_status_checks") => {
                contexts.extend(
                    rule["parameters"]["required_status_checks"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|check| check["context"].as_str().map(str::to_owned)),
                );
                strict_sync |=
                    rule["parameters"]["strict_required_status_checks_policy"] == json!(true);
            }
            Some("pull_request") => {
                requires_review_request = true;
                required_approvals = required_approvals.max(
                    rule["parameters"]["required_approving_review_count"]
                        .as_u64()
                        .unwrap_or_default(),
                );
                require_conversation_resolution |=
                    rule["parameters"]["required_review_thread_resolution"] == json!(true);
            }
            _ => {}
        }
    }
    if !rules.is_empty() {
        other.insert("rules".into(), Value::Array(rules.to_vec()));
    }
    contexts.sort();
    contexts.dedup();
    ProtectionSnapshot {
        requires_review_request,
        required_checks: contexts,
        strict_sync,
        require_conversation_resolution,
        required_approvals,
        other,
    }
}

/// A classic protection sub-record without its `url` / `*_url` fields, which say nothing.
fn without_urls(value: &Value) -> Value {
    match value.as_object() {
        Some(fields) => Value::Object(
            fields
                .iter()
                .filter(|(k, _)| *k != "url" && !k.ends_with("_url"))
                .map(|(k, v)| (k.clone(), without_urls(v)))
                .collect(),
        ),
        None => value.clone(),
    }
}

pub(crate) fn pull_request(
    github: &GitHub,
    full_name: &str,
    number: u64,
) -> Result<Option<ReviewRequest>> {
    let Some(value) = github.api("GET", &format!("repos/{full_name}/pulls/{number}"), None)? else {
        return Ok(None);
    };
    Ok(Some(ReviewRequest {
        index: number,
        state: value["state"].as_str().unwrap_or_default().to_owned(),
        merged: value["merged"] == json!(true),
        merge_commit_sha: value["merge_commit_sha"].as_str().map(str::to_owned),
        head_sha: value["head"]["sha"].as_str().map(str::to_owned),
        base_branch: value["base"]["ref"].as_str().map(str::to_owned),
        raw: value,
    }))
}

/// GitHub has no fast-forward merge method; `rebase` rewrites commits, `squash` changes the tree
/// wrapper, so only `merge_commit` maps onto a platform merge that keeps the admitted commit.
pub(crate) fn merge_method(strategy: Strategy) -> Result<&'static str> {
    match strategy {
        Strategy::MergeCommit => Ok("merge"),
        Strategy::FastForward => Err(reject(
            "STRATEGY_UNSUPPORTED",
            "GitHub merges pull requests with merge, squash or rebase; a fast-forward of the exact candidate is not offered. Choose merge_commit.",
            "choose_merge_commit_strategy",
        )),
    }
}

/// `PUT pulls/{n}/merge` with `sha` pinning the head the platform must merge.
pub(crate) fn request_merge(
    github: &GitHub,
    full_name: &str,
    number: u64,
    method: &str,
    head_sha: &str,
    title: &str,
) -> Result<Value> {
    Ok(github
        .api(
            "PUT",
            &format!("repos/{full_name}/pulls/{number}/merge"),
            Some(json!({"merge_method": method, "sha": head_sha, "commit_title": title})),
        )?
        .unwrap_or(Value::Null))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classic_protection_and_ruleset_rules_map_onto_the_snapshot_and_fast_forward_is_refused() {
        let classic = json!({
            "url": "https://api.github.com/x",
            "required_status_checks": {"url": "u", "strict": false, "contexts": ["canary"], "contexts_url": "u", "checks": [{"context": "canary", "app_id": null}]},
            "required_pull_request_reviews": {"url": "u", "required_approving_review_count": 0, "dismiss_stale_reviews": false, "require_code_owner_reviews": false},
            "enforce_admins": {"url": "u", "enabled": true},
            "required_conversation_resolution": {"enabled": false},
            "allow_force_pushes": {"enabled": false},
            "allow_deletions": {"enabled": false}
        });
        let frozen = snapshot(Some(&classic), &[]);
        assert!(frozen.requires_review_request);
        assert_eq!(frozen.required_checks, vec!["canary"]);
        assert!(!frozen.strict_sync);
        assert_eq!(frozen.required_approvals, 0);
        assert!(!frozen.require_conversation_resolution);
        assert_eq!(frozen.other["enforce_admins"], json!({"enabled": true}));
        assert!(!frozen.other.contains_key("url"));
        assert!(!frozen.other.contains_key("rules"));
        // A condition without a named slot still changes the snapshot; a URL does not.
        let mut admins = classic.clone();
        admins["enforce_admins"]["enabled"] = json!(false);
        assert_ne!(snapshot(Some(&admins), &[]), frozen);
        let mut moved = classic.clone();
        moved["url"] = json!("https://api.github.com/y");
        assert_eq!(snapshot(Some(&moved), &[]), frozen);
        // Ruleset rules in force fold into the same slots and are frozen verbatim.
        let rules = vec![
            json!({"type": "pull_request", "parameters": {"required_approving_review_count": 2, "required_review_thread_resolution": true}}),
            json!({"type": "required_status_checks", "parameters": {"strict_required_status_checks_policy": true, "required_status_checks": [{"context": "lint"}]}}),
        ];
        let ruled = snapshot(None, &rules);
        assert!(ruled.requires_review_request);
        assert_eq!(ruled.required_checks, vec!["lint"]);
        assert!(ruled.strict_sync);
        assert_eq!(ruled.required_approvals, 2);
        assert!(ruled.require_conversation_resolution);
        assert_eq!(ruled.other["protected"], json!(false));
        assert_eq!(ruled.other["rules"].as_array().unwrap().len(), 2);
        let both = snapshot(Some(&classic), &rules);
        assert_eq!(both.required_checks, vec!["canary", "lint"]);
        assert_eq!(both.required_approvals, 2);
        assert_eq!(snapshot(None, &[]).other["protected"], json!(false));
        assert_eq!(merge_method(Strategy::MergeCommit).unwrap(), "merge");
        assert_eq!(
            merge_method(Strategy::FastForward).unwrap_err().code,
            "STRATEGY_UNSUPPORTED"
        );
    }
}
