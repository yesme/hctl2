//! Standalone mechanical Git and SCM toolbox.
//!
//! P1 does not grant this process governance authority. When control arrives in P2, it will ask
//! the tool to execute already-persisted intents and will independently verify readback.

#![forbid(unsafe_code)]

use std::error::Error;
use std::ffi::OsString;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;
use std::time::{Duration, UNIX_EPOCH};

use clap::{CommandFactory, Parser, Subcommand};
use hctl2_facts::{Fact, Outcome, ReaderContext, wait_until};
use serde_json::Value;

mod archive;
mod git;
mod integration;
mod repository;
pub mod site_lock;
mod worktree;

// Next owner: content.rs. B1's immutable byte write/readback primitive
// belongs in content.rs: caller supplies ref/path/bytes; readback returns Git
// locator and digest. The CLI group will be `content`; no B1 commands exist yet.

/// Stable tool-local error code plus a human-readable explanation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolError {
    code: &'static str,
    message: String,
    not_established: bool,
    details: Value,
    recovery_action: Option<&'static str>,
}

impl ToolError {
    pub(crate) fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            not_established: false,
            details: Value::Null,
            recovery_action: None,
        }
    }

    pub(crate) fn not_established(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            not_established: true,
            ..Self::new(code, message)
        }
    }

    pub(crate) fn with_details(mut self, details: Value) -> Self {
        self.details = details;
        self
    }

    pub(crate) fn with_recovery_action(mut self, recovery_action: &'static str) -> Self {
        self.recovery_action = Some(recovery_action);
        self
    }

    fn readback(&self, git: &git::Git, operation: &str) -> ToolOutput {
        ToolOutput::json(
            serde_json::json!({
                "schema": "hctl2.tool-error.v1",
                "evidence_level": "toolbox_readback",
                "observed_at_unix_ms": observed_at_unix_ms(),
                "operation": operation,
                "git": { "path": git.executable(), "version": git.version() },
                "outcome": if self.not_established { "not_established" } else { "unreadable" },
                "error": {
                    "code": self.code,
                    "message": self.message,
                    "recovery_action": self.recovery_action,
                    "details": self.details,
                },
            }),
            if self.not_established { 3 } else { 4 },
        )
    }

    /// Returns the stable machine-readable error code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    /// Returns the human-readable explanation.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for ToolError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl Error for ToolError {}

/// Text and exit code produced by one tool invocation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolOutput {
    body: String,
    exit_code: u8,
}

impl ToolOutput {
    pub(crate) fn json(value: Value, exit_code: u8) -> Self {
        Self {
            body: value.to_string(),
            exit_code,
        }
    }

    /// Returns stdout without a trailing newline.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Returns the process exit code.
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        self.exit_code
    }
}

/// Installed executable name.
pub const PROGRAM_NAME: &str = "hctl2-tool";

const INVALID_ARGUMENT: &str = "HCTL2_TOOL_INVALID_ARGUMENT";
const SERIALIZATION_FAILED: &str = "HCTL2_TOOL_SERIALIZATION_FAILED";

#[derive(Debug, Parser)]
#[command(
    name = PROGRAM_NAME,
    version,
    about = "Mechanical Git/SCM toolbox for HCTL2",
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<ToolCommand>,
}

#[derive(Debug, Subcommand)]
enum ToolCommand {
    /// Wait for one external mechanical fact and print one JSON fact record.
    Wait(WaitArguments),
    /// Inspect a local Git repository without changing it.
    Repo(RepoArguments),
    /// Materialize or verify an isolated ChangeSet worktree.
    Worktree(WorktreeArguments),
    /// Snapshot, salvage, or remove an isolated ChangeSet worktree.
    Archive(ArchiveArguments),
    /// Integrate a caller-specified commit into a local ref using compare-and-swap.
    Integrate(integration::Arguments),
}

#[derive(Debug, clap::Args)]
struct RepoArguments {
    #[command(subcommand)]
    command: RepoCommand,
}

#[derive(Debug, Subcommand)]
enum RepoCommand {
    /// Read repository identities, worktrees, HEAD, a ref, and remotes.
    Inspect {
        /// Any directory in the repository to inspect.
        #[arg(long)]
        path: PathBuf,

        /// Optional Git ref or commit to resolve alongside HEAD.
        #[arg(long = "ref")]
        reference: Option<String>,
    },
}

#[derive(Debug, clap::Args)]
struct WorktreeArguments {
    #[command(subcommand)]
    command: WorktreeCommand,
}

