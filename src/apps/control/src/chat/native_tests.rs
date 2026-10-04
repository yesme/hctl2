//! Native Tuwunel in an isolated directory; no mutations of the developer's chat server.
use super::*;
use crate::chat::{access, drive_using, human_members, tree};
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

fn spawn_tuwunel(name: &str) -> (Native, u16, PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "hctl-chat-{name}-{}-{}",
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
    let _http = std::thread::spawn(move || {
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
    std::mem::forget(shutdown_tx);
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
    (native, port, root)
}

#[test]
fn native_matrix_create_send_resync_freeze_account_data_and_encryption() {
    let (mut native, port, root) = spawn_tuwunel("native");
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
    let config = root.join("tuwunel.toml");
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
                        invites: None,
                    },
                },
                vec![source.clone()],
                vec![],
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
            drive_using(&shared, root, &actor, effect_id, "@hctl2_", || {
                panic!("cancelled intent must not connect")
            })
        } else {
            drive_using(&shared, root, &actor, effect_id, "@hctl2_", || {
                Ok(client.clone())
            })
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
                    invites: None,
                },
            },
            vec![source.clone()],
            vec![],
        )?;
        chat::admit(s, &actor, plan)
    })
    .unwrap();
    let effect_id = created["effect_id"].as_str().unwrap();
    drive_using(&shared, root, &actor, effect_id, "@hctl2_", || {
        Ok(client.clone())
    })
    .unwrap();
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
        tree::reparent_using(&shared, root, &payload, &actor, "@hctl2_", || Ok(
            client.clone()
        ))
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
        let receipt = tree::reparent_using(&shared, root, &payload, &actor, "@hctl2_", || {
            Ok(client.clone())
        })
        .unwrap();
        assert_eq!(receipt["state"], "confirmed");
        assert_eq!(receipt["space_id"], space);
        assert_eq!(
            access(&shared, |s| Ok(s.effect(&effect)?.1)).unwrap(),
            EffectState::Confirmed
        );
        assert_eq!(client.ensure_carrier(&parent).unwrap(), space);
    }
}

