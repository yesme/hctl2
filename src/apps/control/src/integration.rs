//! Integration orchestration (第 6 包，集成一半). Git and platform I/O never run inside a
//! Store transaction: control persists the intent, the executor acts, control reads back.
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use repo::integration::{
    self as domain, Attention, Form, Input, IntentState, Observation, Outcome, Preview, TargetKind,
};
use repo::{Platform, Registration, Result, reject, require_active};
use serde_json::{Value, json};
use store::{Store, TrustedActor};
use tokio::sync::Mutex;

use crate::services::Supervisor;

type Shared = Arc<Mutex<Option<Store>>>;

fn access<T>(shared: &Shared, f: impl FnOnce(&mut Store) -> Result<T>) -> Result<T> {
    let mut slot = shared.blocking_lock();
    f(slot
        .as_mut()
        .ok_or_else(|| reject("STORE_NOT_READY", "storage not ready", "check_status"))?)
}

const OPERATION: &str = "integration.submit";

pub(crate) fn preview(
    shared: &Shared,
    _services: &Supervisor,
    actor: &TrustedActor,
    operation: &str,
    payload: &Value,
) -> Result<Value> {
    if operation != OPERATION {
        return Err(reject(
            "INVALID_INPUT",
            "only integration.submit is previewed",
            "correct_input",
        ));
    }
    let input: Input = serde_json::from_value(payload.clone())?;
    // A key that was already admitted replays its frozen preview; the target is not read again.
    if let Some(frozen) = access(shared, |s| frozen(s, &input))? {
        return Ok(serde_json::to_value(frozen)?);
    }
    let registration = access(shared, |s| require_active(s, &input.repo_id))?;
    let observation = observe(&registration, &input)?;
    let preview = access(shared, |s| domain::prepare(s, actor, input, observation))?;
    Ok(serde_json::to_value(preview)?)
}

pub(crate) fn submit(
    shared: &Shared,
    actor: &TrustedActor,
    request: &proto::SubmitRequest,
    details: &Value,
) -> Result<Value> {
    let input: Input = serde_json::from_slice(&request.payload)?;
    if request.operation != OPERATION
        || request.idempotency_key != input.key
        || request.command_id != format!("integration:{}", input.key)
    {
        return Err(reject(
            "INVALID_INPUT",
            "integration envelope differs",
            "use_original_command",
        ));
    }
    let preview: Preview = serde_json::from_value(details.clone())?;
    if preview.input != input {
        return Err(reject(
            "INVALID_INPUT",
            "integration preview input differs",
            "rebuild_preview",
        ));
    }
    let intent = access(shared, |s| {
        domain::submit(s, actor, &request.command_id, preview)
    })?;
    Ok(json!({"intent_id": intent.intent_id, "repo_id": intent.repo_id, "state": intent.state}))
}

/// The admitted preview for this key, if the same input was already submitted.
fn frozen(store: &Store, input: &Input) -> Result<Option<Preview>> {
    let id = domain::intent_id(store.control_id(), &input.repo_id, &input.key);
    match domain::get(store, &input.repo_id, &id) {
        Ok(intent) if intent.preview.input == *input => Ok(Some(intent.preview)),
        Ok(_) => Err(reject(
            "IDEMPOTENCY_CONFLICT",
            "same integration key with different input",
            "use_new_command_key",
        )),
        Err(error) if error.code == "INTENT_NOT_FOUND" => Ok(None),
        Err(error) => Err(error),
    }
}

pub(crate) fn query(shared: &Shared, kind: &str, payload: &Value) -> Result<Value> {
    let repo_id = field(payload, "repo_id")?;
    access(shared, |s| match kind {
        "integration.show" => domain::show(s, repo_id, field(payload, "intent_id")?),
        "integration.list" => Ok(json!({"items": domain::list(s, repo_id)?})),
        _ => Err(reject(
            "INVALID_INPUT",
            "unknown integration query",
            "correct_input",
        )),
    })
}

fn field<'a>(value: &'a Value, name: &str) -> Result<&'a str> {
    value[name]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| reject("INVALID_INPUT", format!("{name} required"), "correct_input"))
}

