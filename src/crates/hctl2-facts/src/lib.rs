//! Shared readers for external facts used by both admission checks and `hctl2-tool wait`.

#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

/// A closed set of external mechanical facts.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Fact {
    /// All CI checks and commit statuses for a GitHub commit have succeeded.
    CommitCi { repo: String, commit: String },
    /// A GitHub pull request has been merged.
    PullRequestMerged { repo: String, number: u64 },
    /// A GitHub ref is an ancestor-descendant advance from the supplied commit.
    RefAdvanced {
        repo: String,
        reference: String,
        from: String,
    },
    /// A file exists and its SHA-256 is exactly the supplied lowercase hexadecimal value.
    PathDigest { path: PathBuf, sha256: String },
    /// No process with the supplied process ID exists.
    ProcessExited { pid: u32 },
}

/// The one terminal answer returned for a fact query.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Established,
    NotEstablished,
    Unreadable,
    Timeout,
}

/// Structured evidence emitted by one fact query.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FactRecord {
    pub schema: &'static str,
    pub evidence_level: &'static str,
    pub fact: Fact,
    pub outcome: Outcome,
    pub observed_at_unix_ms: u128,
    pub observation: Value,
}

impl FactRecord {
    fn new(fact: Fact, outcome: Outcome, observation: Value) -> Self {
        Self {
            schema: "hctl2.external-fact.v1",
            evidence_level: "toolbox_readback",
            fact,
            outcome,
            observed_at_unix_ms: unix_millis(SystemTime::now()),
            observation,
        }
    }
}

/// Executable locations needed by external fact readers.
#[derive(Clone, Debug)]
pub struct ReaderContext {
    gh: OsString,
    ps: OsString,
}

impl ReaderContext {
    #[must_use]
    pub fn new(gh: impl Into<OsString>) -> Self {
        Self {
            gh: gh.into(),
            ps: OsString::from("ps"),
        }
    }

    #[cfg(test)]
    fn with_ps(mut self, ps: impl Into<OsString>) -> Self {
        self.ps = ps.into();
        self
    }
}

/// Validates fact arguments before any observation is attempted.
///
/// # Errors
///
/// Returns a description when a fact contains an unsupported identifier or value.
pub fn validate(fact: &Fact) -> Result<(), String> {
    match fact {
        Fact::CommitCi { repo, commit } => {
            validate_repo(repo)?;
            validate_git_name("commit", commit)
        }
        Fact::PullRequestMerged { repo, number } => {
            validate_repo(repo)?;
            if *number == 0 {
                Err("pull request number must be positive".to_owned())
            } else {
                Ok(())
            }
        }
        Fact::RefAdvanced {
            repo,
            reference,
            from,
        } => {
            validate_repo(repo)?;
            validate_git_name("ref", reference)?;
            validate_git_name("from commit", from)
        }
        Fact::PathDigest { sha256, .. } => {
            if sha256.len() == 64
                && sha256
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                Ok(())
            } else {
                Err("SHA-256 must be 64 lowercase hexadecimal characters".to_owned())
            }
        }
        Fact::ProcessExited { pid } => {
            if *pid == 0 {
                Err("process ID must be positive".to_owned())
            } else {
                Ok(())
            }
        }
    }
}

fn validate_repo(repo: &str) -> Result<(), String> {
    let mut parts = repo.split('/');
    let owner = parts.next().unwrap_or_default();
    let name = parts.next().unwrap_or_default();
    if !owner.is_empty()
        && !name.is_empty()
        && parts.next().is_none()
        && owner.bytes().all(is_git_name_byte)
        && name.bytes().all(is_git_name_byte)
    {
        Ok(())
    } else {
        Err("GitHub repo must have the form OWNER/REPO".to_owned())
    }
}

fn validate_git_name(label: &str, value: &str) -> Result<(), String> {
    if !value.is_empty() && value.bytes().all(is_git_name_byte) {
        Ok(())
    } else {
        Err(format!("{label} contains unsupported characters"))
    }
}

const fn is_git_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'/')
}

#[derive(Debug)]
enum Observation {
    Pending(Value),
    Terminal(Outcome, Value),
}

/// Result of one external-fact read.
#[derive(Clone, Debug, PartialEq)]
pub enum ReadResult {
    /// The fact can still become true before a caller-supplied deadline.
    Pending(Value),
    /// A terminal answer, including an unreadable fact.
    Answer(FactRecord),
}

