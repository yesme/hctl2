mod common;
use common::*;
use hctl2_store::*;
use serde_json::json;

#[test]
fn empty_store_backup_restores_without_content_services_or_agency_definitions() {
    let temp = Temp::new();
    let mut store = temp.store();
    let backup = temp.path().join("backup");
    let report = store.backup(store.generation(), &backup).unwrap();
    assert_eq!(report.promised_material_count, 0);
    let restored = Store::restore(&temp.path().join("moved"), &backup).unwrap();
    assert_eq!(restored.control_id(), store.control_id());
}

#[test]
fn deleting_projection_rebuilds_from_events_including_sources() {
    let temp = Temp::new();
    let mut store = temp.store();
    let mut r = record("object", 1);
    r.sources.push(reference(ObjectKey {
        scope: Scope::Control,
        kind: "profile".into(),
        id: "shared".into(),
    }));
    submit_record(&mut store, "c1", &r);
    r.version = 2;
    submit_record(&mut store, "c2", &r);
    let before = serde_json::to_value(store.get(&key("object")).unwrap()).unwrap();
    rusqlite::Connection::open(temp.path().join("control/control.sqlite"))
        .unwrap()
        .execute_batch("DROP TABLE objects")
        .unwrap();
    store.rebuild_projections(store.generation()).unwrap();
    assert_eq!(
        serde_json::to_value(store.get(&key("object")).unwrap()).unwrap(),
        before
    );
    drop(store);
    let store = temp.store();
    assert_eq!(store.versions(&key("object")).unwrap().len(), 2);
}

#[test]
fn backup_is_independent_and_restore_advances_past_live_and_backup_generations() {
    let temp = Temp::new();
    let mut store = temp.store();
    let id = store.control_id().to_owned();
    let reference = store
        .save_material(
            store.generation(),
            &actor(),
            &Scope::Project("p".into()),
            "c",
            "body",
            b"context bundle",
        )
        .unwrap();
    let mut rec = record("bundle", 1);
    rec.key.kind = "context_bundle".into();
    rec.materials.push(reference.clone());
    store
        .submit(
            store.generation(),
            &actor(),
            &command("c", rec.key.clone()),
            None,
            |tx| {
                tx.admit_material(&reference)?;
                tx.put(&rec)?;
                tx.enqueue_effect(&effect("e"))?;
                Ok(json!(true))
            },
        )
        .unwrap();
    // Agency Skill installation is deliberately absent; its frozen declaration is a record.
    let mut skill = record("skill", 1);
    skill.key.scope = Scope::Control;
    skill.key.kind = "skill".into();
    submit_record(&mut store, "skill-c", &skill);
    let backup = temp.path().join("backup");
    store.backup(store.generation(), &backup).unwrap();
    assert!(!backup.join("control.sqlite-wal").exists());
    assert_code(
        Store::restore(&temp.path().join("control"), &backup),
        "WRITER_BUSY",
    );
    let generation = store.generation();
    drop(store);
    let live = temp.store();
    let newer = live.generation();
    drop(live);
    let mut restored = Store::restore(&temp.path().join("control"), &backup).unwrap();
    assert_eq!(restored.control_id(), id);
    assert!(restored.generation().0 > newer.0);
    assert_eq!(
        restored.read_material(&actor(), &reference).unwrap(),
        b"context bundle"
    );
    assert_code(
        restored.submit(
            generation,
            &actor(),
            &command("old", key("x")),
            None,
            |_| panic!(),
        ),
        "STALE_WRITER",
    );
    assert_code(
        restored.begin_effect(restored.generation(), "e"),
        "READBACK_REQUIRED",
    );
    let unrelated = Temp::new();
    let other = unrelated.store();
    drop(other);
    assert_code(
        Store::restore(&unrelated.path().join("control"), &backup),
        "RESTORE_IDENTITY_CONFLICT",
    );
    drop(restored);
    std::fs::rename(
        temp.path().join("control/materials.git"),
        temp.path().join("saved-live-materials.git"),
    )
    .unwrap();
    // Backup remains usable with the entire live backend missing, and restores to another machine/path.
    Store::verify_backup(&backup).unwrap();
    let moved = Store::restore(&temp.path().join("moved"), &backup).unwrap();
    assert_eq!(moved.control_id(), id);
    assert_eq!(
        moved.read_material(&actor(), &reference).unwrap(),
        b"context bundle"
    );
}

#[test]
fn missing_promised_bundle_or_profile_definition_invalidates_backup() {
    for kind in ["context_bundle", "profile"] {
        let temp = Temp::new();
        let mut store = temp.store();
        let reference = store
            .save_material(
                store.generation(),
                &actor(),
                &Scope::Control,
                "c",
                "definition",
                b"precise definition",
            )
            .unwrap();
        let mut rec = record("definition", 1);
        rec.key.scope = Scope::Control;
        rec.key.kind = kind.into();
        rec.materials.push(reference.clone());
        store
            .submit(
                store.generation(),
                &actor(),
                &command("c", rec.key.clone()),
                None,
                |tx| {
                    tx.admit_material(&reference)?;
                    tx.put(&rec)?;
                    Ok(json!(true))
                },
            )
            .unwrap();
        let backup = temp.path().join("backup");
        store.backup(store.generation(), &backup).unwrap();
        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(backup.join("materials.git"))
            .args([
                "update-ref",
                "-d",
                &format!("refs/hctl2/materials/{}", reference.material_id),
            ])
            .status()
            .unwrap();
        assert!(status.success());
        assert!(Store::verify_backup(&backup).is_err());
    }
}
