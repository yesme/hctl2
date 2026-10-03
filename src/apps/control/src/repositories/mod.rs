//! Repo RPC orchestration. SQLite is locked only for admission/confirmation, never network I/O.
use crate::scm as platform;

use std::path::Path;
use std::sync::Arc;

use repo::git::Git;
use repo::{Lifecycle, Platform, Prepared, Register, Registration, Result, reject};
use serde_json::{Value, json};
use store::{EffectState, Store, TrustedActor};
use tokio::sync::Mutex;

use crate::services::Supervisor;

type SharedStore = Arc<Mutex<Option<Store>>>;

fn access<T>(shared: &SharedStore, f: impl FnOnce(&mut Store) -> Result<T>) -> Result<T> {
    let mut slot = shared.blocking_lock();
    f(slot
        .as_mut()
        .ok_or_else(|| reject("STORE_NOT_READY", "storage not ready", "check_status"))?)
}

pub(super) fn preview(shared: &SharedStore, operation: &str, payload: &Value) -> Result<Value> {
    if operation == "repo.register" {
        let idem = field(payload, "registration_key")?;
        let request: Register = serde_json::from_value(payload["request"].clone())?;
        let existing = access(shared, |store| {
            let id = repo::registration_id(store.control_id(), idem);
            if store.get(&repo::key(&id))?.is_some() {
                Ok(Some(repo::get(store, &id)?))
            } else {
                Ok(None)
            }
        })?;
        let prepared = if let Some(existing) = existing {
            if request != existing.prepared.request {
                return Err(reject(
                    "IDEMPOTENCY_CONFLICT",
                    "registration key already has another input",
                    "use_original_command",
                ));
            }
            existing.prepared
        } else {
            let local = request
                .local
                .as_ref()
                .map(|input| Git::discover()?.inspect(input))
                .transpose()?;
            repo::prepare(request, local)?
        };
        Ok(json!({"prepared":prepared,"dangerous":true,
            "effects":["record pending Repo", "read/create declared platform", "deliver only frozen refs", "activate only after readback"],
            "in_place":prepared.request.local.as_ref().is_some_and(|l| l.in_place),
            "requires_identity_confirmation":prepared.platform == Platform::Local}))
    } else if operation == "repo.grant" {
        let reg = access(shared, |store| {
            repo::require_active(store, field(payload, "repo_id")?)
        })?;
        require_hosted(&reg)?;
        let username = human_username(payload)?;
        let permission = collaborator_permission(payload)?;
        Ok(json!({"registration":reg,"dangerous":true,"operation":operation,
            "username":username,"permission":permission,
            "effects":[
                format!("ensure an ordinary platform account named {username}, reusing one that already exists"),
                "print an initial password once, and only when this call created the account",
                format!("grant {permission} collaboration on the registered platform repository"),
            ],
            "platform_account_mapping_recorded":false}))
    } else {
        let reg = access(shared, |store| repo::get(store, field(payload, "repo_id")?))?;
        Ok(
            json!({"registration":reg,"dangerous":true,"operation":operation,
            "abandon_leaves_platform_repository": operation == "repo.abandon"}),
        )
    }
}

pub(super) fn query(shared: &SharedStore, kind: &str, payload: &Value) -> Result<Value> {
    access(shared, |store| {
        if kind == "repo.list" {
            Ok(json!({"items":repo::list(store)?}))
        } else {
            Ok(serde_json::to_value(repo::get(
                store,
                field(payload, "repo_id")?,
            )?)?)
        }
    })
}

