//! Herdr owns the terminal and harness. Claude's structured turn signal returns output;
//! physical exit is a separate observation. No Agency-owned harness child.
use crate::herdr::{Client, Server};
use crate::runtime::{Running, Runtime, RuntimeEvent, Session};
use agency_proto::context::{Bundle, Delivery};
use agency_proto::{
    Capabilities, Catalog, EvidenceLevel, ExecutionSpec, FrozenRef, PortError, Profession, Result,
    Sealed,
};
use serde_json::json;
use std::{
    fs,
    io::Read,
    os::unix::{fs::PermissionsExt, net::UnixStream},
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

const OUTPUT_LIMIT: u64 = 16 * 1024 * 1024;

/// Only the exit subscription used by this adapter. It is not a second supervisor.
struct ExitFeed {
    stream: UnixStream,
    pending: Vec<u8>,
    exited: bool,
}
impl ExitFeed {
    fn subscribe(socket: &Path) -> Result<Self> {
        use std::io::{BufRead, BufReader, Write};
        let mut stream = UnixStream::connect(socket)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;
        let request = json!({"id":"exit-feed","protocol":crate::herdr::PROTOCOL,
            "method":"events.subscribe","params":{"subscriptions":[{"type":"pane.exited"}]}});
        writeln!(stream, "{request}")?;
        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.by_ref().take(1024 * 1024).read_line(&mut response)?;
        let value: serde_json::Value = serde_json::from_str(&response)?;
        if value["id"] != "exit-feed"
            || value.get("error").is_some()
            || value.get("result").is_none()
        {
            return Err(PortError::invalid("Herdr exit subscription rejected"));
        }
        // Preserve events read ahead with the acknowledgement.
        let pending = reader.buffer().to_vec();
        let stream = reader.into_inner();
        stream.set_nonblocking(true)?;
        Ok(Self {
            stream,
            pending,
            exited: false,
        })
    }
    fn poll(&mut self, pane: &str) -> Result<bool> {
        if self.exited {
            return Ok(true);
        }
        let mut buffer = [0; 8192];
        let mut disconnected = false;
        for _ in 0..32 {
            match self.stream.read(&mut buffer) {
                Ok(0) => {
                    disconnected = true;
                    break;
                }
                Ok(n) => self.pending.extend_from_slice(&buffer[..n]),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e.into()),
            }
            if self.pending.len() > 1024 * 1024 {
                return Err(PortError::invalid("Herdr event buffer exceeds 1 MiB"));
            }
        }
        while let Some(end) = self.pending.iter().position(|b| *b == b'\n') {
            let line: Vec<u8> = self.pending.drain(..=end).collect();
            let value: serde_json::Value = serde_json::from_slice(&line)?;
            if let Some(error) = value.get("error") {
                return Err(PortError::new(
                    "HERDR_EVENTS_LOST",
                    error.to_string(),
                    "inspect_dispatch",
                ));
            }
            // Subscription names use dots; EventKind serializes with snake_case.
            if value["event"] == "pane_exited" && value["data"]["pane_id"] == pane {
                self.exited = true;
            }
        }
        // The final event can arrive in the same read as EOF. Consume it first.
        if disconnected && !self.exited {
            return Err(PortError::new(
                "HERDR_EVENTS_LOST",
                "exit subscription closed before native exit",
                "inspect_dispatch",
            ));
        }
        Ok(self.exited)
    }
}

pub struct Launch {
    client: Client,
    pane: String,
    events: Mutex<ExitFeed>,
    report_dir: PathBuf,
    stopped: AtomicBool,
    closed: Mutex<bool>,
    _server: Arc<Server>,
}

