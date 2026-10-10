//! Review publishing orchestration (第 6 包 · 验收第 4 条、第 9 条发布半边). The intent is
//! persisted by admission; this worker runs its two stages and reads each back before
//! recording it: push the frozen commit to the policy's branch with the remote's old value
//! pinned, then find-or-create the review request on the platform and record the mapping.
//! A stage that may have reached the outside is read back by its association (branch head,
//! review request by base/head), never redone blindly.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use repo::git::{Credential, PushOutcome};
use repo::review::{self as domain, Attention, Outcome, State};
use repo::{Registration, Result, reject, require_active};
use serde_json::{Value, json};
use store::{Store, TrustedActor};
use tokio::sync::Mutex;

use crate::integration::target::PlatformTarget;
use crate::integration::{Connection, connect_platform};
use crate::services::Supervisor;

#[cfg(test)]
mod tests;

type Shared = Arc<Mutex<Option<Store>>>;

fn access<T>(shared: &Shared, f: impl FnOnce(&mut Store) -> Result<T>) -> Result<T> {
    let mut slot = shared.blocking_lock();
    f(slot
        .as_mut()
        .ok_or_else(|| reject("STORE_NOT_READY", "storage not ready", "check_status"))?)
}

const OPERATION: &str = "review.publish";

/// A human looks at what releasing a `pending_human` intent will do.
pub(crate) fn preview(
    shared: &Shared,
    _actor: &TrustedActor,
    operation: &str,
    payload: &Value,
) -> Result<Value> {
    if operation != OPERATION {
        return Err(reject(
            "INVALID_INPUT",
            "only review.publish is previewed",
            "correct_input",
        ));
    }
    let repo_id = field(payload, "repo_id")?;
    let intent_id = field(payload, "intent_id")?;
    let intent = access(shared, |s| domain::get(s, repo_id, intent_id))?;
    // The client names the round it looked at; a newer one wants a fresh look.
    if payload["round"]
        .as_u64()
        .is_some_and(|round| round != intent.round)
    {
        return Err(reject(
            "FROZEN_INPUT_CHANGED",
            format!(
                "the intent is at round {}, not the one requested",
                intent.round
            ),
            "preview_again",
        ));
    }
    if intent.state != State::PendingHuman {
        return Err(reject(
            "INTENT_NOT_PENDING_HUMAN",
            format!(
                "this publish intent is {:?}; only one waiting for a human is released",
                intent.state
            ),
            "inspect_intent",
        ));
    }
    Ok(json!({
        "repo_id": repo_id,
        "intent_id": intent_id,
        "intent_version": intent.version,
        "change_set_id": intent.change_set_id,
        "change_set_revision_id": intent.target.change_set_revision_id,
        "round": intent.round,
        "branch": intent.branch,
        "target_branch": intent.policy.policy.target_branch,
        "allow_update": intent.policy.policy.allow_update,
        "policy": intent.policy,
        "authorizes": "publishing this revision for review on the platform: push to the branch, then create or update the review request. Not a merge.",
    }))
}

pub(crate) fn submit(
    shared: &Shared,
    actor: &TrustedActor,
    request: &proto::SubmitRequest,
    details: &Value,
) -> Result<Value> {
    let payload: Value = serde_json::from_slice(&request.payload)?;
    let repo_id = field(&payload, "repo_id")?;
    let intent_id = field(&payload, "intent_id")?;
    let Some(round) = details["round"].as_u64() else {
        return Err(reject(
            "INVALID_INPUT",
            "review publish preview names no round",
            "rebuild_preview",
        ));
    };
    let Some(revision_id) = details["change_set_revision_id"].as_str() else {
        return Err(reject(
            "INVALID_INPUT",
            "review publish preview names no revision",
            "rebuild_preview",
        ));
    };
    if request.operation != OPERATION
        || details["intent_id"] != json!(intent_id)
        || details["repo_id"] != json!(repo_id)
        || request.command_id != command_id(intent_id, round)
    {
        return Err(reject(
            "INVALID_INPUT",
            "review publish envelope differs from its preview",
            "rebuild_preview",
        ));
    }
    // The release is bound to the revision and round the human looked at.
    let intent = access(shared, |s| {
        domain::release(
            s,
            actor,
            &request.command_id,
            repo_id,
            intent_id,
            revision_id,
            round,
        )
    })?;
    Ok(json!({"intent_id": intent.intent_id, "repo_id": intent.repo_id, "state": intent.state}))
}

