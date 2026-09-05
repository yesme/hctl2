use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    repo: PathBuf,
    first_commit: String,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root = temporary_directory(name);
        let repo = root.join("repo");
        git(
            None,
            [
                OsStr::new("init"),
                OsStr::new("-b"),
                OsStr::new("main"),
                repo.as_os_str(),
            ],
        );
        git(
            Some(&repo),
            [
                OsStr::new("config"),
                OsStr::new("user.name"),
                OsStr::new("HCTL2 Test"),
            ],
        );
        git(
            Some(&repo),
            [
                OsStr::new("config"),
                OsStr::new("user.email"),
                OsStr::new("hctl2@example.invalid"),
            ],
        );
        fs::create_dir(repo.join(".hctl2")).expect("identity directory");
        fs::write(repo.join(".hctl2/repo.toml"), "repo_id = \"repo-test\"\n")
            .expect("identity file");
        fs::write(repo.join("README.md"), "base\n").expect("tracked file");
        git(Some(&repo), [OsStr::new("add"), OsStr::new(".")]);
        git(
            Some(&repo),
            [OsStr::new("commit"), OsStr::new("-m"), OsStr::new("base")],
        );
        let first_commit = git_stdout(Some(&repo), [OsStr::new("rev-parse"), OsStr::new("HEAD")]);
        Self {
            root,
            repo,
            first_commit,
        }
    }

    fn second_commit(&self) -> String {
        fs::write(self.repo.join("README.md"), "second\n").expect("second revision");
        git(
            Some(&self.repo),
            [OsStr::new("add"), OsStr::new("README.md")],
        );
        git(
            Some(&self.repo),
            [OsStr::new("commit"), OsStr::new("-m"), OsStr::new("second")],
        );
        git_stdout(
            Some(&self.repo),
            [OsStr::new("rev-parse"), OsStr::new("HEAD")],
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn repository_inspection_keeps_three_identity_groups_separate() {
    let fixture = Fixture::new("inspect");
    git(
        Some(&fixture.repo),
        [
            OsStr::new("remote"),
            OsStr::new("add"),
            OsStr::new("origin"),
            OsStr::new("https://secret@example.invalid/project.git"),
        ],
    );
    let output = tool()
        .args(["repo", "inspect", "--path"])
        .arg(&fixture.repo)
        .args(["--ref", "refs/heads/main"])
        .output()
        .expect("tool must run");
    assert_success(&output);
    let record = json_stdout(&output);
    assert_eq!(record["schema"], "hctl2.repository-inspection.v1");
    assert_eq!(record["evidence_level"], "toolbox_readback");
    assert!(
        Path::new(record["git"]["path"].as_str().expect("Git path")).is_absolute(),
        "resolved Git path must be absolute"
    );
    assert!(record["common_directory_identity"]["git_common_dir"].is_string());
    assert_eq!(
        record["stable_repo_identity"]["status"],
        serde_json::Value::String("committed".to_owned())
    );
    assert_eq!(
        record["repository_state"]["head_commit_sha"],
        fixture.first_commit
    );
    assert_eq!(record["auxiliary_evidence"]["remotes"][0]["name"], "origin");
    assert_eq!(
        record["auxiliary_evidence"]["remotes"][0]["urls"][0],
        "https://***@example.invalid/project.git"
    );
}

#[test]
fn repository_inspection_ignores_inherited_git_environment() {
    let fixture = Fixture::new("inspect-environment");
    let output = tool()
        .args(["repo", "inspect", "--path"])
        .arg(&fixture.repo)
        .env("GIT_DIR", fixture.root.join("not-a-git-directory"))
        .env(
            "GIT_COMMON_DIR",
            fixture.root.join("wrong-common-directory"),
        )
        .env("GIT_WORK_TREE", fixture.root.join("wrong-worktree"))
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "core.bare")
        .env("GIT_CONFIG_VALUE_0", "true")
        .output()
        .expect("tool must run");
    assert_success(&output);
    let record = json_stdout(&output);
    let expected_root = fixture.repo.canonicalize().expect("canonical repository");
    assert_eq!(
        record["repository_state"]["worktree_root"].as_str(),
        expected_root.to_str()
    );
    assert_eq!(
        record["repository_state"]["head_commit_sha"],
        fixture.first_commit
    );
}

#[cfg(unix)]
#[test]
fn repository_inspection_rejects_a_committed_identity_symlink() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new("identity-symlink");
    fs::remove_file(fixture.repo.join(".hctl2/repo.toml")).expect("remove identity file");
    symlink("../README.md", fixture.repo.join(".hctl2/repo.toml")).expect("identity symlink");
    git(
        Some(&fixture.repo),
        [OsStr::new("add"), OsStr::new(".hctl2/repo.toml")],
    );
    git(
        Some(&fixture.repo),
        [
            OsStr::new("commit"),
            OsStr::new("-m"),
            OsStr::new("replace identity with symlink"),
        ],
    );

    assert_error_code(
        tool()
            .args(["repo", "inspect", "--path"])
            .arg(&fixture.repo)
            .output()
            .expect("tool must run"),
        "HCTL2_TOOL_REPO_IDENTITY_INVALID",
    );
}

