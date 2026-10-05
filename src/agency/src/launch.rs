//! Non-interactive process supervision with a Herdr-owned display terminal.
//! Completion comes from the OS child status and protocol stream, never the screen.
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
    io::{Read, Write},
    os::unix::{
        fs::{FileTypeExt, OpenOptionsExt},
        net::UnixStream,
        process::{CommandExt, ExitStatusExt},
    },
    path::{Path, PathBuf},
    process::{Child, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

pub struct Launch {
    client: Client,
    pane: String,
    process: Mutex<Option<Process>>,
    report_dir: PathBuf,
    program: PathBuf,
    profile: PathBuf,
    stopped: Arc<AtomicBool>,
    closed: Mutex<bool>,
    _server: Arc<Server>,
}

struct Process {
    child: Child,
    stdout: Capture,
    stderr: Capture,
    terminal: fs::File,
}

struct Capture {
    stream: UnixStream,
    bytes: Vec<u8>,
    eof: bool,
}

impl Capture {
    fn drain(&mut self, terminal: &mut fs::File) -> Result<()> {
        let mut buffer = [0; 8192];
        // Bound work per poll even when a program produces output indefinitely.
        for _ in 0..32 {
            match self.stream.read(&mut buffer) {
                Ok(0) => {
                    self.eof = true;
                    break;
                }
                Ok(n) => {
                    if self.bytes.len() + n > 16 * 1024 * 1024 {
                        return Err(PortError::new(
                            "LAUNCH_OUTPUT_LIMIT",
                            "harness stream exceeds 16 MiB",
                            "reduce_harness_output",
                        ));
                    }
                    self.bytes.extend_from_slice(&buffer[..n]);
                    // Display is best effort and is not the result evidence channel.
                    let _ = terminal.write(&buffer[..n]);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }
}

impl Process {
    fn drain(&mut self) -> Result<()> {
        self.stdout.drain(&mut self.terminal)?;
        self.stderr.drain(&mut self.terminal)
    }
    fn code(&mut self) -> Result<Option<i32>> {
        Ok(self
            .child
            .try_wait()?
            .map(|s| s.code().unwrap_or_else(|| 128 + s.signal().unwrap_or(0))))
    }
    fn stop(&mut self) -> Result<()> {
        if self.child.try_wait()?.is_none() {
            let status = std::process::Command::new("/bin/kill")
                .args(["-KILL", "--", &format!("-{}", self.child.id())])
                .status()?;
            if !status.success() {
                self.child.kill()?;
            }
            self.child.wait()?;
        }
        Ok(())
    }
}

pub struct Finished {
    pub code: i32,
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
        let stamp = crate::storage::nonce()?;
        let program = exec_dir.join(format!("program-{stamp}.sh"));
        let profile = exec_dir.join(format!("program-{stamp}.sb"));
        let report_dir = state_dir.join(format!("launch-{stamp}"));
        crate::storage::private_dir(&report_dir)?;
        fs::write(&program, body)?;
        let runner = report_dir.join("runner.sh");
        fs::write(
            &runner,
            format!(
                "#!/bin/sh\ntty > {tty}\nwhile [ ! -f {done} ]; do sleep 0.05; done\n",
                tty = sh_quote(&report_dir.join("tty")),
                done = sh_quote(&report_dir.join("done"))
            ),
        )?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut mode = fs::metadata(&runner)?.permissions();
            mode.set_mode(0o755);
            fs::set_permissions(&runner, mode)?;
        }
        let client = Client::connect(&server.socket)?;
        let created = client.call(
            "workspace.create",
            json!({"cwd": exec_dir, "label": label, "focus": false}),
        )?;
        let pane = created
            .pointer("/root_pane/pane_id")
            .and_then(|value| value.as_str())
            .ok_or_else(|| {
                PortError::new(
                    "HERDR_PROTOCOL",
                    "workspace has no pane",
                    "retry_herdr_call",
                )
            })?
            .to_owned();
        let launch = Self {
            client,
            pane,
            report_dir,
            program,
            profile,
            process: Mutex::new(None),
            stopped: Arc::new(AtomicBool::new(false)),
            closed: Mutex::new(false),
            _server: server,
        };
        launch.client.call(
            "pane.send_text",
            json!({"pane_id": launch.pane, "text": format!("/bin/sh {}\n", sh_quote(&runner))}),
        )?;
        let ready = Instant::now() + Duration::from_secs(5);
        let tty = loop {
            if let Ok(text) = fs::read_to_string(launch.report_dir.join("tty")) {
                let path = PathBuf::from(text.trim());
                if (path.starts_with("/dev/pts") || path.to_string_lossy().starts_with("/dev/tty"))
                    && fs::metadata(&path).is_ok_and(|m| m.file_type().is_char_device())
                {
                    break path;
                }
            }
            if Instant::now() >= ready {
                return Err(PortError::new(
                    "HERDR_TERMINAL_MISSING",
                    "pane did not supply its terminal",
                    "retry_herdr_call",
                ));
            }
            std::thread::sleep(Duration::from_millis(25));
        };
        let terminal = fs::OpenOptions::new()
            .write(true)
            .custom_flags(rustix::fs::OFlags::NONBLOCK.bits() as i32)
            .open(tty)?;
        let (stdout, child_stdout) = UnixStream::pair()?;
        let (stderr, child_stderr) = UnixStream::pair()?;
        stdout.set_nonblocking(true)?;
        stderr.set_nonblocking(true)?;
        let mut command = crate::confine::pane_program(
            &launch.program,
            exec_dir,
            credential_root,
            state_dir,
            launch
                ._server
                .socket
                .parent()
                .ok_or_else(|| PortError::invalid("socket has no parent"))?,
            &launch.profile,
        )?;
        crate::confine::scrub(&mut command, exec_dir);
        command
            .env(
                "HCTL2_CONFINE_READ",
                launch
                    ._server
                    .read_paths
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
            .process_group(0)
            .stdin(Stdio::null())
            .stdout(Stdio::from(std::os::fd::OwnedFd::from(child_stdout)))
            .stderr(Stdio::from(std::os::fd::OwnedFd::from(child_stderr)));
        let child = command.spawn()?;
        *launch.process.lock().expect("launch process") = Some(Process {
            child,
            terminal,
            stdout: Capture {
                stream: stdout,
                bytes: vec![],
                eof: false,
            },
            stderr: Capture {
                stream: stderr,
                bytes: vec![],
                eof: false,
            },
        });
        Ok(launch)
    }

    pub fn pane(&self) -> &str {
        &self.pane
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Native child status. Files in the program's directory cannot affect it.
    pub fn poll_exit(&self) -> Result<Option<i32>> {
        if self.stopped.load(Ordering::SeqCst) {
            return Ok(None);
        }
        let mut process = self.process.lock().expect("launch process");
        let process = process
            .as_mut()
            .ok_or_else(|| PortError::invalid("launch has no process"))?;
        process.drain()?;
        process.code()
    }

    pub fn wait(&self, timeout: Duration) -> Result<Finished> {
        match self.wait_end(timeout)? {
            WaitEnd::Finished(finished) => Ok(finished),
            WaitEnd::Stopped => Err(PortError::new(
                "LAUNCH_STOPPED",
                "the caller stopped the launch",
                "read_exit_event",
            )),
            WaitEnd::TimedOut => Err(PortError::new(
                "LAUNCH_TIMEOUT",
                "harness did not exit and close its output before the caller timeout",
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
            {
                let mut process = self.process.lock().expect("launch process");
                let process = process
                    .as_mut()
                    .ok_or_else(|| PortError::invalid("launch has no process"))?;
                process.drain()?;
                if let Some(code) = process.code()? {
                    if self.stopped.load(Ordering::SeqCst) {
                        return Ok(WaitEnd::Stopped);
                    }
                    process.drain()?;
                    if process.stdout.eof && process.stderr.eof {
                        return Ok(WaitEnd::Finished(Finished {
                            code,
                            stdout: process.stdout.bytes.clone(),
                            stderr: process.stderr.bytes.clone(),
                        }));
                    }
                }
            }
            if Instant::now() >= deadline {
                return Ok(WaitEnd::TimedOut);
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    pub fn cancel(&self) -> Result<()> {
        self.close()?;
        let mut guard = self.process.lock().expect("launch process");
        if let Some(process) = guard.as_mut() {
            process.stop()?;
        }
        self.stopped.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn close(&self) -> Result<()> {
        let mut closed = self.closed.lock().expect("pane close");
        if !*closed {
            self.client
                .call("pane.close", json!({"pane_id": self.pane}))?;
            *closed = true;
        }
        Ok(())
    }

    fn stop_process(&self) -> Result<()> {
        if let Some(process) = self.process.lock().expect("launch process").as_mut() {
            process.stop()?;
        }
        Ok(())
    }
}

impl Drop for Launch {
    fn drop(&mut self) {
        let _ = self.close();
        if let Some(process) = self.process.get_mut().expect("launch process").as_mut() {
            let _ = process.stop();
        }
        let _ = fs::remove_file(&self.program);
        let _ = fs::remove_file(&self.profile);
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
    if finished.code != 0 || !finished.stdout.windows(8).any(|item| item == b"smoke-ok") {
        return Err(PortError::new(
            "HERDR_SMOKE_FAILED",
            format!(
                "smoke exit {} stdout {:?} stderr {}",
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
        if finished.code != 0 || version.is_empty() {
            return Err(PortError::new(
                "HARNESS_SMOKE_FAILED",
                format!(
                    "confined Claude --version exit {}: {}",
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

impl Session for HerdrSession {
    fn input(&mut self, _: &[u8]) -> Result<()> {
        Err(PortError::invalid("this launch does not take pane input"))
    }
    fn stop(&mut self) -> Result<()> {
        self.launch.cancel()
    }
}

impl Runtime for InstalledHerdr {
    fn catalog(&self) -> Result<Catalog> {
        let herdr_digest = crate::catalog::file_digest(&self.binary)?;
        let harness = FrozenRef {
            id: "herdr".into(),
            revision: "protocol-20".into(),
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
    let ended = match launch.wait_end(timeout) {
        Ok(WaitEnd::Stopped) => {
            let _ = tx.send(RuntimeEvent::Exited {
                code: None,
                requested_stop: true,
            });
            return;
        }
        Ok(WaitEnd::TimedOut) => {
            // A failed pane RPC must not leave our child running past its deadline.
            let _ = launch.close();
            let _ = launch.stop_process();
            let _ = tx.send(RuntimeEvent::DeadlineReached);
            let _ = tx.send(RuntimeEvent::Exited {
                code: None,
                requested_stop: false,
            });
            return;
        }
        Ok(WaitEnd::Finished(finished)) => finished,
        Err(error) => {
            let _ = launch.close();
            let _ = launch.stop_process();
            let _ = tx.send(RuntimeEvent::ProtocolError(error.code));
            let _ = tx.send(RuntimeEvent::Exited {
                code: None,
                requested_stop: false,
            });
            return;
        }
    };
    if let Err(error) = launch.close() {
        let _ = tx.send(RuntimeEvent::Observation {
            kind: "runtime:pane_cleanup_failed".into(),
            payload: json!({"code": error.code, "message": error.message}),
            source: EvidenceLevel::AdapterEvent,
        });
    }
    if ended.code == 0 {
        match proposal_bytes(&ended, claude_result) {
            Ok(bytes) => {
                let _ = tx.send(RuntimeEvent::Proposal {
                    schema: if claude_result {
                        "claude.result.v1".into()
                    } else {
                        "adapter.stdout.v1".into()
                    },
                    bytes,
                    source: EvidenceLevel::AdapterEvent,
                });
                let _ = tx.send(RuntimeEvent::Exited {
                    code: Some(0),
                    requested_stop: false,
                });
            }
            Err(error) => {
                let _ = tx.send(RuntimeEvent::Observation {
                    kind: "runtime:harness_failure".into(),
                    payload: json!({"code": error.code, "message": error.message}),
                    source: EvidenceLevel::AdapterEvent,
                });
                let _ = tx.send(RuntimeEvent::ProtocolError(error.code));
                let _ = tx.send(RuntimeEvent::Exited {
                    code: Some(0),
                    requested_stop: false,
                });
            }
        }
    } else {
        let detail =
            crate::harness::claude::result_from_jsonl(&String::from_utf8_lossy(&ended.stdout))
                .map(|session| session.result)
                .unwrap_or_else(|_| String::from_utf8_lossy(&ended.stderr).into_owned());
        let _ = tx.send(RuntimeEvent::Observation {
            kind: "runtime:harness_failure".into(),
            payload: json!({"exit_code": ended.code, "message": detail.chars().take(4096).collect::<String>()}),
            source: EvidenceLevel::AdapterEvent,
        });
        let _ = tx.send(RuntimeEvent::Exited {
            code: Some(ended.code),
            requested_stop: false,
        });
    }
}

fn proposal_bytes(finished: &Finished, claude_result: bool) -> Result<Vec<u8>> {
    if !claude_result {
        return Ok(finished.stdout.clone());
    }
    let text = String::from_utf8_lossy(&finished.stdout);
    let session = crate::harness::claude::result_from_jsonl(&text)?;
    if session.is_error {
        return Err(PortError::new(
            "HARNESS_RESULT_ERROR",
            session.result,
            "read_claude_result",
        ));
    }
    Ok(session.result.into_bytes())
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
