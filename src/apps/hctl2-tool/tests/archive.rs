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
        git(None, ["init", "-b", "main", repo.to_str().expect("utf-8")]);
        git(Some(&repo), ["config", "user.name", "HCTL2 Test"]);
        git(
            Some(&repo),
            ["config", "user.email", "hctl2@example.invalid"],
        );
        fs::write(repo.join("README.md"), "base\n").expect("tracked file");
        git(Some(&repo), ["add", "."]);
        git(Some(&repo), ["commit", "-m", "base"]);
        let first_commit = git_stdout(Some(&repo), ["rev-parse", "HEAD"]);
        Self {
            root,
            repo,
            first_commit,
        }
    }

    fn materialize(&self, change_set_ref: &str) -> PathBuf {
        let root = self.root.join("worktrees");
        let output = tool()
            .args(["worktree", "materialize", "--repo"])
            .arg(&self.repo)
            .arg("--root")
            .arg(&root)
            .args([
                "--change-set-ref",
                change_set_ref,
                "--baseline",
                &self.first_commit,
            ])
            .output()
            .expect("materialize");
        assert_success(&output);
        PathBuf::from(
            json_stdout(&output)["worktree"]["path"]
                .as_str()
                .expect("worktree path"),
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn snapshot_captures_tracked_and_untracked_but_not_ignored() {
    let fixture = Fixture::new("snapshot-contents");
    let worktree = fixture.materialize("CS-snap");
    fs::write(worktree.join("README.md"), "edited\n").expect("tracked edit");
    fs::write(worktree.join("notes.txt"), "untracked\n").expect("untracked");
    fs::write(worktree.join(".gitignore"), "scratch/\n").expect("gitignore");
    fs::create_dir(worktree.join("scratch")).expect("ignored dir");
    fs::write(worktree.join("scratch/tmp.bin"), "ignored-bytes").expect("ignored file");
    let cached_before = git_stdout(Some(&worktree), ["write-tree"]);

    let first = snapshot(&fixture, "CS-snap");
    assert_success(&first);
    let record = json_stdout(&first);
    assert_eq!(record["schema"], "hctl2.archive.v1");
    assert_eq!(record["evidence_level"], "toolbox_readback");
    assert_eq!(record["outcome"], "established");
    assert_eq!(record["error"], serde_json::Value::Null);
    let tree = record["result_tree_sha"].as_str().expect("tree");
    let commit = record["snapshot_commit_sha"].as_str().expect("commit");
    assert_eq!(record["base_commit_sha"], fixture.first_commit);
    assert_ne!(
        tree,
        git_stdout(Some(&worktree), ["rev-parse", "HEAD^{tree}"])
    );
    assert_eq!(
        git_stdout(Some(&fixture.repo), ["cat-file", "-t", commit]),
        "commit"
    );
    let listing = git_stdout(Some(&fixture.repo), ["ls-tree", "-r", "--name-only", tree]);
    assert!(listing.contains("README.md"));
    assert!(listing.contains("notes.txt"));
    assert!(listing.contains(".gitignore"));
    assert!(!listing.contains("scratch/tmp.bin"));
    assert_eq!(git_stdout(Some(&worktree), ["write-tree"]), cached_before);
    assert_eq!(
        git_stdout(Some(&worktree), ["symbolic-ref", "HEAD"]),
        "refs/heads/hctl2/changeset/CS-snap"
    );
    assert_eq!(
        git_stdout(Some(&worktree), ["rev-parse", "HEAD"]),
        fixture.first_commit
    );

    let second = snapshot(&fixture, "CS-snap");
    assert_success(&second);
    let again = json_stdout(&second);
    assert_eq!(again["result_tree_sha"], tree);
    assert_eq!(again["snapshot_commit_sha"], commit);
    assert_eq!(again["reused"], true);
}

#[test]
fn salvage_remove_keeps_a_reachable_copy_and_lists_ignored_residue() {
    let fixture = Fixture::new("salvage-remove");
    let worktree = fixture.materialize("CS-keep");
    fs::write(worktree.join("only-copy.txt"), "precious\n").expect("untracked copy");
    fs::write(worktree.join(".gitignore"), "noise.log\n").expect("gitignore");
    fs::write(worktree.join("noise.log"), "build noise").expect("ignored");

    let output = tool()
        .args(["archive", "remove", "--repo"])
        .arg(&fixture.repo)
        .args(["--change-set-ref", "CS-keep"])
        .output()
        .expect("remove");
    assert_success(&output);
    let record = json_stdout(&output);
    assert_eq!(record["operation"], "removed_salvaged");
    assert_eq!(record["outcome"], "established");
    let snapshot = record["snapshot"]["snapshot_commit_sha"]
        .as_str()
        .expect("snapshot commit");
    let tree = record["snapshot"]["result_tree_sha"]
        .as_str()
        .expect("snapshot tree");
    assert_eq!(
        git_stdout(Some(&fixture.repo), ["cat-file", "-t", snapshot]),
        "commit"
    );
    let listing = git_stdout(Some(&fixture.repo), ["ls-tree", "-r", "--name-only", tree]);
    assert!(listing.contains("only-copy.txt"));
    assert!(!listing.contains("noise.log"));
    assert_eq!(record["ignored_residue"][0]["path"], "noise.log");
    assert_eq!(record["ignored_residue_truncated"], false);
    assert!(!worktree.exists());
    git(
        Some(&fixture.repo),
        [
            "show-ref",
            "--verify",
            "--quiet",
            "refs/heads/hctl2/changeset/CS-keep",
        ],
    );
    let registered = git(Some(&fixture.repo), ["worktree", "list", "--porcelain"]);
    let text = String::from_utf8_lossy(&registered.stdout);
    assert!(!text.contains("CS-keep"));
}

#[test]
fn discard_requires_matching_confirmation_and_does_not_keep_a_snapshot() {
    let fixture = Fixture::new("discard");
    let worktree = fixture.materialize("CS-drop");
    fs::write(worktree.join("only-copy.txt"), "gone\n").expect("untracked copy");

    let usage = tool()
        .args(["archive", "remove", "--repo"])
        .arg(&fixture.repo)
        .args(["--change-set-ref", "CS-drop", "--discard-unarchived"])
        .output()
        .expect("usage");
    assert!(!usage.status.success());
    assert!(usage.stdout.is_empty());
    assert!(!usage.stderr.is_empty());

    let mismatched = tool()
        .args(["archive", "remove", "--repo"])
        .arg(&fixture.repo)
        .args([
            "--change-set-ref",
            "CS-drop",
            "--discard-unarchived",
            "--confirm-discard",
            "other",
        ])
        .output()
        .expect("mismatch");
    let record = json_stdout(&mismatched);
    assert_eq!(
        record["error"]["code"],
        "HCTL2_TOOL_DISCARD_CONFIRMATION_MISMATCH"
    );
    assert_eq!(
        record["error"]["recovery_action"],
        "pass_matching_confirm_discard"
    );
    assert_error_code(mismatched, "HCTL2_TOOL_DISCARD_CONFIRMATION_MISMATCH");
    assert!(worktree.exists());

    let discarded = tool()
        .args(["archive", "remove", "--repo"])
        .arg(&fixture.repo)
        .args([
            "--change-set-ref",
            "CS-drop",
            "--discard-unarchived",
            "--confirm-discard",
            "CS-drop",
        ])
        .output()
        .expect("discard");
    assert_success(&discarded);
    let record = json_stdout(&discarded);
    assert_eq!(record["operation"], "removed_discarded");
    assert!(record["snapshot"].is_null());
    assert!(!worktree.exists());
    let refs = git_stdout(
        Some(&fixture.repo),
        [
            "for-each-ref",
            "--format=%(refname)",
            "refs/hctl2/changesets/CS-drop/trees/",
        ],
    );
    assert!(refs.is_empty(), "{refs}");
}

#[test]
fn reject_ignored_blocks_salvage_removal() {
    let fixture = Fixture::new("reject-ignored");
    let worktree = fixture.materialize("CS-noise");
    fs::write(worktree.join(".gitignore"), "out/\n").expect("gitignore");
    fs::create_dir(worktree.join("out")).expect("ignored dir");
    fs::write(worktree.join("out/cache"), "bytes").expect("ignored file");
    let output = tool()
        .args(["archive", "remove", "--repo"])
        .arg(&fixture.repo)
        .args(["--change-set-ref", "CS-noise", "--reject-ignored"])
        .output()
        .expect("remove");
    let record = json_stdout(&output);
    assert_eq!(
        record["error"]["recovery_action"],
        "clear_ignored_or_omit_reject_ignored"
    );
    assert!(!record["error"]["details"]["snapshot"]["snapshot_commit_sha"].is_null());
    assert_error_code(output, "HCTL2_TOOL_IGNORED_RESIDUE_PRESENT");
    assert!(worktree.exists());
}

#[test]
fn snapshot_converges_after_a_stale_temp_index() {
    let fixture = Fixture::new("stale-index");
    let worktree = fixture.materialize("CS-retry");
    fs::write(worktree.join("notes.txt"), "retry\n").expect("untracked");
    let common = git_stdout(
        Some(&fixture.repo),
        ["rev-parse", "--path-format=absolute", "--git-common-dir"],
    );
    let stale = PathBuf::from(&common).join("hctl2/archive-CS-retry.index");
    fs::create_dir_all(stale.parent().expect("parent")).expect("lock dir");
    fs::write(&stale, "garbage").expect("stale index");
    let output = snapshot(&fixture, "CS-retry");
    assert_success(&output);
    let tree = json_stdout(&output)["result_tree_sha"]
        .as_str()
        .expect("tree")
        .to_owned();
    assert_success(&snapshot(&fixture, "CS-retry"));
    assert_eq!(
        json_stdout(&snapshot(&fixture, "CS-retry"))["result_tree_sha"],
        tree
    );
    assert!(!stale.exists());
}

#[test]
fn remove_refuses_a_half_deleted_worktree_without_discard() {
    let fixture = Fixture::new("half-deleted");
    let worktree = fixture.materialize("CS-half");
    fs::write(worktree.join("only-copy.txt"), "still here\n").expect("copy");
    fs::remove_dir_all(&worktree).expect("delete worktree files");
    let output = tool()
        .args(["archive", "remove", "--repo"])
        .arg(&fixture.repo)
        .args(["--change-set-ref", "CS-half"])
        .output()
        .expect("remove");
    let record = json_stdout(&output);
    assert_eq!(
        record["error"]["recovery_action"],
        "inspect_worktree_then_retry_same_changeset"
    );
    assert_error_code(output, "HCTL2_TOOL_WORKTREE_STATE_INVALID");
}

#[test]
fn remove_refuses_when_the_harness_moved_the_branch() {
    let fixture = Fixture::new("branch-moved");
    let worktree = fixture.materialize("CS-moved");
    git(Some(&worktree), ["checkout", "--detach", "HEAD"]);
    let output = tool()
        .args(["archive", "remove", "--repo"])
        .arg(&fixture.repo)
        .args(["--change-set-ref", "CS-moved"])
        .output()
        .expect("remove");
    assert_eq!(
        json_stdout(&output)["error"]["recovery_action"],
        "restore_changeset_branch_then_retry"
    );
    assert_error_code(output, "HCTL2_TOOL_WORKTREE_BRANCH_MOVED");
    assert!(worktree.exists());
}

#[test]
fn salvage_remove_does_not_prune_other_worktrees() {
    let fixture = Fixture::new("other-worktree");
    let keep = fixture.materialize("CS-other");
    let remove = fixture.materialize("CS-gone");
    fs::write(keep.join("keep.txt"), "stay\n").expect("kept copy");
    fs::write(remove.join("drop.txt"), "salvage\n").expect("removed copy");
    let output = tool()
        .args(["archive", "remove", "--repo"])
        .arg(&fixture.repo)
        .args(["--change-set-ref", "CS-gone"])
        .output()
        .expect("remove");
    assert_success(&output);
    assert!(!remove.exists());
    assert_eq!(
        fs::read_to_string(keep.join("keep.txt")).expect("preserved"),
        "stay\n"
    );
    let registered = git(Some(&fixture.repo), ["worktree", "list", "--porcelain"]);
    let text = String::from_utf8_lossy(&registered.stdout);
    assert!(text.contains("CS-other"));
    assert!(!text.contains("CS-gone"));
}

#[test]
fn archive_fails_immediately_when_the_site_lock_is_held() {
    let fixture = Fixture::new("archive-lock");
    fixture.materialize("CS-lock");
    let common = PathBuf::from(git_stdout(
        Some(&fixture.repo),
        ["rev-parse", "--path-format=absolute", "--git-common-dir"],
    ));
    let lock_directory = common.join("hctl2");
    fs::create_dir_all(&lock_directory).expect("lock directory");
    let lock_path = lock_directory.join("lock");
    let ready_path = fixture.root.join("holder-ready");
    let mut holder = Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", "archive_lock_holder", "--nocapture"])
        .env("HCTL2_TEST_LOCK_PATH", &lock_path)
        .env("HCTL2_TEST_READY_PATH", &ready_path)
        .stdout(Stdio::null())
        .spawn()
        .expect("lock holder must start");
    wait_for_path(&ready_path);
    let rejected = snapshot(&fixture, "CS-lock");
    holder.kill().expect("holder must be killable");
    holder.wait().expect("holder must exit");
    assert_eq!(
        json_stdout(&rejected)["error"]["recovery_action"],
        "retry_when_site_lock_free"
    );
    assert_error_code(rejected, "HCTL2_TOOL_SITE_BUSY");
}

#[test]
fn archive_lock_holder() {
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
    file.set_len(0).expect("truncate");
    file.write_all(b"{\"operation\":\"test-holder\"}")
        .expect("holder information");
    file.sync_data().expect("sync");
    fs::write(ready, b"ready").expect("ready marker");
    thread::sleep(Duration::from_secs(60));
}

fn snapshot(fixture: &Fixture, change_set_ref: &str) -> Output {
    tool()
        .args(["archive", "snapshot", "--repo"])
        .arg(&fixture.repo)
        .args(["--change-set-ref", change_set_ref])
        .output()
        .expect("snapshot")
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
        .expect("git");
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
        .expect("utf-8")
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
    assert!(record["error"]["recovery_action"].is_string(), "{record}");
    let expected_exit = match record["outcome"].as_str() {
        Some("not_established") => 3,
        Some("unreadable") => 4,
        outcome => panic!("unexpected failure outcome: {outcome:?}"),
    };
    assert_eq!(output.status.code(), Some(expected_exit));
}

fn json_stdout(output: &Output) -> serde_json::Value {
    let stdout = String::from_utf8(output.stdout.clone()).expect("utf-8");
    assert_eq!(stdout.lines().count(), 1, "one JSON record");
    serde_json::from_str(stdout.trim()).expect("json")
}

fn temporary_directory(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "hctl2-archive-{name}-{}-{nanos}-{}",
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
