//! Process Compose client for hosted bundled services.
//!
//! Hosted components start on first consumption: Tuwunel is consumed by every
//! Project (main Room), so it is the baseline; Gitea is consumed the first time a
//! purely local repository (or an explicit local-platform choice) is registered.
//! The consumed set is deployment state kept next to the control root, not a
//! governance record. Health is an observation. It is never written as a
//! governance record either.

use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

const HOSTED: &[&str] = &["tuwunel", "gitea"];
const BASELINE: &[&str] = &["tuwunel"];
const CONSUMED_FILE: &str = "hosted-consumed.json";

#[derive(Clone, Debug)]
pub struct ServiceHealth {
    pub name: String,
    pub consumed: bool,
    pub running: bool,
    pub ready: bool,
    pub pid: Option<i64>,
}

impl ServiceHealth {
    #[must_use]
    pub fn available(&self) -> bool {
        self.running && self.ready
    }
}

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub backend: String,
    pub source: String,
    pub observed_at: String,
    pub last_error: Option<String>,
    pub hosted: Vec<ServiceHealth>,
}

impl Snapshot {
    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({
            "backend": self.backend,
            "source": self.source,
            "observed_at": self.observed_at,
            "last_error": self.last_error,
            "hosted": self.hosted.iter().map(|service| json!({
                "name": service.name,
                "consumed": service.consumed,
                "running": service.running,
                "ready": service.ready,
                "available": service.available(),
                "pid": service.pid,
            })).collect::<Vec<_>>(),
        })
    }
}

pub struct Supervisor {
    root: PathBuf,
    backend: Backend,
    last_error: Mutex<Option<String>>,
}

enum Backend {
    Absent,
    Packaged {
        install_root: PathBuf,
        services_bin: PathBuf,
    },
    Fixture {
        pc_bin: PathBuf,
        configs: Vec<PathBuf>,
        hosted: Vec<String>,
        baseline: Vec<String>,
        socket: PathBuf,
        work: PathBuf,
    },
}

impl Supervisor {
    #[must_use]
    pub fn from_root(root: PathBuf) -> Self {
        let backend = detect(&root);
        Self {
            root,
            backend,
            last_error: Mutex::new(None),
        }
    }

    pub fn set_last_error(&self, error: impl Into<String>) {
        *self
            .last_error
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = Some(error.into());
    }

    #[must_use]
    pub fn snapshot(&self) -> Snapshot {
        let consumed = match self.consumed_names() {
            Ok(names) => names,
            Err(error) => {
                self.set_last_error(error);
                self.baseline_names()
            }
        };
        let hosted = self
            .hosted_names()
            .into_iter()
            .map(|name| {
                let mut health = match &self.backend {
                    Backend::Absent => ServiceHealth {
                        name: name.clone(),
                        consumed: false,
                        running: false,
                        ready: false,
                        pid: None,
                    },
                    Backend::Packaged { .. } | Backend::Fixture { .. } => self.health(&name),
                };
                health.consumed = consumed.contains(&name);
                health
            })
            .collect();
        Snapshot {
            backend: self.backend_name().into(),
            source: format!("process-compose:{}", self.socket_path().display()),
            observed_at: observed_at(),
            last_error: self
                .last_error
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .clone(),
            hosted,
        }
    }

    /// Mark a hosted component as consumed, persist that, and start it without
    /// waiting for its probe. Callers decide when consumption happens (for
    /// example registering a purely local repository consumes Gitea).
    pub fn consume(&self, name: &str) -> Result<Value, String> {
        let name = name.trim();
        if !self.hosted_names().iter().any(|hosted| hosted == name) {
            return Err(format!("{name} is not a hosted component"));
        }
        let mut consumed = self.persisted_consumed().inspect_err(|error| {
            self.set_last_error(error.clone());
        })?;
        if !consumed.iter().any(|item| item == name) {
            consumed.push(name.to_owned());
            self.write_consumed(&consumed)?;
        }
        let result = self.start_components(&[name.to_owned()]);
        match &result {
            Ok(()) => self.clear_last_error(),
            Err(error) => self.set_last_error(error.clone()),
        }
        result.map(|()| json!({"component": name, "consumed": true}))
    }

