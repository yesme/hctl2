//! Acceptance cases from the review: the main path must actually run against
//! a real Store — a Topic created through the chat command path becomes a
//! previewable manifest whose bundle carries the source's body bytes; and two
//! consumers of one manifest save and read back independently.

use agency_proto::context::Manifest;
use agency_proto::{FrozenRef, Owner, OwnerKind, hash};
use chat::{Action, Input, Origin, SourceText};
use context::{
    Assembler, AssemblyRequest, LocalAssembler, MemorySources, StoreSources, bundle_id,
    permission_digest, select_room_manifest,
};
use serde_json::json;
use store::{RecordData, RoomKind, RoomState, Scope, Store, TrustedActor, Version};

fn actor() -> TrustedActor {
    TrustedActor(store::Actor {
        principal: "owner".into(),
        source: store::ActorSource::DirectClient,
        permission_scope: vec![Scope::Control, Scope::Project("A".into())],
        authority: None,
    })
}

fn reference(id: &str, digest: &str) -> FrozenRef {
    FrozenRef {
        id: id.into(),
        revision: digest.into(),
        digest: digest.into(),
    }
}

fn store_reference(kind: &str, id: &str) -> store::Reference {
    store::Reference {
        key: store::ObjectKey {
            scope: Scope::Control,
            kind: kind.into(),
            id: id.into(),
        },
        version: Version::State(1),
    }
}