/// The registered local repository a Repo without a platform integrates in.
fn local_path(registration: &Registration) -> Result<PathBuf> {
    registration
        .prepared
        .local
        .as_ref()
        .map(|snapshot| snapshot.path.clone())
        .ok_or_else(|| {
            reject(
                "LOCAL_TARGET_NOT_ALLOWED",
                "this Repo was not registered from a local repository on this machine",
                "choose_platform_target",
            )
        })
}

/// Read the target before admission. Local heads come from `hctl2-tool repo inspect`.
fn observe(registration: &Registration, input: &Input) -> Result<Observation> {
    match input.target_kind {
        TargetKind::Local => {
            let path = local_path(registration)?;
            let local = inspect_local(&path, &input.target_ref)?;
            Ok(Observation {
                provider_ref: path.display().to_string(),
                head: local.head,
                protection: None,
                continuity: Some(local.continuity),
            })
        }
        TargetKind::Platform => {
            // Platform targets are observed by the platform adapter; until the binding declares
            // the verified capabilities, admission rejects with CAPABILITY_MISSING.
            if registration.prepared.platform == Platform::None {
                return Err(reject(
                    "PLATFORM_NOT_BOUND",
                    "this Repo has no platform",
                    "choose_local_target",
                ));
            }
            let observed = registration.observed.as_ref().ok_or_else(|| {
                reject(
                    "REPO_PENDING",
                    "platform binding not confirmed",
                    "confirm_repo",
                )
            })?;
            Ok(Observation {
                provider_ref: format!("{}/{}", observed.instance, observed.full_name),
                head: None,
                protection: None,
                continuity: Some(
                    json!({"instance": observed.instance, "stable_id": observed.stable_id}),
                ),
            })
        }
    }
}

struct LocalTarget {
    head: Option<String>,
    /// The Git common directory's filesystem identity: a fresh clone at the same path has a
    /// new inode, a moved repository keeps its own.
    continuity: Value,
}

/// Read a local target's head and the identity of the repository that holds it.
fn inspect_local(path: &Path, target_ref: &str) -> Result<LocalTarget> {
    let (code, value) = tool_json(&[
        "repo".into(),
        "inspect".into(),
        "--path".into(),
        path.to_path_buf().into(),
        "--ref".into(),
        target_ref.to_owned().into(),
    ])?;
    let (head, value) = match code {
        0 => (
            value["repository_state"]["requested_ref"]["commit_sha"]
                .as_str()
                .map(str::to_owned),
            value,
        ),
        _ if value["error"]["code"] == "HCTL2_TOOL_REF_NOT_FOUND" => {
            // The ref is absent; the repository itself still has to be identified.
            let (code, value) = tool_json(&[
                "repo".into(),
                "inspect".into(),
                "--path".into(),
                path.to_path_buf().into(),
            ])?;
            if code != 0 {
                return Err(unreadable(&value));
            }
            (None, value)
        }
        _ => return Err(unreadable(&value)),
    };
    let common_dir = value["common_directory_identity"]["git_common_dir"]
        .as_str()
        .ok_or_else(|| {
            reject(
                "TARGET_UNREADABLE",
                "repository common directory missing",
                "inspect_repository",
            )
        })?;
    let metadata = std::fs::metadata(common_dir).map_err(|error| {
        reject(
            "TARGET_UNREADABLE",
            format!("repository common directory unreadable: {error}"),
            "inspect_repository",
        )
    })?;
    use std::os::unix::fs::MetadataExt;
    Ok(LocalTarget {
        head,
        continuity: json!({
            "git_common_dir": common_dir,
            "device": metadata.dev(),
            "inode": metadata.ino(),
        }),
    })
}

fn unreadable(value: &Value) -> repo::StoreError {
    reject(
        "TARGET_UNREADABLE",
        format!(
            "hctl2-tool could not read the target: {}",
            value["error"]["message"]
        ),
        "inspect_repository",
    )
}

/// Run one tool command in-process and parse its JSON record: `(exit_code, record)`.
fn tool_json(arguments: &[OsString]) -> Result<(u8, Value)> {
    let output = tool::run(arguments.iter().cloned()).map_err(|error| {
        reject(
            "TOOL_UNAVAILABLE",
            format!("hctl2-tool {}: {}", error.code(), error.message()),
            "inspect_tool_installation",
        )
    })?;
    let value: Value = serde_json::from_str(output.body()).map_err(|_| {
        reject(
            "TOOL_UNAVAILABLE",
            "hctl2-tool returned no JSON record",
            "inspect_tool_installation",
        )
    })?;
    Ok((output.exit_code(), value))
}

