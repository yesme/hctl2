use super::*;
use std::os::unix::fs::PermissionsExt;
use store::{EffectIntent, ObjectKey, Scope, Version};

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
    let client = Client::Github {
        gh: exe,
        host: "github.com".into(),
    };
    assert_eq!(
        client.read_card(&src, &original).unwrap_err().code,
        "ENTITY_CHANGED"
    );
    std::fs::remove_dir_all(&root).unwrap();
}
