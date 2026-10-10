//! Agency-local work copies. Authority is frozen in Spec and delivered through Bundle.
//! Neither this module nor a harness publishes or admits a ChangeSet Revision.
use agency_proto::{
    ExecutionSpec, FrozenRef, PortError, Result, canonical,
    context::{Bundle, Delivery},
    hash,
};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

#[derive(Clone)]
pub(crate) struct WorkCopy {
    pub cwd: PathBuf,
    pub change_set: String,
    pub baseline: String,
    pub lease: FrozenRef,
    pub generation: u64,
    pub seal_key: String,
}

pub(crate) fn prepare(
    spec: &ExecutionSpec,
    bundle: &Bundle,
    exec: &Path,
    tenant: &Path,
    credential_root: &Path,
) -> Result<Option<WorkCopy>> {
    if spec.write_lease.is_none() && spec.review_publish_policy.is_none() {
        crate::standby::readonly(spec)?;
        return Ok(None);
    }
    let fail = || {
        PortError::new(
            "WRITE_BOUNDARY_MISMATCH",
            "write material differs from this Execution Spec",
            "rebuild_bundle",
        )
    };
    let (Some(repo), Some(base), Some(lease), Some(policy)) = (
        &spec.repo,
        &spec.base,
        &spec.write_lease,
        &spec.review_publish_policy,
    ) else {
        return Err(fail());
    };
    if !spec.permissions.iter().any(|p| p == "git.write")
        || !spec.permissions.iter().any(|p| p == "context.read")
        || spec.permissions.iter().any(|p| {
            !matches!(
                p.as_str(),
                "context.read" | "git.read" | "git.write" | "terminal.observe"
            )
        })
    {
        return Err(PortError::new(
            "PERMISSION_DENIED",
            "write requires context.read and git.write within the frozen content scope",
            "narrow_permissions",
        ));
    }
    bundle.validate_delivery()?;
    let name = format!("write-boundary/{}", spec.owner.id);
    let entries: Vec<_> = bundle
        .entries
        .iter()
        .filter(|e| e.source.id == name)
        .collect();
    if entries.len() != 1 || !entries[0].required {
        return Err(PortError::new(
            "WRITE_MATERIAL_REQUIRED",
            "one required write-boundary Bundle entry must be delivered",
            "deliver_write_boundary",
        ));
    }
    let bytes = match &entries[0].delivery {
        Delivery::Inline { bytes } | Delivery::Pointer { bytes, .. } => bytes,
        Delivery::Recall { .. } => return Err(fail()),
    };
    let material: Value = serde_json::from_slice(bytes)?;
    let set = &material["lease"]["pending"];
    let mut active = set["lease"].clone();
    if !active.is_object() {
        return Err(fail());
    }
    active["state"] = json!("active");
    let change_set = set["change_set_id"].as_str().ok_or_else(fail)?;
    let generation = active["generation"]
        .as_u64()
        .filter(|n| *n > 0)
        .ok_or_else(fail)?;
    if set["repo_id"] != repo.id
        || set["baseline_commit"] != *base
        || active["lease_id"] != lease.id
        || generation.to_string() != lease.revision
        || hash(&canonical(&active)?) != lease.digest
        || active["holder"]["kind"] != "invocation"
        || active["holder"]["invocation_id"] != spec.owner.id
        || active["holder"]["invocation_version"] != spec.owner.generation
        || material["review_publish_policy"] != serde_json::to_value(policy)?
        || hash(&canonical(&material["publication_target"])?) != policy.digest
        || material["publication_target"]["repo_id"] != repo.id
        || material["publication_target"]["binding_version"] != set["binding_version"]
        || material["authorization"] != "publish_for_review_not_integration"
    {
        return Err(fail());
    }
    sha(base)?;
    // #405's WritePreview is Context material, not permission to read control storage.
    // Never use the service cwd or infer a repository from a Task.
    if material["repo_local_machine"] != "control" {
        return Err(PortError::new(
            "WRITE_MATERIAL_REQUIRED",
            "only a locally reachable control Repo is supported",
            "deliver_write_boundary",
        ));
    }
    let source = material["repo_local_path"]
        .as_str()
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .ok_or_else(|| {
            PortError::new(
                "WRITE_MATERIAL_REQUIRED",
                "Bundle must deliver the Repo local path",
                "deliver_write_boundary",
            )
        })?;
    let target = material["publication_target"]["target_branch"]
        .as_str()
        .filter(|s| !s.trim().is_empty() && s.trim() == *s)
        .ok_or_else(|| {
            PortError::new(
                "WRITE_MATERIAL_REQUIRED",
                "Bundle must deliver the frozen publication target",
                "deliver_write_boundary",
            )
        })?;
    if material["objective"]
        .as_str()
        .is_none_or(|s| s.trim().is_empty())
    {
        return Err(PortError::new(
            "WRITE_MATERIAL_REQUIRED",
            "Bundle must deliver the requested objective",
            "deliver_write_boundary",
        ));
    }
    if change_set.is_empty()
        || change_set
            .bytes()
            .any(|b| !b.is_ascii_alphanumeric() && b != b'-' && b != b'_')
    {
        return Err(fail());
    }
    let source = source.canonicalize()?;
    let parent = exec
        .parent()
        .ok_or_else(|| PortError::invalid("execution parent missing"))?;
    let root = parent
        .join("write-worktrees")
        .join(hash(&canonical(&json!([tenant, change_set]))?));
    crate::storage::private_dir(&root)?;
    let credential_root = credential_root.canonicalize()?;
    if crate::confine::allowed_tree_contains_credential(&root.canonicalize()?, &credential_root) {
        return Err(PortError::new(
            "EXECUTION_ROOT_UNSAFE",
            "write worktree overlaps the credential root",
            "choose_execution_directory",
        ));
    }
    let cwd = root.join(change_set);
    let descriptor = canonical(
        &json!({"repo_id":repo.id,"source":source,"change_set":change_set,"baseline":base}),
    )?;
    static COPIES: OnceLock<Mutex<()>> = OnceLock::new();
    let _guard = COPIES
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| PortError::invalid("work copy lock poisoned"))?;
    let record = root.join("boundary.json");
    if record.exists() && fs::read(&record)? != descriptor {
        return Err(PortError::new(
            "WRITE_WORKTREE_CONFLICT",
            "the preserved ChangeSet worktree belongs to another Repo or baseline",
            "inspect_preserved_worktree",
        ));
    }
    if !cwd.exists() {
        let report = tool(
            &root,
            &[
                "worktree",
                "materialize",
                "--repo",
                source.to_str().ok_or_else(fail)?,
                "--root",
                root.to_str().ok_or_else(fail)?,
                "--change-set-ref",
                change_set,
                "--baseline",
                base,
            ],
        )?;
        let materialized = report["worktree"]["path"].as_str().map(PathBuf::from);
        if report["schema"] != "hctl2.worktree.v1"
            || report["outcome"] != "established"
            || report["change_set_ref"] != change_set
            || report["baseline_commit_sha"] != *base
            || materialized.as_ref().and_then(|p| p.canonicalize().ok())
                != Some(cwd.canonicalize()?)
        {
            return Err(PortError::new(
                "WRITE_WORKTREE_CONFLICT",
                "tool materialized a different ChangeSet worktree or baseline",
                "inspect_preserved_worktree",
            ));
        }
    }
    let actual = git(&cwd, &["rev-parse", "--verify", "HEAD^{commit}"])?;
    let symbolic = git_optional(&cwd, &["symbolic-ref", "-q", "HEAD"])?;
    if actual.trim() != base
        || (symbolic.status.success()
            && String::from_utf8_lossy(&symbolic.stdout).trim()
                != format!("refs/heads/hctl2/changeset/{change_set}"))
        || (!symbolic.status.success() && symbolic.status.code() != Some(1))
    {
        return Err(PortError::new(
            "WRITE_WORKTREE_CONFLICT",
            "preserved worktree must remain at its frozen baseline and ChangeSet branch or detached",
            "inspect_preserved_worktree",
        ));
    }
    let common = git_common_dir(&cwd)?;
    if common != git_common_dir(&source)? {
        return Err(PortError::new(
            "WRITE_WORKTREE_CONFLICT",
            "worktree is no longer attached to the Bundle's Repo",
            "inspect_preserved_worktree",
        ));
    }
    // Publication policy may differ on a new authorized lease. Repo and baseline
    // never reset an existing worktree; unsealed edits survive cancellation.
    let _ = target;
    fs::write(record, descriptor)?;
    Ok(Some(WorkCopy {
        cwd: cwd.canonicalize()?,
        change_set: change_set.into(),
        baseline: base.clone(),
        lease: lease.clone(),
        generation,
        seal_key: format!("agency-{}", hash(&canonical(spec)?)),
    }))
}

