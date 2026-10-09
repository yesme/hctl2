//! Hosted Gitea and GitHub as targets, against a scripted `tea` / `gh` and a real bare
//! repository standing in for the platform's Git: the API answers are files the test controls,
//! the merges are real commits, so protection drift, refusals, lost responses and readback are
//! deterministic. The integration rules are shared; GitHub cases cover what differs.
use std::sync::Arc;

use repo::integration::{self as domain, Form, Input, Strategy, TargetKind};
use serde_json::{Value, json};
use store::Store;
use tokio::sync::Mutex;

use super::fixture::*;
use super::target::PlatformTarget;
use super::{Connection, Shared, drive_with, github};

/// A fresh control store with the hosted Repo, and one submitted intent against `main`.
fn scenario(
    temp: &Temp,
    platform: &Platform,
    name: &str,
    form: Form,
    strategy: Strategy,
) -> (Shared, String, String) {
    let mut store = Store::open(&temp.0.join(format!("control-{name}"))).unwrap();
    let repo_id = hosted_repo(&mut store, platform);
    let shared: Shared = Arc::new(Mutex::new(Some(store)));
    let id = submit(&shared, platform, &repo_id, name, form, strategy);
    (shared, repo_id, id)
}

fn submit(
    shared: &Shared,
    platform: &Platform,
    repo_id: &str,
    key: &str,
    form: Form,
    strategy: Strategy,
) -> String {
    let input = Input {
        key: key.into(),
        repo_id: repo_id.into(),
        change_set_revision_id: "rev-1".into(),
        target_kind: TargetKind::Platform,
        target_ref: "refs/heads/main".into(),
        form,
        strategy,
    };
    let target = platform.observe().unwrap();
    let observation = domain::Observation {
        provider_ref: platform.provider_ref(),
        head: target.head,
        protection: Some(target.protection),
        continuity: Some(platform.continuity()),
    };
    let mut guard = shared.blocking_lock();
    let store = guard.as_mut().unwrap();
    let preview = domain::prepare(store, &actor(), input, observation).unwrap();
    domain::submit(store, &actor(), &format!("integration:{key}"), preview)
        .unwrap()
        .intent_id
}

fn shown(shared: &Shared, repo_id: &str, id: &str) -> Value {
    let guard = shared.blocking_lock();
    domain::show(guard.as_ref().unwrap(), repo_id, id).unwrap()
}

fn receipts(shared: &Shared) -> usize {
    let guard = shared.blocking_lock();
    guard
        .as_ref()
        .unwrap()
        .list(domain::RECEIPT_KIND)
        .unwrap()
        .len()
}