#[derive(Debug, Subcommand)]
enum WorktreeCommand {
    /// Materialize one ChangeSet as an isolated worktree and branch.
    Materialize {
        /// Any directory in the source repository.
        #[arg(long)]
        repo: PathBuf,

        /// Directory below which the new worktree is placed.
        #[arg(long)]
        root: PathBuf,

        /// Caller-supplied stable ChangeSet reference.
        #[arg(long)]
        change_set_ref: String,

        /// Git commit from which the ChangeSet starts.
        #[arg(long)]
        baseline: String,
    },
    /// Verify the current worktree, branch, baseline ancestry, and dirtiness.
    Verify {
        /// Any directory in the source repository.
        #[arg(long)]
        repo: PathBuf,

        /// Caller-supplied stable ChangeSet reference.
        #[arg(long)]
        change_set_ref: String,
    },
}

#[derive(Debug, clap::Args)]
struct ArchiveArguments {
    #[command(subcommand)]
    command: ArchiveCommand,
}

#[derive(Debug, Subcommand)]
enum ArchiveCommand {
    /// Write tracked and untracked worktree contents to a reachable snapshot.
    Snapshot {
        /// Any directory in the source repository.
        #[arg(long)]
        repo: PathBuf,

        /// Caller-supplied stable ChangeSet reference.
        #[arg(long)]
        change_set_ref: String,
    },
    /// Remove a ChangeSet worktree after salvage, or discard it if confirmed.
    Remove {
        /// Any directory in the source repository.
        #[arg(long)]
        repo: PathBuf,

        /// Caller-supplied stable ChangeSet reference.
        #[arg(long)]
        change_set_ref: String,

        /// Delete without keeping a snapshot. Requires --confirm-discard.
        #[arg(long, requires = "confirm_discard")]
        discard_unarchived: bool,

        /// Must equal --change-set-ref to authorize --discard-unarchived.
        #[arg(long, requires = "discard_unarchived")]
        confirm_discard: Option<String>,

        /// Refuse salvage-removal when ignored files are present.
        #[arg(long)]
        reject_ignored: bool,
    },
}

#[derive(Debug, clap::Args)]
struct WaitArguments {
    /// Absolute deadline as seconds since the Unix epoch.
    #[arg(long)]
    deadline: u64,

    /// `gh` executable; defaults to `HCTL2_GH`, the packaged binary, or `PATH`.
    #[arg(long)]
    gh: Option<PathBuf>,

    #[command(subcommand)]
    fact: FactArguments,
}

#[derive(Debug, Subcommand)]
enum FactArguments {
    /// Wait until all GitHub checks and commit statuses succeed.
    CommitCi {
        #[arg(long)]
        repo: String,
        #[arg(long)]
        commit: String,
    },
    /// Wait until a GitHub pull request is merged.
    PrMerged {
        #[arg(long)]
        repo: String,
        #[arg(long)]
        number: u64,
    },
    /// Wait until a GitHub ref advances from a commit without diverging.
    RefAdvanced {
        #[arg(long)]
        repo: String,
        #[arg(long = "ref")]
        reference: String,
        #[arg(long)]
        from: String,
    },
    /// Wait until a file exists with the exact SHA-256.
    PathDigest {
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        sha256: String,
    },
    /// Wait until a local process ID no longer exists.
    ProcessExited {
        #[arg(long)]
        pid: u32,
    },
}