pub struct Finished {
    pub code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl Launch {
    pub fn start(
        server: Arc<Server>,
        exec_dir: &Path,
        state_dir: &Path,
        credential_root: &Path,
        body: &str,
        label: &str,
    ) -> Result<Self> {
        if state_dir != server.state {
            return Err(PortError::invalid(
                "launch state differs from the live server",
            ));
        }
        credential_root.canonicalize()?;
        let stamp = crate::storage::nonce()?;
        let report_dir = state_dir.join(format!("launch-{stamp}"));
        crate::storage::private_dir(&report_dir)?;
        // The caller's text is a file argument, never input typed into a shell.
        let program = report_dir.join("program.sh");
        let runner = report_dir.join("runner.sh");
        fs::write(&program, body)?;
        fs::write(report_dir.join("stdout"), [])?;
        fs::write(report_dir.join("stderr"), [])?;
        fs::write(
            &runner,
            format!(
                "#!/bin/sh\n/bin/sh {program} > {stdout} 2> {stderr}\ncode=$?\nprintf '%s\\n' \"$code\" > {exit}\nexit \"$code\"\n",
                program = sh_quote(&program),
                stdout = sh_quote(&report_dir.join("stdout")),
                stderr = sh_quote(&report_dir.join("stderr")),
                exit = sh_quote(&report_dir.join("exit")),
            ),
        )?;
        let events = ExitFeed::subscribe(&server.socket)?;
        let client = Client::connect(&server.socket)?;
        let created = client.call(
            "workspace.create",
            json!({"cwd":exec_dir,"label":label,"focus":false}),
        )?;
        let mut launch = Self {
            pane: created
                .pointer("/root_pane/pane_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| PortError::invalid("workspace has no pane"))?
                .into(),
            client,
            events: Mutex::new(events),
            report_dir,
            stopped: AtomicBool::new(false),
            closed: Mutex::new(false),
            _server: server,
        };
        let tab = created
            .pointer("/tab/tab_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PortError::invalid("workspace has no tab"))?;
        let applied = launch.client.call(
            "layout.apply",
            json!({
                "tab_id":tab,"focus":false,
                "root":{"type":"pane","cwd":exec_dir,"command":["/bin/sh",runner]}
            }),
        )?;
        launch.pane = applied
            .pointer("/layout/root/pane_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PortError::invalid("layout has no pane"))?
            .into();
        Ok(launch)
    }
    pub fn pane(&self) -> &str {
        &self.pane
    }
    pub fn client(&self) -> &Client {
        &self.client
    }
    fn exited(&self) -> Result<bool> {
        self.events.lock().expect("exit feed").poll(&self.pane)
    }
    /// Native exit must be observed before consulting the observational exit code.
    pub fn poll_exit(&self) -> Result<Option<i32>> {
        if self.stopped.load(Ordering::SeqCst) || !self.exited()? {
            return Ok(None);
        }
        self.exit_code()
    }
    fn exit_code(&self) -> Result<Option<i32>> {
        match fs::read_to_string(self.report_dir.join("exit")) {
            Ok(code) => Ok(code.trim().parse::<i32>().ok()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    fn output(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let read = |name: &str| -> Result<Vec<u8>> {
            let mut bytes = Vec::new();
            fs::File::open(self.report_dir.join(name))?
                .take(OUTPUT_LIMIT + 1)
                .read_to_end(&mut bytes)?;
            if bytes.len() as u64 > OUTPUT_LIMIT {
                return Err(PortError::new(
                    "LAUNCH_OUTPUT_LIMIT",
                    "harness stream exceeds 16 MiB",
                    "reduce_harness_output",
                ));
            }
            Ok(bytes)
        };
        Ok((read("stdout")?, read("stderr")?))
    }
    pub fn wait(&self, timeout: Duration) -> Result<Finished> {
        match self.wait_end(timeout)? {
            WaitEnd::Finished(finished) => Ok(finished),
            WaitEnd::Stopped => Err(PortError::new(
                "LAUNCH_STOPPED",
                "caller stopped the launch",
                "read_exit_event",
            )),
            WaitEnd::TimedOut => Err(PortError::new(
                "LAUNCH_TIMEOUT",
                "program is still running",
                "raise_timeout_or_cancel",
            )),
        }
    }
    pub fn wait_end(&self, timeout: Duration) -> Result<WaitEnd> {
        let deadline = Instant::now() + timeout;
        loop {
            if self.stopped.load(Ordering::SeqCst) {
                return Ok(WaitEnd::Stopped);
            }
            if self.exited()? {
                let (stdout, stderr) = self.output()?;
                return Ok(WaitEnd::Finished(Finished {
                    code: self.exit_code()?,
                    stdout,
                    stderr,
                }));
            }
            if Instant::now() >= deadline {
                return Ok(WaitEnd::TimedOut);
            }
            std::thread::sleep(Duration::from_millis(25));
        }
    }
    pub fn cancel(&self) -> Result<()> {
        self.close()?;
        self.stopped.store(true, Ordering::SeqCst);
        Ok(())
    }
    fn close(&self) -> Result<()> {
        let mut closed = self.closed.lock().expect("pane close");
        if !*closed {
            if let Err(error) = self.client.call("pane.close", json!({"pane_id":self.pane})) {
                // Herdr removes a pane after its native exit. Only that observed exit
                // makes a missing pane an idempotent close, not a missing socket.
                if !error.message.contains("pane_not_found") || !self.exited()? {
                    return Err(error);
                }
            }
            *closed = true;
        }
        Ok(())
    }
}
impl Drop for Launch {
    fn drop(&mut self) {
        let _ = self.close();
        let _ = fs::remove_dir_all(&self.report_dir);
    }
}
pub enum WaitEnd {
    Finished(Finished),
    Stopped,
    TimedOut,
}
pub(crate) fn sh_quote(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
}

pub fn locked_digest() -> &'static str {
    static LOCK: std::sync::OnceLock<serde_json::Value> = std::sync::OnceLock::new();
    let lock = LOCK.get_or_init(|| {
        serde_json::from_str(include_str!(env!("HCTL2_DEPENDENCY_LOCK"))).expect("dependency lock")
    });
    let platform = if cfg!(target_os = "linux") {
        "linux_x86_64"
    } else if cfg!(target_arch = "x86_64") {
        "macos_x86_64"
    } else {
        "macos_arm64"
    };
    lock["targets"][platform]["assets"]["herdr"]["sha256"]
        .as_str()
        .expect("locked Herdr digest")
}

pub fn installed_herdr(install_root: &Path) -> Result<PathBuf> {
    let path = install_root.join("libexec/hctl2/herdr");
    if !path.is_file() {
        return Err(PortError::new(
            "HERDR_BINARY_MISSING",
            format!("installed Herdr is not at {}", path.display()),
            "install_locked_herdr",
        ));
    }
    let digest = crate::catalog::file_digest(&path)?;
    if digest != locked_digest() {
        return Err(PortError::new(
            "HERDR_DIGEST_MISMATCH",
            format!("installed Herdr sha256 {digest} does not match lock.json"),
            "install_locked_herdr",
        ));
    }
    Ok(path)
}

pub fn smoke(
    server: Arc<Server>,
    exec_dir: &Path,
    state_dir: &Path,
    credential_root: &Path,
) -> Result<()> {
    let launch = Launch::start(
        server,
        exec_dir,
        state_dir,
        credential_root,
        "printf '%s\\n' smoke-ok\nexit 0\n",
        "smoke",
    )?;
    let finished = launch.wait(Duration::from_secs(15))?;
    if finished.code != Some(0) || !finished.stdout.windows(8).any(|item| item == b"smoke-ok") {
        return Err(PortError::new(
            "HERDR_SMOKE_FAILED",
            format!(
                "smoke exit {:?} stdout {:?} stderr {}",
                finished.code,
                finished.stdout,
                String::from_utf8_lossy(&finished.stderr)
            ),
            "check_locked_herdr",
        ));
    }
    let _ = launch.cancel();
    Ok(())
}

type LaunchScript = dyn Fn(&ExecutionSpec, &Bundle, &Path) -> Result<String> + Send + Sync;

pub struct InstalledHerdr {
    binary: PathBuf,
    profession_id: String,
    profession_revision: String,
    profession_digest: String,
    slot: Mutex<Option<Arc<Server>>>,
    starts: AtomicUsize,
    script: Option<Arc<LaunchScript>>,
    pool: Option<crate::standby::Pool>,
    codex_pool: Option<crate::codex::Pool>,
    codex_revision: Option<String>,
    codex_digest: Option<String>,
    codex_skip: Option<String>,
    read_paths: Vec<PathBuf>,
}

impl InstalledHerdr {
    pub fn open(binary: PathBuf, claude: &Path) -> Result<Self> {
        let idle = match std::env::var("HCTL2_AGENCY_IDLE_MS") {
            Ok(value) => value
                .parse::<u64>()
                .ok()
                .filter(|n| *n > 0)
                .map(Duration::from_millis)
                .ok_or_else(|| {
                    PortError::invalid("HCTL2_AGENCY_IDLE_MS must be a positive integer")
                })?,
            Err(std::env::VarError::NotPresent) => Duration::from_secs(300),
            Err(_) => return Err(PortError::invalid("HCTL2_AGENCY_IDLE_MS is not UTF-8")),
        };
        Self::open_with_idle(binary, claude, idle)
    }

    pub fn open_with_idle(binary: PathBuf, claude: &Path, idle: Duration) -> Result<Self> {
        Self::open_inner(binary, claude, idle, None)
    }

    /// Catalog and run this Codex binary. Does not consult `HCTL2_CODEX` or `PATH`.
    pub fn open_with_codex(
        binary: PathBuf,
        claude: &Path,
        codex: &Path,
        idle: Duration,
    ) -> Result<Self> {
        Self::open_inner(binary, claude, idle, Some(codex.to_path_buf()))
    }

    fn open_inner(
        binary: PathBuf,
        claude: &Path,
        idle: Duration,
        codex_override: Option<PathBuf>,
    ) -> Result<Self> {
        if idle.is_zero() {
            return Err(PortError::invalid("standby idle duration must be positive"));
        }
        let claude = claude.canonicalize()?;
        let mut read_paths = vec![
            claude
                .parent()
                .ok_or_else(|| PortError::invalid("Claude has no parent"))?
                .to_path_buf(),
        ];
        let version = smoke_binary(&binary, &claude, &read_paths)?;
        let (codex, codex_skip) = match codex_override {
            Some(path) => inspect_codex(&binary, path),
            None => probe_codex(&binary),
        };
        if let Some((path, _, _)) = &codex
            && let Some(parent) = path.parent()
        {
            read_paths.push(parent.to_path_buf());
        }
        if !crate::harness::version_at_least(&version, crate::harness::CLAUDE_MINIMUM) {
            return Err(PortError::new(
                "HARNESS_VERSION",
                format!(
                    "claude {version} is below {}",
                    crate::harness::CLAUDE_MINIMUM
                ),
                "upgrade_claude",
            ));
        }
        let digest = crate::catalog::file_digest(&claude)?;
        Ok(Self {
            binary,
            profession_id: "claude-code".into(),
            profession_revision: version,
            profession_digest: digest,
            slot: Mutex::new(None),
            starts: AtomicUsize::new(0),
            script: None,
            pool: Some(crate::standby::Pool::new(claude, idle)),
            codex_pool: codex
                .as_ref()
                .map(|(path, _, _)| crate::codex::Pool::new(path.clone(), idle)),
            codex_revision: codex.as_ref().map(|(_, revision, _)| revision.clone()),
            codex_digest: codex.as_ref().map(|(_, _, digest)| digest.clone()),
            codex_skip,
            read_paths,
        })
    }

    pub fn codex_skip(&self) -> Option<&str> {
        self.codex_skip.as_deref()
    }

    pub fn for_test(
        binary: PathBuf,
        script: impl Fn(&ExecutionSpec, &Bundle, &Path) -> Result<String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            binary,
            profession_id: "test-adapter".into(),
            profession_revision: "test".into(),
            profession_digest: "test".into(),
            slot: Mutex::new(None),
            starts: AtomicUsize::new(0),
            script: Some(Arc::new(script)),
            pool: None,
            codex_pool: None,
            codex_revision: None,
            codex_digest: None,
            codex_skip: None,
            read_paths: vec![],
        }
    }

    pub fn servers_started(&self) -> usize {
        self.starts.load(Ordering::SeqCst)
    }

    pub fn pid(&self) -> Option<u32> {
        self.slot
            .lock()
            .expect("herdr")
            .as_ref()
            .map(|server| server.pid())
    }

    fn ensure(&self, exec: &Path, credential_root: &Path) -> Result<Arc<Server>> {
        let mut slot = self.slot.lock().expect("herdr");
        if let Some(server) = slot.as_ref()
            && Client::connect(&server.socket)
                .and_then(|client| client.ping())
                .is_ok()
        {
            return Ok(Arc::clone(server));
        }
        let state = crate::herdr::state_dir(exec, credential_root)?;
        let parent = exec
            .parent()
            .ok_or_else(|| PortError::invalid("execution directory has no parent"))?;
        // Real dispatch directories share a private hctl2-exec-* parent. Never
        // grant a temporary-directory ancestor that also contains credentials.
        let parent = if crate::confine::allowed_tree_contains_credential(
            &parent.canonicalize()?,
            &credential_root.canonicalize()?,
        ) {
            exec
        } else {
            parent
        };
        let server = Arc::new(Server::start_with_read(
            &self.binary,
            &state,
            credential_root,
            parent,
            &self.read_paths,
        )?);
        if self.pool.is_some() {
            // Herdr's own installer, redirected privately. Never export this
            // variable to Claude: that would hide the user's native login.
            let integration = server.state.join("claude-integration");
            crate::storage::private_dir(&integration)?;
            let output = std::process::Command::new(&self.binary)
                .args(["integration", "install", "claude"])
                .env("CLAUDE_CONFIG_DIR", &integration)
                .output()?;
            if !output.status.success() {
                return Err(PortError::new(
                    "HERDR_INTEGRATION_FAILED",
                    String::from_utf8_lossy(&output.stderr),
                    "inspect_native_integration",
                ));
            }
        }
        self.starts.fetch_add(1, Ordering::SeqCst);
        *slot = Some(Arc::clone(&server));
        Ok(server)
    }
}

fn probe_codex(herdr: &Path) -> (Option<(PathBuf, String, String)>, Option<String>) {
    let Some(codex) = std::env::var_os("HCTL2_CODEX")
        .map(PathBuf::from)
        .or_else(codex_on_path)
    else {
        return (
            None,
            Some("codex is not on PATH and HCTL2_CODEX is unset".into()),
        );
    };
    inspect_codex(herdr, codex)
}

fn inspect_codex(
    herdr: &Path,
    codex: PathBuf,
) -> (Option<(PathBuf, String, String)>, Option<String>) {
    let Ok(codex) = codex.canonicalize() else {
        return (None, Some("codex path cannot be canonicalized".into()));
    };
    let Some(parent) = codex.parent().map(Path::to_path_buf) else {
        return (None, Some("codex has no parent directory".into()));
    };
    let version = match smoke_binary(herdr, &codex, &[parent]) {
        Ok(version) => version,
        Err(error) => return (None, Some(format!("codex smoke failed: {}", error.message))),
    };
    if !crate::harness::version_at_least(&version, crate::harness::CODEX_MINIMUM) {
        return (
            None,
            Some(format!(
                "codex {version} is below {}",
                crate::harness::CODEX_MINIMUM
            )),
        );
    }
    match crate::catalog::file_digest(&codex) {
        Ok(digest) => (Some((codex, version, digest)), None),
        Err(error) => (
            None,
            Some(format!("codex digest failed: {}", error.message)),
        ),
    }
}

fn codex_on_path() -> Option<PathBuf> {
    let output = Command::new("/usr/bin/which").arg("codex").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let path = text.lines().next()?.trim();
    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path))
    }
}

fn smoke_directory() -> Result<tempfile::TempDir> {
    Ok(tempfile::Builder::new()
        .prefix("hctl2-herdr-smoke-")
        .permissions(fs::Permissions::from_mode(0o700))
        .tempdir()?)
}

fn smoke_binary(binary: &Path, claude: &Path, read_paths: &[PathBuf]) -> Result<String> {
    // Time is not unique across parallel callers. TempDir owns an exclusively
    // created directory and outlives every probe Launch and Server, including errors.
    let root = smoke_directory()?;
    let cred = root.path().join("cred");
    let exec = root.path().join("exec");
    fs::create_dir_all(&cred)?;
    fs::create_dir_all(&exec)?;
    let state = crate::herdr::state_dir(&exec, &cred)?;
    let server = Arc::new(Server::start_with_read(
        binary, &state, &cred, &exec, read_paths,
    )?);
    let result = (|| {
        smoke(Arc::clone(&server), &exec, &state, &cred)?;
        let launch = Launch::start(
            Arc::clone(&server),
            &exec,
            &state,
            &cred,
            &format!("{} --version\n", sh_quote(claude)),
            "harness-smoke",
        )?;
        let finished = launch.wait(Duration::from_secs(15))?;
        let text = String::from_utf8_lossy(&finished.stdout);
        let version = text.lines().next().unwrap_or("").trim().to_owned();
        if finished.code != Some(0) || version.is_empty() {
            return Err(PortError::new(
                "HARNESS_SMOKE_FAILED",
                format!(
                    "confined Claude --version exit {:?}: {}",
                    finished.code,
                    String::from_utf8_lossy(&finished.stderr)
                        .chars()
                        .take(4096)
                        .collect::<String>()
                ),
                "check_harness_installation_and_policy",
            ));
        }
        Ok(version)
    })();
    drop(server);
    result
}

struct HerdrSession {
    launch: Arc<Launch>,
}

impl Drop for HerdrSession {
    fn drop(&mut self) {
        let _ = self.launch.cancel();
    }
}

impl Session for HerdrSession {
    fn input(&mut self, _: &[u8]) -> Result<()> {
        Err(PortError::invalid("this launch does not take pane input"))
    }
    fn stop(&mut self) -> Result<()> {
        self.launch.cancel()
    }
}

impl Runtime for InstalledHerdr {
    fn shutdown(&self) -> Result<()> {
        let codex = self
            .codex_pool
            .as_ref()
            .map(|pool| pool.shutdown())
            .transpose();
        let pooled = self.pool.as_ref().map(|pool| pool.shutdown()).transpose();
        let pooled = codex.and(pooled);
        let mut slot = self.slot.lock().expect("herdr");
        if let Some(server) = slot.as_ref() {
            // Detached event readers may still hold Arc<Server> when the Agency
            // process exits. Do not rely on their Drop to stop the native server.
            Client::connect(&server.socket)?.call("server.stop", json!({}))?;
            *slot = None;
        }
        pooled.map(|_| ())
    }

    fn catalog(&self) -> Result<Catalog> {
        let herdr_digest = crate::catalog::file_digest(&self.binary)?;
        let harness = FrozenRef {
            id: "herdr".into(),
            revision: "protocol-22".into(),
            digest: herdr_digest,
        };
        let profession = Profession {
            reference: FrozenRef {
                id: self.profession_id.clone(),
                revision: self.profession_revision.clone(),
                digest: self.profession_digest.clone(),
            },
            harness: harness.clone(),
            model: "none".into(),
            persona: "claude code".into(),
            terms: "read-only Bundle dispatch; native Claude turns in a selection-local Herdr session; no tool input or write lease".into(),
            default_role: "worker".into(),
            skills: vec![],
            capabilities: Capabilities {
                input: false,
                stop: true,
                ..Capabilities::default()
            },
        };
        let mut professions = vec![profession];
        if let (Some(revision), Some(digest)) = (&self.codex_revision, &self.codex_digest) {
            professions.push(Profession {
                reference: FrozenRef {
                    id: "codex-cli".into(),
                    revision: revision.clone(),
                    digest: digest.clone(),
                },
                harness: harness.clone(),
                model: "none".into(),
                persona: "codex".into(),
                terms: "read-only Bundle dispatch; app-server turn/start for one selection thread; Herdr pane runs codex resume --remote; no write lease".into(),
                default_role: "worker".into(),
                skills: vec![],
                capabilities: Capabilities {
                    input: false,
                    stop: true,
                    ..Capabilities::default()
                },
            });
        }
        Ok(Catalog {
            professions,
            harnesses: vec![harness],
            skills: vec![],
        })
    }

    fn start(
        &self,
        spec: &Sealed<ExecutionSpec>,
        bundle: &Sealed<Bundle>,
        exec_root: &Path,
        credential_root: &Path,
    ) -> Result<Running> {
        fs::create_dir_all(exec_root)?;
        let server = self.ensure(exec_root, credential_root)?;
        if spec.document.profession.reference.id == "codex-cli" {
            let Some(pool) = &self.codex_pool else {
                return Err(PortError::new(
                    "HARNESS_NOT_STARTED",
                    self.codex_skip
                        .clone()
                        .unwrap_or_else(|| "codex is not cataloged".into()),
                    "install_codex",
                ));
            };
            return pool.submit(server, spec, bundle, exec_root, credential_root);
        }
        if let Some(pool) = &self.pool {
            return pool.submit(server, spec, bundle, exec_root, credential_root);
        }
        let body = (self.script.as_ref().expect("test adapter"))(
            &spec.document,
            &bundle.document,
            exec_root,
        )?;
        let state = server.state.clone();
        let launch = Launch::start(
            server,
            exec_root,
            &state,
            credential_root,
            &body,
            &spec.document.idempotency_key,
        )?;
        let shared = Arc::new(launch);
        let waiter = Arc::clone(&shared);
        let timeout = remaining(spec.document.deadline_ms);
        let (tx, rx) = std::sync::mpsc::sync_channel(8);
        std::thread::spawn(move || dispatch_events(waiter, timeout, tx));
        Ok(Running {
            session: Arc::new(Mutex::new(Box::new(HerdrSession { launch: shared }))),
            events: rx,
        })
    }
    fn start_for_tenant(
        &self,
        tenant: &Path,
        spec: &Sealed<ExecutionSpec>,
        bundle: &Sealed<Bundle>,
        exec_root: &Path,
        credential_root: &Path,
    ) -> Result<Running> {
        if spec.document.profession.reference.id == "codex-cli" {
            fs::create_dir_all(exec_root)?;
            let server = self.ensure(exec_root, credential_root)?;
            let Some(pool) = &self.codex_pool else {
                return Err(PortError::new(
                    "HARNESS_NOT_STARTED",
                    self.codex_skip
                        .clone()
                        .unwrap_or_else(|| "codex is not cataloged".into()),
                    "install_codex",
                ));
            };
            return pool.submit(server, spec, bundle, exec_root, tenant);
        }
        if let Some(pool) = &self.pool {
            fs::create_dir_all(exec_root)?;
            let server = self.ensure(exec_root, credential_root)?;
            pool.submit(server, spec, bundle, exec_root, tenant)
        } else {
            self.start(spec, bundle, exec_root, credential_root)
        }
    }
}

fn remaining(deadline_ms: u64) -> Duration {
    Duration::from_millis(deadline_ms.saturating_sub(crate::storage::now_ms()))
}

fn dispatch_events(
    launch: Arc<Launch>,
    timeout: Duration,
    tx: std::sync::mpsc::SyncSender<RuntimeEvent>,
) {
    let deadline = Instant::now() + timeout;
    loop {
        if launch.stopped.load(Ordering::SeqCst) {
            let _ = tx.send(RuntimeEvent::Exited {
                code: None,
                requested_stop: true,
            });
            return;
        }
        let observed = (|| -> Result<(Vec<u8>, Vec<u8>, bool)> {
            let exited = launch.exited()?;
            let (stdout, stderr) = launch.output()?;
            Ok((stdout, stderr, exited))
        })();
        let (stdout, _stderr, exited) = match observed {
            Ok(value) => value,
            Err(error) => {
                let _ = tx.send(RuntimeEvent::ProtocolError(error.code));
                // A close failure is not evidence of exit. Keep the session available for retry.
                if launch.close().is_ok() {
                    let _ = tx.send(RuntimeEvent::Exited {
                        code: None,
                        requested_stop: false,
                    });
                }
                return;
            }
        };
        if exited {
            let code = launch.exit_code().ok().flatten();
            // Only the explicit script fixture uses stdout-at-exit; no real harness takes this branch.
            if code == Some(0) {
                let _ = tx.send(RuntimeEvent::Proposal {
                    schema: "adapter.stdout.v1".into(),
                    bytes: stdout,
                    source: EvidenceLevel::AdapterEvent,
                });
                let _ = tx.send(RuntimeEvent::TurnReturned);
            }
            let _ = launch.close();
            let _ = tx.send(RuntimeEvent::Exited {
                code,
                requested_stop: false,
            });
            return;
        }
        if Instant::now() >= deadline {
            let _ = tx.send(RuntimeEvent::DeadlineReached);
            match launch.close() {
                Ok(()) => {
                    let _ = tx.send(RuntimeEvent::Exited {
                        code: None,
                        requested_stop: false,
                    });
                }
                Err(error) => {
                    let _ = tx.send(RuntimeEvent::ProtocolError(error.code));
                }
            }
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

pub(crate) fn task_text(bundle: &Bundle) -> Result<String> {
    let mut text = String::new();
    for entry in &bundle.entries {
        let bytes = match &entry.delivery {
            Delivery::Pointer { bytes, .. } | Delivery::Inline { bytes } => bytes.as_slice(),
            Delivery::Recall { .. } => continue,
        };
        let piece = std::str::from_utf8(bytes)
            .map_err(|_| PortError::invalid("bundle task is not utf-8"))?;
        text.push_str(piece);
        text.push('\n');
    }
    if text.is_empty() {
        return Err(PortError::invalid("bundle has no task text"));
    }
    Ok(text)
}

#[cfg(test)]
mod smoke_tests {
    use super::*;
    use std::{collections::HashSet, sync::Barrier};

    #[test]
    fn parallel_smoke_directories_are_private_and_cleanup_is_independent() {
        let barrier = Barrier::new(32);
        let mut directories = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..32)
                .map(|_| {
                    scope.spawn(|| {
                        barrier.wait();
                        let directory = smoke_directory().unwrap();
                        fs::write(directory.path().join("owned"), b"probe").unwrap();
                        directory
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().unwrap())
                .collect::<Vec<_>>()
        });
        let paths: HashSet<_> = directories.iter().map(|d| d.path().to_path_buf()).collect();
        assert_eq!(paths.len(), 32);
        for directory in &directories {
            assert_eq!(
                fs::metadata(directory.path()).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
        let removed = directories.pop().unwrap();
        let removed_path = removed.path().to_path_buf();
        drop(removed);
        assert!(!removed_path.exists());
        for directory in &directories {
            assert_eq!(fs::read(directory.path().join("owned")).unwrap(), b"probe");
        }
        drop(directories);
        assert!(paths.iter().all(|p| !p.exists()));
    }
}
