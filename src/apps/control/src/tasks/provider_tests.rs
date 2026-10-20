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
        Self::labelled(name, name)
    }
    /// The same script for several tests, each in its own directory.
    fn labelled(name: &str, label: &str) -> Self {
        let root = std::env::temp_dir().join(format!("task-{label}-{}", std::process::id()));
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
    let c = Client::github(f.0.join("native"), "github.com".into());
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
        let client = Client::github(f.0.join("native"), "github.com".into());
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
    let client = Client::github(exe, "github.com".into());
    assert_eq!(
        client.read_card(&src, &original).unwrap_err().code,
        "ENTITY_CHANGED"
    );
    std::fs::remove_dir_all(&root).unwrap();
}

const DONE: &str = "Completed in HCTL2 <!-- hctl2:control:task:complete -->";

/// An open issue 1 and the completion writeback intent for it.
fn completion(f: &Fixture) -> EffectIntent {
    let raw = json!({"id":1,"number":1,"title":"t","body":"b","state":"open","updated_at":"t0","content_version":0});
    std::fs::write(f.0.join("native.card"), raw.to_string()).unwrap();
    let card = normalize(&src(), raw).unwrap();
    effect(
        task::COMPLETION_WRITEBACK,
        json!({"card":card,"fields":{"comment":DONE,"state":"closed"},"lifecycle_version":2}),
    )
}

fn count(f: &Fixture, name: &str) -> usize {
    std::fs::read_to_string(f.0.join(format!("native.{name}")))
        .map(|s| s.len())
        .unwrap_or(0)
}

#[test]
fn completion_writeback_comments_before_closing_and_a_retry_sends_nothing_again() {
    let f = Fixture::labelled("completion", "completion-twice");
    let e = completion(&f);
    let c = f.client();
    let mut dispatched = 0;
    let card = c
        .effect_with_dispatch(
            &src(),
            &e,
            true,
            || {
                dispatched += 1;
                Ok(())
            },
            |_| panic!("nothing was refused"),
        )
        .unwrap();
    // One dispatch boundary for the whole intent, though it sends two writes.
    assert_eq!(dispatched, 1);
    assert_eq!(card.stage, "closed");
    assert_eq!(card.comments, vec![json!({"id":100,"body":DONE})]);
    let calls = std::fs::read_to_string(f.0.join("native.calls")).unwrap();
    let post = calls.find("POST").unwrap();
    let patch = calls.find("PATCH").unwrap();
    assert!(post < patch, "{calls}");
    assert_eq!(
        std::fs::read_to_string(f.0.join("native.patch")).unwrap(),
        r#"{"state":"closed"}"#
    );
    // Driving the same intent again finds both results and writes nothing.
    let again = c
        .effect_with_dispatch(
            &src(),
            &e,
            true,
            || panic!("nothing left to send"),
            |_| panic!("nothing was refused"),
        )
        .unwrap();
    assert_eq!(again.stage, "closed");
    assert_eq!((count(&f, "posts"), count(&f, "closes")), (1, 1));
}

#[test]
fn a_lost_completion_comment_response_is_read_back_and_the_card_still_closes_once() {
    let f = Fixture::labelled("completion", "completion-lost");
    let e = completion(&f);
    std::fs::write(f.0.join("native.comment-lost"), "").unwrap();
    let card = f
        .client()
        .effect_with_dispatch(&src(), &e, true, || Ok(()), |_| panic!("not refused"))
        .unwrap();
    assert_eq!(card.stage, "closed");
    assert_eq!((count(&f, "posts"), count(&f, "closes")), (1, 1));
}

#[test]
fn completion_writeback_refusals_are_terminal_and_never_close_without_the_comment() {
    let f = Fixture::labelled("completion", "completion-refused");
    let e = completion(&f);
    std::fs::write(f.0.join("native.comment-status"), "403").unwrap();
    let mut evidence = None;
    let error = f
        .client()
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
    assert_eq!(error.code, "PROVIDER_REJECTED");
    assert_eq!(
        evidence.unwrap()["prior_read"]["matching_comments"],
        json!([])
    );
    assert_eq!((count(&f, "posts"), count(&f, "closes")), (0, 0));
    assert!(
        !std::fs::read_to_string(f.0.join("native.calls"))
            .unwrap()
            .contains("PATCH")
    );
    // The comment goes out, closing is refused: refused, with the comment as evidence.
    std::fs::remove_file(f.0.join("native.comment-status")).unwrap();
    std::fs::write(f.0.join("native.close-status"), "403").unwrap();
    let mut evidence = None;
    let error = f
        .client()
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
    assert_eq!(error.code, "PROVIDER_REJECTED");
    let evidence = evidence.unwrap();
    assert_eq!(evidence["prior_read"]["comment"]["body"], DONE);
    assert_eq!(evidence["prior_read"]["card"]["stage"], "open");
    assert_eq!((count(&f, "posts"), count(&f, "closes")), (1, 0));
}

#[test]
fn an_unknown_completion_writeback_only_reads_its_comment_back_but_may_still_close() {
    let f = Fixture::labelled("completion", "completion-unknown");
    let e = completion(&f);
    let c = f.client();
    // Nothing read back: no comment is sent again and the card is not closed without it.
    let error = c
        .completion_writeback(
            &src(),
            &e,
            false,
            true,
            || panic!("nothing may be sent"),
            |_| panic!("not refused"),
        )
        .unwrap_err();
    assert_eq!(error.code, "RESULT_UNKNOWN");
    assert_eq!((count(&f, "posts"), count(&f, "closes")), (0, 0));
    // The comment is read back and the card is open: closing still needs the permission.
    std::fs::write(
        f.0.join("native.comments"),
        json!([{"id":100,"body":DONE}]).to_string(),
    )
    .unwrap();
    let error = c
        .completion_writeback(
            &src(),
            &e,
            false,
            false,
            || panic!("nothing may be sent"),
            |_| panic!("not refused"),
        )
        .unwrap_err();
    assert_eq!(error.code, "RESULT_UNKNOWN");
    let mut rechecked = 0;
    let card = c
        .completion_writeback(
            &src(),
            &e,
            false,
            true,
            || {
                rechecked += 1;
                Ok(())
            },
            |_| panic!("not refused"),
        )
        .unwrap();
    assert_eq!(rechecked, 1);
    assert_eq!(card.stage, "closed");
    assert_eq!((count(&f, "posts"), count(&f, "closes")), (0, 1));
}
