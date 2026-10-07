//! Hosted Gitea as a target, against a scripted `tea` and a real bare repository standing in
//! for the platform's Git: the API answers are files the test controls, the merges are real
//! commits, so protection drift, refusals, lost responses and readback are deterministic.
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use repo::integration::{self as domain, Form, Input, Strategy, TargetKind};
use serde_json::{Value, json};
use store::{Actor, ActorSource, Scope, Store, TrustedActor};
use tokio::sync::Mutex;

use super::{Shared, drive_with, gitea};
use crate::scm::Hosted;

struct Temp(PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn actor() -> TrustedActor {
    TrustedActor(Actor {
        principal: "owner".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![Scope::Control],
        authority: None,
    })
}

const TEA: &str = r#"#!/bin/sh
# $0.state/: protection.json (absent = unprotected), rule_name (effective rule the branch
# record names), rule_path (the one path segment the rule answers under), pr_state, pr_head,
# pr_base, merge_sha, posts; switches: fail_merge (405), lose_merge (merge, then die without
# an HTTP line), lose_unmerged (die without merging or an HTTP line), drift_after_merge.
# $0.state/../platform.git is the platform's Git; merges are real commits there.
S="$0.state"; R="$S/../platform.git"
export GIT_AUTHOR_NAME=gitea GIT_AUTHOR_EMAIL=gitea@localhost GIT_COMMITTER_NAME=gitea GIT_COMMITTER_EMAIL=gitea@localhost
method="$4"
if [ "$method" = POST ]; then path="$7"; body="$(cat)"; else path="$5"; fi
case "$method $path" in
  "GET "*/branches/*)
    head="$(git -C "$R" rev-parse --verify -q "refs/heads/${path##*/}")" || { printf 'HTTP/1.1 404 Not Found\n' >&2; printf '{}'; exit 1; }
    if [ -f "$S/protection.json" ]; then protected=true; name="$(cat "$S/rule_name")"; else protected=false; name=""; fi
    printf 'HTTP/1.1 200 OK\n' >&2
    printf '{"name":"%s","commit":{"id":"%s"},"protected":%s,"effective_branch_protection_name":"%s"}' "${path##*/}" "$head" "$protected" "$name" ;;
  "GET "*/branch_protections/*)
    if [ -f "$S/protection.json" ] && [ "${path##*/}" = "$(cat "$S/rule_path")" ]; then printf 'HTTP/1.1 200 OK\n' >&2; cat "$S/protection.json";
    else printf 'HTTP/1.1 404 Not Found\n' >&2; printf '{}'; exit 1; fi ;;
  "GET "*/pulls/*)
    printf 'HTTP/1.1 200 OK\n' >&2
    state="$(cat "$S/pr_state")"
    if [ "$state" = merged ]; then merged=true; sha="\"$(cat "$S/merge_sha")\""; else merged=false; sha=null; fi
    printf '{"number":7,"state":"%s","merged":%s,"merge_commit_sha":%s,"head":{"sha":"%s"},"base":{"ref":"%s"}}' \
      "$([ "$state" = merged ] && echo closed || echo open)" "$merged" "$sha" "$(cat "$S/pr_head")" "$(cat "$S/pr_base")" ;;
  "POST "*/pulls/*/merge)
    printf x >> "$S/posts"
    if [ -f "$S/lose_unmerged" ]; then exit 1; fi
    if [ -f "$S/fail_merge" ]; then printf 'HTTP/1.1 405 Method Not Allowed\n' >&2; printf '{"message":"required status check canary missing"}'; exit 1; fi
    if [ "$(cat "$S/pr_state")" = merged ]; then printf 'HTTP/1.1 405 Method Not Allowed\n' >&2; printf '{"message":"already merged"}'; exit 1; fi
    want="${body#*\"head_commit_id\":\"}"; want="${want%%\"*}"
    if [ "$want" != "$(cat "$S/pr_head")" ]; then printf 'HTTP/1.1 409 Conflict\n' >&2; printf '{"message":"head changed"}'; exit 1; fi
    base="$(cat "$S/pr_base")"; cand="$(cat "$S/pr_head")"
    # Like Gitea 1.27: a fast-forward-only merge's merge commit is the candidate itself.
    case "$body" in
      *fast-forward-only*) merge="$cand" ;;
      *) merge="$(git -C "$R" commit-tree "$cand^{tree}" -p "refs/heads/$base" -p "$cand" -m "merge #7")" ;;
    esac
    git -C "$R" update-ref "refs/heads/$base" "$merge"
    printf merged > "$S/pr_state"; printf '%s' "$merge" > "$S/merge_sha"
    if [ -f "$S/drift_after_merge" ]; then cp "$S/drift_after_merge" "$S/protection.json"; fi
    if [ -f "$S/lose_merge" ]; then exit 1; fi
    printf 'HTTP/1.1 200 OK\n' >&2 ;;
  *) printf 'HTTP/1.1 404 Not Found\n' >&2; printf '{}'; exit 1 ;;
esac
exit 0
"#;

const PROTECTION: &str = r#"{"rule_name":"main","branch_name":"main","enable_push":false,"enable_status_check":true,"status_check_contexts":["canary"],"required_approvals":0,"block_on_outdated_branch":false,"block_admin_merge_override":false,"created_at":"2026-10-07T00:00:00Z","updated_at":"2026-10-07T00:00:00Z"}"#;

struct Platform {
    hosted: Hosted,
    state: PathBuf,
    git_dir: PathBuf,
    /// Base of the published revision (the target's head at the start).
    base: String,
    /// The published candidate commit (review request #7's head).
    candidate: String,
    /// The admitted result tree: the candidate's tree.
    tree: String,
}

impl Platform {
    fn new(dir: &Path) -> Self {
        let tea = dir.join("tea");
        let state = dir.join("tea.state");
        std::fs::create_dir_all(&state).unwrap();
        std::fs::write(&tea, TEA).unwrap();
        std::fs::set_permissions(&tea, std::fs::Permissions::from_mode(0o700)).unwrap();
        let git_dir = dir.join("platform.git");
        let output = Command::new("git")
            .args(["init", "--bare", "--quiet"])
            .arg(&git_dir)
            .output()
            .unwrap();
        assert!(output.status.success());
        let mut platform = Self {
            hosted: Hosted::fixture(
                tea,
                "http://127.0.0.1:3000".into(),
                "admin".into(),
                "fixture-only".into(),
            ),
            state,
            git_dir,
            base: String::new(),
            candidate: String::new(),
            tree: String::new(),
        };
        let base_tree = platform.tree_with("a", "base\n");
        platform.base = platform.git(&["commit-tree", &base_tree, "-m", "base"]);
        platform.tree = platform.tree_with("a", "candidate\n");
        let tree = platform.tree.clone();
        let base = platform.base.clone();
        platform.candidate = platform.git(&["commit-tree", &tree, "-p", &base, "-m", "candidate"]);
        platform.git(&["update-ref", "refs/heads/main", &base]);
        let candidate = platform.candidate.clone();
        platform.git(&["update-ref", "refs/heads/cs-1", &candidate]);
        platform.set("pr_state", "open");
        platform.set("pr_head", &candidate);
        platform.set("pr_base", "main");
        platform.set("rule_name", "main");
        platform.set("rule_path", "main");
        platform.set("protection.json", PROTECTION);
        platform
    }
    fn git(&self, args: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.git_dir)
            .args(args)
            .env("GIT_AUTHOR_NAME", "gitea")
            .env("GIT_AUTHOR_EMAIL", "gitea@localhost")
            .env("GIT_COMMITTER_NAME", "gitea")
            .env("GIT_COMMITTER_EMAIL", "gitea@localhost")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }
    fn tree_with(&self, name: &str, content: &str) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.git_dir)
            .args(["hash-object", "-w", "--stdin"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child
                    .stdin
                    .take()
                    .unwrap()
                    .write_all(content.as_bytes())
                    .unwrap();
                child.wait_with_output()
            })
            .unwrap();
        let blob = String::from_utf8(output.stdout).unwrap().trim().to_owned();
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.git_dir)
            .arg("mktree")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child
                    .stdin
                    .take()
                    .unwrap()
                    .write_all(format!("100644 blob {blob}\t{name}\n").as_bytes())
                    .unwrap();
                child.wait_with_output()
            })
            .unwrap();
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }
    /// The platform merges review request #7 by itself (a human on the native UI, or a
    /// request whose response was lost): a real merge commit, the request reads as merged.
    fn merge_natively(&self, fast_forward: bool) -> String {
        let merge = if fast_forward {
            self.candidate.clone()
        } else {
            self.git(&[
                "commit-tree",
                &format!("{}^{{tree}}", self.candidate),
                "-p",
                "refs/heads/main",
                "-p",
                &self.candidate,
                "-m",
                "merge #7",
            ])
        };
        self.git(&["update-ref", "refs/heads/main", &merge]);
        self.set("pr_state", "merged");
        self.set("merge_sha", &merge);
        merge
    }
    fn set(&self, name: &str, value: &str) {
        std::fs::write(self.state.join(name), value).unwrap();
    }
    fn unset(&self, name: &str) {
        let _ = std::fs::remove_file(self.state.join(name));
    }
    fn read(&self, name: &str) -> String {
        std::fs::read_to_string(self.state.join(name)).unwrap_or_default()
    }
    fn head(&self) -> String {
        self.git(&["rev-parse", "refs/heads/main"])
    }
    fn posts(&self) -> usize {
        self.read("posts").len()
    }
    fn readback_root(&self) -> PathBuf {
        self.state.parent().unwrap().join("readback")
    }
    fn connect(&self) -> impl FnMut(&repo::Registration) -> repo::Result<Hosted> + '_ {
        move |_| {
            Ok(Hosted::fixture(
                self.state.parent().unwrap().join("tea"),
                self.hosted.url.clone(),
                self.hosted.username.clone(),
                self.hosted.token.clone(),
            ))
        }
    }
    fn drive(&self, shared: &Shared, repo_id: &str, id: &str) -> repo::Result<()> {
        drive_with(
            shared,
            repo_id,
            id,
            &self.readback_root(),
            &mut self.connect(),
        )
    }
}

/// A hosted Repo taken to active with capabilities declared verified, one admitted revision
/// published as review request #7 whose head is the platform's candidate commit.
fn hosted_repo(store: &mut Store, platform: &Platform) -> String {
    use repo::{FinishChoice, PlatformObservation, confirm_delivery, confirm_platform, finish};
    let prepared = repo::prepare(
        repo::Register {
            name: "example".into(),
            origin: repo::Origin::Local,
            platform: Some(repo::Platform::Local),
            instance: None,
            platform_repo_id: None,
            platform_path: Some("example".into()),
            local: None,
            remote_evidence: None,
            default_source: None,
        },
        None,
    )
    .unwrap();
    let reg = repo::admit(store, &actor(), "register", "register", prepared).unwrap();
    store
        .begin_effect(
            store.generation(),
            &repo::effect_id(&reg.repo_id, "platform"),
        )
        .unwrap();
    let reg = confirm_platform(
        store,
        &reg.repo_id,
        PlatformObservation {
            instance: "http://127.0.0.1:3000".into(),
            stable_id: "7".into(),
            full_name: "admin/example".into(),
            // The platform's Git, reached the way a clone URL is: this one is a path.
            clone_url: platform.git_dir.to_string_lossy().into_owned(),
            account_id: "1".into(),
            has_issues: true,
            can_write_issues: true,
            credential_ref: String::new(),
        },
    )
    .unwrap();
    store
        .begin_effect(
            store.generation(),
            &repo::effect_id(&reg.repo_id, "delivery"),
        )
        .unwrap();
    let reg = confirm_delivery(store, &reg.repo_id).unwrap();
    finish(
        store,
        &actor(),
        &reg.repo_id,
        reg.version,
        FinishChoice::Confirm("7"),
        "finish",
        "finish",
    )
    .unwrap();
    let repo_id = reg.repo_id;
    // Declared capabilities as the verified adapter will record them.
    {
        use store::{Command, Expected, Record, RecordData, Version};
        let binding = repo::binding(&repo_id);
        let current = store.get(&binding.key).unwrap().unwrap();
        let RecordData::Value { mut value } = current.data else {
            panic!("binding")
        };
        value["capabilities"]["remote_merge"] = json!(true);
        value["capabilities"]["protection_readback"] = json!(true);
        let mut scoped = actor().0;
        scoped.permission_scope.push(Scope::Repo(repo_id.clone()));
        let scoped = TrustedActor(scoped);
        let cmd = Command {
            command_id: "declare".into(),
            idempotency_key: "declare".into(),
            actor: scoped.0.clone(),
            target: binding.key.clone(),
            expected: Expected::Exact(Version::State(current.version)),
            binding: binding.clone(),
            input_digest: Command::digest_input("test.declare", &value).unwrap(),
            operation: "test.declare".into(),
            input: value.clone(),
        };
        store
            .submit(store.generation(), &scoped, &cmd, None, |tx| {
                tx.put(&Record {
                    key: binding.key.clone(),
                    version: current.version + 1,
                    revision_digest: foundation::canonical_json_sha256(&value).unwrap(),
                    data: RecordData::Value {
                        value: value.clone(),
                    },
                    sources: Vec::new(),
                    materials: Vec::new(),
                })?;
                Ok(json!({}))
            })
            .unwrap();
    }
    domain::admit_revision_seam(
        store,
        &actor(),
        &repo_id,
        &domain::AdmittedRevision {
            change_set_revision_id: "rev-1".into(),
            change_set_id: "cs-1".into(),
            parent_revision_id: None,
            base_commit_sha: platform.base.clone(),
            result_tree_sha: platform.tree.clone(),
            producer_ref: json!({"kind":"human_command","command_id":"seal"}),
            review_subject_digest: "d".repeat(64),
        },
    )
    .unwrap();
    domain::admit_platform_binding_seam(
        store,
        &actor(),
        &repo_id,
        "rev-1",
        &domain::ReviewRequestRef {
            index: 7,
            platform_commit_sha: platform.candidate.clone(),
        },
    )
    .unwrap();
    repo_id
}

fn temp(name: &str) -> Temp {
    let dir = std::env::temp_dir().join(format!("hctl2-gitea-integ-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    Temp(dir)
}

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
    let target = gitea::observe(&platform.hosted, "admin/example", "refs/heads/main").unwrap();
    let observation = domain::Observation {
        provider_ref: "http://127.0.0.1:3000/admin/example".into(),
        head: target.head,
        protection: Some(target.protection),
        continuity: Some(json!({"instance": "http://127.0.0.1:3000", "stable_id": "7"})),
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
        let target = gitea::observe(&platform.hosted, "admin/example", "refs/heads/main").unwrap();
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
        let target = gitea::observe(&platform.hosted, "admin/example", "refs/heads/main").unwrap();
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
    let target = gitea::observe(&platform.hosted, "admin/example", "refs/heads/main").unwrap();
    assert!(target.protection.requires_review_request);
    assert_eq!(target.protection.required_checks, vec!["canary"]);
    assert_eq!(target.protection.other["rule_name"], json!("ma*"));
    // Protected, but the rule in force cannot be read: an error, never "unprotected".
    platform.set("rule_path", "elsewhere");
    let err = gitea::observe(&platform.hosted, "admin/example", "refs/heads/main").unwrap_err();
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
