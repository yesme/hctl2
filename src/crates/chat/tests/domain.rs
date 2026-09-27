use chat::*;
use foundation::{bytes_sha256, canonical_json_sha256};
use serde_json::json;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use store::{
    Actor, ActorSource, Command, Expected, ProjectSettings, Record, RecordData, Reference,
    RoomKind, RoomState, Scope, Store, TrustedActor, Version,
};
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Env {
    store: Store,
    root: PathBuf,
}
impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
fn actor() -> TrustedActor {
    TrustedActor(Actor {
        principal: "owner".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![
            Scope::Control,
            Scope::Project("A".into()),
            Scope::Project("B".into()),
        ],
        authority: None,
    })
}
fn server() -> Server {
    Server {
        binding: Reference {
            key: key(Scope::Control, "chat_server", "local"),
            version: Version::State(1),
        },
        url: "http://127.0.0.1:8008".into(),
        server_name: "hctl2.localhost".into(),
        sender: "@hctl2_control:hctl2.localhost".into(),
    }
}
impl Env {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "hctl-chat-domain-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let store = Store::open(&root).unwrap();
        let mut e = Self { store, root };
        for project in ["A", "B"] {
            let data = RecordData::Project {
                repo_id: "same-repo".into(),
                settings: ProjectSettings {
                    publish_review_requires_confirmation: true,
                    selection_policy: json!({}),
                },
                archived: false,
            };
            let record = Record {
                key: key(Scope::Project(project.into()), "project", project),
                version: 1,
                revision_digest: canonical_json_sha256(&serde_json::to_value(&data).unwrap())
                    .unwrap(),
                data,
                sources: vec![],
                materials: vec![],
            };
            let (mut records, effect) = main_room(
                &record,
                server(),
                project.into(),
                &format!("create-{project}"),
            )
            .unwrap();
            records.insert(0, record);
            let command = e.command(&format!("seed-{project}"), records[0].key.clone());
            e.store
                .submit(e.store.generation(), &actor(), &command, None, |tx| {
                    for record in &records {
                        tx.put(record)?;
                    }
                    tx.enqueue_effect(&effect)?;
                    Ok(json!({}))
                })
                .unwrap();
            e.store
                .begin_effect(e.store.generation(), &effect.intent_id)
                .unwrap();
            confirm(
                &mut e.store,
                &actor(),
                &effect.intent_id,
                json!({"matrix_room_id":format!("!{project}:hctl2.localhost")}),
            )
            .unwrap();
        }
        e
    }
    fn command(&self, id: &str, target: store::ObjectKey) -> Command {
        Command {
            command_id: id.into(),
            idempotency_key: id.into(),
            actor: actor().0,
            target,
            expected: Expected::Absent,
            binding: server().binding,
            input_digest: Command::digest_input("fixture", &json!({})).unwrap(),
            operation: "fixture".into(),
            input: json!({}),
        }
    }
    fn put(&mut self, record: Record) {
        let mut c = self.command(
            &format!("put-{}-{}", record.key.id, record.version),
            record.key.clone(),
        );
        if record.version > 1 {
            c.expected = Expected::Exact(Version::State(record.version - 1));
        }
        self.store
            .submit(self.store.generation(), &actor(), &c, None, |tx| {
                tx.put(&record)?;
                Ok(json!({}))
            })
            .unwrap();
    }
    fn source(&self, project: &str) -> (Origin, SourceText) {
        let (r, room) = main_binding(&self.store, project).unwrap();
        let body = "{\"body\":\"尚未确定 é 与 é\",\"msgtype\":\"m.text\"}".to_owned();
        (
            Origin::MainRoom {
                room_id: room.id,
                binding_version: r.version,
            },
            SourceText {
                source: Source::Message {
                    binding: reference(&r),
                    event_id: "$original".into(),
                    content_digest: bytes_sha256(body.as_bytes()),
                },
                body,
                excerpt: "尚未确定 é 与 é".into(),
            },
        )
    }
    fn topic(&self, id: &str, origin: Origin, sources: &[SourceText]) -> Input {
        Input {
            key: id.into(),
            action: Action::CreateTopic {
                project_id: "A".into(),
                project_version: 1,
                name: "只讨论".into(),
                origin,
                brief: Brief {
                    context_and_goal: "人的补写，敏感信息已删".into(),
                    settled_facts_and_reasons: vec![],
                    disagreements_and_questions: vec!["结论未定".into()],
                    constraints_and_materials: vec![],
                    sources: sources.iter().map(|s| s.source.clone()).collect(),
                },
                participants: vec![],
                roster_confirmed: true,
            },
        }
    }
}

