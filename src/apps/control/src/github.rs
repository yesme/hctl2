//! Control-owned GitHub credentials. Native gh performs login; SecretStore owns the token.
use std::io::{IsTerminal, Read};
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use repo::{Result, reject};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::services::Supervisor;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Credential {
    host: String,
    token: String,
    login: String,
    account_id: String,
    config: PathBuf,
    home: PathBuf,
}

fn host(value: &str) -> Result<String> {
    if value.is_empty()
        || value.starts_with('-')
        || value
            .split('.')
            .any(|label| label.is_empty() || label.starts_with('-') || label.ends_with('-'))
        || !value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b".-".contains(&c))
    {
        return Err(reject(
            "INVALID_INPUT",
            "expected a GitHub hostname",
            "correct_input",
        ));
    }
    Ok(value.to_ascii_lowercase())
}

fn private_dir(path: &Path) -> Result<()> {
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    Ok(())
}

fn account(host: &str) -> String {
    format!("github:{host}")
}

fn stored_account(
    root: &Path,
    hostname: &str,
    backend: foundation::SecretBackend,
) -> Result<String> {
    // File stores already have a root-specific directory. A system keyring has one
    // service namespace per user, so its account must distinguish control roots too.
    if backend == foundation::SecretBackend::SystemKeyring {
        let root = root.canonicalize()?;
        Ok(format!(
            "github:{}:{hostname}",
            foundation::bytes_sha256(root.as_os_str().as_encoded_bytes())
        ))
    } else {
        Ok(account(hostname))
    }
}

pub(crate) fn login_required(root: &Path, hostname: &str) -> store::StoreError {
    let quoted_root = format!("'{}'", root.display().to_string().replace('\'', "'\\''"));
    reject(
        "GITHUB_LOGIN_REQUIRED",
        format!(
            "GitHub credential unavailable or rejected; run: hctl2 --root {quoted_root} github login --hostname {hostname}"
        ),
        "github_login",
    )
}

pub(crate) fn executable(services: &Supervisor) -> Result<PathBuf> {
    let name = std::env::var_os("HCTL2_GH")
        .or_else(|| {
            services
                .packaged_paths()
                .map(|(i, _)| i.join("libexec/hctl2/gh").into_os_string())
        })
        .unwrap_or_else(|| "gh".into());
    foundation::git::resolve_executable(&name)
        .ok_or_else(|| reject("PROVIDER_UNAVAILABLE", "gh unavailable", "install_gh"))
}

impl Credential {
    fn pending(root: &Path, hostname: &str, token: String) -> Result<Self> {
        let host = host(hostname)?;
        if token.is_empty() || token.len() > 8192 || !token.bytes().all(|b| b.is_ascii_graphic()) {
            return Err(reject(
                "INVALID_INPUT",
                "expected one nonempty token on standard input",
                "github_login",
            ));
        }
        let base = root.join("github").join(&host);
        let config = base.join("runtime");
        let home = base.join("home");
        private_dir(&base)?;
        private_dir(&config)?;
        private_dir(&home)?;
        Ok(Self {
            host,
            token,
            login: String::new(),
            account_id: String::new(),
            config,
            home,
        })
    }

    pub(crate) fn load(root: &Path, hostname: &str) -> Result<Self> {
        let hostname = host(hostname)?;
        let store =
            crate::config::secret_store(root).map_err(|_| login_required(root, &hostname))?;
        let key = stored_account(root, &hostname, store.backend())?;
        let bytes = store
            .get(&key)
            .map_err(|_| login_required(root, &hostname))?;
        let saved: Self =
            serde_json::from_slice(&bytes).map_err(|_| login_required(root, &hostname))?;
        if saved.host != hostname || saved.login.is_empty() || saved.account_id.is_empty() {
            return Err(login_required(root, &hostname));
        }
        // Paths belong to this control root, including after a backup is restored elsewhere.
        let mut credential = Self::pending(root, &hostname, saved.token)?;
        credential.login = saved.login;
        credential.account_id = saved.account_id;
        Ok(credential)
    }

