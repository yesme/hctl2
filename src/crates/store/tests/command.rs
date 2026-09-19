mod common;
use common::*;
use serde_json::json;
use store::*;

#[test]
fn each_envelope_field_missing_is_rejected() {
    let original = serde_json::to_value(command("c", key("object"))).unwrap();
    for field in [
        "command_id",
        "idempotency_key",
        "actor",
        "target",
        "expected",
        "binding",
        "input_digest",
    ] {
        let mut value = original.clone();
        value.as_object_mut().unwrap().remove(field);
        assert!(
            serde_json::from_value::<Command>(value).is_err(),
            "missing {field}"
        );
    }
}

#[test]
fn empty_envelope_values_and_wrong_digest_are_rejected_before_reduction() {
    let temp = Temp::new();
    let mut store = temp.store();
    for field in [
        "command_id",
        "idempotency_key",
        "principal",
        "operation",
        "input_digest",
        "target",
        "binding",
    ] {
        let mut cmd = command("invalid", key("object"));
        match field {
            "command_id" => cmd.command_id.clear(),
            "idempotency_key" => cmd.idempotency_key.clear(),
            "principal" => cmd.actor.principal.clear(),
            "operation" => cmd.operation.clear(),
            "input_digest" => cmd.input_digest = "0".repeat(64),
            "target" => cmd.target.id.clear(),
            "binding" => cmd.binding.key.id.clear(),
            _ => unreachable!(),
        }
        let trusted = TrustedActor(cmd.actor.clone());
        assert_code(
            store.submit(store.generation(), &trusted, &cmd, None, |_| {
                panic!("invalid {field} reached reducer")
            }),
            "INVALID_INPUT",
        );
        for table in ["commands", "events", "inbox", "outbox"] {
            assert_eq!(count(&temp.path().join("control"), table), 0);
        }
    }
}

#[test]
fn replay_returns_original_result_and_changed_envelope_is_rejected() {
    let temp = Temp::new();
    let mut store = temp.store();
    let cmd = command("c", key("object"));
    let generation = store.generation();
    let result = store
        .submit(generation, &actor(), &cmd, None, |tx| {
            tx.put(&record("object", 1))?;
            tx.enqueue_effect(&effect("e"))?;
            Ok(json!({"accepted":1}))
        })
        .unwrap();
    assert_eq!(
        store
            .submit(generation, &actor(), &cmd, None, |_| panic!(
                "reducer ran twice"
            ))
            .unwrap(),
        result
    );
    for table in ["commands", "events", "outbox"] {
        assert_eq!(count(&temp.path().join("control"), table), 1);
    }
    let mut changed = cmd.clone();
    changed.input = json!({"changed":true});
    changed.input_digest = Command::digest_input(&changed.operation, &changed.input).unwrap();
    assert_code(
        store.submit(generation, &actor(), &changed, None, |_| panic!()),
        "IDEMPOTENCY_CONFLICT",
    );
    let mut foreign = actor();
    foreign.0.principal = "execution-principal".into();
    assert_code(
        store.submit(generation, &foreign, &cmd, None, |_| panic!()),
        "ACTOR_MISMATCH",
    );
    let mut stale = command("stale", key("object"));
    stale.expected = Expected::Exact(Version::State(0));
    assert_code(
        store.submit(generation, &actor(), &stale, None, |_| panic!()),
        "VERSION_CONFLICT",
    );
}

#[test]
fn second_writer_and_old_generation_are_rejected() {
    let temp = Temp::new();
    let store = temp.store();
    let old = store.generation();
    assert_code(Store::open(&temp.path().join("control")), "WRITER_BUSY");
    drop(store);
    let mut store = temp.store();
    assert!(store.generation().0 > old.0);
    assert_code(
        store.submit(
            old,
            &actor(),
            &command("c", key("object")),
            None,
            |_| panic!(),
        ),
        "STALE_WRITER",
    );
    assert_code(
        store.save_material(
            old,
            &actor(),
            &Scope::Project("p".into()),
            "c",
            "body",
            b"bytes",
        ),
        "STALE_WRITER",
    );
}

