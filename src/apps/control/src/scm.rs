//! Native gh / tea / Gitea administration. No HTTP client or account system of our own.
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use repo::git::run;
use repo::{PlatformObservation, Registration, Result, StoreError, reject};
use serde_json::{Value, json};

use crate::services::Supervisor;

pub(crate) struct Hosted {
    tea: PathBuf,
    pub url: String,
    pub username: String,
    pub token: String,
    credential_ref: String,
}
impl Hosted {
    /// Reuse an explicitly provisioned connection. Observing a source must not start
    /// a stopped service, create an account, or replace a missing credential.
    pub fn existing(root: &Path, control_id: &str, services: &Supervisor) -> Result<Self> {
        let (install, state) = services.gitea_paths().ok_or_else(|| {
            reject(
                "PLATFORM_NOT_INSTALLED",
                "hosted Gitea not installed",
                "install_package",
            )
        })?;
        let url = hosted_url(&state.join("config/gitea/app.ini"))?;
        let credential_ref = format!("gitea:{control_id}:admin");
        let token = crate::config::secret_store(root)
            .map_err(|_| {
                reject(
                    "CREDENTIAL_UNAVAILABLE",
                    "stored Gitea credential unavailable",
                    "restore_secret_store",
                )
            })?
            .get(&credential_ref)
            .map_err(|_| {
                reject(
                    "CREDENTIAL_UNAVAILABLE",
                    "stored Gitea credential unavailable",
                    "restore_secret_store",
                )
            })?;
        let token = String::from_utf8(token).map_err(|_| {
            reject(
                "CREDENTIAL_UNAVAILABLE",
                "invalid stored Gitea token",
                "restore_secret_store",
            )
        })?;
        Ok(Self {
            tea: install.join("libexec/hctl2/tea"),
            url,
            username: format!("hctl-{}", &control_id[..16.min(control_id.len())]),
            token,
            credential_ref,
        })
    }
    #[cfg(test)]
    pub(crate) fn fixture(tea: PathBuf, url: String, username: String, token: String) -> Self {
        Self {
            tea,
            url,
            username,
            token,
            credential_ref: "test-only".into(),
        }
    }
    pub fn connect(root: &Path, control_id: &str, services: &Supervisor) -> Result<Self> {
        let (install, state) = services.gitea_paths().ok_or_else(|| {
            reject(
                "PLATFORM_NOT_INSTALLED",
                "hosted Gitea requires an installed package",
                "install_package",
            )
        })?;
        services
            .consume("gitea")
            .map_err(|e| reject(e.code, e.message, e.recovery_action))?;
        // One deadline for the whole bootstrap: liveness first, then the two checks that
        // decide whether *this* instance is actually usable.
        let deadline = Instant::now() + GITEA_BOOTSTRAP_TIMEOUT;
        // Liveness only. The supervisor's probe is an HTTP GET on Gitea's machine-wide
        // port, and it is not a usability verdict: `/api/healthz` only pings the
        // database, so it answers before the first migration creates its tables, and any
        // other instance holding the port answers it while ours never binds. The admin
        // CLI and the API checks below are what decide.
        loop {
            if services
                .snapshot()
                .hosted
                .iter()
                .any(|s| s.name == "gitea" && s.available())
            {
                break;
            }
            if Instant::now() >= deadline {
                return Err(reject(
                    "PLATFORM_NOT_READY",
                    "Gitea consumed but readiness not confirmed",
                    "retry_registration",
                ));
            }
            std::thread::sleep(GITEA_BOOTSTRAP_POLL);
        }
        let config = state.join("config/gitea/app.ini");
        let url = hosted_url(&config)?;
        let gitea = install.join("libexec/hctl2/gitea");
        let username = format!("hctl-{}", &control_id[..16.min(control_id.len())]);
        let admin = || {
            let mut cmd = Command::new(&gitea);
            cmd.arg("--config")
                .arg(&config)
                .arg("--work-path")
                .arg(state.join("data/gitea"))
                .args(["admin", "user"]);
            cmd
        };
        // Gate on this root's own admin interface: it opens this root's database, so it
        // is what proves the first migration finished (`no such table: user` was the
        // observed failure) and it is unaffected by whichever process holds the port.
        let users = retry_bootstrap(
            deadline,
            || {
                let output = run(admin().arg("list"), None)?;
                if output.status.success() {
                    Ok(output)
                } else {
                    Err(reject(
                        "PLATFORM_BOOTSTRAP",
                        format!(
                            "cannot list hosted platform accounts: {}",
                            native_detail(&output.stderr)
                        ),
                        "inspect_gitea_log",
                    ))
                }
            },
            |_| true,
            || rearm(services),
        )?;
        let exists = String::from_utf8_lossy(&users.stdout)
            .lines()
            .any(|l| l.split_whitespace().nth(1) == Some(&username));
        if !exists {
            let created = run(
                admin().args([
                    "create",
                    "--username",
                    &username,
                    "--email",
                    &format!("{username}@localhost"),
                    "--admin",
                    "--random-password",
                    "--random-password-length",
                    "40",
                    "--must-change-password=false",
                ]),
                None,
            )?;
            if !created.status.success() {
                return Err(reject(
                    "PLATFORM_BOOTSTRAP",
                    "cannot create hosted control account",
                    "inspect_gitea_log",
                ));
            }
        }
        let secrets = crate::config::secret_store(root).map_err(|_| {
            reject(
                "CREDENTIAL_UNAVAILABLE",
                "stored Gitea credential unavailable",
                "restore_secret_store",
            )
        })?;
        let credential_ref = format!("gitea:{control_id}:admin");
        let token = match secrets.get(&credential_ref) {
            Ok(bytes) => String::from_utf8(bytes).map_err(|_| {
                reject(
                    "CREDENTIAL_UNAVAILABLE",
                    "invalid stored Gitea token",
                    "restore_secret_store",
                )
            })?,
            Err(_) => {
                // Fixed token name: if the process died before saving, do not silently mint another.
                let result = run(
                    admin().args([
                        "generate-access-token",
                        "--username",
                        &username,
                        "--token-name",
                        "hctl-control",
                        "--scopes",
                        "write:repository,write:issue,write:user",
                        "--raw",
                    ]),
                    None,
                )?;
                if !result.status.success() {
                    return Err(reject(
                        "CREDENTIAL_UNAVAILABLE",
                        "token could not be materialized; restore its SecretStore entry or explicitly revoke the lost hctl-control token before retry",
                        "restore_secret_store",
                    ));
                }
                let stdout = String::from_utf8_lossy(&result.stdout);
                let token = stdout
                    .lines()
                    .find(|l| l.len() == 40 && l.bytes().all(|c| c.is_ascii_hexdigit()))
                    .ok_or_else(|| {
                        reject(
                            "CREDENTIAL_UNAVAILABLE",
                            "native token result could not be read",
                            "restore_secret_store",
                        )
                    })?
                    .to_owned();
                secrets.set(&credential_ref, token.as_bytes())?;
                token
            }
        };
        let hosted = Self {
            tea: install.join("libexec/hctl2/tea"),
            url,
            username,
            token,
            credential_ref,
        };
        // The API must answer for the instance just provisioned. A token minted in this
        // root's database is rejected with 401 by any other instance holding the port,
        // and a connection ours never accepted reports no status at all; both clear once
        // the port is ours again, so retry those inside the same deadline.
        let mut restarted: Option<Instant> = None;
        let who = retry_bootstrap(
            deadline,
            || match hosted.api("GET", "user", None)? {
                Some(who) => Ok(who),
                None => Err(reject(
                    "CREDENTIAL_UNAVAILABLE",
                    "platform account missing",
                    "restore_secret_store",
                )),
            },
            |error| error.code == "PLATFORM_UNAVAILABLE",
            || {
                let _ = services.ensure_up();
                // An instance that lost the port race stays alive without a listener, so
                // starting it again changes nothing: it needs a restart, rate-limited so
                // the port has time to free up in between.
                if restarted.is_none_or(|last| last.elapsed() >= GITEA_RESTART_INTERVAL) {
                    let _ = services.restart("gitea");
                    restarted = Some(Instant::now());
                }
            },
        )?;
        if who["login"].as_str() != Some(&hosted.username)
            || who["is_admin"].as_bool() != Some(true)
        {
            return Err(reject(
                "CREDENTIAL_UNAVAILABLE",
                "token does not identify the hosted control admin",
                "restore_secret_store",
            ));
        }
        Ok(hosted)
    }
    pub(crate) fn api(
        &self,
        method: &str,
        path: &str,
        body: Option<Value>,
    ) -> Result<Option<Value>> {
        let mut cmd = Command::new(&self.tea);
        cmd.env("GITEA_INSTANCE_URL", &self.url)
            .env("GITEA_TOKEN", &self.token)
            .env_remove("GITEA_INSTANCE_INSECURE")
            .env_remove("GITEA_INSTANCE_SSH_HOST")
            .env("NO_COLOR", "1")
            .args(["api", "--include", "-X", method]);
        if body.is_some() {
            cmd.args(["--data", "@-"]);
        }
        cmd.arg(path);
        let output = run(&mut cmd, body.map(|v| v.to_string().into_bytes()))?;
        // tea emits the HTTP status on stderr; JSON error text is not a status code.
        let status = String::from_utf8_lossy(&output.stderr)
            .lines()
            .find(|l| l.starts_with("HTTP/"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse::<u16>().ok());
        if status == Some(404) && method == "GET" {
            return Ok(None);
        }
        if !output.status.success() || !status.is_some_and(|s| (200..300).contains(&s)) {
            return Err(reject(
                if method != "GET" && matches!(status, Some(401 | 403 | 404)) {
                    "NATIVE_REJECTED"
                } else if method != "GET" && status == Some(409) {
                    "NATIVE_CONFLICT"
                } else {
                    "PLATFORM_UNAVAILABLE"
                },
                format!(
                    "tea API {method} {path} not confirmed (HTTP {status:?}, exit {:?})",
                    output.status.code()
                ),
                "read_back_original_intent",
            ));
        }
        Ok(Some(if output.stdout.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&output.stdout)?
        }))
    }
    pub fn repository(
        &self,
        registration: &Registration,
        create: bool,
    ) -> Result<Option<PlatformObservation>> {
        let name = registration
            .prepared
            .request
            .platform_path
            .as_deref()
            .unwrap();
        let full_name = format!("{}/{name}", self.username);
        let endpoint = format!("repos/{full_name}");
        let marker = format!(
            "HCTL registration {}:{}",
            registration.config.control_id, registration.repo_id
        );
        let mut value = self.api("GET", &endpoint, None)?;
        if value.is_none() && create {
            let _ = self.api(
                "POST",
                "user/repos",
                Some(json!({"name":name,"description":marker,"private":true,"auto_init":false,
                    "default_branch": registration.prepared.local.as_ref().map(|s| s.head_branch.trim_start_matches("refs/heads/")).unwrap_or("main")})),
            )?;
            value = self.api("GET", &endpoint, None)?;
        }
        let Some(value) = value else {
            return Ok(None);
        };
        if value["description"].as_str() != Some(&marker)
            || value["full_name"].as_str() != Some(&full_name)
        {
            return Err(reject(
                "PLATFORM_NAME_CONFLICT",
                "existing platform repository does not match original registration correlation",
                "choose_new_name",
            ));
        }
        let clone_url = value["clone_url"]
            .as_str()
            .ok_or_else(|| reject("PLATFORM_READBACK", "clone URL missing", "retry_read"))?;
        // A repository response must not redirect a privileged Git credential to another service.
        if clone_url != format!("{}/{full_name}.git", self.url) {
            return Err(reject(
                "PLATFORM_READBACK",
                "clone target differs from hosted instance",
                "inspect_platform_repository",
            ));
        }
        Ok(Some(PlatformObservation {
            instance: self.url.clone(),
            stable_id: numeric_id(&value["id"])?,
            full_name,
            clone_url: clone_url.into(),
            account_id: numeric_id(&value["owner"]["id"])?,
            has_issues: value["has_issues"].as_bool() == Some(true),
            can_write_issues: true,
            credential_ref: self.credential_ref.clone(),
        }))
    }
}

