//! Real CLI, daemon and packaged Tuwunel. Project creation is a fixture until 辛.
use chat::{Server, key, main_room, reference};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};
use store::{
    Actor, ActorSource, Expected, ProjectSettings, Record, RecordData, Scope, Store, TrustedActor,
    Version,
};

struct Fixture {
    root: PathBuf,
    payload: PathBuf,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.run(&["stop"]);
        let _ = Command::new(self.payload.join("bin/hctl2-services"))
            .env("HCTL2_STATE_ROOT", self.root.join("services"))
            .arg("stop")
            .output();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
impl Fixture {
    fn run(&self, args: &[&str]) -> (bool, Value) {
        let out = Command::new(env!("CARGO_BIN_EXE_hctl2"))
            .env_remove("HCTL2_PROCESS_COMPOSE_BIN")
            .env("HCTL2_INSTALL_ROOT", &self.payload)
            .env("HCTL2_CONTROL_BIN", env!("CARGO_BIN_EXE_hctl2-control"))
            .args(["--json", "--root", self.root.to_str().unwrap()])
            .args(args)
            .output()
            .unwrap();
        (out.status.success(), serde_json::from_slice(&out.stdout).unwrap_or_else(|_| json!({"stdout":String::from_utf8_lossy(&out.stdout),"stderr":String::from_utf8_lossy(&out.stderr)})))
    }
    fn query(&self, kind: &str, input: Value) -> Value {
        let path = self.root.join("query.json");
        std::fs::write(&path, input.to_string()).unwrap();
        let (ok, value) = self.run(&["room", kind, "--input", path.to_str().unwrap()]);
        assert!(ok, "{kind}: {value}");
        value
    }
    fn command(&self, kind: &str, key: &str, input: &Value) -> (bool, Value) {
        let path = self.root.join("command.json");
        std::fs::write(&path, input.to_string()).unwrap();
        let p = path.to_str().unwrap();
        let (ok, preview) = self.run(&["room", kind, "--key", key, "--input", p]);
        if !ok {
            return (ok, preview);
        }
        self.run(&[
            "room",
            kind,
            "--key",
            key,
            "--input",
            p,
            "--preview-token",
            preview["preview_token"].as_str().unwrap(),
        ])
    }
    fn accepted(&self, kind: &str, key: &str, input: &Value) -> Value {
        let (ok, value) = self.command(kind, key, input);
        assert!(ok, "{kind}: {value}");
        value
    }
    fn show(&self, project: &str, room: &str) -> Value {
        let (ok, value) = self.run(&["room", "show", project, room]);
        assert!(ok, "{value}");
        value
    }
}
fn find(root: &Path, predicate: impl Fn(&Path) -> bool + Copy) -> Option<PathBuf> {
    for entry in std::fs::read_dir(root).ok()? {
        let p = entry.ok()?.path();
        if predicate(&p) {
            return Some(p);
        }
        if p.is_dir()
            && let Some(p) = find(&p, predicate)
        {
            return Some(p);
        }
    }
    None
}
#[test]
fn room_cli_native_lifecycle_preview_draft_replay_and_recovery() {
    let root = std::env::temp_dir().join(format!("hctl-room-cli-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let archive = find(Path::new(env!("HCTL2_TEST_DEPENDENCY_PACKAGE")), |p| {
        p.file_name()
            .unwrap()
            .to_str()
            .is_some_and(|n| n.ends_with(".tar.xz") && !n.contains("-sources"))
    })
    .unwrap();
    let extract = root.join("install");
    std::fs::create_dir(&extract).unwrap();
    assert!(
        Command::new("tar")
            .args(["-xf"])
            .arg(archive)
            .arg("-C")
            .arg(&extract)
            .status()
            .unwrap()
            .success()
    );
    let payload = find(&extract, |p| p.join("bin/hctl2-services").is_file()).unwrap();
    let f = Fixture {
        root: root.clone(),
        payload,
    };
    let socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = socket.local_addr().unwrap().port();
    drop(socket);
    // Private test installation only: allocate a port instead of competing for the product default.
    let versions = f.payload.join("lib/hctl2/services/versions.sh");
    let text = std::fs::read_to_string(&versions)
        .unwrap()
        .replace("6167", &port.to_string());
    std::fs::write(versions, text).unwrap();
    let mut store = Store::open(&root).unwrap();
    let server = Server {
        binding: store::Reference {
            key: key(
                Scope::Control,
                "chat_server",
                &format!("hctl2-{}", store.control_id()),
            ),
            version: Version::State(1),
        },
        url: format!("http://127.0.0.1:{port}"),
        server_name: "hctl2.localhost".into(),
        sender: "@hctl2_control:hctl2.localhost".into(),
    };
    let actor = TrustedActor(Actor {
        principal: "fixture".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![
            Scope::Control,
            Scope::Project("A".into()),
            Scope::Project("B".into()),
        ],
        authority: None,
    });
    let mut mains = vec![];
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
            revision_digest: foundation::canonical_json_sha256(
                &serde_json::to_value(&data).unwrap(),
            )
            .unwrap(),
            data,
            sources: vec![],
            materials: vec![],
        };
        let (records, effect) = main_room(
            &record,
            server.clone(),
            project.into(),
            &format!("seed-{project}"),
        )
        .unwrap();
        let command = store::Command {
            command_id: project.into(),
            idempotency_key: project.into(),
            actor: actor.0.clone(),
            target: record.key.clone(),
            expected: Expected::Absent,
            binding: reference(&record),
            input_digest: store::Command::digest_input("fixture", &json!({})).unwrap(),
            operation: "fixture".into(),
            input: json!({}),
        };
        store
            .submit(store.generation(), &actor, &command, None, |tx| {
                tx.put(&record)?;
                for r in &records {
                    tx.put(r)?;
                }
                tx.enqueue_effect(&effect)?;
                Ok(json!({}))
            })
            .unwrap();
        mains.push((records[0].key.id.clone(), effect.intent_id));
    }
    drop(store);
    // Start the native service first: control has to load its new AppService registration.
    assert!(
        Command::new(f.payload.join("bin/hctl2-services"))
            .env("HCTL2_STATE_ROOT", root.join("services"))
            .args(["start", "--no-wait", "tuwunel"])
            .output()
            .unwrap()
            .status
            .success()
    );
    for _ in 0..100 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(f.run(&["start"]).0);
    for (project, (room, effect)) in ["A", "B"].into_iter().zip(&mains) {
        let input = json!({"project_id":project,"effect_id":effect});
        let mut ready = false;
        let mut last = json!(null);
        for _ in 0..60 {
            let (ok, value) = f.command("resume", &format!("resume-{project}"), &input);
            if ok && value["state"] == "confirmed" {
                ready = true;
                break;
            }
            last = value;
            std::thread::sleep(Duration::from_millis(200));
        }
        assert!(
            ready,
            "room creation: {last}; status: {}; log: {}; room: {}",
            f.run(&["services", "status"]).1,
            std::fs::read_to_string(root.join("control.log")).unwrap_or_default(),
            f.show(project, room)
        );
    }
    let a = &mains[0].0;
    let main = f.show("A", a);
    let version = main["binding"]["version"].as_i64().unwrap();
    let send =
        json!({"project_id":"A","room_id":a,"version":version,"body":"日本語与中文 é / é；未定"});
    let sent = f.accepted("send", "send-1", &send);
    assert_eq!(
        f.accepted("send", "send-1", &send)["receipt"],
        sent["receipt"]
    );
    let event = sent["receipt"]["event_id"].as_str().unwrap();
    let draft = f.query("draft",json!({"project_id":"A","project_version":1,"origin":{"kind":"main_room","room_id":a,"binding_version":version},"selection":{"kind":"events","event_ids":[event,"$missing"]}}));
    assert_eq!(draft["automatic_summary"], "not_configured");
    assert_eq!(draft["fragments"][0]["excerpt"], send["body"]);
    assert_eq!(draft["unread"].as_array().unwrap().len(), 1);
    assert_eq!(
        f.run(&["room", "list"]).1["rooms"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let source = draft["fragments"][0]["source"].clone();
    let topic = json!({"project_id":"A","project_version":1,"name":"独立话题","origin":{"kind":"main_room","room_id":a,"binding_version":version},"brief":{"context_and_goal":"人的去敏与补写","settled_facts_and_reasons":[],"disagreements_and_questions":["未定"],"constraints_and_materials":[],"sources":[source]},"participants":[],"roster_confirmed":true});
    let created = f.accepted("create-topic", "topic-1", &topic);
    assert_eq!(
        f.accepted("create-topic", "topic-1", &topic)["room_id"],
        created["room_id"]
    );
    let topic_id = created["room_id"].as_str().unwrap();
    let frozen = f.query("reference", json!({"project_id":"A","source":source}));
    let view = json!({"project_id":"A","room_id":a,"binding_version":version,"client_id":"desktop","draft":"尚未发送","read_cursor":event});
    f.query("save-view-state", view);
    let mut next_send = send.clone();
    next_send["body"] = json!("后续消息不流入 Topic");
    f.accepted("send", "send-2", &next_send);
    let (_, timeline) = f.run(&["room", "timeline", "A", topic_id]);
    assert!(
        !timeline["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["type"] == "m.room.message")
    );
    let closed = f.show("A", topic_id);
    assert_eq!(closed["brief"]["context_and_goal"], "人的去敏与补写");
    f.accepted(
        "close",
        "close-topic",
        &json!({"project_id":"A","room_id":topic_id,"version":closed["binding"]["version"]}),
    );
    assert!(!f.command("send","closed-send",&json!({"project_id":"A","room_id":topic_id,"version":closed["binding"]["version"].as_i64().unwrap()+1,"body":"no"})).0);
    assert!(f.run(&["stop"]).0);
    std::thread::sleep(Duration::from_millis(200));
    // All disposable chat observations can disappear without losing admitted references.
    std::fs::remove_dir_all(root.join("cache")).unwrap();
    assert!(f.run(&["start"]).0);
    for _ in 0..60 {
        if f.run(&["room", "view-state", "A", a, "desktop"]).0 {
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert_eq!(
        f.query("reference", json!({"project_id":"A","source":source})),
        frozen
    );
    let (ok, view) = f.run(&["room", "view-state", "A", a, "desktop"]);
    assert!(ok, "{view}");
    assert_eq!(view["draft"], "尚未发送");
    assert_eq!(
        f.run(&["room", "list"]).1["rooms"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    let (ok, _) = f.run(&["room", "event", "B", &mains[1].0, event]);
    assert!(!ok, "event in A must not be accepted as B's event");
    let backup = root.join("backup");
    let service_backup = root.join("service-backup");
    assert!(f.run(&["backup", "create", backup.to_str().unwrap()]).0);
    assert!(
        f.run(&["services", "backup", service_backup.to_str().unwrap()])
            .0
    );
    assert!(f.run(&["stop"]).0);
    let restored = Fixture {
        root: root.join("restored"),
        payload: f.payload.clone(),
    };
    // A new directory imports the control identity before the daemon starts, rather than
    // overwriting an unrelated initialized control. Native services restore through the CLI.
    drop(Store::restore(&restored.root, &backup).unwrap());
    assert!(restored.run(&["start"]).0);
    let (ok, value) = restored.run(&[
        "services",
        "restore",
        service_backup.to_str().unwrap(),
        "--yes",
    ]);
    assert!(ok, "service restore: {value}");
    for _ in 0..100 {
        if restored.run(&["room", "view-state", "A", a, "desktop"]).0 {
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert_eq!(
        restored.query("reference", json!({"project_id":"A","source":source})),
        frozen
    );
    let (ok, view) = restored.run(&["room", "view-state", "A", a, "desktop"]);
    assert!(ok, "restored view: {view}");
    assert_eq!(view["draft"], "尚未发送");
    // The original creation remains idempotent after restoring into another directory.
    assert_eq!(
        restored.accepted("create-topic", "topic-1", &topic)["room_id"],
        created["room_id"]
    );
}
