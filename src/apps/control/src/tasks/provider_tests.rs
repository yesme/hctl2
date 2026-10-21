use super::*;
use store::{EffectIntent, ObjectKey, Scope, Version};

/// Point a writable path at a built fixture script. The test never writes the
/// script it executes: a file just opened for writing can answer execve with
/// ETXTBSY on Linux while another test thread's forked child still holds the
/// write descriptor. A link keeps `$0` at this path, so the `$0.*` state the
/// fixtures keep stays beside it in the writable directory.
use crate::test_fixtures::link_fixture;

struct Fixture(PathBuf);
impl Fixture {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("task-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        link_fixture(&format!("provider-{name}.sh"), &root.join("native"));
        Self(root)
    }
    fn client(&self) -> Client {
        Client::Gitea(Hosted::fixture(
            self.0.join("native"),
            "http://127.0.0.1:3000".into(),
            "owner".into(),
            "test-only".into(),
        ))
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn preflight_failure_or_stale_card_never_crosses_dispatch_boundary() {
    let f = Fixture::new("preflight");
    let c = f.client();
    let raw = json!({"id":1,"number":1,"title":"old","body":"body","state":"open","updated_at":"t0","content_version":0});
    let card = normalize(&src(), raw.clone()).unwrap();
    let e = effect("task.update", json!({"card":card,"fields":{"title":"new"}}));
    assert_eq!(
        c.effect_with_dispatch(
            &src(),
            &e,
            true,
            || panic!("pre-read must precede dispatch"),
            |_| Ok(())
        )
        .unwrap_err()
        .code,
        "PLATFORM_UNAVAILABLE"
    );
    let mut changed = raw.clone();
    changed["title"] = json!("human edit");
    std::fs::write(f.0.join("native.card"), changed.to_string()).unwrap();
    assert_eq!(
        c.effect_with_dispatch(
            &src(),
            &e,
            true,
            || panic!("conflict must precede dispatch"),
            |_| Ok(())
        )
        .unwrap_err()
        .code,
        "REMOTE_CONFLICT"
    );
    let deletion = effect("task.delete", json!({"card":card}));
    assert_eq!(
        c.effect_with_dispatch(
            &src(),
            &deletion,
            true,
            || panic!("permissions must precede dispatch"),
            |_| Ok(())
        )
        .unwrap_err()
        .code,
        "PERMISSION_DENIED"
    );
    std::fs::write(f.0.join("native.card"), raw.to_string()).unwrap();
    assert_eq!(
        c.effect_with_dispatch(
            &src(),
            &e,
            true,
            || Err(reject("FENCED", "old writer", "retry")),
            |_| Ok(())
        )
        .unwrap_err()
        .code,
        "FENCED"
    );
    let calls = std::fs::read_to_string(f.0.join("native.calls")).unwrap();
    assert!(!calls.contains("PATCH") && !calls.contains("DELETE"));
}

#[test]
fn github_conditional_read_and_rate_limit_stop_native_requests() {
    let f = Fixture::new("conditional");
    let c = Client::github(
        f.0.join("native"),
        "github.com".into(),
        crate::github::Credential::fixture(&f.0),
    );
    assert_eq!(
        c.api("GET", "unauthorized", None).unwrap_err().code,
        "GITHUB_LOGIN_REQUIRED"
    );
    assert_eq!(c.api("GET", "user", None).unwrap(), Some(json!({"id":1})));
    assert_eq!(c.api("GET", "user", None).unwrap(), Some(json!({"id":1})));
    assert!(
        std::fs::read_to_string(f.0.join("native.calls"))
            .unwrap()
            .contains("If-None-Match: \"v1\"")
    );
    assert_eq!(
        c.api("GET", "limited", None).unwrap_err().code,
        "PROVIDER_RATE_LIMITED"
    );
    let before = std::fs::read_to_string(f.0.join("native.calls")).unwrap();
    assert_eq!(
        c.api("GET", "user", None).unwrap_err().code,
        "PROVIDER_RATE_LIMITED"
    );
    assert_eq!(
        std::fs::read_to_string(f.0.join("native.calls")).unwrap(),
        before
    );
}

#[test]
fn write_rejection_is_terminal_only_when_provider_response_proves_no_effect() {
    let f = Fixture::new("reject");
    let raw = json!({"id":1,"number":1,"title":"old","body":"body","state":"open","updated_at":"t0","content_version":0});
    std::fs::write(f.0.join("native.card"), raw.to_string()).unwrap();
    let card = normalize(&src(), raw).unwrap();
    let c = f.client();
    for (status, fields, terminal) in [
        (403, json!({"title":"new"}), true),
        (409, json!({"body":"new"}), true),
        (409, json!({"title":"new","body":"new"}), false),
        (422, json!({"title":"new"}), false),
        (503, json!({"body":"new"}), false),
    ] {
        std::fs::write(f.0.join("native.status"), status.to_string()).unwrap();
        let mut evidence = None;
        let e = effect("task.update", json!({"card":card,"fields":fields}));
        let error = c
            .effect_with_dispatch(
                &src(),
                &e,
                true,
                || Ok(()),
                |v| {
                    evidence = Some(v);
                    Ok(())
                },
            )
            .unwrap_err();
        assert_eq!(
            error.code,
            if terminal {
                "PROVIDER_REJECTED"
            } else {
                "RESULT_UNKNOWN"
            }
        );
        assert_eq!(evidence.is_some(), terminal);
        if let Some(evidence) = evidence {
            assert_eq!(
                evidence["prior_read"]["card"]["entity"],
                serde_json::to_value(&card.entity).unwrap()
            );
        }
    }
}

#[test]
fn incremental_board_read_does_not_rehydrate_unchanged_cards_and_paginates_changes() {
    let f = Fixture::new("incremental");
    let mut cards = vec![];
    for number in 1..=20 {
        cards.push(normalize(&src(),json!({"id":number,"number":number,"title":"card","body":"body","state":"open","updated_at":"2026-09-27T01:02:03Z","comments":1})).unwrap());
    }
    let binding = effect("task.update", json!({})).binding;
    let previous = Snapshot {
        source: binding.clone(),
        observed_at: task::now(),
        complete: true,
        error: None,
        cards,
        stable_groups: vec![],
    };
    std::fs::write(
        f.0.join("native.issues1"),
        serde_json::to_string(&previous.cards.iter().map(|c| &c.raw).collect::<Vec<_>>()).unwrap(),
    )
    .unwrap();
    std::fs::write(f.0.join("native.issues2"), "[]").unwrap();
    let c = f.client();
    let result = c
        .snapshot_since(&src(), binding.clone(), Some(&previous))
        .unwrap();
    assert_eq!(result.cards.len(), 20);
    let calls = std::fs::read_to_string(f.0.join("native.calls")).unwrap();
    assert!(calls.contains("since=2026-09-27T01:02:00Z"));
    assert!(!calls.contains("/comments") && !calls.contains("/dependencies"));
    assert_eq!(calls.lines().count(), 6);
    let new = json!({"id":21,"number":21,"title":"new","body":"body","state":"open","updated_at":"2026-09-27T01:02:30Z","comments":0});
    std::fs::write(f.0.join("native.issues2"), json!([new]).to_string()).unwrap();
    let result = c.snapshot_since(&src(), binding, Some(&previous)).unwrap();
    assert_eq!(result.cards.len(), 21);
    assert!(
        std::fs::read_to_string(f.0.join("native.calls"))
            .unwrap()
            .contains("page=3")
    );
}

pub(super) fn src() -> Source {
    Source {
        id: "source".into(),
        repo_id: "repo".into(),
        port_kind: "task_source".into(),
        candidate: repo::SourceCandidate {
            id: "gitea_issues".into(),
            provider: "gitea_issues".into(),
            actual_source_and_scope: "owner/repo".into(),
            recommended: true,
            create: true,
            field_writeback: true,
            can_claim: true,
            available: true,
        },
        platform: repo::PlatformObservation {
            instance: "http://127.0.0.1:3000".into(),
            stable_id: "1".into(),
            full_name: "owner/repo".into(),
            clone_url: "http://127.0.0.1:3000/owner/repo.git".into(),
            account_id: "1".into(),
            has_issues: true,
            can_write_issues: true,
            credential_ref: "test".into(),
        },
        capabilities: task::Capabilities {
            create: true,
            field_writeback: true,
            conditional_write: true,
            conditional_fields: vec!["body".into()],
            placement: false,
            delete: true,
        },
        board_scope_stable_id: "1".into(),
        binding_revision: 1,
        active: true,
        human_account: None,
        auto_complete_provider_done: false,
    }
}

#[test]
fn create_response_is_only_a_locator_not_a_confirmation() {
    let raw = json!({"id":1,"node_id":"node1","number":1,"title":"new",
        "body":"body\n<!-- original command -->","state":"open","updated_at":"t0"});
    for (name, status, observed, expected) in [
        ("missing", 404, raw.clone(), "RESULT_UNKNOWN"),
        ("unavailable", 503, raw.clone(), "PROVIDER_UNAVAILABLE"),
        (
            "changed-body",
            200,
            {
                let mut changed = raw.clone();
                changed["body"] = json!("a concurrent human edit");
                changed
            },
            "RESULT_UNKNOWN",
        ),
        (
            "other-entity",
            200,
            {
                let mut changed = raw.clone();
                changed["node_id"] = json!("other-node");
                changed
            },
            "ENTITY_CHANGED",
        ),
    ] {
        let f = Fixture::new("create-locator");
        std::fs::write(f.0.join("native.card"), raw.to_string()).unwrap();
        std::fs::write(f.0.join("native.readback"), observed.to_string()).unwrap();
        std::fs::write(f.0.join("native.status"), status.to_string()).unwrap();
        let mut source = src();
        source.candidate.provider = "github_issues".into();
        source.platform.instance = "github.com".into();
        let client = Client::github(
            f.0.join("native"),
            "github.com".into(),
            crate::github::Credential::fixture(&f.0),
        );
        let intent = effect(
            "task.create",
            json!({"title":"new", "body":raw["body"],
            "marker":"<!-- original command -->"}),
        );
        assert_eq!(
            client.effect(&source, &intent, true).unwrap_err().code,
            expected,
            "{name}"
        );
        let calls = std::fs::read_to_string(f.0.join("native.calls")).unwrap();
        assert_eq!(
            calls.lines().filter(|l| l.starts_with("POST ")).count(),
            1,
            "{name}"
        );
        assert!(calls.contains("GET repos/owner/repo/issues/1"), "{name}");
    }
}

pub(super) fn effect(operation: &str, write: Value) -> EffectIntent {
    let binding = Reference {
        key: ObjectKey {
            scope: Scope::Repo("repo".into()),
            kind: "task_source".into(),
            id: "source".into(),
        },
        version: Version::State(1),
    };
    EffectIntent {
        intent_id: "effect".into(),
        owner: binding.clone(),
        binding,
        operation: operation.into(),
        target: "card".into(),
        conflict_scope: "card".into(),
        permission_scope: Scope::Project("P".into()),
        input: json!({"write":write}),
        input_digest: "0".repeat(64),
        idempotency_key: "effect".into(),
    }
}
#[test]
fn tea_http_error_unknown_create_reads_exact_marker_once() {
    let root = std::env::temp_dir().join(format!("task-tea-fixture-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let exe = root.join("tea");
    link_fixture("provider-tea-http-error.sh", &exe);
    let raw = json!([{"id":1,"number":1,"title":"new","body":"body\n\n<!-- exact -->","state":"open","content_version":0,"updated_at":"t0","labels":[]}]);
    std::fs::write(root.join("tea.result"), raw.to_string()).unwrap();
    let client = Client::Gitea(Hosted::fixture(
        exe,
        "http://127.0.0.1:3000".into(),
        "owner".into(),
        "test-only".into(),
    ));
    let e = effect(
        "task.create",
        json!({"title":"new","body":"body\n\n<!-- exact -->","marker":"<!-- exact -->"}),
    );
    assert_eq!(
        client.effect(&src(), &e, false).unwrap_err().code,
        "RESULT_UNKNOWN"
    );
    assert_eq!(client.effect(&src(), &e, true).unwrap().number, 1);
    assert_eq!(client.effect(&src(), &e, false).unwrap().number, 1);
    assert_eq!(
        std::fs::read_to_string(root.join("tea.posts")).unwrap(),
        "x"
    );
    let mut changed = raw;
    changed[0]["body"] = json!("edited <!-- exact -->");
    std::fs::write(root.join("tea.created"), changed.to_string()).unwrap();
    assert_eq!(
        client.effect(&src(), &e, false).unwrap_err().code,
        "RESULT_UNKNOWN"
    );
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn github_native_headers_and_transfer_do_not_retarget_original_entity() {
    let root = std::env::temp_dir().join(format!("task-gh-fixture-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let exe = root.join("gh");
    link_fixture("provider-gh-native-headers.sh", &exe);
    let mut src = src();
    src.candidate.provider = "github_issues".into();
    src.platform.instance = "github.com".into();
    let raw = json!({"id":1,"node_id":"original","number":1,"title":"title","body":"body","state":"open","updated_at":"t0","labels":[]});
    let original = normalize(&src, raw.clone()).unwrap();
    let mut moved = raw;
    moved["node_id"] = json!("transferred");
    std::fs::write(root.join("gh.json"), moved.to_string()).unwrap();
    let client = Client::github(
        exe,
        "github.com".into(),
        crate::github::Credential::fixture(&root),
    );
    assert_eq!(
        client.read_card(&src, &original).unwrap_err().code,
        "ENTITY_CHANGED"
    );
    std::fs::remove_dir_all(&root).unwrap();
}