pub(super) fn github(reg: &Registration, services: &Supervisor) -> Result<PlatformObservation> {
    let requested = std::env::var_os("HCTL2_GH")
        .or_else(|| {
            services
                .gitea_paths()
                .map(|(install, _)| install.join("libexec/hctl2/gh").into_os_string())
        })
        .unwrap_or_else(|| "gh".into());
    let gh = foundation::git::resolve_executable(&requested)
        .ok_or_else(|| reject("PROVIDER_UNAVAILABLE", "gh is not available", "install_gh"))?;
    let instance = reg.prepared.request.instance.as_deref().unwrap();
    let read = |path: &str| -> Result<Value> {
        let output = run(
            Command::new(&gh)
                .env("GH_PROMPT_DISABLED", "1")
                .env("NO_COLOR", "1")
                .args(["api", "--hostname", instance, "--method", "GET", path]),
            None,
        )?;
        if !output.status.success() {
            return Err(reject(
                "PLATFORM_UNAVAILABLE",
                "GitHub identity readback failed",
                "retry_registration",
            ));
        }
        Ok(serde_json::from_slice(&output.stdout)?)
    };
    let value = read(&format!(
        "repos/{}",
        reg.prepared.request.platform_path.as_deref().unwrap()
    ))?;
    let user = read("user")?;
    Ok(PlatformObservation {
        instance: instance.into(),
        stable_id: numeric_id(&value["id"])?,
        full_name: value["full_name"].as_str().unwrap_or("").into(),
        clone_url: value["clone_url"].as_str().unwrap_or("").into(),
        account_id: numeric_id(&user["id"])?,
        has_issues: value["has_issues"].as_bool() == Some(true),
        can_write_issues: value["permissions"]["push"].as_bool() == Some(true)
            || value["permissions"]["triage"].as_bool() == Some(true),
        credential_ref: String::new(), // gh owns its credential store; no copied token.
    })
}
/// How long the Gitea bootstrap waits, covering liveness and the two usability checks.
const GITEA_BOOTSTRAP_TIMEOUT: Duration = Duration::from_secs(30);