pub(crate) fn git_common_dir(path: &Path) -> Result<PathBuf> {
    PathBuf::from(
        git(
            path,
            &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        )?
        .trim(),
    )
    .canonicalize()
    .map_err(Into::into)
}

pub(crate) fn task_text(bundle: &Bundle, copy: Option<&WorkCopy>) -> Result<String> {
    let mut text = crate::launch::task_text(bundle)?;
    if let Some(copy) = copy {
        text.push_str(&format!("\nAgency ChangeSet: {}\nBaseline: {}\nWrite lease: {} generation {}\nAgency execution directory: {}\nEdit and test this ChangeSet in this materialized worktree. Leave changes uncommitted for Agency sealing. Do not push, publish reviews, obtain Git credentials, or change harness global configuration.\n", copy.change_set, copy.baseline, copy.lease.id, copy.generation, copy.cwd.display()));
    }
    Ok(text)
}

fn return_turn(copy: &WorkCopy) -> Result<(Vec<u8>, Value)> {
    let symbolic = git_optional(&copy.cwd, &["symbolic-ref", "-q", "HEAD"])?;
    if symbolic.status.code() == Some(1) {
        // Recognize a detached P1 copy without weakening archive::require_worktree.
        // Restore only the existing branch at the same frozen baseline; no reset/ref write.
        let branch = format!("hctl2/changeset/{}", copy.change_set);
        if git(&copy.cwd, &["rev-parse", "--verify", "HEAD^{commit}"])?.trim() != copy.baseline
            || git(
                &copy.cwd,
                &[
                    "rev-parse",
                    "--verify",
                    &format!("refs/heads/{branch}^{{commit}}"),
                ],
            )?
            .trim()
                != copy.baseline
        {
            return Err(PortError::new(
                "WRITE_WORKTREE_CONFLICT",
                "detached worktree or ChangeSet branch differs from the frozen baseline",
                "inspect_preserved_worktree",
            ));
        }
        git(&copy.cwd, &["checkout", &branch])?;
    }
    let report = tool(
        &copy.cwd,
        &[
            "repo",
            "seal",
            "--path",
            copy.cwd
                .to_str()
                .ok_or_else(|| PortError::invalid("worktree path is not UTF-8"))?,
            "--change-set-ref",
            &copy.change_set,
            "--baseline",
            &copy.baseline,
            "--key",
            &copy.seal_key,
        ],
    )?;
    if report["schema"] != "hctl2.git-seal.v1"
        || report["outcome"] != "established"
        || report["evidence_level"] != "unmediated"
        || report["change_set_id"] != copy.change_set
        || report["base_commit_sha"] != copy.baseline
        || !report["error"].is_null()
        || report["repo_path"]
            .as_str()
            .and_then(|p| Path::new(p).canonicalize().ok())
            != Some(copy.cwd.clone())
    {
        return Err(PortError::new(
            "WRITE_SEAL_READBACK_MISMATCH",
            "tool seal does not describe this frozen ChangeSet",
            "inspect_preserved_worktree",
        ));
    }
    for key in [
        "base_commit_sha",
        "base_tree_sha",
        "result_tree_sha",
        "result_commit_sha",
    ] {
        sha(report[key]
            .as_str()
            .ok_or_else(|| PortError::invalid("tool seal SHA missing"))?)?;
    }
    let location = if report["base_tree_sha"] == report["result_tree_sha"] {
        json!({"kind":"no_changes","repo_path":copy.cwd})
    } else {
        json!({"kind":"commit","repo_path":copy.cwd,"commit_sha":report["result_commit_sha"]})
    };
    let bytes = canonical(&json!({"change_set_id":copy.change_set,
        "lease":{"lease_id":copy.lease.id,"generation":copy.generation},
        "base_commit_sha":report["base_commit_sha"],"parent_revision_id":null,"location":location}))?;
    Ok((bytes, report))
}