    /// Start consumed services without waiting for probes. Store stays available.
    pub fn ensure_up(&self) -> Result<(), String> {
        let (names, record_error) = match self.consumed_names() {
            Ok(names) => (names, None),
            // A corrupt record is not "never consumed": keep the baseline up,
            // report the record, and leave the file for the operator.
            Err(error) => (self.baseline_names(), Some(error)),
        };
        let result = self.start_components(&names);
        match (&result, record_error) {
            (Ok(()), None) => self.clear_last_error(),
            (Ok(()), Some(error)) => self.set_last_error(error),
            (Err(error), _) => self.set_last_error(error.clone()),
        }
        result
    }

    fn clear_last_error(&self) {
        *self
            .last_error
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = None;
    }

    fn start_components(&self, names: &[String]) -> Result<(), String> {
        if names.is_empty() {
            return Ok(());
        }
        match &self.backend {
            Backend::Absent => Ok(()),
            Backend::Packaged {
                install_root,
                services_bin,
            } => {
                fs::create_dir_all(self.state_root()).map_err(io)?;
                let mut cmd = Command::new(services_bin);
                cmd.env("HCTL2_INSTALL_ROOT", install_root);
                self.apply_state_env(&mut cmd);
                let status = cmd
                    .args(["start", "--no-wait"])
                    .args(names)
                    .status()
                    .map_err(io)?;
                if status.success() {
                    Ok(())
                } else {
                    Err(format!("hctl2-services start failed: {status}"))
                }
            }
            Backend::Fixture {
                pc_bin,
                configs,
                socket,
                work,
                ..
            } => {
                fs::create_dir_all(work).map_err(io)?;
                if let Some(parent) = socket.parent() {
                    fs::create_dir_all(parent).map_err(io)?;
                }
                if self.pc_alive() {
                    for name in names {
                        if !self.health(name).running {
                            let _ = self
                                .pc_client(pc_bin, socket)
                                .args(["process", "start", name])
                                .status();
                        }
                    }
                    return Ok(());
                }
                let mut cmd = self.pc_project(pc_bin, socket, configs, work);
                cmd.args(["up", "--detached", "--tui=false", "--keep-project"]);
                cmd.args(names);
                let status = cmd.status().map_err(io)?;
                if status.success() {
                    Ok(())
                } else {
                    Err(format!("process-compose up failed: {status}"))
                }
            }
        }
    }

    /// Tear down the whole Process Compose project. Tests use this in Drop.
    pub fn shutdown_project(&self) -> Result<(), String> {
        match &self.backend {
            Backend::Absent => Ok(()),
            Backend::Packaged {
                install_root,
                services_bin,
            } => {
                let mut down = Command::new(services_bin);
                down.env("HCTL2_INSTALL_ROOT", install_root);
                self.apply_state_env(&mut down);
                let _ = down.arg("stop").status();
                Ok(())
            }
            Backend::Fixture { pc_bin, socket, .. } => {
                if self.pc_alive() {
                    let _ = self
                        .pc_client(pc_bin, socket)
                        .arg("down")
                        .stdout(Stdio::null())
                        .status();
                }
                Ok(())
            }
        }
    }

    pub fn stop_consumed(&self) -> Result<(), String> {
        match &self.backend {
            Backend::Absent => Ok(()),
            Backend::Packaged {
                install_root,
                services_bin,
            } => {
                let mut stop = Command::new(services_bin);
                stop.env("HCTL2_INSTALL_ROOT", install_root);
                self.apply_state_env(&mut stop);
                let consumed = self
                    .consumed_names()
                    .unwrap_or_else(|_| self.baseline_names());
                if consumed.is_empty() {
                    return Ok(());
                }
                let status = stop.arg("stop").args(&consumed).status().map_err(io)?;
                if !(status.success() || status.code() == Some(1)) {
                    return Err(format!("hctl2-services stop failed: {status}"));
                }
                if self.running_names().is_empty() {
                    let mut down = Command::new(services_bin);
                    down.env("HCTL2_INSTALL_ROOT", install_root);
                    self.apply_state_env(&mut down);
                    let _ = down.arg("stop").status();
                }
                Ok(())
            }
            Backend::Fixture { pc_bin, socket, .. } => {
                if !self.pc_alive() {
                    return Ok(());
                }
                let consumed = self
                    .consumed_names()
                    .unwrap_or_else(|_| self.baseline_names());
                for name in consumed {
                    let _ = self
                        .pc_client(pc_bin, socket)
                        .args(["process", "stop", &name])
                        .stdout(Stdio::null())
                        .status();
                }
                if self.running_names().is_empty() {
                    let _ = self
                        .pc_client(pc_bin, socket)
                        .arg("down")
                        .stdout(Stdio::null())
                        .status();
                }
                Ok(())
            }
        }
    }