#[test]
fn gitea_merge_pins_the_published_head_and_signs_one_receipt_only_after_readback() {
    let temp = temp("merge");
    let platform = Platform::new(&temp.0);
    let (shared, repo_id, id) = scenario(
        &temp,
        &platform,
        "one",
        Form::AcceptAdvance,
        Strategy::MergeCommit,
    );
    let before = shown(&shared, &repo_id, &id);
    assert_eq!(
        before["intent"]["preview"]["protection"]["required_checks"],
        json!(["canary"])
    );
    assert!(
        before["intent"]["preview"]["protection"]["requires_review_request"]
            .as_bool()
            .unwrap()
    );
    assert_eq!(
        before["intent"]["preview"]["protection"]["other"]["block_admin_merge_override"],
        json!(false)
    );
    // Expected-head is refused for this binding before anything is admitted.
    {
        let mut guard = shared.blocking_lock();
        let store = guard.as_mut().unwrap();
        let target = platform.observe().unwrap();
        let err = domain::prepare(
            store,
            &actor(),
            Input {
                key: "frozen".into(),
                repo_id: repo_id.clone(),
                change_set_revision_id: "rev-1".into(),
                target_kind: TargetKind::Platform,
                target_ref: "refs/heads/main".into(),
                form: Form::ExpectedHead,
                strategy: Strategy::MergeCommit,
            },
            domain::Observation {
                provider_ref: "x".into(),
                head: target.head,
                protection: Some(target.protection),
                continuity: Some(json!({"instance": "http://127.0.0.1:3000", "stable_id": "7"})),
            },
        )
        .unwrap_err();
        assert_eq!(err.code, "EXPECTED_HEAD_UNSUPPORTED");
    }
    // The platform refuses (a required check is missing): the intent waits, nothing merged,
    // and the refusal proved no write, so the attempt is not marked as dispatched.
    platform.set("fail_merge", "");
    platform.drive(&shared, &repo_id, &id).unwrap();
    let waiting = shown(&shared, &repo_id, &id);
    assert_eq!(waiting["intent"]["state"], "unknown", "{waiting}");
    assert_eq!(waiting["intent"]["attention"]["code"], "NOT_MERGEABLE");
    assert_eq!(waiting["intent"]["attempt"]["dispatched"], json!(false));
    assert_eq!(platform.read("pr_state"), "open");
    assert_eq!(platform.posts(), 1);
    // Protection changed under the frozen snapshot: nothing is even requested — for a named
    // slot (approvals) and for a condition the snapshot only keeps under `other`.
    platform.unset("fail_merge");
    platform.set(
        "protection.json",
        &PROTECTION.replace("\"required_approvals\":0", "\"required_approvals\":2"),
    );
    platform.drive(&shared, &repo_id, &id).unwrap();
    let drifted = shown(&shared, &repo_id, &id);
    assert_eq!(
        drifted["intent"]["attention"]["code"], "PROTECTION_CHANGED",
        "{drifted}"
    );
    assert_eq!(
        drifted["intent"]["attention"]["details"]["current"]["required_approvals"],
        2
    );
    platform.set(
        "protection.json",
        &PROTECTION.replace(
            "\"block_admin_merge_override\":false",
            "\"block_admin_merge_override\":true",
        ),
    );
    platform.drive(&shared, &repo_id, &id).unwrap();
    let bypass = shown(&shared, &repo_id, &id);
    assert_eq!(
        bypass["intent"]["attention"]["code"], "PROTECTION_CHANGED",
        "{bypass}"
    );
    assert_eq!(
        bypass["intent"]["attention"]["details"]["current"]["other"]["block_admin_merge_override"],
        json!(true)
    );
    assert_eq!(platform.posts(), 1);
    // Protection restored and, since the refusal, the target moved on (another change landed
    // on main): accept-advance merges anyway, and the Receipt's "before" is the head read
    // right before this request, not the head the refused attempt saw.
    platform.set("protection.json", PROTECTION);
    let moved_tree = platform.tree_with("a", "moved\n");
    let moved = platform.git(&[
        "commit-tree",
        &moved_tree,
        "-p",
        &platform.base,
        "-m",
        "moved",
    ]);
    platform.git(&["update-ref", "refs/heads/main", &moved]);
    platform.drive(&shared, &repo_id, &id).unwrap();
    let done = shown(&shared, &repo_id, &id);
    assert_eq!(done["intent"]["state"], "succeeded", "{done}");
    let receipt = &done["receipt"];
    assert_eq!(receipt["target_head_before"], json!(moved));
    assert_eq!(
        done["intent"]["attempt"],
        Value::Null,
        "terminal clears the attempt"
    );
    assert_eq!(receipt["target_head_after"], json!(platform.head()));
    assert_eq!(
        receipt["integrated_commit"],
        json!(platform.read("merge_sha"))
    );
    assert_eq!(receipt["integrated_tree"], json!(platform.tree));
    assert_eq!(receipt["evidence_level"], "hctl2-tool");
    assert_eq!(receipt["readback"]["status"], "applied");
    assert_eq!(receipt["readback"]["git"]["contains"], json!(true));
    assert!(
        receipt["readback"]["git"]["commit_parents"]
            .as_array()
            .unwrap()
            .contains(&json!(platform.candidate))
    );
    assert_eq!(platform.posts(), 2);
    assert!(
        platform
            .readback_root()
            .join(format!("{repo_id}.git"))
            .join("HEAD")
            .exists()
    );
    // Terminal: another pass neither posts nor signs again.
    assert!(platform.drive(&shared, &repo_id, &id).is_err());
    assert_eq!(platform.posts(), 2);
    assert_eq!(receipts(&shared), 1);
}

#[test]
fn a_lost_merge_response_is_unknown_until_readback_then_signs_without_a_second_request() {
    let temp = temp("lost");
    let platform = Platform::new(&temp.0);
    let (shared, repo_id, id) = scenario(
        &temp,
        &platform,
        "one",
        Form::AcceptAdvance,
        Strategy::FastForward,
    );
    // The platform merges but the response is lost on the way back.
    platform.set("lose_merge", "");
    platform.drive(&shared, &repo_id, &id).unwrap();
    let unknown = shown(&shared, &repo_id, &id);
    assert_eq!(
        platform.read("pr_state"),
        "merged",
        "the platform did merge"
    );
    // Readback in the same pass already saw the merge: this is a confirmed result, not a loss.
    assert_eq!(unknown["intent"]["state"], "succeeded", "{unknown}");
    assert_eq!(unknown["receipt"]["readback"]["status"], "applied");
    assert_eq!(
        unknown["receipt"]["integrated_commit"],
        json!(platform.candidate)
    );
    assert_eq!(unknown["receipt"]["integrated_tree"], json!(platform.tree));
    assert_eq!(platform.posts(), 1);
    // A request that already reads back as merged before any attempt, with the published
    // head into the frozen target: already applied, no POST, still read back through Git.
    let platform2 = Platform::new(&temp.0.join("second"));
    let (shared2, repo2, id2) = scenario(
        &temp,
        &platform2,
        "two",
        Form::AcceptAdvance,
        Strategy::MergeCommit,
    );
    let merge = platform2.merge_natively(false);
    platform2.drive(&shared2, &repo2, &id2).unwrap();
    let done = shown(&shared2, &repo2, &id2);
    assert_eq!(done["intent"]["state"], "succeeded", "{done}");
    assert_eq!(done["receipt"]["readback"]["status"], "already_applied");
    assert_eq!(done["receipt"]["integrated_commit"], json!(merge));
    // Nothing was written by this intent and no earlier head was observed: not claimed.
    assert!(done["receipt"]["target_head_before"].is_null(), "{done}");
    assert_eq!(done["receipt"]["evidence_level"], "hctl2-tool");
    assert_eq!(done["receipt"]["readback"]["git"]["contains"], json!(true));
    assert_eq!(platform2.posts(), 0);
}

