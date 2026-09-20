//! Native gh / tea / Gitea administration. No HTTP client or account system of our own.
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use foundation::SecretStore;
use repo::git::run;
use repo::{PlatformObservation, Registration, Result, reject};
use serde_json::{Value, json};

use crate::services::Supervisor;

pub(super) struct Hosted {
    tea: PathBuf,
    pub url: String,
    pub username: String,
    pub token: String,
    credential_ref: String,
}
impl Hosted {
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
        let deadline = Instant::now() + Duration::from_secs(30);
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
            std::thread::sleep(Duration::from_millis(100));
        }
        let config = state.join("config/gitea/app.ini");
        let text = std::fs::read_to_string(&config)?;
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
        let users = run(admin().arg("list"), None)?;
        if !users.status.success() {
            return Err(reject(
                "PLATFORM_BOOTSTRAP",
                "cannot list hosted platform accounts",
                "inspect_gitea_log",
            ));
        }
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
        let secrets = SecretStore::detect("hctl2", root.join("secrets"));
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
        let who = hosted.api("GET", "user", None)?.ok_or_else(|| {
            reject(
                "CREDENTIAL_UNAVAILABLE",
                "platform account missing",
                "restore_secret_store",
            )
        })?;
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
    fn api(&self, method: &str, path: &str, body: Option<Value>) -> Result<Option<Value>> {
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
        if status == Some(404) {
            return Ok(None);
        }
        if !output.status.success() || !status.is_some_and(|s| (200..300).contains(&s)) {
            return Err(reject(
                "PLATFORM_UNAVAILABLE",
                format!(
                    "tea API {method} {path} not confirmed (HTTP {status:?}, exit {:?})",
                    output.status.code()
                ),
                "read_back_original_intent",
            ));
        }
        Ok(Some(serde_json::from_slice(&output.stdout)?))
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
#[path = "platform_tests.rs"]
mod tests;