/// Runs one command and returns its stdout plus semantic exit code.
///
/// # Errors
///
/// Returns a stable [`ToolError`] when parsing or record serialization fails.
pub fn run(arguments: impl IntoIterator<Item = OsString>) -> Result<ToolOutput, ToolError> {
    let mut argv = vec![OsString::from(PROGRAM_NAME)];
    argv.extend(arguments);
    let cli = match Cli::try_parse_from(argv) {
        Ok(cli) => cli,
        Err(error)
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            return Ok(ToolOutput {
                body: error.to_string().trim_end().to_owned(),
                exit_code: 0,
            });
        }
        Err(error) => return Err(ToolError::new(INVALID_ARGUMENT, error.to_string().trim())),
    };

    let Some(command) = cli.command else {
        return Ok(ToolOutput {
            body: Cli::command()
                .render_help()
                .to_string()
                .trim_end()
                .to_owned(),
            exit_code: 0,
        });
    };

    match command {
        ToolCommand::Wait(arguments) => run_wait(arguments),
        ToolCommand::Integrate(arguments) => {
            run_git_command("integrate", |git| integration::run(git, &arguments))
        }
        ToolCommand::Repo(arguments) => match arguments.command {
            RepoCommand::Inspect { path, reference } => run_git_command("repo_inspect", |git| {
                repository::inspect(git, path, reference)
            }),
        },
        ToolCommand::Worktree(arguments) => match arguments.command {
            WorktreeCommand::Materialize {
                repo,
                root,
                change_set_ref,
                baseline,
            } => {
                worktree::validate_change_set_ref(&change_set_ref)?;
                run_git_command("worktree_materialize", |git| {
                    worktree::materialize(git, repo, root, change_set_ref, baseline)
                })
            }
            WorktreeCommand::Verify {
                repo,
                change_set_ref,
            } => {
                worktree::validate_change_set_ref(&change_set_ref)?;
                run_git_command("worktree_verify", |git| {
                    worktree::verify(git, repo, change_set_ref)
                })
            }
        },
        ToolCommand::Archive(arguments) => match arguments.command {
            ArchiveCommand::Snapshot {
                repo,
                change_set_ref,
            } => {
                worktree::validate_change_set_ref(&change_set_ref)?;
                run_git_command("archive_snapshot", |git| {
                    archive::snapshot(git, repo, change_set_ref)
                })
            }
            ArchiveCommand::Remove {
                repo,
                change_set_ref,
                discard_unarchived,
                confirm_discard,
                reject_ignored,
            } => {
                worktree::validate_change_set_ref(&change_set_ref)?;
                run_git_command("archive_remove", |git| {
                    archive::remove(
                        git,
                        repo,
                        change_set_ref,
                        discard_unarchived,
                        confirm_discard,
                        reject_ignored,
                    )
                })
            }
        },
    }
}

// Startup/usage errors remain stderr errors in main; every Git observation has
// one JSON readback, including rejected preconditions (3) and unreadable state (4).
fn run_git_command(
    operation: &str,
    action: impl FnOnce(&git::Git) -> Result<ToolOutput, ToolError>,
) -> Result<ToolOutput, ToolError> {
    let git = git::Git::discover()?;
    Ok(action(&git).unwrap_or_else(|error| error.readback(&git, operation)))
}

fn run_wait(arguments: WaitArguments) -> Result<ToolOutput, ToolError> {
    let fact = match arguments.fact {
        FactArguments::CommitCi { repo, commit } => Fact::CommitCi { repo, commit },
        FactArguments::PrMerged { repo, number } => Fact::PullRequestMerged { repo, number },
        FactArguments::RefAdvanced {
            repo,
            reference,
            from,
        } => Fact::RefAdvanced {
            repo,
            reference,
            from,
        },
        FactArguments::PathDigest { path, sha256 } => Fact::PathDigest { path, sha256 },
        FactArguments::ProcessExited { pid } => Fact::ProcessExited { pid },
    };
    let gh = arguments.gh.unwrap_or_else(resolve_gh);
    let deadline = UNIX_EPOCH
        .checked_add(Duration::from_secs(arguments.deadline))
        .ok_or_else(|| ToolError::new(INVALID_ARGUMENT, "deadline is outside SystemTime range"))?;
    let record = wait_until(fact, deadline, &ReaderContext::new(gh.into_os_string()));
    let exit_code = match record.outcome {
        Outcome::Established => 0,
        Outcome::NotEstablished => 3,
        Outcome::Unreadable => 4,
        Outcome::Timeout => 5,
    };
    let body = serde_json::to_string(&record)
        .map_err(|error| ToolError::new(SERIALIZATION_FAILED, error.to_string()))?;
    Ok(ToolOutput { body, exit_code })
}

fn resolve_gh() -> PathBuf {
    if let Some(path) = std::env::var_os("HCTL2_GH") {
        return PathBuf::from(path);
    }
    if let Ok(executable) = std::env::current_exe()
        && let Some(payload_root) = executable.parent().and_then(std::path::Path::parent)
    {
        let packaged = payload_root.join("libexec/hctl2/gh");
        if packaged.is_file() {
            return packaged;
        }
    }
    PathBuf::from("gh")
}

