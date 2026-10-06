//! Hosted Gitea as a target, against a scripted `tea`: the platform's answers are files the
//! test controls, so protection drift, refusals, lost responses and readback are deterministic.
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
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
# $0.state/: head, protection.json (absent = unprotected), pr_state, pr_head, merge_sha,
# fail_merge (refuse with 405), lose_merge (merge, then die without an HTTP line), posts.
S="$0.state"
method="$4"
if [ "$method" = POST ]; then path="$7"; body="$(cat)"; else path="$5"; fi
case "$method $path" in
  "GET "*/branches/*)
    printf 'HTTP/1.1 200 OK\n' >&2
    printf '{"name":"%s","commit":{"id":"%s"}}' "${path##*/}" "$(cat "$S/head")" ;;
  "GET "*/branch_protections/*)
    if [ -f "$S/protection.json" ]; then printf 'HTTP/1.1 200 OK\n' >&2; cat "$S/protection.json";
    else printf 'HTTP/1.1 404 Not Found\n' >&2; printf '{}'; fi ;;
  "GET "*/pulls/*)
    printf 'HTTP/1.1 200 OK\n' >&2
    state="$(cat "$S/pr_state")"
    if [ "$state" = merged ]; then merged=true; sha="\"$(cat "$S/merge_sha")\""; else merged=false; sha=null; fi
    printf '{"number":7,"state":"%s","merged":%s,"merge_commit_sha":%s,"head":{"sha":"%s"},"base":{"ref":"main"}}' \
      "$([ "$state" = merged ] && echo closed || echo open)" "$merged" "$sha" "$(cat "$S/pr_head")" ;;
  "POST "*/pulls/*/merge)
    printf x >> "$S/posts"
    if [ -f "$S/fail_merge" ]; then printf 'HTTP/1.1 405 Method Not Allowed\n' >&2; printf '{"message":"required status check canary missing"}'; exit 1; fi
    if [ "$(cat "$S/pr_state")" = merged ]; then printf 'HTTP/1.1 405 Method Not Allowed\n' >&2; printf '{"message":"already merged"}'; exit 1; fi
    want="${body#*\"head_commit_id\":\"}"; want="${want%%\"*}"
    if [ "$want" != "$(cat "$S/pr_head")" ]; then printf 'HTTP/1.1 409 Conflict\n' >&2; printf '{"message":"head changed"}'; exit 1; fi
    printf merged > "$S/pr_state"
    # Like Gitea 1.27: a fast-forward-only merge's merge commit is the candidate itself.
    case "$body" in *fast-forward-only*) cp "$S/pr_head" "$S/merge_sha" ;; *) printf 'mmmm%s' "$(cat "$S/pr_head" | cut -c5-)" > "$S/merge_sha" ;; esac
    cp "$S/merge_sha" "$S/head"
    if [ -f "$S/lose_merge" ]; then exit 1; fi
    printf 'HTTP/1.1 200 OK\n' >&2 ;;
  "GET "*/git/commits/*)
    printf 'HTTP/1.1 200 OK\n' >&2
    printf '{"sha":"%s","commit":{"tree":{"sha":"%s"}}}' "${path##*/}" "$(cat "$S/tree")" ;;
  *) printf 'HTTP/1.1 404 Not Found\n' >&2; printf '{}'; exit 1 ;;
esac
exit 0
"#;

struct Platform {
    hosted: Hosted,
    state: PathBuf,
}