#[test]
fn a_request_whose_response_was_lost_before_the_merge_is_only_read_back_never_resent() {
    let temp = temp("inflight");
    let platform = Platform::new(&temp.0);
    let (shared, repo_id, id) = scenario(
        &temp,
        &platform,
        "one",
        Form::AcceptAdvance,
        Strategy::MergeCommit,
    );
    // The request leaves, nothing comes back, and the platform has not merged (yet).
    platform.set("lose_unmerged", "");
    platform.drive(&shared, &repo_id, &id).unwrap();
    let unknown = shown(&shared, &repo_id, &id);
    assert_eq!(unknown["intent"]["state"], "unknown", "{unknown}");
    assert_eq!(unknown["intent"]["attention"]["code"], "RESULT_UNKNOWN");
    assert_eq!(unknown["intent"]["attempt"]["dispatched"], json!(true));
    assert_eq!(platform.posts(), 1);
    // Later passes only read back: no second POST, still unknown, no Receipt.
    platform.drive(&shared, &repo_id, &id).unwrap();
    platform.drive(&shared, &repo_id, &id).unwrap();
    assert_eq!(platform.posts(), 1);
    assert_eq!(shown(&shared, &repo_id, &id)["intent"]["state"], "unknown");
    assert_eq!(receipts(&shared), 0);
    // The target stays busy for anyone else meanwhile.
    {
        let mut guard = shared.blocking_lock();
        let store = guard.as_mut().unwrap();
        let target = platform.observe().unwrap();
        let preview = domain::prepare(
            store,
            &actor(),
            Input {
                key: "other".into(),
                repo_id: repo_id.clone(),
                change_set_revision_id: "rev-1".into(),
                target_kind: TargetKind::Platform,
                target_ref: "refs/heads/main".into(),
                form: Form::AcceptAdvance,
                strategy: Strategy::MergeCommit,
            },
            domain::Observation {
                provider_ref: "http://127.0.0.1:3000/admin/example".into(),
                head: target.head,
                protection: Some(target.protection),
                continuity: Some(json!({"instance": "http://127.0.0.1:3000", "stable_id": "7"})),
            },
        )
        .unwrap();
        let err = domain::submit(store, &actor(), "integration:other", preview).unwrap_err();
        assert_eq!(err.code, "TARGET_BUSY");
    }
    // Control restarts: the recorded attempt survives and still forbids resending.
    let dir = temp.0.join("control-one");
    drop(shared.blocking_lock().take());
    let shared: Shared = Arc::new(Mutex::new(Some(Store::open(&dir).unwrap())));
    platform.drive(&shared, &repo_id, &id).unwrap();
    assert_eq!(platform.posts(), 1);
    assert_eq!(shown(&shared, &repo_id, &id)["intent"]["state"], "unknown");
    // The in-flight request finally lands on the platform: readback confirms, one Receipt.
    platform.unset("lose_unmerged");
    let merge = platform.merge_natively(false);
    platform.drive(&shared, &repo_id, &id).unwrap();
    let done = shown(&shared, &repo_id, &id);
    assert_eq!(done["intent"]["state"], "succeeded", "{done}");
    assert_eq!(done["receipt"]["integrated_commit"], json!(merge));
    assert_eq!(done["receipt"]["readback"]["status"], "applied");
    assert_eq!(platform.posts(), 1);
    assert_eq!(receipts(&shared), 1);
}

