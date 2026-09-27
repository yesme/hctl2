//! Native Tuwunel in an isolated directory; no mutations of the developer's chat server.
use super::*;
use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};
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
    assert_eq!(
        client
            .create(&room, "unknown-command", false)
            .unwrap_err()
            .code,
        "RESULT_UNKNOWN"
    );
    let external = created["matrix_room_id"].as_str().unwrap();
    let sent = client
        .send(external, "txn-one", "日本語与中文 e\u{301} / é")
        .unwrap();
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
        vec![sent["event_id"].clone(), second["event_id"].clone()]
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
    shutdown_tx.send(()).unwrap();
    http.join().unwrap();
}
