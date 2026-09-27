use super::*;
use std::os::unix::fs::PermissionsExt;
use store::{EffectIntent, ObjectKey, Scope, Version};

struct Fixture(PathBuf);
impl Fixture {
    fn new(name: &str, script: &str) -> Self {
        let root = std::env::temp_dir().join(format!("task-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let exe = root.join("native");
        std::fs::write(&exe, script).unwrap();
        std::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o700)).unwrap();
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
    let f = Fixture::new(
        "preflight",
        r#"#!/bin/sh
printf '%s\n' "$*" >> "$0.calls"
case "$*" in
 *'GET repos/owner/repo') printf 'HTTP/1.1 200 OK\n' >&2; printf '{"id":1,"full_name":"owner/repo","permissions":{"admin":false}}' ;;
 *'GET user') printf 'HTTP/1.1 200 OK\n' >&2; printf '{"id":1}' ;;
 *'GET repos/owner/repo/issues/1')
   if [ -f "$0.card" ]; then printf 'HTTP/1.1 200 OK\n' >&2; cat "$0.card";
   else printf 'HTTP/1.1 503 Error\n' >&2; printf '{}'; fi ;;
 *) printf 'HTTP/1.1 500 Unexpected\n' >&2; printf '{}' ;;
esac
"#,
    );
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
    let f = Fixture::new(
        "conditional",
        r#"#!/bin/sh
printf '%s\n' "$*" >> "$0.calls"
case "$*" in
 *limited*) printf 'HTTP/2.0 429 Too Many Requests\nRetry-After: 3600\n\n{}' ;;
 *If-None-Match*) printf 'HTTP/2.0 304 Not Modified\n\n' ;;
 *) printf 'HTTP/2.0 200 OK\nETag: "v1"\n\n{"id":1}' ;;
esac
"#,
    );
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
    let f = Fixture::new(
        "reject",
        r#"#!/bin/sh
case "$*" in
 *'GET repos/owner/repo') printf 'HTTP/1.1 200 OK\n' >&2; printf '{"id":1,"full_name":"owner/repo"}' ;;
 *'GET user') printf 'HTTP/1.1 200 OK\n' >&2; printf '{"id":1}' ;;
 *'GET repos/owner/repo/issues/1') printf 'HTTP/1.1 200 OK\n' >&2; cat "$0.card" ;;
 *PATCH*) cat > "$0.input"; printf 'HTTP/1.1 %s Error\n' "$(cat "$0.status")" >&2; printf '{}' ;;
 *) printf 'HTTP/1.1 500 Unexpected\n' >&2; printf '{}' ;;
esac
"#,
    );
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
    let f = Fixture::new(
        "incremental",
        r#"#!/bin/sh
printf '%s\n' "$*" >> "$0.calls"
printf 'HTTP/1.1 200 OK\n' >&2
case "$*" in
 *'GET repos/owner/repo') printf '{"id":1,"full_name":"owner/repo","internal_tracker":{"enable_issue_dependencies":false}}' ;;
 *'GET user') printf '{"id":1}' ;;
 *'issues?'*'page=1') cat "$0.issues1" ;;
 *'issues?'*'page=2') cat "$0.issues2" ;;
 *) printf '[]' ;;
esac
"#,
    );
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
    std::fs::write(&exe,r#"#!/bin/sh
case "$*" in
 *"GET user") printf 'HTTP/1.1 200 OK\n' >&2; printf '{"id":1}' ;;
 *"GET repos/owner/repo") printf 'HTTP/1.1 200 OK\n' >&2; printf '{"id":1,"full_name":"owner/repo"}' ;;
 *POST*) cat > "$0.input"; printf x >> "$0.posts"; cp "$0.result" "$0.created"; printf 'HTTP/1.1 503 Error\n' >&2; printf '{}' ;;
 *page=2*) printf 'HTTP/1.1 200 OK\n' >&2; printf '[]' ;;
 *) printf 'HTTP/1.1 200 OK\n' >&2; if [ -f "$0.created" ]; then cat "$0.created"; else printf '[]'; fi ;;
esac
"#).unwrap();
    std::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o700)).unwrap();
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
    std::fs::write(
        &exe,
        "#!/bin/sh\nprintf 'HTTP/2.0 200 OK\\n\\n'\ncat \"$0.json\"\n",
    )
    .unwrap();
    std::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o700)).unwrap();
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