    pub fn backup(&self, dest: &Path) -> Result<Value, String> {
        self.stop_consumed()?;
        fs::create_dir_all(dest).map_err(io)?;
        fs::set_permissions(dest, fs::Permissions::from_mode(0o700)).map_err(io)?;
        let consumed = self.consumed_names()?;
        let result = match &self.backend {
            Backend::Absent => Ok(json!({"backend":"not_installed"})),
            Backend::Packaged { .. } => {
                let state = self.state_root();
                copy_tree(&state.join("data"), &dest.join("data"))?;
                copy_tree(&state.join("config"), &dest.join("config"))?;
                write_manifest(dest, "packaged", &consumed)?;
                Ok(json!({"backend":"packaged","path": dest.display().to_string()}))
            }
            Backend::Fixture { work, .. } => {
                copy_tree(work, &dest.join("fixture"))?;
                write_manifest(dest, "fixture", &consumed)?;
                Ok(json!({"backend":"fixture","path": dest.display().to_string()}))
            }
        };
        // The consumed record travels with the backup so a restore into a new
        // control root brings the same components back up.
        if result.is_ok() && self.consumed_file().is_file() {
            fs::copy(self.consumed_file(), dest.join(CONSUMED_FILE)).map_err(io)?;
        }
        let _ = self.ensure_up();
        result
    }

    pub fn restore(&self, src: &Path) -> Result<Value, String> {
        self.stop_consumed()?;
        if src.join(CONSUMED_FILE).is_file() {
            let bytes = fs::read(src.join(CONSUMED_FILE)).map_err(io)?;
            let names = parse_consumed(&bytes)?;
            self.write_consumed(&names)?;
        }
        match &self.backend {
            Backend::Absent => Ok(json!({"backend":"not_installed"})),
            Backend::Packaged { .. } => {
                let state = self.state_root();
                if src.join("data").exists() {
                    replace_tree(&src.join("data"), &state.join("data"))?;
                }
                if src.join("config").exists() {
                    replace_tree(&src.join("config"), &state.join("config"))?;
                }
                let _ = self.ensure_up();
                Ok(json!({"backend":"packaged"}))
            }
            Backend::Fixture { work, .. } => {
                if src.join("fixture").exists() {
                    replace_tree(&src.join("fixture"), work)?;
                }
                let _ = self.ensure_up();
                Ok(json!({"backend":"fixture"}))
            }
        }
    }

    fn health(&self, name: &str) -> ServiceHealth {
        let raw = match self.process_json(name) {
            Some(value) => value,
            None => {
                return ServiceHealth {
                    name: name.into(),
                    consumed: false,
                    running: false,
                    ready: false,
                    pid: None,
                };
            }
        };
        let entry = raw.as_array().and_then(|rows| rows.first()).unwrap_or(&raw);
        let running = entry
            .get("is_running")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let ready = entry
            .get("is_ready")
            .and_then(Value::as_str)
            .is_some_and(|value| value == "Ready");
        let pid = entry
            .get("pid")
            .and_then(Value::as_i64)
            .filter(|pid| *pid > 0);
        ServiceHealth {
            name: name.into(),
            consumed: false,
            running,
            ready,
            pid,
        }
    }

