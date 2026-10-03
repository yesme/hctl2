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

/// A subprocess fixture for the native Gitea admin CLI: `admin user list` reads the
/// accounts recorded so far, `admin user create` prints a password and then records one.
fn write_gitea_fixture(path: &Path) {
    std::fs::write(
        path,
        r#"#!/bin/sh
sub="$7"
if [ "$sub" = list ]; then
    cat "$0.users" 2>/dev/null
    exit 0
fi
if [ "$sub" = create ]; then
    name="$9"
    printf "generated random password is '%s'\n" 'fixture-initial-password'
    if [ -f "$0.fail" ]; then
        exit 1
    fi
    printf x >> "$0.creates"
    printf '2   %s   %s@localhost   active   public\n' "$name" "$name" >> "$0.users"
    exit 0
fi
exit 1
"#,
    )
    .unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
}

fn fixture_paths(dir: &Path) -> HostedPaths {
    HostedPaths {
        install: dir.to_path_buf(),
        gitea: dir.join("gitea-fixture"),
        config: dir.join("app.ini"),
        work_path: dir.join("data"),
    }
}

#[test]
fn human_account_is_created_once_and_a_failed_create_leaks_no_password() {
    let temp =
        Temp(std::env::temp_dir().join(format!("hctl2-human-account-{}", std::process::id())));
    let _ = std::fs::remove_dir_all(&temp.0);
    std::fs::create_dir_all(&temp.0).unwrap();
    write_gitea_fixture(&temp.0.join("gitea-fixture"));
    let paths = fixture_paths(&temp.0);
    let creates = temp.0.join("gitea-fixture.creates");

    // A failed create must not be reported as an account with a usable password: Gitea
    // prints the generated password before it creates the user.
    std::fs::write(temp.0.join("gitea-fixture.fail"), b"").unwrap();
    // Not `unwrap_err`: the success type carries an initial password and so has no `Debug`.
    let failed = match ensure_human_account(&paths, "alice") {
        Ok(_) => panic!("a failed native create must not be reported as an account"),
        Err(rejected) => rejected,
    };
    assert_eq!(failed.code, "PLATFORM_BOOTSTRAP");
    assert!(
        !failed.message.contains("fixture-initial-password"),
        "a rejected create must not carry the printed password: {}",
        failed.message
    );
    assert!(!creates.exists(), "a failed create is not a create");
    std::fs::remove_file(temp.0.join("gitea-fixture.fail")).unwrap();

    let created = ensure_human_account(&paths, "alice").unwrap();
    assert!(created.created);
    assert_eq!(
        created.initial_password.as_deref(),
        Some("fixture-initial-password")
    );
    assert_eq!(std::fs::read_to_string(&creates).unwrap(), "x");

    // Retry after a lost result: the account is now listed, so it is reused and the
    // native create is not sent a second time. There is no password to return either.
    let reused = ensure_human_account(&paths, "alice").unwrap();
    assert!(!reused.created);
    assert_eq!(reused.initial_password, None);
    assert_eq!(std::fs::read_to_string(&creates).unwrap(), "x");

    // A different username is a different account.
    let other = ensure_human_account(&paths, "bob").unwrap();
    assert!(other.created);
    assert_eq!(std::fs::read_to_string(&creates).unwrap(), "xx");
}

/// A subprocess fixture for `tea api`: collaborator reads answer from the grants recorded
/// so far, and a PUT records one only when the fixture is not told to lose the response.
fn write_tea_fixture(path: &Path) {
    std::fs::write(
        path,
        r#"#!/bin/sh
method="$4"
if [ "$method" = PUT ]; then
    path="$7"
else
    path="$5"
fi
name="${path##*/}"
if [ "$method" = GET ]; then
    if grep -qx "$name" "$0.grants" 2>/dev/null; then
        printf 'HTTP/1.1 204 No Content\n' >&2
    else
        printf 'HTTP/1.1 404 Not Found\n' >&2
        printf '{}'
    fi
    exit 0
fi
printf x >> "$0.puts"
if [ ! -f "$0.lose" ]; then
    printf '%s\n' "$name" >> "$0.grants"
fi
printf 'HTTP/1.1 204 No Content\n' >&2
exit 0
"#,
    )
    .unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
}

#[test]
fn collaborator_grant_is_confirmed_by_readback_and_not_sent_twice() {
    let temp =
        Temp(std::env::temp_dir().join(format!("hctl2-collaborator-{}", std::process::id())));
    let _ = std::fs::remove_dir_all(&temp.0);
    std::fs::create_dir_all(&temp.0).unwrap();
    write_tea_fixture(&temp.0.join("tea-fixture"));
    let puts = temp.0.join("tea-fixture.puts");
    let hosted = Hosted {
        tea: temp.0.join("tea-fixture"),
        url: "http://127.0.0.1:3000".into(),
        username: "admin".into(),
        token: "fixture-only".into(),
        credential_ref: "fixture".into(),
    };

    // A lost response is not a grant: the PUT succeeded but the readback cannot confirm it.
    std::fs::write(temp.0.join("tea-fixture.lose"), b"").unwrap();
    assert_eq!(
        hosted
            .grant_collaborator("admin/test", "alice", "read")
            .unwrap_err()
            .code,
        "PLATFORM_READBACK"
    );
    std::fs::remove_file(temp.0.join("tea-fixture.lose")).unwrap();

    hosted
        .grant_collaborator("admin/test", "alice", "read")
        .unwrap();
    assert_eq!(std::fs::read_to_string(&puts).unwrap(), "xx");

    // An existing grant is read, not re-sent.
    hosted
        .grant_collaborator("admin/test", "alice", "read")
        .unwrap();
    assert_eq!(std::fs::read_to_string(&puts).unwrap(), "xx");

    // Another account on the same repository is its own grant.
    hosted
        .grant_collaborator("admin/test", "bob", "write")
        .unwrap();
    assert_eq!(std::fs::read_to_string(&puts).unwrap(), "xxx");
}
