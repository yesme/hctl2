mod common;
use common::*;
use repo::changeset::{
    ChangeSetRevision, OwnerGate, ProducerRef, Seal, admit, get_revision, list_revisions,
    open_change_set,
};
use store::{Actor, ActorSource, Scope, Store, TrustedActor};

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

fn seal(change_set_id: &str, lease_id: &str, key: &str, base: &str, tree: &str) -> Seal {
    Seal {
        association_key: key.into(),
        change_set_id: change_set_id.into(),
        lease_id: lease_id.into(),
        lease_generation: 1,
        base_commit_sha: base.into(),
        result_tree_sha: tree.into(),
        result_commit_sha: Some(sha(9)),
        parent_revision_id: None,
        producer_ref: invocation("inv-1"),
        owner: OwnerGate::Active,
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
        "inv-1",
    )
    .unwrap()
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
    let mut closed = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "closed",
        &sha(1),
        &sha(2),
    );
    closed.owner = OwnerGate::Cancelled;
    assert_eq!(
        admit(&mut store, &actor("repo-1"), closed)
            .unwrap_err()
            .code,
        "CHANGESET_OWNER_CLOSED"
    );
    let mut replaced = seal(
        &set.change_set_id,
        &set.lease.lease_id,
        "replaced",
        &sha(1),
        &sha(2),
    );
    replaced.owner = OwnerGate::Superseded;
    assert_eq!(
        admit(&mut store, &actor("repo-1"), replaced)
            .unwrap_err()
            .code,
        "CHANGESET_OWNER_CLOSED"
    );
    let mut stale = seal(&set.change_set_id, "lease-other", "stale", &sha(1), &sha(2));
    stale.lease_generation = 2;
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