/// Acceptance 1: real Store, real chat command path, preview by object ID.
#[test]
fn topic_created_through_chat_previews_with_source_bytes() {
    let root = std::env::temp_dir().join(format!(
        "hctl-context-native-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let mut store = Store::open(&root).unwrap();
    let trusted = actor();
    // Seed the project and its bound main Room (the same shape chat's own
    // domain tests use).
    let binding_reference = store::Reference {
        key: store::ObjectKey {
            scope: Scope::Control,
            kind: "chat_server".into(),
            id: "test".into(),
        },
        version: Version::State(1),
    };
    let server = chat::Server {
        binding: binding_reference,
        url: "http://127.0.0.1:1".into(),
        server_name: "hctl2.localhost".into(),
        sender: "@hctl2_control:hctl2.localhost".into(),
    };
    seed_project(&mut store, &trusted, "A");
    let mut main = chat::Room {
        project_id: "A".into(),
        id: "main-1".into(),
        name: "主房间".into(),
        server: server.clone(),
        matrix_room_id: Some("!main:hctl2.localhost".into()),
        participants: vec![],
        brief: None,
        origin: None,
    };
    main.matrix_room_id = Some("!main:hctl2.localhost".into());
    seed(
        &mut store,
        &trusted,
        &Scope::Project("A".into()),
        "room_binding",
        &main.id,
        &serde_json::to_value(&main).unwrap(),
    );
    seed_room_identity(&mut store, &trusted, "A", &main.id);
    // Create a Topic through the real chat command path with one source; the
    // admit stores the chat_source_reference record and its material bytes.
    let (binding_record, _) = chat::main_binding(&store, "A").unwrap();
    let body = "{\"body\":\"决策原文\",\"msgtype\":\"m.text\"}".to_owned();
    let source_text = SourceText {
        source: chat::Source::Message {
            binding: chat::reference(&binding_record),
            event_id: "$source-1".into(),
            content_digest: hash(body.as_bytes()),
        },
        body: body.clone(),
        excerpt: "决策原文".into(),
    };
    let input = Input {
        key: "topic-1".into(),
        action: Action::CreateTopic {
            project_id: "A".into(),
            project_version: 1,
            name: "话题".into(),
            origin: Box::new(Origin::Room {
                room_id: main.id.clone(),
                binding_version: binding_record.version,
            }),
            brief: Box::new(chat::Brief {
                context_and_goal: "背景".into(),
                settled_facts_and_reasons: vec![],
                disagreements_and_questions: vec![],
                constraints_and_materials: vec![],
                sources: vec![source_text.source.clone()],
            }),
            participants: vec![],
            roster_confirmed: true,
            invites: Some(vec![]),
        },
    };
    let plan = chat::prepare(&store, input, vec![source_text], vec![]).unwrap();
    let created = chat::admit(&mut store, &trusted, plan).unwrap();
    let topic_room = created["room_id"].as_str().unwrap().to_owned();

    // The chat_source_reference is bound to the main Room's binding (the
    // source message lives there): selecting the main Room finds it.
    let (manifest, consumer) =
        select_room_manifest(&store, &trusted, "A", &main.id, 64 * 1024).unwrap();
    assert_eq!(manifest.sources.len(), 1);
    assert!(manifest.sources[0].id.starts_with("chat_source_reference/"));
    let permitted = context::permitted_source_ids(&store, "A").unwrap();
    assert_eq!(permitted.len(), 1);
    let assembler = LocalAssembler {
        permitted: permitted.into_iter().collect(),
        budget: 64 * 1024,
    };
    let assembly = assembler
        .assemble(
            &StoreSources::new(&store, &trusted, "A"),
            AssemblyRequest { manifest, consumer },
        )
        .unwrap();
    let bundle = assembly.bundle.document;
    assert_eq!(bundle.entries.len(), 1);
    match &bundle.entries[0].delivery {
        agency_proto::context::Delivery::Inline { bytes } => {
            assert_eq!(
                bytes,
                body.as_bytes(),
                "the source's body bytes must be delivered"
            );
        }
        other => panic!("expected inline delivery, got {other:?}"),
    }
    assert_eq!(bundle.entries[0].bytes_digest, hash(body.as_bytes()));
    // A room with no frozen sources must be an honest error, not an empty
    // bundle.
    let error = select_room_manifest(&store, &trusted, "A", &topic_room, 1024).unwrap_err();
    assert_eq!(error.code, "SOURCE_UNAVAILABLE");
}

/// Acceptance 2: one manifest, two consumers — each saves and reads back;
/// the same id with different content is a conflict.
#[test]
fn two_consumers_save_independently_and_conflicts_are_detected() {
    let root = std::env::temp_dir().join(format!(
        "hctl-context-native2-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let mut store = Store::open(&root).unwrap();
    let trusted = actor();
    let make = |consumer: &str| {
        let source_bytes = b"line one".to_vec();
        let room = reference("chat_source_reference/s1", &hash(&source_bytes));
        let mut sources = MemorySources::new();
        sources.push(context::SourceKind::Room, room.clone(), &source_bytes);
        let manifest = Manifest {
            id: "manifest-1".into(),
            purpose: "shared review".into(),
            scope: "project A".into(),
            parent: None,
            sources: vec![room],
            selection_policy: reference("policy/p1", &hash(b"p")),
            freshness: "now".into(),
            coverage: "room line".into(),
            known_gaps: vec![],
            required_skills: vec![],
            permission_digest: permission_digest(&["chat_source_reference/s1".to_owned()]),
            redaction: reference("redaction/none", &hash(b"r")),
            budget: 1024,
        };
        let assembler = LocalAssembler {
            permitted: ["chat_source_reference/s1".to_owned()]
                .into_iter()
                .collect(),
            budget: 1024,
        };
        let owner = Owner {
            project: "A".into(),
            kind: OwnerKind::RoomInvocation,
            id: consumer.into(),
            generation: 1,
        };
        assembler
            .assemble(
                &sources,
                AssemblyRequest {
                    manifest,
                    consumer: owner,
                },
            )
            .unwrap()
    };
    let assembly_a = make("invocation-a");
    let assembly_b = make("invocation-b");
    // Consumer A saves, then consumer B with the same manifest: B's bundle
    // must be stored and readable — not swallowed as a replay.
    let saved_a = context::save_assembly(&mut store, &trusted, "A", "ctx-a", &assembly_a).unwrap();
    assert_eq!(saved_a["replayed"], json!(false));
    let saved_b = context::save_assembly(&mut store, &trusted, "A", "ctx-b", &assembly_b).unwrap();
    assert_eq!(saved_b["replayed"], json!(false));
    assert_ne!(assembly_a.bundle.document.id, assembly_b.bundle.document.id);
    let read_a = context::read_bundle(&store, "A", &assembly_a.bundle.document.id)
        .unwrap()
        .expect("bundle A stored");
    let read_b = context::read_bundle(&store, "A", &assembly_b.bundle.document.id)
        .unwrap()
        .expect("bundle B stored");
    read_a.verify().unwrap();
    read_b.verify().unwrap();
    assert_eq!(read_a.document.consumer.id, "invocation-a");
    assert_eq!(read_b.document.consumer.id, "invocation-b");
    // Same manifest id, different content → conflict, not a replay.
    let mut tampered = make("invocation-a");
    tampered.manifest.document.budget = 2048;
    tampered.manifest = agency_proto::Sealed::new(tampered.manifest.document).unwrap();
    let error = context::save_assembly(&mut store, &trusted, "A", "ctx-a2", &tampered).unwrap_err();
    assert_eq!(error.code, "CONTEXT_CONFLICT");
    // Bundle ids include the consumer generation: the next generation of the
    // same owner gets its own bundle, no collision.
    let generation_two = {
        let mut bundle = assembly_a.bundle.document.clone();
        bundle.consumer.generation = 2;
        bundle.id = bundle_id(
            &assembly_a.bundle.document.manifest.digest,
            &bundle.consumer,
        );
        bundle.id.clone()
    };
    assert_ne!(
        generation_two, assembly_a.bundle.document.id,
        "the next generation of the same owner must not collide with the frozen bundle"
    );
}

fn seed(
    store: &mut Store,
    actor: &TrustedActor,
    scope: &Scope,
    kind: &str,
    id: &str,
    data: &serde_json::Value,
) {
    use serde_json::value::Value as Json;
    let record_value = |version: i64| -> store::Record {
        store::Record {
            key: store::ObjectKey {
                scope: scope.clone(),
                kind: kind.into(),
                id: id.into(),
            },
            version,
            revision_digest: foundation::canonical_json_sha256(data).unwrap(),
            data: RecordData::Value {
                value: data.clone(),
            },
            sources: vec![],
            materials: vec![],
        }
    };
    let _ = Json::Null;
    let record = record_value(1);
    let command = store::Command {
        command_id: format!("seed-{kind}-{id}"),
        idempotency_key: format!("seed-{kind}-{id}"),
        actor: actor.0.clone(),
        target: record.key.clone(),
        expected: store::Expected::Absent,
        binding: store_reference("module", "context"),
        input_digest: store::Command::digest_input("fixture.seed", &json!({"id": id})).unwrap(),
        operation: "fixture.seed".into(),
        input: json!({"id": id}),
    };
    store
        .submit(store.generation(), actor, &command, None, |tx| {
            tx.put(&record)?;
            Ok(json!({}))
        })
        .unwrap();
}

fn seed_project(store: &mut Store, actor: &TrustedActor, project: &str) {
    let data = RecordData::Project {
        repo_id: "fixture-repo".into(),
        settings: store::ProjectSettings {
            publish_review_requires_confirmation: false,
            selection_policy: json!({}),
        },
        archived: false,
    };
    let record = store::Record {
        key: store::ObjectKey {
            scope: Scope::Project(project.into()),
            kind: "project".into(),
            id: project.into(),
        },
        version: 1,
        revision_digest: foundation::canonical_json_sha256(&serde_json::to_value(&data).unwrap())
            .unwrap(),
        data,
        sources: vec![],
        materials: vec![],
    };
    let command = store::Command {
        command_id: format!("seed-project-{project}"),
        idempotency_key: format!("seed-project-{project}"),
        actor: actor.0.clone(),
        target: record.key.clone(),
        expected: store::Expected::Absent,
        binding: store_reference("module", "context"),
        input_digest: store::Command::digest_input("fixture.seed", &json!({"project": project}))
            .unwrap(),
        operation: "fixture.seed".into(),
        input: json!({"project": project}),
    };
    store
        .submit(store.generation(), actor, &command, None, |tx| {
            tx.put(&record)?;
            Ok(json!({}))
        })
        .unwrap();
}

fn seed_room_identity(store: &mut Store, actor: &TrustedActor, project: &str, room: &str) {
    let key = store::ObjectKey {
        scope: Scope::Project(project.into()),
        kind: "room".into(),
        id: room.into(),
    };
    let data = RecordData::Room {
        room_kind: RoomKind::Main,
        state: RoomState::Active,
    };
    let record = store::Record {
        key,
        version: 1,
        revision_digest: foundation::canonical_json_sha256(&serde_json::to_value(&data).unwrap())
            .unwrap(),
        data,
        sources: vec![],
        materials: vec![],
    };
    let command = store::Command {
        command_id: format!("seed-room-{room}"),
        idempotency_key: format!("seed-room-{room}"),
        actor: actor.0.clone(),
        target: record.key.clone(),
        expected: store::Expected::Absent,
        binding: store_reference("module", "context"),
        input_digest: store::Command::digest_input("fixture.seed", &json!({"room": room})).unwrap(),
        operation: "fixture.seed".into(),
        input: json!({"room": room}),
    };
    store
        .submit(store.generation(), actor, &command, None, |tx| {
            tx.put(&record)?;
            Ok(json!({}))
        })
        .unwrap();
}

// Silence unused warnings for helpers used by one of the two tests.
#[allow(dead_code)]
fn _unused() {
    let _ = bundle_id(
        "",
        &Owner {
            project: String::new(),
            kind: OwnerKind::RoomInvocation,
            id: String::new(),
            generation: 0,
        },
    );
    let _: Version = Version::State(1);
}