#[test]
fn repository_inspection_rejects_non_repo_bare_repo_and_submodule() {
    let root = temporary_directory("repo-errors");
    let non_repo = root.join("plain");
    fs::create_dir(&non_repo).expect("plain directory");
    assert_error_code(
        tool()
            .args(["repo", "inspect", "--path"])
            .arg(&non_repo)
            .output()
            .expect("tool must run"),
        "HCTL2_TOOL_NOT_GIT_REPOSITORY",
    );

    let bare = root.join("bare.git");
    git(
        None,
        [OsStr::new("init"), OsStr::new("--bare"), bare.as_os_str()],
    );
    assert_error_code(
        tool()
            .args(["repo", "inspect", "--path"])
            .arg(&bare)
            .output()
            .expect("tool must run"),
        "HCTL2_TOOL_BARE_REPOSITORY_UNSUPPORTED",
    );

    let parent = Fixture::new("submodule-parent");
    let child = Fixture::new("submodule-child");
    git(
        Some(&parent.repo),
        [
            OsStr::new("-c"),
            OsStr::new("protocol.file.allow=always"),
            OsStr::new("submodule"),
            OsStr::new("add"),
            child.repo.as_os_str(),
            OsStr::new("child"),
        ],
    );
    assert_error_code(
        tool()
            .args(["repo", "inspect", "--path"])
            .arg(parent.repo.join("child"))
            .output()
            .expect("tool must run"),
        "HCTL2_TOOL_SUBMODULE_UNSUPPORTED",
    );
    fs::remove_dir_all(root).expect("fixture cleanup");
}

#[test]
fn worktree_materialization_is_idempotent_and_verifies_dirtiness() {
    let fixture = Fixture::new("materialize");
    let worktrees = fixture.root.join("worktrees");
    let first = materialize(&fixture, &worktrees, "CS-123", &fixture.first_commit);
    assert_success(&first);
    let first_record = json_stdout(&first);
    assert_eq!(first_record["operation"], "created");
    assert!(
        first_record["site_lock"]["filesystem"]
            .as_str()
            .is_some_and(|filesystem| !filesystem.is_empty() && filesystem != "/")
    );
    let holder: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(fixture.repo.join(".git/hctl2/lock")).expect("holder information"),
    )
    .expect("holder information must be JSON");
    assert_eq!(holder["operation"], "worktree_materialize");
    assert_eq!(holder["change_set_ref"], "CS-123");
    let worktree = PathBuf::from(
        first_record["worktree"]["path"]
            .as_str()
            .expect("worktree path"),
    );
    assert!(worktree.is_dir());
    assert_eq!(
        git_stdout(
            Some(&worktree),
            [
                OsStr::new("log"),
                OsStr::new("-1"),
                OsStr::new("--format=%H")
            ]
        ),
        fixture.first_commit
    );

    let second = materialize(
        &fixture,
        &fixture.root.join("other-root"),
        "CS-123",
        &fixture.first_commit,
    );
    assert_success(&second);
    let second_record = json_stdout(&second);
    assert_eq!(second_record["operation"], "already_materialized");
    assert_eq!(
        second_record["worktree"]["path"],
        first_record["worktree"]["path"]
    );

    fs::write(worktree.join("README.md"), "changed\n").expect("tracked change");
    fs::write(worktree.join("new.txt"), "new\n").expect("untracked change");
    let verification = tool()
        .args(["worktree", "verify", "--repo"])
        .arg(&fixture.repo)
        .args(["--change-set-ref", "CS-123"])
        .output()
        .expect("tool must run");
    assert_success(&verification);
    let record = json_stdout(&verification);
    assert_eq!(record["verification"]["tracked_changes"], 1);
    assert_eq!(record["verification"]["untracked_changes"], 1);
    assert_eq!(record["verification"]["clean"], false);

    git(Some(&worktree), [OsStr::new("add"), OsStr::new(".")]);
    git(
        Some(&worktree),
        [
            OsStr::new("commit"),
            OsStr::new("-m"),
            OsStr::new("harness commit"),
        ],
    );
    let after = tool()
        .args(["worktree", "verify", "--repo"])
        .arg(&fixture.repo)
        .args(["--change-set-ref", "CS-123"])
        .output()
        .expect("tool must run");
    assert_success(&after);
    assert_eq!(json_stdout(&after)["verification"]["clean"], true);
}

