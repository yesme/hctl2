mod common;
use common::*;
use repo::changeset::{
    ChangeSet, ChangeSetRevision, LeaseRef, LeaseState, OwnerGate, ProducerRef, Seal,
    admit as admit_with_owner, get_revision, list_revisions, open_change_set,
};
use store::{
    Actor, ActorSource, Command, Expected, ObjectKey, Record, RecordData, Reference, Scope, Store,
    TrustedActor, Version,
};

fn actor(repo_id: &str) -> TrustedActor {
    TrustedActor(Actor {
        principal: "owner".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![Scope::Control, Scope::Repo(repo_id.into())],
        authority: None,
    })
}

fn sha(byte: u8) -> String {
    format!("{byte:02x}").repeat(20)
}

fn invocation(id: &str) -> ProducerRef {
    ProducerRef::Invocation {
        invocation_id: id.into(),
        invocation_version: 1,
    }
}

fn admit(store: &mut Store, actor: &TrustedActor, seal: Seal) -> repo::Result<ChangeSetRevision> {
    admit_with_owner(store, actor, seal, OwnerGate::Active)
}

fn seal(change_set_id: &str, lease_id: &str, key: &str, base: &str, tree: &str) -> Seal {
    Seal {
        association_key: key.into(),
        change_set_id: change_set_id.into(),
        change_set_version: 1,
        lease: Some(LeaseRef {
            lease_id: lease_id.into(),
            generation: 1,
        }),
        base_commit_sha: base.into(),
        result_tree_sha: tree.into(),
        result_commit_sha: Some(sha(9)),
        parent_revision_id: None,
        producer_ref: invocation("inv-1"),
    }
}

fn opened(store: &mut Store) -> repo::changeset::ChangeSet {
    open_change_set(
        store,
        &actor("repo-1"),
        "repo-1",
        3,
        &sha(1),
        "write-1",
        &invocation("inv-1"),
    )
    .unwrap()
}

fn replace_set(store: &mut Store, set: &mut repo::changeset::ChangeSet) {
    let previous = set.version;
    set.version += 1;
    let key = ObjectKey {
        scope: Scope::Repo(set.repo_id.clone()),
        kind: "changeset".into(),
        id: set.change_set_id.clone(),
    };
    let value = serde_json::to_value(&set).unwrap();
    let record = Record {
        key: key.clone(),
        version: set.version,
        revision_digest: foundation::canonical_json_sha256(&value).unwrap(),
        data: RecordData::Value {
            value: value.clone(),
        },
        sources: vec![],
        materials: vec![],
    };
    let actor = actor(&set.repo_id);
    let command = Command {
        command_id: format!("fixture-{}-{}", set.change_set_id, set.version),
        idempotency_key: format!("fixture-{}-{}", set.change_set_id, set.version),
        actor: actor.0.clone(),
        target: key.clone(),
        expected: Expected::Exact(Version::State(previous)),
        binding: Reference {
            key,
            version: Version::State(1),
        },
        input_digest: Command::digest_input("fixture.replace", &value).unwrap(),
        operation: "fixture.replace".into(),
        input: value,
    };
    store
        .submit(store.generation(), &actor, &command, None, |tx| {
            tx.put(&record)?;
            Ok(serde_json::json!({"replaced": true}))
        })
        .unwrap();
}

