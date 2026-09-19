mod common;
use common::*;
use hctl2_store::*;
use serde_json::json;

#[test]
fn material_crash_child() {
    let Ok(root) = std::env::var("HCTL2_MATERIAL_CRASH_ROOT") else {
        return;
    };
    let mut store = Store::open(std::path::Path::new(&root)).unwrap();
    let material = store
        .save_material(
            store.generation(),
            &actor(),
            &Scope::Project("p".into()),
            "c",
            "body",
            b"contract",
        )
        .unwrap();
    if std::env::var("HCTL2_MATERIAL_CRASH_PHASE").unwrap() == "after-admission" {
        store
            .submit(
                store.generation(),
                &actor(),
                &command("c", key("object")),
                None,
                |tx| {
                    tx.admit_material(&material)?;
                    let mut rec = record("object", 1);
                    rec.materials.push(material);
                    tx.put(&rec)?;
                    Ok(json!("accepted"))
                },
            )
            .unwrap();
    }
    std::process::exit(73);
}

#[test]
fn process_loss_after_save_or_admission_keeps_bytes_without_double_admission() {
    for phase in ["after-save", "after-admission"] {
        let temp = Temp::new();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "material_crash_child", "--nocapture"])
            .env("HCTL2_MATERIAL_CRASH_ROOT", temp.path().join("control"))
            .env("HCTL2_MATERIAL_CRASH_PHASE", phase)
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(73));
        let mut store = temp.store();
        let material = store
            .save_material(
                store.generation(),
                &actor(),
                &Scope::Project("p".into()),
                "c",
                "body",
                b"contract",
            )
            .unwrap();
        if phase == "after-save" {
            assert_eq!(count(&temp.path().join("control"), "materials"), 0);
            assert_code(
                store.read_material(&actor(), &material),
                "MATERIAL_NOT_ADMITTED",
            );
        }
        store
            .submit(
                store.generation(),
                &actor(),
                &command("c", key("object")),
                None,
                |tx| {
                    assert_eq!(
                        phase, "after-save",
                        "lost admission response must return the original result"
                    );
                    tx.admit_material(&material)?;
                    let mut rec = record("object", 1);
                    rec.materials.push(material.clone());
                    tx.put(&rec)?;
                    Ok(json!("accepted"))
                },
            )
            .unwrap();
        assert_eq!(
            store.read_material(&actor(), &material).unwrap(),
            b"contract"
        );
        assert_eq!(count(&temp.path().join("control"), "materials"), 1);
        assert_eq!(count(&temp.path().join("control"), "events"), 1);
    }
}

#[test]
fn candidate_survives_restart_gc_and_response_loss_without_becoming_authority() {
    let temp = Temp::new();
    let mut store = temp.store();
    let generation = store.generation();
    let scope = Scope::Project("p".into());
    let reference = store
        .save_material(generation, &actor(), &scope, "c", "body", b"contract")
        .unwrap();
    assert_code(
        store.read_material(&actor(), &reference),
        "MATERIAL_NOT_ADMITTED",
    );
    assert!(store.versions(&key("object")).unwrap().is_empty());
    drop(store);
    let status = std::process::Command::new("git")
        .arg("-C")
        .arg(temp.path().join("control/materials.git"))
        .args(["gc", "--prune=now"])
        .status()
        .unwrap();
    assert!(status.success());
    store = temp.store();
    assert_eq!(
        store
            .save_material(
                store.generation(),
                &actor(),
                &scope,
                "c",
                "body",
                b"contract"
            )
            .unwrap(),
        reference
    );
    let mut rec = record("object", 1);
    rec.materials.push(reference.clone());
    let cmd = command("c", key("object"));
    store
        .submit(store.generation(), &actor(), &cmd, None, |tx| {
            tx.admit_material(&reference)?;
            tx.put(&rec)?;
            Ok(json!("accepted"))
        })
        .unwrap();
    drop(store);
    store = temp.store();
    assert_eq!(
        store
            .submit(store.generation(), &actor(), &cmd, None, |_| panic!())
            .unwrap(),
        json!("accepted")
    );
    assert_eq!(
        store.read_material(&actor(), &reference).unwrap(),
        b"contract"
    );
    assert_eq!(store.versions(&key("object")).unwrap().len(), 1);
    let another = store
        .save_material(
            store.generation(),
            &actor(),
            &scope,
            "other",
            "body",
            b"contract",
        )
        .unwrap();
    assert_eq!(reference.byte_digest, another.byte_digest);
    assert_ne!(reference.material_id, another.material_id);
    assert!(store.versions(&key("other")).unwrap().is_empty());
}