#[test]
fn verification_reports_a_moved_changeset_branch_as_not_established() {
    let fixture = Fixture::new("verify-moved");
    let created = materialize(
        &fixture,
        &fixture.root.join("worktrees"),
        "CS-moved",
        &fixture.first_commit,
    );
    assert_success(&created);
    let worktree = PathBuf::from(
        json_stdout(&created)["worktree"]["path"]
            .as_str()
            .expect("worktree path"),
    );
    git(
        Some(&worktree),
        [
            OsStr::new("switch"),
            OsStr::new("-c"),
            OsStr::new("user-moved-branch"),
        ],
    );

    let output = tool()
        .args(["worktree", "verify", "--repo"])
        .arg(&fixture.repo)
        .args(["--change-set-ref", "CS-moved"])
        .output()
        .expect("tool must run");
    assert_eq!(output.status.code(), Some(3));
    let record = json_stdout(&output);
    assert_eq!(record["outcome"], "not_established");
    assert_eq!(record["verification"]["branch_matches"], false);
    assert!(record["verification"]["clean"].is_null());
}

#[test]
fn materialization_rejects_missing_baseline_changed_baseline_and_unwritable_root() {
    let fixture = Fixture::new("materialize-errors");
    let worktrees = fixture.root.join("worktrees");
    assert_error_code(
        materialize(
            &fixture,
            &worktrees,
            "CS-missing",
            "0000000000000000000000000000000000000000",
        ),
        "HCTL2_TOOL_BASELINE_NOT_FOUND",
    );

    let created = materialize(&fixture, &worktrees, "CS-baseline", &fixture.first_commit);
    assert_success(&created);
    let second_commit = fixture.second_commit();
    assert_error_code(
        materialize(&fixture, &worktrees, "CS-baseline", &second_commit),
        "HCTL2_TOOL_BASELINE_MISMATCH",
    );

    let readonly = fixture.root.join("readonly");
    fs::create_dir(&readonly).expect("readonly root");
    let mut permissions = fs::metadata(&readonly).expect("metadata").permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&readonly, permissions).expect("set readonly");
    let rejected = materialize(&fixture, &readonly, "CS-readonly", &fixture.first_commit);
    let mut permissions = fs::metadata(&readonly).expect("metadata").permissions();
    permissions.set_readonly(false);
    fs::set_permissions(&readonly, permissions).expect("restore permissions");
    assert_error_code(rejected, "HCTL2_TOOL_WORKTREE_ROOT_UNWRITABLE");

    assert_error_code(
        materialize(
            &fixture,
            &fixture.repo.join(".git/hctl2/checkouts"),
            "CS-internal",
            &fixture.first_commit,
        ),
        "HCTL2_TOOL_WORKTREE_ROOT_INTERNAL",
    );
}

