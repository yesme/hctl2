use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);
const TARGET: &str = "refs/heads/main";

struct Fixture {
    root: PathBuf,
    repo: PathBuf,
    base: String,
    candidate: String,
    tree: String,
}

impl Fixture {
    fn new(name: &str) -> Self {
        Self::with_format(name, "sha1")
    }

    fn with_format(name: &str, format: &str) -> Self {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "hctl2-integration-{name}-{}-{time}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let repo = root.join("repo");
        fs::create_dir(&repo).unwrap();
        git(
            &repo,
            &["init", "-b", "main", &format!("--object-format={format}")],
        );
        git(&repo, &["config", "user.name", "HCTL2 Test"]);
        git(&repo, &["config", "user.email", "hctl2@example.invalid"]);
        git(&repo, &["config", "commit.gpgsign", "false"]);
        fs::write(repo.join("base.txt"), "base\n").unwrap();
        let base = commit(&repo, "base");
        git(&repo, &["switch", "-c", "candidate"]);
        fs::write(repo.join("candidate.txt"), "candidate\n").unwrap();
        let candidate = commit(&repo, "candidate");
        let tree = git(&repo, &["rev-parse", "HEAD^{tree}"]);
        git(&repo, &["switch", "--detach", "main"]);
        Self {
            root,
            repo,
            base,
            candidate,
            tree,
        }
    }

    fn tool(&self, strategy: &str, expected: &str, key: &str) -> Command {
        self.command(
            strategy,
            expected,
            key,
            &self.base,
            &self.candidate,
            &self.tree,
            TARGET,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn command(
        &self,
        strategy: &str,
        expected: &str,
        key: &str,
        base: &str,
        candidate: &str,
        tree: &str,
        target: &str,
    ) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_hctl2-tool"));
        isolate_git(&mut command);
        command.args(["integrate", "--repo"]).arg(&self.repo).args([
            "--commit",
            candidate,
            "--base-commit-sha",
            base,
            "--result-tree-sha",
            tree,
            "--target-ref",
            target,
            "--expected-head",
            expected,
            "--strategy",
            strategy,
            "--idempotency-key",
            key,
        ]);
        command
    }

    fn advance_target(&self) -> String {
        git(&self.repo, &["switch", "main"]);
        fs::write(self.repo.join("target.txt"), "target\n").unwrap();
        let head = commit(&self.repo, "target");
        git(&self.repo, &["switch", "--detach"]);
        head
    }

    fn refs(&self) -> String {
        git(
            &self.repo,
            &["for-each-ref", "--format=%(refname) %(objectname)"],
        )
    }

    fn head(&self) -> String {
        git(&self.repo, &["rev-parse", TARGET])
    }

    // Fault injection delegates all real repository operations to the same host Git.
    // Only the precise target update/read is paused or fails; no product-only test flags.
    #[cfg(unix)]
    fn wrapper(&self, mode: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let path = self.root.join(format!("git-{mode}.sh"));
        let source = r#"#!/bin/sh
if [ "$HCTL2_TEST_MODE" = checkout-after-prepare ] && [ "$3" = update-ref ] && [ "$7" = --create-reflog ]; then
    "$HCTL2_TEST_GIT" "$@" || exit $?
    "$HCTL2_TEST_GIT" -C "$2" worktree add "$HCTL2_TEST_WORKTREE" main || exit $?
    exit 0
fi
if [ "$3" = update-ref ] && [ "$7" = refs/heads/main ]; then
    case "$HCTL2_TEST_MODE" in
        before)
            : > "$HCTL2_TEST_READY"
            while [ ! -f "$HCTL2_TEST_RELEASE" ]; do sleep 0.01; done
            ;;
        after)
            "$HCTL2_TEST_GIT" "$@" || exit $?
            : > "$HCTL2_TEST_READY"
            while [ ! -f "$HCTL2_TEST_RELEASE" ]; do sleep 0.01; done
            exit 0
            ;;
        read-failure)
            "$HCTL2_TEST_GIT" "$@" || exit $?
            : > "$HCTL2_TEST_READY"
            exit 0
            ;;
        advanced-after|failed-and-advanced-after|ancestry-failure)
            "$HCTL2_TEST_GIT" "$@" || exit $?
            "$HCTL2_TEST_GIT" -C "$2" update-ref refs/heads/main "$HCTL2_TEST_OTHER" "$8" || exit $?
            if [ "$HCTL2_TEST_MODE" = failed-and-advanced-after ]; then exit 73; fi
            exit 0
            ;;
        rejected)
            echo 'injected target update rejection' >&2
            exit 73
            ;;
        failed-after)
            "$HCTL2_TEST_GIT" "$@" || exit $?
            exit 73
            ;;
    esac
fi
if [ "$HCTL2_TEST_MODE" = ancestry-failure ] && [ "$3" = merge-base ] && [ "$5" = "$HCTL2_TEST_RESULT" ] && [ "$6" = "$HCTL2_TEST_OTHER" ]; then
    echo 'injected unreadable ancestry' >&2
    exit 73
fi
if [ "$HCTL2_TEST_MODE" = read-failure ] && [ "$3" = for-each-ref ] && [ "$5" = refs/heads/main ] && [ -f "$HCTL2_TEST_READY" ]; then
    echo 'injected unreadable target' >&2
    exit 73