    fn running_names(&self) -> Vec<String> {
        let Some(raw) = self.list_json() else {
            return Vec::new();
        };
        let rows = raw
            .as_array()
            .or_else(|| raw.get("data").and_then(Value::as_array));
        let Some(rows) = rows else {
            return Vec::new();
        };
        rows.iter()
            .filter(|row| {
                row.get("is_running")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            })
            .filter_map(|row| row.get("name").and_then(Value::as_str).map(str::to_owned))
            .collect()
    }

    fn list_json(&self) -> Option<Value> {
        let output = match &self.backend {
            Backend::Absent => return None,
            Backend::Packaged { install_root, .. } => {
                let pc = install_root.join("libexec/hctl2/process-compose");
                if !pc.exists() {
                    return None;
                }
                Command::new(pc)
                    .env("XDG_CONFIG_HOME", self.state_root().join("config"))
                    .env("PC_DISABLE_DOTENV", "1")
                    .args([
                        "--use-uds",
                        "--unix-socket",
                        &self.socket_path().to_string_lossy(),
                    ])
                    .args(["process", "list", "--output", "json"])
                    .output()
                    .ok()?
            }
            Backend::Fixture { pc_bin, socket, .. } => self
                .pc_client(pc_bin, socket)
                .args(["process", "list", "--output", "json"])
                .output()
                .ok()?,
        };
        if !output.status.success() {
            return None;
        }
        serde_json::from_slice(&output.stdout).ok()
    }

    fn process_json(&self, name: &str) -> Option<Value> {
        let output = match &self.backend {
            Backend::Absent => return None,
            Backend::Packaged { install_root, .. } => {
                let pc = install_root.join("libexec/hctl2/process-compose");
                if !pc.exists() {
                    return None;
                }
                Command::new(pc)
                    .env("XDG_CONFIG_HOME", self.state_root().join("config"))
                    .env("PC_DISABLE_DOTENV", "1")
                    .args([
                        "--use-uds",
                        "--unix-socket",
                        &self.socket_path().to_string_lossy(),
                    ])
                    .args(["process", "get", name, "--output", "json"])
                    .output()
                    .ok()?
            }
            Backend::Fixture { pc_bin, socket, .. } => self
                .pc_client(pc_bin, socket)
                .args(["process", "get", name, "--output", "json"])
                .output()
                .ok()?,
        };
        if !output.status.success() {
            return None;
        }
        serde_json::from_slice(&output.stdout).ok()
    }

    fn pc_alive(&self) -> bool {
        match &self.backend {
            Backend::Fixture { pc_bin, socket, .. } => self
                .pc_client(pc_bin, socket)
                .args(["process", "list", "--output", "json"])
                .output()
                .is_ok_and(|output| output.status.success()),
            Backend::Packaged { .. } => self
                .hosted_names()
                .iter()
                .any(|name| self.health(name).running),
            Backend::Absent => false,
        }
    }

    fn pc_client(&self, pc_bin: &Path, socket: &Path) -> Command {
        let mut cmd = Command::new(pc_bin);
        cmd.env("PC_DISABLE_DOTENV", "1");
        if let Backend::Fixture { work, .. } = &self.backend {
            cmd.env("HCTL2_PC_WORK", work);
            cmd.env("XDG_CONFIG_HOME", work.join("config"));
        }
        cmd.arg("--use-uds").arg("--unix-socket").arg(socket);
        cmd.stderr(Stdio::null());
        cmd
    }

    fn pc_project(
        &self,
        pc_bin: &Path,
        socket: &Path,
        configs: &[PathBuf],
        work: &Path,
    ) -> Command {
        let config_home = work.join("config");
        let _ = fs::create_dir_all(config_home.join("process-compose"));
        let mut cmd = Command::new(pc_bin);
        cmd.env("PC_DISABLE_DOTENV", "1");
        cmd.env("HCTL2_PC_WORK", work);
        cmd.env("XDG_CONFIG_HOME", &config_home);
        cmd.arg("--use-uds").arg("--unix-socket").arg(socket);
        for config in configs {
            cmd.arg("--config").arg(config);
        }
        cmd.stderr(Stdio::null());
        cmd
    }

    fn hosted_names(&self) -> Vec<String> {
        match &self.backend {
            Backend::Fixture { hosted, .. } => hosted.clone(),
            _ => HOSTED.iter().map(|name| (*name).to_string()).collect(),
        }
    }

