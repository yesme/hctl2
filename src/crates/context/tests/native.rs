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
    let plan = chat::prepare(&store, input.clone(), vec![source_text.clone()], vec![]).unwrap();
    let created = chat::admit(&mut store, &trusted, plan).unwrap();
    let topic_room = created["room_id"].as_str().unwrap().to_owned();

    // Dispatch consumes the new Topic, not the room that held its sources.
    let (manifest, consumer) =
        select_room_manifest(&store, &trusted, "A", &topic_room, 64 * 1024).unwrap();
    assert_eq!(manifest.sources.len(), 2);
    let permitted = context::permitted_source_ids(&store, "A").unwrap();
    let assembler = LocalAssembler {
        permitted: permitted.into_iter().collect(),
        budget: 64 * 1024,
    };
    let first_sources = manifest.sources.clone();
    let assembly = assembler
        .assemble(
            &StoreSources::new(&store, &trusted, "A"),
            AssemblyRequest { manifest, consumer },
        )
        .unwrap();
    context::save_assembly(&mut store, &trusted, "A", "topic-first", &assembly).unwrap();
    let first_manifest = assembly.manifest.document.id.clone();
    let bundle = assembly.bundle.document;
    let source_entry = bundle
        .entries
        .iter()
        .find(|entry| entry.source.id.starts_with("chat_source_reference/"))
        .unwrap();
    match &source_entry.delivery {
        agency_proto::context::Delivery::Inline { bytes } => {
            assert_eq!(
                bytes,
                body.as_bytes(),
                "the source's body bytes must be delivered"
            );
        }
        other => panic!("expected inline delivery, got {other:?}"),
    }
    assert_eq!(source_entry.bytes_digest, hash(body.as_bytes()));
    assert!(bundle.entries.iter().any(|entry| matches!(&entry.delivery,
        agency_proto::context::Delivery::Inline { bytes } if String::from_utf8_lossy(bytes).contains("背景"))));

    // A sibling Topic shares the source Room, but neither its brief nor its
    // message belongs in the first Topic's selection.
    let mut sibling = input;
    sibling.key = "topic-2".into();
    let mut other_text = source_text;
    other_text.body = "{\"body\":\"sibling-only\",\"msgtype\":\"m.text\"}".into();
    other_text.excerpt = "sibling-only".into();
    if let chat::Source::Message {
        event_id,
        content_digest,
        ..
    } = &mut other_text.source
    {
        *event_id = "$source-2".into();
        *content_digest = hash(other_text.body.as_bytes());
    }
    if let Action::CreateTopic { brief, .. } = &mut sibling.action {
        brief.context_and_goal = "sibling-only".into();
        brief.sources = vec![other_text.source.clone()];
    }
    let plan = chat::prepare(&store, sibling, vec![other_text], vec![]).unwrap();
    let sibling_id = chat::admit(&mut store, &trusted, plan).unwrap()["room_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let (next, next_owner) =
        select_room_manifest(&store, &trusted, "A", &topic_room, 64 * 1024).unwrap();
    assert_eq!(next.sources, first_sources);
    assert_ne!(
        next.id, first_manifest,
        "changed frozen policy gets a new Manifest identity"
    );
    let assembler = LocalAssembler {
        permitted: context::permitted_source_ids(&store, "A")
            .unwrap()
            .into_iter()
            .collect(),
        budget: 64 * 1024,
    };
    let next = assembler
        .assemble(
            &StoreSources::new(&store, &trusted, "A"),
            AssemblyRequest {
                manifest: next,
                consumer: next_owner,
            },
        )
        .unwrap();
    context::save_assembly(&mut store, &trusted, "A", "topic-second", &next).unwrap();
    assert!(
        context::read_manifest(&store, &trusted, "A", &first_manifest)
            .unwrap()
            .is_some()
    );
    let (sibling_manifest, _) =
        select_room_manifest(&store, &trusted, "A", &sibling_id, 64 * 1024).unwrap();
    assert_ne!(sibling_manifest.sources, next.manifest.document.sources);
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
    let read_a = context::read_bundle(&store, &trusted, "A", &assembly_a.bundle.document.id)
        .unwrap()
        .expect("bundle A stored");
    let read_b = context::read_bundle(&store, &trusted, "A", &assembly_b.bundle.document.id)
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
    tampered.bundle.document.budget = 2048;
    tampered.bundle.document.manifest.digest = tampered.manifest.digest.clone();
    tampered.bundle.document.manifest.revision = tampered.manifest.digest.clone();
    tampered.bundle = agency_proto::Sealed::new(tampered.bundle.document).unwrap();
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
        context::Assembly {
            manifest: assembly_a.manifest.clone(),
            bundle: agency_proto::Sealed::new(bundle).unwrap(),
        }
    };
    assert_ne!(
        generation_two.bundle.document.id, assembly_a.bundle.document.id,
        "the next generation of the same owner must not collide with the frozen bundle"
    );
    context::save_assembly(
        &mut store,
        &trusted,
        "A",
        "ctx-generation-two",
        &generation_two,
    )
    .unwrap();
    assert_eq!(
        context::read_bundle(&store, &trusted, "A", &generation_two.bundle.document.id)
            .unwrap()
            .unwrap()
            .document
            .consumer
            .generation,
        2
    );
}