#[test]
fn a_merged_request_must_still_be_the_published_head_aimed_at_the_frozen_target() {
    let temp = temp("mismatch");
    // Merged, but with another head than the published revision: not this intent's result.
    let platform = Platform::new(&temp.0.join("source"));
    let (shared, repo_id, id) = scenario(
        &temp,
        &platform,
        "source",
        Form::AcceptAdvance,
        Strategy::MergeCommit,
    );
    let other = platform.git(&[
        "commit-tree",
        &format!("{}^{{tree}}", platform.candidate),
        "-p",
        &platform.base,
        "-m",
        "other",
    ]);
    platform.set("pr_head", &other);
    platform.set("pr_state", "merged");
    platform.set("merge_sha", &other);
    platform.git(&["update-ref", "refs/heads/main", &other]);
    platform.drive(&shared, &repo_id, &id).unwrap();
    let failed = shown(&shared, &repo_id, &id);
    assert_eq!(failed["intent"]["state"], "failed", "{failed}");
    assert_eq!(failed["intent"]["failure"]["code"], "SOURCE_HEAD_MISMATCH");
    assert_eq!(platform.posts(), 0);
    assert_eq!(receipts(&shared), 0);
    // Merged into another branch than the frozen target: no Receipt for `main`.
    let platform = Platform::new(&temp.0.join("target"));
    let (shared, repo_id, id) = scenario(
        &temp,
        &platform,
        "target",
        Form::AcceptAdvance,
        Strategy::MergeCommit,
    );
    platform.git(&["update-ref", "refs/heads/feature", &platform.base]);
    platform.set("pr_base", "feature");
    platform.set("pr_state", "merged");
    platform.set("merge_sha", &platform.candidate);
    platform.drive(&shared, &repo_id, &id).unwrap();
    let failed = shown(&shared, &repo_id, &id);
    assert_eq!(failed["intent"]["state"], "failed", "{failed}");
    assert_eq!(failed["intent"]["failure"]["code"], "TARGET_MISMATCH");
    assert_eq!(platform.posts(), 0);
    // Still open but retargeted: refused before anything is sent.
    let platform = Platform::new(&temp.0.join("retargeted"));
    let (shared, repo_id, id) = scenario(
        &temp,
        &platform,
        "retargeted",
        Form::AcceptAdvance,
        Strategy::MergeCommit,
    );
    platform.set("pr_base", "feature");
    platform.drive(&shared, &repo_id, &id).unwrap();
    let failed = shown(&shared, &repo_id, &id);
    assert_eq!(failed["intent"]["state"], "failed", "{failed}");
    assert_eq!(failed["intent"]["failure"]["code"], "TARGET_MISMATCH");
    assert_eq!(platform.posts(), 0);
}

#[test]
fn a_merge_commit_the_target_does_not_carry_is_not_confirmed() {
    let temp = temp("carry");
    let platform = Platform::new(&temp.0);
    let (shared, repo_id, id) = scenario(
        &temp,
        &platform,
        "one",
        Form::AcceptAdvance,
        Strategy::MergeCommit,
    );
    // The request reads as merged with a real merge commit, but `main` points elsewhere.
    let merge = platform.merge_natively(false);
    let unrelated_tree = platform.tree_with("z", "elsewhere\n");
    let unrelated = platform.git(&["commit-tree", &unrelated_tree, "-m", "unrelated"]);
    platform.git(&["update-ref", "refs/heads/main", &unrelated]);
    platform.drive(&shared, &repo_id, &id).unwrap();
    let unknown = shown(&shared, &repo_id, &id);
    assert_eq!(unknown["intent"]["state"], "unknown", "{unknown}");
    assert_eq!(unknown["intent"]["attention"]["code"], "RESULT_UNKNOWN");
    assert_eq!(
        unknown["intent"]["attention"]["details"]["readback"]["contains"],
        json!(false)
    );
    assert_eq!(receipts(&shared), 0);
    assert_eq!(platform.posts(), 0);
    // `main` carries the merge commit again and has moved on past it: confirmed, and the
    // Receipt records the head that was actually read, not the merge commit.
    let later_tree = platform.tree_with("a", "later\n");
    let later = platform.git(&["commit-tree", &later_tree, "-p", &merge, "-m", "later"]);
    platform.git(&["update-ref", "refs/heads/main", &later]);
    platform.drive(&shared, &repo_id, &id).unwrap();
    let done = shown(&shared, &repo_id, &id);
    assert_eq!(done["intent"]["state"], "succeeded", "{done}");
    assert_eq!(done["receipt"]["integrated_commit"], json!(merge));
    assert_eq!(done["receipt"]["target_head_after"], json!(later));
    assert_eq!(done["receipt"]["integrated_tree"], json!(platform.tree));
    assert_eq!(platform.posts(), 0);
}