    fn baseline_names(&self) -> Vec<String> {
        match &self.backend {
            Backend::Fixture { baseline, .. } => baseline.clone(),
            _ => BASELINE.iter().map(|name| (*name).to_string()).collect(),
        }
    }

    /// Baseline plus persisted consumption, in hosted order. A missing record
    /// means nothing was consumed yet; an unreadable one is an error, never an
    /// empty set.
    fn consumed_names(&self) -> Result<Vec<String>, String> {
        let baseline = self.baseline_names();
        let persisted = self.persisted_consumed()?;
        Ok(self
            .hosted_names()
            .into_iter()
            .filter(|name| baseline.contains(name) || persisted.contains(name))
            .collect())
    }

    fn consumed_file(&self) -> PathBuf {
        self.root.join(CONSUMED_FILE)
    }

    fn persisted_consumed(&self) -> Result<Vec<String>, String> {
        let path = self.consumed_file();
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(format!("{}: {error}", path.display())),
        };
        parse_consumed(&bytes).map_err(|error| format!("{}: {error}", path.display()))
    }

    fn write_consumed(&self, consumed: &[String]) -> Result<(), String> {
        fs::create_dir_all(&self.root).map_err(io)?;
        let path = self.consumed_file();
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, json!({"consumed": consumed}).to_string()).map_err(io)?;
        fs::rename(&tmp, &path).map_err(io)
    }

    fn backend_name(&self) -> &'static str {
        match self.backend {
            Backend::Absent => "not_installed",
            Backend::Packaged { .. } => "packaged",
            Backend::Fixture { .. } => "fixture",
        }
    }

    fn nested_state(&self) -> bool {
        self.root != crate::default_root()
    }

    fn state_root(&self) -> PathBuf {
        if self.nested_state() {
            self.root.join("services")
        } else {
            default_services_state()
        }
    }

    fn apply_state_env(&self, cmd: &mut Command) {
        if self.nested_state() {
            cmd.env("HCTL2_STATE_ROOT", self.state_root());
        }
    }

    fn socket_path(&self) -> PathBuf {
        match &self.backend {
            Backend::Fixture { socket, .. } => socket.clone(),
            Backend::Packaged { .. } => packaged_pc_socket(&self.state_root()),
            Backend::Absent => PathBuf::from(""),
        }
    }
}

fn detect(root: &Path) -> Backend {
    if let Some(fixture) = fixture_backend(root) {
        return fixture;
    }
    if let Some(install) = packaged_install_root() {
        let services_bin = install.join("bin/hctl2-services");
        let pc = install.join("libexec/hctl2/process-compose");
        if services_bin.exists() && pc.exists() {
            return Backend::Packaged {
                install_root: install,
                services_bin,
            };
        }
    }
    Backend::Absent
}

fn fixture_backend(root: &Path) -> Option<Backend> {
    let pc_bin = PathBuf::from(std::env::var_os("HCTL2_PROCESS_COMPOSE_BIN")?);
    if !pc_bin.exists() {
        return None;
    }
    let configs = std::env::var("HCTL2_PROCESS_COMPOSE_CONFIG")
        .ok()?
        .split(':')
        .filter(|item| !item.is_empty())
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    if configs.is_empty() {
        return None;
    }
    let baseline = env_list("HCTL2_CONSUMED_SERVICES", &["ready-ok"]);
    let hosted = env_list("HCTL2_HOSTED_SERVICES", &["ready-ok", "never-ready"]);
    let work = root.join("services/fixture");
    let socket = fixture_socket(root);
    Some(Backend::Fixture {
        pc_bin,
        configs,
        hosted,
        baseline,
        socket,
        work,
    })
}