#[test]
fn inbox_is_deduplicated_in_the_command_transaction() {
    let temp = Temp::new();
    let mut store = temp.store();
    let cmd = command("c", key("object"));
    let input = InboxEntry {
        binding: cmd.binding.clone(),
        message_key: "event-1".into(),
        digest: "b".repeat(64),
    };
    store
        .submit(store.generation(), &actor(), &cmd, Some(&input), |tx| {
            tx.put(&record("object", 1))?;
            Ok(json!("once"))
        })
        .unwrap();
    let cmd2 = command("c2", key("object"));
    assert_eq!(
        store
            .submit(
                store.generation(),
                &actor(),
                &cmd2,
                Some(&input),
                |_| panic!()
            )
            .unwrap(),
        json!("once")
    );
    assert_eq!(count(&temp.path().join("control"), "events"), 1);
    assert_eq!(count(&temp.path().join("control"), "inbox"), 1);
    let mut updated = input.clone();
    updated.binding.version = Version::State(2);
    let mut replay = command("binding-v2-replay", key("object"));
    replay.binding = updated.binding.clone();
    replay.expected = Expected::Exact(Version::State(1));
    assert_eq!(
        store
            .submit(
                store.generation(),
                &actor(),
                &replay,
                Some(&updated),
                |_| panic!("binding version created another effect")
            )
            .unwrap(),
        json!("once")
    );
    for table in ["commands", "events", "inbox"] {
        assert_eq!(count(&temp.path().join("control"), table), 1);
    }
    let audit: String = rusqlite::Connection::open(temp.path().join("control/control.sqlite"))
        .unwrap()
        .query_row("SELECT binding FROM inbox", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        serde_json::from_str::<Reference>(&audit).unwrap(),
        input.binding
    );
    let altered = InboxEntry {
        digest: "c".repeat(64),
        ..updated
    };
    assert_code(
        store.submit(
            store.generation(),
            &actor(),
            &cmd2,
            Some(&altered),
            |_| panic!(),
        ),
        "INBOX_CONFLICT",
    );
}

#[test]
fn one_provider_event_can_be_processed_for_independent_project_targets() {
    let temp = Temp::new();
    let mut store = temp.store();
    let input = InboxEntry {
        binding: command("unused", key("unused")).binding,
        message_key: "same-event".into(),
        digest: "a".repeat(64),
    };
    for project in ["p", "q"] {
        let mut r = record("task", 1);
        r.key.scope = Scope::Project(project.into());
        let cmd = command(project, r.key.clone());
        store
            .submit(store.generation(), &actor(), &cmd, Some(&input), |tx| {
                tx.put(&r)?;
                Ok(json!(project))
            })
            .unwrap();
    }
    assert_eq!(count(&temp.path().join("control"), "inbox"), 2);
    assert_eq!(count(&temp.path().join("control"), "events"), 2);
}

#[test]
fn cross_module_failure_leaves_no_half_records() {
    let temp = Temp::new();
    let mut store = temp.store();
    let cmd = command("c", key("object"));
    let input = InboxEntry {
        binding: cmd.binding.clone(),
        message_key: "event".into(),
        digest: "a".repeat(64),
    };
    assert!(
        store
            .submit(store.generation(), &actor(), &cmd, Some(&input), |tx| {
                tx.put(&record("object", 1))?;
                tx.put(&record("target", 1))?;
                tx.enqueue_effect(&effect("e"))?;
                Err(StoreError {
                    code: "DOMAIN_REFUSED",
                    message: "fixture rejection".into(),
                    recovery_action: "fix",
                })
            })
            .is_err()
    );
    for table in ["objects", "events", "commands", "outbox", "inbox"] {
        assert_eq!(count(&temp.path().join("control"), table), 0);
    }
}

#[test]
fn ignored_mutation_error_cannot_commit_partial_command() {
    let temp = Temp::new();
    let mut store = temp.store();
    assert_code(
        store.submit(
            store.generation(),
            &actor(),
            &command("c", key("object")),
            None,
            |tx| {
                tx.put(&record("object", 1))?;
                tx.enqueue_effect(&effect("e"))?;
                let _ = tx.put(&record("object", 3));
                Ok(json!("caller ignored rejection"))
            },
        ),
        "COMMAND_ABORTED",
    );
    for table in ["objects", "events", "commands", "outbox"] {
        assert_eq!(count(&temp.path().join("control"), table), 0);
    }
}