/// Reads a fact once. Pending facts retain their latest observation; every terminal state is a
/// structured answer.
///
/// This is the shared entry point intended for both control admission and the waiting CLI.
#[must_use]
pub fn read_once(fact: &Fact, context: &ReaderContext) -> ReadResult {
    if let Err(message) = validate(fact) {
        return ReadResult::Answer(FactRecord::new(
            fact.clone(),
            Outcome::Unreadable,
            json!({ "error": message }),
        ));
    }
    match observe(fact, context) {
        Ok(Observation::Pending(value)) => ReadResult::Pending(value),
        Ok(Observation::Terminal(outcome, value)) => {
            ReadResult::Answer(FactRecord::new(fact.clone(), outcome, value))
        }
        Err(value) => ReadResult::Answer(FactRecord::new(fact.clone(), Outcome::Unreadable, value)),
    }
}

/// Waits until a fact reaches a terminal answer or the absolute deadline is reached.
#[must_use]
pub fn wait_until(fact: Fact, deadline: SystemTime, context: &ReaderContext) -> FactRecord {
    let mut pending = json!({ "state": "not_observed" });
    let mut path_waiter = match &fact {
        Fact::PathDigest { path, .. } => PathWaiter::new(path),
        _ => None,
    };

    loop {
        match read_once(&fact, context) {
            ReadResult::Answer(record) => return record,
            ReadResult::Pending(value) => pending = value,
        }

        let now = SystemTime::now();
        let Ok(remaining) = deadline.duration_since(now) else {
            return FactRecord::new(fact, Outcome::Timeout, pending);
        };
        if remaining.is_zero() {
            return FactRecord::new(fact, Outcome::Timeout, pending);
        }

        let interval = poll_interval(&fact).min(remaining);
        if let Some(waiter) = path_waiter.as_mut() {
            waiter.wait(interval);
        } else {
            std::thread::sleep(interval);
        }
    }
}

fn observe(fact: &Fact, context: &ReaderContext) -> Result<Observation, Value> {
    match fact {
        Fact::CommitCi { repo, commit } => observe_commit_ci(repo, commit, context),
        Fact::PullRequestMerged { repo, number } => observe_pull_request(repo, *number, context),
        Fact::RefAdvanced {
            repo,
            reference,
            from,
        } => observe_ref(repo, reference, from, context),
        Fact::PathDigest { path, sha256 } => observe_path(path, sha256),
        Fact::ProcessExited { pid } => observe_process(*pid, context),
    }
}

fn observe_commit_ci(
    repo: &str,
    commit: &str,
    context: &ReaderContext,
) -> Result<Observation, Value> {
    let checks = run_gh_json(
        context,
        [
            "api".to_owned(),
            "--method".to_owned(),
            "GET".to_owned(),
            "--paginate".to_owned(),
            "--slurp".to_owned(),
            "-H".to_owned(),
            "X-GitHub-Api-Version: 2022-11-28".to_owned(),
            format!("repos/{repo}/commits/{commit}/check-runs?filter=latest&per_page=100"),
        ],
    )?;
    let statuses = run_gh_json(
        context,
        [
            "api".to_owned(),
            "--method".to_owned(),
            "GET".to_owned(),
            "--paginate".to_owned(),
            "--slurp".to_owned(),
            "-H".to_owned(),
            "X-GitHub-Api-Version: 2022-11-28".to_owned(),
            format!("repos/{repo}/commits/{commit}/status?per_page=100"),
        ],
    )?;
    evaluate_commit_ci(&checks, &statuses)
}