fn lease_transaction<T: serde::Serialize>(
    store: &mut Store,
    key: &str,
    run: impl FnOnce(&mut store::CommandTransaction<'_>) -> repo::Result<T>,
) -> repo::Result<serde_json::Value> {
    let actor = actor("repo-1");
    let target = ObjectKey {
        scope: Scope::Repo("repo-1".into()),
        kind: "lease_test".into(),
        id: key.into(),
    };
    let command = Command {
        command_id: key.into(),
        idempotency_key: key.into(),
        actor: actor.0.clone(),
        binding: Reference {
            key: target.clone(),
            version: Version::State(1),
        },
        target,
        expected: Expected::Absent,
        operation: "lease.test".into(),
        input: serde_json::json!({}),
        input_digest: Command::digest_input("lease.test", &serde_json::json!({})).unwrap(),
    };
    store.submit(store.generation(), &actor, &command, None, |tx| {
        Ok(serde_json::to_value(run(tx)?)?)
    })
}

#[test]
fn pending_lease_is_not_a_grant_and_authorization_failure_rolls_it_back() {
    use repo::changeset::{acquire_lease, plan_lease};
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let pending = plan_lease(
        &store,
        "repo-1",
        3,
        &sha(1),
        "write",
        None,
        &invocation("writer"),
    )
    .unwrap();
    assert_eq!(pending.pending.lease.state, LeaseState::Pending);
    assert!(store.list("changeset").unwrap().is_empty());
    let error = lease_transaction::<serde_json::Value>(&mut store, "failed-start", |tx| {
        acquire_lease(tx, &pending)?;
        Err(repo::reject("TEST_ABORT", "owner admission failed", "test"))
    })
    .unwrap_err();
    assert_eq!(error.code, "TEST_ABORT");
    assert!(store.list("changeset").unwrap().is_empty());
    let active: ChangeSet = serde_json::from_value(
        lease_transaction(&mut store, "start", |tx| acquire_lease(tx, &pending)).unwrap(),
    )
    .unwrap();
    assert_eq!(active.lease.state, LeaseState::Active);
    assert_eq!(active.lease.holder, invocation("writer"));
}

#[test]
fn second_writer_cannot_take_active_or_revoking_lease_even_after_restart() {
    use repo::changeset::{acquire_lease, plan_lease, revoke_lease};
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let pending = plan_lease(
        &store,
        "repo-1",
        3,
        &sha(1),
        "write",
        None,
        &invocation("first"),
    )
    .unwrap();
    let stale = pending.clone();
    let active: ChangeSet = serde_json::from_value(
        lease_transaction(&mut store, "start", |tx| acquire_lease(tx, &pending)).unwrap(),
    )
    .unwrap();
    assert_eq!(
        lease_transaction(&mut store, "stale-start", |tx| acquire_lease(tx, &stale))
            .unwrap_err()
            .code,
        "VERSION_CONFLICT"
    );
    assert_eq!(
        plan_lease(
            &store,
            "repo-1",
            3,
            &sha(1),
            "next",
            Some(&active.change_set_id),
            &invocation("next")
        )
        .unwrap_err()
        .code,
        "WRITE_LEASE_BUSY"
    );
    let lease = LeaseRef {
        lease_id: active.lease.lease_id.clone(),
        generation: active.lease.generation,
    };
    assert_eq!(
        lease_transaction(&mut store, "wrong-producer", |tx| revoke_lease(
            tx,
            "repo-1",
            &active.change_set_id,
            &lease,
            &invocation("other")
        ))
        .unwrap_err()
        .code,
        "LEASE_NOT_CURRENT"
    );
    let revoked: ChangeSet = serde_json::from_value(
        lease_transaction(&mut store, "cancel", |tx| {
            revoke_lease(
                tx,
                "repo-1",
                &active.change_set_id,
                &lease,
                &invocation("first"),
            )
        })
        .unwrap(),
    )
    .unwrap();
    assert_eq!(revoked.lease.state, LeaseState::Revoking);
    drop(store);
    let store = Store::open(&temp.0).unwrap();
    assert_eq!(
        plan_lease(
            &store,
            "repo-1",
            3,
            &sha(1),
            "next",
            Some(&active.change_set_id),
            &invocation("next")
        )
        .unwrap_err()
        .code,
        "WRITE_LEASE_BUSY"
    );
}

#[test]
fn lease_preview_cannot_cross_repo_or_tamper_with_identity_generation_or_state() {
    use repo::changeset::{acquire_lease, plan_lease};
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let active = opened(&mut store);
    assert_eq!(
        plan_lease(
            &store,
            "repo-2",
            3,
            &sha(1),
            "next",
            Some(&active.change_set_id),
            &invocation("next")
        )
        .unwrap_err()
        .code,
        "CHANGESET_NOT_FOUND"
    );
    let pending = plan_lease(
        &store,
        "repo-1",
        3,
        &sha(1),
        "other",
        None,
        &invocation("writer"),
    )
    .unwrap();
    for field in ["generation", "id", "state", "version"] {
        let mut wrong = pending.clone();
        match field {
            "generation" => wrong.pending.lease.generation += 1,
            "id" => wrong.pending.lease.lease_id = "borrowed".into(),
            "state" => wrong.pending.lease.state = LeaseState::Active,
            "version" => wrong.pending.version += 1,
            _ => unreachable!(),
        }
        assert_eq!(
            lease_transaction(&mut store, field, |tx| acquire_lease(tx, &wrong))
                .unwrap_err()
                .code,
            "LEASE_PREVIEW_MISMATCH",
            "{field}"
        );
    }
    assert_eq!(store.list("changeset").unwrap().len(), 1);
}

#[test]
fn admitted_result_replays_after_owner_closes_or_lease_changes() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let mut set = opened(&mut store);
    let request = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "lost-response",
        &sha(1),
        &sha(2),
    );
    let first = admit(&mut store, &actor("repo-1"), request.clone()).unwrap();
    for owner in [OwnerGate::Cancelled, OwnerGate::Superseded] {
        assert_eq!(
            admit_with_owner(&mut store, &actor("repo-1"), request.clone(), owner).unwrap(),
            first
        );
    }
    set.lease.generation += 1;
    replace_set(&mut store, &mut set);
    drop(store);
    let mut store = Store::open(&temp.0).unwrap();
    assert_eq!(
        admit(&mut store, &actor("repo-1"), request.clone()).unwrap(),
        first
    );
    let mut changed = request;
    changed.result_tree_sha = sha(3);
    assert_eq!(
        admit(&mut store, &actor("repo-1"), changed)
            .unwrap_err()
            .code,
        "IDEMPOTENCY_CONFLICT"
    );
}