    pub(crate) fn apply(&self, cmd: &mut Command) {
        clean_environment(cmd, &self.home, &self.config);
        if self.host == "github.com" || self.host.ends_with(".ghe.com") {
            cmd.env("GH_TOKEN", &self.token);
        } else {
            // gh's native variable for self-hosted Enterprise; never the parent's token.
            cmd.env("GH_ENTERPRISE_TOKEN", &self.token);
        }
    }

    pub(crate) fn git(&self) -> repo::git::Credential {
        repo::git::Credential::Static {
            user: self.login.clone(),
            token: self.token.clone(),
        }
    }

    pub(crate) fn git_pair(&self) -> (String, String) {
        (self.login.clone(), self.token.clone())
    }

    pub(crate) fn reference(&self) -> String {
        account(&self.host)
    }

    pub(crate) fn account_id(&self) -> &str {
        &self.account_id
    }

    pub(crate) fn rejected(&self) -> store::StoreError {
        // home = <root>/github/<host>/home
        login_required(
            self.home
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap(),
            &self.host,
        )
    }

    fn save(mut self, root: &Path, gh: &Path) -> Result<Value> {
        let mut cmd = Command::new(gh);
        self.apply(&mut cmd);
        cmd.args(["api", "--hostname", &self.host, "--method", "GET", "user"]);
        let output = repo::git::run(&mut cmd, None)?;
        if !output.status.success() {
            return Err(login_required(root, &self.host));
        }
        let value: Value = serde_json::from_slice(&output.stdout).map_err(|_| {
            reject(
                "PROVIDER_RESPONSE",
                "GitHub user response was not JSON",
                "github_login",
            )
        })?;
        self.login = value["login"]
            .as_str()
            .filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))
            .ok_or_else(|| {
                reject(
                    "PROVIDER_RESPONSE",
                    "GitHub account login missing",
                    "github_login",
                )
            })?
            .into();
        self.account_id = value["id"]
            .as_u64()
            .filter(|id| *id > 0)
            .ok_or_else(|| {
                reject(
                    "PROVIDER_RESPONSE",
                    "GitHub account ID missing",
                    "github_login",
                )
            })?
            .to_string();
        let store = crate::config::secret_store(root).map_err(|_| {
            reject(
                "CREDENTIAL_UNAVAILABLE",
                "cannot open control secret backend",
                "fix_control_config",
            )
        })?;
        let key = stored_account(root, &self.host, store.backend())?;
        store.set(&key, &serde_json::to_vec(&self)?).map_err(|_| {
            reject(
                "CREDENTIAL_UNAVAILABLE",
                "cannot save GitHub credential",
                "fix_control_config",
            )
        })?;
        Ok(self.status())
    }

    fn status(&self) -> Value {
        json!({"configured":true,"hostname":self.host,"account":self.login,"account_id":self.account_id})
    }

    #[cfg(test)]
    pub(crate) fn fixture(root: &Path) -> Self {
        let mut value = Self::pending(
            root,
            "github.com",
            "hctl2-test-token-not-a-credential".into(),
        )
        .unwrap();
        value.login = "owner".into();
        value.account_id = "1".into();
        value
    }
}

fn clean_environment(cmd: &mut Command, home: &Path, config: &Path) {
    // Do not inherit tokens, debug flags, a config locator or a user's credential helpers.
    let path = std::env::var_os("PATH").unwrap_or_else(|| "/usr/bin:/bin".into());
    cmd.env_clear()
        .env("PATH", path)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("GH_CONFIG_DIR", config)
        .env("GH_PROMPT_DISABLED", "1")
        .env("NO_COLOR", "1")
        .env("DBUS_SESSION_BUS_ADDRESS", "unix:path=/dev/null");
}

/// No online probe and no secret in the status payload.
pub fn status(root: &Path, hostname: &str) -> Value {
    match Credential::load(root, hostname) {
        Ok(value) => value.status(),
        Err(_) => json!({"configured":false,"hostname":hostname}),
    }
}

