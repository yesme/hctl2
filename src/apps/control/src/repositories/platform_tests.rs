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

/// A subprocess fixture for the native Gitea admin CLI: `admin user list` reads the
/// accounts recorded so far, `admin user create` prints a password and then records one.
fn write_gitea_fixture(path: &Path) {
    std::fs::write(
        path,
        r#"#!/bin/sh
sub="$7"
if [ "$sub" = list ]; then
    printf 'ID\tUsername\tEmail\tIsActive\n'
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

#[test]
fn the_account_list_header_is_not_an_account() {
    // `gitea admin user list` prints a header whose own second column is the literal
    // `Username` (`cmd/admin_user_list.go:44`), tab-expanded to spaces by `:41`. Reading it
    // as an account would skip creating an account really named that, and the collaborator
    // grant would then meet a 422 instead of an account.
    let tabbed = b"ID\tUsername\tEmail\tIsActive\n2\talice\talice@localhost\ttrue\n";
    assert!(account_listed(tabbed, "alice"));
    assert!(!account_listed(tabbed, "Username"));
    assert!(!account_listed(tabbed, "bob"));
    let padded =
        b"ID     Username  Email            IsActive\n2      alice     alice@localhost  true\n";
    assert!(account_listed(padded, "alice"));
    assert!(!account_listed(padded, "Username"));
    // An account really carrying that name is still found.
    let real = b"ID\tUsername\tEmail\tIsActive\n3\tUsername\tu@localhost\ttrue\n";
    assert!(account_listed(real, "Username"));
}

/// A subprocess fixture for `tea api`: a PUT records the account and the permission it was
/// granted, unless the fixture is told to lose the response, and a collaborator read answers
/// from the accounts recorded so far. Like the platform, the read names the account only and
/// never says which permission it holds.
fn write_tea_fixture(path: &Path) {
    std::fs::write(
        path,
        r#"#!/bin/sh
method="$4"
if [ "$method" = PUT ]; then
    path="$7"
    body="$(cat)"
else
    path="$5"
fi
name="${path##*/}"
if [ "$method" = GET ]; then
    if cut -f1 "$0.grants" 2>/dev/null | grep -qx "$name"; then
        printf 'HTTP/1.1 204 No Content\n' >&2
    else
        printf 'HTTP/1.1 404 Not Found\n' >&2
        printf '{}'
    fi
    exit 0
fi
printf x >> "$0.puts"
if [ ! -f "$0.lose" ]; then
    permission="${body#*\"permission\":\"}"
    permission="${permission%%\"*}"
    printf '%s\t%s\n' "$name" "$permission" >> "$0.grants"
fi
printf 'HTTP/1.1 204 No Content\n' >&2
exit 0
"#,
    )
    .unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
}

#[test]
fn collaborator_grant_writes_the_requested_permission_and_confirms_it_by_readback() {
    let temp =
        Temp(std::env::temp_dir().join(format!("hctl2-collaborator-{}", std::process::id())));
    let _ = std::fs::remove_dir_all(&temp.0);
    std::fs::create_dir_all(&temp.0).unwrap();
    write_tea_fixture(&temp.0.join("tea-fixture"));
    let puts = temp.0.join("tea-fixture.puts");
    let grants = temp.0.join("tea-fixture.grants");
    let hosted = Hosted {
        tea: temp.0.join("tea-fixture"),
        url: "http://127.0.0.1:3000".into(),
        username: "admin".into(),
        token: "fixture-only".into(),
        credential_ref: "fixture".into(),
    };
    // The permission the platform is left holding for one account.
    let held = |name: &str| -> String {
        std::fs::read_to_string(&grants)
            .unwrap_or_default()
            .lines()
            .filter_map(|line| line.split_once('\t'))
            .filter(|(granted, _)| *granted == name)
            .map(|(_, permission)| permission.to_owned())
            .next_back()
            .unwrap_or_default()
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
    assert_eq!(held("alice"), "");
    std::fs::remove_file(temp.0.join("tea-fixture.lose")).unwrap();

    hosted
        .grant_collaborator("admin/test", "alice", "read")
        .unwrap();
    assert_eq!(std::fs::read_to_string(&puts).unwrap(), "xx");
    assert_eq!(held("alice"), "read");

    // Re-running one intent writes again: a readback cannot say which permission holds, so
    // the write is what makes the reported permission true. Here it changes nothing.
    hosted
        .grant_collaborator("admin/test", "alice", "read")
        .unwrap();
    assert_eq!(std::fs::read_to_string(&puts).unwrap(), "xxx");
    assert_eq!(held("alice"), "read");

    // An account already on the list still receives a different requested permission, instead
    // of keeping the old one while the caller is told the new one.
    hosted
        .grant_collaborator("admin/test", "alice", "write")
        .unwrap();
    assert_eq!(std::fs::read_to_string(&puts).unwrap(), "xxxx");
    assert_eq!(held("alice"), "write");

    // Another account on the same repository is its own grant and disturbs neither.
    hosted
        .grant_collaborator("admin/test", "bob", "read")
        .unwrap();
    assert_eq!(std::fs::read_to_string(&puts).unwrap(), "xxxxx");
    assert_eq!(held("bob"), "read");
    assert_eq!(held("alice"), "write");
}