fn env_list(key: &str, default: &[&str]) -> Vec<String> {
    std::env::var(key)
        .ok()
        .map(|value| {
            value
                .split(',')
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_else(|| default.iter().map(|item| (*item).to_string()).collect())
}

fn packaged_install_root() -> Option<PathBuf> {
    if let Some(root) = std::env::var_os("HCTL2_INSTALL_ROOT") {
        let path = PathBuf::from(root);
        return is_payload(&path).then_some(path);
    }
    let exe = std::env::current_exe().ok()?;
    let resolved = fs::canonicalize(&exe).unwrap_or_else(|_| exe.clone());
    for candidate in payload_candidates(&resolved)
        .into_iter()
        .chain(payload_candidates(&exe))
    {
        if is_payload(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn is_payload(path: &Path) -> bool {
    path.join("libexec/hctl2/process-compose").is_file()
        && path.join("bin/hctl2-services").is_file()
}

fn payload_candidates(exe: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Some(bin) = exe.parent() else {
        return out;
    };
    if let Some(parent) = bin.parent() {
        out.push(parent.to_path_buf());
        let lib = parent.join("lib/hctl2");
        if let Ok(entries) = fs::read_dir(lib) {
            out.extend(entries.flatten().map(|entry| entry.path()));
        }
    }
    out
}

fn default_services_state() -> PathBuf {
    if let Some(root) = std::env::var_os("HCTL2_STATE_ROOT") {
        return PathBuf::from(root);
    }
    if let Some(xdg) = std::env::var_os("XDG_STATE_HOME") {
        return PathBuf::from(xdg).join("hctl2");
    }
    match std::env::var_os("HOME") {
        Some(home) => PathBuf::from(home).join(".local/state/hctl2"),
        None => PathBuf::from(".local/state/hctl2"),
    }
}

fn fixture_socket(root: &Path) -> PathBuf {
    let digest = foundation::bytes_sha256(root.to_string_lossy().as_bytes());
    let dir = std::env::temp_dir().join(format!("hctl2-pc-{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    dir.join(format!("{}.sock", &digest[..16.min(digest.len())]))
}

fn packaged_pc_socket(state: &Path) -> PathBuf {
    let digest = foundation::bytes_sha256(state.to_string_lossy().as_bytes());
    let uid = uid();
    let dir = PathBuf::from(format!("/tmp/hctl2-process-compose-{uid}"));
    let _ = fs::create_dir_all(&dir);
    dir.join(format!("{}.sock", &digest[..16.min(digest.len())]))
}

fn uid() -> u32 {
    Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|text| text.trim().parse().ok())
        .unwrap_or(0)
}

fn parse_consumed(bytes: &[u8]) -> Result<Vec<String>, String> {
    let value = serde_json::from_slice::<Value>(bytes)
        .map_err(|error| format!("consumed record is not JSON: {error}"))?;
    let items = value
        .get("consumed")
        .and_then(Value::as_array)
        .ok_or_else(|| "consumed record has no \"consumed\" array".to_owned())?;
    items
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .ok_or_else(|| "consumed record holds a non-string entry".to_owned())
        })
        .collect()
}

fn observed_at() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

fn write_manifest(dest: &Path, backend: &str, components: &[String]) -> Result<(), String> {
    let body = json!({
        "backend": backend,
        "components": components,
        "observed_at": observed_at(),
    });
    fs::write(dest.join("manifest.json"), body.to_string()).map_err(io)
}

fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    if !from.exists() {
        return Ok(());
    }
    fs::create_dir_all(to).map_err(io)?;
    for entry in fs::read_dir(from).map_err(io)? {
        let entry = entry.map_err(io)?;
        let dest = to.join(entry.file_name());
        if entry.file_type().map_err(io)?.is_dir() {
            copy_tree(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), dest).map_err(io)?;
        }
    }
    Ok(())
}

fn replace_tree(from: &Path, to: &Path) -> Result<(), String> {
    let aside = to.with_extension("aside");
    if to.exists() {
        if aside.exists() {
            fs::remove_dir_all(&aside).map_err(io)?;
        }
        fs::rename(to, &aside).map_err(io)?;
    }
    match copy_tree(from, to) {
        Ok(()) => {
            if aside.exists() {
                let _ = fs::remove_dir_all(&aside);
            }
            Ok(())
        }
        Err(error) => {
            let _ = fs::remove_dir_all(to);
            if aside.exists() {
                let _ = fs::rename(&aside, to);
            }
            Err(error)
        }
    }
}

fn io(error: std::io::Error) -> String {
    error.to_string()
}