#[test]
fn protection_in_force_through_a_glob_rule_is_read_and_drift_after_the_write_blocks_the_receipt() {
    let temp = temp("glob");
    let platform = Platform::new(&temp.0);
    // `main` is protected by the rule `ma*`: the branch names it, the rule answers under its
    // encoded name, and a lookup by branch name would find nothing.
    platform.set("rule_name", "ma*");
    platform.set("rule_path", "ma%2A");
    platform.set(
        "protection.json",
        &PROTECTION.replace(
            "\"rule_name\":\"main\",\"branch_name\":\"main\"",
            "\"rule_name\":\"ma*\",\"branch_name\":\"\"",
        ),
    );
    let target = platform.observe().unwrap();
    assert!(target.protection.requires_review_request);
    assert_eq!(target.protection.required_checks, vec!["canary"]);
    assert_eq!(target.protection.other["rule_name"], json!("ma*"));
    // Protected, but the rule in force cannot be read: an error, never "unprotected".
    platform.set("rule_path", "elsewhere");
    let err = platform.observe().unwrap_err();
    assert_eq!(err.code, "PROTECTION_UNREAD");
    platform.set("rule_path", "ma%2A");
    let (shared, repo_id, id) = scenario(
        &temp,
        &platform,
        "one",
        Form::AcceptAdvance,
        Strategy::FastForward,
    );
    // The platform merges, and in the same instant the rule gains a required approval: the
    // write may have happened under the old rule, so the result is not confirmed.
    platform.set(
        "drift_after_merge",
        &platform
            .read("protection.json")
            .replace("\"required_approvals\":0", "\"required_approvals\":2"),
    );
    platform.drive(&shared, &repo_id, &id).unwrap();
    let drifted = shown(&shared, &repo_id, &id);
    assert_eq!(drifted["intent"]["state"], "unknown", "{drifted}");
    assert_eq!(drifted["intent"]["attention"]["code"], "PROTECTION_CHANGED");
    assert_eq!(
        drifted["intent"]["attention"]["details"]["frozen"]["required_approvals"],
        0
    );
    assert_eq!(
        drifted["intent"]["attention"]["details"]["current"]["required_approvals"],
        2
    );
    assert_eq!(platform.read("pr_state"), "merged");
    assert_eq!(receipts(&shared), 0);
    assert_eq!(platform.posts(), 1);
    // Rule restored: the attempt is read back (not resent) and confirmed.
    platform.unset("drift_after_merge");
    platform.set(
        "protection.json",
        &platform
            .read("protection.json")
            .replace("\"required_approvals\":2", "\"required_approvals\":0"),
    );
    platform.drive(&shared, &repo_id, &id).unwrap();
    let done = shown(&shared, &repo_id, &id);
    assert_eq!(done["intent"]["state"], "succeeded", "{done}");
    assert_eq!(
        done["receipt"]["integrated_commit"],
        json!(platform.candidate)
    );
    assert_eq!(platform.posts(), 1);
    assert_eq!(receipts(&shared), 1);
}

#[test]
fn a_review_request_whose_head_is_not_the_published_revision_is_not_merged() {
    let temp = temp("head");
    let platform = Platform::new(&temp.0);
    let (shared, repo_id, id) = scenario(
        &temp,
        &platform,
        "one",
        Form::AcceptAdvance,
        Strategy::MergeCommit,
    );
    // Someone pushed another commit to the review request's branch after publication.
    let pushed = platform.git(&[
        "commit-tree",
        &format!("{}^{{tree}}", platform.candidate),
        "-p",
        &platform.candidate,
        "-m",
        "pushed",
    ]);
    platform.set("pr_head", &pushed);
    platform.drive(&shared, &repo_id, &id).unwrap();
    let failed = shown(&shared, &repo_id, &id);
    assert_eq!(failed["intent"]["state"], "failed", "{failed}");
    assert_eq!(failed["intent"]["failure"]["code"], "SOURCE_HEAD_MISMATCH");
    assert_eq!(platform.posts(), 0);
    assert_eq!(platform.read("pr_state"), "open");
}

#[test]
fn github_merge_pins_the_head_only_reads_back_after_a_lost_response_and_signs_from_git() {
    let temp = temp("github");
    let platform = Platform::github(&temp.0);
    // GitHub offers no fast-forward of the exact candidate: refused before anything is frozen.
    assert_eq!(
        platform
            .connection()
            .check_strategy(Strategy::FastForward)
            .unwrap_err()
            .code,
        "STRATEGY_UNSUPPORTED"
    );
    let (shared, repo_id, id) = scenario(
        &temp,
        &platform,
        "one",
        Form::AcceptAdvance,
        Strategy::MergeCommit,
    );
    let before = shown(&shared, &repo_id, &id);
    let protection = &before["intent"]["preview"]["protection"];
    assert_eq!(protection["required_checks"], json!(["canary"]));
    assert!(protection["requires_review_request"].as_bool().unwrap());
    assert_eq!(
        protection["other"]["enforce_admins"],
        json!({"enabled": true})
    );
    assert!(protection["other"]["url"].is_null(), "{protection}");
    // Refused by the platform (a required check is missing): waits, no write, not dispatched.
    platform.set("fail_merge", "");
    platform.drive(&shared, &repo_id, &id).unwrap();
    let waiting = shown(&shared, &repo_id, &id);
    assert_eq!(
        waiting["intent"]["attention"]["code"], "NOT_MERGEABLE",
        "{waiting}"
    );
    assert_eq!(waiting["intent"]["attempt"]["dispatched"], json!(false));
    assert_eq!(platform.posts(), 1);
    // The request leaves and nothing comes back: only read back from here on.
    platform.unset("fail_merge");
    platform.set("lose_unmerged", "");
    platform.drive(&shared, &repo_id, &id).unwrap();
    let unknown = shown(&shared, &repo_id, &id);
    assert_eq!(unknown["intent"]["state"], "unknown", "{unknown}");
    assert_eq!(unknown["intent"]["attempt"]["dispatched"], json!(true));
    platform.drive(&shared, &repo_id, &id).unwrap();
    assert_eq!(platform.posts(), 2);
    assert_eq!(receipts(&shared), 0);
    // The platform merges it after all: confirmed from its Git, one Receipt, still 2 PUTs.
    platform.unset("lose_unmerged");
    let merge = platform.merge_natively(false);
    platform.drive(&shared, &repo_id, &id).unwrap();
    let done = shown(&shared, &repo_id, &id);
    assert_eq!(done["intent"]["state"], "succeeded", "{done}");
    let receipt = &done["receipt"];
    assert_eq!(receipt["integrated_commit"], json!(merge));
    assert_eq!(receipt["integrated_tree"], json!(platform.tree));
    assert_eq!(receipt["target_head_after"], json!(platform.head()));
    assert_eq!(receipt["evidence_level"], "hctl2-tool");
    assert_eq!(receipt["readback"]["git"]["contains"], json!(true));
    assert!(
        receipt["readback"]["git"]["commit_parents"]
            .as_array()
            .unwrap()
            .contains(&json!(platform.candidate))
    );
    assert_eq!(platform.posts(), 2);
    assert_eq!(receipts(&shared), 1);
}