/// One periodic worker owns execution and readback of open intents; queries never do.
pub(crate) async fn reconcile(shared: Shared, root: PathBuf, _services: Arc<Supervisor>) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut last_attempt: HashMap<String, Instant> = HashMap::new();
    let mut previous_error = None;
    loop {
        interval.tick().await;
        let error = reconcile_once(&shared, &root, &mut last_attempt)
            .await
            .err();
        let code = error.as_ref().map(|e| e.code);
        if code != previous_error {
            if let Some(error) = error {
                eprintln!(
                    "Integration delivery: {} ({})",
                    error.code, error.recovery_action
                );
            }
            previous_error = code;
        }
    }
}

/// Attempts that left the human something to do are retried only every so often.
const RETRY_AFTER: Duration = Duration::from_secs(10);

pub async fn reconcile_once(
    shared: &Shared,
    _root: &Path,
    last_attempt: &mut HashMap<String, Instant>,
) -> Result<()> {
    let open = {
        let lock = shared.lock().await;
        let Some(store) = lock.as_ref() else {
            return Ok(());
        };
        domain::open(store)?
    };
    let mut first_error = None;
    for intent in open {
        if intent.state == IntentState::Unknown
            && last_attempt
                .get(&intent.intent_id)
                .is_some_and(|at| at.elapsed() < RETRY_AFTER)
        {
            continue;
        }
        last_attempt.insert(intent.intent_id.clone(), Instant::now());
        let shared = Arc::clone(shared);
        let repo_id = intent.repo_id.clone();
        let id = intent.intent_id.clone();
        let result = tokio::task::spawn_blocking(move || drive(&shared, &repo_id, &id))
            .await
            .unwrap_or_else(|_| {
                Err(reject(
                    "STORAGE_IO",
                    "integration worker failed",
                    "retry_later",
                ))
            });
        if let Err(e) = result {
            first_error.get_or_insert(e);
        }
    }
    last_attempt.retain(|_, at| at.elapsed() < RETRY_AFTER * 6);
    first_error.map_or(Ok(()), Err)
}

/// Execute one open intent once and record exactly what was read back.
pub(crate) fn drive(shared: &Shared, repo_id: &str, intent_id: &str) -> Result<()> {
    let (outcome, replan) = match plan_attempt(shared, repo_id, intent_id)? {
        Planned::Local {
            path,
            attempt,
            preview,
        } => integrate_local(&path, &attempt, &preview)?,
        Planned::Refused(outcome) => (outcome, false),
    };
    access(shared, |s| domain::confirm(s, repo_id, intent_id, outcome))?;
    if replan {
        // The executor proved nothing was written under the old plan; the next pass may
        // read the target again (accept-advance only).
        access(shared, |s| domain::clear_attempt(s, repo_id, intent_id))?;
    }
    Ok(())
}

/// What one attempt will run with, frozen in the intent before the executor starts.
pub enum Planned {
    Local {
        path: PathBuf,
        attempt: domain::AttemptInput,
        preview: Box<Preview>,
    },
    /// Nothing may run: the readback to record instead.
    Refused(Outcome),
}