/// Owner-only foreground entry point, deliberately not a governance command with a saved input.
pub fn login(root: &Path, hostname: &str, with_token: bool) -> Result<Value> {
    let hostname = host(hostname)?;
    let base = root.join("github").join(&hostname);
    private_dir(&base)?;
    let _lock =
        foundation::ExclusiveFileLock::try_acquire(&base.join("login.lock")).map_err(|_| {
            reject(
                "GITHUB_LOGIN_BUSY",
                "another login is using this control root",
                "retry_login",
            )
        })?;
    let services = Supervisor::from_root(root.to_path_buf());
    let gh = executable(&services)?;
    let token = if with_token {
        if std::io::stdin().is_terminal() {
            return Err(reject(
                "INVALID_INPUT",
                "token input must be a private file or pipe, not an echoing terminal",
                "github_login",
            ));
        }
        let mut bytes = Vec::new();
        std::io::stdin().take(8194).read_to_end(&mut bytes)?;
        if bytes.len() > 8193 {
            return Err(reject(
                "INVALID_INPUT",
                "token exceeds input limit",
                "github_login",
            ));
        }
        String::from_utf8(bytes)
            .map_err(|_| reject("INVALID_INPUT", "token must be ASCII", "github_login"))?
            .trim()
            .to_owned()
    } else {
        native_login(root, &base, &gh, &hostname)?
    };
    Credential::pending(root, &hostname, token)?.save(root, &gh)
}

fn login_command(gh: &Path) -> Command {
    #[cfg(target_os = "macos")]
    {
        // Pinned gh's go-keyring invokes this absolute binary even with --insecure-storage.
        let mut cmd = Command::new("/usr/bin/sandbox-exec");
        cmd.args([
            "-p",
            "(version 1)(allow default)(deny process-exec (literal \"/usr/bin/security\"))",
        ])
        .arg(gh);
        cmd
    }
    #[cfg(not(target_os = "macos"))]
    {
        Command::new(gh)
    }
}

