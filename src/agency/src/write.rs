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
    let cwd = root.join("worktree");
    let repository = root.join("repository.git");
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
    if !repository.exists() {
        git(
            &root,
            &[
                "init",
                "--bare",
                "--template=",
                repository.to_str().ok_or_else(fail)?,
            ],
        )?;
    }
    if !cwd.exists() {
        // No remote or source configuration is persisted. Fetch only the exact
        // baseline from a local path, with credential helpers and hooks disabled.
        git(
            &repository,
            &[
                "fetch",
                "--no-tags",
                "--no-write-fetch-head",
                "--",
                source.to_str().ok_or_else(fail)?,
                base,
            ],
        )?;
        git(
            &repository,
            &[
                "worktree",
                "add",
                "--detach",
                "--",
                cwd.to_str().ok_or_else(fail)?,
                base,
            ],
        )?;
        git(
            &repository,
            &[
                "update-ref",
                &format!("refs/hctl2/changesets/{change_set}/baseline"),
                base,
            ],
        )?;
    }
    let actual = git(&cwd, &["rev-parse", "--verify", "HEAD^{commit}"])?;
    if actual.trim() != base
        || git_optional(&cwd, &["symbolic-ref", "-q", "HEAD"])?
            .status
            .success()
    {
        return Err(PortError::new(
            "WRITE_WORKTREE_CONFLICT",
            "preserved worktree must remain detached at its frozen baseline",
            "inspect_preserved_worktree",
        ));
    }
    let common = PathBuf::from(
        git(
            &cwd,
            &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        )?
        .trim(),
    );
    if common.canonicalize()? != repository.canonicalize()? {
        return Err(PortError::new(
            "WRITE_WORKTREE_CONFLICT",
            "worktree is no longer attached to the Agency private repository",
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
    }))
}

pub(crate) fn task_text(bundle: &Bundle, copy: Option<&WorkCopy>) -> Result<String> {
    let mut text = crate::launch::task_text(bundle)?;
    if let Some(copy) = copy {
        text.push_str(&format!("\nAgency ChangeSet: {}\nBaseline: {}\nWrite lease: {} generation {}\nAgency execution directory: {}\nEdit and test this ChangeSet in this detached worktree. Leave changes uncommitted for Agency sealing. Do not push, publish reviews, obtain Git credentials, or change harness global configuration.\n", copy.change_set, copy.baseline, copy.lease.id, copy.generation, copy.cwd.display()));
    }
    Ok(text)
}

// Replaced by the #405 public seal entry only after that PR is on main.
pub(crate) fn return_turn(_copy: &WorkCopy, _answer: &[u8]) -> Result<(String, Vec<u8>)> {
    Err(PortError::new(
        "WRITE_SEAL_UNAVAILABLE",
        "hctl2-tool sealing dependency #405 is not installed; worktree preserved",
        "retry_after_seal_available",
    ))
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
        match return_turn(copy, answer) {
            Ok(output) => return Ok(Some(output)),
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