#[test]
fn github_protection_announced_but_unreadable_is_an_error_and_ruleset_rules_are_frozen() {
    let temp = temp("github-rules");
    let platform = Platform::github(&temp.0);
    // The branch record says protected, the classic endpoint has nothing: not "unprotected".
    platform.unset("protection.json");
    platform.set("announce_protected", "");
    assert_eq!(platform.observe().unwrap_err().code, "PROTECTION_UNREAD");
    platform.unset("announce_protected");
    // No classic protection, but a ruleset puts two rules in force on the branch.
    platform.set(
        "rules.json",
        &json!([
            {"type": "pull_request", "parameters": {"required_approving_review_count": 1, "required_review_thread_resolution": true}},
            {"type": "required_status_checks", "parameters": {"strict_required_status_checks_policy": true, "required_status_checks": [{"context": "lint"}]}}
        ])
        .to_string(),
    );
    let target = platform.observe().unwrap();
    assert_eq!(target.protection.other["protected"], json!(false));
    assert!(target.protection.requires_review_request);
    assert_eq!(target.protection.required_checks, vec!["lint"]);
    assert_eq!(target.protection.required_approvals, 1);
    assert!(target.protection.require_conversation_resolution);
    assert!(target.protection.strict_sync);
    let (shared, repo_id, id) = scenario(
        &temp,
        &platform,
        "one",
        Form::AcceptAdvance,
        Strategy::MergeCommit,
    );
    // A rule changes after the preview: nothing is requested.
    platform.set(
        "rules.json",
        &json!([
            {"type": "pull_request", "parameters": {"required_approving_review_count": 2, "required_review_thread_resolution": true}},
            {"type": "required_status_checks", "parameters": {"strict_required_status_checks_policy": true, "required_status_checks": [{"context": "lint"}]}}
        ])
        .to_string(),
    );
    platform.drive(&shared, &repo_id, &id).unwrap();
    let drifted = shown(&shared, &repo_id, &id);
    assert_eq!(
        drifted["intent"]["attention"]["code"], "PROTECTION_CHANGED",
        "{drifted}"
    );
    assert_eq!(
        drifted["intent"]["attention"]["details"]["current"]["required_approvals"],
        2
    );
    assert_eq!(platform.posts(), 0);
    // A pull request already merged into another branch than the target: no Receipt for main.
    platform.git(&["update-ref", "refs/heads/release", &platform.base]);
    platform.set("pr_base", "release");
    platform.set("pr_state", "merged");
    platform.set("merge_sha", &platform.candidate);
    platform.set(
        "rules.json",
        &json!([
            {"type": "pull_request", "parameters": {"required_approving_review_count": 1, "required_review_thread_resolution": true}},
            {"type": "required_status_checks", "parameters": {"strict_required_status_checks_policy": true, "required_status_checks": [{"context": "lint"}]}}
        ])
        .to_string(),
    );
    platform.drive(&shared, &repo_id, &id).unwrap();
    let failed = shown(&shared, &repo_id, &id);
    assert_eq!(failed["intent"]["state"], "failed", "{failed}");
    assert_eq!(failed["intent"]["failure"]["code"], "TARGET_MISMATCH");
    assert_eq!(platform.posts(), 0);
    assert_eq!(receipts(&shared), 0);
}

