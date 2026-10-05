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
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
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
fn sh_quote(path: &Path) -> String {
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
    script: Arc<LaunchScript>,
    claude_result: bool,
    read_paths: Vec<PathBuf>,
}

impl InstalledHerdr {
    pub fn open(binary: PathBuf, claude: &Path) -> Result<Self> {
        let claude = claude.canonicalize()?;
        let read_paths = vec![
            claude
                .parent()
                .ok_or_else(|| PortError::invalid("Claude has no parent"))?
                .to_path_buf(),
        ];
        let version = smoke_binary(&binary, &claude, &read_paths)?;
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
            script: Arc::new(move |_spec, bundle, exec| claude_script(&claude, bundle, exec)),
            claude_result: true,
            read_paths,
        })
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
            script: Arc::new(script),
            claude_result: false,
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
        self.starts.fetch_add(1, Ordering::SeqCst);
        *slot = Some(Arc::clone(&server));
        Ok(server)
    }
}

fn smoke_binary(binary: &Path, claude: &Path, read_paths: &[PathBuf]) -> Result<String> {
    let root = std::env::temp_dir().join(format!(
        "hctl2-herdr-smoke-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let cred = root.join("cred");
    let exec = root.join("exec");
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
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&state);
    result
}

fn claude_script(claude: &Path, bundle: &Bundle, exec: &Path) -> Result<String> {
    let prompt = exec.join("prompt.txt");
    fs::write(&prompt, task_text(bundle)?)?;
    let home = std::env::var("HOME").unwrap_or_default();
    let path = std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".into());
    let user = std::env::var("USER").unwrap_or_default();
    Ok(format!(
        "export HOME={home}\nexport PATH={path}\nexport USER={user}\nunset CLAUDE_CONFIG_DIR\nexec {claude} -p --output-format stream-json --verbose --permission-mode dontAsk < {prompt}\n",
        home = sh_quote(Path::new(&home)),
        path = sh_quote(Path::new(&path)),
        user = sh_quote(Path::new(&user)),
        claude = sh_quote(claude),
        prompt = sh_quote(&prompt),
    ))
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
        let mut slot = self.slot.lock().expect("herdr");
        if let Some(server) = slot.as_ref() {
            // Detached event readers may still hold Arc<Server> when the Agency
            // process exits. Do not rely on their Drop to stop the native server.
            Client::connect(&server.socket)?.call("server.stop", json!({}))?;
            *slot = None;
        }
        Ok(())
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
            terms:
                "the adapter writes the launch script; bundle text is the task, not a shell program"
                    .into(),
            default_role: "worker".into(),
            skills: vec![],
            capabilities: Capabilities {
                input: false,
                stop: true,
                ..Capabilities::default()
            },
        };
        Ok(Catalog {
            professions: vec![profession],
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
        let body = (self.script)(&spec.document, &bundle.document, exec_root)?;
        let server = self.ensure(exec_root, credential_root)?;
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
        let claude_result = self.claude_result;
        let (tx, rx) = std::sync::mpsc::sync_channel(8);
        std::thread::spawn(move || dispatch_events(waiter, timeout, claude_result, tx));
        Ok(Running {
            session: Arc::new(Mutex::new(Box::new(HerdrSession { launch: shared }))),
            events: rx,
        })
    }
}

fn remaining(deadline_ms: u64) -> Duration {
    Duration::from_millis(deadline_ms.saturating_sub(crate::storage::now_ms()))
}

fn dispatch_events(
    launch: Arc<Launch>,
    timeout: Duration,
    claude_result: bool,
    tx: std::sync::mpsc::SyncSender<RuntimeEvent>,
) {
    let deadline = Instant::now() + timeout;
    let mut returned = false;
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
        let (stdout, stderr, exited) = match observed {
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
        if claude_result && !returned {
            // The file may end midway through a JSONL record. Only complete records count.
            let complete = stdout
                .iter()
                .rposition(|b| *b == b'\n')
                .map_or(0, |i| i + 1);
            match crate::harness::claude::result_from_jsonl(&String::from_utf8_lossy(
                &stdout[..complete],
            )) {
                Ok(session) if !session.is_error => {
                    let _ = tx.send(RuntimeEvent::Proposal {
                        schema: "claude.result.v1".into(),
                        bytes: session.result.into_bytes(),
                        source: EvidenceLevel::AdapterEvent,
                    });
                    let _ = tx.send(RuntimeEvent::TurnReturned);
                    returned = true;
                }
                Ok(session) => {
                    let _ = tx.send(RuntimeEvent::Observation {
                        kind: "runtime:harness_failure".into(),
                        payload: json!({"code":"HARNESS_RESULT_ERROR","message":session.result}),
                        source: EvidenceLevel::AdapterEvent,
                    });
                    let _ = tx.send(RuntimeEvent::ProtocolError("HARNESS_RESULT_ERROR".into()));
                    if launch.close().is_ok() {
                        let _ = tx.send(RuntimeEvent::Exited {
                            code: launch.exit_code().ok().flatten(),
                            requested_stop: false,
                        });
                    }
                    return;
                }
                Err(error) if error.code == "HARNESS_RESULT_MISSING" && !exited => {}
                Err(error) => {
                    let _ = tx.send(RuntimeEvent::Observation {
                        kind:"runtime:harness_failure".into(),
                        payload:json!({"code":error.code,"message":String::from_utf8_lossy(&stderr).chars().take(4096).collect::<String>()}),
                        source:EvidenceLevel::AdapterEvent,
                    });
                    let _ = tx.send(RuntimeEvent::ProtocolError(error.code));
                    if launch.close().is_ok() {
                        let _ = tx.send(RuntimeEvent::Exited {
                            code: launch.exit_code().ok().flatten(),
                            requested_stop: false,
                        });
                    }
                    return;
                }
            }
        }
        if exited {
            let code = launch.exit_code().ok().flatten();
            // Only the explicit script fixture uses stdout-at-exit; no real harness takes this branch.
            if !claude_result && code == Some(0) {
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
            if !returned {
                let _ = tx.send(RuntimeEvent::DeadlineReached);
            }
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

fn task_text(bundle: &Bundle) -> Result<String> {
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
