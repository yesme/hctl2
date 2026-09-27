use super::*;
use std::collections::BTreeMap;

#[test]
fn mechanical_selection_uses_server_order_relations_and_mentions_not_body() {
    let events = vec![
        json!({"event_id":"$z","type":"m.room.message","origin_server_ts":100,"content":{"body":"@human word"}}),
        json!({"event_id":"$a","type":"m.room.message","origin_server_ts":1,"content":{"body":"different","m.relates_to":{"m.in_reply_to":{"event_id":"$z"}},"m.mentions":{"user_ids":["@human:test"]}}}),
    ];
    assert_eq!(
        select(
            &events,
            &Selection::Range {
                start: "$z".into(),
                end: "$a".into()
            }
        )
        .unwrap(),
        vec!["$z", "$a"]
    );
    assert_eq!(
        select(
            &events,
            &Selection::Replies {
                event_id: "$z".into()
            }
        )
        .unwrap(),
        vec!["$a"]
    );
    assert_eq!(
        select(
            &events,
            &Selection::Mentions {
                user_id: "@human:test".into()
            }
        )
        .unwrap(),
        vec!["$a"]
    );
    assert!(
        select(
            &events,
            &Selection::Range {
                start: "$a".into(),
                end: "$z".into()
            }
        )
        .is_err()
    );
}

#[test]
fn same_structured_action_has_same_input_and_bridge_or_missing_actor_rejected() {
    let binding = Reference {
        key: chat::key(Scope::Control, "chat_server", "local"),
        version: Version::State(1),
    };
    let policy = chat::IdentityPolicy {
        binding,
        humans: BTreeMap::from([
            ("@alice:test".into(), "owner".into()),
            ("@bridge:test".into(), "owner".into()),
        ]),
        service_users: vec!["@bridge:test".into()],
        allowed_actions: vec!["close".into()],
    };
    let event = json!({"type":"io.hctl2.action","sender":"@alice:test","event_id":"$act","content":{
        "target":{"key":{"scope":{"kind":"project","id":"p"},"kind":"room_binding","id":"topic"},"version":{"state":2}},
        "action":{"kind":"close","project_id":"p","room_id":"topic","version":2}}});
    let direct = chat::normalize_human_action(&policy, "p", &event).unwrap();
    let provider = chat::normalize_human_action(&policy, "p", &event).unwrap();
    assert_eq!(direct.digest, provider.digest);
    assert_eq!(
        serde_json::to_value(direct.input).unwrap(),
        serde_json::to_value(provider.input).unwrap()
    );
    for sender in ["@bridge:test", "@hctl2_control:test", "@unmapped:test"] {
        let mut bad = event.clone();
        bad["sender"] = json!(sender);
        assert!(chat::normalize_human_action(&policy, "p", &bad).is_err());
    }
    let mut ordinary = event.clone();
    ordinary["type"] = json!("m.room.message");
    assert!(chat::normalize_human_action(&policy, "p", &ordinary).is_err());
    let mut wrong_target = event.clone();
    wrong_target["content"]["target"]["key"]["id"] = json!("other");
    assert!(chat::normalize_human_action(&policy, "p", &wrong_target).is_err());
    let mut missing = event;
    missing["content"].as_object_mut().unwrap().remove("target");
    assert!(chat::normalize_human_action(&policy, "p", &missing).is_err());
}

#[test]
fn inbox_durable_duplicate_conflict_corruption_and_no_governance_command() {
    let root = std::env::temp_dir().join(format!(
        "hctl-inbox-{}-{}",
        std::process::id(),
        ruma::TransactionId::new()
    ));
    let inbox = observations(&root);
    let event = json!({"event_id":"$event","room_id":"!room:test","type":"m.room.message","content":{"body":"hello","number":1.5}});
    let body = json!({"events":[event]}).to_string();
    inbox
        .accept("txn", body.as_bytes(), std::slice::from_ref(&event))
        .unwrap();
    observations(&root)
        .accept("txn", body.as_bytes(), &[event])
        .unwrap();
    assert_eq!(
        inbox
            .accept("txn", b"{\"events\":[]}", &[])
            .unwrap_err()
            .code,
        "INBOX_CONFLICT"
    );
    assert!(!root.join("control.sqlite").exists());
    std::fs::write(root.join("cache/chat-inbox.sqlite"), b"corrupt").unwrap();
    assert!(
        observations(&root)
            .accept("new", b"{\"events\":[]}", &[])
            .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn remote_or_redirectable_configuration_is_rejected() {
    for url in [
        "https://127.0.0.1:8008",
        "http://example.com",
        "http://127.0.0.1:8008/path",
        "http://user@127.0.0.1:8008",
    ] {
        let server = chat::Server {
            binding: Reference {
                key: chat::key(Scope::Control, "chat_server", "local"),
                version: Version::State(1),
            },
            url: url.into(),
            server_name: "local".into(),
            sender: "@hctl2_control:local".into(),
        };
        assert!(matrix::Client::new(server, "secret".into()).is_err());
    }
}
