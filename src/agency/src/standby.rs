//! Selection-local FIFO. Herdr owns the process; Claude owns history and turn boundaries.
use crate::{
    herdr::{Client, Server},
    runtime::{Running, RuntimeEvent, Session},
};
use agency_proto::{
    EvidenceLevel, ExecutionSpec, PortError, Result, Sealed, canonical, context::Bundle, hash,
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

const LIMIT: u64 = 16 * 1024 * 1024;

struct Token {
    writing: bool,
    cancelled: AtomicBool,
    finished: AtomicBool,
}
struct Handle(Arc<Token>);
impl Session for Handle {
    fn input(&mut self, _: &[u8]) -> Result<()> {
        Err(PortError::invalid("standby input is not advertised"))
    }
    fn stop(&mut self) -> Result<()> {
        if self.0.writing || !self.0.finished.load(Ordering::SeqCst) {
            self.0.cancelled.store(true, Ordering::SeqCst);
        }
        Ok(())
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
struct Job {
    server: Arc<Server>,
    spec: Sealed<ExecutionSpec>,
    text: String,
    copy: Option<crate::write::WorkCopy>,
    token: Arc<Token>,
    tx: mpsc::SyncSender<RuntimeEvent>,
}
struct Worker {
    tx: mpsc::Sender<Job>,
    join: JoinHandle<Result<()>>,
}

pub(crate) struct Pool {
    claude: PathBuf,
    idle: Duration,
    stopped: Arc<AtomicBool>,
    workers: Mutex<HashMap<String, Worker>>,
}
impl Pool {
    pub(crate) fn new(claude: PathBuf, idle: Duration) -> Self {
        Self {
            claude,
            idle,
            stopped: Arc::new(AtomicBool::new(false)),
            workers: Mutex::new(HashMap::new()),
        }
    }
    pub(crate) fn submit(
        &self,
        server: Arc<Server>,
        spec: &Sealed<ExecutionSpec>,
        bundle: &Sealed<Bundle>,
        exec: &Path,
        tenant: &Path,
        credential_root: &Path,
    ) -> Result<Running> {
        let copy = crate::write::prepare(
            &spec.document,
            &bundle.document,
            exec,
            tenant,
            credential_root,
        )?;
        let text = crate::write::task_text(&bundle.document, copy.as_ref())?;
        let key = selection_key(&spec.document, tenant)?;
        let token = Arc::new(Token {
            writing: copy.is_some(),
            cancelled: AtomicBool::new(false),
            finished: AtomicBool::new(false),
        });
        let (tx, rx) = mpsc::sync_channel(8);
        let job = Job {
            server: Arc::clone(&server),
            spec: spec.clone(),
            text,
            copy,
            token: Arc::clone(&token),
            tx,
        };
        let mut workers = self.workers.lock().expect("standby workers");
        if self.stopped.load(Ordering::SeqCst) {
            return Err(PortError::invalid("runtime is stopping"));
        }
        if !workers.contains_key(&key) {
            let (tx, receiver) = mpsc::channel();
            let claude = self.claude.clone();
            let stopped = Arc::clone(&self.stopped);
            let idle = self.idle;
            // Native session cwd is stable for the selection, not an old dispatch directory.
            let parent = exec
                .parent()
                .ok_or_else(|| PortError::invalid("execution parent missing"))?;
            let cwd = parent.join(format!("session-{key}"));
            crate::storage::private_dir(&cwd)?;
            let worker_key = key.clone();
            let join = std::thread::spawn(move || {
                worker(server, claude, cwd, worker_key, idle, stopped, receiver)
            });
            workers.insert(key.clone(), Worker { tx, join });
        }
        workers
            .get(&key)
            .expect("inserted worker")
            .tx
            .send(job)
            .map_err(|_| {
                PortError::new(
                    "STANDBY_UNAVAILABLE",
                    "selection worker stopped",
                    "retry_after_runtime_recovery",
                )
            })?;
        Ok(Running {
            session: Arc::new(Mutex::new(Box::new(Handle(token)))),
            events: rx,
        })
    }
    pub(crate) fn shutdown(&self) -> Result<()> {
        self.stopped.store(true, Ordering::SeqCst);
        let workers = std::mem::take(&mut *self.workers.lock().expect("standby workers"));
        let mut error = None;
        for (_, Worker { tx, join }) in workers {
            drop(tx);
            match join.join() {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    error.get_or_insert(e);
                }
                Err(_) => {
                    error.get_or_insert_with(|| PortError::invalid("standby worker panicked"));
                }
            }
        }
        error.map_or(Ok(()), Err)
    }
}
impl Drop for Pool {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}
pub(crate) fn readonly(spec: &ExecutionSpec) -> Result<()> {
    if spec.write_lease.is_some()
        || spec.review_publish_policy.is_some()
        || spec.permissions.iter().any(|p| p != "context.read")
    {
        return Err(PortError::new(
            "STANDBY_READONLY",
            "this runtime supports only read-only Bundle dispatch",
            "use_readonly_spec",
        ));
    }
    if !spec.permissions.iter().any(|p| p == "context.read") {
        return Err(PortError::new(
            "PERMISSION_DENIED",
            "this dispatch does not authorize Context delivery",
            "review_execution_spec",
        ));
    }
    Ok(())
}
pub(crate) fn selection_key(spec: &ExecutionSpec, tenant: &Path) -> Result<String> {
    Ok(hash(&canonical(&json!([
        tenant,
        spec.binding.id,
        spec.project.id,
        spec.selection.id
    ]))?))
}
fn worker(
    server: Arc<Server>,
    claude: PathBuf,
    cwd: PathBuf,
    key: String,
    idle: Duration,
    stopped: Arc<AtomicBool>,
    receiver: mpsc::Receiver<Job>,
) -> Result<()> {
    let dir = server.state.join(format!("standby-{key}"));
    crate::storage::private_dir(&dir)?;
    let mut native: Option<Native> = None;
    let mut last = Instant::now();
    let mut returned: Option<Job> = None;
    loop {
        let shutdown = stopped.load(Ordering::SeqCst);
        if returned
            .as_ref()
            .is_some_and(|job| job.token.cancelled.load(Ordering::SeqCst))
            || shutdown
        {
            if let Some(session) = native.as_mut() {
                session.close()?;
            }
            native = None;
            release_returned(&mut returned, true);
        }
        if shutdown {
            break;
        }
        let job = match receiver.recv_timeout(Duration::from_millis(25)) {
            Ok(job) => job,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if native.is_some() && last.elapsed() >= idle {
                    native.as_mut().expect("native session").close()?;
                    native = None;
                    release_returned(&mut returned, false);
                }
                continue;
            }
        };
        // A new frozen Spec must not enter a process retaining the old write lease.
        // The native history is resumed by ID/thread when run_job opens it again.
        if returned.is_some() {
            if let Some(session) = native.as_mut() {
                session.close()?;
            }
            native = None;
            release_returned(&mut returned, false);
        }
        let result = run_job(
            &job.server,
            &claude,
            &cwd,
            &dir,
            &stopped,
            &mut native,
            &job,
        );
        let sealed =
            result.is_ok() && job.copy.is_some() && job.token.finished.load(Ordering::SeqCst);
        job.token.finished.store(true, Ordering::SeqCst);
        if let Err(error) = result {
            let _ = job.tx.send(RuntimeEvent::Observation {
                kind: "harness_failure".into(),
                payload: json!({"code":error.code,"message":error.message}),
                source: EvidenceLevel::AdapterEvent,
            });
            let _ = job.tx.send(RuntimeEvent::ProtocolError(error.code));
            // No next turn may enter an uncertain old turn.
            if let Some(session) = native.as_mut() {
                session.close()?;
                if job.copy.is_some() {
                    let _ = job.tx.send(RuntimeEvent::TurnStopped {
                        requested_stop: job.token.cancelled.load(Ordering::SeqCst)
                            || stopped.load(Ordering::SeqCst),
                        session_closed: true,
                    });
                }
            }
            native = None;
        }
        if sealed {
            returned = Some(job);
        } else {
            if job.copy.is_some() {
                if let Some(session) = native.as_mut() {
                    session.close()?;
                    let _ = job.tx.send(RuntimeEvent::TurnStopped {
                        requested_stop: job.token.cancelled.load(Ordering::SeqCst)
                            || stopped.load(Ordering::SeqCst),
                        session_closed: true,
                    });
                }
                native = None;
            }
            let _ = job.tx.send(RuntimeEvent::DispatchReleased);
        }
        last = Instant::now();
    }
    if let Some(session) = native.as_mut() {
        session.close()?;
    }
    release_returned(&mut returned, true);
    for job in receiver.try_iter() {
        job.token.finished.store(true, Ordering::SeqCst);
        let _ = job.tx.send(RuntimeEvent::TurnStopped {
            requested_stop: true,
            session_closed: false,
        });
        let _ = job.tx.send(RuntimeEvent::DispatchReleased);
    }
    Ok(())
}
// DispatchReleased removes the stop handle from the tenant. A sealed writing
// turn retains that handle until its native session is physically closed.
fn release_returned(returned: &mut Option<Job>, requested_stop: bool) {
    if let Some(job) = returned.take() {
        let _ = job.tx.send(RuntimeEvent::TurnStopped {
            requested_stop,
            session_closed: true,
        });
        let _ = job.tx.send(RuntimeEvent::DispatchReleased);
    }
}

fn run_job(
    server: &Arc<Server>,
    claude: &Path,
    cwd: &Path,
    dir: &Path,
    stopped: &AtomicBool,
    native: &mut Option<Native>,
    job: &Job,
) -> Result<()> {
    if job.token.cancelled.load(Ordering::SeqCst) || stopped.load(Ordering::SeqCst) {
        let _ = job.tx.send(RuntimeEvent::TurnStopped {
            requested_stop: true,
            session_closed: false,
        });
        return Ok(());
    }
    if crate::storage::now_ms() >= job.spec.document.deadline_ms {
        let _ = job.tx.send(RuntimeEvent::DeadlineReached);
        let _ = job.tx.send(RuntimeEvent::TurnStopped {
            requested_stop: false,
            session_closed: false,
        });
        return Ok(());
    }
    let cwd = job.copy.as_ref().map_or(cwd, |copy| copy.cwd.as_path());
    let writing = job.copy.is_some();
    if let Some(session) = native.as_mut()
        && (session.cwd != cwd || session.writing != writing)
    {
        session.close()?;
        *native = None;
    }
    let marker = format!("HCTL2_DISPATCH_{}", job.spec.digest);
    write_json(
        &dir.join("job.json"),
        &json!({"id":job.spec.document.idempotency_key,"digest":job.spec.digest,
        "permissions":job.spec.document.permissions,"text":job.text,"marker":marker}),
    )?;
    for name in ["started.json", "returned.json", "rejected.json"] {
        match fs::remove_file(dir.join(name)) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    let mut recovered = false;
    loop {
        if native.is_none() {
            let (session, resumed, resume_failed) =
                Native::open(server, claude, cwd, dir, writing)?;
            let _ = job.tx.send(RuntimeEvent::Observation {
                kind: "session_opened".into(),
                payload: json!({"resumed":resumed,"resume_failed":resume_failed}),
                source: EvidenceLevel::AdapterEvent,
            });
            *native = Some(session);
        }
        let requested =
            job.token.cancelled.load(Ordering::SeqCst) || stopped.load(Ordering::SeqCst);
        let expired = crate::storage::now_ms() >= job.spec.document.deadline_ms;
        if requested || expired {
            if expired {
                let _ = job.tx.send(RuntimeEvent::DeadlineReached);
            }
            let _ = job.tx.send(RuntimeEvent::TurnStopped {
                requested_stop: requested,
                session_closed: false,
            });
            return Ok(());
        }
        let session = native.as_mut().expect("native session");
        match session
            .client
            .call("agent.prompt", json!({"target":session.pane,"text":marker}))
        {
            Ok(_) => break,
            Err(error) if !recovered && rejected_before_delivery(&error) => {
                // Only Herdr's explicit pre-queue rejection permits a retry.
                // A transport error might have delivered text and is not replayed.
                session.close()?;
                *native = None;
                recovered = true;
            }
            Err(error) => return Err(error),
        }
    }
    let session = native.as_mut().expect("native session");
    let mut turn = None;
    let mut interrupt: Option<(Instant, bool)> = None;
    loop {
        let requested =
            job.token.cancelled.load(Ordering::SeqCst) || stopped.load(Ordering::SeqCst);
        let expired = crate::storage::now_ms() >= job.spec.document.deadline_ms;
        if interrupt.is_none() && (requested || expired) {
            if expired {
                let _ = job.tx.send(RuntimeEvent::DeadlineReached);
            }
            // A key acknowledgement is not evidence of a returned turn.
            if session
                .client
                .call(
                    "agent.send_keys",
                    json!({"target":session.pane,"keys":["esc"]}),
                )
                .is_err()
            {
                session.close()?;
                let _ = job.tx.send(RuntimeEvent::TurnStopped {
                    requested_stop: requested,
                    session_closed: true,
                });
                *native = None;
                return Ok(());
            }
            interrupt = Some((Instant::now(), requested));
        }
        if let Some(started) = read_json(&dir.join("started.json"))? {
            check_job(&started, job, &session.id)?;
            if started["text"].as_str() != Some(job.text.as_str()) {
                return Err(PortError::new(
                    "STANDBY_PROMPT_MISMATCH",
                    "native turn did not receive this dispatch's text",
                    "inspect_dispatch",
                ));
            }
            turn = started["turnId"].as_str().map(str::to_owned);
        }
        if let Some(rejected) = read_json(&dir.join("rejected.json"))? {
            check_job(&rejected, job, &session.id)?;
            return Err(PortError::new(
                "STANDBY_PROMPT_MISMATCH",
                "native composer rejected another dispatch's text",
                "inspect_dispatch",
            ));
        }
        if let Some(returned) = read_json(&dir.join("returned.json"))?
            && turn.is_some()
        {
            // A completion can land between the start-record read and this one.
            // Wait until the start record is visible before comparing turn ids.
            check_job(&returned, job, &session.id)?;
            if turn.as_deref() != returned["turnId"].as_str() {
                return Err(PortError::invalid(
                    "completion does not match the native turn",
                ));
            }
            if returned.get("agentId").is_some() {
                return Err(PortError::invalid(
                    "subagent completion cannot return this dispatch",
                ));
            }
            match returned["reason"].as_str() {
                // Preserve a real answer racing cancellation. Governance decides
                // whether that late Proposal is admissible, not this adapter.
                Some("answer" | "refusal") => {
                    let bytes = returned["answer"]
                        .as_str()
                        .ok_or_else(|| PortError::invalid("turn answer missing"))?
                        .as_bytes()
                        .to_vec();
                    let (schema, bytes) = if let Some(copy) = &job.copy {
                        let output = crate::write::seal_return(
                            copy,
                            &bytes,
                            job.spec.document.deadline_ms,
                            &job.tx,
                            || {
                                job.token.cancelled.load(Ordering::SeqCst)
                                    || stopped.load(Ordering::SeqCst)
                            },
                        )?;
                        let Some(output) = output else {
                            session.close()?;
                            *native = None;
                            let _ = job.tx.send(RuntimeEvent::TurnStopped {
                                requested_stop: job.token.cancelled.load(Ordering::SeqCst)
                                    || stopped.load(Ordering::SeqCst),
                                session_closed: true,
                            });
                            return Ok(());
                        };
                        output
                    } else {
                        ("claude.turn.v1".into(), bytes)
                    };
                    job.token.finished.store(true, Ordering::SeqCst);
                    let _ = job.tx.send(RuntimeEvent::Proposal {
                        schema,
                        bytes,
                        source: EvidenceLevel::AdapterEvent,
                    });
                    let _ = job.tx.send(RuntimeEvent::TurnReturned);
                    return Ok(());
                }
                Some("aborted") => {
                    let requested_stop = interrupt.is_none_or(|(_, requested)| requested);
                    let _ = job.tx.send(RuntimeEvent::TurnStopped {
                        requested_stop,
                        session_closed: false,
                    });
                    return Ok(());
                }
                Some("error") => {
                    return Err(PortError::new(
                        "HARNESS_TURN_ERROR",
                        returned["answer"]
                            .as_str()
                            .filter(|s| !s.is_empty())
                            .unwrap_or("Claude ended this turn with an API error"),
                        "inspect_dispatch",
                    ));
                }
                _ => return Err(PortError::invalid("unknown native turn completion reason")),
            }
        }
        if interrupt.is_some_and(|(sent, _)| sent.elapsed() >= Duration::from_secs(3)) {
            let requested_stop = interrupt.expect("interrupt").1;
            session.close()?;
            let _ = job.tx.send(RuntimeEvent::TurnStopped {
                requested_stop,
                session_closed: true,
            });
            *native = None;
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}
fn rejected_before_delivery(error: &PortError) -> bool {
    error.code == "HERDR_REJECTED"
        && serde_json::from_str::<Value>(&error.message).is_ok_and(|v| {
            matches!(
                v["code"].as_str(),
                Some("agent_not_found" | "agent_not_ready" | "agent_pane_busy")
            )
        })
}

fn check_job(value: &Value, job: &Job, session: &str) -> Result<()> {
    if value["job"] != job.spec.document.idempotency_key
        || value["digest"] != job.spec.digest
        || value["session"] != session
    {
        return Err(PortError::invalid(
            "native event belongs to another dispatch or session",
        ));
    }
    Ok(())
}
fn write_json(path: &Path, value: &Value) -> Result<()> {
    let temp = path.with_extension("pending");
    let mut file = crate::storage::private_file(&temp)?;
    file.set_len(0)?;
    file.write_all(&canonical(value)?)?;
    file.sync_all()?;
    fs::rename(&temp, path)?;
    fs::File::open(
        path.parent()
            .ok_or_else(|| PortError::invalid("state parent missing"))?,
    )?
    .sync_all()?;
    Ok(())
}
fn read_json(path: &Path) -> Result<Option<Value>> {
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > LIMIT {
        return Err(PortError::invalid("native turn exceeds output limit"));
    }
    // Native $.fs.write is not atomic. Wait for a full JSON value, never consume a prefix.
    match serde_json::from_slice(&bytes) {
        Ok(v) => Ok(Some(v)),
        Err(e) if e.is_eof() => Ok(None),
        Err(e) => Err(e.into()),
    }
}

struct Native {
    client: Client,
    pane: String,
    id: String,
    closed: bool,
    cwd: PathBuf,
    writing: bool,
}
impl Native {
    fn open(
        server: &Arc<Server>,
        claude: &Path,
        cwd: &Path,
        dir: &Path,
        writing: bool,
    ) -> Result<(Self, bool, bool)> {
        prepare_plugin(dir)?;
        let prior = read_json(&dir.join("resume.json"))?
            .and_then(|v| v["session"].as_str().map(str::to_owned));
        let resume_failed = prior.is_some();
        if let Some(id) = prior {
            match Self::boot(server, claude, cwd, dir, Some(&id), writing) {
                Ok(session) if session.id == id => return Ok((session, true, false)),
                Ok(mut session) => {
                    session.close()?;
                }
                Err(error) if error.code == "STANDBY_START_TIMEOUT" => {}
                Err(error) => return Err(error),
            }
        }
        Self::boot(server, claude, cwd, dir, None, writing).map(|s| (s, false, resume_failed))
    }
    fn boot(
        server: &Arc<Server>,
        claude: &Path,
        cwd: &Path,
        dir: &Path,
        resume: Option<&str>,
        writing: bool,
    ) -> Result<Self> {
        let integration = server.state.join("claude-integration");
        let hook = integration.join("hooks/herdr-agent-state.sh");
        if !hook.exists() {
            return Err(PortError::invalid(
                "private Herdr Claude integration missing",
            ));
        }
        let command = format!("bash {} session", crate::launch::sh_quote(&hook));
        let mut settings = json!({"hooks":{"SessionStart":[{"matcher":"^(startup|resume|clear|compact|fork)$","hooks":[{"type":"command","command":command,"timeout":10}]}]}});
        if writing {
            settings["sandbox"] = json!({"enabled":false,"autoAllowBashIfSandboxed":true,"allowUnsandboxedCommands":false,
                "filesystem":{"denyRead":crate::confine::sensitive_paths()},"network":{"allowedDomains":[]}});
            settings["permissions"] =
                json!({"deny":["Bash(git push *)","Bash(gh *)","Bash(git credential *)"]});
        }
        write_json(&dir.join("settings.json"), &settings)?;
        let bin = dir.join("bin");
        crate::storage::private_dir(&bin)?;
        let executable = bin.join("claude");
        match fs::read_link(&executable) {
            Ok(existing) if existing == claude => {}
            Ok(_) => return Err(PortError::invalid("private Claude link changed")),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                std::os::unix::fs::symlink(claude, &executable)?
            }
            Err(e) => return Err(e.into()),
        }
        match fs::remove_file(dir.join("session.json")) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        let client = Client::connect(&server.socket)?;
        let created=client.call("workspace.create",json!({"cwd":cwd,"focus":false,"label":"agency-standby","env":{
            "HOME":std::env::var("HOME").unwrap_or_default(),"USER":std::env::var("USER").unwrap_or_default(),"PATH":format!("{}:{}", bin.display(), std::env::var("PATH").unwrap_or_else(|_|"/usr/bin:/bin".into()))}}))?;
        let pane = created
            .pointer("/root_pane/pane_id")
            .and_then(Value::as_str)
            .ok_or_else(|| PortError::invalid("workspace pane missing"))?
            .to_owned();
        let mut native = Self {
            client,
            pane,
            id: String::new(),
            closed: false,
            cwd: cwd.to_path_buf(),
            writing,
        };
        let result = (|| {
            // A fixed non-login shell keeps the native agent executable pinned;
            // user shell startup files must not replace this session-local PATH.
            let tab = created
                .pointer("/tab/tab_id")
                .and_then(Value::as_str)
                .ok_or_else(|| PortError::invalid("workspace tab missing"))?;
            let mut shell = vec![
                "/usr/bin/env".into(),
                format!(
                    "PATH={}:{}",
                    bin.display(),
                    std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".into())
                ),
                format!("HOME={}", std::env::var("HOME").unwrap_or_default()),
                format!("USER={}", std::env::var("USER").unwrap_or_default()),
            ];
            if writing {
                let temp = dir.join("tmp");
                crate::storage::private_dir(&temp)?;
                shell.extend([
                    "GIT_CONFIG_GLOBAL=/dev/null".into(),
                    "GIT_CONFIG_NOSYSTEM=1".into(),
                    "GIT_TERMINAL_PROMPT=0".into(),
                    format!("TMPDIR={}", temp.display()),
                    format!("CLAUDE_CODE_TMPDIR={}", temp.display()),
                ]);
            }
            shell.push("/bin/sh".into());
            let applied = native.client.call(
                "layout.apply",
                json!({"tab_id":tab,"focus":false,
                "root":{"type":"pane","cwd":cwd,"command":shell}}),
            )?;
            native.pane = applied
                .pointer("/layout/root/pane_id")
                .and_then(Value::as_str)
                .ok_or_else(|| PortError::invalid("layout pane missing"))?
                .into();
            let mut argv = vec![
                "--restricted".into(),
                "--settings".into(),
                dir.join("settings.json").display().to_string(),
                "--strict-mcp-config".into(),
                "--tools".into(),
                if writing {
                    "Read,Edit,Write,Bash,Glob,Grep".into()
                } else {
                    String::new()
                },
                "--plugin-dir".into(),
                dir.join("plugin").display().to_string(),
            ];
            if writing {
                argv.extend([
                    "--permission-mode".into(),
                    "acceptEdits".into(),
                    "--allowedTools".into(),
                    "Bash(*)".into(),
                    "--permission-prompts".into(),
                    "none".into(),
                ]);
            }
            if let Some(id) = resume {
                argv.extend(["--resume".into(), id.into()]);
            }
            native.client.call_retrying_busy(
                "agent.start",
                json!({"name":format!("agency-{}", &hash(dir.as_os_str().as_encoded_bytes())[..24]),
                "kind":"claude","pane_id":native.pane,"args":argv,"timeout_ms":30_000}),
            )?;
            let timeout = Instant::now() + Duration::from_secs(30);
            let mut trusted = false;
            let mut trust_key_at = Instant::now();
            while Instant::now() < timeout {
                if let Some(value) = read_json(&dir.join("session.json"))? {
                    let id = value["session"]
                        .as_str()
                        .filter(|s| !s.is_empty())
                        .ok_or_else(|| PortError::invalid("native session identity missing"))?;
                    if native
                        .client
                        .call("agent.get", json!({"target":native.pane}))
                        .is_ok_and(|v| {
                            v["agent"]["agent"] == "claude"
                                && v["agent"]["interactive_ready"] == true
                        })
                    {
                        native.id = id.into();
                        write_json(&dir.join("resume.json"), &json!({"session":id}))?;
                        return Ok(());
                    }
                }
                if !trusted {
                    let screen=native.client.call("pane.read",json!({"pane_id":native.pane,"source":"visible","format":"text","strip_ansi":true}))?;
                    let text = screen
                        .pointer("/read/text")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let normalized: String = text.chars().filter(|c| !c.is_whitespace()).collect();
                    let path: String = cwd
                        .display()
                        .to_string()
                        .chars()
                        .filter(|c| !c.is_whitespace())
                        .collect();
                    if normalized.contains(&path)
                        && text.contains("Yes, I trust this folder")
                        && text.contains("Enter to confirm")
                        && trust_key_at.elapsed() >= Duration::from_millis(200)
                    {
                        // Owner authorized only Agency-prepared execution directories.
                        // Observe the selected option before confirming it. The first
                        // render can precede the TUI installing its input handler.
                        let confirm = text.contains("❯ Yes, I trust this folder");
                        let select = text.contains("❯ No, exit");
                        if confirm || select {
                            native.client.call(
                                "pane.send_keys",
                                json!({"pane_id":native.pane,"keys":[if confirm {"enter"} else {"down"}]}),
                            )?;
                            trusted = confirm;
                            trust_key_at = Instant::now();
                        }
                    }
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(PortError::new(
                "STANDBY_START_TIMEOUT",
                "native session did not become ready",
                "inspect_herdr_pane",
            ))
        })();
        if let Err(error) = result {
            if let Ok(screen) = native.client.call(
                "pane.read",
                json!({"pane_id":native.pane,"source":"visible","format":"text","strip_ansi":true}),
            ) {
                let _ = write_json(&dir.join("boot-failure.json"), &screen);
            }
            native.close()?;
            return Err(error);
        }
        Ok(native)
    }
    fn close(&mut self) -> Result<()> {
        if !self.closed {
            self.client
                .call("pane.close", json!({"pane_id":self.pane}))?;
            self.closed = true;
        }
        Ok(())
    }
}
impl Drop for Native {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
fn prepare_plugin(dir: &Path) -> Result<()> {
    let plugin = dir.join("plugin");
    crate::storage::private_dir(&plugin.join(".claude-plugin"))?;
    crate::storage::private_dir(&plugin.join("hooks"))?;
    write_json(
        &plugin.join(".claude-plugin/plugin.json"),
        &json!({"name":"hctl2-turn","version":"0.1.0","description":"Return the native Claude turn to its Agency dispatch"}),
    )?;
    write_json(
        &plugin.join("hooks/hooks.json"),
        &json!({"modules":["./register.js"]}),
    )?;
    fs::write(
        plugin.join("hooks/register.js"),
        include_str!("harness/turn.js").replace("__HCTL_ROOT__", &serde_json::to_string(dir)?),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_explicit_pre_delivery_rejections_allow_recovery() {
        for code in ["agent_not_found", "agent_not_ready", "agent_pane_busy"] {
            let error = PortError::new(
                "HERDR_REJECTED",
                json!({"code":code}).to_string(),
                "inspect_dispatch",
            );
            assert!(rejected_before_delivery(&error));
        }
        for (code, message) in [
            ("HERDR_REJECTED", r#"{"code":"agent_prompt_failed"}"#),
            ("HERDR_REJECTED", r#"{"code":"agent_blocked"}"#),
            ("HERDR_REJECTED", "agent_not_found"),
            ("HERDR_REJECTED", "{}"),
            ("HERDR_IO", r#"{"code":"agent_not_found"}"#),
        ] {
            assert!(!rejected_before_delivery(&PortError::new(
                code,
                message,
                "inspect_dispatch"
            )));
        }
    }
}