/// A native answer is saved locally, but a failed seal is not a returned round.
/// Retry only the tool step; never redispatch the request to the harness.
pub(crate) fn seal_return(
    copy: &WorkCopy,
    answer: &[u8],
    deadline: u64,
    tx: &std::sync::mpsc::SyncSender<crate::runtime::RuntimeEvent>,
    stopped: impl Fn() -> bool,
) -> Result<Option<(String, Vec<u8>)>> {
    let path = copy
        .cwd
        .parent()
        .ok_or_else(|| PortError::invalid("work copy parent missing"))?
        .join("pending-answer.json");
    let mut file = crate::storage::private_file(&path)?;
    use std::io::Write;
    file.set_len(0)?;
    file.write_all(&canonical(
        &json!({"lease":copy.lease,"answer":String::from_utf8_lossy(answer)}),
    )?)?;
    file.sync_all()?;
    let mut previous = None;
    loop {
        if stopped() || crate::storage::now_ms() >= deadline {
            if crate::storage::now_ms() >= deadline {
                let _ = tx.send(crate::runtime::RuntimeEvent::DeadlineReached);
            }
            return Ok(None);
        }
        match return_turn(copy) {
            Ok((bytes, report)) => {
                let _ = tx.send(crate::runtime::RuntimeEvent::Observation {
                    kind: "git_sealed".into(),
                    payload: report,
                    source: agency_proto::EvidenceLevel::Unmediated,
                });
                let _ = tx.send(crate::runtime::RuntimeEvent::Observation {
                    kind: "native_answer".into(),
                    payload: json!({"text":String::from_utf8_lossy(answer)}),
                    source: agency_proto::EvidenceLevel::AdapterEvent,
                });
                return Ok(Some(("hctl2.changeset-output.v1".into(), bytes)));
            }
            Err(error) => {
                if previous.as_deref() != Some(error.code.as_str()) {
                    let _ = tx.send(crate::runtime::RuntimeEvent::Observation {
                        kind:"seal_failed".into(), payload:json!({"code":error.code,"message":error.message,"worktree":copy.cwd}), source:agency_proto::EvidenceLevel::AdapterEvent,
                    });
                    previous = Some(error.code);
                }
            }
        }
        for _ in 0..40 {
            if stopped() || crate::storage::now_ms() >= deadline {
                if crate::storage::now_ms() >= deadline {
                    let _ = tx.send(crate::runtime::RuntimeEvent::DeadlineReached);
                }
                return Ok(None);
            }
            std::thread::sleep(Duration::from_millis(25));
        }
    }
}