#[test]
fn a_source_branch_that_moves_after_a_dispatched_request_is_settled_by_git_not_by_its_head() {
    let temp = temp("moved-source");
    let platform = Platform::new(&temp.0);
    let (shared, repo_id, id) = scenario(
        &temp,
        &platform,
        "one",
        Form::AcceptAdvance,
        Strategy::MergeCommit,
    );
    // The request left, nothing came back; someone then pushes to the source branch, so the
    // request's head (which follows the branch) is no longer the pinned candidate.
    platform.set("lose_unmerged", "");
    platform.drive(&shared, &repo_id, &id).unwrap();
    assert_eq!(shown(&shared, &repo_id, &id)["intent"]["state"], "unknown");
    platform.unset("lose_unmerged");
    let pushed = platform.git(&[
        "commit-tree",
        &format!("{}^{{tree}}", platform.candidate),
        "-p",
        &platform.candidate,
        "-m",
        "pushed later",
    ]);
    platform.set("pr_head", &pushed);
    // Not merged yet: a moved head after a possible write proves nothing; stays unknown,
    // keeps the attempt and the target, sends nothing.
    platform.drive(&shared, &repo_id, &id).unwrap();
    let still = shown(&shared, &repo_id, &id);
    assert_eq!(still["intent"]["state"], "unknown", "{still}");
    assert_eq!(still["intent"]["attention"]["code"], "RESULT_UNKNOWN");
    assert_eq!(still["intent"]["attempt"]["dispatched"], json!(true));
    assert_eq!(platform.posts(), 1);
    assert_eq!(receipts(&shared), 0);
    // The pinned request lands after all (the platform merges the candidate, the branch
    // keeps its newer head): the Git readback sees the candidate as the merge commit's
    // parent and confirms — the request's current head is irrelevant.
    let merge = platform.merge_natively(false);
    platform.drive(&shared, &repo_id, &id).unwrap();
    let done = shown(&shared, &repo_id, &id);
    assert_eq!(done["intent"]["state"], "succeeded", "{done}");
    assert_eq!(done["receipt"]["integrated_commit"], json!(merge));
    assert!(
        done["receipt"]["readback"]["git"]["commit_parents"]
            .as_array()
            .unwrap()
            .contains(&json!(platform.candidate))
    );
    assert_eq!(platform.posts(), 1);
    assert_eq!(receipts(&shared), 1);
    // The same within one round: the merge lands and the source branch is pushed before
    // the readback — the request's head is no longer the candidate, the Git facts still say
    // the candidate was merged.
    let platform3 = Platform::new(&temp.0.join("same-round"));
    let (shared3, repo3, id3) = scenario(
        &temp,
        &platform3,
        "three",
        Form::AcceptAdvance,
        Strategy::MergeCommit,
    );
    let pushed3 = platform3.git(&[
        "commit-tree",
        &format!("{}^{{tree}}", platform3.candidate),
        "-p",
        &platform3.candidate,
        "-m",
        "pushed right after",
    ]);
    platform3.set("advance_source_after_merge", &pushed3);
    platform3.drive(&shared3, &repo3, &id3).unwrap();
    let done3 = shown(&shared3, &repo3, &id3);
    assert_eq!(done3["intent"]["state"], "succeeded", "{done3}");
    assert_eq!(
        done3["receipt"]["integrated_commit"],
        json!(platform3.read("merge_sha"))
    );
    assert_eq!(platform3.read("pr_head"), pushed3);
    assert_eq!(platform3.posts(), 1);
    // Before anything was sent, a moved head is still final (the existing rule).
    let platform2 = Platform::new(&temp.0.join("unsent"));
    let (shared2, repo2, id2) = scenario(
        &temp,
        &platform2,
        "two",
        Form::AcceptAdvance,
        Strategy::MergeCommit,
    );
    platform2.set("pr_head", &pushed);
    platform2.drive(&shared2, &repo2, &id2).unwrap();
    assert_eq!(
        shown(&shared2, &repo2, &id2)["intent"]["failure"]["code"],
        "SOURCE_HEAD_MISMATCH"
    );
    assert_eq!(platform2.posts(), 0);
}

