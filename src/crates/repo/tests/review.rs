//! Review publishing as a domain: the intent lives and dies with the admission transaction,
//! one ChangeSet publishes under one frozen policy, and the two stages record only what was
//! read back.
mod common;
use common::*;
use repo::changeset::{self, LeaseRef, OwnerGate, ProducerRef, Seal};
use repo::review::{self, Outcome, Policy, Publication, State};
use repo::{Platform, admit as admit_repo, prepare as prepare_registration};
use serde_json::json;
use store::{Actor, ActorSource, Scope, Store, TrustedActor};

fn scoped(repo_id: &str) -> TrustedActor {
    let mut actor = actor().0;
    actor.permission_scope.push(Scope::Repo(repo_id.into()));
    TrustedActor(actor)
}

fn sha(byte: u8) -> String {
    format!("{byte:02x}").repeat(20)
}

fn policy(repo_id: &str) -> Policy {
    Policy {
        repo_id: repo_id.into(),
        binding_version: 1,
        branch_rule: "hctl2/{change_set}".into(),
        target_branch: "main".into(),
        allow_update: true,
        description_source: "none".into(),
        requires_human_confirmation: false,
        audit_scope: "minimal".into(),
    }
}

fn repo_with_policy(store: &mut Store) -> (String, review::FrozenPolicy, changeset::ChangeSet) {
    let prepared = prepare_registration(request(Platform::None), None).unwrap();
    let repo_id = admit_repo(store, &actor(), "register", "register", prepared)
        .unwrap()
        .repo_id;
    let frozen = review::freeze_policy(store, &actor(), "policy-1", policy(&repo_id)).unwrap();
    let holder = ProducerRef::Invocation {
        invocation_id: "inv-1".into(),
        invocation_version: 1,
    };
    let set = changeset::open_change_set(
        store,
        &scoped(&repo_id),
        &repo_id,
        1,
        &sha(0xaa),
        "cs-key",
        &holder,
    )
    .unwrap();
    (repo_id, frozen, set)
}

fn seal(set: &changeset::ChangeSet, tree: &str, key: &str) -> Seal {
    Seal {
        association_key: key.into(),
        change_set_id: set.change_set_id.clone(),
        change_set_version: set.version,
        lease: Some(LeaseRef {
            lease_id: set.lease.lease_id.clone(),
            generation: set.lease.generation,
        }),
        base_commit_sha: set.baseline_commit.clone(),
        result_tree_sha: tree.into(),
        result_commit_sha: None,
        parent_revision_id: None,
        producer_ref: set.lease.holder.clone(),
    }
}

