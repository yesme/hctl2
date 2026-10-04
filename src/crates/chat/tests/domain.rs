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
            Scope::Repo("same-repo".into()),
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
            Origin::Room {
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
                origin: Box::new(origin),
                brief: Box::new(Brief {
                    context_and_goal: "人的补写，敏感信息已删".into(),
                    settled_facts_and_reasons: vec![],
                    disagreements_and_questions: vec!["结论未定".into()],
                    constraints_and_materials: vec![],
                    sources: sources.iter().map(|s| s.source.clone()).collect(),
                }),
                participants: vec![],
                roster_confirmed: true,
                invites: None,
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
    let plan = prepare(&e.store, input.clone(), vec![source.clone()], vec![]).unwrap();
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
                    body: "must not send".into(),
                    thread_root: None
                }
            },
            vec![],
            vec![]
        )
        .unwrap_err()
        .code,
        "ROOM_READ_ONLY"
    );
}

#[test]
fn close_cancels_only_undispatched_creation_and_retains_unknown_readback() {
    for attempted in [false, true] {
        let mut e = Env::new();
        let (origin, source) = e.source("A");
        let plan = prepare(
            &e.store,
            e.topic("unfinished", origin, std::slice::from_ref(&source)),
            vec![source],
            vec![],
        )
        .unwrap();
        let created = admit(&mut e.store, &actor(), plan).unwrap();
        let id = created["room_id"].as_str().unwrap();
        let effect = created["effect_id"].as_str().unwrap();
        if attempted {
            e.store.begin_effect(e.store.generation(), effect).unwrap();
        }
        let close = prepare(
            &e.store,
            Input {
                key: "close-unfinished".into(),
                action: Action::Close {
                    project_id: "A".into(),
                    room_id: id.into(),
                    version: 1,
                },
            },
            vec![],
            vec![],
        )
        .unwrap();
        admit(&mut e.store, &actor(), close.clone()).unwrap();
        assert_eq!(
            e.store.effect(effect).unwrap().1,
            if attempted {
                store::EffectState::Unknown
            } else {
                store::EffectState::Cancelled
            }
        );
        assert_eq!(
            e.store
                .pending_effects()
                .unwrap()
                .contains(&effect.to_owned()),
            attempted
        );
        // Replay must not resurrect a cancelled create or change the archived identity.
        admit(&mut e.store, &actor(), close).unwrap();
        let identity = required(&e.store, &key(Scope::Project("A".into()), "room", id)).unwrap();
        assert!(matches!(
            identity.data,
            RecordData::Room {
                state: RoomState::Archived,
                ..
            }
        ));
        if attempted {
            confirm(
                &mut e.store,
                &actor(),
                effect,
                json!({"matrix_room_id":"!original:hctl2.localhost"}),
            )
            .unwrap();
            assert_eq!(
                e.store.effect(effect).unwrap().1,
                store::EffectState::Confirmed
            );
            assert_eq!(
                required(&e.store, &identity.key).unwrap().version,
                identity.version
            );
        } else {
            assert!(e.store.begin_effect(e.store.generation(), effect).is_err());
        }
    }
}

#[test]
fn stale_close_does_not_cancel_creation_or_archive_room() {
    let mut e = Env::new();
    let (origin, source) = e.source("A");
    let plan = prepare(
        &e.store,
        e.topic("stale-close", origin, std::slice::from_ref(&source)),
        vec![source],
        vec![],
    )
    .unwrap();
    let created = admit(&mut e.store, &actor(), plan).unwrap();
    let id = created["room_id"].as_str().unwrap();
    let effect = created["effect_id"].as_str().unwrap();
    let close = prepare(
        &e.store,
        Input {
            key: "stale-close-command".into(),
            action: Action::Close {
                project_id: "A".into(),
                room_id: id.into(),
                version: 1,
            },
        },
        vec![],
        vec![],
    )
    .unwrap();
    let (binding, current) = room(&e.store, "A", id).unwrap();
    e.put(value_record(binding.key, 2, &current).unwrap());
    assert_eq!(
        admit(&mut e.store, &actor(), close).unwrap_err().code,
        "VERSION_CONFLICT"
    );
    assert_eq!(
        e.store.effect(effect).unwrap().1,
        store::EffectState::Pending
    );
    assert!(matches!(
        required(&e.store, &key(Scope::Project("A".into()), "room", id))
            .unwrap()
            .data,
        RecordData::Room {
            state: RoomState::Active,
            ..
        }
    ));
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
    let plan = prepare(&e.store, input, sources.clone(), vec![]).unwrap();
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
    let (_, message) = e.source("A");
    let mut supplemented = sources.clone();
    supplemented.push(message);
    assert!(
        prepare(
            &e.store,
            e.topic("supplemented", origin.clone(), &supplemented),
            supplemented.clone(),
            vec![]
        )
        .is_ok()
    );
    let (_, foreign) = e.source("B");
    supplemented.push(foreign);
    assert!(
        prepare(
            &e.store,
            e.topic("foreign-message", origin.clone(), &supplemented),
            supplemented,
            vec![]
        )
        .is_err()
    );
    let shared = value_record(
        key(Scope::Repo("same-repo".into()), "write_lease", "lease"),
        1,
        &json!({"state":"waiting"}),
    )
    .unwrap();
    e.put(shared.clone());
    let req = value_record(
        key(Scope::Project("A".into()), "request", "repo-blocker"),
        1,
        &RequestSource {
            question: "等待共享租约".into(),
            blockers: vec![reference(&shared)],
        },
    )
    .unwrap();
    e.put(req.clone());
    assert!(
        origin_checks(
            &e.store,
            "A",
            &Origin::Request {
                request: reference(&req),
                blockers: vec![reference(&shared)]
            }
        )
        .is_ok()
    );
}