#[test]
fn every_association_is_recorded_even_when_the_revision_already_exists() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let set = opened(&mut store);
    let first = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "first",
        &sha(1),
        &sha(2),
    );
    let original = admit(&mut store, &actor("repo-1"), first.clone()).unwrap();
    let mut another = first;
    another.association_key = "another".into();
    assert_eq!(
        admit(&mut store, &actor("repo-1"), another.clone()).unwrap(),
        original
    );
    another.result_tree_sha = sha(3);
    assert_eq!(
        admit(&mut store, &actor("repo-1"), another)
            .unwrap_err()
            .code,
        "IDEMPOTENCY_CONFLICT"
    );
    assert_eq!(
        list_revisions(&store, &set.change_set_id).unwrap(),
        vec![original]
    );
}

#[test]
fn a_different_invocation_cannot_borrow_the_lease() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let set = opened(&mut store);
    let mut request = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "wrong-producer",
        &sha(1),
        &sha(2),
    );
    request.producer_ref = invocation("inv-OTHER");
    assert_eq!(
        admit(&mut store, &actor("repo-1"), request)
            .unwrap_err()
            .code,
        "LEASE_PRODUCER_MISMATCH"
    );
    assert!(
        list_revisions(&store, &set.change_set_id)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn uppercase_object_ids_cannot_make_a_second_identity() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let set = opened(&mut store);
    let request = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "lower",
        &sha(0xab),
        &sha(0xcd),
    );
    let original = admit(&mut store, &actor("repo-1"), request.clone()).unwrap();
    for field in ["base", "tree", "commit"] {
        let mut upper = request.clone();
        upper.association_key = format!("upper-{field}");
        match field {
            "base" => upper.base_commit_sha.make_ascii_uppercase(),
            "tree" => upper.result_tree_sha.make_ascii_uppercase(),
            "commit" => upper.result_commit_sha = Some(sha(0xef).to_ascii_uppercase()),
            _ => unreachable!(),
        }
        assert_eq!(
            admit(&mut store, &actor("repo-1"), upper).unwrap_err().code,
            "GIT_SHA_INVALID",
            "{field}"
        );
    }
    assert_eq!(
        list_revisions(&store, &set.change_set_id).unwrap(),
        vec![original]
    );
}