/// One command identity per publish round: the same round replays, the next round is a new
/// authorization. The CLI builds the same string from `review.show`.
pub(crate) fn command_id(intent_id: &str, round: u64) -> String {
    format!("review-publish:{intent_id}:{round}")
}

pub(crate) fn query(shared: &Shared, kind: &str, payload: &Value) -> Result<Value> {
    let repo_id = field(payload, "repo_id")?;
    access(shared, |s| match kind {
        "review.show" => domain::show(s, repo_id, field(payload, "intent_id")?),
        "review.list" => Ok(json!({"items": domain::list(s, repo_id)?})),
        _ => Err(reject(
            "INVALID_INPUT",
            "unknown review query",
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

/// One periodic worker owns both stages of every open publish intent.
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
                    "Review publishing: {} ({})",
                    error.code, error.recovery_action
                );
            }
            previous_error = code;
        }
    }
}

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
        if intent.state == State::Unknown
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
                        "review publishing worker failed",
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

pub(crate) fn drive(
    shared: &Shared,
    root: &Path,
    services: &Supervisor,
    repo_id: &str,
    intent_id: &str,
) -> Result<()> {
    drive_with(shared, repo_id, intent_id, &mut |registration| {
        let connection = connect_platform(root, services, registration)?;
        let credential = match &connection {
            Connection::Gitea(hosted) => Credential::Static {
                user: hosted.username.clone(),
                token: hosted.token.clone(),
            },
            // GitHub's own credential helper for Git; control copies no token.
            Connection::GitHub(github) => Credential::Helper(format!(
                "!{} auth git-credential",
                github.gh_path().display()
            )),
        };
        Ok((connection, credential))
    })
}

/// Where the push reads its objects from and what it pushes with.
pub(crate) struct Site {
    pub local_path: PathBuf,
    pub clone_url: String,
    pub full_name: String,
}

