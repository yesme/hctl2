//! Start a program in a Herdr pane without treating the screen as completion.
//! The caller's text is a script file. The pane only receives a fixed launcher.
//! The launcher writes the script's exit code after the script returns.
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
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

pub struct Launch {
    client: Client,
    pane: String,
    exit_path: PathBuf,
    stdout_path: PathBuf,
    stopped: Arc<AtomicBool>,
    _server: Arc<Server>,
}

pub struct Finished {
    pub code: i32,
    pub stdout: Vec<u8>,
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
        let _ = (state_dir, credential_root);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let program = exec_dir.join(format!("program-{label}-{stamp}.sh"));
        let exit_path = exec_dir.join(format!("exit-{label}-{stamp}"));
        let stdout_path = exec_dir.join(format!("stdout-{label}-{stamp}"));
        let stderr_path = exec_dir.join(format!("stderr-{label}-{stamp}"));
        fs::write(&program, body)?;
        let runner = exec_dir.join(format!("runner-{label}-{stamp}.sh"));
        fs::write(
            &runner,
            runner_script(&program, &exit_path, &stdout_path, &stderr_path),
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
        client.call(
            "pane.send_text",
            json!({"pane_id": pane, "text": format!("/bin/sh {}\n", sh_quote(&runner))}),
        )?;
        Ok(Self {
            client,
            pane,
            exit_path,
            stdout_path,
            stopped: Arc::new(AtomicBool::new(false)),
            _server: server,
        })
    }

    pub fn pane(&self) -> &str {
        &self.pane
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Exit code written by the launcher. Screen text is not consulted.
    pub fn poll_exit(&self) -> Result<Option<i32>> {
        if !self.exit_path.is_file() {
            return Ok(None);
        }
        let text = fs::read_to_string(&self.exit_path)?;
        let code = text
            .trim()
            .parse::<i32>()
            .map_err(|_| PortError::invalid("launcher exit file is not an integer"))?;
        Ok(Some(code))
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
                "launcher exit file was not written before the caller timeout",
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
            if let Some(code) = self.poll_exit()? {
                if self.stopped.load(Ordering::SeqCst) {
                    return Ok(WaitEnd::Stopped);
                }
                let stdout = fs::read(&self.stdout_path).unwrap_or_default();
                return Ok(WaitEnd::Finished(Finished { code, stdout }));
            }
            if Instant::now() >= deadline {
                return Ok(WaitEnd::TimedOut);
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    pub fn cancel(&self) -> Result<()> {
        self.stopped.store(true, Ordering::SeqCst);
        self.client
            .call("pane.close", json!({"pane_id": self.pane}))
            .map(|_| ())
    }
}

pub enum WaitEnd {
    Finished(Finished),
    Stopped,
    TimedOut,
}

fn runner_script(
    program: &Path,
    exit_path: &Path,
    stdout_path: &Path,
    stderr_path: &Path,
) -> String {
    format!(
        "#!/bin/sh\nset +e\n/bin/sh {program} >{stdout} 2>{stderr}\ncode=$?\nprintf '%s\\n' \"$code\" > {exit}\nexit \"$code\"\n",
        program = sh_quote(program),
        stdout = sh_quote(stdout_path),
        stderr = sh_quote(stderr_path),
        exit = sh_quote(exit_path),
    )
}

fn sh_quote(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
}

pub fn locked_digest() -> &'static str {
    if cfg!(target_os = "linux") {
        "976150a14d490c94b243ea2e1a7eb2dfb67f12e36b182db90936f6728e6aecf4"
    } else if cfg!(target_arch = "x86_64") {
        "ab50262c8190cd7aa9056d249d255c08c328c3e8716de9cfa29db4f131b8e2c1"
    } else {
        "a5d4f4d504d8b309c91f811050559300faba31258425f53c50852fc96f6ae574"
    }
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
            format!("smoke exit {} stdout {:?}", finished.code, finished.stdout),
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
}

impl InstalledHerdr {
    pub fn open(binary: PathBuf, claude: &Path) -> Result<Self> {
        let version = command_version(claude)?;
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
        smoke_binary(&binary)?;
        let digest = crate::catalog::file_digest(claude)?;
        let claude = claude.to_path_buf();
        Ok(Self {
            binary,
            profession_id: "claude-code".into(),
            profession_revision: version,
            profession_digest: digest,
            slot: Mutex::new(None),
            starts: AtomicUsize::new(0),
            script: Arc::new(move |_spec, bundle, exec| claude_script(&claude, bundle, exec)),
            claude_result: true,
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
        let server = Arc::new(Server::start(&self.binary, &state, credential_root, exec)?);
        self.starts.fetch_add(1, Ordering::SeqCst);
        *slot = Some(Arc::clone(&server));
        Ok(server)
    }
}

fn smoke_binary(binary: &Path) -> Result<()> {
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
    let server = Arc::new(Server::start(binary, &state, &cred, &exec)?);
    let result = smoke(Arc::clone(&server), &exec, &state, &cred);
    drop(server);
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&state);
    result
}

fn command_version(binary: &Path) -> Result<String> {
    let output = std::process::Command::new(binary)
        .arg("--version")
        .output()
        .map_err(|error| {
            PortError::new(
                "HARNESS_NOT_STARTED",
                format!("{} did not answer --version: {error}", binary.display()),
                "install_claude_code",
            )
        })?;
    if !output.status.success() {
        return Err(PortError::new(
            "HARNESS_NOT_STARTED",
            format!("{} --version failed", binary.display()),
            "install_claude_code",
        ));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text.lines().next().unwrap_or("").trim().to_owned())
}

fn claude_script(claude: &Path, bundle: &Bundle, exec: &Path) -> Result<String> {
    let prompt = exec.join("prompt.txt");
    fs::write(&prompt, task_text(bundle)?)?;
    let home = std::env::var("HOME").unwrap_or_default();
    let path = std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".into());
    Ok(format!(
        "export HOME={home}\nexport PATH={path}\nexport CLAUDE_CONFIG_DIR={config}\nexec {claude} -p --output-format stream-json --verbose --permission-mode dontAsk < {prompt}\n",
        home = sh_quote(Path::new(&home)),
        path = sh_quote(Path::new(&path)),
        config = sh_quote(&Path::new(&home).join(".claude")),
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
        let state = crate::herdr::state_dir(exec_root, credential_root)?;
        let server = self.ensure(exec_root, credential_root)?;
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
            let _ = launch.cancel();
            let _ = tx.send(RuntimeEvent::DeadlineReached);
            let _ = tx.send(RuntimeEvent::Exited {
                code: None,
                requested_stop: false,
            });
            return;
        }
        Ok(WaitEnd::Finished(finished)) => finished,
        Err(error) => {
            let _ = tx.send(RuntimeEvent::ProtocolError(error.code));
            let _ = tx.send(RuntimeEvent::Exited {
                code: None,
                requested_stop: false,
            });
            return;
        }
    };
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
                let _ = tx.send(RuntimeEvent::ProtocolError(error.code));
                let _ = tx.send(RuntimeEvent::Exited {
                    code: Some(0),
                    requested_stop: false,
                });
            }
        }
    } else {
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