#[test]
fn admission_records_five_identity_fields_and_replays_the_same_key() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let set = opened(&mut store);
    let mut request = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "seal-1",
        &sha(1),
        &sha(2),
    );
    let first = admit(&mut store, &actor("repo-1"), request.clone()).unwrap();
    assert_eq!(first.change_set_id, set.change_set_id);
    assert_eq!(first.base_commit_sha, sha(1));
    assert_eq!(first.result_tree_sha, sha(2));
    assert!(first.parent_revision_id.is_none());
    assert!(!first.change_set_revision_id.is_empty());
    assert_eq!(first.producer_ref, invocation("inv-1"));
    let encoded = serde_json::to_value(&first).unwrap();
    assert_eq!(encoded.as_object().unwrap().len(), 8);
    assert!(encoded.get("result_commit_sha").is_none());
    let again = admit(&mut store, &actor("repo-1"), request.clone()).unwrap();
    assert_eq!(again, first);
    request.result_tree_sha = sha(3);
    assert_eq!(
        admit(&mut store, &actor("repo-1"), request)
            .unwrap_err()
            .code,
        "IDEMPOTENCY_CONFLICT"
    );
    assert_eq!(
        list_revisions(&store, &set.change_set_id).unwrap(),
        vec![first]
    );
}

#[test]
fn another_commit_wrapper_keeps_the_revision_and_digest() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let set = opened(&mut store);
    let mut first_seal = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "wrap-1",
        &sha(1),
        &sha(2),
    );
    first_seal.result_commit_sha = Some(sha(4));
    let first = admit(&mut store, &actor("repo-1"), first_seal).unwrap();
    let mut second_seal = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "wrap-2",
        &sha(1),
        &sha(2),
    );
    second_seal.result_commit_sha = Some(sha(5));
    second_seal.producer_ref = ProducerRef::HumanCommand {
        command_id: "later".into(),
    };
    second_seal.lease = None;
    let second = admit(&mut store, &actor("repo-1"), second_seal).unwrap();
    assert_eq!(second, first);
    assert_eq!(second.review_subject_digest, first.review_subject_digest);
    let stored = get_revision(&store, &first.change_set_revision_id).unwrap();
    assert!(
        serde_json::to_value(&stored)
            .unwrap()
            .get("result_commit_sha")
            .is_none()
    );
}

#[test]
fn base_or_tree_change_appends_a_revision_and_leaves_the_old_one() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let set = opened(&mut store);
    let original = admit(
        &mut store,
        &actor("repo-1"),
        seal(
            &set.change_set_id,
            &set.lease.lease_id,
            "rev-1",
            &sha(1),
            &sha(2),
        ),
    )
    .unwrap();
    let tree = admit(
        &mut store,
        &actor("repo-1"),
        seal(
            &set.change_set_id,
            &set.lease.lease_id,
            "rev-2",
            &sha(1),
            &sha(3),
        ),
    )
    .unwrap();
    let base = admit(
        &mut store,
        &actor("repo-1"),
        seal(
            &set.change_set_id,
            &set.lease.lease_id,
            "rev-3",
            &sha(7),
            &sha(3),
        ),
    )
    .unwrap();
    assert_ne!(tree.change_set_revision_id, original.change_set_revision_id);
    assert_ne!(base.change_set_revision_id, tree.change_set_revision_id);
    assert_ne!(base.review_subject_digest, tree.review_subject_digest);
    assert_eq!(
        get_revision(&store, &original.change_set_revision_id).unwrap(),
        original
    );
    assert_eq!(list_revisions(&store, &set.change_set_id).unwrap().len(), 3);
}

