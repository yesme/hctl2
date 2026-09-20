mod common;
use common::*;
use serde_json::json;
use store::*;

#[test]
fn p21_repo_payload_round_trips_without_adding_a_registration_field() {
    let old = json!({"type":"repo", "platform_binding":null});
    let record: RecordData = serde_json::from_value(old.clone()).unwrap();
    assert_eq!(serde_json::to_value(record).unwrap(), old);
}

fn task(project: &str, id: &str) -> Record {
    let mut r = record(id, 1);
    r.key.scope = Scope::Project(project.into());
    r.key.kind = "task".into();
    r.data = RecordData::Task {
        entity: Some(ExternalEntity {
            provider: "github".into(),
            account_stable_id: "account".into(),
            external_entity_kind: "issue".into(),
            immutable_external_entity_id: "entity-id".into(),
        }),
        source: reference(ObjectKey {
            scope: Scope::Repo("r".into()),
            kind: "task_source".into(),
            id: "source".into(),
        }),
    };
    r
}

#[test]
fn task_mapping_is_unique_per_project_not_per_repo_binding_or_placement() {
    let temp = Temp::new();
    let mut store = temp.store();
    submit_record(&mut store, "p", &task("p", "task-p"));
    submit_record(&mut store, "q", &task("q", "task-q"));
    let duplicate = task("p", "other");
    assert_code(
        store.submit(
            store.generation(),
            &actor(),
            &command("duplicate", duplicate.key.clone()),
            None,
            |tx| {
                tx.put(&duplicate)?;
                Ok(json!(true))
            },
        ),
        "UNIQUENESS_CONFLICT",
    );
    let mut moved = task("p", "task-p");
    moved.version = 2;
    if let RecordData::Task {
        entity: Some(entity),
        ..
    } = &mut moved.data
    {
        entity.immutable_external_entity_id = "different-card".into();
    }
    let mut cmd = command("move", moved.key.clone());
    cmd.expected = Expected::Exact(Version::State(1));
    assert_code(
        store.submit(store.generation(), &actor(), &cmd, None, |tx| {
            tx.put(&moved)?;
            Ok(json!(true))
        }),
        "INVALID_INPUT",
    );
}

#[test]
fn same_repo_projects_and_shared_refs_keep_their_scopes() {
    let temp = Temp::new();
    let mut store = temp.store();
    for p in ["p", "q"] {
        let mut project = record(p, 1);
        project.key = ObjectKey {
            scope: Scope::Project(p.into()),
            kind: "project".into(),
            id: p.into(),
        };
        project.data = RecordData::Project {
            repo_id: "r".into(),
            settings: ProjectSettings {
                publish_review_requires_confirmation: false,
                selection_policy: json!({}),
            },
            archived: false,
        };
        project.sources.push(reference(ObjectKey {
            scope: Scope::Control,
            kind: "profile".into(),
            id: "shared".into(),
        }));
        submit_record(&mut store, p, &project);
        let mut room = record(&format!("main-{p}"), 1);
        room.key.scope = Scope::Project(p.into());
        room.key.kind = "room".into();
        room.data = RecordData::Room {
            room_kind: RoomKind::Main,
            state: RoomState::Active,
        };
        submit_record(&mut store, &format!("room-{p}"), &room);
        let mut duplicate = room.clone();
        duplicate.key.id = "another-main".into();
        assert_code(
            store.submit(
                store.generation(),
                &actor(),
                &command(&format!("dup-{p}"), duplicate.key.clone()),
                None,
                |tx| {
                    tx.put(&duplicate)?;
                    Ok(json!(true))
                },
            ),
            "UNIQUENESS_CONFLICT",
        );
    }
    let mut invalid = record("wrong", 1);
    invalid.key.scope = Scope::Repo("r".into());
    invalid.key.kind = "room".into();
    invalid.data = RecordData::Room {
        room_kind: RoomKind::Topic,
        state: RoomState::Active,
    };
    assert_code(
        store.submit(
            store.generation(),
            &actor(),
            &command("wrong", invalid.key.clone()),
            None,
            |tx| {
                tx.put(&invalid)?;
                Ok(json!(true))
            },
        ),
        "INVALID_INPUT",
    );
}

#[test]
fn room_three_states_and_project_settings_are_versioned_not_repo_policy() {
    let temp = Temp::new();
    let mut store = temp.store();
    let mut room = record("topic", 1);
    room.key.kind = "room".into();
    room.data = RecordData::Room {
        room_kind: RoomKind::Topic,
        state: RoomState::Active,
    };
    submit_record(&mut store, "open", &room);
    room.version = 2;
    room.data = RecordData::Room {
        room_kind: RoomKind::Topic,
        state: RoomState::ReadOnly,
    };
    submit_record(&mut store, "project-archive-fixture", &room);
    room.version = 3;
    room.data = RecordData::Room {
        room_kind: RoomKind::Topic,
        state: RoomState::Active,
    };
    submit_record(&mut store, "project-restore-fixture", &room);
    room.version = 4;
    room.data = RecordData::Room {
        room_kind: RoomKind::Topic,
        state: RoomState::Archived,
    };
    submit_record(&mut store, "close-fixture", &room);
    assert_eq!(store.versions(&room.key).unwrap().len(), 4);
    let mut project = record("p", 1);
    project.key.kind = "project".into();
    project.data = RecordData::Project {
        repo_id: "r".into(),
        settings: ProjectSettings {
            publish_review_requires_confirmation: false,
            selection_policy: json!({}),
        },
        archived: false,
    };
    submit_record(&mut store, "p1", &project);
    project.version = 2;
    if let RecordData::Project { settings, .. } = &mut project.data {
        settings.publish_review_requires_confirmation = true;
    }
    submit_record(&mut store, "p2", &project);
    assert!(
        matches!(&store.versions(&project.key).unwrap()[0].data,RecordData::Project{settings,..} if !settings.publish_review_requires_confirmation)
    );
}