/// Begin the attempt and freeze its executor input. A recorded attempt is reused verbatim, so
/// a confirmation lost between the executor's write and control's readback retries the exact
/// same input and the executor answers from its own retry record.
pub fn plan_attempt(shared: &Shared, repo_id: &str, intent_id: &str) -> Result<Planned> {
    let (intent, _) = access(shared, |s| domain::begin(s, repo_id, intent_id))?;
    let registration = access(shared, |s| require_active(s, repo_id))?;
    let preview = &intent.preview;
    match preview.target.kind {
        TargetKind::Platform => Ok(Planned::Refused(Outcome::Attention(Attention {
            code: "PLATFORM_INTEGRATION_UNAVAILABLE".into(),
            message:
                "platform targets are executed by the platform adapter, which is not wired yet"
                    .into(),
            recovery_action: "wait_for_platform_adapter".into(),
            details: Value::Null,
        }))),
        TargetKind::Local => {
            let path = local_path(&registration)?;
            // The target must still be the repository the preview froze, not a replacement
            // at the same path. A mismatch writes nothing and keeps the intent waiting.
            let local = inspect_local(&path, &preview.target.target_ref)?;
            if preview.target.continuity.as_ref() != Some(&local.continuity) {
                return Ok(Planned::Refused(Outcome::Attention(Attention {
                    code: "TARGET_IDENTITY_MISMATCH".into(),
                    message: "the repository at the target path is not the one the preview froze (a new clone or another repository); nothing was written".into(),
                    recovery_action: "restore_original_repository_or_preview_a_new_intent".into(),
                    details: json!({"frozen": preview.target.continuity, "observed": local.continuity}),
                })));
            }
            let attempt = match &intent.attempt {
                Some(attempt) => attempt.clone(),
                None => {
                    let commit = match result_commit(&path, &preview.source)? {
                        Ok(commit) => commit,
                        Err(reason) => return Ok(Planned::Refused(Outcome::Failed(reason))),
                    };
                    let expected = match preview.form {
                        Form::ExpectedHead => preview.expected_head.clone(),
                        // Accept the target wherever it is now; the CAS below still pins this attempt.
                        Form::AcceptAdvance => local.head.clone(),
                    };
                    let Some(expected) = expected else {
                        return Ok(Planned::Refused(Outcome::Failed(Attention {
                            code: "TARGET_MISSING".into(),
                            message: "target ref does not exist".into(),
                            recovery_action: "create_target_ref_then_new_intent".into(),
                            details: json!({"target_ref": preview.target.target_ref}),
                        })));
                    };
                    let number = intent.attempts + 1;
                    let attempt = domain::AttemptInput {
                        number,
                        idempotency_key: format!("{}:{number}", intent.intent_id),
                        commit,
                        expected_head: expected,
                    };
                    access(shared, |s| {
                        domain::record_attempt(s, repo_id, intent_id, attempt.clone())
                    })?;
                    attempt
                }
            };
            Ok(Planned::Local {
                path,
                attempt,
                preview: Box::new(preview.clone()),
            })
        }
    }
}