#[test]
fn unknown_effect_retains_conflict_and_requires_exact_readback() {
    let temp = Temp::new();
    let mut store = temp.store();
    let e = effect("e");
    store
        .submit(
            store.generation(),
            &actor(),
            &command("c", key("object")),
            None,
            |tx| {
                tx.enqueue_effect(&e)?;
                Ok(json!(null))
            },
        )
        .unwrap();
    store.begin_effect(store.generation(), "e").unwrap();
    assert_code(
        store.begin_effect(store.generation(), "e"),
        "READBACK_REQUIRED",
    );
    assert_code(
        store.submit(
            store.generation(),
            &actor(),
            &command("other", key("other")),
            None,
            |tx| {
                tx.enqueue_effect(&effect("e2"))?;
                Ok(json!(null))
            },
        ),
        "EFFECT_CONFLICT",
    );
    let confirm = command("confirm", key("object"));
    assert_code(
        store.submit(store.generation(), &actor(), &confirm, None, |tx| {
            tx.confirm_effect(
                "e",
                &Readback::Confirmed {
                    binding: e.binding.clone(),
                    target: "wrong".into(),
                    input_digest: e.input_digest.clone(),
                    result: json!(true),
                },
            )?;
            Ok(json!(null))
        }),
        "READBACK_MISMATCH",
    );
    assert_eq!(store.effect("e").unwrap().1, EffectState::Unknown);
    store
        .submit(store.generation(), &actor(), &confirm, None, |tx| {
            tx.confirm_effect(
                "e",
                &Readback::Confirmed {
                    binding: e.binding.clone(),
                    target: e.target.clone(),
                    input_digest: e.input_digest.clone(),
                    result: json!(true),
                },
            )?;
            tx.put(&record("object", 1))?;
            Ok(json!(true))
        })
        .unwrap();
    assert_eq!(store.effect("e").unwrap().1, EffectState::Confirmed);
    assert_code(
        store.begin_effect(store.generation(), "e"),
        "READBACK_REQUIRED",
    );
}

#[test]
fn duplicate_transport_delivery_keeps_one_effect_and_the_frozen_idempotency_key() {
    let temp = Temp::new();
    let mut store = temp.store();
    let cmd = command("c", key("object"));
    store
        .submit(store.generation(), &actor(), &cmd, None, |tx| {
            tx.enqueue_effect(&effect("e"))?;
            Ok(json!(true))
        })
        .unwrap();
    let sent = store.begin_effect(store.generation(), "e").unwrap();
    let mut provider = rusqlite::Connection::open_in_memory().unwrap();
    provider
        .execute_batch(
            "CREATE TABLE sends(idempotency_key TEXT NOT NULL);
        CREATE TABLE effects(idempotency_key TEXT PRIMARY KEY, input_digest TEXT NOT NULL);",
        )
        .unwrap();
    let mut deliver = |intent: &EffectIntent| {
        // Test adapter for a provider with idempotent writes. Duplicate delivery is a
        // transport property, not permission for the kernel to resend an unknown action.
        let tx = provider.transaction().unwrap();
        tx.execute("INSERT INTO sends VALUES(?1)", [&intent.idempotency_key])
            .unwrap();
        tx.execute(
            "INSERT INTO effects VALUES(?1,?2) ON CONFLICT(idempotency_key) DO NOTHING",
            rusqlite::params![intent.idempotency_key, intent.input_digest],
        )
        .unwrap();
        let digest: String = tx
            .query_row(
                "SELECT input_digest FROM effects WHERE idempotency_key=?1",
                [&intent.idempotency_key],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(digest, intent.input_digest);
        tx.commit().unwrap();
    };
    deliver(&sent);
    drop(store); // The first response never reached the kernel.
    let mut store = temp.store();
    store
        .submit(store.generation(), &actor(), &cmd, None, |_| {
            panic!("command ran twice")
        })
        .unwrap();
    assert_code(
        store.begin_effect(store.generation(), "e"),
        "READBACK_REQUIRED",
    );
    let (frozen, state) = store.effect("e").unwrap();
    assert_eq!(state, EffectState::Unknown);
    assert_eq!(
        serde_json::to_value(&frozen).unwrap(),
        serde_json::to_value(&sent).unwrap()
    );
    // The duplicate of the already-sent request arrives, with the persisted intent unchanged.
    deliver(&frozen);
    for (table, expected) in [("sends", 2), ("effects", 1)] {
        let count: i64 = provider
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, expected);
    }
    assert_eq!(count(&temp.path().join("control"), "outbox"), 1);
    store
        .submit(
            store.generation(),
            &actor(),
            &command("confirm", key("object")),
            None,
            |tx| {
                tx.confirm_effect(
                    "e",
                    &Readback::Confirmed {
                        binding: frozen.binding.clone(),
                        target: frozen.target.clone(),
                        input_digest: frozen.input_digest.clone(),
                        result: json!(true),
                    },
                )?;
                Ok(json!(true))
            },
        )
        .unwrap();
    assert_eq!(store.effect("e").unwrap().1, EffectState::Confirmed);
}

