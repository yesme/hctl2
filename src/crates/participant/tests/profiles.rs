use agency_proto::{Capabilities, FrozenRef, hash};
use participant::profiles::*;
use serde_json::json;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use store::{Actor, ActorSource, RecordData, Scope, Store, TrustedActor};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Root(PathBuf);
impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
struct Env {
    store: Store,
    root: Root,
}
impl Env {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "hctl-profile-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        Self {
            store: Store::open(&root).unwrap(),
            root: Root(root),
        }
    }
    fn create(&mut self) -> serde_json::Value {
        let p = prepare_profile(
            &self.store,
            input(
                "create",
                ProfileAction::Create {
                    id: "research".into(),
                    profile: profile(),
                },
            ),
            &actor(),
        )
        .unwrap();
        admit_profile(&mut self.store, &actor(), p).unwrap()
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
fn profile() -> WorkerProfile {
    WorkerProfile {
        harness: FrozenRef {
            id: "script".into(),
            revision: "1".into(),
            digest: hash(b"script"),
        },
        model: "fixture".into(),
        mode: "read_only".into(),
        permissions: vec!["context.read".into()],
        environment: vec![],
        required_capabilities: Capabilities::default(),
        max_context_bytes: 65536,
    }
}
fn input(key: &str, action: ProfileAction) -> ProfileInput {
    ProfileInput {
        key: key.into(),
        action,
    }
}

#[test]
fn malformed_harness_is_local_invalid_input_not_an_agency_failure() {
    let e = Env::new();
    let mut bad = profile();
    bad.harness.digest = "not-a-digest".into();
    let error = prepare_profile(
        &e.store,
        input(
            "bad-harness",
            ProfileAction::Create {
                id: "research".into(),
                profile: bad,
            },
        ),
        &actor(),
    )
    .unwrap_err();
    assert_eq!(error.code, "INVALID_INPUT");
    assert_eq!(error.recovery_action, "correct_input");
    assert!(e.store.list("worker_profile_revision").unwrap().is_empty());
}

#[test]
fn update_moves_only_pointer_and_keeps_old_revision_usable() {
    let mut e = Env::new();
    let original = e.create();
    let old = serde_json::from_value(original["revision"].clone()).unwrap();
    let mut next = profile();
    next.max_context_bytes = 1024;
    let p = prepare_profile(
        &e.store,
        input(
            "update",
            ProfileAction::Update {
                id: "research".into(),
                version: 1,
                profile: next.clone(),
            },
        ),
        &actor(),
    )
    .unwrap();
    let current = admit_profile(&mut e.store, &actor(), p).unwrap();
    assert_eq!(current["version"], 2);
    assert_ne!(current["revision"], original["revision"]);
    assert_eq!(profile_at(&e.store, &old).unwrap().1, profile());
    assert_eq!(
        profile_at(
            &e.store,
            &serde_json::from_value(current["revision"].clone()).unwrap()
        )
        .unwrap()
        .1,
        next
    );
    assert_eq!(e.store.list("worker_profile_revision").unwrap().len(), 2);
}
#[test]
fn replay_is_bound_to_input_and_actor_and_survives_restart() {
    let mut e = Env::new();
    let original = e.create();
    let command = input(
        "create",
        ProfileAction::Create {
            id: "research".into(),
            profile: profile(),
        },
    );
    let p = prepare_profile(&e.store, command.clone(), &actor()).unwrap();
    assert_eq!(
        admit_profile(&mut e.store, &actor(), p.clone()).unwrap(),
        original
    );
    let mut other = actor();
    other.0.principal = "other".into();
    assert_eq!(
        prepare_profile(&e.store, command.clone(), &other)
            .unwrap_err()
            .code,
        "ACTOR_MISMATCH"
    );
    let mut changed = command;
    if let ProfileAction::Create { profile, .. } = &mut changed.action {
        profile.model = "changed".into();
    }
    assert_eq!(
        prepare_profile(&e.store, changed, &actor())
            .unwrap_err()
            .code,
        "IDEMPOTENCY_CONFLICT"
    );
    let root = e.root.0.clone();
    drop(e.store);
    e.store = Store::open(&root).unwrap();
    assert_eq!(admit_profile(&mut e.store, &actor(), p).unwrap(), original);
}
#[test]
fn stale_or_tampered_previews_do_not_write() {
    let mut e = Env::new();
    let i = input(
        "first",
        ProfileAction::Create {
            id: "research".into(),
            profile: profile(),
        },
    );
    let mut p = prepare_profile(&e.store, i.clone(), &actor()).unwrap();
    p.result = json!({"invented":true});
    assert_eq!(
        admit_profile(&mut e.store, &actor(), p).unwrap_err().code,
        "VERSION_CONFLICT"
    );
    assert!(e.store.list("worker_profile").unwrap().is_empty());
    let stale = prepare_profile(&e.store, i, &actor()).unwrap();
    e.create();
    assert_eq!(
        admit_profile(&mut e.store, &actor(), stale)
            .unwrap_err()
            .code,
        "VERSION_CONFLICT"
    );
    assert_eq!(e.store.list("worker_profile").unwrap().len(), 1);
}
#[test]
fn profiles_do_not_grant_command_or_write_authority() {
    let mut e = Env::new();
    for permission in [
        "command.submit",
        "task.complete",
        "room.dispatch",
        "git.write",
        "terminal.input",
    ] {
        let mut p = profile();
        p.permissions = vec![permission.into()];
        let i = input(
            permission,
            ProfileAction::Create {
                id: permission.into(),
                profile: p,
            },
        );
        assert_eq!(
            prepare_profile(&e.store, i, &actor()).unwrap_err().code,
            "PERMISSION_SCOPE_INVALID"
        );
    }
    let mut restricted = actor();
    restricted.0.permission_scope = vec![Scope::Project("A".into())];
    assert_eq!(
        prepare_profile(
            &e.store,
            input(
                "bad-scope",
                ProfileAction::Create {
                    id: "x".into(),
                    profile: profile()
                }
            ),
            &restricted
        )
        .unwrap_err()
        .code,
        "PERMISSION_DENIED"
    );
    assert!(e.store.list("worker_profile").unwrap().is_empty());
    e.create();
    let record = e.store.list("worker_profile_revision").unwrap().remove(0);
    assert!(matches!(record.data, RecordData::Value { .. }));
}