fn native_login(root: &Path, base: &Path, gh: &Path, hostname: &str) -> Result<String> {
    let config = base.join("login");
    private_dir(&base.join("home"))?;
    private_dir(&config)?;
    let mut cmd = login_command(gh);
    clean_environment(&mut cmd, &base.join("home"), &config);
    // Device flow remains native. If browser launch fails, gh displays the URL for the owner.
    cmd.env_remove("GH_PROMPT_DISABLED")
        .args([
            "auth",
            "login",
            "--hostname",
            hostname,
            "--web",
            "--insecure-storage",
            "--git-protocol",
            "https",
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    let result = cmd.status().map_err(|_| {
        reject(
            "GITHUB_LOGIN_FAILED",
            "cannot run isolated native gh login",
            "github_login",
        )
    });
    let token = match result {
        Ok(status) if status.success() => {
            let mut read = login_command(gh);
            clean_environment(&mut read, &base.join("home"), &config);
            read.args(["auth", "token", "--hostname", hostname]);
            match repo::git::run(&mut read, None) {
                Ok(out) if out.status.success() => String::from_utf8(out.stdout)
                    .map(|s| s.trim().to_owned())
                    .map_err(|_| login_required(root, hostname)),
                _ => Err(login_required(root, hostname)),
            }
        }
        _ => Err(reject(
            "GITHUB_LOGIN_FAILED",
            "native gh login did not finish; no credential imported",
            "github_login",
        )),
    };
    // Only this locked, control-owned staging directory; never the user's gh directory.
    std::fs::remove_dir_all(&config)?;
    token
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "control-gh-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            private_dir(&root).unwrap();
            Self(root)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn system_keyring_accounts_distinguish_roots_but_not_aliases_without_touching_a_keyring() {
        let first = Temp::new();
        let second = Temp::new();
        let system = foundation::SecretBackend::SystemKeyring;
        assert_ne!(
            stored_account(&first.0, "github.com", system).unwrap(),
            stored_account(&second.0, "github.com", system).unwrap()
        );
        let alias = second.0.join("alias");
        std::os::unix::fs::symlink(&first.0, &alias).unwrap();
        assert_eq!(
            stored_account(&first.0, "github.com", system).unwrap(),
            stored_account(&alias, "github.com", system).unwrap()
        );
        assert_eq!(
            stored_account(&first.0, "github.com", foundation::SecretBackend::UserFile).unwrap(),
            account("github.com")
        );
    }

    #[test]
    fn missing_control_credential_never_executes_gh_or_reads_a_personal_login() {
        let temp = Temp::new();
        let personal = temp.0.join("personal-gh");
        private_dir(&personal).unwrap();
        std::fs::write(
            personal.join("hosts.yml"),
            "github.com:\n  oauth_token: personal-only\n",
        )
        .unwrap();
        let error = Credential::load(&temp.0, "github.com").err().unwrap();
        assert_eq!(error.code, "GITHUB_LOGIN_REQUIRED");
        assert!(error.message.contains("github login --hostname github.com"));
        assert!(!error.message.contains("personal-only"));
        assert!(!temp.0.join("github").exists());
        assert_eq!(
            std::fs::read_to_string(personal.join("hosts.yml")).unwrap(),
            "github.com:\n  oauth_token: personal-only\n"
        );
    }

    #[test]
    fn secret_status_is_metadata_only_and_restored_roots_recompute_private_paths() {
        let first = Temp::new();
        let restored = Temp::new();
        let original = Credential::fixture(&first.0);
        let bytes = serde_json::to_vec(&original).unwrap();
        crate::config::secret_store(&restored.0)
            .unwrap()
            .set("github:github.com", &bytes)
            .unwrap();
        let loaded = Credential::load(&restored.0, "github.com").unwrap();
        assert_eq!(loaded.config, restored.0.join("github/github.com/runtime"));
        assert_eq!(
            loaded.git_pair(),
            ("owner".into(), "hctl2-test-token-not-a-credential".into())
        );
        let state = status(&restored.0, "github.com");
        assert_eq!(state["account"], "owner");
        assert!(!state.to_string().contains(&loaded.token));
        assert_eq!(
            std::fs::metadata(&loaded.config)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
    }

    #[test]
    fn process_credentials_are_scoped_and_never_arguments_for_public_or_enterprise_hosts() {
        let temp = Temp::new();
        for hostname in ["github.com", "tenant.ghe.com", "git.example.org"] {
            let credential =
                Credential::pending(&temp.0, hostname, "control-owned-fake-token".into()).unwrap();
            let mut command = Command::new("/usr/bin/env");
            command
                .env("GH_TOKEN", "parent-token")
                .env("GITHUB_TOKEN", "parent-other")
                .env("GH_ENTERPRISE_TOKEN", "parent-enterprise")
                .env("GH_DEBUG", "api")
                .env("GH_CONFIG_DIR", "/personal/gh");
            credential.apply(&mut command);
            assert_eq!(command.get_args().count(), 0);
            let output = command.output().unwrap();
            let environment = String::from_utf8(output.stdout).unwrap();
            assert!(
                !environment.contains("parent-")
                    && !environment.contains("GH_DEBUG=")
                    && !environment.contains("/personal/gh")
            );
            assert!(
                environment.contains(&format!("GH_CONFIG_DIR={}\n", credential.config.display()))
            );
            let variable = if hostname == "git.example.org" {
                "GH_ENTERPRISE_TOKEN"
            } else {
                "GH_TOKEN"
            };
            assert!(environment.contains(&format!("{variable}=control-owned-fake-token\n")));
        }
    }

    #[test]
    fn malformed_control_tokens_never_become_commands_or_error_text() {
        let temp = Temp::new();
        for value in ["", "fake token", "fake\nsecond", "fake\0token"] {
            let error = Credential::pending(&temp.0, "github.com", value.into())
                .err()
                .unwrap();
            assert_eq!(error.code, "INVALID_INPUT");
            assert!(!error.message.contains("fake"));
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn native_login_boundary_blocks_the_pinned_gh_keychain_program() {
        let temp = Temp::new();
        let mut command = login_command(Path::new("/usr/bin/security"));
        clean_environment(&mut command, &temp.0, &temp.0);
        let output = command.arg("help").output().unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("Operation not permitted"));
    }
}