/// Run one attempt of both stages; tests supply the platform and the Git credential.
pub(crate) fn drive_with(
    shared: &Shared,
    repo_id: &str,
    intent_id: &str,
    connect: &mut dyn FnMut(&Registration) -> Result<(Connection, Credential)>,
) -> Result<()> {
    let (intent, _) = access(shared, |s| domain::begin(s, repo_id, intent_id))?;
    let round = Round {
        repo_id: repo_id.into(),
        intent_id: intent_id.into(),
        revision_id: intent.target.change_set_revision_id.clone(),
        round: intent.round,
    };
    let registration = access(shared, |s| require_active(s, repo_id))?;
    let observed = registration.observed.clone().ok_or_else(|| {
        reject(
            "REPO_PENDING",
            "platform binding not confirmed",
            "confirm_repo",
        )
    })?;
    // The intent was authorized under one platform binding version; a rebind since then
    // is a new authorization, not a new endpoint for this one.
    let binding_version = access(shared, |s| {
        Ok(s.get(&repo::binding(repo_id).key)?.map(|r| r.version))
    })?;
    if binding_version != Some(intent.binding_version as i64) {
        let outcome = Outcome::Attention(Attention {
            code: "BINDING_CHANGED".into(),
            message: "the Repo's platform binding changed since this publish was authorized; nothing was sent".into(),
            recovery_action: "authorize_a_new_dispatch_under_the_current_binding".into(),
            details: json!({"authorized": intent.binding_version, "current": binding_version}),
        });
        access(shared, |s| round.confirm(s, outcome))?;
        return Ok(());
    }
    let Some(local) = registration.prepared.local.as_ref() else {
        let outcome = Outcome::Failed(Attention {
            code: "PUSH_SOURCE_MISSING".into(),
            message: "this Repo has no local repository on this machine to push the revision from"
                .into(),
            recovery_action: "register_repo_from_local_repository".into(),
            details: Value::Null,
        });
        access(shared, |s| round.confirm(s, outcome))?;
        return Ok(());
    };
    let site = Site {
        local_path: local.path.clone(),
        clone_url: observed.clone_url.clone(),
        full_name: observed.full_name.clone(),
    };
    let (connection, credential) = connect(&registration)?;
    // A round superseded underneath this pass (a newer revision admitted before the worker
    // took it) refuses every persisted step with FROZEN_INPUT_CHANGED; the pass ends there
    // and the next tick reads the intent afresh.
    let outcome = match publish(shared, intent, &round, &site, &connection, &credential) {
        Ok(outcome) => outcome,
        Err(error) if error.code == "FROZEN_INPUT_CHANGED" => return Ok(()),
        Err(error) => return Err(error),
    };
    if let Some(outcome) = outcome {
        match access(shared, |s| round.confirm(s, outcome)) {
            Ok(_) => {}
            Err(error) if error.code == "FROZEN_INPUT_CHANGED" => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn now() -> u64 {
    crate::dispatch::now_ms()
}

/// The intent, revision and round one worker pass is about; every persisted step names them.
struct Round {
    repo_id: String,
    intent_id: String,
    revision_id: String,
    round: u64,
}

impl Round {
    fn confirm(&self, store: &mut Store, outcome: Outcome) -> Result<domain::Intent> {
        domain::confirm(
            store,
            &self.repo_id,
            &self.intent_id,
            &self.revision_id,
            self.round,
            outcome,
            now(),
        )
    }
}

/// Both stages with readback. `Ok(None)` means a stage was confirmed but the round is not
/// finished and nothing needs recording beyond what the stage recorded.
fn publish(
    shared: &Shared,
    intent: domain::Intent,
    round: &Round,
    site: &Site,
    connection: &dyn PlatformTarget,
    credential: &Credential,
) -> Result<Option<Outcome>> {
    let (repo_id, intent_id) = (round.repo_id.as_str(), round.intent_id.as_str());
    // The commit the branch will carry: frozen once per revision, before anything leaves.
    let commit = match intent.target.commit_sha.clone() {
        Some(commit) => commit,
        None => {
            let source = repo::integration::AdmittedRevision {
                change_set_revision_id: intent.target.change_set_revision_id.clone(),
                change_set_id: intent.change_set_id.clone(),
                parent_revision_id: None,
                base_commit_sha: intent.target.base_commit_sha.clone(),
                result_tree_sha: intent.target.result_tree_sha.clone(),
                producer_ref: Value::Null,
                review_subject_digest: String::new(),
            };
            match crate::integration::result_commit(&site.local_path, &source)? {
                Ok(commit) => {
                    access(shared, |s| {
                        domain::freeze_commit(
                            s,
                            repo_id,
                            intent_id,
                            &round.revision_id,
                            round.round,
                            &commit,
                        )
                    })?;
                    commit
                }
                Err(reason) => {
                    return Ok(Some(Outcome::Attention(Attention {
                        code: reason.code,
                        message: reason.message,
                        recovery_action: reason.recovery_action,
                        details: reason.details,
                    })));
                }
            }
        }
    };
    let reference = format!("refs/heads/{}", intent.branch);
    let git = repo::git::Git::discover()?;

    // Stage 1: the branch carries the frozen commit.
    if intent.push.confirmed_commit.as_deref() != Some(commit.as_str()) {
        let expected = intent.push.confirmed_commit.clone();
        let remote = |git: &repo::git::Git| {
            git.remote_ref(&site.local_path, &site.clone_url, &reference, credential)
        };
        let mut head = remote(&git)?;
        if head.as_deref() != Some(commit.as_str()) {
            if head != expected {
                return Ok(Some(Outcome::Attention(diverged(
                    &intent.branch,
                    &expected,
                    &head,
                ))));
            }
            access(shared, |s| {
                domain::mark_push_dispatched(
                    s,
                    repo_id,
                    intent_id,
                    &round.revision_id,
                    round.round,
                    true,
                )
            })?;
            let pushed = git.push_ref(
                &site.local_path,
                &site.clone_url,
                &commit,
                &reference,
                expected.as_deref(),
                credential,
            );
            head = remote(&git)?;
            match pushed {
                Ok(PushOutcome::Pushed)
                | Ok(PushOutcome::StaleLease)
                | Ok(PushOutcome::Unknown(_))
                | Err(_) => {}
            }
            if head.as_deref() != Some(commit.as_str()) {
                // The remote says what happened: still at the expected value is a proven
                // no-write, anything else is someone else's branch now.
                if head == expected {
                    access(shared, |s| {
                        domain::mark_push_dispatched(
                            s,
                            repo_id,
                            intent_id,
                            &round.revision_id,
                            round.round,
                            false,
                        )
                    })?;
                    let detail = match pushed {
                        Ok(PushOutcome::Unknown(text)) => text,
                        Ok(_) => "the push did not land".into(),
                        Err(error) => error.message,
                    };
                    return Ok(Some(Outcome::Attention(Attention {
                        code: "PUSH_FAILED".into(),
                        message: format!("the platform did not take the branch update: {detail}"),
                        recovery_action: "inspect_platform_then_retry_same_intent".into(),
                        details: json!({"branch": intent.branch, "commit": commit}),
                    })));
                }
                return Ok(Some(Outcome::Attention(diverged(
                    &intent.branch,
                    &expected,
                    &head,
                ))));
            }
        }
        access(shared, |s| {
            domain::confirm_push(
                s,
                repo_id,
                intent_id,
                &round.revision_id,
                round.round,
                &commit,
                now(),
            )
        })?;
    }

    // 测试缝（第 9 包验收第 4 条）：分支已推送并确认，平台还没被问过评审请求——与平台失联
    // 时同形状的 Unknown。标记删掉后，下面这段照既有路径回读分支、只建一条请求。
    if crate::test_seams::held("hold-publish-after-push") {
        return Ok(Some(Outcome::Unknown(Attention {
            code: "PLATFORM_UNAVAILABLE".into(),
            message: "the platform was not asked for the review request".into(),
            recovery_action: "retry_same_intent_when_platform_is_up".into(),
            details: json!({"test_seam": "hold-publish-after-push"}),
        })));
    }

    // Stage 2: one review request from the branch into the target, carrying that commit.
    let base = intent.policy.policy.target_branch.clone();
    let title = format!(
        "hctl2: ChangeSet {} · revision {}",
        short(&intent.change_set_id),
        short(&intent.target.change_set_revision_id)
    );
    let body = association(&intent, &commit);
    let existing = match connection.find_review_request(&site.full_name, &base, &intent.branch) {
        Ok(existing) => existing,
        Err(error) => return Ok(Some(Outcome::Attention(unavailable(error)))),
    };
    let request = match existing {
        Some(request) if request.merged || request.state != "open" => {
            return Ok(Some(Outcome::Attention(Attention {
                code: "REVIEW_REQUEST_CLOSED".into(),
                message: format!(
                    "review request #{} for this branch is {}; a closed request is not reopened or replaced by control",
                    request.index,
                    if request.merged { "merged" } else { "closed" }
                ),
                recovery_action: "reopen_or_authorize_a_new_dispatch".into(),
                details: json!({"index": request.index, "state": request.state, "merged": request.merged}),
            })));
        }
        Some(request) => {
            if intent
                .review
                .index
                .is_some_and(|index| index != request.index)
            {
                return Ok(Some(Outcome::Attention(Attention {
                    code: "REVIEW_REQUEST_CHANGED".into(),
                    message: "the platform now has another review request for this branch than the one recorded".into(),
                    recovery_action: "inspect_platform".into(),
                    details: json!({"recorded": intent.review.index, "found": request.index}),
                })));
            }
            // The audit association is part of publishing: a refused update leaves the round
            // open with the reason, to be retried once the platform lets it through.
            if let Err(error) =
                connection.update_review_request(&site.full_name, request.index, &title, &body)
            {
                if error.code == "PLATFORM_UNAVAILABLE" {
                    return Ok(Some(Outcome::Attention(unavailable(error))));
                }
                return Ok(Some(Outcome::Attention(Attention {
                    code: "AUDIT_UPDATE_REJECTED".into(),
                    message: format!(
                        "the platform refused to update the review request's title and body: {}",
                        error.message
                    ),
                    recovery_action: "inspect_platform_permissions_then_retry_same_intent".into(),
                    details: json!({"platform_code": error.code, "index": request.index}),
                })));
            }
            request
        }
        None => {
            access(shared, |s| {
                domain::mark_review_dispatched(
                    s,
                    repo_id,
                    intent_id,
                    &round.revision_id,
                    round.round,
                    true,
                )
            })?;
            let created = connection.create_review_request(
                &site.full_name,
                &base,
                &intent.branch,
                &title,
                &body,
            );
            match created {
                Ok(_) => {}
                Err(error) if error.code == "PLATFORM_UNAVAILABLE" => {
                    // May or may not have been created: readback below decides, the create
                    // is not repeated until it says there is none.
                }
                Err(error) => {
                    access(shared, |s| {
                        domain::mark_review_dispatched(
                            s,
                            repo_id,
                            intent_id,
                            &round.revision_id,
                            round.round,
                            false,
                        )
                    })?;
                    return Ok(Some(Outcome::Attention(Attention {
                        code: "REVIEW_REQUEST_REJECTED".into(),
                        message: error.message,
                        recovery_action: "inspect_platform_then_retry_same_intent".into(),
                        details: json!({"platform_code": error.code, "branch": intent.branch, "target_branch": base}),
                    })));
                }
            }
            match connection.find_review_request(&site.full_name, &base, &intent.branch) {
                Ok(Some(request)) => request,
                Ok(None) => {
                    return Ok(Some(Outcome::Unknown(Attention {
                        code: "RESULT_UNKNOWN".into(),
                        message: "the review request does not read back yet; it is read back, not created again".into(),
                        recovery_action: "read_back_review_request_with_same_intent".into(),
                        details: json!({"branch": intent.branch, "target_branch": base}),
                    })));
                }
                Err(error) => return Ok(Some(Outcome::Unknown(unavailable(error)))),
            }
        }
    };
    if request.head_sha.as_deref() != Some(commit.as_str()) {
        return Ok(Some(Outcome::Unknown(Attention {
            code: "RESULT_UNKNOWN".into(),
            message: "the review request does not carry the pushed commit yet".into(),
            recovery_action: "read_back_review_request_with_same_intent".into(),
            details: json!({"index": request.index, "review_head": request.head_sha, "pushed": commit}),
        })));
    }
    Ok(Some(Outcome::Published {
        index: request.index,
        platform_commit_sha: commit,
        readback: json!({"review_request": request.raw, "branch": intent.branch}),
    }))
}

fn diverged(branch: &str, expected: &Option<String>, head: &Option<String>) -> Attention {
    Attention {
        code: "BRANCH_DIVERGED".into(),
        message: "the publish branch is not where this intent left it; nothing was written".into(),
        recovery_action: "inspect_branch_then_retry_or_authorize_new_dispatch".into(),
        details: json!({"branch": branch, "expected": expected, "observed": head}),
    }
}

fn unavailable(error: repo::StoreError) -> Attention {
    Attention {
        code: "PLATFORM_UNAVAILABLE".into(),
        message: format!("the platform did not answer: {}", error.message),
        recovery_action: "retry_same_intent_when_platform_is_up".into(),
        details: json!({"platform_code": error.code}),
    }
}

fn short(id: &str) -> &str {
    let start = id.find('-').map_or(0, |i| i + 1);
    &id[start..(start + 12).min(id.len())]
}

/// The minimal audit association the frozen scope allows in the request body.
fn association(intent: &domain::Intent, commit: &str) -> String {
    format!(
        "Published by hctl2 for review.\n\n- ChangeSet: `{}`\n- ChangeSet Revision: `{}`\n- base commit: `{}`\n- result tree: `{}`\n- platform commit: `{}`\n- publish intent: `{}` (round {})\n\nThis request is the review of that exact revision; merging it is authorized separately.",
        intent.change_set_id,
        intent.target.change_set_revision_id,
        intent.target.base_commit_sha,
        intent.target.result_tree_sha,
        commit,
        intent.intent_id,
        intent.round
    )
}