/// Interval of the supervisor-derived liveness poll.
const GITEA_BOOTSTRAP_POLL: Duration = Duration::from_millis(100);

/// Interval between bootstrap attempts once the process is live.
const GITEA_BOOTSTRAP_RETRY: Duration = Duration::from_millis(250);

/// Shortest gap between restarts while the API still answers for another instance.
const GITEA_RESTART_INTERVAL: Duration = Duration::from_secs(3);

/// Retries `attempt` until it succeeds or `deadline` passes, returning the last
/// failure. `retryable` decides which failures may be retried, and `rearm` runs before
/// each wait so a component that lost a race can be brought back.
fn retry_bootstrap<T>(
    deadline: Instant,
    mut attempt: impl FnMut() -> Result<T>,
    retryable: impl Fn(&StoreError) -> bool,
    mut rearm: impl FnMut(),
) -> Result<T> {
    loop {
        let error = match attempt() {
            Ok(value) => return Ok(value),
            Err(error) => error,
        };
        if Instant::now() >= deadline || !retryable(&error) {
            return Err(error);
        }
        rearm();
        std::thread::sleep(GITEA_BOOTSTRAP_RETRY);
    }
}

/// Re-requests the start of the consumed components between bootstrap attempts.
///
/// Only for the migration phase: a restart there could interrupt the first migration,
/// so an instance that is still creating its tables is left alone to finish.
fn rearm(services: &Supervisor) {
    let _ = services.ensure_up();
}