#[test]
fn the_intent_is_written_with_the_admission_or_not_at_all() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let (repo_id, frozen, set) = repo_with_policy(&mut store);
    // A policy frozen for another Repo: the admission transaction fails as a whole.
    let other_prepared = prepare_registration(request(Platform::None), None).unwrap();
    let other_repo = admit_repo(
        &mut store,
        &actor(),
        "register-2",
        "register-2",
        other_prepared,
    )
    .unwrap()
    .repo_id;
    let foreign =
        review::freeze_policy(&mut store, &actor(), "policy-x", policy(&other_repo)).unwrap();
    let publication = Publication {
        policy: foreign,
        authorizing_actor: actor().0,
    };
    let err = changeset::admit_with_publication(
        &mut store,
        &scoped(&repo_id),
        seal(&set, &sha(0x11), "assoc-1"),
        OwnerGate::Active,
        Some(&publication),
        1_000,
    )
    .unwrap_err();
    assert_eq!(err.code, "POLICY_REPO_MISMATCH");
    assert!(
        changeset::list_revisions(&store, &set.change_set_id)
            .unwrap()
            .is_empty()
    );
    assert!(review::list(&store, &repo_id).unwrap().is_empty());
    // The right policy: revision and intent land together, the intent queued and carrying
    // the authorizing command's envelope.
    let publication = Publication {
        policy: frozen.clone(),
        authorizing_actor: actor().0,
    };
    let (revision, intent) = changeset::admit_with_publication(
        &mut store,
        &scoped(&repo_id),
        seal(&set, &sha(0x11), "assoc-1"),
        OwnerGate::Active,
        Some(&publication),
        1_000,
    )
    .unwrap();
    let intent = intent.unwrap();
    assert_eq!(intent.state, State::Pending);
    assert_eq!(intent.round, 1);
    assert_eq!(intent.branch, format!("hctl2/{}", set.change_set_id));
    assert_eq!(
        intent.target.change_set_revision_id,
        revision.change_set_revision_id
    );
    assert_eq!(intent.authorized_by, revision.producer_ref);
    assert_eq!(intent.authorizing_actor.principal, "owner");
    assert_eq!(review::open(&store).unwrap().len(), 1);
    assert_eq!(
        review::intent_id(store.control_id(), &repo_id, &set.change_set_id),
        intent.intent_id
    );
    // Re-admitting the same revision is idempotent for the intent too.
    let (_, again) = changeset::admit_with_publication(
        &mut store,
        &scoped(&repo_id),
        seal(&set, &sha(0x11), "assoc-1"),
        OwnerGate::Active,
        Some(&publication),
        2_000,
    )
    .unwrap();
    assert_eq!(again.unwrap().version, intent.version);
    // Another frozen policy for the same ChangeSet is refused.
    let mut changed = policy(&repo_id);
    changed.target_branch = "release".into();
    let other_policy = review::freeze_policy(&mut store, &actor(), "policy-2", changed).unwrap();
    let err = changeset::admit_with_publication(
        &mut store,
        &scoped(&repo_id),
        seal(&set, &sha(0x22), "assoc-2"),
        OwnerGate::Active,
        Some(&Publication {
            policy: other_policy,
            authorizing_actor: actor().0,
        }),
        3_000,
    )
    .unwrap_err();
    assert_eq!(err.code, "POLICY_CHANGED");
    assert_eq!(
        changeset::list_revisions(&store, &set.change_set_id)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn stages_record_only_what_was_read_back_and_the_mapping_is_written_once() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let (repo_id, frozen, set) = repo_with_policy(&mut store);
    let publication = Publication {
        policy: frozen,
        authorizing_actor: actor().0,
    };
    let (revision, intent) = changeset::admit_with_publication(
        &mut store,
        &scoped(&repo_id),
        seal(&set, &sha(0x11), "assoc-1"),
        OwnerGate::Active,
        Some(&publication),
        1_000,
    )
    .unwrap();
    let id = intent.unwrap().intent_id;
    review::begin(&mut store, &repo_id, &id).unwrap();
    // The commit is frozen before the push; another commit cannot replace it once dispatched.
    let frozen = review::freeze_commit(&mut store, &repo_id, &id, &sha(0xc1)).unwrap();
    assert_eq!(
        frozen.target.commit_sha.as_deref(),
        Some(sha(0xc1).as_str())
    );
    review::mark_push_dispatched(&mut store, &repo_id, &id, true).unwrap();
    assert_eq!(
        review::freeze_commit(&mut store, &repo_id, &id, &sha(0xc2))
            .unwrap_err()
            .code,
        "PUSH_IN_FLIGHT"
    );
    // Confirming a push of some other commit is refused; the frozen one is recorded.
    assert_eq!(
        review::confirm_push(&mut store, &repo_id, &id, &sha(0xc2), 5)
            .unwrap_err()
            .code,
        "FROZEN_INPUT_CHANGED"
    );
    let pushed = review::confirm_push(&mut store, &repo_id, &id, &sha(0xc1), 5).unwrap();
    assert_eq!(
        pushed.push.confirmed_commit.as_deref(),
        Some(sha(0xc1).as_str())
    );
    assert_eq!(pushed.push.confirmed_at_unix_ms, Some(5));
    // Waiting on the platform keeps the intent open and the stage-1 confirmation.
    let waiting = review::confirm(
        &mut store,
        &repo_id,
        &id,
        Outcome::Attention(review::Attention {
            code: "PLATFORM_UNAVAILABLE".into(),
            message: "down".into(),
            recovery_action: "retry".into(),
            details: json!({}),
        }),
        6,
    )
    .unwrap();
    assert_eq!(waiting.state, State::Unknown);
    assert_eq!(
        waiting.push.confirmed_commit.as_deref(),
        Some(sha(0xc1).as_str())
    );
    assert_eq!(review::open(&store).unwrap().len(), 1);
    // Published with a commit other than the frozen one is refused; the frozen one writes
    // the mapping in the shape the integration half reads, and settles the round.
    review::begin(&mut store, &repo_id, &id).unwrap();
    assert_eq!(
        review::confirm(
            &mut store,
            &repo_id,
            &id,
            Outcome::Published {
                index: 9,
                platform_commit_sha: sha(0xc2),
                readback: json!({}),
            },
            7,
        )
        .unwrap_err()
        .code,
        "FROZEN_INPUT_CHANGED"
    );
    let done = review::confirm(
        &mut store,
        &repo_id,
        &id,
        Outcome::Published {
            index: 9,
            platform_commit_sha: sha(0xc1),
            readback: json!({"ok": true}),
        },
        7,
    )
    .unwrap();
    assert_eq!(done.state, State::Published);
    assert_eq!(done.review.index, Some(9));
    assert!(review::open(&store).unwrap().is_empty());
    let mapped =
        repo::integration::review_request(&store, &repo_id, &revision.change_set_revision_id)
            .unwrap()
            .unwrap();
    assert_eq!(mapped.index, 9);
    assert_eq!(mapped.platform_commit_sha, sha(0xc1));
    let shown = review::show(&store, &repo_id, &id).unwrap();
    assert_eq!(shown["stages"]["push"]["confirmed"], true);
    assert_eq!(shown["stages"]["review_request"]["index"], 9);
    assert_eq!(shown["mappings"].as_array().unwrap().len(), 1);
    // Settled: begin and confirm refuse; a human release is a no-op on a published intent.
    assert_eq!(
        review::begin(&mut store, &repo_id, &id).unwrap_err().code,
        "INTENT_TERMINAL"
    );
    assert_eq!(
        review::release(&mut store, &actor(), "release", &repo_id, &id)
            .unwrap()
            .state,
        State::Published
    );
}