#[test]
fn materialization_fails_immediately_when_another_process_holds_the_site_lock() {
    let fixture = Fixture::new("site-lock");
    let common = fixture.repo.join(".git");
    let lock_directory = common.join("hctl2");
    fs::create_dir(&lock_directory).expect("lock directory");
    let lock_path = lock_directory.join("lock");
    let ready_path = fixture.root.join("holder-ready");
    let mut holder = Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", "site_lock_holder", "--nocapture"])
        .env("HCTL2_TEST_LOCK_PATH", &lock_path)
        .env("HCTL2_TEST_READY_PATH", &ready_path)
        .stdout(Stdio::null())
        .spawn()
        .expect("lock holder must start");
    wait_for_path(&ready_path);

    let rejected = materialize(
        &fixture,
        &fixture.root.join("worktrees"),
        "CS-locked",
        &fixture.first_commit,
    );
    holder.kill().expect("holder must be killable");
    holder.wait().expect("holder must exit");
    assert_eq!(
        json_stdout(&rejected)["error"]["details"]["holder"]["operation"],
        "test-holder"
    );
    assert_error_code(rejected, "HCTL2_TOOL_SITE_BUSY");
}

#[test]
fn site_lock_holder() {
    let Some(path) = std::env::var_os("HCTL2_TEST_LOCK_PATH") else {
        return;
    };
    let ready = PathBuf::from(std::env::var_os("HCTL2_TEST_READY_PATH").expect("ready path"));
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(path)
        .expect("lock file");
    file.try_lock().expect("site lock");
    file.set_len(0).expect("truncate holder information");
    file.write_all(b"{\"operation\":\"test-holder\"}")
        .expect("holder information");
    file.sync_data().expect("sync holder information");
    fs::write(ready, b"ready").expect("ready marker");
    thread::sleep(Duration::from_secs(60));
}

#[test]
fn rejects_git_below_the_pinned_minimum() {
    let root = temporary_directory("old-git");
    let fake = root.join("git");
    fs::write(&fake, "#!/bin/sh\necho 'git version 2.38.0'\n").expect("fake git");
    make_executable(&fake);
    let output = tool()
        .args(["repo", "inspect", "--path", "."])
        .env("HCTL2_GIT", &fake)
        .output()
        .expect("tool must run");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .starts_with("error[HCTL2_TOOL_GIT_VERSION_UNSUPPORTED]:")
    );
    fs::remove_dir_all(root).expect("fixture cleanup");
}

#[test]
fn materialization_recovers_an_existing_branch_and_empty_directory() {
    let fixture = Fixture::new("existing-branch");
    git(
        Some(&fixture.repo),
        ["branch", "hctl2/changeset/CS-retry", &fixture.first_commit],
    );
    let root = fixture.root.join("worktrees");
    fs::create_dir_all(root.join("CS-retry")).expect("empty interrupted directory");
    let created = materialize(&fixture, &root, "CS-retry", &fixture.first_commit);
    assert_success(&created);
    let record = json_stdout(&created);
    assert_eq!(record["worktree"]["detached"], false);
    assert_eq!(record["verification"]["branch_matches"], true);
    let path = record["worktree"]["path"].as_str().expect("worktree path");
    assert_eq!(
        git_stdout(Some(Path::new(path)), ["symbolic-ref", "HEAD"]),
        "refs/heads/hctl2/changeset/CS-retry"
    );
    assert_success(&materialize(
        &fixture,
        &root,
        "CS-retry",
        &fixture.first_commit,
    ));
}

#[test]
fn missing_marker_is_rebuilt_after_commits_without_using_the_new_root() {
    let fixture = Fixture::new("missing-marker");
    let created = materialize(
        &fixture,
        &fixture.root.join("worktrees"),
        "CS-cache",
        &fixture.first_commit,
    );
    assert_success(&created);
    let record = json_stdout(&created);
    let path = Path::new(record["worktree"]["path"].as_str().expect("worktree path"));
    fs::write(path.join("README.md"), "committed progress\n").expect("progress");
    git(Some(path), ["add", "README.md"]);
    git(Some(path), ["commit", "-m", "progress"]);
    fs::write(path.join("only-copy.txt"), "uncommitted progress").expect("untracked progress");
    git(
        Some(&fixture.repo),
        [
            "update-ref",
            "-d",
            "refs/hctl2/changesets/CS-cache/baseline",
        ],
    );
    let next_root = fixture.root.join("must-not-be-created");
    let rebuilt = materialize(&fixture, &next_root, "CS-cache", &fixture.first_commit);
    assert_success(&rebuilt);
    let rebuilt = json_stdout(&rebuilt);
    assert_eq!(rebuilt["worktree"]["path"], record["worktree"]["path"]);
    assert_eq!(rebuilt["baseline_commit_sha"], fixture.first_commit);
    assert_eq!(rebuilt["verification"]["untracked_changes"], 1);
    assert!(!next_root.exists());

    // A non-ancestor supplied by the caller cannot rebuild the cache.
    git(
        Some(&fixture.repo),
        [
            "update-ref",
            "-d",
            "refs/hctl2/changesets/CS-cache/baseline",
        ],
    );
    let unrelated = fixture.second_commit();
    assert_error_code(
        materialize(&fixture, &next_root, "CS-cache", &unrelated),
        "HCTL2_TOOL_WORKTREE_VERIFICATION_FAILED",
    );
    assert!(
        !git_status(
            &fixture.repo,
            [
                "show-ref",
                "--verify",
                "--quiet",
                "refs/hctl2/changesets/CS-cache/baseline"
            ]
        )
        .success()
    );
    assert_eq!(
        fs::read_to_string(path.join("only-copy.txt")).expect("preserved"),
        "uncommitted progress"
    );
}