#[test]
fn admission_rechecks_permissions_versions_and_generation() {
    let temp = Temp::new();
    let mut store = temp.store();
    let old = store.generation();
    let reference = store
        .save_material(
            old,
            &actor(),
            &Scope::Project("p".into()),
            "c",
            "body",
            b"body",
        )
        .unwrap();
    submit_record(&mut store, "advance", &record("object", 1));
    assert_code(
        store.submit(old, &actor(), &command("c", key("object")), None, |tx| {
            tx.admit_material(&reference)?;
            Ok(json!(true))
        }),
        "VERSION_CONFLICT",
    );
    let mut denied = actor();
    denied.0.permission_scope = vec![Scope::Project("q".into())];
    assert_code(
        store.read_material(&denied, &reference),
        "PERMISSION_DENIED",
    );
    let mut cmd = command("c", key("new"));
    cmd.actor = denied.0.clone();
    assert_code(
        store.submit(old, &denied, &cmd, None, |_| panic!()),
        "PERMISSION_DENIED",
    );
    drop(store);
    let mut store = temp.store();
    assert_code(
        store.submit(old, &actor(), &command("c", key("new")), None, |tx| {
            tx.admit_material(&reference)?;
            Ok(json!(true))
        }),
        "STALE_WRITER",
    );
}

#[test]
fn delivery_requires_exact_grant_and_recipient_byte_readback() {
    let temp = Temp::new();
    let mut store = temp.store();
    let material = store
        .save_material(
            store.generation(),
            &actor(),
            &Scope::Project("p".into()),
            "c",
            "body",
            b"context",
        )
        .unwrap();
    let grant = DeliveryGrant {
        recipient: "consumer-1".into(),
        purpose: "dispatch".into(),
        materials: vec![material.clone()],
    };
    store
        .submit(
            store.generation(),
            &actor(),
            &command("c", key("object")),
            None,
            |tx| {
                tx.admit_material(&material)?;
                tx.enqueue_delivery("d", &grant)?;
                Ok(json!(true))
            },
        )
        .unwrap();
    assert!(!store.delivery_confirmed("d").unwrap());
    assert_code(
        store.delivery_bytes("d", "consumer-2", "dispatch"),
        "PERMISSION_DENIED",
    );
    assert_eq!(
        store.delivery_bytes("d", "consumer-1", "dispatch").unwrap(),
        vec![b"context".to_vec()]
    );
    assert_code(
        store.submit(
            store.generation(),
            &actor(),
            &command("ack", key("object")),
            None,
            |tx| {
                tx.confirm_delivery("d", "consumer-1", "dispatch", &[b"digest only".to_vec()])?;
                Ok(json!(true))
            },
        ),
        "MATERIAL_DIGEST_MISMATCH",
    );
    store
        .submit(
            store.generation(),
            &actor(),
            &command("ack", key("object")),
            None,
            |tx| {
                tx.confirm_delivery("d", "consumer-1", "dispatch", &[b"context".to_vec()])?;
                Ok(json!(true))
            },
        )
        .unwrap();
}

#[test]
fn missing_locator_or_bytes_is_reported_not_recreated() {
    for remove_locator in [false, true] {
        let temp = Temp::new();
        let mut store = temp.store();
        let material = store
            .save_material(
                store.generation(),
                &actor(),
                &Scope::Project("p".into()),
                "c",
                "body",
                b"body",
            )
            .unwrap();
        let mut rec = record("object", 1);
        rec.materials.push(material.clone());
        store
            .submit(
                store.generation(),
                &actor(),
                &command("c", key("object")),
                None,
                |tx| {
                    tx.admit_material(&material)?;
                    tx.put(&rec)?;
                    Ok(json!(true))
                },
            )
            .unwrap();
        if remove_locator {
            rusqlite::Connection::open(temp.path().join("control/control.sqlite"))
                .unwrap()
                .execute("DELETE FROM materials", [])
                .unwrap();
            assert_code(
                store.read_material(&actor(), &material),
                "MATERIAL_NOT_ADMITTED",
            );
        } else {
            let status = std::process::Command::new("git")
                .arg("-C")
                .arg(temp.path().join("control/materials.git"))
                .args([
                    "update-ref",
                    "-d",
                    &format!("refs/hctl2/materials/{}", material.material_id),
                ])
                .status()
                .unwrap();
            assert!(status.success());
            assert_code(
                store.read_material(&actor(), &material),
                "MATERIAL_UNAVAILABLE",
            );
        }
        assert!(
            store
                .backup(store.generation(), &temp.path().join("backup"))
                .is_err()
        );
        assert_eq!(store.versions(&key("object")).unwrap().len(), 1);
    }
}