/// `hctl2-tool integrate` on the registered local repository with a frozen attempt. The tool's
/// compare-and-swap on the target ref is the write; its JSON record is the readback. Returns
/// the outcome and whether the next pass may plan afresh (only after a proven no-write).
fn integrate_local(
    path: &Path,
    attempt: &domain::AttemptInput,
    preview: &Preview,
) -> Result<(Outcome, bool)> {
    let form = preview.form;
    let (code, record) = tool_json(&[
        "integrate".into(),
        "--repo".into(),
        path.to_path_buf().into(),
        "--commit".into(),
        attempt.commit.clone().into(),
        "--base-commit-sha".into(),
        preview.source.base_commit_sha.clone().into(),
        "--result-tree-sha".into(),
        preview.source.result_tree_sha.clone().into(),
        "--target-ref".into(),
        preview.target.target_ref.clone().into(),
        "--expected-head".into(),
        attempt.expected_head.clone().into(),
        "--strategy".into(),
        preview.strategy.tool_name().into(),
        "--idempotency-key".into(),
        attempt.idempotency_key.clone().into(),
    ])?;
    if code == 0 {
        let after = record["after_head"].as_str().unwrap_or_default().to_owned();
        let new = record["new_head"].as_str().unwrap_or_default().to_owned();
        let tree = record["integrated_tree_sha"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        if after.is_empty() || new.is_empty() || tree.is_empty() {
            return Ok((
                Outcome::Unknown(attention(
                    "READBACK_INCOMPLETE",
                    &record,
                    "read_target_with_same_intent",
                )),
                false,
            ));
        }
        return Ok((
            Outcome::Succeeded {
                target_head_before: record["before_head"].as_str().map(str::to_owned),
                target_head_after: after,
                integrated_commit: new,
                integrated_tree: tree,
                evidence_level: "hctl2-tool".into(),
                observed_at_unix_ms: record["observed_at_unix_ms"].as_u64().unwrap_or_default(),
                readback: record,
            },
            false,
        ));
    }
    let tool_code = record["error"]["code"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    Ok(match tool_code.as_str() {
        // The result may already be written: never a failure, never a new plan.
        "HCTL2_TOOL_INTEGRATION_RESULT_UNKNOWN" | "HCTL2_TOOL_INTEGRATION_KEY_REUSED" => (
            Outcome::Unknown(attention(
                "RESULT_UNKNOWN",
                &record,
                "read_target_with_same_intent",
            )),
            false,
        ),
        "HCTL2_TOOL_INTEGRATION_TARGET_CHECKED_OUT" => (
            Outcome::Attention(attention(
                "TARGET_CHECKED_OUT",
                &record,
                "switch_worktrees_away_from_target_then_retry_same_intent",
            )),
            false,
        ),
        "HCTL2_TOOL_SITE_BUSY" | "HCTL2_TOOL_SITE_LOCK_UNAVAILABLE" => (
            Outcome::Attention(attention("SITE_BUSY", &record, "retry_same_intent")),
            false,
        ),
        // Rejected before any write (the tool read the target and refused): under
        // accept-advance the next pass may plan against the moved target.
        "HCTL2_TOOL_INTEGRATION_HEAD_DRIFT" | "HCTL2_TOOL_INTEGRATION_CAS_REJECTED"
            if form == Form::AcceptAdvance =>
        {
            (
                Outcome::Attention(attention("TARGET_ADVANCED", &record, "retry_same_intent")),
                true,
            )
        }
        "HCTL2_TOOL_INTEGRATION_HEAD_DRIFT" | "HCTL2_TOOL_INTEGRATION_CAS_REJECTED" => (
            Outcome::Failed(attention(
                "TARGET_HEAD_MISMATCH",
                &record,
                "preview_a_new_intent_against_the_current_head",
            )),
            false,
        ),
        _ => (
            Outcome::Failed(attention(
                "INTEGRATION_REJECTED",
                &record,
                "inspect_tool_record_then_new_intent",
            )),
            false,
        ),
    })
}

/// The commit object handed to Git for an admitted `(base, result tree)`.
///
/// Identity lives in base and tree, not in a commit (`spec/repo.md`). An executor's own commit
/// with exactly that tree on top of the base is reused when the repository has one; otherwise
/// a wrapper commit is written with a fixed identity so retries resolve to the same object.
fn result_commit(
    repo_path: &Path,
    source: &domain::AdmittedRevision,
) -> Result<std::result::Result<String, Attention>> {
    let git = repo::git::Git::discover()?;
    let listing = repo::git::run(
        git.command(repo_path)
            .args(["log", "--all", "--format=%H %T %P"]),
        None,
    )?;
    if listing.status.success() {
        for line in String::from_utf8_lossy(&listing.stdout).lines() {
            let mut fields = line.split_whitespace();
            let (Some(commit), Some(tree)) = (fields.next(), fields.next()) else {
                continue;
            };
            if tree == source.result_tree_sha
                && fields.any(|parent| parent == source.base_commit_sha)
            {
                return Ok(Ok(commit.to_owned()));
            }
        }
    }
    let message = format!(
        "hctl2 integration of ChangeSet Revision {}",
        source.change_set_revision_id
    );
    let wrapped = repo::git::run(
        git.command(repo_path)
            .args([
                "commit-tree",
                &source.result_tree_sha,
                "-p",
                &source.base_commit_sha,
                "-m",
                &message,
            ])
            .env("GIT_AUTHOR_NAME", "hctl2")
            .env("GIT_AUTHOR_EMAIL", "hctl2@control.invalid")
            .env("GIT_AUTHOR_DATE", "@0 +0000")
            .env("GIT_COMMITTER_NAME", "hctl2")
            .env("GIT_COMMITTER_EMAIL", "hctl2@control.invalid")
            .env("GIT_COMMITTER_DATE", "@0 +0000"),
        None,
    )?;
    if !wrapped.status.success() {
        return Ok(Err(Attention {
            code: "RESULT_UNAVAILABLE".into(),
            message: format!(
                "the admitted result tree or its base is not in the target repository: {}",
                String::from_utf8_lossy(&wrapped.stderr).trim()
            ),
            recovery_action: "deliver_admitted_revision_to_target_repository".into(),
            details: json!({"base_commit_sha": source.base_commit_sha, "result_tree_sha": source.result_tree_sha}),
        }));
    }
    Ok(Ok(String::from_utf8_lossy(&wrapped.stdout)
        .trim()
        .to_owned()))
}

fn attention(code: &str, record: &Value, recovery: &str) -> Attention {
    Attention {
        code: code.into(),
        message: record["error"]["message"]
            .as_str()
            .map_or_else(|| code.to_lowercase(), str::to_owned),
        recovery_action: recovery.into(),
        details: json!({"tool": record}),
    }
}