#[test]
fn selected_task_comments_are_project_local_exact_and_not_the_whole_board() {
    use context::{SelectionRequest, SourceKind, Sources, select_context};
    let root = std::env::temp_dir().join(format!(
        "hctl-context-task-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut store = Store::open(&root).unwrap();
    let mut trusted = actor();
    trusted
        .0
        .permission_scope
        .push(Scope::Repo("fixture-repo".into()));
    trusted
        .0
        .permission_scope
        .push(Scope::Repo("other-repo".into()));
    seed_project(&mut store, &trusted, "A");
    let room = json!({"project_id":"A","id":"main","name":"main","server":{
        "binding":store_reference("chat_server","fixture"), "url":"http://127.0.0.1:1","server_name":"fixture","sender":"owner"},
        "matrix_room_id":"!main:fixture","participants":[],"brief":null,"origin":null});
    seed(
        &mut store,
        &trusted,
        &Scope::Project("A".into()),
        "room_binding",
        "main",
        &room,
    );
    let source_key = store::ObjectKey {
        scope: Scope::Repo("fixture-repo".into()),
        kind: "task_source".into(),
        id: "issues".into(),
    };
    let source_ref = store::Reference {
        key: source_key.clone(),
        version: Version::State(1),
    };
    let source = json!({"id":"issues","repo_id":"fixture-repo","port_kind":"github",
        "candidate":{"id":"issues","provider":"github","actual_source_and_scope":"fixture","recommended":true,"create":true,"field_writeback":true,"can_claim":true,"available":true},
        "platform":{"instance":"github.com","stable_id":"1","full_name":"owner/fixture","clone_url":"https://github.com/owner/fixture.git","account_id":"1","has_issues":true,"can_write_issues":true,"credential_ref":""},
        "capabilities":{"create":true,"field_writeback":true,"conditional_write":false,"conditional_fields":[],"placement":true,"delete":false},
        "board_scope_stable_id":"board1","binding_revision":1,"active":true});
    seed(
        &mut store,
        &trusted,
        &source_key.scope,
        "task_source",
        "issues",
        &source,
    );
    seed(
        &mut store,
        &trusted,
        &Scope::Project("A".into()),
        "task_source_reference",
        "issues",
        &json!({"project_id":"A","source":source_ref,"approved_scope":"board1","group":null}),
    );
    let entity = json!({"provider":"github","account_stable_id":"1","external_entity_kind":"issue","immutable_external_entity_id":"card1"});
    let card = json!({"entity":entity,"number":1,"title":"task","body":"body","stage":"open","remote_revision":"r1","content_version":null,"groups":[],"dependencies":{"parent":null,"children":[],"blocked_by":[],"blocking":[]},"comments":[{"id":"c1","body":"only-the-bound-card"}],"raw":{},"tombstone":false});
    let mut other = card.clone();
    other["entity"]["immutable_external_entity_id"] = json!("card2");
    other["comments"] = json!([{"body":"other-card-secret"}]);
    let snapshot = json!({"source":source_ref,"observed_at":1,"complete":true,"error":null,"cards":[card,other],"stable_groups":[]});
    // Same source id in another Repo must never change the Project-local lookup.
    seed(
        &mut store,
        &trusted,
        &Scope::Repo("other-repo".into()),
        "task_snapshot",
        "issues",
        &json!({"private":"wrong-repo"}),
    );
    seed(
        &mut store,
        &trusted,
        &Scope::Repo("fixture-repo".into()),
        "task_snapshot",
        "issues",
        &snapshot,
    );
    let task = json!({"id":"T","project_id":"A","source_id":"issues","repo_id":"fixture-repo","entity":entity,"number":1,"title":"task","lifecycle":"open","lifecycle_version":1,"archived":false,"revision":null,"snapshot":null,"state_version":1,"needs_attention":false,"pending_contract":null,"run_occupancy":null});
    seed(
        &mut store,
        &trusted,
        &Scope::Project("A".into()),
        "task_state",
        "T",
        &task,
    );
    let consumer = Owner {
        project: "A".into(),
        kind: OwnerKind::RoomInvocation,
        id: "actual-invocation".into(),
        generation: 3,
    };
    let (manifest, chosen) = select_context(
        &store,
        &trusted,
        SelectionRequest {
            consumer: consumer.clone(),
            room_id: "main".into(),
            task_id: Some("T".into()),
            budget: 4096,
        },
    )
    .unwrap();
    assert_eq!(chosen, consumer);
    assert_eq!(manifest.sources.len(), 1);
    let adapter = StoreSources::new(&store, &trusted, "A");
    let source = adapter
        .exact(SourceKind::TaskComments, &manifest.sources[0])
        .unwrap();
    let provenance: serde_json::Value = serde_json::from_str(&source.reference.revision).unwrap();
    let snapshot_record = task::required(
        &store,
        &task::key(
            Scope::Repo("fixture-repo".into()),
            "task_snapshot",
            "issues",
        ),
    )
    .unwrap();
    assert_eq!(
        provenance["snapshot"]["reference"],
        json!(task::reference(&snapshot_record))
    );
    assert_eq!(
        provenance["snapshot"]["digest"],
        json!(
            context::frozen_from_record(&snapshot_record)
                .unwrap()
                .digest
        )
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&source.bytes).unwrap(),
        json!([{"id":"c1","body":"only-the-bound-card"}])
    );
    let mut stale = source.reference.clone();
    stale.revision = hash(b"wrong version");
    assert_eq!(
        adapter
            .exact(SourceKind::TaskComments, &stale)
            .unwrap_err()
            .code,
        "SOURCE_VERSION_CHANGED"
    );
    assert!(
        StoreSources::new(&store, &trusted, "B")
            .exact(SourceKind::TaskComments, &source.reference)
            .is_err()
    );
    assert!(
        adapter
            .exact(
                SourceKind::TaskComments,
                &reference("task_snapshot/issues", &hash(b"raw snapshot"))
            )
            .is_err()
    );
    let limited = TrustedActor(store::Actor {
        permission_scope: vec![Scope::Project("B".into())],
        ..trusted.0.clone()
    });
    assert_eq!(
        StoreSources::new(&store, &limited, "A")
            .exact(SourceKind::TaskComments, &source.reference)
            .unwrap_err()
            .code,
        "PERMISSION_DENIED"
    );
    let assembly = LocalAssembler {
        permitted: context::permitted_source_ids(&store, "A")
            .unwrap()
            .into_iter()
            .collect(),
        budget: 4096,
    }
    .assemble(
        &adapter,
        AssemblyRequest {
            manifest,
            consumer: chosen,
        },
    )
    .unwrap();
    context::save_assembly(&mut store, &trusted, "A", "task-context", &assembly).unwrap();
    let saved = context::read_bundle(&store, &trusted, "A", &assembly.bundle.document.id)
        .unwrap()
        .unwrap();
    assert_eq!(saved.document.consumer, consumer);
    assert_eq!(saved.document.entries[0].bytes_digest, hash(&source.bytes));

    let mut changed = snapshot;
    changed["cards"][0]["comments"] = json!([{"body":"new comment"}]);
    seed(
        &mut store,
        &trusted,
        &source_key.scope,
        "task_snapshot",
        "issues",
        &changed,
    );
    assert_eq!(
        StoreSources::new(&store, &trusted, "A")
            .exact(SourceKind::TaskComments, &source.reference)
            .unwrap_err()
            .code,
        "SOURCE_VERSION_CHANGED"
    );
    changed["complete"] = json!(false);
    seed(
        &mut store,
        &trusted,
        &source_key.scope,
        "task_snapshot",
        "issues",
        &changed,
    );
    assert_eq!(
        select_context(
            &store,
            &trusted,
            SelectionRequest {
                consumer: consumer.clone(),
                room_id: "main".into(),
                task_id: Some("T".into()),
                budget: 4096,
            }
        )
        .unwrap_err()
        .code,
        "READBACK_REQUIRED"
    );
    // A failed refresh does not rewrite an already frozen consumer's bytes.
    assert_eq!(
        context::read_bundle(&store, &trusted, "A", &assembly.bundle.document.id)
            .unwrap()
            .unwrap(),
        saved
    );
    changed["complete"] = json!(true);
    seed(
        &mut store,
        &trusted,
        &source_key.scope,
        "task_snapshot",
        "issues",
        &changed,
    );
    let mut foreign = task;
    foreign["repo_id"] = json!("other-repo");
    seed(
        &mut store,
        &trusted,
        &Scope::Project("A".into()),
        "task_state",
        "T",
        &foreign,
    );
    assert_eq!(
        select_context(
            &store,
            &trusted,
            SelectionRequest {
                consumer,
                room_id: "main".into(),
                task_id: Some("T".into()),
                budget: 4096,
            }
        )
        .unwrap_err()
        .code,
        "PERMISSION_DENIED"
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
    let object = store::ObjectKey {
        scope: scope.clone(),
        kind: kind.into(),
        id: id.into(),
    };
    let previous = store.get(&object).unwrap();
    let version = previous.as_ref().map_or(1, |r| r.version + 1);
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
    let record = record_value(version);
    let command = store::Command {
        command_id: format!("seed-{scope:?}-{kind}-{id}-{version}"),
        idempotency_key: format!("seed-{scope:?}-{kind}-{id}-{version}"),
        actor: actor.0.clone(),
        target: record.key.clone(),
        expected: previous.map_or(store::Expected::Absent, |r| {
            store::Expected::Exact(Version::State(r.version))
        }),
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