fn evaluate_commit_ci(checks: &Value, statuses: &Value) -> Result<Observation, Value> {
    let check_pages = response_pages(checks);
    let status_pages = response_pages(statuses);
    let mut check_runs = Vec::new();
    for page in check_pages {
        check_runs.extend(
            page.get("check_runs")
                .and_then(Value::as_array)
                .ok_or_else(
                    || json!({ "error": "GitHub check-runs response has no check_runs array" }),
                )?,
        );
    }
    let mut status_rows = Vec::new();
    for page in &status_pages {
        status_rows.extend(
            page.get("statuses").and_then(Value::as_array).ok_or_else(
                || json!({ "error": "GitHub status response has no statuses array" }),
            )?,
        );
    }
    let value = json!({ "check_runs": check_runs, "combined_status": statuses });

    let checks_pending = check_runs
        .iter()
        .any(|run| run.get("status").and_then(Value::as_str) != Some("completed"));
    let checks_failed = check_runs.iter().any(|run| {
        run.get("status").and_then(Value::as_str) == Some("completed")
            && !matches!(
                run.get("conclusion").and_then(Value::as_str),
                Some("success" | "neutral" | "skipped")
            )
    });
    let statuses_state = status_pages
        .first()
        .and_then(|page| page.get("state"))
        .and_then(Value::as_str);
    let statuses_failed =
        !status_rows.is_empty() && matches!(statuses_state, Some("failure" | "error"));
    let statuses_pending = !status_rows.is_empty() && statuses_state == Some("pending");

    if checks_failed || statuses_failed {
        Ok(Observation::Terminal(Outcome::NotEstablished, value))
    } else if checks_pending
        || statuses_pending
        || (check_runs.is_empty() && status_rows.is_empty())
    {
        Ok(Observation::Pending(value))
    } else {
        Ok(Observation::Terminal(Outcome::Established, value))
    }
}

fn response_pages(value: &Value) -> Vec<&Value> {
    value
        .as_array()
        .map_or_else(|| vec![value], |pages| pages.iter().collect())
}

fn observe_pull_request(
    repo: &str,
    number: u64,
    context: &ReaderContext,
) -> Result<Observation, Value> {
    let value = run_gh_json(
        context,
        [
            "pr".to_owned(),
            "view".to_owned(),
            number.to_string(),
            "--repo".to_owned(),
            repo.to_owned(),
            "--json".to_owned(),
            "mergedAt,state,url,headRefOid".to_owned(),
        ],
    )?;
    if value
        .get("mergedAt")
        .is_some_and(|merged| !merged.is_null())
    {
        Ok(Observation::Terminal(Outcome::Established, value))
    } else if value.get("state").and_then(Value::as_str) == Some("CLOSED") {
        Ok(Observation::Terminal(Outcome::NotEstablished, value))
    } else {
        Ok(Observation::Pending(value))
    }
}

fn observe_ref(
    repo: &str,
    reference: &str,
    from: &str,
    context: &ReaderContext,
) -> Result<Observation, Value> {
    let normalized = reference.strip_prefix("refs/").unwrap_or(reference);
    let current = run_gh_json(
        context,
        [
            "api".to_owned(),
            "--method".to_owned(),
            "GET".to_owned(),
            "-H".to_owned(),
            "X-GitHub-Api-Version: 2022-11-28".to_owned(),
            format!("repos/{repo}/git/ref/{normalized}"),
        ],
    )?;
    let Some(current_sha) = current.pointer("/object/sha").and_then(Value::as_str) else {
        return Err(
            json!({ "error": "GitHub ref response has no object.sha", "response": current }),
        );
    };
    if current_sha == from {
        return Ok(Observation::Pending(json!({ "ref": current })));
    }

    let comparison = run_gh_json(
        context,
        [
            "api".to_owned(),
            "--method".to_owned(),
            "GET".to_owned(),
            "-H".to_owned(),
            "X-GitHub-Api-Version: 2022-11-28".to_owned(),
            format!("repos/{repo}/compare/{from}...{current_sha}"),
        ],
    )?;
    let value = json!({ "ref": current, "comparison": comparison });
    if value.pointer("/comparison/status").and_then(Value::as_str) == Some("ahead") {
        Ok(Observation::Terminal(Outcome::Established, value))
    } else {
        Ok(Observation::Terminal(Outcome::NotEstablished, value))
    }
}

fn observe_path(path: &Path, expected: &str) -> Result<Observation, Value> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(Observation::Pending(json!({ "exists": false })));
        }
        Err(error) => {
            return Err(json!({ "error": error.to_string(), "path": path }));
        }
    };
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| json!({ "error": error.to_string(), "path": path }))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let actual = lower_hex(&hasher.finalize());
    let value = json!({ "exists": true, "sha256": actual });
    if actual == expected {
        Ok(Observation::Terminal(Outcome::Established, value))
    } else {
        Ok(Observation::Pending(value))
    }
}

