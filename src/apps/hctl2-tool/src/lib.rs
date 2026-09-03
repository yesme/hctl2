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

/// Stable tool-local error code plus a human-readable explanation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolError {
    code: &'static str,
    message: String,
}

impl ToolError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
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
    }
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
}