/// The public sandbox `yesme/hctl2-canary` (protected `main`: review request + check `canary`):
/// a real pull request merged by control through `gh`, read back from GitHub's Git.
#[test]
#[ignore = "UNVERIFIED: requires HCTL2_GITHUB_LIVE=1 and a gh login with push to yesme/hctl2-canary"]
fn live_github_canary_protected_main_is_merged_only_through_a_pull_request_with_a_pinned_head() {
    use crate::services::Supervisor;
    assert!(std::env::var_os("HCTL2_GITHUB_LIVE").is_some());
    let temp = temp("canary-live");
    let services = Supervisor::from_root(temp.0.clone());
    let github = github::GitHub::connect(&services, "github.com").unwrap();
    let full = "yesme/hctl2-canary";
    let api = |method: &str, path: &str, body: Option<Value>| {
        github
            .api(method, path, body)
            .unwrap()
            .unwrap_or(Value::Null)
    };
    let repo_meta = api("GET", &format!("repos/{full}"), None);
    let base = api("GET", &format!("repos/{full}/branches/main"), None)["commit"]["sha"]
        .as_str()
        .unwrap()
        .to_owned();
    // A branch with one commit, made through the API, and a pull request onto main.
    let branch = format!("hctl2-integration-{}", crate::dispatch::now_ms());
    api(
        "POST",
        &format!("repos/{full}/git/refs"),
        Some(json!({"ref": format!("refs/heads/{branch}"), "sha": base})),
    );
    let content =
        base64_standard(format!("integrated at {}\n", crate::dispatch::now_ms()).as_bytes());
    let created = api(
        "PUT",
        &format!("repos/{full}/contents/{branch}.txt"),
        Some(json!({"message": "hctl2 canary change", "content": content, "branch": branch})),
    );
    let head = created["commit"]["sha"].as_str().unwrap().to_owned();
    let tree = created["commit"]["tree"]["sha"]
        .as_str()
        .unwrap()
        .to_owned();
    let pr = api(
        "POST",
        &format!("repos/{full}/pulls"),
        Some(
            json!({"title": format!("hctl2 integration {branch}"), "head": branch, "base": "main"}),
        ),
    );
    let number = pr["number"].as_u64().unwrap();
    // The required check must finish before the platform lets anyone merge.
    for _ in 0..60 {
        let status = api(
            "GET",
            &format!("repos/{full}/commits/{head}/check-runs"),
            None,
        );
        if status["check_runs"].as_array().is_some_and(|runs| {
            runs.iter()
                .any(|r| r["name"] == "canary" && r["conclusion"] == "success")
        }) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
    let mut store = Store::open(&temp.0.join("control")).unwrap();
    let stable = repo_meta["id"].to_string();
    let repo_id = github_repo_with(
        &mut store,
        repo::PlatformObservation {
            instance: "github.com".into(),
            stable_id: stable.clone(),
            full_name: full.into(),
            clone_url: repo_meta["clone_url"].as_str().unwrap().into(),
            account_id: "0".into(),
            has_issues: true,
            can_write_issues: true,
            credential_ref: String::new(),
        },
        &base,
        &tree,
        number,
        &head,
    );
    let shared: Shared = Arc::new(Mutex::new(Some(store)));
    let target = github.observe(full, "refs/heads/main").unwrap();
    let id = {
        let input = Input {
            key: "canary".into(),
            repo_id: repo_id.clone(),
            change_set_revision_id: "rev-1".into(),
            target_kind: TargetKind::Platform,
            target_ref: "refs/heads/main".into(),
            form: Form::AcceptAdvance,
            strategy: Strategy::MergeCommit,
        };
        let observation = domain::Observation {
            provider_ref: format!("github.com/{full}"),
            head: target.head,
            protection: Some(target.protection),
            continuity: Some(json!({"instance": "github.com", "stable_id": stable})),
        };
        let mut guard = shared.blocking_lock();
        let store = guard.as_mut().unwrap();
        let preview = domain::prepare(store, &actor(), input, observation).unwrap();
        domain::submit(store, &actor(), "integration:canary", preview)
            .unwrap()
            .intent_id
    };
    let preview = shown(&shared, &repo_id, &id)["intent"]["preview"].clone();
    assert!(
        preview["protection"]["requires_review_request"]
            .as_bool()
            .unwrap(),
        "{preview}"
    );
    assert_eq!(preview["protection"]["required_checks"], json!(["canary"]));
    assert_eq!(
        preview["protection"]["other"]["enforce_admins"]["enabled"],
        json!(true)
    );
    let readback_root = temp.0.join("readback");
    let mut connect = |_: &repo::Registration| {
        Ok(Connection::GitHub(
            github::GitHub::connect(&services, "github.com").unwrap(),
        ))
    };
    for _ in 0..12 {
        drive_with(&shared, &repo_id, &id, &readback_root, &mut connect).unwrap();
        if shown(&shared, &repo_id, &id)["intent"]["state"] == "succeeded" {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
    let done = shown(&shared, &repo_id, &id);
    assert_eq!(done["intent"]["state"], "succeeded", "{done}");
    let merge = done["receipt"]["integrated_commit"]
        .as_str()
        .unwrap()
        .to_owned();
    let main = api("GET", &format!("repos/{full}/branches/main"), None)["commit"]["sha"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(main, merge, "main carries the merge commit");
    assert_eq!(done["receipt"]["target_head_after"], main);
    assert_eq!(done["receipt"]["evidence_level"], "hctl2-tool");
    assert_eq!(done["receipt"]["readback"]["git"]["contains"], json!(true));
    assert!(
        done["receipt"]["readback"]["git"]["commit_parents"]
            .as_array()
            .unwrap()
            .contains(&json!(head))
    );
    assert!(done["receipt"]["integrated_tree"].is_string());
    assert_eq!(
        api("GET", &format!("repos/{full}/pulls/{number}"), None)["merged"],
        json!(true)
    );
    eprintln!(
        "LIVE canary: pr #{number} head {head} merged as {merge}; receipt {}",
        done["receipt"]["receipt_id"]
    );
}

/// Standard base64 for the contents API; no new dependency for one test.
fn base64_standard(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let n =
            chunk.iter().fold(0u32, |acc, b| (acc << 8) | u32::from(*b)) << (8 * (3 - chunk.len()));
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(TABLE[((n >> (18 - 6 * i)) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}