#[test]
fn independent_main_rooms_topic_human_edits_retry_close_and_restart() {
    let mut e = Env::new();
    assert_ne!(
        main_binding(&e.store, "A").unwrap().1.id,
        main_binding(&e.store, "B").unwrap().1.id
    );
    let (origin, source) = e.source("A");
    let input = e.topic("topic", origin, std::slice::from_ref(&source));
    let plan = prepare(&e.store, input.clone(), vec![source.clone()]).unwrap();
    assert_eq!(
        e.store.list("room").unwrap().len(),
        2,
        "preview must not create Room"
    );
    let result = admit(&mut e.store, &actor(), plan.clone()).unwrap();
    assert_eq!(admit(&mut e.store, &actor(), plan).unwrap(), result);
    let id = result["room_id"].as_str().unwrap();
    let (binding, room) = room(&e.store, "A", id).unwrap();
    let material = room.brief.unwrap();
    let bytes = e.store.read_material(&actor(), &material).unwrap();
    let brief: Brief = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(brief.context_and_goal, "人的补写，敏感信息已删");
    assert_eq!(brief.disagreements_and_questions, vec!["结论未定"]);
    assert_eq!(brief.sources, vec![source.source.clone()]);
    let close = prepare(
        &e.store,
        Input {
            key: "close".into(),
            action: Action::Close {
                project_id: "A".into(),
                room_id: id.into(),
                version: binding.version,
            },
        },
        vec![],
    )
    .unwrap();
    admit(&mut e.store, &actor(), close).unwrap();
    let identity = required(&e.store, &key(Scope::Project("A".into()), "room", id)).unwrap();
    assert!(matches!(
        identity.data,
        RecordData::Room {
            room_kind: RoomKind::Topic,
            state: RoomState::Archived
        }
    ));
    assert!(e.store.list("request").unwrap().is_empty());
    assert!(e.store.list("task").unwrap().is_empty());
    // Snapshot restore rebuilds bindings/materials, not an authoritative chat cache.
    let backup = e.root.join("backup");
    e.store.backup(e.store.generation(), &backup).unwrap();
    assert!(e.store.read_material(&actor(), &material).is_ok());
    let restored_root = e.root.join("restored");
    let restored = Store::restore(&restored_root, &backup).unwrap();
    assert_eq!(restored.read_material(&actor(), &material).unwrap(), bytes);
    let (closed, _) = chat::room(&restored, "A", id).unwrap();
    assert_eq!(
        prepare(
            &restored,
            Input {
                key: "closed-send".into(),
                action: Action::Send {
                    project_id: "A".into(),
                    room_id: id.into(),
                    version: closed.version,
                    body: "must not send".into()
                }
            },
            vec![]
        )
        .unwrap_err()
        .code,
        "ROOM_READ_ONLY"
    );
}

#[test]
fn mechanical_draft_is_verbatim_unsectioned_and_cannot_masquerade_as_summary() {
    let e = Env::new();
    let (_, source) = e.source("A");
    let allowed = vec![source.clone()];
    let mut draft = mechanical_draft(allowed.clone(), vec![]);
    verify_draft(&draft, &allowed).unwrap();
    assert_eq!(draft.automatic_summary, "not_configured");
    draft.fragments[0].excerpt = "已经确定".into();
    assert!(verify_draft(&draft, &allowed).is_err());
    draft.fragments[0] = source;
    draft.automatic_summary = "completed".into();
    assert!(verify_draft(&draft, &allowed).is_err());
}