// Re-enter this test in a child and exit without destructors. This exercises real SQLite
// process-loss windows, not just a reducer returning Err.
#[test]
fn crash_child() {
    let Ok(root) = std::env::var("HCTL_STORE_CRASH_ROOT") else {
        return;
    };
    let mode = std::env::var("HCTL_STORE_CRASH_MODE").unwrap();
    let mut store = Store::open(std::path::Path::new(&root)).unwrap();
    let cmd = command("c", key("object"));
    store
        .submit(store.generation(), &actor(), &cmd, None, |tx| {
            tx.put(&record("object", 1))?;
            tx.enqueue_effect(&effect("e"))?;
            if mode == "before_commit" {
                std::process::exit(73);
            }
            Ok(json!("accepted"))
        })
        .unwrap();
    if mode == "after_commit" {
        std::process::exit(73);
    }
    store.begin_effect(store.generation(), "e").unwrap();
    let remote =
        rusqlite::Connection::open(std::path::Path::new(&root).join("remote-fixture.sqlite"))
            .unwrap();
    remote
        .execute_batch(
            "CREATE TABLE effects(id TEXT PRIMARY KEY); INSERT INTO effects VALUES('e');",
        )
        .unwrap();
    if mode == "before_receipt_commit" {
        let e = effect("e");
        let mut c = command("confirm", key("object"));
        c.expected = Expected::Exact(Version::State(1));
        store
            .submit(store.generation(), &actor(), &c, None, |tx| {
                tx.confirm_effect(
                    "e",
                    &Readback::Confirmed {
                        binding: e.binding.clone(),
                        target: e.target.clone(),
                        input_digest: e.input_digest.clone(),
                        result: json!(true),
                    },
                )?;
                std::process::exit(73);
            })
            .unwrap();
    }
    std::process::exit(73);
}

#[test]
fn process_crashes_cover_commit_delivery_and_receipt_windows() {
    for mode in [
        "before_commit",
        "after_commit",
        "after_remote",
        "before_receipt_commit",
    ] {
        let temp = Temp::new();
        let root = temp.path().join("control");
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "crash_child", "--nocapture"])
            .env("HCTL_STORE_CRASH_ROOT", &root)
            .env("HCTL_STORE_CRASH_MODE", mode)
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(73));
        let mut store = Store::open(&root).unwrap();
        if mode == "before_commit" {
            for table in ["events", "commands", "outbox"] {
                assert_eq!(count(&root, table), 0);
            }
            continue;
        }
        assert_eq!(
            store
                .submit(
                    store.generation(),
                    &actor(),
                    &command("c", key("object")),
                    None,
                    |_| panic!()
                )
                .unwrap(),
            json!("accepted")
        );
        if mode == "after_commit" {
            assert_code(
                store.begin_effect(store.generation(), "e"),
                "READBACK_REQUIRED",
            );
            store
                .resume_pending_effect(store.generation(), "e", true)
                .unwrap();
            store.begin_effect(store.generation(), "e").unwrap();
            assert_code(
                store.begin_effect(store.generation(), "e"),
                "READBACK_REQUIRED",
            );
        } else {
            assert_eq!(store.effect("e").unwrap().1, EffectState::Unknown);
            assert_code(
                store.resume_pending_effect(store.generation(), "e", true),
                "READBACK_REQUIRED",
            );
            let remote = rusqlite::Connection::open(root.join("remote-fixture.sqlite")).unwrap();
            let effect_count: i64 = remote
                .query_row("SELECT count(*) FROM effects", [], |r| r.get(0))
                .unwrap();
            assert_eq!(effect_count, 1);
            let mut c = command("confirm", key("object"));
            c.expected = Expected::Exact(Version::State(1));
            let e = effect("e");
            let result = store
                .submit(store.generation(), &actor(), &c, None, |tx| {
                    tx.confirm_effect(
                        "e",
                        &Readback::Confirmed {
                            binding: e.binding.clone(),
                            target: e.target.clone(),
                            input_digest: e.input_digest.clone(),
                            result: json!(true),
                        },
                    )?;
                    Ok(json!(true))
                })
                .unwrap();
            assert_eq!(
                store
                    .submit(store.generation(), &actor(), &c, None, |_| panic!())
                    .unwrap(),
                result
            );
            assert_eq!(store.effect("e").unwrap().1, EffectState::Confirmed);
        }
    }
}