/// Last non-empty stderr line of a native command, for the rejection message.
fn native_detail(stderr: &[u8]) -> String {
    String::from_utf8_lossy(stderr)
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("no diagnostic output")
        .trim()
        .to_owned()
}

fn hosted_url(config: &Path) -> Result<String> {
    let text = std::fs::read_to_string(config)?;
    let url = text
        .lines()
        .find_map(|l| l.strip_prefix("ROOT_URL = "))
        .ok_or_else(|| {
            reject(
                "PLATFORM_CONFIG",
                "Gitea ROOT_URL missing",
                "inspect_services_config",
            )
        })?
        .trim()
        .trim_end_matches('/')
        .to_owned();
    if !url.starts_with("http://127.0.0.1:")
        || url["http://127.0.0.1:".len()..].parse::<u16>().is_err()
    {
        return Err(reject(
            "PLATFORM_CONFIG",
            "unexpected hosted Gitea address",
            "inspect_services_config",
        ));
    }
    Ok(url)
}

fn numeric_id(value: &Value) -> Result<String> {
    value
        .as_u64()
        .filter(|id| *id > 0)
        .map(|id| id.to_string())
        .ok_or_else(|| {
            reject(
                "PLATFORM_ID_REQUIRED",
                "platform stable ID missing",
                "retry_read",
            )
        })
}

#[cfg(test)]
#[path = "repositories/platform_tests.rs"]
mod tests;
