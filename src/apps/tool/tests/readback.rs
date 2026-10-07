//! `hctl2-tool readback`: a remote ref fetched into a repository the caller owns, and the
//! Git facts about one commit on it — reported, never interpreted.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

struct Temp(PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.invalid")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.invalid")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn commit(repo: &Path, name: &str, content: &str) -> String {
    fs::write(repo.join(name), content).unwrap();
    git(repo, &["add", name]);
    git(repo, &["commit", "-q", "-m", name]);
    git(repo, &["rev-parse", "HEAD"])
}

fn readback(mirror: &Path, remote: &Path, reference: &str, sha: &str) -> (Output, Value) {
    let output = Command::new(std::env::var("CARGO_BIN_EXE_hctl2-tool").unwrap())
        .args(["readback", "--path"])
        .arg(mirror)
        .arg("--remote")
        .arg(remote)
        .args(["--ref", reference, "--commit", sha])
        .output()
        .unwrap();
    let value: Value = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    (output, value)
}

#[test]
fn reports_the_fetched_head_and_whether_it_carries_the_commit() {
    let temp = Temp(std::env::temp_dir().join(format!("hctl2-readback-{}", std::process::id())));
    let _ = fs::remove_dir_all(&temp.0);
    fs::create_dir_all(&temp.0).unwrap();
    // The "platform": a working repository with a base, a candidate on top, and a merge.
    let platform = temp.0.join("platform");
    fs::create_dir(&platform).unwrap();
    git(&platform, &["init", "-q", "-b", "main"]);
    git(&platform, &["config", "user.name", "t"]);
    git(&platform, &["config", "user.email", "t@example.invalid"]);
    let base = commit(&platform, "a", "base\n");
    git(&platform, &["switch", "-q", "-c", "cs"]);
    let candidate = commit(&platform, "a", "candidate\n");
    let candidate_tree = git(&platform, &["rev-parse", &format!("{candidate}^{{tree}}")]);
    git(&platform, &["switch", "-q", "main"]);
    // The mirror the caller owns: bare, empty.
    let mirror = temp.0.join("mirror.git");
    git(&temp.0, &["init", "-q", "--bare", "mirror.git"]);

    // Before any merge: main is the base; the candidate is on the remote but not on main.
    let (output, facts) = readback(&mirror, &platform, "refs/heads/main", &candidate);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(facts["schema"], "hctl2.readback.v1");
    assert_eq!(facts["remote_head"], base);
    assert_eq!(facts["head"], base);
    assert_eq!(facts["commit_present"], false, "{facts}");
    assert_eq!(facts["contains"], false);

    // Merged with a merge commit: main carries it, the candidate is one of its parents.
    git(&platform, &["merge", "-q", "--no-ff", "-m", "merge", "cs"]);
    let merge = git(&platform, &["rev-parse", "HEAD"]);
    let (output, facts) = readback(&mirror, &platform, "refs/heads/main", &merge);
    assert!(output.status.success());
    assert_eq!(facts["head"], merge);
    assert_eq!(facts["commit_present"], true);
    assert_eq!(facts["commit_tree"], candidate_tree);
    assert_eq!(
        facts["commit_parents"],
        serde_json::json!([base, candidate])
    );
    assert_eq!(facts["contains"], true);
    assert_eq!(facts["head_tree"], candidate_tree);

    // main moves on: the merge commit is still carried, the head is the newer commit.
    let later = commit(&platform, "b", "later\n");
    let (_, facts) = readback(&mirror, &platform, "refs/heads/main", &merge);
    assert_eq!(facts["head"], later);
    assert_eq!(facts["contains"], true);

    // main rewritten to an unrelated history: the merge commit is no longer carried, and the
    // object is not fetched for a ref that does not reach it.
    git(&platform, &["switch", "-q", "--orphan", "fresh"]);
    let unrelated = commit(&platform, "z", "elsewhere\n");
    git(&platform, &["branch", "-f", "main", &unrelated]);
    let (_, facts) = readback(&mirror, &platform, "refs/heads/main", &merge);
    assert_eq!(facts["head"], unrelated);
    assert_eq!(facts["contains"], false, "{facts}");
    // The merge commit is still in the mirror from the earlier fetch; the fact stays honest.
    assert_eq!(facts["commit_present"], true);

    // A ref the remote does not have: no head, nothing carried.
    let (output, facts) = readback(&mirror, &platform, "refs/heads/missing", &merge);
    assert!(output.status.success());
    assert!(facts["remote_head"].is_null());
    assert!(facts["head"].is_null());
    assert_eq!(facts["contains"], false);

    // Bad inputs are refused before any fetch.
    let (output, facts) = readback(&mirror, &platform, "main", &merge);
    assert!(!output.status.success());
    assert_eq!(facts["error"]["code"], "HCTL2_TOOL_INVALID_ARGUMENT");
    let (output, _) = readback(&mirror, &platform, "refs/heads/main", "abc");
    assert!(!output.status.success());
    // A remote that is not there is an error, not an empty answer.
    let (output, facts) = readback(&mirror, &temp.0.join("nowhere"), "refs/heads/main", &merge);
    assert!(!output.status.success());
    assert_eq!(
        facts["error"]["code"],
        "HCTL2_TOOL_READBACK_REMOTE_UNREACHABLE"
    );
}
