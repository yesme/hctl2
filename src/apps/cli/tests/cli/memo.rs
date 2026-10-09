use super::{Temp, run};
use chat::{Action as ChatAction, Input as ChatInput, Source, SourceText};
use serde_json::{Value, json};
use store::{
    Actor, ActorSource, Command as StoreCommand, Expected, RecordData, Scope, TrustedActor,
};

fn ok(root: &std::path::Path, args: &[&str]) -> Value {
    let (ok, out, err) = run(root, args);
    assert!(ok, "{args:?}: {out} {err}");
    serde_json::from_str(&out).unwrap()
}

fn failure(root: &std::path::Path, args: &[&str]) -> Value {
    let (ok, out, err) = run(root, args);
    assert!(!ok, "{args:?} must fail: {out} {err}");
    serde_json::from_str(&out).unwrap()
}

fn actor() -> TrustedActor {
    TrustedActor(Actor {
        principal: "owner".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![Scope::Control, Scope::Project("P".into())],
        authority: None,
    })
}

/// Only the preconditions are seeded: one Project with a main Room and one frozen
/// Message. Every Memo command below runs through the real CLI and RPC.
fn seed_frozen_source(root: &std::path::Path, text: &str) -> (Source, i64) {
    let scope = Scope::Project("P".into());
    let actor = actor();
    let mut store = store::Store::open(root).unwrap();
    let mut project =
        chat::value_record(chat::key(scope.clone(), "project", "P"), 1, &json!({})).unwrap();
    project.data = RecordData::Project {
        repo_id: "repo-fixture".into(),
        archived: false,
        settings: store::ProjectSettings {
            selection_policy: json!({}),
            publish_review_requires_confirmation: false,
        },
    };
    let mut room =
        chat::value_record(chat::key(scope.clone(), "room", "main"), 1, &json!({})).unwrap();
    room.data = RecordData::Room {
        room_kind: store::RoomKind::Main,
        state: store::RoomState::Active,
    };
    let binding = chat::value_record(
        chat::key(scope, "room_binding", "main"),
        1,
        &json!({"project_id":"P","id":"main","name":"Main","server":{
            "binding":{"key":chat::key(Scope::Control,"chat_server","fixture"),"version":store::Version::State(1)},
            "url":"http://127.0.0.1:8008","server_name":"fixture","sender":"@control:fixture"},
            "matrix_room_id":"!main:fixture","participants":[],"brief":null,"origin":null}),
    )
    .unwrap();
    let seed = StoreCommand {
        command_id: "fixture-project".into(),
        idempotency_key: "fixture-project".into(),
        actor: actor.0.clone(),
        target: project.key.clone(),
        expected: Expected::Absent,
        binding: chat::reference(&project),
        operation: "fixture".into(),
        input: json!({}),
        input_digest: StoreCommand::digest_input("fixture", &json!({})).unwrap(),
    };
    store
        .submit(store.generation(), &actor, &seed, None, |tx| {
            tx.put(&project)?;
            tx.put(&room)?;
            tx.put(&binding)?;
            Ok(json!({}))
        })
        .unwrap();
    let (binding, room) = chat::main_binding(&store, "P").unwrap();
    let source = Source::Message {
        binding: chat::reference(&binding),
        event_id: "$frozen".into(),
        content_digest: foundation::bytes_sha256(text.as_bytes()),
    };
    let input = ChatInput {
        key: "fixture-freeze".into(),
        action: ChatAction::Freeze {
            project_id: "P".into(),
            room_id: room.id,
            version: binding.version,
            event_id: "$frozen".into(),
        },
    };
    let plan = chat::prepare(
        &store,
        input,
        vec![SourceText {
            source: source.clone(),
            body: text.into(),
            excerpt: text.into(),
        }],
        vec![],
    )
    .unwrap();
    chat::admit(&mut store, &actor, plan).unwrap();
    let version = chat::required(
        &store,
        &chat::key(Scope::Project("P".into()), "project", "P"),
    )
    .unwrap()
    .version;
    (source, version)
}

