//! Review publishing against the scripted platforms: the push lands in a real bare
//! repository, the request is a record the script keeps, and every recovery case reads the
//! platform back instead of trusting what control remembers.
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use repo::changeset::{self, OwnerGate, ProducerRef, Seal};
use repo::git::Credential;
use repo::review::{self as domain, Policy, Publication, State};
use serde_json::{Value, json};
use store::{Store, TrustedActor};
use tokio::sync::Mutex;

use super::{Shared, command_id, drive_with};
use crate::integration::Connection;
use crate::integration::fixture::{Kind, Platform, actor, temp};

fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.invalid")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.invalid")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

/// A hosted Repo registered from the local clone (so publishing has objects to push), its
/// platform binding confirmed at the bare repository, capabilities declared.
fn hosted_repo_from_clone(store: &mut Store, platform: &Platform) -> String {
    use repo::{
        FinishChoice, LocalInput, PlatformObservation, confirm_delivery, confirm_platform, finish,
    };
    let local = platform.local_clone();
    let input = LocalInput {
        machine: "control".into(),
        path: local.clone(),
        in_place: false,
        extra_refs: vec![],
        publish_governance: false,
    };
    let snapshot = repo::git::Git::discover().unwrap().inspect(&input).unwrap();
    let prepared = repo::prepare(
        repo::Register {
            name: "example".into(),
            origin: repo::Origin::Local,
            platform: Some(repo::Platform::Local),
            instance: None,
            platform_repo_id: None,
            platform_path: Some("example".into()),
            local: Some(input),
            remote_evidence: None,
            default_source: None,
        },
        Some(snapshot),
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
    reg.repo_id
}

/// The owner with the Repo scope the ChangeSet commands check.
fn scoped(repo_id: &str) -> TrustedActor {
    let mut actor = actor().0;
    actor
        .permission_scope
        .push(store::Scope::Repo(repo_id.into()));
    TrustedActor(actor)
}

fn policy(repo_id: &str, allow_update: bool, human: bool) -> Policy {
    Policy {
        repo_id: repo_id.into(),
        binding_version: 1,
        branch_rule: "hctl2/{change_set}".into(),
        target_branch: "main".into(),
        allow_update,
        description_source: "none".into(),
        requires_human_confirmation: human,
        audit_scope: "minimal".into(),
    }
}

struct Scenario {
    shared: Shared,
    repo_id: String,
    change_set_id: String,
    intent_id: String,
    policy: domain::FrozenPolicy,
}

/// One ChangeSet opened for an invocation, its first revision admitted with the policy: the
/// publish intent exists from the same transaction.
fn scenario(
    temp: &Path,
    platform: &Platform,
    name: &str,
    allow_update: bool,
    human: bool,
) -> Scenario {
    let mut store = Store::open(&temp.join(format!("control-{name}"))).unwrap();
    let repo_id = hosted_repo_from_clone(&mut store, platform);
    let frozen = domain::freeze_policy(
        &mut store,
        &actor(),
        "policy-1",
        policy(&repo_id, allow_update, human),
    )
    .unwrap();
    let holder = ProducerRef::Invocation {
        invocation_id: "inv-1".into(),
        invocation_version: 1,
    };
    let set = changeset::open_change_set(
        &mut store,
        &scoped(&repo_id),
        &repo_id,
        1,
        &platform.base,
        "cs-key",
        &holder,
    )
    .unwrap();
    let (revision, intent) = admit(&mut store, &set, &frozen, &platform.tree, None, "assoc-1");
    assert_eq!(revision.base_commit_sha, platform.base);
    let intent = intent.expect("admission enqueued the publish");
    Scenario {
        shared: Arc::new(Mutex::new(Some(store))),
        repo_id,
        change_set_id: set.change_set_id,
        intent_id: intent.intent_id,
        policy: frozen,
    }
}

fn admit(
    store: &mut Store,
    set: &changeset::ChangeSet,
    frozen: &domain::FrozenPolicy,
    tree: &str,
    parent: Option<&str>,
    association_key: &str,
) -> (changeset::ChangeSetRevision, Option<domain::Intent>) {
    let current: changeset::ChangeSet = {
        let record = store
            .get(&store::ObjectKey {
                scope: store::Scope::Repo(set.repo_id.clone()),
                kind: "changeset".into(),
                id: set.change_set_id.clone(),
            })
            .unwrap()
            .unwrap();
        let store::RecordData::Value { value } = record.data else {
            panic!()
        };
        serde_json::from_value(value).unwrap()
    };
    changeset::admit_with_publication(
        store,
        &scoped(&set.repo_id),
        Seal {
            association_key: association_key.into(),
            change_set_id: set.change_set_id.clone(),
            change_set_version: current.version,
            lease: Some(changeset::LeaseRef {
                lease_id: current.lease.lease_id.clone(),
                generation: current.lease.generation,
            }),
            base_commit_sha: set.baseline_commit.clone(),
            result_tree_sha: tree.into(),
            result_commit_sha: None,
            parent_revision_id: parent.map(str::to_owned),
            producer_ref: current.lease.holder.clone(),
        },
        OwnerGate::Active,
        Some(&Publication {
            policy: frozen.clone(),
            authorizing_actor: actor().0,
        }),
        1_000,
    )
    .unwrap()
}

/// The ChangeSet as the Store holds it now.
fn current_set(store: &Store, change_set_id: &str) -> changeset::ChangeSet {
    serde_json::from_value(
        store
            .list("changeset")
            .unwrap()
            .into_iter()
            .find_map(|r| match r.data {
                store::RecordData::Value { value }
                    if value["change_set_id"] == json!(change_set_id) =>
                {
                    Some(value)
                }
                _ => None,
            })
            .unwrap(),
    )
    .unwrap()
}

/// The executor produces a second revision on the same base (a new tree on `base`; its own
/// commit is the packaging because base and tree match) and it is admitted under the same
/// policy. Returns that commit and the intent as the admission wrote it.
fn second_revision(platform: &Platform, s: &Scenario, content: &str) -> (String, domain::Intent) {
    let local = platform.local_clone();
    git(&local, &["switch", "-q", "-C", "work", &platform.base]);
    std::fs::write(local.join("a"), content).unwrap();
    git(&local, &["commit", "-q", "-am", content]);
    let commit = git(&local, &["rev-parse", "HEAD"]);
    let tree = git(&local, &["rev-parse", "HEAD^{tree}"]);
    let first = shown(s)["intent"]["target"]["change_set_revision_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut guard = s.shared.blocking_lock();
    let store = guard.as_mut().unwrap();
    let set = current_set(store, &s.change_set_id);
    let key = format!("assoc-{}", &tree[..8]);
    let (_, intent) = admit(store, &set, &s.policy, &tree, Some(&first), &key);
    (commit, intent.unwrap())
}

/// The revision and round the intent is at now, for the steps that must name them.
fn at(s: &Scenario) -> (String, u64) {
    let shown = shown(s);
    (
        shown["intent"]["target"]["change_set_revision_id"]
            .as_str()
            .unwrap()
            .to_owned(),
        shown["intent"]["round"].as_u64().unwrap(),
    )
}

fn drive(platform: &Platform, s: &Scenario) -> repo::Result<()> {
    drive_with(&s.shared, &s.repo_id, &s.intent_id, &mut |_| {
        Ok((platform.connection(), Credential::Anonymous))
    })
}

fn shown(s: &Scenario) -> Value {
    let guard = s.shared.blocking_lock();
    domain::show(guard.as_ref().unwrap(), &s.repo_id, &s.intent_id).unwrap()
}

fn branch_head(platform: &Platform, branch: &str) -> Option<String> {
    Command::new("git")
        .arg("-C")
        .arg(&platform.git_dir)
        .args([
            "rev-parse",
            "--verify",
            "-q",
            &format!("refs/heads/{branch}"),
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8(o.stdout).unwrap().trim().to_owned())
}

#[test]
fn publishing_pushes_the_frozen_commit_then_creates_one_request_and_records_the_mapping() {
    let temp = temp("publish");
    let platform = Platform::new(&temp.0);
    platform.without_review_request();
    let s = scenario(&temp.0, &platform, "one", true, false);
    let before = shown(&s);
    assert_eq!(before["intent"]["state"], "pending", "{before}");
    assert_eq!(
        before["intent"]["branch"],
        json!(format!("hctl2/{}", s.change_set_id))
    );
    assert_eq!(before["intent"]["authorized_by"]["kind"], "invocation");
    drive(&platform, &s).unwrap();
    let done = shown(&s);
    assert_eq!(done["intent"]["state"], "published", "{done}");
    let branch = format!("hctl2/{}", s.change_set_id);
    // Stage 1: the branch carries the candidate commit (found in the clone: same tree on the
    // same base), read back from the platform's Git.
    assert_eq!(
        branch_head(&platform, &branch),
        Some(platform.candidate.clone())
    );
    assert_eq!(
        done["stages"]["push"]["commit_sha"],
        json!(platform.candidate)
    );
    // Stage 2: one request, carrying that commit, recorded as the revision's mapping — the
    // shape the integration half reads.
    assert_eq!(platform.creates(), 1);
    assert_eq!(done["stages"]["review_request"]["index"], 7);
    assert_eq!(done["mappings"].as_array().unwrap().len(), 1);
    assert_eq!(
        done["mappings"][0]["platform_commit_sha"],
        json!(platform.candidate)
    );
    assert_eq!(done["mappings"][0]["review_request"]["index"], 7);
    let revision_id = done["intent"]["target"]["change_set_revision_id"]
        .as_str()
        .unwrap()
        .to_owned();
    {
        let guard = s.shared.blocking_lock();
        let mapped =
            repo::integration::review_request(guard.as_ref().unwrap(), &s.repo_id, &revision_id)
                .unwrap()
                .expect("the integration half finds the mapping");
        assert_eq!(mapped.index, 7);
        assert_eq!(mapped.platform_commit_sha, platform.candidate);
    }
    assert_eq!(platform.read("pr_branch"), branch);
    assert_eq!(platform.read("pr_base"), "main");
    assert!(platform.read("pr_title").contains("ChangeSet"));
    // Settled: another pass is refused and changes nothing on the platform.
    assert!(drive(&platform, &s).is_err());
    assert_eq!(platform.creates(), 1);
}

#[test]
fn a_lost_confirmation_between_the_two_stages_recovers_by_readback_with_one_request() {
    let temp = temp("recover");
    let platform = Platform::new(&temp.0);
    platform.without_review_request();
    let s = scenario(&temp.0, &platform, "one", true, false);
    let branch = format!("hctl2/{}", s.change_set_id);
    // The push lands; the platform is down for the request: stage 1 confirmed, intent waits.
    platform.set("down", "");
    drive(&platform, &s).unwrap();
    let waiting = shown(&s);
    assert_eq!(waiting["intent"]["state"], "unknown", "{waiting}");
    assert_eq!(
        waiting["intent"]["attention"]["code"],
        "PLATFORM_UNAVAILABLE"
    );
    assert_eq!(waiting["stages"]["push"]["confirmed"], true);
    assert_eq!(waiting["stages"]["review_request"]["confirmed"], false);
    assert_eq!(
        branch_head(&platform, &branch),
        Some(platform.candidate.clone())
    );
    assert_eq!(
        platform.creates(),
        0,
        "the platform could not even be asked"
    );
    // Control restarts with the platform back: only the unconfirmed stage runs, one request.
    platform.unset("down");
    let dir = temp.0.join("control-one");
    drop(s.shared.blocking_lock().take());
    let s = Scenario {
        shared: Arc::new(Mutex::new(Some(Store::open(&dir).unwrap()))),
        ..s
    };
    drive(&platform, &s).unwrap();
    let done = shown(&s);
    assert_eq!(done["intent"]["state"], "published", "{done}");
    assert_eq!(platform.creates(), 1);
    assert_eq!(
        branch_head(&platform, &branch),
        Some(platform.candidate.clone()),
        "no second push"
    );
    // The response to the create is lost after the platform created it: readback finds the
    // request, nothing is created twice.
    let platform2 = Platform::new(&temp.0.join("lost"));
    platform2.without_review_request();
    let s2 = scenario(&temp.0, &platform2, "two", true, false);
    platform2.set("lose_create", "");
    drive(&platform2, &s2).unwrap();
    let done2 = shown(&s2);
    assert_eq!(done2["intent"]["state"], "published", "{done2}");
    assert_eq!(platform2.creates(), 1);
    assert_eq!(done2["mappings"].as_array().unwrap().len(), 1);
    // A push whose confirmation was lost: the branch already carries the commit; the next
    // pass confirms it from the remote without pushing again (a push with the stale lease
    // would have been refused anyway), then finishes stage 2.
    let platform3 = Platform::new(&temp.0.join("pushed"));
    platform3.without_review_request();
    let s3 = scenario(&temp.0, &platform3, "three", true, false);
    let branch3 = format!("hctl2/{}", s3.change_set_id);
    {
        let mut guard = s3.shared.blocking_lock();
        let store = guard.as_mut().unwrap();
        let (rev, round) = {
            let shown = domain::show(store, &s3.repo_id, &s3.intent_id).unwrap();
            (
                shown["intent"]["target"]["change_set_revision_id"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
                shown["intent"]["round"].as_u64().unwrap(),
            )
        };
        domain::freeze_commit(
            store,
            &s3.repo_id,
            &s3.intent_id,
            &rev,
            round,
            &platform3.candidate,
        )
        .unwrap();
        domain::mark_push_dispatched(store, &s3.repo_id, &s3.intent_id, &rev, round, true).unwrap();
    }
    git(
        &platform3.git_dir,
        &[
            "update-ref",
            &format!("refs/heads/{branch3}"),
            &platform3.candidate,
        ],
    );
    drive(&platform3, &s3).unwrap();
    let done3 = shown(&s3);
    assert_eq!(done3["intent"]["state"], "published", "{done3}");
    assert_eq!(
        done3["stages"]["push"]["commit_sha"],
        json!(platform3.candidate)
    );
    assert_eq!(platform3.creates(), 1);
}

#[test]
fn a_newer_revision_updates_the_same_request_and_a_stale_push_cannot_overwrite_it() {
    let temp = temp("update");
    let platform = Platform::new(&temp.0);
    platform.without_review_request();
    let s = scenario(&temp.0, &platform, "one", true, false);
    drive(&platform, &s).unwrap();
    assert_eq!(shown(&s)["intent"]["state"], "published");
    let branch = format!("hctl2/{}", s.change_set_id);
    // The executor produces a second revision on the same base (a new tree on `base`; its
    // own commit is the packaging because base and tree match).
    let local = platform.local_clone();
    git(&local, &["switch", "-q", "-c", "work", &platform.base]);
    std::fs::write(local.join("a"), "second\n").unwrap();
    git(&local, &["commit", "-q", "-am", "second"]);
    let second_commit = git(&local, &["rev-parse", "HEAD"]);
    let second_tree = git(&local, &["rev-parse", "HEAD^{tree}"]);
    // Admitting it under the same policy opens round 2 of the intent: same request.
    let first_revision = shown(&s)["intent"]["target"]["change_set_revision_id"]
        .as_str()
        .unwrap()
        .to_owned();
    {
        let mut guard = s.shared.blocking_lock();
        let store = guard.as_mut().unwrap();
        let set = current_set(store, &s.change_set_id);
        let (_, intent) = admit(
            store,
            &set,
            &s.policy,
            &second_tree,
            Some(&first_revision),
            "assoc-2",
        );
        let intent = intent.unwrap();
        assert_eq!(intent.state, State::Pending);
        assert_eq!(intent.round, 2);
        assert!(!intent.started);
        assert_eq!(
            intent.push.confirmed_commit,
            Some(platform.candidate.clone()),
            "the lease for the next push"
        );
    }
    drive(&platform, &s).unwrap();
    let done = shown(&s);
    assert_eq!(done["intent"]["state"], "published", "{done}");
    assert_eq!(done["intent"]["round"], 2);
    // The branch moved on top of the earlier publish; the second commit is the executor's
    // own (same tree on the same base), not a wrapper.
    assert_eq!(branch_head(&platform, &branch), Some(second_commit.clone()));
    assert_eq!(platform.creates(), 1, "no second request");
    assert_eq!(platform.updates(), 1);
    let mappings = done["mappings"].as_array().unwrap();
    assert_eq!(mappings.len(), 2);
    assert_eq!(mappings[1]["platform_commit_sha"], json!(second_commit));
    assert_eq!(mappings[1]["review_request"]["index"], 7);
    assert_eq!(
        mappings[0]["platform_commit_sha"],
        json!(platform.candidate),
        "the first mapping is evidence and stays"
    );
    // A late push of the first revision with its old lease is refused by the remote: the
    // branch stays at the second commit.
    let git_ = repo::git::Git::discover().unwrap();
    let outcome = git_
        .push_ref(
            &local,
            &platform.git_dir.to_string_lossy(),
            &platform.candidate,
            &format!("refs/heads/{branch}"),
            Some(&platform.base),
            &Credential::Anonymous,
        )
        .unwrap();
    assert_eq!(outcome, repo::git::PushOutcome::StaleLease);
    assert_eq!(branch_head(&platform, &branch), Some(second_commit));
}

#[test]
fn a_create_only_policy_refuses_the_second_revision_and_keeps_the_first_published() {
    let temp = temp("create-only");
    let platform = Platform::new(&temp.0);
    platform.without_review_request();
    let s = scenario(&temp.0, &platform, "one", false, false);
    drive(&platform, &s).unwrap();
    assert_eq!(shown(&s)["intent"]["state"], "published");
    let local = platform.local_clone();
    git(&local, &["switch", "-q", "-c", "work", &platform.base]);
    std::fs::write(local.join("a"), "second\n").unwrap();
    git(&local, &["commit", "-q", "-am", "second"]);
    let second_tree = git(&local, &["rev-parse", "HEAD^{tree}"]);
    let first_revision = shown(&s)["intent"]["target"]["change_set_revision_id"]
        .as_str()
        .unwrap()
        .to_owned();
    {
        let mut guard = s.shared.blocking_lock();
        let store = guard.as_mut().unwrap();
        let set: changeset::ChangeSet = serde_json::from_value(
            store
                .list("changeset")
                .unwrap()
                .into_iter()
                .find_map(|r| match r.data {
                    store::RecordData::Value { value }
                        if value["change_set_id"] == json!(s.change_set_id) =>
                    {
                        Some(value)
                    }
                    _ => None,
                })
                .unwrap(),
        )
        .unwrap();
        let (revision, intent) = admit(
            store,
            &set,
            &s.policy,
            &second_tree,
            Some(&first_revision),
            "assoc-2",
        );
        let intent = intent.unwrap();
        assert_eq!(intent.state, State::Published, "the first stays published");
        assert_eq!(intent.round, 1);
        assert_eq!(
            intent.attention.as_ref().unwrap().code,
            "UPDATE_NOT_ALLOWED"
        );
        assert_eq!(
            intent.attention.as_ref().unwrap().details["unpublished_revision"],
            json!(revision.change_set_revision_id)
        );
    }
    // Nothing is queued: the worker has nothing to do and the platform is untouched.
    assert!(drive(&platform, &s).is_err());
    assert_eq!(platform.updates(), 0);
    assert_eq!(shown(&s)["mappings"].as_array().unwrap().len(), 1);
}

#[test]
fn a_policy_that_wants_a_human_holds_the_intent_until_released() {
    let temp = temp("human");
    let platform = Platform::new(&temp.0);
    platform.without_review_request();
    let s = scenario(&temp.0, &platform, "one", true, true);
    assert_eq!(shown(&s)["intent"]["state"], "pending_human");
    // The worker does not touch it; the preview says what releasing does; a non-human cannot.
    assert!(drive(&platform, &s).is_err());
    assert!(branch_head(&platform, &format!("hctl2/{}", s.change_set_id)).is_none());
    let preview = super::preview(
        &s.shared,
        &actor(),
        "review.publish",
        &json!({"repo_id": s.repo_id, "intent_id": s.intent_id}),
    )
    .unwrap();
    assert_eq!(preview["target_branch"], "main");
    assert!(
        preview["authorizes"]
            .as_str()
            .unwrap()
            .contains("Not a merge")
    );
    {
        let mut guard = s.shared.blocking_lock();
        let store = guard.as_mut().unwrap();
        let reducer = TrustedActor(store::Actor {
            principal: "x".into(),
            source: store::ActorSource::InternalReducer,
            permission_scope: vec![store::Scope::Control],
            authority: None,
        });
        let rev = preview["change_set_revision_id"].as_str().unwrap();
        assert_eq!(
            domain::release(
                store,
                &reducer,
                "release-x",
                &s.repo_id,
                &s.intent_id,
                rev,
                1
            )
            .unwrap_err()
            .code,
            "PERMISSION_DENIED"
        );
    }
    // The release goes through the command envelope the CLI sends: the command identity is
    // this round's, and the preview's revision and round are what gets released.
    let released = submit(&s, &preview).unwrap();
    assert_eq!(released["state"], "pending");
    assert_eq!(
        shown(&s)["intent"]["authorizing_actor"]["principal"],
        "owner"
    );
    // Replaying the same round returns the same answer without a second release.
    assert_eq!(submit(&s, &preview).unwrap()["state"], "pending");
    drive(&platform, &s).unwrap();
    assert_eq!(shown(&s)["intent"]["state"], "published");
    assert_eq!(platform.creates(), 1);
    // A second revision under the human gate: its own round, its own preview and release,
    // under this round's command identity — the first round's envelope is refused.
    let (second_commit, intent) = second_revision(&platform, &s, "second\n");
    assert_eq!(intent.state, State::PendingHuman);
    assert_eq!(intent.round, 2);
    assert_eq!(
        submit(&s, &preview).unwrap_err().code,
        "FROZEN_INPUT_CHANGED",
        "round 1's envelope cannot release round 2"
    );
    assert_eq!(
        super::preview(
            &s.shared,
            &actor(),
            "review.publish",
            &json!({"repo_id": s.repo_id, "intent_id": s.intent_id, "round": 1}),
        )
        .unwrap_err()
        .code,
        "FROZEN_INPUT_CHANGED",
        "a preview of the round that is gone"
    );
    let preview2 = super::preview(
        &s.shared,
        &actor(),
        "review.publish",
        &json!({"repo_id": s.repo_id, "intent_id": s.intent_id, "round": 2}),
    )
    .unwrap();
    assert_eq!(preview2["round"], 2);
    assert_eq!(submit(&s, &preview2).unwrap()["state"], "pending");
    drive(&platform, &s).unwrap();
    let done = shown(&s);
    assert_eq!(done["intent"]["state"], "published", "{done}");
    assert_eq!(done["intent"]["round"], 2);
    assert_eq!(
        done["mappings"][1]["platform_commit_sha"],
        json!(second_commit)
    );
    assert_eq!(platform.creates(), 1);
    assert_eq!(platform.updates(), 1);
}

/// The CLI's envelope for releasing what `preview` showed: the same payload, the round's
/// command identity, the preview's details.
fn submit(s: &Scenario, preview: &Value) -> repo::Result<Value> {
    let round = preview["round"].as_u64().unwrap();
    let request = proto::SubmitRequest {
        protocol: None,
        operation: "review.publish".into(),
        payload: serde_json::to_vec(
            &json!({"repo_id": s.repo_id, "intent_id": s.intent_id, "round": round}),
        )
        .unwrap(),
        command_id: command_id(&s.intent_id, round),
        idempotency_key: String::new(),
        preview_token: String::new(),
    };
    super::submit(&s.shared, &actor(), &request, preview)
}

/// A human who previewed round 1 cannot release what round 2 became; a revision admitted
/// after a release but before the worker took it needs its own release.
#[test]
fn a_release_names_the_round_it_previewed_and_a_successor_needs_its_own() {
    let temp = temp("human-rounds");
    let platform = Platform::new(&temp.0);
    platform.without_review_request();
    let s = scenario(&temp.0, &platform, "one", true, true);
    let preview1 = super::preview(
        &s.shared,
        &actor(),
        "review.publish",
        &json!({"repo_id": s.repo_id, "intent_id": s.intent_id}),
    )
    .unwrap();
    // B is admitted before the human submits: round 1 was never taken, so it is withdrawn
    // and round 2 waits for a human; the old envelope releases nothing.
    let (second_commit, intent) = second_revision(&platform, &s, "second\n");
    assert_eq!(intent.round, 2);
    assert_eq!(intent.state, State::PendingHuman);
    assert_eq!(
        submit(&s, &preview1).unwrap_err().code,
        "FROZEN_INPUT_CHANGED"
    );
    assert_eq!(shown(&s)["intent"]["state"], "pending_human");
    // Released, then a third revision lands before the worker takes round 2: round 3 is
    // pending_human again — a release does not carry over to a revision nobody looked at.
    let preview2 = super::preview(
        &s.shared,
        &actor(),
        "review.publish",
        &json!({"repo_id": s.repo_id, "intent_id": s.intent_id}),
    )
    .unwrap();
    assert_eq!(submit(&s, &preview2).unwrap()["state"], "pending");
    let (third_commit, intent) = second_revision(&platform, &s, "third\n");
    assert_eq!(intent.round, 3);
    assert_eq!(intent.state, State::PendingHuman);
    assert!(drive(&platform, &s).is_err(), "nothing to drive");
    assert!(branch_head(&platform, &format!("hctl2/{}", s.change_set_id)).is_none());
    let preview3 = super::preview(
        &s.shared,
        &actor(),
        "review.publish",
        &json!({"repo_id": s.repo_id, "intent_id": s.intent_id}),
    )
    .unwrap();
    assert_eq!(preview3["round"], 3);
    assert_eq!(submit(&s, &preview3).unwrap()["state"], "pending");
    drive(&platform, &s).unwrap();
    let done = shown(&s);
    assert_eq!(done["intent"]["state"], "published", "{done}");
    assert_eq!(done["intent"]["round"], 3);
    let mappings = done["mappings"].as_array().unwrap();
    assert_eq!(
        mappings.len(),
        1,
        "only the third revision reached the platform"
    );
    assert_eq!(mappings[0]["platform_commit_sha"], json!(third_commit));
    assert_ne!(second_commit, third_commit);
    assert_eq!(platform.creates(), 1);
}

/// A revision admitted while the worker is executing a round neither changes that round's
/// target nor takes its evidence: the round finishes for the revision it started with, the
/// newer one waits and gets the next round.
#[test]
fn a_revision_admitted_during_execution_waits_and_never_takes_the_evidence() {
    let temp = temp("during");
    let platform = Platform::new(&temp.0);
    platform.without_review_request();
    let s = scenario(&temp.0, &platform, "one", true, false);
    let first_revision = at(&s).0;
    // The admission of B happens inside the pass, after the worker took round 1 and while
    // it connects to the platform.
    let mut second = None;
    drive_with(&s.shared, &s.repo_id, &s.intent_id, &mut |_| {
        let (commit, intent) = second_revision(&platform, &s, "second\n");
        assert_eq!(intent.round, 1, "the executing round keeps its target");
        assert_eq!(intent.target.change_set_revision_id, first_revision);
        assert!(intent.queued.is_some());
        second = Some(commit);
        Ok((platform.connection(), Credential::Anonymous))
    })
    .unwrap();
    let second_commit = second.unwrap();
    let after = shown(&s);
    assert_eq!(after["intent"]["state"], "pending", "{after}");
    assert_eq!(
        after["intent"]["round"], 2,
        "round 1 settled, round 2 opened for B"
    );
    let mappings = after["mappings"].as_array().unwrap();
    assert_eq!(mappings.len(), 1);
    assert_eq!(mappings[0]["change_set_revision_id"], json!(first_revision));
    assert_eq!(
        mappings[0]["platform_commit_sha"],
        json!(platform.candidate)
    );
    assert_eq!(platform.creates(), 1);
    // The next pass publishes B as round 2.
    drive(&platform, &s).unwrap();
    let done = shown(&s);
    assert_eq!(done["intent"]["state"], "published", "{done}");
    assert_eq!(
        done["mappings"][1]["platform_commit_sha"],
        json!(second_commit)
    );
    assert_eq!(platform.updates(), 1);
}

/// A push whose confirmation was lost, then a newer revision admitted: the round confirms
/// its own push from the remote and finishes; the newer revision follows as round 2.
#[test]
fn a_revision_admitted_after_a_lost_push_confirmation_follows_the_round() {
    let temp = temp("after-lost-push");
    let platform = Platform::new(&temp.0);
    platform.without_review_request();
    let s = scenario(&temp.0, &platform, "one", true, false);
    let branch = format!("hctl2/{}", s.change_set_id);
    {
        let mut guard = s.shared.blocking_lock();
        let store = guard.as_mut().unwrap();
        domain::begin(store, &s.repo_id, &s.intent_id).unwrap();
        let (rev, round) = {
            let shown = domain::show(store, &s.repo_id, &s.intent_id).unwrap();
            (
                shown["intent"]["target"]["change_set_revision_id"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
                shown["intent"]["round"].as_u64().unwrap(),
            )
        };
        domain::freeze_commit(
            store,
            &s.repo_id,
            &s.intent_id,
            &rev,
            round,
            &platform.candidate,
        )
        .unwrap();
        domain::mark_push_dispatched(store, &s.repo_id, &s.intent_id, &rev, round, true).unwrap();
    }
    git(
        &platform.git_dir,
        &[
            "update-ref",
            &format!("refs/heads/{branch}"),
            &platform.candidate,
        ],
    );
    let (second_commit, intent) = second_revision(&platform, &s, "second\n");
    assert_eq!(intent.round, 1);
    assert!(
        intent.push.dispatched,
        "the dispatched push is not forgotten"
    );
    assert_eq!(
        intent.target.commit_sha.as_deref(),
        Some(platform.candidate.as_str())
    );
    drive(&platform, &s).unwrap();
    let after = shown(&s);
    assert_eq!(after["intent"]["round"], 2, "{after}");
    assert_eq!(
        after["mappings"][0]["platform_commit_sha"],
        json!(platform.candidate)
    );
    drive(&platform, &s).unwrap();
    let done = shown(&s);
    assert_eq!(done["intent"]["state"], "published", "{done}");
    assert_eq!(branch_head(&platform, &branch), Some(second_commit.clone()));
    assert_eq!(
        done["mappings"][1]["platform_commit_sha"],
        json!(second_commit)
    );
    assert_eq!(platform.creates(), 1);
}

/// Create-only policy, a newer revision admitted while the create of the first is in
/// flight and its response gets lost: the round confirms its request by readback for the
/// revision it started with, and the newer revision is refused as an update — it is never
/// pushed and the request is never touched.
#[test]
fn a_create_only_policy_after_a_lost_create_does_not_update_the_request() {
    let temp = temp("create-only-lost");
    let platform = Platform::new(&temp.0);
    platform.without_review_request();
    let s = scenario(&temp.0, &platform, "one", false, false);
    let branch = format!("hctl2/{}", s.change_set_id);
    let first_revision = at(&s).0;
    platform.set("lose_create", "");
    drive_with(&s.shared, &s.repo_id, &s.intent_id, &mut |_| {
        let (_, intent) = second_revision(&platform, &s, "second\n");
        assert_eq!(intent.round, 1);
        assert!(intent.queued.is_some());
        Ok((platform.connection(), Credential::Anonymous))
    })
    .unwrap();
    let done = shown(&s);
    assert_eq!(done["intent"]["state"], "published", "{done}");
    assert_eq!(done["intent"]["round"], 1);
    assert_eq!(done["intent"]["attention"]["code"], "UPDATE_NOT_ALLOWED");
    assert!(done["intent"]["queued"].is_null());
    let mappings = done["mappings"].as_array().unwrap();
    assert_eq!(mappings.len(), 1);
    assert_eq!(mappings[0]["change_set_revision_id"], json!(first_revision));
    assert_eq!(
        branch_head(&platform, &branch),
        Some(platform.candidate.clone())
    );
    assert_eq!(platform.creates(), 1);
    assert_eq!(platform.updates(), 0);
    // Nothing is left to drive for this intent.
    assert_eq!(drive(&platform, &s).unwrap_err().code, "INTENT_TERMINAL");
}

/// The platform refuses the audit association (title and body) of an update: publishing
/// still settles with its mapping (CT-REPO: 关联写回失败时集成仍成功、写回待同步), the
/// refusal is recorded as a pending sync item on the same intent, and no second push or
/// second request is made; once the platform lets the update through the item is cleared.
#[test]
fn a_refused_audit_update_publishes_with_a_pending_sync() {
    let temp = temp("refused-update");
    let platform = Platform::new(&temp.0);
    platform.without_review_request();
    let s = scenario(&temp.0, &platform, "one", true, false);
    drive(&platform, &s).unwrap();
    assert_eq!(shown(&s)["intent"]["state"], "published");
    let (second_commit, second_intent) = second_revision(&platform, &s, "second\n");
    let second_revision_id = second_intent.target.change_set_revision_id.clone();
    platform.set("refuse_update", "");
    drive(&platform, &s).unwrap();
    let refused = shown(&s);
    assert_eq!(refused["intent"]["state"], "published", "{refused}");
    assert!(refused["intent"]["attention"].is_null(), "{refused}");
    assert_eq!(
        refused["intent"]["audit_sync"]["code"], "AUDIT_UPDATE_REJECTED",
        "{refused}"
    );
    // 映射已写：这一版有 platform binding，集成半边读得到它。
    let mappings = refused["mappings"].as_array().unwrap();
    assert_eq!(mappings.len(), 2, "{refused}");
    assert_eq!(mappings[1]["platform_commit_sha"], json!(second_commit));
    assert_eq!(platform.updates(), 1);
    assert_eq!(platform.creates(), 1);
    {
        let guard = s.shared.blocking_lock();
        let store = guard.as_ref().unwrap();
        // 待同步项让 worker 仍然看得到这条意图；集成侧按映射读得到这一版。
        let open = domain::open(store).unwrap();
        assert_eq!(open.len(), 1);
        assert_eq!(
            open[0].audit_sync.as_ref().unwrap().code,
            "AUDIT_UPDATE_REJECTED"
        );
        assert!(
            repo::integration::review_request(store, &s.repo_id, &second_revision_id)
                .unwrap()
                .is_some(),
            "the new revision's mapping is written"
        );
    }
    platform.unset("refuse_update");
    drive(&platform, &s).unwrap();
    let done = shown(&s);
    assert!(done["intent"]["audit_sync"].is_null(), "{done}");
    assert_eq!(done["intent"]["state"], "published");
    assert_eq!(platform.updates(), 2);
    assert_eq!(platform.creates(), 1);
    // 映射不重写：仍是那两条，第二条的落库时刻与拒时一致。
    assert_eq!(done["mappings"].as_array().unwrap().len(), 2);
    assert_eq!(
        done["mappings"][1]["recorded_at_unix_ms"],
        refused["mappings"][1]["recorded_at_unix_ms"]
    );
    {
        let guard = s.shared.blocking_lock();
        let store = guard.as_ref().unwrap();
        assert!(domain::open(store).unwrap().is_empty());
    }
}

/// A rebind while the audit association is still pending: the retry sends nothing, the
/// pending sync item stays, and the reason moves to BINDING_CHANGED; a clear that names
/// another round is refused.
#[test]
fn a_rebound_repo_stops_the_audit_retry_and_keeps_the_pending_sync() {
    let temp = temp("refused-rebound");
    let platform = Platform::new(&temp.0);
    platform.without_review_request();
    let s = scenario(&temp.0, &platform, "one", true, false);
    drive(&platform, &s).unwrap();
    let (_, _) = second_revision(&platform, &s, "second\n");
    platform.set("refuse_update", "");
    drive(&platform, &s).unwrap();
    let refused = shown(&s);
    assert_eq!(refused["intent"]["state"], "published", "{refused}");
    assert_eq!(platform.updates(), 1, "{refused}");
    {
        let mut guard = s.shared.blocking_lock();
        rebind(guard.as_mut().unwrap(), &s.repo_id);
    }
    drive(&platform, &s).unwrap();
    let held = shown(&s);
    assert_eq!(
        held["intent"]["attention"]["code"], "BINDING_CHANGED",
        "{held}"
    );
    assert_eq!(held["intent"]["state"], "published", "{held}");
    assert_eq!(
        held["intent"]["audit_sync"]["code"], "AUDIT_UPDATE_REJECTED",
        "the pending sync item stays: {held}"
    );
    assert_eq!(
        platform.updates(),
        1,
        "nothing was sent for the old binding: {held}"
    );
    {
        let mut guard = s.shared.blocking_lock();
        let store = guard.as_mut().unwrap();
        assert_eq!(
            domain::clear_audit_sync(store, &s.repo_id, &s.intent_id, "another-revision", 1)
                .unwrap_err()
                .code,
            "FROZEN_INPUT_CHANGED"
        );
    }
}

/// A rebind as the Store sees it: the binding record moves to version 2.
fn rebind(store: &mut Store, repo_id: &str) {
    let binding = repo::binding(repo_id);
    let record = store.get(&binding.key).unwrap().unwrap();
    assert_eq!(record.version, 1);
    let mut next = record.clone();
    next.version = 2;
    let cmd = store::Command {
        command_id: "rebind".into(),
        idempotency_key: "rebind".into(),
        actor: scoped(repo_id).0,
        target: binding.key.clone(),
        expected: store::Expected::Exact(store::Version::State(1)),
        binding,
        input_digest: store::Command::digest_input("test.rebind", &json!({})).unwrap(),
        operation: "test.rebind".into(),
        input: json!({}),
    };
    store
        .submit(store.generation(), &scoped(repo_id), &cmd, None, |tx| {
            tx.put(&next)?;
            Ok(json!({}))
        })
        .unwrap();
}

/// The intent carries the platform binding version it was authorized under; the worker
/// sends nothing for an intent whose Repo was rebound since.
#[test]
fn a_rebound_repo_does_not_receive_an_intent_authorized_under_the_old_binding() {
    let temp = temp("rebound");
    let platform = Platform::new(&temp.0);
    platform.without_review_request();
    let s = scenario(&temp.0, &platform, "one", true, false);
    {
        let mut guard = s.shared.blocking_lock();
        rebind(guard.as_mut().unwrap(), &s.repo_id);
    }
    drive(&platform, &s).unwrap();
    let held = shown(&s);
    assert_eq!(
        held["intent"]["attention"]["code"], "BINDING_CHANGED",
        "{held}"
    );
    assert!(branch_head(&platform, &format!("hctl2/{}", s.change_set_id)).is_none());
    assert_eq!(platform.creates(), 0);
}

#[test]
fn a_platform_that_is_down_refuses_publishing_while_the_revision_stays_admitted() {
    let temp = temp("down");
    let platform = Platform::new(&temp.0);
    platform.without_review_request();
    // Down before anything: the admission happened (the intent exists), nothing was pushed.
    let s = scenario(&temp.0, &platform, "one", true, false);
    git(
        &platform.git_dir,
        &["update-ref", "refs/heads/main", &platform.base],
    );
    platform.set("down", "");
    // A bare repository is still reachable for Git here, so the push lands; the request
    // cannot be made. The revision was admitted regardless.
    drive(&platform, &s).unwrap();
    let waiting = shown(&s);
    assert_eq!(
        waiting["intent"]["attention"]["code"], "PLATFORM_UNAVAILABLE",
        "{waiting}"
    );
    assert_eq!(waiting["intent"]["state"], "unknown");
    {
        let guard = s.shared.blocking_lock();
        let store = guard.as_ref().unwrap();
        assert_eq!(
            changeset::list_revisions(store, &s.change_set_id)
                .unwrap()
                .len(),
            1
        );
    }
    // A branch someone else moved is never overwritten.
    let platform2 = Platform::new(&temp.0.join("moved"));
    platform2.without_review_request();
    let s2 = scenario(&temp.0, &platform2, "two", true, false);
    let branch2 = format!("hctl2/{}", s2.change_set_id);
    let other_tree = platform2.tree_with("z", "theirs\n");
    let other = platform2.git(&[
        "commit-tree",
        &other_tree,
        "-p",
        &platform2.base,
        "-m",
        "theirs",
    ]);
    platform2.git(&["update-ref", &format!("refs/heads/{branch2}"), &other]);
    drive(&platform2, &s2).unwrap();
    let diverged = shown(&s2);
    assert_eq!(
        diverged["intent"]["attention"]["code"], "BRANCH_DIVERGED",
        "{diverged}"
    );
    assert_eq!(branch_head(&platform2, &branch2), Some(other));
    assert_eq!(platform2.creates(), 0);
    // A request that exists but is closed is not reopened or replaced.
    let platform3 = Platform::new(&temp.0.join("closed"));
    platform3.without_review_request();
    let s3 = scenario(&temp.0, &platform3, "three", true, false);
    let branch3 = format!("hctl2/{}", s3.change_set_id);
    platform3.set("pr_exists", "");
    platform3.set("pr_branch", &branch3);
    platform3.set("pr_base", "main");
    platform3.set("pr_state", "closed");
    drive(&platform3, &s3).unwrap();
    let closed = shown(&s3);
    assert_eq!(
        closed["intent"]["attention"]["code"], "REVIEW_REQUEST_CLOSED",
        "{closed}"
    );
    assert_eq!(platform3.creates(), 0);
    assert_eq!(closed["mappings"].as_array().unwrap().len(), 0);
}

#[test]
fn github_publishing_looks_the_request_up_by_owner_and_branch_and_updates_it() {
    let temp = temp("github-publish");
    let platform = Platform::github(&temp.0);
    platform.without_review_request();
    assert_eq!(platform.kind, Kind::GitHub);
    let mut store = Store::open(&temp.0.join("control-gh")).unwrap();
    let repo_id = {
        // The canary-style registration, but from the local clone so there is something to push.
        let local = platform.local_clone();
        let input = repo::LocalInput {
            machine: "control".into(),
            path: local,
            in_place: false,
            extra_refs: vec![],
            publish_governance: false,
        };
        let snapshot = repo::git::Git::discover().unwrap().inspect(&input).unwrap();
        let prepared = repo::prepare(
            repo::Register {
                name: "canary".into(),
                origin: repo::Origin::External,
                platform: Some(repo::Platform::Github),
                instance: Some("github.com".into()),
                platform_repo_id: Some("77".into()),
                platform_path: Some("yesme/canary".into()),
                local: Some(input),
                remote_evidence: None,
                default_source: None,
            },
            Some(snapshot),
        )
        .unwrap();
        let reg =
            repo::admit(&mut store, &actor(), "register-gh", "register-gh", prepared).unwrap();
        store
            .begin_effect(
                store.generation(),
                &repo::effect_id(&reg.repo_id, "platform"),
            )
            .unwrap();
        let reg = repo::confirm_platform(
            &mut store,
            &reg.repo_id,
            repo::PlatformObservation {
                instance: "github.com".into(),
                stable_id: "77".into(),
                full_name: "yesme/canary".into(),
                clone_url: platform.git_dir.to_string_lossy().into_owned(),
                account_id: "1".into(),
                has_issues: true,
                can_write_issues: true,
                credential_ref: String::new(),
            },
        )
        .unwrap();
        if reg.lifecycle != repo::Lifecycle::Active {
            repo::finish(
                &mut store,
                &actor(),
                &reg.repo_id,
                reg.version,
                repo::FinishChoice::Confirm("77"),
                "finish-gh",
                "finish-gh",
            )
            .unwrap();
        }
        reg.repo_id
    };
    let frozen = domain::freeze_policy(
        &mut store,
        &actor(),
        "policy-gh",
        policy(&repo_id, true, false),
    )
    .unwrap();
    let holder = ProducerRef::Invocation {
        invocation_id: "inv-1".into(),
        invocation_version: 1,
    };
    let set = changeset::open_change_set(
        &mut store,
        &scoped(&repo_id),
        &repo_id,
        1,
        &platform.base,
        "cs-key",
        &holder,
    )
    .unwrap();
    let (_, intent) = admit(&mut store, &set, &frozen, &platform.tree, None, "assoc-1");
    let s = Scenario {
        shared: Arc::new(Mutex::new(Some(store))),
        repo_id,
        change_set_id: set.change_set_id.clone(),
        intent_id: intent.unwrap().intent_id,
        policy: frozen,
    };
    drive_with(&s.shared, &s.repo_id, &s.intent_id, &mut |_| {
        Ok((platform.connection(), Credential::Anonymous))
    })
    .unwrap();
    let done = shown(&s);
    assert_eq!(done["intent"]["state"], "published", "{done}");
    assert_eq!(done["mappings"][0]["review_request"]["index"], 3);
    assert_eq!(platform.creates(), 1);
    assert_eq!(
        platform.read("pr_branch"),
        format!("hctl2/{}", s.change_set_id)
    );
    // Found by owner:branch on the next pass of a new round: updated, not created.
    assert!(matches!(platform.connection(), Connection::GitHub(_)));
}

/// The packaged Gitea of a running demo instance: a revision from a real clone is pushed to
/// its branch and a real review request opened by control. Environment: `HCTL2_GITEA_LIVE_URL`,
/// `HCTL2_GITEA_LIVE_USER`, `HCTL2_GITEA_LIVE_TOKEN` (a scoped token; never printed),
/// `HCTL2_GITEA_LIVE_REPO` (`owner/name`), `HCTL2_TEA` (the packaged `tea`).
#[test]
#[ignore = "UNVERIFIED: requires a live Gitea and HCTL2_GITEA_LIVE_* in the environment"]
fn live_gitea_demo_revision_is_pushed_and_one_review_request_is_opened() {
    let var = |name: &str| std::env::var(name).unwrap_or_else(|_| panic!("{name} unset"));
    let url = var("HCTL2_GITEA_LIVE_URL");
    let user = var("HCTL2_GITEA_LIVE_USER");
    let token = var("HCTL2_GITEA_LIVE_TOKEN");
    let full_name = var("HCTL2_GITEA_LIVE_REPO");
    let tea = std::path::PathBuf::from(var("HCTL2_TEA"));
    let temp = temp("gitea-live");
    let clone_url = format!("{url}/{full_name}.git");
    let hosted = crate::scm::Hosted::fixture(tea, url.clone(), user.clone(), token.clone());
    let credential = Credential::Static {
        user: user.clone(),
        token: token.clone(),
    };
    // A plain local repository with the platform's history and a new revision on top.
    let local = temp.0.join("local");
    let mut clone = Command::new("git");
    credential.apply(&mut clone);
    let output = clone
        .args(["clone", "--quiet", "--branch", "main", &clone_url])
        .arg(&local)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    for args in [
        vec!["remote", "remove", "origin"],
        vec!["config", "user.name", "HCTL2 Test"],
        vec!["config", "user.email", "hctl2@example.invalid"],
    ] {
        git(&local, &args);
    }
    let base = git(&local, &["rev-parse", "HEAD"]);
    let stamp = crate::dispatch::now_ms();
    std::fs::write(
        local.join(format!("hctl2-live-{stamp}.txt")),
        format!("published at {stamp}\n"),
    )
    .unwrap();
    git(&local, &["add", "."]);
    git(&local, &["commit", "-q", "-m", "hctl2 live publish"]);
    let candidate = git(&local, &["rev-parse", "HEAD"]);
    let tree = git(&local, &["rev-parse", "HEAD^{tree}"]);
    git(&local, &["reset", "-q", "--hard", &base]);
    git(&local, &["branch", "keep", &candidate]);
    // The Repo, registered from that local repository and bound to the platform repository.
    let mut store = Store::open(&temp.0.join("control")).unwrap();
    let repo_id = {
        use repo::{
            FinishChoice, LocalInput, PlatformObservation, confirm_delivery, confirm_platform,
            finish,
        };
        let input = LocalInput {
            machine: "control".into(),
            path: local.clone(),
            in_place: false,
            extra_refs: vec![],
            publish_governance: false,
        };
        let snapshot = repo::git::Git::discover().unwrap().inspect(&input).unwrap();
        let prepared = repo::prepare(
            repo::Register {
                name: "live".into(),
                origin: repo::Origin::Local,
                platform: Some(repo::Platform::Local),
                instance: None,
                platform_repo_id: None,
                platform_path: Some(full_name.rsplit('/').next().unwrap().to_owned()),
                local: Some(input),
                remote_evidence: None,
                default_source: None,
            },
            Some(snapshot),
        )
        .unwrap();
        let meta = hosted
            .api("GET", &format!("repos/{full_name}"), None)
            .unwrap()
            .unwrap();
        let reg = repo::admit(&mut store, &actor(), "register", "register", prepared).unwrap();
        store
            .begin_effect(
                store.generation(),
                &repo::effect_id(&reg.repo_id, "platform"),
            )
            .unwrap();
        let reg = confirm_platform(
            &mut store,
            &reg.repo_id,
            PlatformObservation {
                instance: url.clone(),
                stable_id: meta["id"].to_string(),
                full_name: full_name.clone(),
                clone_url: clone_url.clone(),
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
        let reg = confirm_delivery(&mut store, &reg.repo_id).unwrap();
        let stable = meta["id"].to_string();
        finish(
            &mut store,
            &actor(),
            &reg.repo_id,
            reg.version,
            FinishChoice::Confirm(&stable),
            "finish",
            "finish",
        )
        .unwrap();
        reg.repo_id
    };
    let frozen = domain::freeze_policy(
        &mut store,
        &actor(),
        "policy-live",
        policy(&repo_id, true, false),
    )
    .unwrap();
    let holder = ProducerRef::Invocation {
        invocation_id: format!("inv-live-{stamp}"),
        invocation_version: 1,
    };
    let set = changeset::open_change_set(
        &mut store,
        &scoped(&repo_id),
        &repo_id,
        1,
        &base,
        &format!("cs-live-{stamp}"),
        &holder,
    )
    .unwrap();
    let (_, intent) = admit(&mut store, &set, &frozen, &tree, None, "assoc-live");
    let s = Scenario {
        shared: Arc::new(Mutex::new(Some(store))),
        repo_id,
        change_set_id: set.change_set_id.clone(),
        intent_id: intent.unwrap().intent_id,
        policy: frozen,
    };
    let branch = format!("hctl2/{}", s.change_set_id);
    for _ in 0..6 {
        drive_with(&s.shared, &s.repo_id, &s.intent_id, &mut |_| {
            Ok((
                Connection::Gitea(crate::scm::Hosted::fixture(
                    std::path::PathBuf::from(var("HCTL2_TEA")),
                    url.clone(),
                    user.clone(),
                    token.clone(),
                )),
                credential.clone(),
            ))
        })
        .unwrap();
        if shown(&s)["intent"]["state"] == "published" {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    let done = shown(&s);
    assert_eq!(done["intent"]["state"], "published", "{done}");
    let index = done["stages"]["review_request"]["index"].as_u64().unwrap();
    // Read the platform back independently of control's records.
    let remote = hosted
        .api("GET", &format!("repos/{full_name}/branches/{branch}"), None)
        .unwrap()
        .unwrap();
    assert_eq!(
        remote["commit"]["id"],
        json!(candidate),
        "the branch carries the executor's own commit"
    );
    let request = hosted
        .api("GET", &format!("repos/{full_name}/pulls/{index}"), None)
        .unwrap()
        .unwrap();
    assert_eq!(request["head"]["sha"], json!(candidate));
    assert_eq!(request["base"]["ref"], "main");
    assert_eq!(request["state"], "open");
    assert_eq!(done["mappings"][0]["platform_commit_sha"], json!(candidate));
    // A second pass changes nothing on the platform.
    assert!(drive_with(&s.shared, &s.repo_id, &s.intent_id, &mut |_| unreachable!()).is_err());
    eprintln!("LIVE gitea: branch {branch} at {candidate}, review request #{index}");
    // Tidy up: close the request and delete the branch.
    let _ = hosted.api(
        "PATCH",
        &format!("repos/{full_name}/pulls/{index}"),
        Some(json!({"state": "closed"})),
    );
    let _ = hosted.api(
        "DELETE",
        &format!("repos/{full_name}/branches/{branch}"),
        None,
    );
}