fi
exec "$HCTL2_TEST_GIT" "$@"
"#;
        fs::write(&path, source).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    #[cfg(unix)]
    fn inject(&self, command: &mut Command, mode: &str) {
        let real_git = Command::new("/bin/sh")
            .args(["-c", "command -v git"])
            .output()
            .unwrap();
        assert!(real_git.status.success());
        command
            .env("HCTL2_GIT", self.wrapper(mode))
            .env(
                "HCTL2_TEST_GIT",
                String::from_utf8(real_git.stdout).unwrap().trim(),
            )
            .env("HCTL2_TEST_MODE", mode)
            .env("HCTL2_TEST_READY", self.root.join("ready"))
            .env("HCTL2_TEST_RELEASE", self.root.join("release"));
    }

    #[cfg(unix)]
    fn paused(&self, command: &mut Command, mode: &str) -> Paused {
        self.inject(command, mode);
        let child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut paused = Paused {
            child: Some(child),
            release: self.root.join("release"),
        };
        let deadline = Instant::now() + Duration::from_secs(30);
        while !self.root.join("ready").exists() {
            if let Some(status) = paused.child.as_mut().unwrap().try_wait().unwrap() {
                let output = paused.child.take().unwrap().wait_with_output().unwrap();
                panic!(
                    "tool exited before barrier ({status}): {} {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            assert!(
                Instant::now() < deadline,
                "tool did not reach target CAS barrier"
            );
            thread::sleep(Duration::from_millis(10));
        }
        paused
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct Paused {
    child: Option<Child>,
    release: PathBuf,
}

impl Paused {
    fn finish(mut self) -> Output {
        fs::write(&self.release, "release").unwrap();
        self.child.take().unwrap().wait_with_output().unwrap()
    }

    fn kill_parent(&mut self) {
        let child = self.child.as_mut().unwrap();
        child.kill().unwrap();
        assert!(!child.wait().unwrap().success());
    }
}

impl Drop for Paused {
    fn drop(&mut self) {
        let _ = fs::write(&self.release, "release");
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn isolate_git(command: &mut Command) {
    for key in [
        "HCTL2_GIT",
        "GIT_DIR",
        "GIT_COMMON_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_NAMESPACE",
        "GIT_CONFIG_COUNT",
        "GIT_CONFIG_PARAMETERS",
        "GIT_AUTHOR_DATE",
        "GIT_COMMITTER_DATE",
    ] {
        command.env_remove(key);
    }
    command
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("LC_ALL", "C");
}

fn git(repo: &Path, arguments: &[&str]) -> String {
    let mut command = Command::new("git");
    isolate_git(&mut command);
    let output = command
        .arg("-C")
        .arg(repo)
        .args(arguments)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {arguments:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .trim_end_matches(['\r', '\n'])
        .to_owned()
}

fn commit(repo: &Path, message: &str) -> String {
    git(repo, &["add", "-A"]);
    git(repo, &["commit", "-m", message]);
    git(repo, &["rev-parse", "HEAD"])
}

fn record(output: Output, exit: i32) -> Value {
    assert_eq!(
        output.status.code(),
        Some(exit),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty(), "observation must use stdout JSON");
    let value: Value = serde_json::from_slice(&output.stdout).expect("one JSON record");
    assert_eq!(value["evidence_level"], "toolbox_readback");
    value
}

fn rejected_output(command: &mut Command, code: &str, exit: i32) -> Value {
    let value = record(command.output().unwrap(), exit);
    assert_eq!(value["error"]["code"], code, "{value}");
    assert!(
        value["error"]["details"]["recovery_action"].is_string(),
        "{value}"
    );
    assert_eq!(
        value["outcome"],
        if exit == 3 {
            "not_established"
        } else {
            "unreadable"
        }
    );
    value
}

#[test]
fn checked_out_target_override_warns_and_preserves_dirty_index_and_files() {
    let fixture = Fixture::new("ff");
    git(&fixture.repo, &["switch", "main"]);
    fs::write(fixture.repo.join("base.txt"), "staged user edit\n").unwrap();
    git(&fixture.repo, &["add", "base.txt"]);
    fs::write(fixture.repo.join("base.txt"), "unstaged user edit\n").unwrap();
    fs::write(fixture.repo.join("untracked"), "unique\n").unwrap();
    let index = fs::read(fixture.repo.join(".git/index")).unwrap();
    let first = record(
        fixture
            .tool("fast-forward", &fixture.base, "ff-1")
            .arg("--allow-checked-out-target")
            .output()
            .unwrap(),
        0,
    );
    assert_eq!(first["schema"], "hctl2.integration.v1");
    assert_eq!(first["status"], "applied");
    assert_eq!(first["before_head"], fixture.base);
    assert_eq!(first["after_head"], fixture.candidate);
    assert_eq!(first["integrated_tree_sha"], fixture.tree);
    assert_eq!(
        first["checked_out_worktrees"],
        json!([fixture.repo.canonicalize().unwrap()])
    );
    assert_eq!(
        first["warnings"][0]["code"],
        "HCTL2_TOOL_INTEGRATION_TARGET_CHECKED_OUT"
    );
    assert_eq!(fixture.head(), fixture.candidate);
    assert_eq!(index, fs::read(fixture.repo.join(".git/index")).unwrap());
    assert_eq!(
        fs::read_to_string(fixture.repo.join("base.txt")).unwrap(),
        "unstaged user edit\n"
    );
    assert_eq!(
        fs::read_to_string(fixture.repo.join("untracked")).unwrap(),
        "unique\n"
    );
    assert!(!fixture.repo.join("candidate.txt").exists());
    let refs = fixture.refs();
    let second = record(
        fixture
            .tool("fast-forward", &fixture.base, "ff-1")
            .arg("--allow-checked-out-target")
            .output()
            .unwrap(),
        0,
    );
    assert_eq!(second["status"], "already_applied");
    assert_eq!(second["new_head"], first["new_head"]);
    assert_eq!(fixture.refs(), refs);
}

#[test]
fn merge_commit_uses_native_merge_tree_and_keeps_same_commit_on_retry() {
    let fixture = Fixture::new("merge");
    let expected = fixture.advance_target();
    let native_tree = git(
        &fixture.repo,
        &["merge-tree", "--write-tree", &expected, &fixture.candidate],
    );
    let index = fs::read(fixture.repo.join(".git/index")).unwrap();
    let first = record(
        fixture
            .tool("merge-commit", &expected, "merge-1")
            .output()
            .unwrap(),
        0,
    );
    let new = first["new_head"].as_str().unwrap();
    assert_eq!(first["integrated_tree_sha"], native_tree);
    assert_ne!(first["integrated_tree_sha"], fixture.tree);
    assert_eq!(
        git(&fixture.repo, &["show", "-s", "--format=%P", new]),
        format!("{expected} {}", fixture.candidate)
    );
    assert_eq!(index, fs::read(fixture.repo.join(".git/index")).unwrap());
    assert!(!fixture.repo.join("candidate.txt").exists());
    assert_eq!(
        fs::read_to_string(fixture.repo.join("target.txt")).unwrap(),
        "target\n"
    );
    git(&fixture.repo, &["config", "user.name", "Changed identity"]);
    let second = record(
        fixture
            .tool("merge-commit", &expected, "merge-1")
            .env("GIT_COMMITTER_DATE", "2020-01-01T00:00:00Z")
            .output()
            .unwrap(),
        0,
    );
    assert_eq!(second["status"], "already_applied");
    assert_eq!(second["new_head"], new);
}

#[test]
fn preconditions_are_ordered_and_leave_every_ref_unchanged() {
    let fixture = Fixture::new("order");
    fixture.advance_target();
    let before = fixture.refs();
    let base_tree = git(
        &fixture.repo,
        &["rev-parse", &format!("{}^{{tree}}", fixture.base)],
    );
    rejected_output(
        &mut fixture.command(
            "merge-commit",
            &fixture.base,
            "order",
            &fixture.candidate,
            &fixture.candidate,
            &base_tree,
            TARGET,
        ),
        "HCTL2_TOOL_INTEGRATION_TREE_MISMATCH",
        3,
    );
    assert_eq!(fixture.refs(), before);
    rejected_output(
        &mut fixture.command(
            "merge-commit",
            &fixture.base,
            "order",
            &fixture.candidate,
            &fixture.candidate,
            &fixture.tree,
            TARGET,
        ),
        "HCTL2_TOOL_INTEGRATION_BASELINE_NOT_ANCESTOR",
        3,
    );
    assert_eq!(fixture.refs(), before);
    rejected_output(
        &mut fixture.tool("merge-commit", &fixture.base, "order"),
        "HCTL2_TOOL_INTEGRATION_HEAD_DRIFT",
        3,
    );
    assert_eq!(fixture.refs(), before);
}

#[test]
fn fast_forward_rejects_advanced_base_and_non_descendant_candidate() {
    let fixture = Fixture::new("non-ff");
    let advanced = fixture.advance_target();
    let before = fixture.refs();
    rejected_output(
        &mut fixture.tool("fast-forward", &advanced, "advanced"),
        "HCTL2_TOOL_INTEGRATION_NOT_FAST_FORWARD",
        3,
    );
    rejected_output(
        &mut fixture.command(
            "fast-forward",
            &advanced,
            "non-descendant",
            &advanced,
            &fixture.candidate,
            &fixture.tree,
            TARGET,
        ),
        "HCTL2_TOOL_INTEGRATION_NOT_FAST_FORWARD",
        3,
    );
    assert_eq!(fixture.refs(), before);
}

#[test]
fn conflicts_return_exact_paths_and_never_publish_the_conflicted_tree() {
    let fixture = Fixture::new("conflict");
    git(&fixture.repo, &["switch", "main"]);
    let name = "conflict\nwith\ttab.txt";
    fs::write(fixture.repo.join(name), "target\n").unwrap();
    let expected = commit(&fixture.repo, "target conflict");
    git(&fixture.repo, &["switch", "candidate"]);
    fs::write(fixture.repo.join(name), "candidate\n").unwrap();
    let candidate = commit(&fixture.repo, "candidate conflict");
    let tree = git(&fixture.repo, &["rev-parse", "HEAD^{tree}"]);
    let before = fixture.refs();
    let value = rejected_output(
        &mut fixture.command(
            "merge-commit",
            &expected,
            "conflict",
            &fixture.base,
            &candidate,
            &tree,
            TARGET,
        ),
        "HCTL2_TOOL_INTEGRATION_CONFLICT",
        3,
    );
    assert_eq!(
        value["error"]["details"]["conflict_paths"],
        json!([{"path": name, "path_bytes": name.as_bytes()}])
    );
    assert_eq!(fixture.refs(), before);
    assert_eq!(
        fs::read_to_string(fixture.repo.join(name)).unwrap(),
        "candidate\n"
    );
}

#[test]
fn missing_symbolic_and_nonlocal_targets_are_typed_rejections() {
    let fixture = Fixture::new("targets");
    let before = fixture.refs();
    rejected_output(
        &mut fixture.command(
            "fast-forward",
            &fixture.base,
            "missing",
            &fixture.base,
            &fixture.candidate,
            &fixture.tree,
            "refs/heads/absent",
        ),
        "HCTL2_TOOL_INTEGRATION_TARGET_MISSING",
        3,
    );
    for target in [
        "HEAD",
        "main",
        "refs/remotes/origin/main",
        "refs/hctl2/other",
        "refs/heads/../bad",
    ] {
        rejected_output(
            &mut fixture.command(
                "fast-forward",
                &fixture.base,
                "bad",
                &fixture.base,
                &fixture.candidate,
                &fixture.tree,
                target,
            ),
            "HCTL2_TOOL_INVALID_ARGUMENT",
            3,
        );
    }
    assert_eq!(fixture.refs(), before);
    git(&fixture.repo, &["symbolic-ref", "refs/heads/alias", TARGET]);
    let before = fixture.refs();
    rejected_output(
        &mut fixture.command(
            "fast-forward",
            &fixture.base,
            "symbolic",
            &fixture.base,
            &fixture.candidate,
            &fixture.tree,
            "refs/heads/alias",
        ),
        "HCTL2_TOOL_INTEGRATION_SYMBOLIC_REF",
        3,
    );
    assert_eq!(fixture.refs(), before);
}

#[test]
fn moving_inputs_missing_objects_and_empty_keys_are_rejected_before_writes() {
    let fixture = Fixture::new("inputs");
    let before = fixture.refs();
    rejected_output(
        &mut fixture.command(
            "fast-forward",
            &fixture.base,
            "moving",
            &fixture.base,
            "candidate",
            &fixture.tree,
            TARGET,
        ),
        "HCTL2_TOOL_INVALID_OBJECT_ID",
        3,
    );
    rejected_output(
        &mut fixture.command(
            "fast-forward",
            &fixture.base,
            "missing",
            &fixture.base,
            &"f".repeat(40),
            &fixture.tree,
            TARGET,
        ),
        "HCTL2_TOOL_COMMIT_NOT_FOUND",
        3,
    );
    rejected_output(
        &mut fixture.tool("fast-forward", &fixture.base, ""),
        "HCTL2_TOOL_INVALID_ARGUMENT",
        3,
    );
    assert_eq!(fixture.refs(), before);
}

#[test]
fn annotated_tag_ids_are_not_silently_peeled_into_frozen_commit_inputs() {
    let fixture = Fixture::new("tag-input");
    git(
        &fixture.repo,
        &[
            "tag",
            "-a",
            "candidate-tag",
            &fixture.candidate,
            "-m",
            "tag",
        ],
    );
    let tag = git(&fixture.repo, &["rev-parse", "candidate-tag"]);
    let refs = fixture.refs();
    rejected_output(
        &mut fixture.command(
            "merge-commit",
            &fixture.base,
            "tag",
            &fixture.base,
            &tag,
            &fixture.tree,
            TARGET,
        ),
        "HCTL2_TOOL_INVALID_COMMIT_ID",
        3,
    );
    assert_eq!(fixture.refs(), refs);
}

#[test]
fn missing_git_identity_rejects_merge_without_changing_refs() {
    let fixture = Fixture::new("identity");
    git(&fixture.repo, &["config", "--unset", "user.name"]);
    git(&fixture.repo, &["config", "--unset", "user.email"]);
    git(&fixture.repo, &["config", "user.useConfigOnly", "true"]);
    let refs = fixture.refs();
    let mut command = fixture.tool("merge-commit", &fixture.base, "identity");
    for name in [
        "GIT_AUTHOR_NAME",
        "GIT_AUTHOR_EMAIL",
        "GIT_COMMITTER_NAME",
        "GIT_COMMITTER_EMAIL",
        "EMAIL",
    ] {
        command.env_remove(name);
    }
    rejected_output(&mut command, "HCTL2_TOOL_INTEGRATION_COMMIT_FAILED", 4);
    assert_eq!(fixture.refs(), refs);
}

#[test]
fn candidate_equal_to_expected_head_is_a_repeatable_noop_for_both_strategies() {
    let fixture = Fixture::new("noop");
    git(&fixture.repo, &["switch", "main"]);
    let tree = git(
        &fixture.repo,
        &["rev-parse", &format!("{}^{{tree}}", fixture.base)],
    );
    let objects = git(&fixture.repo, &["count-objects", "-v"]);
    let reflog = git(&fixture.repo, &["reflog", "show", TARGET]);
    for strategy in ["fast-forward", "merge-commit"] {
        for _ in 0..2 {
            let value = record(
                fixture
                    .command(
                        strategy,
                        &fixture.base,
                        strategy,
                        &fixture.base,
                        &fixture.base,
                        &tree,
                        TARGET,
                    )
                    .output()
                    .unwrap(),
                0,
            );
            assert_eq!(value["status"], "already_applied");
            assert_eq!(fixture.head(), fixture.base);
            assert_eq!(value["new_head"], fixture.base);
            assert_eq!(value["after_head"], fixture.base);
            assert_eq!(git(&fixture.repo, &["count-objects", "-v"]), objects);
            assert_eq!(git(&fixture.repo, &["reflog", "show", TARGET]), reflog);
        }
    }
}

#[test]
fn key_reuse_with_changed_fields_cannot_reapply_or_retarget() {
    let fixture = Fixture::new("key");
    record(
        fixture
            .tool("fast-forward", &fixture.base, "bound-key")
            .output()
            .unwrap(),
        0,
    );
    let refs = fixture.refs();
    rejected_output(
        &mut fixture.tool("merge-commit", &fixture.base, "bound-key"),
        "HCTL2_TOOL_INTEGRATION_KEY_REUSED",
        3,
    );
    rejected_output(
        fixture
            .tool("fast-forward", &fixture.base, "bound-key")
            .arg("--allow-checked-out-target"),
        "HCTL2_TOOL_INTEGRATION_KEY_REUSED",
        3,
    );
    rejected_output(
        &mut fixture.command(
            "fast-forward",
            &fixture.base,
            "bound-key",
            &fixture.base,
            &fixture.candidate,
            &fixture.tree,
            "refs/heads/elsewhere",
        ),
        "HCTL2_TOOL_INTEGRATION_KEY_REUSED",
        3,
    );
    assert_eq!(fixture.refs(), refs);
}

#[test]
fn unknown_retry_never_rewinds_a_diverged_or_deleted_target() {
    let fixture = Fixture::new("unknown-retry");
    record(
        fixture
            .tool("fast-forward", &fixture.base, "unknown")
            .output()
            .unwrap(),
        0,
    );
    let advanced = git(
        &fixture.repo,
        &[
            "commit-tree",
            &fixture.tree,
            "-p",
            &fixture.base,
            "-m",
            "external",
        ],
    );
    git(
        &fixture.repo,
        &["update-ref", TARGET, &advanced, &fixture.candidate],
    );
    rejected_output(
        &mut fixture.tool("fast-forward", &fixture.base, "unknown"),
        "HCTL2_TOOL_INTEGRATION_RESULT_UNKNOWN",
        4,
    );
    assert_eq!(fixture.head(), advanced);
    git(&fixture.repo, &["update-ref", "-d", TARGET, &advanced]);
    let refs = fixture.refs();
    rejected_output(
        &mut fixture.tool("fast-forward", &fixture.base, "unknown"),
        "HCTL2_TOOL_INTEGRATION_RESULT_UNKNOWN",
        4,
    );
    assert_eq!(fixture.refs(), refs);
}

#[test]
fn sha256_repository_and_linked_worktree_share_retry_identity() {
    let fixture = Fixture::with_format("sha256", "sha256");
    assert_eq!(fixture.candidate.len(), 64);
    let first = record(
        fixture
            .tool("fast-forward", &fixture.base, "sha256")
            .output()
            .unwrap(),
        0,
    );
    let other = fixture.root.join("linked");
    git(
        &fixture.repo,
        &[
            "worktree",
            "add",
            "--detach",
            other.to_str().unwrap(),
            &fixture.base,
        ],
    );
    // Use the same CLI inputs but another path into the common Git directory.
    let mut command = fixture.tool("fast-forward", &fixture.base, "sha256");
    let arguments = command.get_args().map(OsStr::to_owned).collect::<Vec<_>>();
    let mut rewritten = arguments;
    rewritten[2] = other.into_os_string();
    command = Command::new(env!("CARGO_BIN_EXE_hctl2-tool"));
    isolate_git(&mut command);
    let second = record(command.args(rewritten).output().unwrap(), 0);
    assert_eq!(second["status"], "already_applied");
    assert_eq!(second["prepared_ref"], first["prepared_ref"]);
}

#[test]
#[cfg(unix)]
fn competing_tool_is_busy_and_external_git_wins_real_target_cas() {
    let fixture = Fixture::new("cas");
    let external = git(
        &fixture.repo,
        &[
            "commit-tree",
            &fixture.tree,
            "-p",
            &fixture.base,
            "-m",
            "external winner",
        ],
    );
    let paused = fixture.paused(
        &mut fixture.tool("fast-forward", &fixture.base, "first"),
        "before",
    );
    let busy = rejected_output(
        &mut fixture.tool("fast-forward", &fixture.base, "second"),
        "HCTL2_TOOL_SITE_BUSY",
        3,
    );
    assert_eq!(busy["error"]["details"]["holder"]["operation"], "integrate");
    let prepared_ref = git(
        &fixture.repo,
        &[
            "for-each-ref",
            "--format=%(refname)",
            "refs/hctl2/integrations/",
        ],
    );
    assert_eq!(
        busy["error"]["details"]["holder"]["intent_digest"],
        prepared_ref.rsplit('/').next().unwrap()
    );
    assert!(
        busy["error"]["details"]["holder"]
            .get("change_set_ref")
            .is_none()
    );
    git(
        &fixture.repo,
        &["update-ref", TARGET, &external, &fixture.base],
    );
    let value = record(paused.finish(), 4);
    assert_eq!(
        value["error"]["code"],
        "HCTL2_TOOL_INTEGRATION_RESULT_UNKNOWN"
    );
    assert_eq!(value["error"]["details"]["observed_head"], external);
    assert_eq!(fixture.head(), external);
}

#[test]
#[cfg(unix)]
fn killed_after_target_cas_recovers_original_merge_commit_by_readback() {
    let fixture = Fixture::new("killed-after");
    let expected = fixture.advance_target();
    let mut paused = fixture.paused(
        &mut fixture.tool("merge-commit", &expected, "killed"),
        "after",
    );
    let applied = fixture.head();
    assert_ne!(applied, expected);
    paused.kill_parent();
    // Retry while the old Git wrapper is still paused: the dead parent no longer holds the OS lock.
    let value = record(
        fixture
            .tool("merge-commit", &expected, "killed")
            .env("GIT_COMMITTER_DATE", "2020-01-01T00:00:00Z")
            .output()
            .unwrap(),
        0,
    );
    assert_eq!(value["status"], "already_applied");
    assert_eq!(value["new_head"], applied);
    paused.finish();
}

#[test]
#[cfg(unix)]
fn killed_before_target_cas_reuses_prepared_commit_and_old_child_cannot_overwrite() {
    let fixture = Fixture::new("killed-before");
    let expected = fixture.advance_target();
    let mut paused = fixture.paused(
        &mut fixture.tool("merge-commit", &expected, "prepared"),
        "before",
    );
    let prepared = git(
        &fixture.repo,
        &[
            "for-each-ref",
            "--format=%(objectname)",
            "refs/hctl2/integrations/",
        ],
    );
    assert_eq!(fixture.head(), expected);
    paused.kill_parent();
    let value = record(
        fixture
            .tool("merge-commit", &expected, "prepared")
            .env("GIT_COMMITTER_DATE", "2020-01-01T00:00:00Z")
            .output()
            .unwrap(),
        0,
    );
    assert_eq!(value["new_head"], prepared);
    paused.finish();
    assert_eq!(fixture.head(), prepared);
}

#[test]
#[cfg(unix)]
fn unreadable_post_cas_target_returns_unknown_then_retry_reports_applied() {
    let fixture = Fixture::new("unreadable");
    let mut command = fixture.tool("fast-forward", &fixture.base, "unreadable");
    fixture.inject(&mut command, "read-failure");
    let value = rejected_output(&mut command, "HCTL2_TOOL_INTEGRATION_RESULT_UNKNOWN", 4);
    assert_eq!(value["error"]["details"]["new_head"], fixture.candidate);
    assert_eq!(fixture.head(), fixture.candidate);
    let retry = record(
        fixture
            .tool("fast-forward", &fixture.base, "unreadable")
            .output()
            .unwrap(),
        0,
    );
    assert_eq!(retry["status"], "already_applied");
}

#[test]
#[cfg(unix)]
fn successful_cas_followed_by_descendant_head_reports_actual_after_head() {
    let fixture = Fixture::new("post-cas-drift");
    let other = git(
        &fixture.repo,
        &[
            "commit-tree",
            &fixture.tree,
            "-p",
            &fixture.candidate,
            "-m",
            "external advance",
        ],
    );
    let mut command = fixture.tool("fast-forward", &fixture.base, "advanced-after");
    fixture.inject(&mut command, "advanced-after");
    command.env("HCTL2_TEST_OTHER", &other);
    let value = record(command.output().unwrap(), 0);
    assert_eq!(value["status"], "applied");
    assert_eq!(value["before_head"], fixture.base);
    assert_eq!(value["new_head"], fixture.candidate);
    assert_eq!(value["after_head"], other);
    assert_eq!(fixture.head(), other);
}

#[test]
#[cfg(unix)]
fn failed_exit_after_update_still_confirms_result_in_descendant_head() {
    let fixture = Fixture::new("failed-post-cas-drift");
    let other = git(
        &fixture.repo,
        &[
            "commit-tree",
            &fixture.tree,
            "-p",
            &fixture.candidate,
            "-m",
            "external advance",
        ],
    );
    let mut command = fixture.tool("fast-forward", &fixture.base, "failed-and-advanced-after");
    fixture.inject(&mut command, "failed-and-advanced-after");
    command.env("HCTL2_TEST_OTHER", &other);
    let value = record(command.output().unwrap(), 0);
    assert_eq!(value["status"], "applied");
    assert_eq!(value["new_head"], fixture.candidate);
    assert_eq!(value["after_head"], other);
    assert_eq!(fixture.head(), other);
}

#[test]
#[cfg(unix)]
fn native_reference_transaction_hook_can_reject_target_and_retry_uses_prepared_commit() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = Fixture::new("hook");
    let hook = fixture.repo.join(".git/hooks/reference-transaction");
    fs::write(
        &hook,
        r#"#!/bin/sh
if [ "$1" = prepared ]; then
    while read -r old new ref; do
        if [ "$ref" = refs/heads/main ]; then exit 1; fi
    done
fi
exit 0
"#,
    )
    .unwrap();
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
    rejected_output(
        &mut fixture.tool("merge-commit", &fixture.base, "hook"),
        "HCTL2_TOOL_INTEGRATION_CAS_REJECTED",
        3,
    );
    let prepared = git(
        &fixture.repo,
        &[
            "for-each-ref",
            "--format=%(objectname)",
            "refs/hctl2/integrations/",
        ],
    );
    assert_eq!(fixture.head(), fixture.base);
    fs::rename(&hook, hook.with_extension("disabled")).unwrap();
    let retry = record(
        fixture
            .tool("merge-commit", &fixture.base, "hook")
            .env("GIT_COMMITTER_DATE", "2020-01-01T00:00:00Z")
            .output()
            .unwrap(),
        0,
    );
    assert_eq!(retry["new_head"], prepared);
}

#[test]
#[cfg(unix)]
fn rejected_cas_is_retryable_and_exit_failure_after_update_does_not_hide_readback() {
    let fixture = Fixture::new("rejection");
    let mut command = fixture.tool("fast-forward", &fixture.base, "rejection");
    fixture.inject(&mut command, "rejected");
    rejected_output(&mut command, "HCTL2_TOOL_INTEGRATION_CAS_REJECTED", 3);
    assert_eq!(fixture.head(), fixture.base);
    let mut retry = fixture.tool("fast-forward", &fixture.base, "rejection");
    fixture.inject(&mut retry, "failed-after");
    let value = record(retry.output().unwrap(), 0);
    assert_eq!(value["status"], "applied");
    assert_eq!(value["after_head"], fixture.candidate);
}

#[test]
fn checked_out_main_is_rejected_before_any_ref_write_and_detach_allows_retry() {
    let fixture = Fixture::new("checked-out-main");
    git(&fixture.repo, &["switch", "main"]);
    fs::write(fixture.repo.join("unrelated.txt"), "user staging\n").unwrap();
    git(&fixture.repo, &["add", "unrelated.txt"]);
    fs::write(fixture.repo.join("untracked.txt"), "only copy\n").unwrap();
    let index = fs::read(fixture.repo.join(".git/index")).unwrap();
    let refs = fixture.refs();
    let value = rejected_output(
        &mut fixture.tool("fast-forward", &fixture.base, "checked"),
        "HCTL2_TOOL_INTEGRATION_TARGET_CHECKED_OUT",
        3,
    );
    assert_eq!(
        value["error"]["details"]["worktree_paths"],
        json!([fixture.repo.canonicalize().unwrap()])
    );
    assert_eq!(fixture.refs(), refs);
    assert_eq!(fs::read(fixture.repo.join(".git/index")).unwrap(), index);
    assert_eq!(
        fs::read_to_string(fixture.repo.join("untracked.txt")).unwrap(),
        "only copy\n"
    );
    assert!(!git(&fixture.repo, &["status", "--short"]).contains("candidate.txt"));
    git(&fixture.repo, &["switch", "--detach"]);
    let retry = record(
        fixture
            .tool("fast-forward", &fixture.base, "checked")
            .output()
            .unwrap(),
        0,
    );
    assert_eq!(retry["status"], "applied");
    assert_eq!(retry["warnings"], json!([]));
    assert_eq!(fixture.head(), fixture.candidate);
}

#[test]
fn linked_changeset_target_is_rejected_with_its_exact_path_and_no_lost_bytes() {
    let fixture = Fixture::new("checked-out-linked");
    let target = "refs/heads/hctl2/changeset/CS-9";
    git(
        &fixture.repo,
        &["branch", "hctl2/changeset/CS-9", &fixture.base],
    );
    let linked = fixture.root.join("linked changeset");
    git(
        &fixture.repo,
        &[
            "worktree",
            "add",
            linked.to_str().unwrap(),
            "hctl2/changeset/CS-9",
        ],
    );
    fs::write(linked.join("base.txt"), "staged\n").unwrap();
    git(&linked, &["add", "base.txt"]);
    fs::write(linked.join("base.txt"), "unstaged\n").unwrap();
    fs::write(linked.join("untracked"), "unique\n").unwrap();
    let index_path = git(
        &linked,
        &["rev-parse", "--path-format=absolute", "--git-path", "index"],
    );
    let index = fs::read(&index_path).unwrap();
    let refs = fixture.refs();
    let value = rejected_output(
        &mut fixture.command(
            "merge-commit",
            &fixture.base,
            "linked",
            &fixture.base,
            &fixture.candidate,
            &fixture.tree,
            target,
        ),
        "HCTL2_TOOL_INTEGRATION_TARGET_CHECKED_OUT",
        3,
    );
    assert_eq!(
        value["error"]["details"]["worktree_paths"],
        json!([linked.canonicalize().unwrap()])
    );
    assert_eq!(fixture.refs(), refs);
    assert_eq!(fs::read(&index_path).unwrap(), index);
    assert_eq!(
        fs::read_to_string(linked.join("base.txt")).unwrap(),
        "unstaged\n"
    );
    assert_eq!(
        fs::read_to_string(linked.join("untracked")).unwrap(),
        "unique\n"
    );
    git(&linked, &["switch", "--detach"]);
    let retry = record(
        fixture
            .command(
                "merge-commit",
                &fixture.base,
                "linked",
                &fixture.base,
                &fixture.candidate,
                &fixture.tree,
                target,
            )
            .output()
            .unwrap(),
        0,
    );
    assert_eq!(retry["status"], "applied");
}

#[test]
fn successful_retry_accepts_descendants_without_reapplying_even_when_checked_out() {
    for strategy in ["fast-forward", "merge-commit"] {
        let fixture = Fixture::new(strategy);
        let first = record(
            fixture
                .tool(strategy, &fixture.base, "descendant")
                .output()
                .unwrap(),
            0,
        );
        let new = first["new_head"].as_str().unwrap();
        git(&fixture.repo, &["switch", "main"]);
        fs::write(fixture.repo.join("later.txt"), "later content\n").unwrap();
        let after = commit(&fixture.repo, "later work");
        let refs = fixture.refs();
        let retry = record(
            fixture
                .tool(strategy, &fixture.base, "descendant")
                .output()
                .unwrap(),
            0,
        );
        assert_eq!(retry["status"], "already_applied");
        assert_eq!(retry["new_head"], new);
        assert_eq!(retry["before_head"], after);
        assert_eq!(retry["after_head"], after);
        assert_eq!(retry["integrated_tree_sha"], first["integrated_tree_sha"]);
        assert_ne!(
            git(&fixture.repo, &["rev-parse", "HEAD^{tree}"]),
            retry["integrated_tree_sha"].as_str().unwrap()
        );
        assert_eq!(fixture.refs(), refs);
    }
}

#[test]
fn reflogs_identify_target_and_prepared_ref_writes_by_key_digest() {
    let fixture = Fixture::new("reflogs");
    let value = record(
        fixture
            .tool("merge-commit", &fixture.base, "log-key")
            .output()
            .unwrap(),
        0,
    );
    let reference = value["prepared_ref"].as_str().unwrap();
    let key_digest = reference.rsplit('/').nth(1).unwrap();
    let expected = format!("hctl2 integrate {}", &key_digest[..12]);
    for reference in [TARGET, reference] {
        assert_eq!(
            git(
                &fixture.repo,
                &["reflog", "show", "-1", "--format=%gs", reference]
            ),
            expected
        );
    }
}

#[test]
#[cfg(unix)]
fn checkout_appearing_after_preparation_is_rechecked_before_target_cas() {
    let fixture = Fixture::new("checkout-race");
    let linked = fixture.root.join("late-checkout");
    let mut command = fixture.tool("merge-commit", &fixture.base, "late-checkout");
    fixture.inject(&mut command, "checkout-after-prepare");
    command.env("HCTL2_TEST_WORKTREE", &linked);
    rejected_output(&mut command, "HCTL2_TOOL_INTEGRATION_TARGET_CHECKED_OUT", 3);
    assert_eq!(fixture.head(), fixture.base);
    let prepared = git(
        &fixture.repo,
        &[
            "for-each-ref",
            "--format=%(objectname)",
            "refs/hctl2/integrations/",
        ],
    );
    assert!(!prepared.is_empty());
    git(&linked, &["switch", "--detach"]);
    let retry = record(
        fixture
            .tool("merge-commit", &fixture.base, "late-checkout")
            .output()
            .unwrap(),
        0,
    );
    assert_eq!(retry["new_head"], prepared);
}

#[test]
#[cfg(unix)]
fn non_descendant_after_cas_is_unknown_even_if_git_reports_success() {
    let fixture = Fixture::new("diverged-after");
    let other = git(
        &fixture.repo,
        &[
            "commit-tree",
            &fixture.tree,
            "-p",
            &fixture.base,
            "-m",
            "diverged history",
        ],
    );
    let mut command = fixture.tool("fast-forward", &fixture.base, "diverged-after");
    fixture.inject(&mut command, "advanced-after");
    command.env("HCTL2_TEST_OTHER", &other);
    rejected_output(&mut command, "HCTL2_TOOL_INTEGRATION_RESULT_UNKNOWN", 4);
    assert_eq!(fixture.head(), other);
}

#[test]
#[cfg(unix)]
fn unreadable_post_cas_ancestry_remains_unknown_and_keeps_observed_head() {
    let fixture = Fixture::new("ancestry-failure");
    let other = git(
        &fixture.repo,
        &[
            "commit-tree",
            &fixture.tree,
            "-p",
            &fixture.candidate,
            "-m",
            "later work",
        ],
    );
    let mut command = fixture.tool("fast-forward", &fixture.base, "ancestry-failure");
    fixture.inject(&mut command, "ancestry-failure");
    command
        .env("HCTL2_TEST_OTHER", &other)
        .env("HCTL2_TEST_RESULT", &fixture.candidate);
    let value = rejected_output(&mut command, "HCTL2_TOOL_INTEGRATION_RESULT_UNKNOWN", 4);
    assert_eq!(value["error"]["details"]["observed_head"], other);
    let retry = record(
        fixture
            .tool("fast-forward", &fixture.base, "ancestry-failure")
            .output()
            .unwrap(),
        0,
    );
    assert_eq!(retry["status"], "already_applied");
    assert_eq!(retry["after_head"], other);
}

#[test]
fn tampered_prepared_commit_is_rejected_without_touching_target() {
    let fixture = Fixture::new("cache");
    let first = record(
        fixture
            .tool("fast-forward", &fixture.base, "cache")
            .output()
            .unwrap(),
        0,
    );
    git(
        &fixture.repo,
        &[
            "update-ref",
            first["prepared_ref"].as_str().unwrap(),
            &fixture.base,
            &fixture.candidate,
        ],
    );
    let refs = fixture.refs();
    rejected_output(
        &mut fixture.tool("fast-forward", &fixture.base, "cache"),
        "HCTL2_TOOL_INTEGRATION_CACHE_INVALID",
        3,
    );
    assert_eq!(fixture.refs(), refs);
}