#[test]
fn closed_owner_or_stale_lease_does_not_admit_existing_git_bytes() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let set = opened(&mut store);
    let closed = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "closed",
        &sha(1),
        &sha(2),
    );
    assert_eq!(
        admit_with_owner(&mut store, &actor("repo-1"), closed, OwnerGate::Cancelled)
            .unwrap_err()
            .code,
        "CHANGESET_OWNER_CLOSED"
    );
    let replaced = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "replaced",
        &sha(1),
        &sha(2),
    );
    assert_eq!(
        admit_with_owner(
            &mut store,
            &actor("repo-1"),
            replaced,
            OwnerGate::Superseded
        )
        .unwrap_err()
        .code,
        "CHANGESET_OWNER_CLOSED"
    );
    let mut stale = seal(&set.change_set_id, "lease-other", "stale", &sha(1), &sha(2));
    stale.lease.as_mut().unwrap().generation = 2;
    assert_eq!(
        admit(&mut store, &actor("repo-1"), stale).unwrap_err().code,
        "LEASE_NOT_CURRENT"
    );
    assert!(
        list_revisions(&store, &set.change_set_id)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn human_seal_is_not_recorded_as_an_invocation() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let set = opened(&mut store);
    let mut request = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "human",
        &sha(1),
        &sha(2),
    );
    request.producer_ref = ProducerRef::HumanCommand {
        command_id: "cmd-seal".into(),
    };
    request.lease = None;
    let revision = admit(&mut store, &actor("repo-1"), request).unwrap();
    assert_eq!(
        revision.producer_ref,
        ProducerRef::HumanCommand {
            command_id: "cmd-seal".into(),
        }
    );
    let encoded = serde_json::to_value(ChangeSetRevision::clone(&revision)).unwrap();
    assert_eq!(encoded["producer_ref"]["kind"], "human_command");
    assert!(encoded.get("invocation_id").is_none());
}

fn rejection(store: &mut Store, set: &ChangeSet, request: Seal, code: &str) {
    let err = admit(store, &actor(&set.repo_id), request).unwrap_err();
    assert_eq!(err.code, code);
    assert!(!err.recovery_action.is_empty());
    assert!(
        list_revisions(store, &set.change_set_id)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn revoking_or_revoked_lease_is_rejected_independently() {
    for state in [
        LeaseState::Pending,
        LeaseState::Revoking,
        LeaseState::Revoked,
    ] {
        let temp = Temp::new();
        let mut store = Store::open(&temp.0).unwrap();
        let mut set = opened(&mut store);
        set.lease.state = state;
        replace_set(&mut store, &mut set);
        let mut request = seal(
            &set.change_set_id,
            &set.lease.lease_id,
            "inactive",
            &sha(1),
            &sha(2),
        );
        request.change_set_version = set.version;
        rejection(&mut store, &set, request, "LEASE_NOT_CURRENT");
    }
}

#[test]
fn wrong_lease_id_is_rejected_with_the_right_generation() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let set = opened(&mut store);
    let request = seal(
        &set.change_set_id,
        "wrong-lease",
        "wrong-id",
        &sha(1),
        &sha(2),
    );
    rejection(&mut store, &set, request, "LEASE_NOT_CURRENT");
}

#[test]
fn wrong_generation_is_rejected_with_the_right_lease_id() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let set = opened(&mut store);
    let mut request = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "wrong-generation",
        &sha(1),
        &sha(2),
    );
    request.lease.as_mut().unwrap().generation = 2;
    rejection(&mut store, &set, request, "LEASE_NOT_CURRENT");
}