#[test]
fn memo_publish_previews_then_confirms_and_earlier_revisions_stay_readable() {
    let temp = Temp::new();
    let root = &temp.0;
    assert!(run(root, &["init", "--secret-backend", "user-file"]).0);
    let (source, project_version) = seed_frozen_source(root, "frozen decision");
    let (started, out, err) = run(root, &["start"]);
    assert!(started, "start: {out} {err}");
    let action = root.join("memo-action.json");
    let body = root.join("memo-body.md");
    std::fs::write(&body, "settled by the owner").unwrap();
    let first = json!({"project_id":"P","project_version":project_version,"memo_id":"M",
        "applicability":"repo","sources":[source],"supersedes":null,"expires_at":null});
    std::fs::write(&action, serde_json::to_vec(&first).unwrap()).unwrap();
    let publish = [
        "memo",
        "publish",
        "--input",
        action.to_str().unwrap(),
        "--body",
        body.to_str().unwrap(),
        "--key",
        "publish-one",
    ];
    // Text that never went through the command is not a Memo, and a preview writes nothing.
    assert!(
        ok(root, &["memo", "list", "P"])["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let preview = ok(root, &publish);
    assert_eq!(
        preview["effect_summary"]["input"]["action"]["body"],
        "settled by the owner"
    );
    assert_eq!(preview["effect_summary"]["result"]["revision"], 1);
    assert_eq!(
        preview["effect_summary"]["result"]["memo"]["content_digest"],
        foundation::bytes_sha256(b"settled by the owner")
    );
    assert!(
        preview["effect_summary"]["result"]["memo"]["author"]
            .as_str()
            .unwrap()
            .starts_with("local-owner:"),
        "the author is the authenticated principal, not a payload field: {preview}"
    );
    assert!(
        ok(root, &["memo", "list", "P"])["manifest"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    // A body edited after the preview is not the previewed publication.
    std::fs::write(&body, "settled by somebody else").unwrap();
    let mut confirm = publish.to_vec();
    confirm.extend([
        "--preview-token",
        preview["preview_token"].as_str().unwrap(),
    ]);
    assert_eq!(failure(root, &confirm)["error"]["code"], "PREVIEW_REQUIRED");
    std::fs::write(&body, "settled by the owner").unwrap();
    let published = ok(root, &confirm);
    assert_eq!(published["memo_id"], "M");
    assert_eq!(published["revision"], 1);
    assert_eq!(published["body_bytes"], 20);
    assert_eq!(published["memo"]["supersedes"], Value::Null);
    assert_eq!(published["memo"]["applicability"], "repo");
    assert!(
        published["memo"]["author"]
            .as_str()
            .unwrap()
            .starts_with("local-owner:"),
        "the author is the authenticated principal: {published}"
    );
    assert_eq!(
        published["memo"]["sources"][0]["key"]["kind"],
        "chat_source_reference"
    );
    // The token is consumed: the same confirmation now needs a fresh preview.
    assert_eq!(failure(root, &confirm)["error"]["code"], "PREVIEW_REQUIRED");
    let shown = ok(root, &["memo", "show", "P", "M"]);
    assert_eq!(shown["body"], "settled by the owner");
    assert_eq!(shown["current"], true);
    assert_eq!(shown["revision"], published["revision_reference"]);
    assert_eq!(
        ok(root, &["memo", "show", "P", "M", "--revision", "1"])["body"],
        "settled by the owner"
    );

    // An update is a new revision naming its exact predecessor.
    let restated = root.join("memo-body-2.md");
    std::fs::write(&restated, "restated by the owner").unwrap();
    let mut update = first.clone();
    update["supersedes"] = json!(1);
    std::fs::write(&action, serde_json::to_vec(&update).unwrap()).unwrap();
    let revise = [
        "memo",
        "publish",
        "--input",
        action.to_str().unwrap(),
        "--body",
        restated.to_str().unwrap(),
        "--key",
        "publish-two",
    ];
    let preview = ok(root, &revise);
    assert_eq!(preview["effect_summary"]["result"]["revision"], 2);
    let mut confirm = revise.to_vec();
    confirm.extend([
        "--preview-token",
        preview["preview_token"].as_str().unwrap(),
    ]);
    let revised = ok(root, &confirm);
    assert_eq!(revised["revision"], 2);
    assert_eq!(revised["memo"]["supersedes"]["key"]["id"], "M:1");
    assert_ne!(
        revised["revision_reference"], published["revision_reference"],
        "a new revision is a new object"
    );
    assert_eq!(
        ok(root, &["memo", "show", "P", "M"])["body"],
        "restated by the owner"
    );
    let earlier = ok(root, &["memo", "show", "P", "M", "--revision", "1"]);
    assert_eq!(earlier["body"], "settled by the owner");
    assert_eq!(earlier["current"], false);
    assert_eq!(earlier["revision"], published["revision_reference"]);
    assert_eq!(
        failure(root, &["memo", "show", "P", "M", "--revision", "9"])["error"]["code"],
        "MEMO_NOT_FOUND"
    );

    // A published revision is not rewritable: republishing over it names no predecessor.
    std::fs::write(&action, serde_json::to_vec(&first).unwrap()).unwrap();
    let overwrite = [
        "memo",
        "publish",
        "--input",
        action.to_str().unwrap(),
        "--body",
        body.to_str().unwrap(),
        "--key",
        "publish-three",
    ];
    let rejected = failure(root, &overwrite);
    assert_eq!(rejected["error"]["code"], "SUPERSEDES_REQUIRED");
    assert_eq!(
        rejected["error"]["recovery_action"],
        "name_superseded_revision"
    );

    // The pointer manifest filters an expired Memo mechanically; an explicit
    // reference still reads it back.
    let bounded = root.join("memo-body-3.md");
    std::fs::write(&bounded, "bounded").unwrap();
    let mut expiring = first.clone();
    expiring["memo_id"] = json!("B");
    expiring["expires_at"] = json!(1);
    let expiring_action = root.join("memo-action-b.json");
    std::fs::write(&expiring_action, serde_json::to_vec(&expiring).unwrap()).unwrap();
    let expire = [
        "memo",
        "publish",
        "--input",
        expiring_action.to_str().unwrap(),
        "--body",
        bounded.to_str().unwrap(),
        "--key",
        "publish-bounded",
    ];
    let preview = ok(root, &expire);
    let mut confirm = expire.to_vec();
    confirm.extend([
        "--preview-token",
        preview["preview_token"].as_str().unwrap(),
    ]);
    assert_eq!(ok(root, &confirm)["memo_id"], "B");
    let listed = ok(root, &["memo", "list", "P"]);
    assert_eq!(listed["items"].as_array().unwrap().len(), 2);
    assert_eq!(listed["manifest"].as_array().unwrap().len(), 1);
    assert_eq!(listed["manifest"][0]["key"]["id"], "M:2");
    assert!(
        listed["now"].as_u64().unwrap() > 1_700_000_000,
        "the filter uses the control's own clock: {listed}"
    );
    let item = listed["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["memo_id"] == "B")
        .unwrap();
    assert_eq!(item["expired"], true);
    assert_eq!(item["revision"], 1);
    assert_eq!(ok(root, &["memo", "show", "P", "B"])["body"], "bounded");
    assert!(
        ok(root, &["memo", "list", "Q"])["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(run(root, &["stop"]).0);
}

#[test]
fn memo_cli_local_input_errors_are_stdout_json() {
    let temp = Temp::new();
    let root = &temp.0;
    let body = root.join("body.md");
    std::fs::write(&body, "text").unwrap();
    let binary = root.join("body.bin");
    std::fs::write(&binary, [0xff_u8, 0xfe]).unwrap();
    let action = root.join("action.json");
    let absent = root.join("absent.json");
    let cases: [(&std::path::Path, &std::path::Path, &str); 6] = [
        (&absent, &body, "the input file does not exist"),
        (&action, &body, "not-json"),
        (&action, &body, "[]"),
        (&action, &body, r#"{"kind":"update"}"#),
        (&action, &binary, "{}"),
        (&action, &body, r#"{"body":"other text"}"#),
    ];
    for (path, body, contents) in cases {
        if path == action {
            std::fs::write(path, contents).unwrap();
        }
        let failure = failure(
            root,
            &[
                "memo",
                "publish",
                "--input",
                path.to_str().unwrap(),
                "--body",
                body.to_str().unwrap(),
                "--key",
                "bad",
            ],
        );
        assert_eq!(
            failure["error"]["code"], "MEMO_COMMAND_FAILED",
            "{contents}"
        );
        assert_eq!(
            failure["error"]["recovery_action"],
            "inspect_error_and_retry"
        );
    }
    // The queries reach the socket and report a typed error, not a panic.
    for args in [vec!["memo", "show", "P", "M"], vec!["memo", "list", "P"]] {
        let (succeeded, out, _) = run(root, &args);
        assert!(!succeeded, "{args:?} must fail without a daemon: {out}");
    }
}
