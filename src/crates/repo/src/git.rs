//! Native Git is the only repository engine. Inspection never writes the input copy.
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use crate::{LocalInput, LocalSnapshot, Result, reject};

/// Bound child lifetime/output without a shell. A timeout is never evidence of failure to write.
pub fn run(command: &mut Command, input: Option<Vec<u8>>) -> Result<Output> {
    command
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|_| {
        reject(
            "PROVIDER_UNAVAILABLE",
            "cannot start required binary",
            "check_installation",
        )
    })?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let read = |pipe: Box<dyn Read + Send>| {
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let result = pipe
                .take(16 * 1024 * 1024 + 1)
                .read_to_end(&mut bytes)
                .map(|_| bytes);
            let _ = sender.send(result);
        });
        receiver
    };
    let out = read(Box::new(stdout));
    let err = read(Box::new(stderr));
    if let Some(input) = input {
        let mut stdin = child.stdin.take().unwrap();
        std::thread::spawn(move || {
            let _ = stdin.write_all(&input);
        });
    }
    let deadline = Instant::now() + Duration::from_secs(90);
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(reject(
                "RESULT_UNKNOWN",
                "provider command timed out",
                "read_back_original_intent",
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let stdout = out
        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
        .map_err(|_| {
            reject(
                "RESULT_UNKNOWN",
                "stdout did not close",
                "read_back_original_intent",
            )
        })??;
    let stderr = err
        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
        .map_err(|_| {
            reject(
                "RESULT_UNKNOWN",
                "stderr did not close",
                "read_back_original_intent",
            )
        })??;
    if stdout.len() > 16 * 1024 * 1024 || stderr.len() > 16 * 1024 * 1024 {
        return Err(reject(
            "OUTPUT_LIMIT",
            "provider output exceeded limit",
            "narrow_input",
        ));
    }
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

pub struct Git {
    executable: PathBuf,
}
impl Git {
    pub fn discover() -> Result<Self> {
        let name = std::env::var_os("HCTL2_GIT").unwrap_or_else(|| "git".into());
        let executable = foundation::git::resolve_executable(&name).ok_or_else(|| {
            reject(
                "GIT_UNAVAILABLE",
                "host Git not found",
                "install_git_2_39_or_newer",
            )
        })?;
        let banner = run(Self::isolated_command(&executable).arg("--version"), None)?;
        let (major, minor, _) =
            foundation::git::parse_version(&String::from_utf8_lossy(&banner.stdout))
                .ok_or_else(|| reject("GIT_VERSION", "cannot parse Git version", "check_git"))?;
        if !foundation::git::version_supported(major, minor) {
            return Err(reject(
                "GIT_VERSION",
                "Git 2.39 or newer required",
                "upgrade_git",
            ));
        }
        Ok(Self { executable })
    }
    fn isolated_command(executable: &Path) -> Command {
        let mut cmd = foundation::git::sanitized_command(executable);
        // These are registration/delivery subprocesses, not interactive user Git.
        cmd.env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env_remove("GIT_TEMPLATE_DIR")
            .env_remove("GIT_CURL_VERBOSE")
            .env_remove("GIT_ASKPASS")
            .env_remove("SSH_ASKPASS");
        for (name, _) in std::env::vars_os() {
            if name.to_string_lossy().starts_with("GIT_TRACE") {
                cmd.env_remove(name);
            }
        }
        cmd.args([
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.fsmonitor=false",
        ]);
        cmd
    }
    fn command(&self, path: &Path) -> Command {
        let mut cmd = Self::isolated_command(&self.executable);
        cmd.arg("-C").arg(path);
        cmd
    }
    fn text(&self, path: &Path, args: &[&str]) -> Result<String> {
        let output = run(self.command(path).args(args), None)?;
        if !output.status.success() {
            return Err(reject(
                "GIT_READ_FAILED",
                format!("Git {} failed; path is not treated as pure local", args[0]),
                "inspect_local_path",
            ));
        }
        String::from_utf8(output.stdout).map_err(|_| {
            reject(
                "GIT_FORMAT",
                "Git returned non UTF-8 metadata",
                "inspect_local_path",
            )
        })
    }
    pub fn inspect(&self, input: &LocalInput) -> Result<LocalSnapshot> {
        if input.machine != "control" {
            return Err(reject(
                "MACHINE_UNREACHABLE",
                "this CLI accepts paths on the control machine only",
                "run_on_source_machine",
            ));
        }
        let path = input.path.canonicalize().map_err(|_| {
            reject(
                "LOCAL_READ_FAILED",
                "local path cannot be read",
                "correct_local_path",
            )
        })?;
        self.text(&path, &["rev-parse", "--git-dir"])?;
        let remotes = self.text(&path, &["remote"])?;
        let mut remote_urls = BTreeMap::new();
        for remote in remotes.lines() {
            // Multiple URLs are evidence too; refuse ambiguity rather than silently discard one.
            let urls = self.text(&path, &["remote", "get-url", "--all", remote])?;
            if urls.lines().count() != 1 {
                return Err(reject(
                    "REMOTE_AMBIGUOUS",
                    "remote has multiple URLs",
                    "inspect_local_remotes",
                ));
            }
            remote_urls.insert(remote.into(), urls.trim().into());
        }
        let symbolic = run(
            self.command(&path).args(["symbolic-ref", "-q", "HEAD"]),
            None,
        )?;
        let branch = if symbolic.status.success() {
            String::from_utf8_lossy(&symbolic.stdout).trim().to_owned()
        } else if symbolic.status.code() == Some(1) {
            "refs/heads/main".into()
        } else {
            return Err(reject(
                "GIT_READ_FAILED",
                "cannot inspect HEAD",
                "inspect_local_path",
            ));
        };
        let head = run(
            self.command(&path)
                .args(["rev-parse", "--verify", "--quiet", "HEAD^{commit}"]),
            None,
        )?;
        let mut refs: BTreeMap<String, String> = BTreeMap::new();
        if head.status.success() {
            self.validate_ref(&path, &branch)?;
            if !symbolic.status.success() && !input.extra_refs.contains(&branch) {
                let existing =
                    self.text(&path, &["for-each-ref", "--format=%(objectname)", &branch])?;
                if !existing.trim().is_empty()
                    && existing.trim() != String::from_utf8_lossy(&head.stdout).trim()
                {
                    return Err(reject(
                        "DETACHED_HEAD_AMBIGUOUS",
                        "detached HEAD differs from existing main; check out the intended branch or explicitly select refs/heads/main",
                        "select_initial_branch",
                    ));
                }
            }
            refs.insert(
                branch.clone(),
                String::from_utf8_lossy(&head.stdout).trim().into(),
            );
        } else {
            // A symbolic, nonexistent branch proves unborn HEAD; corruption and detached failure do not.
            if !symbolic.status.success()
                || !self
                    .text(&path, &["for-each-ref", "--format=%(refname)", &branch])?
                    .trim()
                    .is_empty()
            {
                return Err(reject(
                    "GIT_READ_FAILED",
                    "HEAD is not a readable commit or unborn branch",
                    "inspect_local_path",
                ));
            }
        }
        for reference in &input.extra_refs {
            self.validate_ref(&path, reference)?;
            self.text(
                &path,
                &["rev-parse", "--verify", &format!("{reference}^{{commit}}")],
            )?;
            let sha = self.text(&path, &["rev-parse", "--verify", reference])?;
            refs.insert(reference.clone(), sha.trim().into());
        }
        let mut governance_paths = Vec::new();
        for sha in refs.values() {
            // Reachable history matters too: deleting a material in HEAD doesn't unpublish its parent.
            let paths = self.text(
                &path,
                &[
                    "log",
                    "--format=",
                    "--name-only",
                    sha,
                    "--",
                    ".memo",
                    ".hctl2",
                ],
            )?;
            governance_paths.extend(paths.lines().filter(|s| !s.is_empty()).map(str::to_owned));
        }
        governance_paths.sort();
        governance_paths.dedup();
        Ok(LocalSnapshot {
            path,
            remotes: remote_urls,
            refs,
            head_branch: branch,
            governance_paths,
        })
    }
    fn validate_ref(&self, path: &Path, reference: &str) -> Result<()> {
        if !(reference.starts_with("refs/heads/") || reference.starts_with("refs/tags/"))
            || reference.starts_with("refs/heads/hctl2/")
            || reference.starts_with("refs/tags/hctl2/")
        {
            return Err(reject(
                "PRIVATE_REF",
                "private retention and unadmitted change refs are not initial publication refs",
                "select_public_refs",
            ));
        }
        self.text(path, &["check-ref-format", reference])
            .map(|_| ())
    }
    pub fn recheck(&self, input: &LocalInput, snapshot: &LocalSnapshot) -> Result<()> {
        if self.inspect(input)? != *snapshot {
            return Err(reject(
                "LOCAL_INPUT_CHANGED",
                "local evidence changed since preview",
                "preview_again",
            ));
        }
        Ok(())
    }

    /// Fresh, independent Git copy. Only explicitly frozen refs are fetched; never --mirror.
    pub fn copy(&self, snapshot: &LocalSnapshot, destination: &Path) -> Result<PathBuf> {
        std::fs::create_dir_all(destination)?;
        self.text(
            destination,
            &[
                "init",
                "--template=",
                "--initial-branch",
                snapshot.head_branch.trim_start_matches("refs/heads/"),
            ],
        )?;
        for (reference, sha) in &snapshot.refs {
            let output = run(
                self.command(destination)
                    .args([
                        "fetch",
                        "--update-head-ok",
                        "--no-tags",
                        "--no-write-fetch-head",
                    ])
                    .arg(&snapshot.path)
                    .arg(format!("{sha}:{reference}")),
                None,
            )?;
            if !output.status.success() {
                return Err(reject(
                    "COPY_FAILED",
                    "cannot materialize frozen input",
                    "restore_frozen_source",
                ));
            }
        }
        if !snapshot.refs.is_empty() {
            self.text(
                destination,
                &[
                    "checkout",
                    snapshot.head_branch.trim_start_matches("refs/heads/"),
                ],
            )?;
        }
        Ok(destination.to_owned())
    }

    /// The caller supplies a process-scoped credential helper. No token enters URL/config/argv.
    pub fn delivered(
        &self,
        path: &Path,
        snapshot: &LocalSnapshot,
        url: &str,
        credential: Option<(&str, &str)>,
    ) -> Result<bool> {
        let actual = self.remote_refs(path, url, credential)?;
        Ok(snapshot.refs.iter().all(|(r, s)| actual.get(r) == Some(s)))
    }

    /// Deliver only absent refs, then verify the complete frozen set.
    pub fn deliver(
        &self,
        path: &Path,
        snapshot: &LocalSnapshot,
        url: &str,
        credential: Option<(&str, &str)>,
    ) -> Result<()> {
        let remote_refs = self.remote_refs(path, url, credential)?;
        let mut missing = Vec::new();
        for (reference, sha) in &snapshot.refs {
            match remote_refs.get(reference) {
                Some(current) if current == sha => {}
                Some(_) => {
                    return Err(reject(
                        "INITIAL_REF_CONFLICT",
                        "target branch already has a different version; no force overwrite",
                        "inspect_platform_repository",
                    ));
                }
                None => missing.push((reference, sha)),
            }
        }
        if !missing.is_empty() {
            let mut cmd = self.command(path);
            credentials(&mut cmd, credential);
            cmd.args(["push", "--porcelain", "--atomic", "--no-verify"]);
            // Empty expected old ref means create-only CAS, never permission to overwrite.
            for (reference, _) in &missing {
                cmd.arg(format!("--force-with-lease={reference}:"));
            }
            cmd.arg(url);
            for (reference, sha) in &missing {
                cmd.arg(format!("{sha}:{reference}"));
            }
            let _output = run(&mut cmd, None)?;
        }
        let actual = self.remote_refs(path, url, credential)?;
        if snapshot.refs.iter().any(|(r, s)| actual.get(r) != Some(s)) {
            return Err(reject(
                "RESULT_UNKNOWN",
                "initial Git delivery not confirmed",
                "read_back_original_intent",
            ));
        }
        Ok(())
    }
    fn remote_refs(
        &self,
        path: &Path,
        url: &str,
        credential: Option<(&str, &str)>,
    ) -> Result<BTreeMap<String, String>> {
        let mut cmd = self.command(path);
        // ls-remote needs no local repository. Do not discover configuration in a
        // caller-selected directory (which may itself live inside another worktree).
        cmd.arg("--git-dir=/dev/null");
        credentials(&mut cmd, credential);
        let output = run(cmd.args(["ls-remote", "--refs", url]), None)?;
        if !output.status.success() {
            return Err(reject(
                "PLATFORM_UNAVAILABLE",
                "Git target could not be read",
                "retry_registration",
            ));
        }
        let mut refs = BTreeMap::new();
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let (sha, reference) = line
                .split_once('\t')
                .ok_or_else(|| reject("GIT_FORMAT", "invalid ls-remote output", "retry_read"))?;
            refs.insert(reference.into(), sha.into());
        }
        Ok(refs)
    }
    pub fn switch_remote(&self, snapshot: &LocalSnapshot, url: &str) -> Result<()> {
        let (name, old) = snapshot
            .remotes
            .iter()
            .next()
            .ok_or_else(|| reject("IN_PLACE_SCOPE", "no selected remote", "preview_again"))?;
        let current = self.text(&snapshot.path, &["remote", "get-url", name])?;
        if current.trim() == url {
            return Ok(());
        }
        if current.trim() != old {
            return Err(reject(
                "LOCAL_INPUT_CHANGED",
                "remote changed since confirmation",
                "inspect_local_remotes",
            ));
        }
        self.text(&snapshot.path, &["remote", "set-url", name, url])?;
        if self
            .text(&snapshot.path, &["remote", "get-url", name])?
            .trim()
            != url
        {
            return Err(reject(
                "RESULT_UNKNOWN",
                "remote change not confirmed",
                "inspect_local_remotes",
            ));
        }
        Ok(())
    }
}

fn credentials(cmd: &mut Command, credential: Option<(&str, &str)>) {
    if let Some((user, token)) = credential {
        // Git's native per-process helper, not a generated executable or global config write.
        cmd.args(["-c", "credential.helper=", "-c", "credential.helper=!f() { if test \"$1\" = get; then printf 'username=%s\\npassword=%s\\n' \"$HCTL2_GIT_USER\" \"$HCTL2_GIT_TOKEN\"; fi; }; f"])
            .env("HCTL2_GIT_USER", user).env("HCTL2_GIT_TOKEN", token);
    }
}
