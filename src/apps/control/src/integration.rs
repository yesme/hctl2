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

use crate::scm as platform;
use crate::services::Supervisor;

pub(crate) mod gitea;
#[cfg(test)]
mod tests;

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
    services: &Supervisor,
    root: &Path,
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
    let observation = observe(root, services, &registration, &input)?;
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
fn observe(
    root: &Path,
    services: &Supervisor,
    registration: &Registration,
    input: &Input,
) -> Result<Observation> {
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
            let (head, protection) = match registration.prepared.platform {
                Platform::Local => {
                    let hosted =
                        platform::Hosted::connect(root, &registration.config.control_id, services)?;
                    let target = gitea::observe(&hosted, &observed.full_name, &input.target_ref)?;
                    (target.head, Some(target.protection))
                }
                // GitHub observation lands with the canary path (验收第 11 条).
                Platform::Github | Platform::None => (None, None),
            };
            Ok(Observation {
                provider_ref: format!("{}/{}", observed.instance, observed.full_name),
                head,
                protection,
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
pub(crate) async fn reconcile(shared: Shared, root: PathBuf, services: Arc<Supervisor>) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut last_attempt: HashMap<String, Instant> = HashMap::new();
    let mut previous_error = None;
    loop {
        interval.tick().await;
        let error = reconcile_once(&shared, &root, &services, &mut last_attempt)
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
    root: &Path,
    services: &Arc<Supervisor>,
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
        let root = root.to_path_buf();
        let services = Arc::clone(services);
        let result =
            tokio::task::spawn_blocking(move || drive(&shared, &root, &services, &repo_id, &id))
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
pub(crate) fn drive(
    shared: &Shared,
    root: &Path,
    services: &Supervisor,
    repo_id: &str,
    intent_id: &str,
) -> Result<()> {
    drive_with(shared, repo_id, intent_id, &mut |registration| {
        platform::Hosted::connect(root, &registration.config.control_id, services)
    })
}

/// `drive` with the platform connection supplied by the caller (tests pass a fixture).
pub(crate) fn drive_with(
    shared: &Shared,
    repo_id: &str,
    intent_id: &str,
    connect: &mut dyn FnMut(&Registration) -> Result<platform::Hosted>,
) -> Result<()> {
    let (outcome, replan) = match plan_attempt_with(shared, repo_id, intent_id, connect)? {
        Planned::Local {
            path,
            attempt,
            preview,
        } => integrate_local(&path, &attempt, &preview)?,
        Planned::Gitea {
            hosted,
            full_name,
            review,
            attempt,
            preview,
        } => integrate_gitea(&hosted, &full_name, &review, &attempt, &preview)?,
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
pub(crate) enum Planned {
    Local {
        path: PathBuf,
        attempt: domain::AttemptInput,
        preview: Box<Preview>,
    },
    Gitea {
        hosted: platform::Hosted,
        full_name: String,
        review: domain::ReviewRequestRef,
        attempt: domain::AttemptInput,
        preview: Box<Preview>,
    },
    /// Nothing may run: the readback to record instead.
    Refused(Outcome),
}

/// Begin the attempt and freeze its executor input. A recorded attempt is reused verbatim, so
/// a confirmation lost between the executor's write and control's readback retries the exact
/// same input and the executor answers from its own retry record.
/// Freeze the next attempt of a local-target intent without executing it; `None` when the
/// plan was refused. Exposed for tests that simulate a crash between write and confirmation.
pub fn plan_local_attempt(
    shared: &Shared,
    repo_id: &str,
    intent_id: &str,
) -> Result<Option<domain::AttemptInput>> {
    match plan_attempt_with(shared, repo_id, intent_id, &mut |_| {
        Err(reject(
            "PLATFORM_UNAVAILABLE",
            "no platform connection in this context",
            "use_local_target",
        ))
    })? {
        Planned::Local { attempt, .. } => Ok(Some(attempt)),
        Planned::Gitea { attempt, .. } => Ok(Some(attempt)),
        Planned::Refused(_) => Ok(None),
    }
}

pub(crate) fn plan_attempt_with(
    shared: &Shared,
    repo_id: &str,
    intent_id: &str,
    connect: &mut dyn FnMut(&Registration) -> Result<platform::Hosted>,
) -> Result<Planned> {
    let (intent, _) = access(shared, |s| domain::begin(s, repo_id, intent_id))?;
    let registration = access(shared, |s| require_active(s, repo_id))?;
    let preview = &intent.preview;
    match preview.target.kind {
        TargetKind::Platform if registration.prepared.platform != Platform::Local => {
            Ok(Planned::Refused(Outcome::Attention(Attention {
                code: "PLATFORM_INTEGRATION_UNAVAILABLE".into(),
                message: "this platform's merge adapter is not wired yet (GitHub lands with the canary path)".into(),
                recovery_action: "wait_for_platform_adapter".into(),
                details: Value::Null,
            })))
        }
        TargetKind::Platform => {
            let observed = registration.observed.clone().ok_or_else(|| {
                reject("REPO_PENDING", "platform binding not confirmed", "confirm_repo")
            })?;
            let continuity = json!({"instance": observed.instance, "stable_id": observed.stable_id});
            if preview.target.continuity.as_ref() != Some(&continuity) {
                return Ok(Planned::Refused(Outcome::Attention(Attention {
                    code: "TARGET_IDENTITY_MISMATCH".into(),
                    message: "the platform repository is not the one the preview froze; nothing was written".into(),
                    recovery_action: "preview_a_new_intent".into(),
                    details: json!({"frozen": preview.target.continuity, "observed": continuity}),
                })));
            }
            // The platform merges a review request; the revision must have been published.
            let Some(review) = access(shared, |s| {
                domain::review_request(s, repo_id, &preview.source.change_set_revision_id)
            })? else {
                return Ok(Planned::Refused(Outcome::Attention(Attention {
                    code: "REVIEW_REQUEST_MISSING".into(),
                    message: "this revision has no review request on the platform yet; publish it first".into(),
                    recovery_action: "publish_review_then_retry_same_intent".into(),
                    details: json!({"change_set_revision_id": preview.source.change_set_revision_id}),
                })));
            };
            let hosted = connect(&registration)?;
            let attempt = match &intent.attempt {
                Some(attempt) => attempt.clone(),
                None => {
                    let expected = match preview.form {
                        Form::ExpectedHead => preview.expected_head.clone(),
                        Form::AcceptAdvance => {
                            gitea::observe(&hosted, &observed.full_name, &preview.target.target_ref)?.head
                        }
                    };
                    let Some(expected) = expected else {
                        return Ok(Planned::Refused(Outcome::Failed(Attention {
                            code: "TARGET_MISSING".into(),
                            message: "target branch does not exist on the platform".into(),
                            recovery_action: "create_target_branch_then_new_intent".into(),
                            details: json!({"target_ref": preview.target.target_ref}),
                        })));
                    };
                    let number = intent.attempts + 1;
                    let attempt = domain::AttemptInput {
                        number,
                        idempotency_key: format!("{}:{number}", intent.intent_id),
                        commit: review.platform_commit_sha.clone(),
                        expected_head: expected,
                    };
                    access(shared, |s| {
                        domain::record_attempt(s, repo_id, intent_id, attempt.clone())
                    })?;
                    attempt
                }
            };
            Ok(Planned::Gitea {
                hosted,
                full_name: observed.full_name,
                review,
                attempt,
                preview: Box::new(preview.clone()),
            })
        }
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
        // On a retry the tool recognises its own earlier write (`already_applied`); its
        // `before_head` is then the head *now*, not the head the original write started from.
        // That original head is the frozen attempt's expected head, which the tool's own
        // compare-and-swap verified when it wrote.
        let before = if record["status"] == "already_applied" {
            Some(attempt.expected_head.clone())
        } else {
            record["before_head"].as_str().map(str::to_owned)
        };
        return Ok((
            Outcome::Succeeded {
                target_head_before: before,
                target_head_after: after,
                integrated_commit: new,
                integrated_tree: Some(tree),
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

/// Merge on the hosted Gitea: compare protection with the frozen snapshot, ask the platform to
/// merge the published review request with the source head pinned, then read the request and
/// the branch back. Only that readback can confirm; "accepted" alone never does.
fn integrate_gitea(
    hosted: &platform::Hosted,
    full_name: &str,
    review: &domain::ReviewRequestRef,
    attempt: &domain::AttemptInput,
    preview: &Preview,
) -> Result<(Outcome, bool)> {
    let current = gitea::observe(hosted, full_name, &preview.target.target_ref)?;
    if preview.protection.as_ref() != Some(&current.protection) {
        return Ok((
            Outcome::Attention(Attention {
                code: "PROTECTION_CHANGED".into(),
                message:
                    "target protection differs from the frozen snapshot; nothing was requested"
                        .into(),
                recovery_action: "preview_a_new_intent_or_restore_protection".into(),
                details: json!({"frozen": preview.protection, "current": current.protection}),
            }),
            false,
        ));
    }
    let before = match gitea::review_request(hosted, full_name, review.index)? {
        Some(request) => request,
        None => {
            return Ok((
                Outcome::Attention(Attention {
                    code: "REVIEW_REQUEST_MISSING".into(),
                    message: format!("review request #{} is not on the platform", review.index),
                    recovery_action: "publish_review_then_retry_same_intent".into(),
                    details: Value::Null,
                }),
                false,
            ));
        }
    };
    if !before.merged {
        if preview.form == Form::ExpectedHead
            && current.head.as_deref() != Some(attempt.expected_head.as_str())
        {
            return Ok((
                Outcome::Failed(Attention {
                    code: "TARGET_HEAD_MISMATCH".into(),
                    message: "target head differs from the frozen expected head".into(),
                    recovery_action: "preview_a_new_intent_against_the_current_head".into(),
                    details: json!({"expected": attempt.expected_head, "observed": current.head}),
                }),
                false,
            ));
        }
        if before.head_sha.as_deref() != Some(attempt.commit.as_str()) {
            return Ok((
                Outcome::Failed(Attention {
                    code: "SOURCE_HEAD_MISMATCH".into(),
                    message: "the review request's head is not the published revision commit"
                        .into(),
                    recovery_action: "publish_the_revision_again_then_new_intent".into(),
                    details: json!({"published": attempt.commit, "review_head": before.head_sha}),
                }),
                false,
            ));
        }
        let message = format!(
            "hctl2 integration of ChangeSet Revision {}",
            preview.source.change_set_revision_id
        );
        match gitea::request_merge(
            hosted,
            full_name,
            review.index,
            gitea::merge_style(preview.strategy),
            &attempt.commit,
            &message,
        ) {
            Ok(_) => {}
            // The platform refused the write before doing it (checks, approvals, conflicts):
            // the human can clear the cause, the same intent retries.
            Err(error) if matches!(error.code, "NATIVE_REJECTED" | "NATIVE_CONFLICT") => {
                return Ok((
                    Outcome::Attention(Attention {
                        code: "NOT_MERGEABLE".into(),
                        message: error.message,
                        recovery_action: "satisfy_protection_then_retry_same_intent".into(),
                        details: json!({"platform_code": error.code}),
                    }),
                    false,
                ));
            }
            // Anything else may have reached the platform: fall through to readback, which
            // alone decides; the request is never resent blindly.
            Err(_) => {}
        }
    }
    // Readback: the request must be merged and the branch must carry the merge commit.
    let after = gitea::review_request(hosted, full_name, review.index)?;
    let head = gitea::observe(hosted, full_name, &preview.target.target_ref)?.head;
    let unmerged = after.clone().map(|r| r.raw);
    let Some(after) = after.filter(|r| r.merged) else {
        return Ok((
            Outcome::Unknown(Attention {
                code: "RESULT_UNKNOWN".into(),
                message: "the platform accepted the request but the review request does not read back as merged".into(),
                recovery_action: "read_back_review_request_with_same_intent".into(),
                details: json!({"review_request": unmerged}),
            }),
            false,
        ));
    };
    let (Some(merge_commit), Some(head)) = (after.merge_commit_sha.clone(), head) else {
        return Ok((
            Outcome::Unknown(Attention {
                code: "RESULT_UNKNOWN".into(),
                message: "merged review request without a merge commit or branch head to read"
                    .into(),
                recovery_action: "read_back_review_request_with_same_intent".into(),
                details: json!({"review_request": after.raw}),
            }),
            false,
        ));
    };
    // Gitea's REST API returns no tree ids. A fast-forward merge commit is the published
    // candidate itself, so its tree is the admitted result tree once the ids agree; a merge
    // commit's tree stays unread rather than guessed.
    let integrated_tree = match preview.strategy {
        domain::Strategy::FastForward if merge_commit == attempt.commit => {
            Some(preview.source.result_tree_sha.clone())
        }
        domain::Strategy::FastForward => {
            return Ok((
                Outcome::Unknown(Attention {
                    code: "RESULT_UNKNOWN".into(),
                    message:
                        "fast-forward reported a merge commit other than the published candidate"
                            .into(),
                    recovery_action: "read_back_review_request_with_same_intent".into(),
                    details: json!({"merge_commit_sha": merge_commit, "published": attempt.commit}),
                }),
                false,
            ));
        }
        domain::Strategy::MergeCommit => None,
    };
    Ok((
        Outcome::Succeeded {
            target_head_before: Some(attempt.expected_head.clone()),
            target_head_after: head,
            integrated_commit: merge_commit,
            integrated_tree,
            evidence_level: "platform_adapter".into(),
            observed_at_unix_ms: crate::dispatch::now_ms(),
            readback: json!({"status": if before.merged { "already_applied" } else { "applied" }, "review_request": after.raw}),
        },
        false,
    ))
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