/// Patch 1a: what a person sees in the chat client. A created Topic Room
/// receives the confirmed brief as its opening message (idempotent), the
/// confirmed human invite list is delivered per person, and carrier Space
/// membership follows the main Room.
#[test]
fn native_topic_opening_invites_and_space_members_follow_main_room() {
    use store::{ProjectSettings, Record, RecordData, RoomKind, RoomState};
    let (_native, port, root) = spawn_tuwunel("topic-opening");
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
    // Humans register through the shared token and join the main Room.
    let (alice_id, alice_token) = client
        .human_register("alice", "pw-alice", "test-registration")
        .unwrap();
    let (bob_id, bob_token) = client
        .human_register("bob", "pw-bob", "test-registration")
        .unwrap();
    let main = chat::Room {
        project_id: "p".into(),
        id: "main".into(),
        name: "主房间".into(),
        server: server.clone(),
        matrix_room_id: None,
        participants: vec![],
        brief: None,
        origin: None,
    };
    let scope = Scope::Project("p".into());
    let actor = TrustedActor(Actor {
        principal: "owner".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![Scope::Control, scope.clone()],
        authority: None,
    });
    let mut store = Store::open(&root.join("opening-control")).unwrap();
    let project = Record {
        key: chat::key(scope.clone(), "project", "p"),
        version: 1,
        revision_digest: foundation::canonical_json_sha256(&json!({
            "repo_id":"fixture-repo","archived":false
        }))
        .unwrap(),
        data: RecordData::Project {
            repo_id: "fixture-repo".into(),
            settings: ProjectSettings {
                publish_review_requires_confirmation: true,
                selection_policy: json!({}),
            },
            archived: false,
        },
        sources: vec![],
        materials: vec![],
    };
    let identity = Record {
        key: chat::key(scope.clone(), "room", "main"),
        version: 1,
        revision_digest: foundation::canonical_json_sha256(&json!({
            "room_kind":"main","state":"active"
        }))
        .unwrap(),
        data: RecordData::Room {
            room_kind: RoomKind::Main,
            state: RoomState::Active,
        },
        sources: vec![],
        materials: vec![],
    };
    let created = client.create(&main, "opening-main", true).unwrap();
    let main_external = created["matrix_room_id"].as_str().unwrap().to_owned();
    let mut bound_main = main.clone();
    bound_main.matrix_room_id = Some(main_external.clone());
    let binding = chat::value_record(
        chat::key(scope.clone(), "room_binding", "main"),
        1,
        &bound_main,
    )
    .unwrap();
    let command = store::Command {
        command_id: "seed-opening".into(),
        idempotency_key: "seed-opening".into(),
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
    client
        .members(
            &main_external,
            &[alice_id.clone(), bob_id.clone()],
            true,
            true,
        )
        .unwrap();
    client.human_join(&alice_token, &main_external).unwrap();
    client.human_join(&bob_token, &main_external).unwrap();
    // A managed digital participant joins too; it must not become a default.
    assert_eq!(
        client.register_virtual_user("hctl2_worker").unwrap(),
        "@hctl2_worker:hctl2.localhost"
    );
    client
        .members(
            &main_external,
            &["@hctl2_worker:hctl2.localhost".to_owned()],
            true,
            true,
        )
        .unwrap();
    let shared = Arc::new(Mutex::new(Some(store)));
    let defaults = human_members(&client, &main_external, &main.server.sender, "@hctl2_").unwrap();
    assert_eq!(
        defaults,
        vec![alice_id.clone(), bob_id.clone()],
        "default invite list is the human members; control and managed accounts are not"
    );
    // One human message is the brief source.
    let sent = client
        .send(&main_external, "opening-source", "原始讨论")
        .unwrap();
    let event = client
        .event(&main_external, sent["event_id"].as_str().unwrap())
        .unwrap();
    let binding_reference = Reference {
        key: chat::key(scope.clone(), "room_binding", "main"),
        version: store::Version::State(1),
    };
    let source = source_text(binding_reference, &event).unwrap();
    let brief = chat::Brief {
        context_and_goal: "开场提要：确认后的正文".into(),
        settled_facts_and_reasons: vec!["已定：方向一致".into()],
        disagreements_and_questions: vec![],
        constraints_and_materials: vec![],
        sources: vec![source.source.clone()],
    };
    let opening = chat::opening_body(&brief);
    let result = access(&shared, |s| {
        let plan = chat::prepare(
            s,
            Input {
                key: "topic-opening".into(),
                action: Action::CreateTopic {
                    project_id: "p".into(),
                    project_version: 1,
                    name: "开场话题".into(),
                    origin: Box::new(Origin::Room {
                        room_id: "main".into(),
                        binding_version: 1,
                    }),
                    brief: Box::new(brief.clone()),
                    participants: vec![],
                    roster_confirmed: true,
                    invites: None,
                },
            },
            vec![source.clone()],
            vec![alice_id.clone(), bob_id.clone()],
        )?;
        assert_eq!(plan.invites, vec![alice_id.clone(), bob_id.clone()]);
        chat::admit(s, &actor, plan)
    })
    .unwrap();
    let create = result["effect_id"].as_str().unwrap().to_owned();
    let opening_effect = result["opening_effect_id"].as_str().unwrap().to_owned();
    let invite_ids: Vec<String> = result["invite_effect_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(invite_ids.len(), 2);
    let result_brief = access(&shared, |s| {
        let (_, room) = chat::room(s, "p", result["room_id"].as_str().unwrap())?;
        Ok(room.brief)
    })
    .unwrap();
    // Create first; the opening drive before it would only report pending.
    drive_using(&shared, &root, &actor, &create, "@hctl2_", || {
        Ok(client.clone())
    })
    .unwrap();
    let receipt = drive_using(&shared, &root, &actor, &opening_effect, "@hctl2_", || {
        Ok(client.clone())
    })
    .unwrap();
    let topic_external = access(&shared, |s| {
        let (_, room) = chat::room(s, "p", result["room_id"].as_str().unwrap())?;
        Ok(room.matrix_room_id)
    })
    .unwrap()
    .expect("created Topic is bound")
    .to_owned();
    // Retry sends no second opening: same event, same first timeline entry.
    let replayed = drive_using(&shared, &root, &actor, &opening_effect, "@hctl2_", || {
        Ok(client.clone())
    })
    .unwrap();
    assert_eq!(replayed, receipt);
    let timeline = client.timeline(&topic_external, None).unwrap();
    let messages: Vec<&Value> = timeline["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["type"] == "m.room.message")
        .collect();
    assert_eq!(messages.len(), 1, "exactly one opening message");
    assert_eq!(messages[0]["content"]["body"], json!(opening));
    // Invites: each human is invited, control and the managed worker are not.
    for id in &invite_ids {
        drive_using(&shared, &root, &actor, id, "@hctl2_", || Ok(client.clone())).unwrap();
    }
    let topic_members = client.member_state(&topic_external).unwrap();
    // The exact member set: the creator joined, the two confirmed humans are
    // invited, and nobody else — not the control account's invite, not the
    // managed worker that sits in the source Room.
    assert_eq!(
        topic_members,
        vec![
            (alice_id.clone(), "invite".to_owned()),
            (bob_id.clone(), "invite".to_owned()),
            (
                "@hctl2_control:hctl2.localhost".to_owned(),
                "join".to_owned()
            ),
        ],
        "confirmed list only: removed and non-human members must be absent"
    );
    // Carrier Space of the main Room carries the humans; a later main-Room
    // invite syncs to every carrier Space of the Project.
    let main_space = client.carrier(&bound_main).unwrap().unwrap();
    let space_members =
        human_members(&client, &main_space, &main.server.sender, "@hctl2_").unwrap();
    assert!(space_members.contains(&alice_id) && space_members.contains(&bob_id));
    let (carol_id, carol_token) = client
        .human_register("carol", "pw-carol", "test-registration")
        .unwrap();
    client
        .members(&main_external, std::slice::from_ref(&carol_id), true, true)
        .unwrap();
    // An invited-but-not-joined human already counts as a current member.
    let with_carol_invited =
        human_members(&client, &main_external, &main.server.sender, "@hctl2_").unwrap();
    assert!(with_carol_invited.contains(&carol_id));
    client.human_join(&carol_token, &main_external).unwrap();
    // The topic gains a child so its own carrier Space exists too. The nested
    // creation is driven from the unknown state with a new human already in the
    // main Room: the carrier converge must still write on retry, not refuse.
    let (dave_id, dave_token) = client
        .human_register("dave", "pw-dave", "test-registration")
        .unwrap();
    client
        .members(&main_external, std::slice::from_ref(&dave_id), true, true)
        .unwrap();
    client.human_join(&dave_token, &main_external).unwrap();
    let topic_room_id = result["room_id"].as_str().unwrap().to_owned();
    let topic_binding_version = access(&shared, |s| {
        let (binding, _) = chat::room(s, "p", &topic_room_id)?;
        Ok(binding.version)
    })
    .unwrap();
    let opening_event = client
        .event(&topic_external, receipt["event_id"].as_str().unwrap())
        .unwrap();
    let topic_source = source_text(
        Reference {
            key: chat::key(scope.clone(), "room_binding", &topic_room_id),
            version: store::Version::State(topic_binding_version),
        },
        &opening_event,
    )
    .unwrap();
    let nested = access(&shared, |s| {
        let plan = chat::prepare(
            s,
            Input {
                key: "nested-opening".into(),
                action: Action::CreateTopic {
                    project_id: "p".into(),
                    project_version: 1,
                    name: "嵌套话题".into(),
                    origin: Box::new(Origin::Room {
                        room_id: topic_room_id.clone(),
                        binding_version: topic_binding_version,
                    }),
                    brief: Box::new(chat::Brief {
                        context_and_goal: "嵌套".into(),
                        settled_facts_and_reasons: vec![],
                        disagreements_and_questions: vec![],
                        constraints_and_materials: vec![],
                        sources: vec![topic_source.source.clone()],
                    }),
                    participants: vec![],
                    roster_confirmed: true,
                    invites: Some(vec![]),
                },
            },
            vec![topic_source.clone()],
            vec![],
        )?;
        chat::admit(s, &actor, plan)
    })
    .unwrap();
    let nested_create = nested["effect_id"].as_str().unwrap().to_owned();
    // Enter delivery before driving: every retry from here is unknown-state.
    access(&shared, |s| s.begin_effect(s.generation(), &nested_create)).unwrap();
    drive_using(&shared, &root, &actor, &nested_create, "@hctl2_", || {
        Ok(client.clone())
    })
    .unwrap();
    // A real `project members` invite on the main Room now syncs the new
    // human into every carrier Space of the Project.
    let (main_binding_record, _) = access(&shared, |s| chat::room(s, "p", "main")).unwrap();
    let members_result = access(&shared, |s| {
        let plan = project::prepare(
            s,
            project::Input {
                key: "members-sync".into(),
                action: project::Action::Members {
                    project_id: "p".into(),
                    project_version: 1,
                    rooms: vec![chat::reference(&main_binding_record)],
                    users: vec![carol_id.clone()],
                    invite: true,
                },
            },
            None,
            &actor,
            0,
        )?;
        project::admit(s, &actor, plan)
    })
    .unwrap();
    let members_effect = members_result["effect_ids"][0].as_str().unwrap().to_owned();
    drive_using(&shared, &root, &actor, &members_effect, "@hctl2_", || {
        Ok(client.clone())
    })
    .unwrap();
    let rooms = access(&shared, |s| {
        s.list("room_binding")?
            .iter()
            .filter(|r| r.key.scope == Scope::Project("p".into()))
            .map(chat::decode::<Room>)
            .collect::<Result<Vec<_>>>()
    })
    .unwrap();
    let mut spaces = vec![];
    for room in &rooms {
        if room.matrix_room_id.is_some()
            && let Some(space) = client.carrier(room).unwrap()
        {
            spaces.push(space);
        }
    }
    assert!(spaces.len() >= 2, "main and topic carriers both exist");
    for space in &spaces {
        let members = human_members(&client, space, &main.server.sender, "@hctl2_").unwrap();
        assert!(
            members.contains(&carol_id),
            "Space {space} did not follow the main Room invite"
        );
    }
    // The Space converged during an unknown-state create retry — the first
    // Topic's carrier, which the nested creation materialized — must contain
    // the human who joined the main Room before the retry, without any
    // `project members` operation mentioning them: creation-time convergence
    // takes the main Room's current humans, not a stale list, and writes even
    // though the intent is in the unknown state.
    let first_topic_room = access(&shared, |s| {
        let (_, room) = chat::room(s, "p", &topic_room_id)?;
        Ok(room)
    })
    .unwrap();
    let retried_space = client.carrier(&first_topic_room).unwrap().unwrap();
    let retried_space_members =
        human_members(&client, &retried_space, &main.server.sender, "@hctl2_").unwrap();
    assert!(
        retried_space_members.contains(&dave_id),
        "the carrier converged on an unknown-state retry must take current humans: {retried_space_members:?}"
    );
    // Sync, not convergence: a new human joins after every Space already
    // exists; only the `project members` invite can put them into the Spaces.
    let (erin_id, erin_token) = client
        .human_register("erin", "pw-erin", "test-registration")
        .unwrap();
    client
        .members(&main_external, std::slice::from_ref(&erin_id), true, true)
        .unwrap();
    client.human_join(&erin_token, &main_external).unwrap();
    let members_sync = |key: &str, invite: bool, user: &str| {
        let outcome = access(&shared, |s| {
            let plan = project::prepare(
                s,
                project::Input {
                    key: key.into(),
                    action: project::Action::Members {
                        project_id: "p".into(),
                        project_version: 1,
                        rooms: vec![{
                            let (binding, _) = chat::room(s, "p", "main")?;
                            chat::reference(&binding)
                        }],
                        users: vec![user.to_owned()],
                        invite,
                    },
                },
                None,
                &actor,
                0,
            )?;
            project::admit(s, &actor, plan)
        })
        .unwrap();
        let effect = outcome["effect_ids"][0].as_str().unwrap().to_owned();
        drive_using(&shared, &root, &actor, &effect, "@hctl2_", || {
            Ok(client.clone())
        })
        .unwrap()
    };
    members_sync("members-erin-invite", true, &erin_id);
    let main_state = client.member_state(&main_external).unwrap();
    assert!(
        main_state
            .iter()
            .any(|(member, state)| member == &erin_id && state == "join")
    );
    for space in &spaces {
        let members = human_members(&client, space, &main.server.sender, "@hctl2_").unwrap();
        assert!(
            members.contains(&erin_id),
            "Space {space} missed the sync-only invite"
        );
    }
    // Space membership does not open Topic Rooms.
    let first_topic_members = client.member_state(&topic_external).unwrap();
    assert!(
        !first_topic_members
            .iter()
            .any(|(member, _)| member == &erin_id),
        "entering a carrier Space must not invite anyone into a Topic Room"
    );
    // Removal side: the same operation on the main Room removes from every
    // carrier Space as well.
    members_sync("members-erin-remove", false, &erin_id);
    let main_state = client.member_state(&main_external).unwrap();
    assert!(
        !main_state.iter().any(
            |(member, state)| member == &erin_id && matches!(state.as_str(), "join" | "invite")
        )
    );
    for space in &spaces {
        let members = human_members(&client, space, &main.server.sender, "@hctl2_").unwrap();
        assert!(
            !members.contains(&erin_id),
            "Space {space} missed the synced removal"
        );
    }
    // The stored brief material is untouched by the opening projection.
    let brief_after = access(&shared, |s| {
        let (binding, room) = chat::room(s, "p", &topic_room_id)?;
        Ok((binding.version, room.brief))
    })
    .unwrap();
    let brief_at_admit = result_brief.clone();
    assert_eq!(
        brief_after.1, brief_at_admit,
        "the stored brief is authoritative and unchanged"
    );
}

/// A Topic closed while its opening was in delivery resolves the unknown
/// intent by readback instead of failing forever: what arrived is confirmed,
/// what never left is recorded as undelivered.
#[test]
fn native_closed_topic_resolves_unknown_opening_by_readback() {
    use store::{ProjectSettings, Record, RecordData, RoomKind, RoomState};
    let (_native, port, root) = spawn_tuwunel("closed-opening");
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
    let main = chat::Room {
        project_id: "p".into(),
        id: "main".into(),
        name: "主房间".into(),
        server,
        matrix_room_id: None,
        participants: vec![],
        brief: None,
        origin: None,
    };
    let scope = Scope::Project("p".into());
    let actor = TrustedActor(Actor {
        principal: "owner".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![Scope::Control, scope.clone()],
        authority: None,
    });
    let mut store = Store::open(&root.join("closed-control")).unwrap();
    let project = Record {
        key: chat::key(scope.clone(), "project", "p"),
        version: 1,
        revision_digest: foundation::canonical_json_sha256(
            &json!({"repo_id":"fixture-repo","archived":false}),
        )
        .unwrap(),
        data: RecordData::Project {
            repo_id: "fixture-repo".into(),
            settings: ProjectSettings {
                publish_review_requires_confirmation: true,
                selection_policy: json!({}),
            },
            archived: false,
        },
        sources: vec![],
        materials: vec![],
    };
    let identity = Record {
        key: chat::key(scope.clone(), "room", "main"),
        version: 1,
        revision_digest: foundation::canonical_json_sha256(
            &json!({"room_kind":"main","state":"active"}),
        )
        .unwrap(),
        data: RecordData::Room {
            room_kind: RoomKind::Main,
            state: RoomState::Active,
        },
        sources: vec![],
        materials: vec![],
    };
    let created = client.create(&main, "closed-main", true).unwrap();
    let main_external = created["matrix_room_id"].as_str().unwrap().to_owned();
    let mut bound_main = main.clone();
    bound_main.matrix_room_id = Some(main_external.clone());
    let binding = chat::value_record(
        chat::key(scope.clone(), "room_binding", "main"),
        1,
        &bound_main,
    )
    .unwrap();
    let command = store::Command {
        command_id: "seed-closed".into(),
        idempotency_key: "seed-closed".into(),
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
    let sent = client
        .send(&main_external, "closed-source", "来源")
        .unwrap();
    let event = client
        .event(&main_external, sent["event_id"].as_str().unwrap())
        .unwrap();
    let source = source_text(
        Reference {
            key: chat::key(scope.clone(), "room_binding", "main"),
            version: store::Version::State(1),
        },
        &event,
    )
    .unwrap();
    let result = access(&shared, |s| {
        let plan = chat::prepare(
            s,
            Input {
                key: "closed-topic".into(),
                action: Action::CreateTopic {
                    project_id: "p".into(),
                    project_version: 1,
                    name: "将关闭".into(),
                    origin: Box::new(Origin::Room {
                        room_id: "main".into(),
                        binding_version: 1,
                    }),
                    brief: Box::new(chat::Brief {
                        context_and_goal: "关闭前".into(),
                        settled_facts_and_reasons: vec![],
                        disagreements_and_questions: vec![],
                        constraints_and_materials: vec![],
                        sources: vec![source.source.clone()],
                    }),
                    participants: vec![],
                    roster_confirmed: true,
                    invites: Some(vec![]),
                },
            },
            vec![source.clone()],
            vec![],
        )?;
        chat::admit(s, &actor, plan)
    })
    .unwrap();
    let create = result["effect_id"].as_str().unwrap().to_owned();
    let opening = result["opening_effect_id"].as_str().unwrap().to_owned();
    drive_using(&shared, &root, &actor, &create, "@hctl2_", || {
        Ok(client.clone())
    })
    .unwrap();
    // The opening entered delivery, then the Topic closed before it was sent.
    access(&shared, |s| s.begin_effect(s.generation(), &opening)).unwrap();
    let close = access(&shared, |s| {
        let (binding, _) = chat::room(s, "p", result["room_id"].as_str().unwrap())?;
        let plan = chat::prepare(
            s,
            Input {
                key: "close-unknown-opening".into(),
                action: Action::Close {
                    project_id: "p".into(),
                    room_id: result["room_id"].as_str().unwrap().to_owned(),
                    version: binding.version,
                },
            },
            vec![],
            vec![],
        )?;
        chat::admit(s, &actor, plan)
    })
    .unwrap();
    assert!(close.is_object());
    let receipt = drive_using(&shared, &root, &actor, &opening, "@hctl2_", || {
        Ok(client.clone())
    })
    .unwrap();
    assert_eq!(receipt["room_closed"], json!(true));
    assert_eq!(receipt["delivered"], json!(false), "{receipt}");
    assert_eq!(
        access(&shared, |s| Ok(s.effect(&opening)?.1)).unwrap(),
        store::EffectState::Confirmed,
        "the reconcile loop must not retry a closed Room's opening forever"
    );
}