#[test]
fn invocation_requires_a_lease_reference() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let set = opened(&mut store);
    let mut request = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "no-lease",
        &sha(1),
        &sha(2),
    );
    request.lease = None;
    rejection(&mut store, &set, request, "LEASE_NOT_CURRENT");
}

#[test]
fn a_different_invocation_version_cannot_borrow_the_lease() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let set = opened(&mut store);
    let mut request = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "wrong-version",
        &sha(1),
        &sha(2),
    );
    request.producer_ref = ProducerRef::Invocation {
        invocation_id: "inv-1".into(),
        invocation_version: 2,
    };
    rejection(&mut store, &set, request, "LEASE_PRODUCER_MISMATCH");
}

#[test]
fn parent_from_another_changeset_is_rejected() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let first_set = opened(&mut store);
    let parent = admit(
        &mut store,
        &actor("repo-1"),
        seal(
            &first_set.change_set_id,
            &first_set.lease.lease_id,
            "parent",
            &sha(1),
            &sha(2),
        ),
    )
    .unwrap();
    let other = open_change_set(
        &mut store,
        &actor("repo-1"),
        "repo-1",
        3,
        &sha(1),
        "other",
        &invocation("inv-1"),
    )
    .unwrap();
    let mut request = seal(
        &other.change_set_id,
        &other.lease.lease_id,
        "foreign-parent",
        &sha(1),
        &sha(3),
    );
    request.parent_revision_id = Some(parent.change_set_revision_id.clone());
    rejection(&mut store, &other, request, "CHANGESET_PARENT_MISMATCH");
    assert_eq!(
        get_revision(&store, &parent.change_set_revision_id).unwrap(),
        parent
    );
}

#[test]
fn absent_parent_is_not_admitted_and_a_valid_parent_links_a_new_revision() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let set = opened(&mut store);
    let mut request = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "parent-missing",
        &sha(1),
        &sha(2),
    );
    request.parent_revision_id = Some("csr-missing".into());
    rejection(
        &mut store,
        &set,
        request.clone(),
        "CHANGESET_REVISION_NOT_FOUND",
    );
    request.parent_revision_id = None;
    let parent = admit(&mut store, &actor("repo-1"), request.clone()).unwrap();
    request.association_key = "child".into();
    request.parent_revision_id = Some(parent.change_set_revision_id.clone());
    let child = admit(&mut store, &actor("repo-1"), request).unwrap();
    assert_eq!(
        child.parent_revision_id.as_deref(),
        Some(parent.change_set_revision_id.as_str())
    );
    assert_ne!(child.change_set_revision_id, parent.change_set_revision_id);
    assert_eq!(list_revisions(&store, &set.change_set_id).unwrap().len(), 2);
}

#[test]
fn invalid_base_object_id_is_rejected_independently() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let set = opened(&mut store);
    let request = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "bad-base",
        "not-a-sha",
        &sha(2),
    );
    rejection(&mut store, &set, request, "GIT_SHA_INVALID");
}

#[test]
fn invalid_tree_object_id_is_rejected_independently() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let set = opened(&mut store);
    let request = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "bad-tree",
        &sha(1),
        "not-a-sha",
    );
    rejection(&mut store, &set, request, "GIT_SHA_INVALID");
}

#[test]
fn invalid_commit_wrapper_is_rejected_independently() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let set = opened(&mut store);
    let mut request = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "bad-commit",
        &sha(1),
        &sha(2),
    );
    request.result_commit_sha = Some("not-a-sha".into());
    rejection(&mut store, &set, request, "GIT_SHA_INVALID");
}

#[test]
fn absent_changeset_is_rejected_without_a_revision() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let request = seal("cs-missing", "lease-missing", "missing", &sha(1), &sha(2));
    assert_eq!(
        admit(&mut store, &actor("repo-1"), request)
            .unwrap_err()
            .code,
        "CHANGESET_NOT_FOUND"
    );
    assert!(store.list("changeset_revision").unwrap().is_empty());
}

