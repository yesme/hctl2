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
fn profile_preview_rejects_empty_and_edge_whitespace_ids_without_writes() {
    let e = Env::new();
    for id in [
        "",
        " ",
        "\t",
        " research",
        "research ",
        " research ",
        "\nresearch",
        "research\u{00a0}",
    ] {
        for action in [
            ProfileAction::Create {
                id: id.into(),
                profile: profile(),
            },
            ProfileAction::Update {
                id: id.into(),
                version: 1,
                profile: profile(),
            },
        ] {
            let error = prepare_profile(&e.store, input("invalid-id", action), &actor())
                .expect_err("invalid Profile id must be rejected during preview");
            assert_eq!(error.code, "INVALID_INPUT", "id: {id:?}");
            assert_eq!(error.recovery_action, "correct_input", "id: {id:?}");
        }
    }
    for kind in [
        "worker_profile",
        "worker_profile_revision",
        "profile_command",
    ] {
        assert!(e.store.list(kind).unwrap().is_empty(), "{kind}");
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

#[test]
fn write_profile_previews_without_granting_a_lease_or_governance_authority() {
    let e = Env::new();
    let mut write = profile();
    write.mode = "write".into();
    write.permissions = vec!["context.read".into(), "git.read".into(), "git.write".into()];
    let plan = prepare_profile(
        &e.store,
        input(
            "writer",
            ProfileAction::Create {
                id: "writer".into(),
                profile: write,
            },
        ),
        &actor(),
    )
    .unwrap();
    assert_eq!(plan.input.action.kind(), "create");
    assert!(e.store.list("changeset").unwrap().is_empty());
    assert!(e.store.list("room_invocation").unwrap().is_empty());
    assert!(e.store.list("review_publish_intent").unwrap().is_empty());
}

#[test]
fn write_profiles_require_scoped_write_permission_and_cannot_grant_commands() {
    let e = Env::new();
    for permission in [
        "context.read",
        "command.submit",
        "task.complete",
        "integration.submit",
        "review.publish",
        "git.push",
        "terminal.input",
    ] {
        let mut write = profile();
        write.mode = "write".into();
        write.permissions = vec![permission.into()];
        let error = prepare_profile(
            &e.store,
            input(
                permission,
                ProfileAction::Create {
                    id: "writer".into(),
                    profile: write,
                },
            ),
            &actor(),
        )
        .unwrap_err();
        assert_eq!(error.code, "PERMISSION_SCOPE_INVALID", "{permission}");
    }
}

#[test]
fn frozen_legacy_herdr_profile_reads_back_unchanged_alongside_harness_resumes() {
    let mut e = Env::new();
    // Golden sample from Demo 3 acceptance (`2026-10-20-演示3验收-ubuntu.md`):
    // legacy write profile frozen when Profession.harness was `herdr` (`protocol-22`)
    // and model was `none`, with exact worker_profile_revision digest
    // `6b356f220a4949ee5dadfb8d90c0a0068fefe0096a69041a6c623ad49d69e78a`.
    let legacy_revision_id = "6b356f220a4949ee5dadfb8d90c0a0068fefe0096a69041a6c623ad49d69e78a";
    let legacy_profile = WorkerProfile {
        harness: FrozenRef {
            id: "herdr".into(),
            revision: "protocol-22".into(),
            digest: "18a8dc65f1c2fa485884344356dea1cfd911c6f06cf46fa78e193f4087f4dba7".into(),
        },
        model: "none".into(),
        mode: "write".into(),
        permissions: vec!["context.read".into(), "git.read".into(), "git.write".into()],
        environment: vec![],
        required_capabilities: Capabilities {
            stop: true,
            ..Capabilities::default()
        },
        max_context_bytes: 262_144,
    };
    let legacy_plan = prepare_profile(
        &e.store,
        input(
            "legacy-herdr-write",
            ProfileAction::Create {
                id: "legacy-write".into(),
                profile: legacy_profile.clone(),
            },
        ),
        &actor(),
    )
    .unwrap();
    let legacy_created = admit_profile(&mut e.store, &actor(), legacy_plan).unwrap();
    assert_eq!(legacy_created["revision"]["key"]["id"], legacy_revision_id);
    let legacy_ref = serde_json::from_value(legacy_created["revision"].clone()).unwrap();
    assert_eq!(profile_at(&e.store, &legacy_ref).unwrap().1, legacy_profile);

    let claude_plan = prepare_profile(
        &e.store,
        input(
            "claude-write",
            ProfileAction::Create {
                id: "demo3-claude-write".into(),
                profile: WorkerProfile {
                    harness: FrozenRef {
                        id: "claude-code".into(),
                        revision: "2.1.303 (Claude Code)".into(),
                        digest: "375988f457756524234695cf2d8f64ace99c5f68682e47ea24862a843c191450"
                            .into(),
                    },
                    model: "claude-code/default".into(),
                    ..legacy_profile.clone()
                },
            },
        ),
        &actor(),
    )
    .unwrap();
    let claude_created = admit_profile(&mut e.store, &actor(), claude_plan).unwrap();

    let codex_plan = prepare_profile(
        &e.store,
        input(
            "codex-write",
            ProfileAction::Create {
                id: "demo3-codex-write".into(),
                profile: WorkerProfile {
                    harness: FrozenRef {
                        id: "codex-cli".into(),
                        revision: "codex-cli 0.163.0".into(),
                        digest: "399ff69c431c066e40de318fba45c2b07f45a76b19939d247ee4828fc70c4215"
                            .into(),
                    },
                    model: "codex-cli/default".into(),
                    ..legacy_profile.clone()
                },
            },
        ),
        &actor(),
    )
    .unwrap();
    let codex_created = admit_profile(&mut e.store, &actor(), codex_plan).unwrap();

    assert_ne!(
        claude_created["revision"]["key"]["id"],
        codex_created["revision"]["key"]["id"]
    );
    assert_ne!(claude_created["revision"]["key"]["id"], legacy_revision_id);
    assert_ne!(codex_created["revision"]["key"]["id"], legacy_revision_id);
    // Already-frozen legacy Profile remains readable byte-for-byte without rewrite.
    assert_eq!(profile_at(&e.store, &legacy_ref).unwrap().1, legacy_profile);
}