#[test]
fn occupied_paths_are_preserved_with_structured_recovery() {
    let fixture = Fixture::new("occupied");
    let root = fixture.root.join("worktrees");
    let target = root.join("CS-occupied");
    fs::create_dir_all(&target).expect("occupied target");
    fs::write(target.join("only-copy"), "precious").expect("only copy");
    let output = materialize(&fixture, &root, "CS-occupied", &fixture.first_commit);
    let record = json_stdout(&output);
    assert_eq!(
        record["error"]["details"]["occupied_path"].as_str(),
        target.canonicalize().expect("canonical target").to_str()
    );
    assert_eq!(
        record["error"]["details"]["recovery_action"],
        "preserve_occupied_path_then_retry"
    );
    assert_error_code(output, "HCTL2_TOOL_WORKTREE_PATH_OCCUPIED");
    assert_eq!(
        fs::read_to_string(target.join("only-copy")).expect("preserved"),
        "precious"
    );
}

#[test]
fn nested_roots_are_rejected_before_creating_directories() {
    let fixture = Fixture::new("nested");
    let created = materialize(
        &fixture,
        &fixture.root.join("worktrees"),
        "CS-first",
        &fixture.first_commit,
    );
    assert_success(&created);
    let record = json_stdout(&created);
    let other_worktree = Path::new(record["worktree"]["path"].as_str().expect("worktree path"));
    for containing in [&fixture.repo, other_worktree] {
        let nested = containing.join("new/nested");
        assert_error_code(
            materialize(&fixture, &nested, "CS-nested", &fixture.first_commit),
            "HCTL2_TOOL_WORKTREE_ROOT_NESTED",
        );
        assert!(!containing.join("new").exists());
    }
}

#[test]
fn inspection_accepts_an_unborn_head_and_labels_identity_provenance() {
    let fixture = Fixture::new("identity-source");
    let empty = fixture.root.join("empty");
    git(
        None,
        [
            OsStr::new("init"),
            OsStr::new("-b"),
            OsStr::new("main"),
            empty.as_os_str(),
        ],
    );
    let output = tool()
        .args(["repo", "inspect", "--path"])
        .arg(&empty)
        .output()
        .expect("inspect");
    assert_success(&output);
    let record = json_stdout(&output);
    assert!(record["repository_state"]["head_commit_sha"].is_null());
    assert_eq!(record["stable_repo_identity"]["status"], "missing");

    fs::create_dir(fixture.repo.join("subdir")).expect("subdir");
    let output = tool()
        .args(["repo", "inspect", "--path"])
        .arg(fixture.repo.join("subdir"))
        .output()
        .expect("inspect subdir");
    assert_success(&output);
    let record = json_stdout(&output);
    assert_eq!(
        record["stable_repo_identity"]["content"],
        "repo_id = \"repo-test\"\n"
    );
    assert_eq!(
        record["stable_repo_identity"]["source"]["commit_sha"],
        fixture.first_commit
    );

    let created = materialize(
        &fixture,
        &fixture.root.join("worktrees"),
        "CS-identity",
        &fixture.first_commit,
    );
    assert_success(&created);
    let record = json_stdout(&created);
    let path = Path::new(record["worktree"]["path"].as_str().expect("worktree path"));
    git(Some(path), ["rm", ".hctl2/repo.toml"]);
    git(Some(path), ["commit", "-m", "remove identity in changeset"]);
    let output = tool()
        .args(["repo", "inspect", "--path"])
        .arg(path)
        .output()
        .expect("inspect changeset");
    assert_success(&output);
    let observed = json_stdout(&output);
    assert_eq!(observed["stable_repo_identity"]["status"], "missing");
    assert_eq!(
        observed["stable_repo_identity"]["source"]["commit_sha"],
        git_stdout(Some(path), ["rev-parse", "HEAD"])
    );
    assert_eq!(
        observed["stable_repo_identity"]["source"]["worktree_root"].as_str(),
        path.to_str()
    );
}