fn observe_process(pid: u32, context: &ReaderContext) -> Result<Observation, Value> {
    let output = Command::new(&context.ps)
        .args(["-p", &pid.to_string(), "-o", "pid="])
        .output()
        .map_err(
            |error| json!({ "error": error.to_string(), "program": context.ps.to_string_lossy() }),
        )?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if output.status.success() && !stdout.trim().is_empty() {
        Ok(Observation::Pending(json!({ "running": true, "pid": pid })))
    } else if !output.status.success() && stdout.trim().is_empty() && output.stderr.is_empty() {
        Ok(Observation::Terminal(
            Outcome::Established,
            json!({ "running": false, "pid": pid }),
        ))
    } else {
        Err(json!({
            "error": String::from_utf8_lossy(&output.stderr),
            "exit_code": output.status.code()
        }))
    }
}

fn run_gh_json<I>(context: &ReaderContext, arguments: I) -> Result<Value, Value>
where
    I: IntoIterator<Item = String>,
{
    let output = Command::new(&context.gh)
        .args(arguments)
        .env("GH_PROMPT_DISABLED", "1")
        .env("NO_COLOR", "1")
        .output()
        .map_err(
            |error| json!({ "error": error.to_string(), "program": context.gh.to_string_lossy() }),
        )?;
    if !output.status.success() {
        return Err(json!({
            "error": String::from_utf8_lossy(&output.stderr),
            "exit_code": output.status.code()
        }));
    }
    serde_json::from_slice(&output.stdout).map_err(|error| {
        json!({
            "error": format!("gh returned invalid JSON: {error}"),
            "stdout": String::from_utf8_lossy(&output.stdout)
        })
    })
}

fn poll_interval(fact: &Fact) -> Duration {
    match fact {
        Fact::CommitCi { .. } | Fact::PullRequestMerged { .. } | Fact::RefAdvanced { .. } => {
            Duration::from_secs(5)
        }
        Fact::PathDigest { .. } => Duration::from_secs(1),
        Fact::ProcessExited { .. } => Duration::from_millis(250),
    }
}

struct PathWaiter {
    _watcher: RecommendedWatcher,
    receiver: Receiver<notify::Result<notify::Event>>,
}

impl PathWaiter {
    fn new(path: &Path) -> Option<Self> {
        let root = nearest_existing_directory(path)?;
        let (sender, receiver) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |event| {
            let _ = sender.send(event);
        })
        .ok()?;
        watcher.watch(&root, RecursiveMode::Recursive).ok()?;
        Some(Self {
            _watcher: watcher,
            receiver,
        })
    }

    fn wait(&mut self, timeout: Duration) {
        let _ = self.receiver.recv_timeout(timeout);
    }
}

fn nearest_existing_directory(path: &Path) -> Option<PathBuf> {
    let mut candidate = path.parent()?;
    loop {
        if candidate.is_dir() {
            return Some(candidate.to_owned());
        }
        candidate = candidate.parent()?;
    }
}

fn unix_millis(time: SystemTime) -> u128 {
    time.duration_since(UNIX_EPOCH)
        .map_or(0, |value| value.as_millis())
}