#[test]
fn open_replays_its_result_and_rejects_changed_input() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let mut set = opened(&mut store);
    let original = set.clone();
    set.lease.generation += 1;
    replace_set(&mut store, &mut set);
    assert_eq!(opened(&mut store), original);
    for (base, binding, holder) in [
        (sha(2), 3, invocation("inv-1")),
        (sha(1), 4, invocation("inv-1")),
        (sha(1), 3, invocation("another")),
    ] {
        assert_eq!(
            open_change_set(
                &mut store,
                &actor("repo-1"),
                "repo-1",
                binding,
                &base,
                "write-1",
                &holder
            )
            .unwrap_err()
            .code,
            "IDEMPOTENCY_CONFLICT"
        );
    }
}

#[test]
fn raw_keys_are_namespaced_by_operation_repo_and_changeset() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let a = opened(&mut store);
    let b = open_change_set(
        &mut store,
        &actor("repo-2"),
        "repo-2",
        3,
        &sha(1),
        "write-1",
        &invocation("inv-1"),
    )
    .unwrap();
    assert_ne!(a.change_set_id, b.change_set_id);
    for set in [a, b] {
        let request = seal(
            &set.change_set_id,
            &set.lease.lease_id,
            "write-1",
            &sha(1),
            &sha(2),
        );
        assert_eq!(
            admit(&mut store, &actor(&set.repo_id), request)
                .unwrap()
                .change_set_id,
            set.change_set_id
        );
    }
}

#[test]
fn human_sealing_uses_its_own_command_not_the_revoking_invocations_lease() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let mut set = opened(&mut store);
    set.lease.state = LeaseState::Revoking;
    replace_set(&mut store, &mut set);
    let mut request = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "human-residual",
        &sha(1),
        &sha(2),
    );
    request.change_set_version = set.version;
    request.lease = None;
    request.producer_ref = ProducerRef::HumanCommand {
        command_id: "cmd-residual".into(),
    };
    let original = admit_with_owner(
        &mut store,
        &actor("repo-1"),
        request.clone(),
        OwnerGate::Cancelled,
    )
    .unwrap();
    assert_eq!(original.producer_ref, request.producer_ref);
    assert_eq!(
        admit_with_owner(&mut store, &actor("repo-1"), request, OwnerGate::Superseded).unwrap(),
        original
    );
    let row = store
        .get(&ObjectKey {
            scope: Scope::Repo("repo-1".into()),
            kind: "changeset".into(),
            id: set.change_set_id,
        })
        .unwrap()
        .unwrap();
    let RecordData::Value { value } = row.data else {
        panic!("changeset payload")
    };
    assert_eq!(value["lease"]["state"], "revoking");
}

#[test]
fn human_sealing_rejects_provider_or_reducer_provenance_and_missing_permissions() {
    for source in [
        ActorSource::ProviderEvent,
        ActorSource::InternalReducer,
        ActorSource::DirectClient,
    ] {
        let temp = Temp::new();
        let mut store = Store::open(&temp.0).unwrap();
        let set = opened(&mut store);
        let mut caller = actor("repo-1");
        caller.0.source = source.clone();
        if source == ActorSource::InternalReducer {
            caller.0.authority = Some(Reference {
                key: ObjectKey {
                    scope: Scope::Repo("repo-1".into()),
                    kind: "changeset".into(),
                    id: set.change_set_id.clone(),
                },
                version: Version::State(1),
            });
        }
        if source == ActorSource::DirectClient {
            caller
                .0
                .permission_scope
                .retain(|scope| *scope != Scope::Control);
        }
        let mut request = seal(
            &set.change_set_id,
            &set.lease.lease_id,
            "not-human",
            &sha(1),
            &sha(2),
        );
        request.lease = None;
        request.producer_ref = ProducerRef::HumanCommand {
            command_id: "fake-human".into(),
        };
        assert_eq!(
            admit(&mut store, &caller, request).unwrap_err().code,
            "PERMISSION_DENIED"
        );
        assert!(
            list_revisions(&store, &set.change_set_id)
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn human_command_cannot_borrow_an_invocation_lease() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let set = opened(&mut store);
    let mut request = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "human-with-lease",
        &sha(1),
        &sha(2),
    );
    request.producer_ref = ProducerRef::HumanCommand {
        command_id: "cmd-borrow".into(),
    };
    rejection(&mut store, &set, request, "HUMAN_SEAL_LEASE");
}

#[test]
fn admission_does_not_replay_for_an_unauthorized_repo_actor() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let set = opened(&mut store);
    let request = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "authorized",
        &sha(1),
        &sha(2),
    );
    admit(&mut store, &actor("repo-1"), request.clone()).unwrap();
    assert_eq!(
        admit(&mut store, &actor("repo-2"), request)
            .unwrap_err()
            .code,
        "PERMISSION_DENIED"
    );
}

