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

#[test]
fn retry_bootstrap_waits_out_a_not_yet_migrated_database() {
    // The packaged lifecycle test caught `no such table: user` because the readiness
    // probe passed before the first migration finished. The bootstrap must keep asking
    // this root's own admin interface instead of failing on its first answer.
    let attempts = std::cell::Cell::new(0);
    let rearms = std::cell::Cell::new(0);
    let listed = retry_bootstrap(
        Instant::now() + Duration::from_secs(5),
        || {
            attempts.set(attempts.get() + 1);
            if attempts.get() < 3 {
                Err(reject(
                    "PLATFORM_BOOTSTRAP",
                    "cannot list hosted platform accounts: Command error: SQL logic error: no such table: user (1)",
                    "inspect_gitea_log",
                ))
            } else {
                Ok("listed")
            }
        },
        |_| true,
        || rearms.set(rearms.get() + 1),
    )
    .unwrap();
    assert_eq!(listed, "listed");
    assert_eq!(attempts.get(), 3);
    assert_eq!(
        rearms.get(),
        2,
        "every wait re-requests the component start"
    );
}

#[test]
fn retry_bootstrap_keeps_the_last_failure_and_respects_the_deadline() {
    let started = Instant::now();
    let error = retry_bootstrap(
        started + Duration::from_millis(300),
        || -> Result<()> {
            Err(reject(
                "PLATFORM_BOOTSTRAP",
                "cannot list hosted platform accounts: Command error: SQL logic error: no such table: user (1)",
                "inspect_gitea_log",
            ))
        },
        |_| true,
        || {},
    )
    .unwrap_err();
    assert_eq!(error.code, "PLATFORM_BOOTSTRAP");
    assert!(
        error.message.contains("no such table: user"),
        "the native detail must survive: {}",
        error.message
    );
    assert!(
        started.elapsed() >= Duration::from_millis(300),
        "retrying stops at the deadline"
    );
}

#[test]
fn retry_bootstrap_does_not_retry_a_failure_it_cannot_fix() {
    // A foreign instance holding Gitea's port answers 401 and is worth waiting out; a
    // rejected credential is not.
    let attempts = std::cell::Cell::new(0);
    let error = retry_bootstrap(
        Instant::now() + Duration::from_secs(5),
        || -> Result<()> {
            attempts.set(attempts.get() + 1);
            Err(reject(
                "NATIVE_REJECTED",
                "rejected",
                "read_back_original_intent",
            ))
        },
        |error| error.code == "PLATFORM_UNAVAILABLE",
        || {},
    )
    .unwrap_err();
    assert_eq!(error.code, "NATIVE_REJECTED");
    assert_eq!(attempts.get(), 1);
}

#[test]
fn retry_bootstrap_waits_out_an_instance_that_is_not_ours() {
    // `PLATFORM_UNAVAILABLE` is what the API reports when another process on the port
    // answers for a token minted in this root's database: worth waiting out.
    let attempts = std::cell::Cell::new(0);
    let rearms = std::cell::Cell::new(0);
    let who = retry_bootstrap(
        Instant::now() + Duration::from_secs(5),
        || {
            attempts.set(attempts.get() + 1);
            if attempts.get() < 3 {
                Err(reject(
                    "PLATFORM_UNAVAILABLE",
                    "tea API GET user not confirmed (HTTP Some(401), exit Some(0))",
                    "read_back_original_intent",
                ))
            } else {
                Ok("control admin")
            }
        },
        |error| error.code == "PLATFORM_UNAVAILABLE",
        || rearms.set(rearms.get() + 1),
    )
    .unwrap();
    assert_eq!(who, "control admin");
    assert_eq!(attempts.get(), 3);
    assert_eq!(rearms.get(), 2);
}

#[test]
fn native_detail_reports_the_last_non_empty_stderr_line() {
    assert_eq!(native_detail(b""), "no diagnostic output");
    assert_eq!(
        native_detail(b"level=warn\n\nCommand error: SQL logic error: no such table: user (1)\n\n"),
        "Command error: SQL logic error: no such table: user (1)"
    );
}