#[cfg(unix)]
#[test]
fn config_file_overrides_and_checkout_hooks_are_preserved() {
    let fixture = Fixture::new("config-hooks");
    let hooks = fixture.root.join("hooks");
    fs::create_dir(&hooks).expect("hooks");
    let hook = hooks.join("post-checkout");
    fs::write(&hook, "#!/bin/sh\nprintf 'called' > hook-ran\n").expect("hook");
    make_executable(&hook);
    let config = fixture.root.join("global-config");
    fs::write(
        &config,
        format!("[core]\n hooksPath = {}\n", hooks.display()),
    )
    .expect("config");
    let output = tool()
        .args(["worktree", "materialize", "--repo"])
        .arg(&fixture.repo)
        .arg("--root")
        .arg(fixture.root.join("worktrees"))
        .args([
            "--change-set-ref",
            "CS-config",
            "--baseline",
            &fixture.first_commit,
        ])
        .env("GIT_CONFIG_GLOBAL", &config)
        .env("GIT_CONFIG_SYSTEM", &config)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "core.hooksPath")
        .env("GIT_CONFIG_VALUE_0", fixture.root.join("wrong-hooks"))
        .output()
        .expect("materialize");
    assert_success(&output);
    let record = json_stdout(&output);
    let path = Path::new(record["worktree"]["path"].as_str().expect("worktree path"));
    assert_eq!(
        fs::read_to_string(path.join("hook-ran")).expect("hook ran"),
        "called"
    );
}

#[cfg(unix)]
#[test]
fn failed_checkout_verification_does_not_create_a_baseline_marker() {
    let fixture = Fixture::new("hook-branch");
    let hooks = fixture.root.join("hooks");
    fs::create_dir(&hooks).expect("hooks");
    let hook = hooks.join("post-checkout");
    fs::write(&hook, "#!/bin/sh\ngit symbolic-ref HEAD refs/heads/main\n").expect("hook");
    make_executable(&hook);
    git(
        Some(&fixture.repo),
        [
            OsStr::new("config"),
            OsStr::new("core.hooksPath"),
            hooks.as_os_str(),
        ],
    );
    let output = materialize(
        &fixture,
        &fixture.root.join("worktrees"),
        "CS-hook",
        &fixture.first_commit,
    );
    assert_error_code(output, "HCTL2_TOOL_WORKTREE_VERIFICATION_FAILED");
    assert!(
        !git_status(
            &fixture.repo,
            [
                "show-ref",
                "--verify",
                "--quiet",
                "refs/hctl2/changesets/CS-hook/baseline"
            ]
        )
        .success()
    );
}

#[test]
fn verification_rejects_a_foreign_repository_at_the_registered_path() {
    let fixture = Fixture::new("replaced-checkout");
    let created = materialize(
        &fixture,
        &fixture.root.join("worktrees"),
        "CS-replaced",
        &fixture.first_commit,
    );
    assert_success(&created);
    let record = json_stdout(&created);
    let path = Path::new(record["worktree"]["path"].as_str().expect("worktree path"));
    fs::rename(path, fixture.root.join("preserved-checkout")).expect("preserve original");
    git(
        None,
        [
            OsStr::new("clone"),
            fixture.repo.as_os_str(),
            path.as_os_str(),
        ],
    );
    git(
        Some(path),
        [
            "switch",
            "-c",
            "hctl2/changeset/CS-replaced",
            &fixture.first_commit,
        ],
    );
    let output = tool()
        .args(["worktree", "verify", "--repo"])
        .arg(&fixture.repo)
        .args(["--change-set-ref", "CS-replaced"])
        .output()
        .expect("verify");
    assert_eq!(output.status.code(), Some(3));
    let record = json_stdout(&output);
    assert_eq!(record["verification"]["repository_matches"], false);
    assert!(record["verification"]["clean"].is_null());
}