#[test]
fn topic_can_be_source_and_closing_parent_does_not_close_child_or_share_roster() {
    let mut e = Env::new();
    let (origin, source) = e.source("A");
    let input = e.topic("parent", origin, std::slice::from_ref(&source));
    let plan = prepare(&e.store, input, vec![source], vec![]).unwrap();
    let parent = admit(&mut e.store, &actor(), plan).unwrap();
    let parent_id = parent["room_id"].as_str().unwrap();
    let effect_id = parent["effect_id"].as_str().unwrap();
    e.store
        .begin_effect(e.store.generation(), effect_id)
        .unwrap();
    confirm(
        &mut e.store,
        &actor(),
        effect_id,
        json!({"matrix_room_id":"!parent:hctl2.localhost"}),
    )
    .unwrap();
    let (r, _) = room(&e.store, "A", parent_id).unwrap();
    let text = SourceText {
        source: Source::Message {
            binding: reference(&r),
            event_id: "$topic-event".into(),
            content_digest: bytes_sha256(b"topic"),
        },
        body: "topic".into(),
        excerpt: "topic".into(),
    };
    let origin = Origin::Room {
        room_id: parent_id.into(),
        binding_version: r.version,
    };
    let plan = prepare(
        &e.store,
        e.topic("nested", origin.clone(), std::slice::from_ref(&text)),
        vec![text],
        vec![],
    )
    .unwrap();
    assert_eq!(plan.effects[0].input["parent"]["id"], parent_id);
    let child = admit(&mut e.store, &actor(), plan).unwrap();
    let child_id = child["room_id"].as_str().unwrap();
    let close = prepare(
        &e.store,
        Input {
            key: "parent-close".into(),
            action: Action::Close {
                project_id: "A".into(),
                room_id: parent_id.into(),
                version: r.version,
            },
        },
        vec![],
        vec![],
    )
    .unwrap();
    admit(&mut e.store, &actor(), close).unwrap();
    let (_, child) = room(&e.store, "A", child_id).unwrap();
    assert!(writable(&e.store, &child).is_ok());
    assert_eq!(child.origin, Some(origin));
    assert!(child.participants.is_empty());
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
    assert!(prepare(&e.store, input, vec![source.clone()], vec![]).is_err());
    let (_, foreign) = e.source("B");
    assert!(validate_sources("A", &origin, &[foreign]).is_err());
    let plan = prepare(
        &e.store,
        e.topic("stale", origin, std::slice::from_ref(&source)),
        vec![source],
        vec![],
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
fn topic_roster_cannot_inherit_another_rooms_selection() {
    let mut e = Env::new();
    let (origin, source) = e.source("A");
    let mut input = e.topic("roster", origin, std::slice::from_ref(&source));
    for (selection_id, owner_room, accepted) in [
        ("other", main_binding(&e.store, "A").unwrap().1.id, false),
        ("own", topic_id("A", "roster"), true),
    ] {
        let selection = value_record(
            key(Scope::Project("A".into()), "room_selection", selection_id),
            1,
            &json!({"room_id":owner_room}),
        )
        .unwrap();
        let r = reference(&selection);
        e.put(selection);
        if let Action::CreateTopic { participants, .. } = &mut input.action {
            *participants = vec![r];
        }
        assert_eq!(
            prepare(&e.store, input.clone(), vec![source.clone()], vec![]).is_ok(),
            accepted
        );
    }
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
    let plan = prepare(&e.store, freeze, vec![source.clone()], vec![]).unwrap();
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

#[test]
fn topic_invites_freeze_the_confirmed_list_and_render_the_opening_message() {
    let e = Env::new();
    let (origin, source) = e.source("A");
    let brief = Brief {
        context_and_goal: "目标：确认名单与开场消息".into(),
        settled_facts_and_reasons: vec!["已定：缺省名单来自来源 Room 的人类成员".into()],
        disagreements_and_questions: vec!["待答：无".into()],
        constraints_and_materials: vec![],
        sources: vec![source.source.clone()],
    };
    let body = opening_body(&brief);
    assert!(body.contains("话题与目标\n目标：确认名单与开场消息"));
    assert!(body.contains("- 已定：缺省名单来自来源 Room 的人类成员"));
    let Source::Message {
        binding, event_id, ..
    } = &source.source
    else {
        panic!("message source required");
    };
    let rendered_source = format!("- 消息：房间绑定 {}，事件 {event_id}", binding.key.id);
    assert!(body.contains(&rendered_source), "{body}");
    let input = |invites: Option<Vec<String>>, key: &str| Input {
        key: key.into(),
        action: Action::CreateTopic {
            project_id: "A".into(),
            project_version: 1,
            name: "邀请".into(),
            origin: Box::new(origin.clone()),
            brief: Box::new(brief.clone()),
            participants: vec![],
            roster_confirmed: true,
            invites,
        },
    };
    // Explicit confirmed list governs: removed defaults get no effect,
    // additions are included, nobody outside the list is invited.
    let plan = prepare(
        &e.store,
        input(
            Some(vec![
                "@carol:hctl2.localhost".into(),
                "@alice:hctl2.localhost".into(),
                "@alice:hctl2.localhost".into(),
            ]),
            "invite-list",
        ),
        vec![source.clone()],
        vec![
            "@alice:hctl2.localhost".into(),
            "@bob:hctl2.localhost".into(),
        ],
    )
    .unwrap();
    assert_eq!(
        plan.invites,
        vec![
            "@alice:hctl2.localhost".to_owned(),
            "@carol:hctl2.localhost".to_owned(),
        ],
        "confirmed list is sorted, deduplicated and exactly what was submitted"
    );
    assert_eq!(
        plan.invite_defaults,
        vec![
            "@alice:hctl2.localhost".to_owned(),
            "@bob:hctl2.localhost".to_owned(),
        ]
    );
    let members: Vec<&store::EffectIntent> = plan
        .effects
        .iter()
        .filter(|effect| effect.operation == "chat.members")
        .collect();
    assert_eq!(members.len(), 2, "one effect per confirmed member");
    let invited: Vec<String> = members
        .iter()
        .map(|effect| effect.input["users"][0].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(invited, plan.invites);
    assert!(!invited.contains(&"@bob:hctl2.localhost".to_owned()));
    // Every invite effect targets exactly this Room with its own conflict
    // scope, so independent retries cannot block each other.
    for effect in members {
        assert_eq!(
            effect.input["room"]["id"],
            json!(plan.result["room_id"]),
            "invite effects target the new Room snapshot"
        );
        assert!(effect.conflict_scope.contains(":invite:"));
    }
    // The opening message effect carries the deterministic rendering and a
    // stable key, ordered after creation in the same transaction.
    let opening: Vec<&store::EffectIntent> = plan
        .effects
        .iter()
        .filter(|effect| effect.operation == "chat.opening")
        .collect();
    assert_eq!(opening.len(), 1);
    assert_eq!(opening[0].input["body"], json!(body));
    assert_eq!(
        plan.result["opening_effect_id"],
        json!(opening[0].intent_id)
    );
    assert_eq!(
        plan.effects[0].operation, "chat.create",
        "creation is dispatched first"
    );
    // Default list when the input carries none.
    let plan = prepare(
        &e.store,
        input(None, "invite-default"),
        vec![source.clone()],
        vec!["@alice:hctl2.localhost".into()],
    )
    .unwrap();
    assert_eq!(plan.invites, vec!["@alice:hctl2.localhost".to_owned()]);
    // Invalid user IDs are rejected instead of silently dropped.
    assert_eq!(
        prepare(
            &e.store,
            input(Some(vec!["not-a-user".into()]), "invite-invalid"),
            vec![source.clone()],
            vec![],
        )
        .unwrap_err()
        .code,
        "INVALID_INPUT"
    );
}