pub(crate) fn observed_at_unix_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis())
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::run;

    #[test]
    fn reports_version() {
        let output = run([OsString::from("--version")]).expect("version must parse");

        assert_eq!(
            output.body(),
            format!("hctl2-tool {}", env!("CARGO_PKG_VERSION"))
        );
        assert_eq!(output.exit_code(), 0);
    }

    #[test]
    fn rejects_unknown_arguments_with_a_stable_code() {
        let error = run([OsString::from("not-a-command")]).expect_err("argument must be rejected");

        assert_eq!(error.code(), "HCTL2_TOOL_INVALID_ARGUMENT");
    }

    #[test]
    fn timed_out_fact_is_one_structured_answer() {
        let output = run([
            OsString::from("wait"),
            OsString::from("--deadline"),
            OsString::from("0"),
            OsString::from("path-digest"),
            OsString::from("--path"),
            OsString::from("missing"),
            OsString::from("--sha256"),
            OsString::from("00".repeat(32)),
        ])
        .expect("wait must return a record");

        assert_eq!(output.exit_code(), 5);
        let record: serde_json::Value =
            serde_json::from_str(output.body()).expect("answer must be JSON");
        assert_eq!(record["outcome"], "timeout");
        assert_eq!(record["evidence_level"], "toolbox_readback");
    }

    fn unix_deadline_secs_from_now(seconds: u64) -> OsString {
        use std::time::{SystemTime, UNIX_EPOCH};

        OsString::from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock must follow the epoch")
                .as_secs()
                .saturating_add(seconds)
                .to_string(),
        )
    }

    #[test]
    fn wait_exit_codes_cover_all_four_outcomes() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        use std::time::{SystemTime, UNIX_EPOCH};

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must follow the epoch")
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("hctl2-tool-wait-{}-{nanos}", std::process::id()));
        fs::create_dir(&directory).expect("fixture directory");
        let artifact = directory.join("artifact");
        fs::write(&artifact, b"hctl2").expect("artifact");
        let digest = "84afea2356518c906764be570e37420d2c9fa9ab3cab9f2ef801149822298f25";

        let established = run([
            OsString::from("wait"),
            OsString::from("--deadline"),
            unix_deadline_secs_from_now(5),
            OsString::from("path-digest"),
            OsString::from("--path"),
            artifact.into_os_string(),
            OsString::from("--sha256"),
            OsString::from(digest),
        ])
        .expect("established wait");
        assert_eq!(established.exit_code(), 0);
        let record: serde_json::Value = serde_json::from_str(established.body()).expect("json");
        assert_eq!(record["outcome"], "established");

        let gh_closed = directory.join("gh-closed");
        fs::write(
            &gh_closed,
            "#!/bin/sh\necho '{\"mergedAt\":null,\"state\":\"CLOSED\",\"url\":\"u\",\"headRefOid\":\"abc\"}'\n",
        )
        .expect("gh fixture");
        fs::set_permissions(&gh_closed, fs::Permissions::from_mode(0o755)).expect("chmod");
        let not_established = run([
            OsString::from("wait"),
            OsString::from("--deadline"),
            unix_deadline_secs_from_now(5),
            OsString::from("--gh"),
            gh_closed.into_os_string(),
            OsString::from("pr-merged"),
            OsString::from("--repo"),
            OsString::from("yesme/hctl2"),
            OsString::from("--number"),
            OsString::from("82"),
        ])
        .expect("not-established wait");
        assert_eq!(not_established.exit_code(), 3);

        let gh_broken = directory.join("gh-broken");
        fs::write(&gh_broken, "#!/bin/sh\necho broken >&2\nexit 1\n").expect("gh fixture");
        fs::set_permissions(&gh_broken, fs::Permissions::from_mode(0o755)).expect("chmod");
        let unreadable = run([
            OsString::from("wait"),
            OsString::from("--deadline"),
            unix_deadline_secs_from_now(5),
            OsString::from("--gh"),
            gh_broken.into_os_string(),
            OsString::from("pr-merged"),
            OsString::from("--repo"),
            OsString::from("yesme/hctl2"),
            OsString::from("--number"),
            OsString::from("82"),
        ])
        .expect("unreadable wait");
        assert_eq!(unreadable.exit_code(), 4);

        let timeout = run([
            OsString::from("wait"),
            OsString::from("--deadline"),
            OsString::from("0"),
            OsString::from("path-digest"),
            OsString::from("--path"),
            OsString::from("missing"),
            OsString::from("--sha256"),
            OsString::from("00".repeat(32)),
        ])
        .expect("timeout wait");
        assert_eq!(timeout.exit_code(), 5);
        fs::remove_dir_all(directory).expect("fixture must be removed");
    }

    #[test]
    fn wait_has_no_flag_to_force_an_outcome_from_restatement() {
        let error = run([
            OsString::from("wait"),
            OsString::from("--deadline"),
            OsString::from("0"),
            OsString::from("--outcome"),
            OsString::from("established"),
            OsString::from("pr-merged"),
            OsString::from("--repo"),
            OsString::from("yesme/hctl2"),
            OsString::from("--number"),
            OsString::from("1"),
        ])
        .expect_err("restatement override must not parse");
        assert_eq!(error.code(), "HCTL2_TOOL_INVALID_ARGUMENT");
    }
}