#[test]
fn a_human_gate_holds_the_queue_and_only_a_direct_client_releases_it() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let (repo_id, _, set) = repo_with_policy(&mut store);
    let mut gated = policy(&repo_id);
    gated.requires_human_confirmation = true;
    let frozen = review::freeze_policy(&mut store, &actor(), "policy-h", gated).unwrap();
    let (_, intent) = changeset::admit_with_publication(
        &mut store,
        &scoped(&repo_id),
        seal(&set, &sha(0x11), "assoc-1"),
        OwnerGate::Active,
        Some(&Publication {
            policy: frozen,
            authorizing_actor: actor().0,
        }),
        1_000,
    )
    .unwrap();
    let intent = intent.unwrap();
    assert_eq!(intent.state, State::PendingHuman);
    assert!(review::open(&store).unwrap().is_empty(), "nothing queued");
    assert_eq!(
        review::begin(&mut store, &repo_id, &intent.intent_id)
            .unwrap_err()
            .code,
        "INTENT_TERMINAL"
    );
    let reducer = TrustedActor(Actor {
        principal: "x".into(),
        source: ActorSource::InternalReducer,
        permission_scope: vec![Scope::Control],
        authority: None,
    });
    assert_eq!(
        review::release(&mut store, &reducer, "r", &repo_id, &intent.intent_id)
            .unwrap_err()
            .code,
        "PERMISSION_DENIED"
    );
    let released = review::release(
        &mut store,
        &actor(),
        "release-1",
        &repo_id,
        &intent.intent_id,
    )
    .unwrap();
    assert_eq!(released.state, State::Pending);
    assert_eq!(review::open(&store).unwrap().len(), 1);
    // A second release is idempotent.
    assert_eq!(
        review::release(
            &mut store,
            &actor(),
            "release-2",
            &repo_id,
            &intent.intent_id
        )
        .unwrap()
        .version,
        released.version
    );
    // Policies are frozen by digest: same content is idempotent, other content under the
    // same id is refused, invalid shapes never freeze.
    assert_eq!(
        review::freeze_policy(&mut store, &actor(), "policy-h", {
            let mut p = policy(&repo_id);
            p.requires_human_confirmation = true;
            p
        })
        .unwrap()
        .digest,
        released.policy.digest
    );
    assert_eq!(
        review::freeze_policy(&mut store, &actor(), "policy-h", policy(&repo_id))
            .unwrap_err()
            .code,
        "POLICY_CONFLICT"
    );
    let mut bad = policy(&repo_id);
    bad.branch_rule = "refs/heads/x".into();
    assert_eq!(
        review::freeze_policy(&mut store, &actor(), "policy-bad", bad)
            .unwrap_err()
            .code,
        "INVALID_INPUT"
    );
}