fn sha(value: &str) -> Result<()> {
    if value.len() == 40
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        Ok(())
    } else {
        Err(PortError::invalid(
            "exact lowercase baseline SHA-1 required",
        ))
    }
}

fn git(path: &Path, args: &[&str]) -> Result<String> {
    let output = git_optional(path, args)?;
    if !output.status.success() {
        return Err(PortError::new(
            "WRITE_GIT_FAILED",
            String::from_utf8_lossy(&output.stderr)
                .chars()
                .take(4096)
                .collect::<String>(),
            "inspect_preserved_worktree",
        ));
    }
    String::from_utf8(output.stdout)
        .map_err(|_| PortError::invalid("Git returned non-UTF-8 identity"))
}
fn git_optional(path: &Path, args: &[&str]) -> Result<Output> {
    let mut cmd = Command::new("/usr/bin/git");
    cmd.env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("LANG", "C.UTF-8")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_ALLOW_PROTOCOL", "file")
        .args([
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "credential.helper=",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "protocol.file.allow=always",
        ])
        .args(args)
        .current_dir(path);
    run(&mut cmd)
}

fn tool(path: &Path, args: &[&str]) -> Result<Value> {
    let program = std::env::var_os("HCTL2_TOOL_BIN")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HCTL2_INSTALL_ROOT")
                .map(|root| PathBuf::from(root).join("libexec/hctl2/hctl2-tool"))
        })
        .ok_or_else(|| {
            PortError::new(
                "WRITE_TOOL_UNAVAILABLE",
                "installed hctl2-tool is required to materialize and seal a ChangeSet",
                "install_toolbox",
            )
        })?;
    // Buck supplies a relative artifact path; resolve it before moving the tool
    // into the Agency private directory.
    let program = program.canonicalize().map_err(|error| {
        PortError::new(
            "WRITE_TOOL_UNAVAILABLE",
            format!("installed hctl2-tool cannot be resolved: {error}"),
            "install_toolbox",
        )
    })?;
    let mut cmd = Command::new(program);
    cmd.env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("LANG", "C.UTF-8")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(args)
        .current_dir(path);
    let output = run(&mut cmd)?;
    if !output.status.success() {
        let native = serde_json::from_slice::<Value>(&output.stdout).ok();
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = native
            .as_ref()
            .and_then(|v| v["error"]["code"].as_str())
            .or_else(|| {
                stderr
                    .split("error[")
                    .nth(1)
                    .and_then(|s| s.split(']').next())
            })
            .unwrap_or("WRITE_TOOL_FAILED");
        return Err(PortError::new(
            code,
            format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout)
                    .chars()
                    .take(4096)
                    .collect::<String>(),
                String::from_utf8_lossy(&output.stderr)
                    .chars()
                    .take(4096)
                    .collect::<String>()
            ),
            "inspect_preserved_worktree",
        ));
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

pub(crate) fn run(cmd: &mut Command) -> Result<Output> {
    let mut stdout = tempfile::tempfile()?;
    let mut stderr = tempfile::tempfile()?;
    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(stdout.try_clone()?)
        .stderr(stderr.try_clone()?)
        .process_group(0)
        .spawn()?;
    let deadline = Instant::now() + Duration::from_secs(30);
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = rustix::process::kill_process_group(
                rustix::process::Pid::from_child(&child),
                rustix::process::Signal::KILL,
            );
            let _ = child.wait();
            return Err(PortError::new(
                "WRITE_OPERATION_TIMEOUT",
                "local Git operation exceeded 30 seconds",
                "inspect_preserved_worktree",
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    stdout.seek(SeekFrom::Start(0))?;
    stderr.seek(SeekFrom::Start(0))?;
    let mut out = Vec::new();
    let mut err = Vec::new();
    Read::by_ref(&mut stdout)
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut out)?;
    Read::by_ref(&mut stderr)
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut err)?;
    if out.len() > 16 * 1024 * 1024 || err.len() > 16 * 1024 * 1024 {
        return Err(PortError::invalid("local operation output exceeds limit"));
    }
    Ok(Output {
        status,
        stdout: out,
        stderr: err,
    })
}