fn lower_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{
        Fact, Observation, Outcome, ReadResult, ReaderContext, evaluate_commit_ci, observe_path,
        observe_process, read_once, validate, wait_until,
    };
    use serde_json::json;
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;
    use std::thread;
    use std::time::{Duration, SystemTime};

    fn temporary_directory(name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "hctl2-facts-{name}-{}-{}",
            std::process::id(),
            super::unix_millis(SystemTime::now())
        ));
        fs::create_dir(&path).expect("temporary directory must be created");
        path
    }

    #[test]
    fn validates_closed_fact_arguments() {
        assert!(
            validate(&Fact::PullRequestMerged {
                repo: "yesme/hctl2".to_owned(),
                number: 82,
            })
            .is_ok()
        );
        assert!(
            validate(&Fact::PullRequestMerged {
                repo: "not-a-repo".to_owned(),
                number: 0,
            })
            .is_err()
        );
    }

    #[test]
    fn commit_ci_distinguishes_pending_failure_and_success() {
        let statuses = json!({ "state": "pending", "statuses": [] });
        assert!(matches!(
            evaluate_commit_ci(&json!({ "check_runs": [] }), &statuses),
            Ok(Observation::Pending(_))
        ));

        let failed = json!({
            "check_runs": [{"status": "completed", "conclusion": "failure"}]
        });
        assert!(matches!(
            evaluate_commit_ci(&failed, &json!({ "state": "pending", "statuses": [] })),
            Ok(Observation::Terminal(Outcome::NotEstablished, _))
        ));

        let passed = json!({
            "check_runs": [{"status": "completed", "conclusion": "success"}]
        });
        assert!(matches!(
            evaluate_commit_ci(&passed, &json!({ "state": "pending", "statuses": [] })),
            Ok(Observation::Terminal(Outcome::Established, _))
        ));
    }

    #[test]
    fn path_fact_reads_sha256_directly() {
        let directory = temporary_directory("digest");
        let path = directory.join("artifact");
        fs::write(&path, b"hctl2").expect("fixture must be written");
        let digest = super::lower_hex(&Sha256::digest(b"hctl2"));

        assert!(matches!(
            observe_path(&path, &digest),
            Ok(Observation::Terminal(Outcome::Established, _))
        ));
        fs::remove_dir_all(directory).expect("fixture must be removed");
    }

    #[test]
    fn wrong_digest_times_out_instead_of_claiming_a_terminal_negative() {
        let directory = temporary_directory("timeout");
        let path = directory.join("artifact");
        fs::write(&path, b"wrong").expect("fixture must be written");
        let fact = Fact::PathDigest {
            path,
            sha256: "00".repeat(32),
        };
        let record = wait_until(fact, SystemTime::now(), &ReaderContext::new("gh"));

        assert_eq!(record.outcome, Outcome::Timeout);
        fs::remove_dir_all(directory).expect("fixture must be removed");
    }

    #[test]
    fn process_reader_treats_an_empty_ps_answer_as_exited() {
        let context = ReaderContext::new("gh").with_ps("false");
        let result = observe_process(u32::MAX, &context);

        assert!(matches!(
            result,
            Ok(Observation::Terminal(Outcome::Established, _))
        ));
    }

    #[test]
    fn process_reader_does_not_hide_a_broken_ps_command() {
        let context = ReaderContext::new("gh").with_ps("true");
        let result = observe_process(1, &context);

        assert!(result.is_err());
    }

    #[test]
    fn wait_record_is_toolbox_readback_evidence() {
        let fact = Fact::ProcessExited { pid: u32::MAX };
        let record = wait_until(
            fact,
            SystemTime::now() + Duration::from_secs(1),
            &ReaderContext::new("gh"),
        );

        assert_eq!(record.schema, "hctl2.external-fact.v1");
        assert_eq!(record.evidence_level, "toolbox_readback");
    }

    fn write_executable(directory: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
        let path = directory.join(name);
        fs::write(&path, body).expect("fixture program must be written");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
            .expect("fixture program must be executable");
        path
    }

    fn assert_closed_fact_set(fact: &Fact) {
        match fact {
            Fact::CommitCi { .. }
            | Fact::PullRequestMerged { .. }
            | Fact::RefAdvanced { .. }
            | Fact::PathDigest { .. }
            | Fact::ProcessExited { .. } => {}
        }
    }

    #[test]
    fn fact_set_is_closed_and_has_no_restatement_kind() {
        let facts = [
            Fact::CommitCi {
                repo: "yesme/hctl2".to_owned(),
                commit: "abc".to_owned(),
            },
            Fact::PullRequestMerged {
                repo: "yesme/hctl2".to_owned(),
                number: 1,
            },
            Fact::RefAdvanced {
                repo: "yesme/hctl2".to_owned(),
                reference: "main".to_owned(),
                from: "abc".to_owned(),
            },
            Fact::PathDigest {
                path: "missing".into(),
                sha256: "00".repeat(32),
            },
            Fact::ProcessExited { pid: 1 },
        ];
        for fact in &facts {
            assert_closed_fact_set(fact);
        }
        assert_eq!(facts.len(), 5);
    }

    #[test]
    fn pull_request_outcomes_distinguish_established_not_established_and_unreadable() {
        let directory = temporary_directory("pr");
        let established = write_executable(
            &directory,
            "gh-merged",
            r#"#!/bin/sh
echo '{"mergedAt":"2026-01-01T00:00:00Z","state":"MERGED","url":"u","headRefOid":"abc"}'
"#,
        );
        let closed = write_executable(
            &directory,
            "gh-closed",
            r#"#!/bin/sh
echo '{"mergedAt":null,"state":"CLOSED","url":"u","headRefOid":"abc"}'
"#,
        );
        let broken = write_executable(
            &directory,
            "gh-broken",
            r#"#!/bin/sh
echo 'not json from a model restatement' >&2
exit 1
"#,
        );
        let fact = Fact::PullRequestMerged {
            repo: "yesme/hctl2".to_owned(),
            number: 82,
        };

        let merged = read_once(&fact, &ReaderContext::new(established.into_os_string()));
        let ReadResult::Answer(merged) = merged else {
            panic!("merged pull request must be terminal");
        };
        assert_eq!(merged.outcome, Outcome::Established);
        assert_eq!(merged.evidence_level, "toolbox_readback");

        let rejected = read_once(&fact, &ReaderContext::new(closed.into_os_string()));
        let ReadResult::Answer(rejected) = rejected else {
            panic!("closed pull request must be terminal");
        };
        assert_eq!(rejected.outcome, Outcome::NotEstablished);

        let unreadable = read_once(&fact, &ReaderContext::new(broken.into_os_string()));
        let ReadResult::Answer(unreadable) = unreadable else {
            panic!("broken gh must be terminal");
        };
        assert_eq!(unreadable.outcome, Outcome::Unreadable);
        fs::remove_dir_all(directory).expect("fixture must be removed");
    }

    #[test]
    fn restatement_text_in_github_payload_does_not_establish_a_pull_request() {
        let directory = temporary_directory("restatement");
        let gh = write_executable(
            &directory,
            "gh",
            r#"#!/bin/sh
echo '{"mergedAt":null,"state":"OPEN","url":"u","headRefOid":"abc","body":"the model reports this PR is merged"}'
"#,
        );
        let fact = Fact::PullRequestMerged {
            repo: "yesme/hctl2".to_owned(),
            number: 82,
        };
        let result = read_once(&fact, &ReaderContext::new(gh.into_os_string()));
        assert!(
            matches!(result, ReadResult::Pending(_)),
            "unstructured restatement must not satisfy the merge fact: {result:?}"
        );
        fs::remove_dir_all(directory).expect("fixture must be removed");
    }

    #[test]
    fn commit_ci_success_failure_and_unreadable_go_through_gh() {
        let directory = temporary_directory("ci");
        let passing = write_executable(
            &directory,
            "gh-pass",
            r#"#!/bin/sh
if printf '%s' "$*" | grep check-runs >/dev/null; then
  echo '{"check_runs":[{"status":"completed","conclusion":"success"}]}'
else
  echo '{"state":"success","statuses":[{"state":"success"}]}'
fi
"#,
        );
        let failing = write_executable(
            &directory,
            "gh-fail",
            r#"#!/bin/sh
if printf '%s' "$*" | grep check-runs >/dev/null; then
  echo '{"check_runs":[{"status":"completed","conclusion":"failure"}]}'
else
  echo '{"state":"pending","statuses":[]}'
fi
"#,
        );
        let fact = Fact::CommitCi {
            repo: "yesme/hctl2".to_owned(),
            commit: "abc".to_owned(),
        };
        let ok = read_once(&fact, &ReaderContext::new(passing.into_os_string()));
        let ReadResult::Answer(ok) = ok else {
            panic!("successful checks must be terminal");
        };
        assert_eq!(ok.outcome, Outcome::Established);

        let bad = read_once(&fact, &ReaderContext::new(failing.into_os_string()));
        let ReadResult::Answer(bad) = bad else {
            panic!("failed checks must be terminal");
        };
        assert_eq!(bad.outcome, Outcome::NotEstablished);
        fs::remove_dir_all(directory).expect("fixture must be removed");
    }

    #[test]
    fn ref_advance_establishes_only_when_github_compare_status_is_ahead() {
        let directory = temporary_directory("ref");
        let ahead = write_executable(
            &directory,
            "gh-ahead",
            r#"#!/bin/sh
if printf '%s' "$*" | grep '/git/ref/' >/dev/null; then
  echo '{"object":{"sha":"def"}}'
else
  echo '{"status":"ahead"}'
fi
"#,
        );
        let diverged = write_executable(
            &directory,
            "gh-diverged",
            r#"#!/bin/sh
if printf '%s' "$*" | grep '/git/ref/' >/dev/null; then
  echo '{"object":{"sha":"def"}}'
else
  echo '{"status":"diverged"}'
fi
"#,
        );
        let fact = Fact::RefAdvanced {
            repo: "yesme/hctl2".to_owned(),
            reference: "heads/main".to_owned(),
            from: "abc".to_owned(),
        };
        let ok = read_once(&fact, &ReaderContext::new(ahead.into_os_string()));
        let ReadResult::Answer(ok) = ok else {
            panic!("ahead ref must be terminal");
        };
        assert_eq!(ok.outcome, Outcome::Established);

        let bad = read_once(&fact, &ReaderContext::new(diverged.into_os_string()));
        let ReadResult::Answer(bad) = bad else {
            panic!("diverged ref must be terminal");
        };
        assert_eq!(bad.outcome, Outcome::NotEstablished);
        fs::remove_dir_all(directory).expect("fixture must be removed");
    }

    #[test]
    fn missing_path_times_out_wrong_digest_stays_pending_directory_is_unreadable() {
        let directory = temporary_directory("path-outcomes");
        let missing = directory.join("missing");
        let file = directory.join("artifact");
        fs::write(&file, b"hctl2").expect("fixture must be written");
        let digest = super::lower_hex(&Sha256::digest(b"hctl2"));
        let context = ReaderContext::new("gh");

        let timeout = wait_until(
            Fact::PathDigest {
                path: missing,
                sha256: digest.clone(),
            },
            SystemTime::now(),
            &context,
        );
        assert_eq!(timeout.outcome, Outcome::Timeout);

        let established = read_once(
            &Fact::PathDigest {
                path: file.clone(),
                sha256: digest,
            },
            &context,
        );
        let ReadResult::Answer(established) = established else {
            panic!("matching digest must be terminal");
        };
        assert_eq!(established.outcome, Outcome::Established);

        let unreadable = read_once(
            &Fact::PathDigest {
                path: directory.clone(),
                sha256: "00".repeat(32),
            },
            &context,
        );
        let ReadResult::Answer(unreadable) = unreadable else {
            panic!("opening a directory must be terminal");
        };
        assert_eq!(unreadable.outcome, Outcome::Unreadable);
        fs::remove_dir_all(directory).expect("fixture must be removed");
    }

    #[test]
    fn invalid_arguments_are_unreadable_not_not_established() {
        let record = read_once(
            &Fact::PullRequestMerged {
                repo: "not-a-repo".to_owned(),
                number: 0,
            },
            &ReaderContext::new("gh"),
        );
        let ReadResult::Answer(record) = record else {
            panic!("invalid arguments must be terminal");
        };
        assert_eq!(record.outcome, Outcome::Unreadable);
        assert_eq!(record.evidence_level, "toolbox_readback");
    }

    #[test]
    fn concurrent_waiters_see_the_same_path_digest() {
        let directory = temporary_directory("concurrent");
        let path = directory.join("artifact");
        let digest = super::lower_hex(&Sha256::digest(b"hctl2"));
        let fact = Fact::PathDigest {
            path: path.clone(),
            sha256: digest,
        };
        let deadline = SystemTime::now() + Duration::from_secs(5);
        let first_fact = fact.clone();
        let first =
            thread::spawn(move || wait_until(first_fact, deadline, &ReaderContext::new("gh")));
        let second = thread::spawn(move || wait_until(fact, deadline, &ReaderContext::new("gh")));
        thread::sleep(Duration::from_millis(50));
        fs::write(&path, b"hctl2").expect("artifact must appear for both waiters");
        let first = first.join().expect("first waiter must finish");
        let second = second.join().expect("second waiter must finish");
        assert_eq!(first.outcome, Outcome::Established);
        assert_eq!(second.outcome, Outcome::Established);
        assert_eq!(first.evidence_level, "toolbox_readback");
        assert_eq!(second.evidence_level, "toolbox_readback");
        fs::remove_dir_all(directory).expect("fixture must be removed");
    }

    #[test]
    fn process_exit_uses_real_ps_for_an_unused_pid() {
        let unused = (10_000..50_000)
            .find(|pid| {
                Command::new("ps")
                    .args(["-p", &pid.to_string(), "-o", "pid="])
                    .output()
                    .map(|output| {
                        !output.status.success()
                            && output.stdout.is_empty()
                            && output.stderr.is_empty()
                    })
                    .unwrap_or(false)
            })
            .expect("an unused pid must exist");
        let record = wait_until(
            Fact::ProcessExited { pid: unused },
            SystemTime::now() + Duration::from_secs(1),
            &ReaderContext::new("gh"),
        );
        assert_eq!(record.outcome, Outcome::Established);
        assert_eq!(record.evidence_level, "toolbox_readback");
    }
}