#[test]
fn request_without_messages_has_exact_frozen_blockers_and_keeps_request_open() {
    let mut e = Env::new();
    let blocker = value_record(
        key(Scope::Project("A".into()), "run", "r"),
        1,
        &json!({"state":"waiting"}),
    )
    .unwrap();
    e.put(blocker.clone());
    let req = value_record(
        key(Scope::Project("A".into()), "request", "q"),
        1,
        &RequestSource {
            question: "要不要继续？".into(),
            blockers: vec![reference(&blocker)],
        },
    )
    .unwrap();
    e.put(req.clone());
    let origin = Origin::Request {
        request: reference(&req),
        blockers: vec![reference(&blocker)],
    };
    let sources = request_texts(&e.store, &reference(&req), &[reference(&blocker)]).unwrap();
    assert_eq!(
        mechanical_draft(sources.clone(), vec![]).fragments[0].excerpt,
        "要不要继续？"
    );
    let input = e.topic("request-topic", origin.clone(), &sources);
    let plan = prepare(&e.store, input, sources.clone()).unwrap();
    let result = admit(&mut e.store, &actor(), plan).unwrap();
    let room_id = result["room_id"].as_str().unwrap();
    let plan = prepare(
        &e.store,
        Input {
            key: "close-request-topic".into(),
            action: Action::Close {
                project_id: "A".into(),
                room_id: room_id.into(),
                version: 1,
            },
        },
        vec![],
    )
    .unwrap();
    admit(&mut e.store, &actor(), plan).unwrap();
    assert_eq!(required(&e.store, &req.key).unwrap().version, 1);
    let wrong = Origin::Request {
        request: reference(&req),
        blockers: vec![Reference {
            version: Version::State(2),
            ..reference(&blocker)
        }],
    };
    assert!(origin_checks(&e.store, "A", &wrong).is_err());
    assert!(origin_checks(&e.store, "B", &origin).is_err());
    assert!(validate_sources("A", &origin, &sources[..1]).is_err());
}

#[test]
fn stale_cross_project_unconfirmed_roster_and_bot_writes_rejected() {
    let mut e = Env::new();
    let (origin, source) = e.source("A");
    let mut input = e.topic("bad", origin.clone(), std::slice::from_ref(&source));
    if let Action::CreateTopic {
        roster_confirmed, ..
    } = &mut input.action
    {
        *roster_confirmed = false;
    }
    assert!(prepare(&e.store, input, vec![source.clone()]).is_err());
    let (_, foreign) = e.source("B");
    assert!(validate_sources("A", &origin, &[foreign]).is_err());
    let plan = prepare(
        &e.store,
        e.topic("stale", origin, std::slice::from_ref(&source)),
        vec![source],
    )
    .unwrap();
    let mut bot = actor();
    bot.0.source = ActorSource::ProviderEvent;
    assert_eq!(
        admit(&mut e.store, &bot, plan.clone()).unwrap_err().code,
        "PERMISSION_DENIED"
    );
    let mut project = required(&e.store, &key(Scope::Project("A".into()), "project", "A")).unwrap();
    project.version += 1;
    e.put(project);
    assert_eq!(
        admit(&mut e.store, &actor(), plan).unwrap_err().code,
        "VERSION_CONFLICT"
    );
}

#[test]
fn rebind_keeps_old_event_reference() {
    let mut e = Env::new();
    let (_, source) = e.source("A");
    let (r, main) = main_binding(&e.store, "A").unwrap();
    let freeze = Input {
        key: "freeze".into(),
        action: Action::Freeze {
            project_id: "A".into(),
            room_id: main.id.clone(),
            version: r.version,
            event_id: "$original".into(),
        },
    };
    let plan = prepare(&e.store, freeze, vec![source.clone()]).unwrap();
    admit(&mut e.store, &actor(), plan).unwrap();
    let old = e.store.list("chat_source_reference").unwrap()[0].clone();
    let plan = prepare(
        &e.store,
        Input {
            key: "upgrade".into(),
            action: Action::Rebind {
                project_id: "A".into(),
                room_id: main.id.clone(),
                version: r.version,
                matrix_room_id: "!upgraded:hctl2.localhost".into(),
            },
        },
        vec![],
    )
    .unwrap();
    admit(&mut e.store, &actor(), plan).unwrap();
    assert_eq!(main_binding(&e.store, "A").unwrap().1.id, main.id);
    assert_eq!(
        decode::<Source>(&required(&e.store, &old.key).unwrap()).unwrap(),
        source.source
    );
    assert_eq!(
        e.store.read_material(&actor(), &old.materials[0]).unwrap(),
        source.body.as_bytes()
    );
    let other = main_binding(&e.store, "B")
        .unwrap()
        .1
        .matrix_room_id
        .unwrap();
    assert!(unique_binding(&e.store, &main.id, &other).is_err());
}