#[cfg(unix)]
#[test]
fn symlinks_cannot_redirect_checkouts_or_lock_file_writes() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new("symlinks");
    let alias = fixture.root.join("alias");
    symlink(&fixture.repo, &alias).expect("root alias");
    assert_error_code(
        materialize(
            &fixture,
            &alias.join("nested"),
            "CS-alias",
            &fixture.first_commit,
        ),
        "HCTL2_TOOL_WORKTREE_ROOT_NESTED",
    );
    assert!(!fixture.repo.join("nested").exists());
    let root = fixture.root.join("worktrees");
    fs::create_dir(&root).expect("root");
    symlink(fixture.root.join("missing"), root.join("CS-dangling")).expect("dangling target");
    assert_error_code(
        materialize(&fixture, &root, "CS-dangling", &fixture.first_commit),
        "HCTL2_TOOL_WORKTREE_PATH_OCCUPIED",
    );
    let lock = fixture.repo.join(".git/hctl2/lock");
    fs::remove_file(&lock).expect("remove unlocked test lock");
    let data = fixture.root.join("only-copy");
    fs::write(&data, "preserve").expect("data");
    symlink(&data, &lock).expect("lock symlink");
    assert_error_code(
        materialize(&fixture, &root, "CS-lock", &fixture.first_commit),
        "HCTL2_TOOL_SITE_LOCK_UNAVAILABLE",
    );
    assert_eq!(fs::read_to_string(data).expect("preserved"), "preserve");
}

fn git_status<const N: usize>(repository: &Path, arguments: [&str; N]) -> std::process::ExitStatus {
    Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(arguments)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("git status")
}

fn materialize(fixture: &Fixture, root: &Path, change_set_ref: &str, baseline: &str) -> Output {
    tool()
        .args(["worktree", "materialize", "--repo"])
        .arg(&fixture.repo)
        .arg("--root")
        .arg(root)
        .args(["--change-set-ref", change_set_ref, "--baseline", baseline])
        .output()
        .expect("tool must run")
}

fn tool() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hctl2-tool"))
}

fn git<I, S>(repository: Option<&Path>, arguments: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new("git");
    if let Some(repository) = repository {
        command.arg("-C").arg(repository);
    }
    let output = command
        .args(arguments)
        .env("LC_ALL", "C")
        .output()
        .expect("git must run");
    assert!(
        output.status.success(),
        "git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn git_stdout<I, S>(repository: Option<&Path>, arguments: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    String::from_utf8(git(repository, arguments).stdout)
        .expect("git output must be UTF-8")
        .trim()
        .to_owned()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "tool failed: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_error_code(output: Output, code: &str) {
    let record = json_stdout(&output);
    assert!(
        output.stderr.is_empty(),
        "observation error leaked to stderr"
    );
    assert_eq!(record["schema"], "hctl2.tool-error.v1");
    assert_eq!(record["evidence_level"], "toolbox_readback");
    assert_eq!(record["error"]["code"], code, "{record}");
    let expected_exit = match record["outcome"].as_str() {
        Some("not_established") => 3,
        Some("unreadable") => 4,
        outcome => panic!("unexpected failure outcome: {outcome:?}"),
    };
    assert_eq!(output.status.code(), Some(expected_exit));
}

fn json_stdout(output: &Output) -> serde_json::Value {
    let stdout = String::from_utf8(output.stdout.clone()).expect("record must be UTF-8");
    assert_eq!(stdout.lines().count(), 1, "one JSON record per invocation");
    serde_json::from_str(stdout.trim()).expect("record must be JSON")
}

fn temporary_directory(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock must follow epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "hctl2-tool-{name}-{}-{nanos}-{}",
        std::process::id(),
        TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).expect("temporary directory");
    path
}

fn wait_for_path(path: &Path) {
    for _ in 0..100 {
        if path.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("timed out waiting for {}", path.display());
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("chmod");
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {}