pub(super) fn submit(
    shared: &SharedStore,
    services: &Supervisor,
    root: &Path,
    actor: &TrustedActor,
    request: &proto::SubmitRequest,
    payload: &Value,
    preview: &Value,
) -> Result<Value> {
    let registration = match request.operation.as_str() {
        "repo.register" => {
            let idem = field(payload, "registration_key")?;
            if request.idempotency_key != idem
                || request.command_id != format!("repo.register:{idem}")
            {
                return Err(reject(
                    "INVALID_INPUT",
                    "registration envelope must match registration_key",
                    "use_original_command",
                ));
            }
            let prepared: Prepared = serde_json::from_value(preview["prepared"].clone())?;
            let exists = access(shared, |store| {
                Ok(store
                    .get(&repo::key(&repo::registration_id(store.control_id(), idem)))?
                    .is_some())
            })?;
            if !exists
                && let (Some(input), Some(snapshot)) = (&prepared.request.local, &prepared.local)
            {
                Git::discover()?.recheck(input, snapshot)?;
            }
            access(shared, |store| {
                repo::admit(store, actor, &request.command_id, idem, prepared)
            })?
        }
        "repo.resume" => access(shared, |store| repo::get(store, field(payload, "repo_id")?))?,
        "repo.grant" => {
            let id = field(payload, "repo_id")?;
            let username = human_username(payload)?;
            let permission = collaborator_permission(payload)?;
            let reg = access(shared, |store| repo::require_active(store, id))?;
            require_hosted(&reg)?;
            let hosted = platform::Hosted::connect(root, &reg.config.control_id, services)?;
            let observed = hosted.repository(&reg, false)?.ok_or_else(|| {
                reject(
                    "PLATFORM_UNAVAILABLE",
                    "registered platform repository missing",
                    "read_back_original_intent",
                )
            })?;
            let paths = platform::hosted_paths(services)?;
            let account = platform::ensure_human_account(&paths, username)?;
            if let Err(error) =
                hosted.grant_collaborator(&observed.full_name, username, permission)
            {
                // The account can outlive a failed grant. A freshly printed initial password
                // is discarded rather than carried in an error body, so say that it was.
                return Err(if account.created {
                    reject(
                        error.code,
                        format!(
                            "{}; account {username} now exists and its initial password was discarded, reset it with the native admin CLI before retrying",
                            error.message
                        ),
                        error.recovery_action,
                    )
                } else {
                    error
                });
            }
            return Ok(json!({
                "repo_id": reg.repo_id,
                "full_name": observed.full_name,
                "username": account.username,
                "account_created": account.created,
                "initial_password": account.initial_password,
                "permission": permission,
            }));
        }
        "repo.confirm" | "repo.abandon" => {
            let id = field(payload, "repo_id")?;
            let expected = payload["version"].as_i64().ok_or_else(|| {
                reject(
                    "INVALID_INPUT",
                    "version required",
                    "preview_current_version",
                )
            })?;
            let choice = if request.operation == "repo.abandon" {
                repo::FinishChoice::Abandon
            } else {
                repo::FinishChoice::Confirm(field(payload, "platform_repo_id")?)
            };
            return access(shared, |store| {
                repo::finish(
                    store,
                    actor,
                    id,
                    expected,
                    choice,
                    &request.command_id,
                    &request.idempotency_key,
                )
                .and_then(|reg| Ok(serde_json::to_value(reg)?))
            });
        }
        _ => {
            return Err(reject(
                "INVALID_INPUT",
                "unknown Repo command",
                "correct_input",
            ));
        }
    };
    let id = registration.repo_id.clone();
    let outcome = drive(shared, services, root, actor, registration);
    let current = access(shared, |store| repo::get(store, &id))?;
    Ok(match outcome {
        Ok(()) => json!({"registration":current}),
        Err(error) => {
            json!({"registration":current,"error":{"code":error.code,"message":error.message,"recovery_action":error.recovery_action}})
        }
    })
}

fn drive(
    shared: &SharedStore,
    services: &Supervisor,
    root: &Path,
    actor: &TrustedActor,
    mut reg: Registration,
) -> Result<()> {
    if reg.abandoned {
        return reconcile_residual(shared, services, root, actor, &reg);
    }
    if reg.lifecycle == Lifecycle::Active {
        return Ok(());
    }
    if reg.observed.is_none() {
        // Service/bootstrap failure has not attempted repository creation. Keep the intent
        // pending until the adapter is ready; it can then be safely abandoned as unsent.
        let hosted = if reg.prepared.platform == Platform::Local {
            Some(platform::Hosted::connect(
                root,
                &reg.config.control_id,
                services,
            )?)
        } else {
            None
        };
        let state = access(shared, |store| {
            repo::begin_step(store, actor, &reg.repo_id, "platform")
        })?;
        let observed = if let Some(hosted) = hosted {
            hosted
                .repository(&reg, state == EffectState::Pending)?
                .ok_or_else(|| {
                    reject(
                        "RESULT_UNKNOWN",
                        "original platform repository not found; unknown creation is not resent",
                        "read_back_original_intent",
                    )
                })?
        } else {
            platform::github(&reg, services)?
        };
        reg = access(shared, |store| {
            repo::confirm_platform(store, &reg.repo_id, observed)
        })?;
    }
    if reg.delivered {
        return Ok(());
    }
    let hosted = platform::Hosted::connect(root, &reg.config.control_id, services)?;
    let observed = hosted.repository(&reg, false)?.ok_or_else(|| {
        reject(
            "RESULT_UNKNOWN",
            "original platform repository missing",
            "read_back_original_intent",
        )
    })?;
    reg = access(shared, |store| {
        repo::refresh_platform(store, &reg.repo_id, observed.clone())
    })?;
    access(shared, |store| {
        repo::begin_step(store, actor, &reg.repo_id, "delivery")
    })?;
    if let Some(snapshot) = &reg.prepared.local {
        let git = Git::discover()?;
        let input = reg.prepared.request.local.as_ref().unwrap();
        // Empty repositories only create the platform repository: no Git contact is needed.
        if !snapshot.refs.is_empty()
            && !git.delivered(
                root,
                snapshot,
                &observed.clone_url,
                Some((&hosted.username, &hosted.token)),
            )?
        {
            // Privileged Git never executes inside the user's repository/configuration.
            let path = git.copy(snapshot, &root.join("imports").join(&reg.repo_id))?;
            git.deliver(
                &path,
                snapshot,
                &observed.clone_url,
                Some((&hosted.username, &hosted.token)),
            )?;
        }
        if input.in_place {
            git.switch_remote(snapshot, &observed.clone_url)?;
        }
    }
    access(shared, |store| repo::confirm_delivery(store, &reg.repo_id))?;
    Ok(())
}

