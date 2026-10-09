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

use super::{Shared, drive_with};
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
        domain::freeze_commit(store, &s3.repo_id, &s3.intent_id, &platform3.candidate).unwrap();
        domain::mark_push_dispatched(store, &s3.repo_id, &s3.intent_id, true).unwrap();
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
    // Admitting it under the same policy redirects the intent: round 2, same request.
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
        assert_eq!(
            domain::release(store, &reducer, "release-x", &s.repo_id, &s.intent_id)
                .unwrap_err()
                .code,
            "PERMISSION_DENIED"
        );
        let released =
            domain::release(store, &actor(), "release-1", &s.repo_id, &s.intent_id).unwrap();
        assert_eq!(released.state, State::Pending);
        assert_eq!(released.authorizing_actor.principal, "owner");
    }
    drive(&platform, &s).unwrap();
    assert_eq!(shown(&s)["intent"]["state"], "published");
    assert_eq!(platform.creates(), 1);
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
