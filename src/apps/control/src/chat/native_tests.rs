//! Native Tuwunel in an isolated directory; no mutations of the developer's chat server.
use super::*;
use crate::chat::{access, drive_using, tree};
use chat::{Action, Input, Origin, Room};
use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Arc,
};
use store::{Actor, ActorSource, Expected, Scope, Store, TrustedActor};
use tokio::sync::Mutex;
struct Native {
    root: PathBuf,
    child: Child,
}
impl Drop for Native {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
fn find(root: &Path) -> Option<PathBuf> {
    for e in std::fs::read_dir(root).ok()? {
        let p = e.ok()?.path();
        if p.file_name()? == "tuwunel" && p.is_file() {
            return Some(p);
        }
        if p.is_dir()
            && let Some(p) = find(&p)
        {
            return Some(p);
        }
    }
    None
}

#[test]
fn native_matrix_create_send_resync_freeze_account_data_and_encryption() {
    let root = std::env::temp_dir().join(format!(
        "hctl-chat-native-{}-{}",
        std::process::id(),
        ruma::TransactionId::new()
    ));
    std::fs::create_dir_all(root.join("appservices")).unwrap();
    let callback = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    callback.set_nonblocking(true).unwrap();
    let callback_port = callback.local_addr().unwrap().port();
    let server_socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = server_socket.local_addr().unwrap().port();
    drop(server_socket);
    let registration = json!({"id":"test","url":format!("http://127.0.0.1:{callback_port}"),"as_token":"as-test-secret","hs_token":"hs-test-secret",
        "sender_localpart":"hctl2_control","namespaces":{"users":[{"exclusive":true,"regex":"^@hctl2_.*"}],"aliases":[{"exclusive":true,"regex":"^#hctl2_.*"}],"rooms":[]},"rate_limited":false});
    std::fs::write(root.join("appservices/test.yaml"), registration.to_string()).unwrap();
    let inbox = crate::chat::inbox::Inbox {
        root: root.clone(),
        token: "hs-test-secret".into(),
    };
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let http = std::thread::spawn(move || {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                axum::serve(
                    tokio::net::TcpListener::from_std(callback).unwrap(),
                    inbox.router(),
                )
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .unwrap();
            });
    });
    let config = root.join("tuwunel.toml");
    std::fs::write(&config,format!("[global]\nserver_name = \"hctl2.localhost\"\ndatabase_path = \"{}/data\"\nappservice_dir = \"{}/appservices\"\naddress = [\"127.0.0.1\"]\nport = {port}\nallow_federation = false\nallow_registration = true\nregistration_token = \"test-registration\"\nallow_encryption = true\nlog_colors = false\nlog_journald = false\n",root.display(),root.display())).unwrap();
    let log = std::fs::File::create(root.join("server.log")).unwrap();
    let binary = find(Path::new(env!("HCTL2_TEST_TUWUNEL"))).expect("pinned Tuwunel missing");
    let child = Command::new(binary)
        .arg("--config")
        .arg(&config)
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .spawn()
        .unwrap();
    let mut native = Native {
        root: root.clone(),
        child,
    };
    for _ in 0..300 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        if let Some(status) = native.child.try_wait().unwrap() {
            panic!(
                "Tuwunel exited {status}: {}",
                std::fs::read_to_string(root.join("server.log")).unwrap()
            );
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let server = Server {
        binding: Reference {
            key: chat::key(store::Scope::Control, "chat_server", "test"),
            version: store::Version::State(1),
        },
        url: format!("http://127.0.0.1:{port}"),
        server_name: "hctl2.localhost".into(),
        sender: "@hctl2_control:hctl2.localhost".into(),
    };
    let client = Client::new(server.clone(), "as-test-secret".into()).unwrap();
    assert_eq!(
        client.register_virtual_user("hctl2_worker").unwrap(),
        "@hctl2_worker:hctl2.localhost"
    );
    assert!(client.register_virtual_user("human").is_err());
    let room = chat::Room {
        project_id: "p".into(),
        id: "main".into(),
        name: "测试房间".into(),
        server,
        matrix_room_id: None,
        participants: vec![],
        brief: None,
        origin: None,
    };
    let created = client.create(&room, "creation-one", true).unwrap();
    assert_eq!(
        client.create(&room, "creation-one", false).unwrap(),
        created
    );
    let recovered = client.create(&room, "unknown-command", false).unwrap();
    assert_eq!(
        client.create(&room, "unknown-command", false).unwrap(),
        recovered
    );
    // Lost public alias: the creator's native joined-room inventory still recovers
    // the exact initial marker, rather than creating another message room.
    client
        .request(ruma::api::client::alias::delete_alias::v3::Request::new(
            ruma::RoomAliasId::parse(format!(
                "#hctl2_{}:{}",
                foundation::bytes_sha256(b"unknown-command"),
                room.server.server_name
            ))
            .unwrap(),
        ))
        .unwrap();
    assert_eq!(
        client.create(&room, "unknown-command", false).unwrap(),
        recovered
    );
    let external = created["matrix_room_id"].as_str().unwrap();
    let users = vec!["@hctl2_worker:hctl2.localhost".to_owned()];
    assert_eq!(
        client
            .members(external, &users, true, false)
            .unwrap_err()
            .code,
        "RESULT_UNKNOWN"
    );
    assert_eq!(
        client.members(external, &users, true, true).unwrap()["members"][0]["membership"],
        "invite"
    );
    assert_eq!(
        client.members(external, &users, true, false).unwrap()["members"][0]["membership"],
        "invite"
    );
    assert_eq!(
        client.members(external, &users, false, true).unwrap()["members"][0]["membership"],
        "leave"
    );
    assert_eq!(
        client.members(external, &users, false, false).unwrap()["members"][0]["membership"],
        "leave"
    );
    let mut bound_main = room.clone();
    bound_main.matrix_room_id = Some(external.into());
    let space = client.ensure_carrier(&bound_main).unwrap();
    assert_eq!(client.ensure_carrier(&bound_main).unwrap(), space);
    assert!(client.state(&space, "m.room.create").unwrap()["type"] == "m.space");
    let mut child = chat::Room {
        id: "topic".into(),
        name: "话题".into(),
        ..room.clone()
    };
    child.matrix_room_id = Some(
        client.create(&child, "child-command", true).unwrap()["matrix_room_id"]
            .as_str()
            .unwrap()
            .into(),
    );
    client
        .attach(&space, child.matrix_room_id.as_deref().unwrap(), false)
        .unwrap();
    let child_space = client.ensure_carrier(&child).unwrap();
    // Crash one step earlier: wrapper attachment exists but its pointer is not yet stored.
    // Recovery must not mistake the wrapper itself for an old external parent.
    let mut interrupted = chat::Room {
        id: "interrupted-carrier".into(),
        ..room.clone()
    };
    interrupted.matrix_room_id = Some(
        client
            .create(&interrupted, "interrupted-room", true)
            .unwrap()["matrix_room_id"]
            .as_str()
            .unwrap()
            .into(),
    );
    let carrier_key = format!(
        "carrier:{}:{}:{}",
        interrupted.project_id,
        interrupted.id,
        interrupted.matrix_room_id.as_deref().unwrap()
    );
    let wrapper = client
        .create_native(&interrupted, &carrier_key, true, false, || Ok(()))
        .unwrap()["matrix_room_id"]
        .as_str()
        .unwrap()
        .to_owned();
    client
        .attach(
            &wrapper,
            interrupted.matrix_room_id.as_deref().unwrap(),
            true,
        )
        .unwrap();
    assert_eq!(client.ensure_carrier(&interrupted).unwrap(), wrapper);
    assert!(
        !client.room_state(&wrapper).unwrap().iter().any(|event| {
            event["type"] == "m.space.child"
                && event["state_key"] == wrapper
                && event["content"]["via"]
                    .as_array()
                    .is_some_and(|via| !via.is_empty())
        }),
        "recovering the wrapper must not create a self-edge"
    );
    let child_state = client
        .room_state(child.matrix_room_id.as_deref().unwrap())
        .unwrap();
    let old_parent = child_state
        .iter()
        .find(|event| event["type"] == "m.space.parent" && event["state_key"] == space)
        .unwrap();
    assert!(
        old_parent["content"]["via"]
            .as_array()
            .is_none_or(Vec::is_empty)
    );
    assert!(child_state.iter().any(|event| {
        event["type"] == "m.space.parent"
            && event["state_key"] == child_space
            && event["content"]["via"]
                .as_array()
                .is_some_and(|via| !via.is_empty())
    }));
    let mut grandchild = chat::Room {
        id: "nested".into(),
        ..room.clone()
    };
    grandchild.matrix_room_id = Some(
        client.create(&grandchild, "nested-command", true).unwrap()["matrix_room_id"]
            .as_str()
            .unwrap()
            .into(),
    );
    client
        .attach(
            &child_space,
            grandchild.matrix_room_id.as_deref().unwrap(),
            false,
        )
        .unwrap();
    let rooms = vec![bound_main.clone(), child.clone(), grandchild.clone()];
    let projection = crate::chat::tree::project_hierarchy(&client, &rooms);
    assert_eq!(projection["nested"].parents[0]["room_id"], "topic");
    assert_eq!(projection["topic"].parents[0]["room_id"], "main");
    assert!(!projection["main"].children.contains(&"main".into()));
    // A fresh projection has no cache. Both non-canonical and both canonical parents survive.
    for canonical in [false, true] {
        client
            .attach(
                &space,
                grandchild.matrix_room_id.as_deref().unwrap(),
                canonical,
            )
            .unwrap();
        client
            .attach(
                &child_space,
                grandchild.matrix_room_id.as_deref().unwrap(),
                canonical,
            )
            .unwrap();
        assert_eq!(
            crate::chat::tree::project_hierarchy(&client, &rooms)["nested"]
                .parents
                .len(),
            2
        );
    }
    client.attach(&child_space, external, false).unwrap();
    let cyclic = crate::chat::tree::project_hierarchy(&client, &rooms);
    assert!(cyclic["main"].needs_attention && cyclic["topic"].needs_attention);
    assert!(cyclic["main"].parents.is_empty());
    client
        .put_state(&child_space, "m.space.child", external, json!({}))
        .unwrap();
    client
        .put_state(external, "m.space.parent", &child_space, json!({}))
        .unwrap();
    let foreign = chat::Room {
        project_id: "other-project".into(),
        id: "foreign".into(),
        ..room.clone()
    };
    let mut foreign = foreign;
    foreign.matrix_room_id = Some(
        client.create(&foreign, "foreign", true).unwrap()["matrix_room_id"]
            .as_str()
            .unwrap()
            .into(),
    );
    let foreign_space = client.ensure_carrier(&foreign).unwrap();
    client
        .attach(
            &foreign_space,
            grandchild.matrix_room_id.as_deref().unwrap(),
            false,
        )
        .unwrap();
    let foreign_projection = crate::chat::tree::project_hierarchy(&client, &rooms);
    assert_eq!(foreign_projection["nested"].parents.len(), 2);
    assert!(
        foreign_projection["nested"]
            .external_links
            .contains(&foreign_space)
    );
    assert!(foreign_projection["nested"].needs_attention);
    let inaccessible = Client::new(room.server.clone(), "invalid-token".into()).unwrap();
    let partial = crate::chat::tree::project_hierarchy(&inaccessible, &rooms);
    assert!(
        partial
            .values()
            .all(|r| r.parents.is_empty() && r.children.is_empty() && r.needs_attention)
    );
    let sent = client
        .send(external, "txn-one", "日本語与中文 e\u{301} / é")
        .unwrap();
    let reply = client
        .send_thread(
            external,
            "thread-1",
            "讨论串回复",
            Some(sent["event_id"].as_str().unwrap()),
        )
        .unwrap();
    assert!(
        client
            .send_thread(
                external,
                "nested-thread",
                "不能嵌套",
                Some(reply["event_id"].as_str().unwrap())
            )
            .is_err()
    );
    let thread_source = source_text(
        Reference {
            key: chat::key(store::Scope::Project("p".into()), "room_binding", "main"),
            version: store::Version::State(1),
        },
        &client
            .event(external, reply["event_id"].as_str().unwrap())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(thread_source.excerpt, "讨论串回复");
    assert_eq!(
        client
            .thread_events(external, sent["event_id"].as_str().unwrap())
            .unwrap()
            .len(),
        1
    );
    assert!(
        client
            .context(external, sent["event_id"].as_str().unwrap())
            .unwrap()["end"]
            .is_string()
    );
    assert_eq!(
        client
            .send(external, "txn-one", "日本語与中文 e\u{301} / é")
            .unwrap(),
        sent
    );
    let event = client
        .event(external, sent["event_id"].as_str().unwrap())
        .unwrap();
    let binding = Reference {
        key: chat::key(store::Scope::Project("p".into()), "room_binding", "main"),
        version: store::Version::State(1),
    };
    let frozen = source_text(binding.clone(), &event).unwrap();
    let before = client.sync(None).unwrap();
    let second = client.send(external, "txn-two", "第二条").unwrap();
    assert_eq!(
        client.last_activity(external).unwrap(),
        client
            .event(external, second["event_id"].as_str().unwrap())
            .unwrap()["origin_server_ts"]
            .as_u64()
            .map(|ms| ms / 1000)
    );
    let after = client
        .sync(Some(before["next_batch"].as_str().unwrap().into()))
        .unwrap();
    assert!(
        after["rooms"][external]["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_id"] == second["event_id"])
    );
    let timeline = client.timeline(external, None).unwrap();
    let ids: Vec<_> = timeline["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["type"] == "m.room.message")
        .map(|e| e["event_id"].clone())
        .collect();
    assert_eq!(
        ids,
        vec![
            sent["event_id"].clone(),
            reply["event_id"].clone(),
            second["event_id"].clone()
        ]
    );
    client
        .set_view_state(
            external,
            "desktop",
            "未发送草稿",
            Some(before["next_batch"].as_str().unwrap().into()),
        )
        .unwrap();
    assert_eq!(
        client.view_state(external, "desktop").unwrap()["draft"],
        "未发送草稿"
    );
    assert_eq!(client.view_state(external, "other").unwrap()["draft"], "");
    let second_client = Client::new(room.server.clone(), "as-test-secret".into()).unwrap();
    assert_eq!(
        source_text(
            binding,
            &second_client
                .event(external, sent["event_id"].as_str().unwrap())
                .unwrap()
        )
        .unwrap()
        .source,
        frozen.source
    );
    assert_eq!(
        second_client.view_state(external, "desktop").unwrap()["draft"],
        "未发送草稿"
    );
    // Tuwunel really sent transactions; acknowledgement follows durable inbox insertion.
    for _ in 0..100 {
        if root.join("cache/chat-inbox.sqlite").exists() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(root.join("cache/chat-inbox.sqlite").exists());
    recovery_regressions(&root, &client, &bound_main, &frozen);
    let mut encryption = ruma::api::client::state::send_state_event::v3::Request::new_raw(
        room_id(external).unwrap(),
        StateEventType::RoomEncryption,
        String::new(),
        ruma::serde::Raw::from_json_string(json!({"algorithm":"m.megolm.v1.aes-sha2"}).to_string())
            .unwrap(),
    );
    encryption.timestamp = None;
    client.request(encryption).unwrap();
    let other = client
        .create(
            &chat::Room {
                id: "other".into(),
                ..room.clone()
            },
            "other-command",
            true,
        )
        .unwrap();
    let other_id = other["matrix_room_id"].as_str().unwrap();
    let isolated = client.sync_room(None, Some(other_id)).unwrap();
    assert_eq!(isolated["rooms"].as_object().unwrap().len(), 1);
    assert!(isolated["rooms"].get(external).is_none());
    assert_eq!(client.guard(external).unwrap_err().code, "CHAT_ENCRYPTED");
    assert_eq!(
        client
            .send(external, "txn-three", "must not send")
            .unwrap_err()
            .code,
        "CHAT_ENCRYPTED"
    );
    native.child.kill().unwrap();
    native.child.wait().unwrap();
    assert_eq!(client.guard(external).unwrap_err().code, "CHAT_UNAVAILABLE");
    // Native transaction IDs remain idempotent after a real server restart.
    let log = std::fs::File::create(root.join("restart.log")).unwrap();
    native.child = Command::new(find(Path::new(env!("HCTL2_TEST_TUWUNEL"))).unwrap())
        .arg("--config")
        .arg(&config)
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .spawn()
        .unwrap();
    for _ in 0..300 {
        if client.guard(other_id).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let retained = client
        .send(other_id, "restart-transaction", "重启前后唯一")
        .unwrap();
    native.child.kill().unwrap();
    native.child.wait().unwrap();
    let log = std::fs::File::create(root.join("restart2.log")).unwrap();
    native.child = Command::new(find(Path::new(env!("HCTL2_TEST_TUWUNEL"))).unwrap())
        .arg("--config")
        .arg(&config)
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .spawn()
        .unwrap();
    for _ in 0..300 {
        if client.guard(other_id).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(
        client
            .send(other_id, "restart-transaction", "重启前后唯一")
            .unwrap(),
        retained
    );
    shutdown_tx.send(()).unwrap();
    http.join().unwrap();
}

fn recovery_regressions(root: &Path, client: &Client, main: &Room, source: &SourceText) {
    use store::{EffectState, ProjectSettings, Record, RecordData, RoomKind, RoomState};
    let scope = Scope::Project(main.project_id.clone());
    let actor = TrustedActor(Actor {
        principal: "owner".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![Scope::Control, scope.clone()],
        authority: None,
    });
    let mut store = Store::open(&root.join("recovery-control")).unwrap();
    let project_data = RecordData::Project {
        repo_id: "fixture-repo".into(),
        settings: ProjectSettings {
            publish_review_requires_confirmation: true,
            selection_policy: json!({}),
        },
        archived: false,
    };
    let project = Record {
        key: chat::key(scope.clone(), "project", &main.project_id),
        version: 1,
        revision_digest: foundation::canonical_json_sha256(
            &serde_json::to_value(&project_data).unwrap(),
        )
        .unwrap(),
        data: project_data,
        sources: vec![],
        materials: vec![],
    };
    let identity_data = RecordData::Room {
        room_kind: RoomKind::Main,
        state: RoomState::Active,
    };
    let identity = Record {
        key: chat::key(scope.clone(), "room", &main.id),
        version: 1,
        revision_digest: foundation::canonical_json_sha256(
            &serde_json::to_value(&identity_data).unwrap(),
        )
        .unwrap(),
        data: identity_data,
        sources: vec![],
        materials: vec![],
    };
    let binding =
        chat::value_record(chat::key(scope.clone(), "room_binding", &main.id), 1, main).unwrap();
    let command = store::Command {
        command_id: "seed-recovery".into(),
        idempotency_key: "seed-recovery".into(),
        actor: actor.0.clone(),
        target: project.key.clone(),
        expected: Expected::Absent,
        binding: main.server.binding.clone(),
        input_digest: store::Command::digest_input("fixture", &json!({})).unwrap(),
        operation: "fixture".into(),
        input: json!({}),
    };
    store
        .submit(store.generation(), &actor, &command, None, |tx| {
            for record in [&project, &identity, &binding] {
                tx.put(record)?;
            }
            Ok(json!({}))
        })
        .unwrap();
    let shared = Arc::new(Mutex::new(Some(store)));
    // Missing, partial and complete native results all retain the original intent.
    // Closing never permits a new create or completion write, even after first dispatch.
    for outcome in ["cancelled", "absent", "partial", "complete"] {
        let result = access(&shared, |s| {
            let plan = chat::prepare(
                s,
                Input {
                    key: outcome.into(),
                    action: Action::CreateTopic {
                        project_id: main.project_id.clone(),
                        project_version: 1,
                        name: outcome.into(),
                        origin: Box::new(Origin::Room {
                            room_id: main.id.clone(),
                            binding_version: 1,
                        }),
                        brief: Box::new(chat::Brief {
                            context_and_goal: "恢复验证".into(),
                            settled_facts_and_reasons: vec![],
                            disagreements_and_questions: vec![],
                            constraints_and_materials: vec![],
                            sources: vec![source.source.clone()],
                        }),
                        participants: vec![],
                        roster_confirmed: true,
                    },
                },
                vec![source.clone()],
            )?;
            chat::admit(s, &actor, plan)
        })
        .unwrap();
        let id = result["room_id"].as_str().unwrap();
        let effect_id = result["effect_id"].as_str().unwrap();
        let effect = access(&shared, |s| {
            if outcome != "cancelled" {
                s.begin_effect(s.generation(), effect_id)?;
            }
            Ok(s.effect(effect_id)?.0)
        })
        .unwrap();
        let original: Room = serde_json::from_value(effect.input["room"].clone()).unwrap();
        let native_result = if matches!(outcome, "partial" | "complete") {
            Some(
                client
                    .create(&original, &effect.idempotency_key, true)
                    .unwrap(),
            )
        } else {
            None
        };
        if outcome == "complete" {
            let external = native_result.as_ref().unwrap()["matrix_room_id"]
                .as_str()
                .unwrap();
            let space = client.ensure_carrier(main).unwrap();
            client.attach(&space, external, true).unwrap();
            client
                .put_state(external, "io.hctl2.topic_creation", "", json!({"command":effect.idempotency_key,"parent_room_id":main.id,"project_id":main.project_id}))
                .unwrap();
        }
        access(&shared, |s| {
            let plan = chat::prepare(
                s,
                Input {
                    key: format!("close-{outcome}"),
                    action: Action::Close {
                        project_id: main.project_id.clone(),
                        room_id: id.into(),
                        version: 1,
                    },
                },
                vec![],
            )?;
            chat::admit(s, &actor, plan)
        })
        .unwrap();
        let inventory = || {
            let mut rooms = client
                .request(ruma::api::client::membership::joined_rooms::v3::Request::new())
                .unwrap()
                .joined_rooms;
            rooms.sort();
            rooms
        };
        let before = inventory();
        let driven = if outcome == "cancelled" {
            drive_using(&shared, root, &actor, effect_id, || {
                panic!("cancelled intent must not connect")
            })
        } else {
            drive_using(&shared, root, &actor, effect_id, || Ok(client.clone()))
        };
        let expected = match outcome {
            "cancelled" => {
                assert_eq!(driven.unwrap_err().code, "EFFECT_CANCELLED");
                EffectState::Cancelled
            }
            "complete" => {
                assert_eq!(driven.unwrap(), native_result.unwrap());
                EffectState::Confirmed
            }
            _ => {
                assert_eq!(driven.unwrap_err().code, "RESULT_UNKNOWN");
                if let Some(receipt) = native_result {
                    assert_eq!(
                        client
                            .state(
                                receipt["matrix_room_id"].as_str().unwrap(),
                                "io.hctl2.topic_creation"
                            )
                            .unwrap_err()
                            .code,
                        "CHAT_NOT_FOUND"
                    );
                }
                EffectState::Unknown
            }
        };
        assert_eq!(
            inventory(),
            before,
            "readback must not create more native Rooms"
        );
        access(&shared, |s| {
            assert_eq!(s.effect(effect_id)?.1, expected);
            assert!(matches!(
                chat::required(s, &chat::key(scope.clone(), "room", id))?.data,
                RecordData::Room {
                    state: RoomState::Archived,
                    ..
                }
            ));
            Ok(())
        })
        .unwrap();
    }
    // Crash after the native carrier pointer is stored but before outbox confirmation.
    // A new reparent must finish the old intent rather than merely using its Space.
    let created = access(&shared, |s| {
        let plan = chat::prepare(
            s,
            Input {
                key: "carrier-parent".into(),
                action: Action::CreateTopic {
                    project_id: main.project_id.clone(),
                    project_version: 1,
                    name: "carrier parent".into(),
                    origin: Box::new(Origin::Room {
                        room_id: main.id.clone(),
                        binding_version: 1,
                    }),
                    brief: Box::new(chat::Brief {
                        context_and_goal: "恢复验证".into(),
                        settled_facts_and_reasons: vec![],
                        disagreements_and_questions: vec![],
                        constraints_and_materials: vec![],
                        sources: vec![source.source.clone()],
                    }),
                    participants: vec![],
                    roster_confirmed: true,
                },
            },
            vec![source.clone()],
        )?;
        chat::admit(s, &actor, plan)
    })
    .unwrap();
    let effect_id = created["effect_id"].as_str().unwrap();
    drive_using(&shared, root, &actor, effect_id, || Ok(client.clone())).unwrap();
    let id = created["room_id"].as_str().unwrap();
    let (binding, parent) = access(&shared, |s| chat::room(s, &main.project_id, id)).unwrap();
    let effect = tree::carrier_intent(&shared, &actor, &parent).unwrap();
    access(&shared, |s| s.begin_effect(s.generation(), &effect)).unwrap();
    let space = client.ensure_carrier(&parent).unwrap();
    assert_eq!(
        access(&shared, |s| Ok(s.effect(&effect)?.1)).unwrap(),
        EffectState::Unknown
    );
    let main_space = client.carrier(main).unwrap().unwrap();
    let payload = json!({"project_id":main.project_id,"room_id":main.id,"parent_room_id":id,"binding_version":1,"parent_binding_version":binding.version,"old_space_ids":[]});
    // Native clients can create cycles. Display projection hides those edges;
    // a write must still reject the descendant using the unfiltered native graph.
    client.attach(&space, &main_space, false).unwrap();
    let projected = tree::project_hierarchy(client, &[main.clone(), parent.clone()]);
    assert!(projected[&main.id].parents.is_empty());
    assert!(projected[id].parents.is_empty());
    assert_eq!(
        tree::reparent_using(&shared, root, &payload, &actor, || Ok(client.clone()))
            .unwrap_err()
            .code,
        "CHAT_HIERARCHY_CYCLE"
    );
    assert_eq!(
        access(&shared, |s| Ok(s.effect(&effect)?.1)).unwrap(),
        EffectState::Unknown
    );
    client
        .put_state(&space, "m.space.child", &main_space, json!({}))
        .unwrap();
    client
        .put_state(&main_space, "m.space.parent", &space, json!({}))
        .unwrap();
    // Remove the original parent edge before a valid reverse reparent.
    // Immutable origin is unaffected by these native content writes.
    client
        .put_state(&main_space, "m.space.child", &space, json!({}))
        .unwrap();
    client
        .put_state(&space, "m.space.parent", &main_space, json!({}))
        .unwrap();
    for _ in 0..2 {
        let receipt =
            tree::reparent_using(&shared, root, &payload, &actor, || Ok(client.clone())).unwrap();
        assert_eq!(receipt["state"], "confirmed");
        assert_eq!(receipt["space_id"], space);
        assert_eq!(
            access(&shared, |s| Ok(s.effect(&effect)?.1)).unwrap(),
            EffectState::Confirmed
        );
        assert_eq!(client.ensure_carrier(&parent).unwrap(), space);
    }
}