fn reconcile_residual(
    shared: &SharedStore,
    services: &Supervisor,
    root: &Path,
    actor: &TrustedActor,
    reg: &Registration,
) -> Result<()> {
    let unresolved = access(shared, |store| {
        Ok(store
            .pending_effects()?
            .iter()
            .any(|id| id.starts_with(&format!("repo:{}:", reg.repo_id))))
    })?;
    if !unresolved {
        return Ok(());
    }
    let (observed, delivered) = if reg.prepared.platform == Platform::Local {
        let hosted = platform::Hosted::connect(root, &reg.config.control_id, services)?;
        let observed = hosted.repository(reg, false)?.ok_or_else(|| {
            reject(
                "RESULT_UNKNOWN",
                "original repository not observed; retain its conflict scope",
                "read_back_original_intent",
            )
        })?;
        let delivered = match &reg.prepared.local {
            Some(snapshot) if !snapshot.refs.is_empty() => Git::discover()?.delivered(
                root,
                snapshot,
                &observed.clone_url,
                Some((&hosted.username, &hosted.token)),
            )?,
            _ => true,
        };
        (observed, delivered)
    } else {
        (platform::github(reg, services)?, true)
    };
    access(shared, |store| {
        repo::reconcile_residual(store, actor, &reg.repo_id, observed, delivered)
    })?;
    Ok(())
}

fn field<'a>(value: &'a Value, field: &str) -> Result<&'a str> {
    value[field]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| reject("INVALID_INPUT", format!("missing {field}"), "correct_input"))
}

/// Only the hosted local platform has accounts control creates; GitHub keeps its own.
fn require_hosted(reg: &Registration) -> Result<()> {
    if reg.prepared.platform == Platform::Local {
        Ok(())
    } else {
        Err(reject(
            "INVALID_INPUT",
            "collaboration is only granted on the hosted local platform",
            "correct_input",
        ))
    }
}

/// The username is interpolated into a platform API path, so it is restricted here instead
/// of relying on the platform to reject a path-shaped name after the fact.
fn human_username(payload: &Value) -> Result<&str> {
    let username = field(payload, "username")?;
    let valid = username.len() <= 40
        && username
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        && !username.starts_with('.')
        && !username.ends_with('.')
        && !username.contains("..");
    if valid {
        Ok(username)
    } else {
        Err(reject(
            "INVALID_INPUT",
            "username must be at most 40 characters of ASCII letters, digits, '-', '_' or '.', without a leading, trailing or doubled '.'",
            "correct_input",
        ))
    }
}

fn collaborator_permission(payload: &Value) -> Result<&str> {
    let permission = field(payload, "permission")?;
    if matches!(permission, "read" | "write") {
        Ok(permission)
    } else {
        Err(reject(
            "INVALID_INPUT",
            "permission must be read or write",
            "correct_input",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(result: Result<&str>) -> &'static str {
        result.unwrap_err().code
    }

    #[test]
    fn a_username_that_would_reshape_the_platform_path_never_reaches_the_platform() {
        // The username is interpolated into repos/{full}/collaborators/{username}, so a
        // path-shaped name is rejected here rather than by the platform after the request.
        for username in [
            "alice",
            "alice.smith",
            "alice-smith_1",
            &"a".repeat(40),
        ] {
            assert_eq!(
                human_username(&json!({"username":username})).unwrap(),
                username
            );
        }
        for username in [
            "a/b",
            "../alice",
            "alice/../admin",
            ".alice",
            "alice.",
            "al..ice",
            &"a".repeat(41),
            "al ice",
            "alicé",
            "",
        ] {
            assert_eq!(
                code(human_username(&json!({"username":username}))),
                "INVALID_INPUT",
                "{username} must be refused"
            );
        }
        // Missing and blank are refused by `field`, not by the charset rule.
        assert_eq!(code(human_username(&json!({}))), "INVALID_INPUT");
        assert_eq!(
            code(human_username(&json!({"username":"  "}))),
            "INVALID_INPUT"
        );
    }

    #[test]
    fn collaboration_permission_is_read_or_write_and_nothing_else() {
        assert_eq!(
            collaborator_permission(&json!({"permission":"read"})).unwrap(),
            "read"
        );
        assert_eq!(
            collaborator_permission(&json!({"permission":"write"})).unwrap(),
            "write"
        );
        // `admin` would hand over the repository; `owner` is not a Gitea collaborator level.
        for permission in ["admin", "owner", "READ", "none", ""] {
            assert_eq!(
                code(collaborator_permission(&json!({"permission":permission}))),
                "INVALID_INPUT",
                "{permission} must be refused"
            );
        }
        assert_eq!(
            code(collaborator_permission(&json!({}))),
            "INVALID_INPUT"
        );
    }
}
