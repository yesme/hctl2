//! Read a remote ref back through Git itself: fetch it into a repository the caller owns and
//! report what the fetched objects say about one commit. Facts only — whether those facts
//! amount to a confirmed integration is the caller's decision, and no platform API answer
//! stands in for what Git can show.
//!
//! Credentials never travel in the URL or on the command line: when `HCTL2_GIT_USER` and
//! `HCTL2_GIT_TOKEN` are set (in the process environment, or handed to an in-process caller),
//! Git's per-process credential helper reads them; nothing is written to Git configuration.

use std::ffi::OsString;
use std::path::PathBuf;

use serde_json::json;

use crate::git::{Git, args};
use crate::repository::redact_remote_url;
use crate::{ToolError, ToolOutput, observed_at_unix_ms};

#[derive(Debug, clap::Args)]
pub(crate) struct Arguments {
    /// Repository (bare or not) that receives the fetched ref; the caller owns it.
    #[arg(long)]
    path: PathBuf,
    /// Remote to fetch from, as a URL without credentials.
    #[arg(long)]
    remote: String,
    /// Fully qualified ref on the remote (refs/heads/...); fetched under the same name here.
    #[arg(long = "ref")]
    reference: String,
    /// Full object ID the ref is expected to carry (a merge commit or the candidate itself).
    #[arg(long)]
    commit: String,
}

const CREDENTIAL_HELPER: &str = "!f() { if test \"$1\" = get; then printf 'username=%s\\npassword=%s\\n' \"$HCTL2_GIT_USER\" \"$HCTL2_GIT_TOKEN\"; fi; }; f";

pub(crate) fn run(git: &Git, input: &Arguments) -> Result<ToolOutput, ToolError> {
    if !input.reference.starts_with("refs/") || input.reference.contains("..") {
        return Err(ToolError::new(
            "HCTL2_TOOL_INVALID_ARGUMENT",
            "ref must be fully qualified (refs/...)",
        ));
    }
    if !is_object_id(&input.commit) {
        return Err(ToolError::new(
            "HCTL2_TOOL_INVALID_ARGUMENT",
            "commit must be a full object ID",
        ));
    }
    let repo = input.path.canonicalize().map_err(|error| {
        ToolError::new(
            "HCTL2_TOOL_REPOSITORY_PATH_INVALID",
            format!("cannot resolve {}: {error}", input.path.display()),
        )
    })?;
    let probe = git.invoke(&repo, &args(&["rev-parse", "--git-dir"]))?;
    if !probe.success() {
        return Err(ToolError::new(
            "HCTL2_TOOL_NOT_GIT_REPOSITORY",
            format!(
                "{} is not a Git repository: {}",
                repo.display(),
                probe.stderr()
            ),
        ));
    }

    // What the remote says the ref is, before any object lands here.
    let mut listing = credential_args(git);
    listing.extend(args(&[
        "ls-remote",
        "--refs",
        &input.remote,
        &input.reference,
    ]));
    let listing = git.checked(
        &repo,
        &listing,
        "HCTL2_TOOL_READBACK_REMOTE_UNREACHABLE",
        "list remote ref",
    )?;
    let remote_head = listing
        .stdout_text()?
        .lines()
        .filter_map(|line| {
            let (sha, name) = line.split_once('\t')?;
            (name == input.reference).then(|| sha.to_owned())
        })
        .next();

    let mut head = None;
    if remote_head.is_some() {
        let mut fetch = credential_args(git);
        fetch.extend(args(&[
            "fetch",
            "--quiet",
            "--no-tags",
            "--no-write-fetch-head",
            "--no-auto-gc",
            &input.remote,
            &format!("+{0}:{0}", input.reference),
        ]));
        git.checked(
            &repo,
            &fetch,
            "HCTL2_TOOL_READBACK_FETCH_FAILED",
            "fetch remote ref",
        )?;
        let local = git.checked(
            &repo,
            &args(&[
                "rev-parse",
                "--verify",
                &format!("{}^{{commit}}", input.reference),
            ]),
            "HCTL2_TOOL_GIT_INSPECTION_FAILED",
            "resolve fetched ref",
        )?;
        head = Some(local.stdout_text()?);
    }

    let head_tree = match &head {
        Some(head) => Some(tree_of(git, &repo, head)?),
        None => None,
    };
    let described = git.invoke(
        &repo,
        &args(&["log", "-1", "--format=%T %P", &input.commit, "--"]),
    )?;
    let commit_present = described.success();
    let (commit_tree, commit_parents) = if commit_present {
        let text = described.stdout_text()?;
        let mut fields = text.split_whitespace().map(str::to_owned);
        (fields.next(), fields.collect::<Vec<_>>())
    } else {
        (None, Vec::new())
    };
    let contains = match &head {
        Some(head) if commit_present => is_ancestor(git, &repo, &input.commit, head)?,
        _ => false,
    };

    Ok(ToolOutput::json(
        json!({
            "schema": "hctl2.readback.v1", "evidence_level": "unmediated",
            "observed_at_unix_ms": observed_at_unix_ms(), "operation": "readback",
            "git": {"path": git.executable(), "version": git.version()},
            "repository": repo, "remote": redact_remote_url(&input.remote),
            "ref": input.reference, "remote_head": remote_head,
            "head": head, "head_tree": head_tree,
            "commit": input.commit, "commit_present": commit_present,
            "commit_tree": commit_tree, "commit_parents": commit_parents,
            "contains": contains,
        }),
        0,
    ))
}

/// Git's own per-process credential helper, only when a credential is actually available.
fn credential_args(git: &Git) -> Vec<OsString> {
    let present = |name: &str| {
        git.extra_env().iter().any(|(n, _)| n == name) || std::env::var_os(name).is_some()
    };
    if present("HCTL2_GIT_USER") && present("HCTL2_GIT_TOKEN") {
        let mut values = args(&["-c", "credential.helper="]);
        values.push(OsString::from("-c"));
        values.push(OsString::from(format!(
            "credential.helper={CREDENTIAL_HELPER}"
        )));
        values
    } else {
        Vec::new()
    }
}

fn tree_of(git: &Git, repo: &std::path::Path, commit: &str) -> Result<String, ToolError> {
    git.checked(
        repo,
        &args(&["rev-parse", "--verify", &format!("{commit}^{{tree}}")]),
        "HCTL2_TOOL_GIT_INSPECTION_FAILED",
        "resolve tree",
    )?
    .stdout_text()
}

fn is_ancestor(
    git: &Git,
    repo: &std::path::Path,
    base: &str,
    head: &str,
) -> Result<bool, ToolError> {
    let output = git.invoke(repo, &args(&["merge-base", "--is-ancestor", base, head]))?;
    match output.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(ToolError::new(
            "HCTL2_TOOL_GIT_INSPECTION_FAILED",
            format!("could not inspect ancestry: {}", output.stderr()),
        )),
    }
}

fn is_object_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
