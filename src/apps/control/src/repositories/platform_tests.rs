use super::*;
use std::os::unix::fs::PermissionsExt;
use store::{Actor, ActorSource, Scope, Store, TrustedActor};

struct Temp(PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn lost_create_response_reads_original_correlation_without_second_post() {
    let temp =
        Temp(std::env::temp_dir().join(format!("hctl2-repo-platform-{}", std::process::id())));
    std::fs::create_dir_all(&temp.0).unwrap();
    let mut store = Store::open(&temp.0.join("control")).unwrap();
    let actor = TrustedActor(Actor {
        principal: "owner".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![Scope::Control],
        authority: None,
    });
    let request = serde_json::from_value(
        json!({"name":"test", "origin":"local", "platform":"local", "platform_path":"test"}),
    )
    .unwrap();
    let prepared = repo::prepare(request, None).unwrap();
    let reg = repo::admit(&mut store, &actor, "register", "register", prepared).unwrap();
    let executable = temp.0.join("tea-fixture");
    // A subprocess fixture, not an HTTP server: the native tea behavior is also exercised by complete-test.
    std::fs::write(
        &executable,
        r#"#!/bin/sh
if [ "$4" = POST ]; then
    printf x >> "$0.posts"
    touch "$0.created"
    printf 'HTTP/1.1 503 Service Unavailable\n' >&2
    printf '{}'
elif [ -f "$0.created" ]; then
    printf 'HTTP/1.1 200 OK\n' >&2
    cat "$0.json"
else
    printf 'HTTP/1.1 404 Not Found\n' >&2
    printf '{}'
fi
"#,
    )
    .unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut value = json!({"id":7,"owner":{"id":1},"has_issues":true,"full_name":"admin/test",
        "clone_url":"http://127.0.0.1:3000/admin/test.git",
        "description":format!("HCTL registration {}:{}", reg.config.control_id, reg.repo_id)});
    let response = temp.0.join("tea-fixture.json");
    std::fs::write(&response, value.to_string()).unwrap();
    let hosted = Hosted {
        tea: executable,
        url: "http://127.0.0.1:3000".into(),
        username: "admin".into(),
        token: "fixture-only".into(),
        credential_ref: "fixture".into(),
    };
    assert!(hosted.repository(&reg, false).unwrap().is_none());
    assert!(!temp.0.join("tea-fixture.posts").exists());
    assert_eq!(
        hosted.repository(&reg, true).unwrap_err().code,
        "PLATFORM_UNAVAILABLE",
        "tea exit zero with HTTP 503 must not confirm a platform step"
    );
    let observed = hosted.repository(&reg, false).unwrap().unwrap();
    assert_eq!(observed.stable_id, "7");
    assert_eq!(
        std::fs::read_to_string(temp.0.join("tea-fixture.posts")).unwrap(),
        "x"
    );
    value["description"] = json!("someone else's repository");
    std::fs::write(&response, value.to_string()).unwrap();
    assert_eq!(
        hosted.repository(&reg, true).unwrap_err().code,
        "PLATFORM_NAME_CONFLICT"
    );
    assert_eq!(
        std::fs::read_to_string(temp.0.join("tea-fixture.posts")).unwrap(),
        "x"
    );
}