#[test]
fn empty_or_unversioned_producer_and_empty_association_are_rejected() {
    for producer in [
        invocation(""),
        ProducerRef::Invocation {
            invocation_id: "inv-1".into(),
            invocation_version: 0,
        },
        ProducerRef::HumanCommand {
            command_id: "".into(),
        },
    ] {
        let temp = Temp::new();
        let mut store = Store::open(&temp.0).unwrap();
        assert_eq!(
            open_change_set(
                &mut store,
                &actor("repo-1"),
                "repo-1",
                3,
                &sha(1),
                "invalid",
                &producer
            )
            .unwrap_err()
            .code,
            "INVALID_INPUT"
        );
    }
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let set = opened(&mut store);
    let request = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        " ",
        &sha(1),
        &sha(2),
    );
    rejection(&mut store, &set, request, "INVALID_INPUT");
}

#[test]
fn admission_rejects_an_empty_or_unversioned_producer() {
    for producer in [
        invocation(""),
        ProducerRef::Invocation {
            invocation_id: "inv-1".into(),
            invocation_version: 0,
        },
    ] {
        let temp = Temp::new();
        let mut store = Store::open(&temp.0).unwrap();
        let set = opened(&mut store);
        let mut request = seal(
            &set.change_set_id,
            &set.lease.lease_id,
            "bad-producer",
            &sha(1),
            &sha(2),
        );
        request.producer_ref = producer;
        rejection(&mut store, &set, request, "INVALID_INPUT");
    }
}

#[test]
fn fresh_admission_uses_the_frozen_changeset_version() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let mut set = opened(&mut store);
    let mut request = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "fresh",
        &sha(1),
        &sha(2),
    );
    replace_set(&mut store, &mut set);
    rejection(&mut store, &set, request.clone(), "VERSION_CONFLICT");
    request.change_set_version = set.version;
    assert_eq!(
        admit(&mut store, &actor("repo-1"), request)
            .unwrap()
            .change_set_id,
        set.change_set_id
    );
}

#[test]
fn opening_requires_a_canonical_baseline_repo_and_key() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    for (repo_id, key, base, code) in [
        ("repo-1", "new", "not-a-sha".into(), "GIT_SHA_INVALID"),
        (
            "repo-1",
            "new",
            sha(0xab).to_ascii_uppercase(),
            "GIT_SHA_INVALID",
        ),
        (" ", "new", sha(1), "INVALID_INPUT"),
        ("repo-1", " ", sha(1), "INVALID_INPUT"),
    ] {
        let err = open_change_set(
            &mut store,
            &actor(repo_id),
            repo_id,
            3,
            &base,
            key,
            &invocation("inv-1"),
        )
        .unwrap_err();
        assert_eq!(err.code, code);
        assert!(!err.recovery_action.is_empty());
    }
    assert!(store.list("changeset").unwrap().is_empty());
}