impl Platform {
    fn new(dir: &Path) -> Self {
        let tea = dir.join("tea");
        std::fs::write(&tea, TEA).unwrap();
        std::fs::set_permissions(&tea, std::fs::Permissions::from_mode(0o700)).unwrap();
        let state = dir.join("tea.state");
        std::fs::create_dir_all(&state).unwrap();
        std::fs::write(state.join("head"), "aaaa").unwrap();
        std::fs::write(state.join("pr_state"), "open").unwrap();
        std::fs::write(state.join("pr_head"), "head7777").unwrap();
        // The platform's merge carries exactly the admitted tree (a fast-forward does by nature).
        std::fs::write(state.join("tree"), "t".repeat(40)).unwrap();
        std::fs::write(
            state.join("protection.json"),
            json!({"branch_name":"main","enable_push":false,"enable_status_check":true,"status_check_contexts":["canary"],"required_approvals":0,"block_on_outdated_branch":false}).to_string(),
        )
        .unwrap();
        Self {
            hosted: Hosted::fixture(
                tea,
                "http://127.0.0.1:3000".into(),
                "admin".into(),
                "fixture-only".into(),
            ),
            state,
        }
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
    fn posts(&self) -> usize {
        self.read("posts").len()
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
}

/// A hosted Repo taken to active with capabilities declared verified, one admitted revision
/// published as review request #7.
fn hosted_repo(store: &mut Store) -> String {
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
            clone_url: "http://127.0.0.1:3000/admin/example.git".into(),
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
            base_commit_sha: "b".repeat(40),
            result_tree_sha: "t".repeat(40),
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
            platform_commit_sha: "head7777".into(),
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

#[test]
fn gitea_merge_pins_the_published_head_and_signs_one_receipt_only_after_readback() {
    let temp = temp("merge");
    let platform = Platform::new(&temp.0);
    let mut store = Store::open(&temp.0.join("control")).unwrap();
    let repo_id = hosted_repo(&mut store);
    let shared: Shared = Arc::new(Mutex::new(Some(store)));
    let id = submit(
        &shared,
        &platform,
        &repo_id,
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
    // The platform refuses (a required check is missing): the intent waits, nothing merged.
    platform.set("fail_merge", "");
    drive_with(&shared, &repo_id, &id, &mut platform.connect()).unwrap();
    let waiting = shown(&shared, &repo_id, &id);
    assert_eq!(waiting["intent"]["state"], "unknown", "{waiting}");
    assert_eq!(waiting["intent"]["attention"]["code"], "NOT_MERGEABLE");
    assert_eq!(platform.read("pr_state"), "open");
    assert_eq!(platform.posts(), 1);
    // Protection changed under the frozen snapshot: nothing is even requested.
    platform.unset("fail_merge");
    platform.set("protection.json", &json!({"branch_name":"main","enable_push":false,"enable_status_check":true,"status_check_contexts":["canary"],"required_approvals":2}).to_string());
    drive_with(&shared, &repo_id, &id, &mut platform.connect()).unwrap();
    let drifted = shown(&shared, &repo_id, &id);
    assert_eq!(
        drifted["intent"]["attention"]["code"], "PROTECTION_CHANGED",
        "{drifted}"
    );
    assert_eq!(
        drifted["intent"]["attention"]["details"]["current"]["required_approvals"],
        2
    );
    assert_eq!(platform.posts(), 1);
    // Protection restored; the merge goes through and reads back as merged.
    platform.set("protection.json", &json!({"branch_name":"main","enable_push":false,"enable_status_check":true,"status_check_contexts":["canary"],"required_approvals":0,"block_on_outdated_branch":false}).to_string());
    drive_with(&shared, &repo_id, &id, &mut platform.connect()).unwrap();
    let done = shown(&shared, &repo_id, &id);
    assert_eq!(done["intent"]["state"], "succeeded", "{done}");
    let receipt = &done["receipt"];
    assert_eq!(receipt["target_head_before"], "aaaa");
    assert_eq!(receipt["target_head_after"], platform.read("head"));
    assert_eq!(receipt["integrated_commit"], platform.read("merge_sha"));
    assert!(
        receipt["integrated_tree"].is_null(),
        "merge commits have no readable tree on Gitea"
    );
    assert_eq!(receipt["evidence_level"], "platform_adapter");
    assert_eq!(receipt["readback"]["status"], "applied");
    assert_eq!(platform.posts(), 2);
    // Terminal: another pass neither posts nor signs again.
    assert!(drive_with(&shared, &repo_id, &id, &mut platform.connect()).is_err());
    assert_eq!(platform.posts(), 2);
    let guard = shared.blocking_lock();
    assert_eq!(
        guard
            .as_ref()
            .unwrap()
            .list(domain::RECEIPT_KIND)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn a_lost_merge_response_is_unknown_until_readback_then_signs_without_a_second_request() {
    let temp = temp("lost");
    let platform = Platform::new(&temp.0);
    let mut store = Store::open(&temp.0.join("control")).unwrap();
    let repo_id = hosted_repo(&mut store);
    let shared: Shared = Arc::new(Mutex::new(Some(store)));
    let id = submit(
        &shared,
        &platform,
        &repo_id,
        "one",
        Form::AcceptAdvance,
        Strategy::FastForward,
    );
    // The platform merges but the response is lost on the way back.
    platform.set("lose_merge", "");
    drive_with(&shared, &repo_id, &id, &mut platform.connect()).unwrap();
    let unknown = shown(&shared, &repo_id, &id);
    assert_eq!(
        platform.read("pr_state"),
        "merged",
        "the platform did merge"
    );
    // Readback in the same pass already saw the merge: this is a confirmed result, not a loss.
    assert_eq!(unknown["intent"]["state"], "succeeded", "{unknown}");
    assert_eq!(unknown["receipt"]["readback"]["status"], "applied");
    assert_eq!(platform.posts(), 1);
    // A request that already reads back as merged before any attempt: already applied, no POST.
    let platform2_dir = temp.0.join("second");
    std::fs::create_dir_all(&platform2_dir).unwrap();
    let platform2 = Platform::new(&platform2_dir);
    let mut store2 = Store::open(&temp.0.join("control2")).unwrap();
    let repo2 = hosted_repo(&mut store2);
    let shared2: Shared = Arc::new(Mutex::new(Some(store2)));
    let id2 = submit(
        &shared2,
        &platform2,
        &repo2,
        "two",
        Form::AcceptAdvance,
        Strategy::MergeCommit,
    );
    platform2.set("pr_state", "merged");
    platform2.set("merge_sha", "mmmm7777");
    platform2.set("head", "mmmm7777");
    drive_with(&shared2, &repo2, &id2, &mut platform2.connect()).unwrap();
    let done = shown(&shared2, &repo2, &id2);
    assert_eq!(done["intent"]["state"], "succeeded", "{done}");
    assert_eq!(done["receipt"]["readback"]["status"], "already_applied");
    assert_eq!(platform2.posts(), 0);
}

#[test]
fn a_review_request_whose_head_is_not_the_published_revision_is_not_merged() {
    let temp = temp("head");
    let platform = Platform::new(&temp.0);
    let mut store = Store::open(&temp.0.join("control")).unwrap();
    let repo_id = hosted_repo(&mut store);
    let shared: Shared = Arc::new(Mutex::new(Some(store)));
    let id = submit(
        &shared,
        &platform,
        &repo_id,
        "one",
        Form::AcceptAdvance,
        Strategy::MergeCommit,
    );
    // Someone pushed another commit to the review request's branch after publication.
    platform.set("pr_head", "head9999");
    drive_with(&shared, &repo_id, &id, &mut platform.connect()).unwrap();
    let failed = shown(&shared, &repo_id, &id);
    assert_eq!(failed["intent"]["state"], "failed", "{failed}");
    assert_eq!(failed["intent"]["failure"]["code"], "SOURCE_HEAD_MISMATCH");
    assert_eq!(platform.posts(), 0);
    assert_eq!(platform.read("pr_state"), "open");
}
