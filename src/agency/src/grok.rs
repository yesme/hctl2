//! Grok Build over ACP (`grok agent stdio`). There is no interactive pane.
//! One process stays up for a selection and takes further `session/prompt`s.
//! Idle close keeps the session id and the next start uses `session/load`.
//! This client does not pass `--always-approve` or a bypass permission mode.
//! Login stays in `~/.grok`. This module does not copy it or edit that directory.

use agency_proto::{EvidenceLevel, ExecutionSpec, PortError, Result, Sealed, context::Bundle};
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use crate::runtime::{Running, RuntimeEvent, Session};

const PROFESSION: &str = "grok-build";

pub(crate) fn installed() -> Option<(PathBuf, String)> {
    let path = PathBuf::from(std::env::var_os("HCTL2_GROK")?);
    let path = path.canonicalize().ok()?;
    let digest = crate::catalog::file_digest(&path).ok()?;
    Some((path, digest))
}

pub(crate) fn argv(leader_socket: &Path) -> Vec<String> {
    vec![
        "agent".into(),
        "--no-leader".into(),
        "--leader-socket".into(),
        leader_socket.display().to_string(),
        "stdio".into(),
    ]
}

pub(crate) struct Pool {
    bin: PathBuf,
    idle: Duration,
    stopped: Arc<AtomicBool>,
    workers: Mutex<std::collections::HashMap<String, Worker>>,
}

struct Worker {
    tx: mpsc::Sender<Job>,
    join: JoinHandle<()>,
}

struct Job {
    text: String,
    budget: Duration,
    deadline_ms: u64,
    credential_root: PathBuf,
    state: PathBuf,
    cwd: PathBuf,
    copy: Option<crate::write::WorkCopy>,
    observation: crate::observation::ObservationConfig,
    multiplexer: Option<crate::observation::Multiplexer>,
    token: Arc<Token>,
    tx: mpsc::SyncSender<RuntimeEvent>,
}

struct Token {
    cancelled: AtomicBool,
}

struct Handle(Arc<Token>);

impl Session for Handle {
    fn input(&mut self, _: &[u8]) -> Result<()> {
        Err(PortError::invalid("grok stdio has no pane input"))
    }
    fn stop(&mut self) -> Result<()> {
        self.0.cancelled.store(true, Ordering::SeqCst);
        Ok(())
    }
}

impl Pool {
    pub(crate) fn new(bin: PathBuf, idle: Duration) -> Self {
        Self {
            bin,
            idle,
            stopped: Arc::new(AtomicBool::new(false)),
            workers: Mutex::new(std::collections::HashMap::new()),
        }
    }

    pub(crate) fn submit(
        &self,
        spec: &Sealed<ExecutionSpec>,
        bundle: &Sealed<Bundle>,
        exec_root: &Path,
        tenant: &Path,
        credential_root: &Path,
        view: View,
    ) -> Result<Running> {
        crate::confine::reject_unenforceable_isolation(
            &spec.document.required_capabilities.isolation_effects,
        )?;
        if spec.document.profession.reference.id != PROFESSION {
            return Err(PortError::invalid("not a grok-build dispatch"));
        }
        let copy = crate::write::prepare(
            &spec.document,
            &bundle.document,
            exec_root,
            tenant,
            credential_root,
        )?;
        let text = crate::write::task_text(&bundle.document, copy.as_ref())?;
        let key = crate::standby::selection_key(&spec.document, tenant)?;
        let parent = exec_root
            .parent()
            .ok_or_else(|| PortError::invalid("execution parent missing"))?;
        let state = parent.join(format!("grok-session-{key}"));
        crate::storage::private_dir(&state)?;
        let cwd = copy
            .as_ref()
            .map(|item| item.cwd.clone())
            .unwrap_or_else(|| state.clone());
        let token = Arc::new(Token {
            cancelled: AtomicBool::new(false),
        });
        let (tx, rx) = mpsc::sync_channel(8);
        let job = Job {
            text,
            budget: budget(spec.document.deadline_ms),
            deadline_ms: spec.document.deadline_ms,
            credential_root: credential_root.to_path_buf(),
            state,
            cwd,
            copy,
            observation: view.observation,
            multiplexer: view.multiplexer,
            token: Arc::clone(&token),
            tx,
        };
        let mut workers = self.workers.lock().expect("grok workers");
        if self.stopped.load(Ordering::SeqCst) {
            return Err(PortError::invalid("runtime is stopping"));
        }
        if !workers.contains_key(&key) {
            let (tx, receiver) = mpsc::channel();
            let bin = self.bin.clone();
            let idle = self.idle;
            let stopped = Arc::clone(&self.stopped);
            let join = std::thread::spawn(move || worker(bin, idle, stopped, receiver));
            workers.insert(key.clone(), Worker { tx, join });
        }
        workers
            .get(&key)
            .expect("inserted grok worker")
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
        let workers = std::mem::take(&mut *self.workers.lock().expect("grok workers"));
        for (_, Worker { tx, join }) in workers {
            drop(tx);
            let _ = join.join();
        }
        Ok(())
    }
}

fn budget(deadline_ms: u64) -> Duration {
    Duration::from_millis(deadline_ms.saturating_sub(crate::storage::now_ms()))
}

fn worker(bin: PathBuf, idle: Duration, stopped: Arc<AtomicBool>, receiver: mpsc::Receiver<Job>) {
    let mut live: Option<Link<std::process::ChildStdin>> = None;
    let mut watch: Option<Watch> = None;
    let mut last = Instant::now();
    loop {
        if stopped.load(Ordering::SeqCst) {
            break;
        }
        let job = match receiver.recv_timeout(Duration::from_millis(25)) {
            Ok(job) => job,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if live.is_some() && last.elapsed() >= idle {
                    if let Some(link) = live.as_mut() {
                        link.close();
                    }
                    live = None;
                }
                continue;
            }
        };
        if job.token.cancelled.load(Ordering::SeqCst) {
            let _ = job.tx.send(RuntimeEvent::TurnStopped {
                requested_stop: true,
                session_closed: false,
            });
            let _ = job.tx.send(RuntimeEvent::DispatchReleased);
            continue;
        }
        match run_job(&bin, &mut live, &job) {
            Ok(turn) if turn.stop_reason == "cancelled" => {
                let _ = job.tx.send(RuntimeEvent::Observation {
                    kind: "grok.stop".into(),
                    payload: json!({
                        "stop_reason": turn.stop_reason,
                        "session": turn.opened,
                    }),
                    source: EvidenceLevel::AdapterEvent,
                });
                let _ = job.tx.send(RuntimeEvent::TurnStopped {
                    requested_stop: job.token.cancelled.load(Ordering::SeqCst),
                    session_closed: false,
                });
                let _ = job.tx.send(RuntimeEvent::DispatchReleased);
            }
            Ok(turn) => {
                let _ = job.tx.send(RuntimeEvent::Observation {
                    kind: "grok.stop".into(),
                    payload: json!({
                        "stop_reason": turn.stop_reason,
                        "session": turn.opened,
                    }),
                    source: EvidenceLevel::AdapterEvent,
                });
                let _ = watch_turn(
                    &mut watch,
                    &job.state,
                    &turn,
                    &job.text,
                    job.observation,
                    job.multiplexer.as_ref(),
                );
                if let Some(copy) = job.copy.clone() {
                    match crate::write::seal_return(
                        &copy,
                        turn.text.as_bytes(),
                        job.deadline_ms,
                        &job.tx,
                        || {
                            stopped.load(Ordering::SeqCst)
                                || job.token.cancelled.load(Ordering::SeqCst)
                        },
                    ) {
                        Ok(Some((schema, bytes))) => {
                            let _ = job.tx.send(RuntimeEvent::Proposal {
                                schema,
                                bytes,
                                source: EvidenceLevel::AdapterEvent,
                            });
                            let _ = job.tx.send(RuntimeEvent::TurnReturned);
                        }
                        Ok(None) => {
                            let _ = job.tx.send(RuntimeEvent::TurnStopped {
                                requested_stop: job.token.cancelled.load(Ordering::SeqCst),
                                session_closed: false,
                            });
                        }
                        Err(error) => {
                            let _ = job.tx.send(RuntimeEvent::ProtocolError(error.message));
                        }
                    }
                    if let Some(link) = live.as_mut() {
                        link.close();
                    }
                    live = None;
                } else {
                    let _ = job.tx.send(RuntimeEvent::Proposal {
                        schema: crate::observation::TURN_OUTPUT_SCHEMA.into(),
                        bytes: turn.text.into_bytes(),
                        source: EvidenceLevel::AdapterEvent,
                    });
                    let _ = job.tx.send(RuntimeEvent::TurnReturned);
                }
                let _ = job.tx.send(RuntimeEvent::DispatchReleased);
            }
            Err(error) => {
                if let Some(link) = live.as_mut() {
                    link.close();
                }
                live = None;
                let _ = job.tx.send(RuntimeEvent::ProtocolError(error.message));
                let _ = job.tx.send(RuntimeEvent::DispatchReleased);
            }
        }
        last = Instant::now();
    }
    if let Some(link) = live.as_mut() {
        link.close();
    }
}

fn run_job(
    bin: &Path,
    live: &mut Option<Link<std::process::ChildStdin>>,
    job: &Job,
) -> Result<Turn> {
    let limit = deadline(job.budget);
    if live.as_ref().is_some_and(|link| link.cwd != job.cwd)
        && let Some(mut previous) = live.take()
    {
        previous.close();
    }
    if live.is_none() {
        *live = Some(Link::spawn(
            bin,
            &job.state,
            &job.credential_root,
            &job.cwd,
            limit,
        )?);
    }
    let link = live.as_mut().expect("grok link");
    let saved = fs::read_to_string(job.state.join("session-id")).ok();
    let saved = saved.as_deref().map(str::trim).filter(|id| !id.is_empty());
    if link.dialogue.session_id.is_none() {
        let opened = link.dialogue.ensure(saved, limit)?;
        if let Some(id) = &link.dialogue.session_id {
            fs::write(job.state.join("session-id"), id)?;
        }
        link.opened = opened;
    }
    link.dialogue
        .prompt(&job.text, limit, &job.token.cancelled)
        .map(|mut turn| {
            turn.opened = link.opened.clone();
            turn.session_id = link.dialogue.session_id.clone().unwrap_or_default();
            turn
        })
}

pub(crate) struct View {
    pub observation: crate::observation::ObservationConfig,
    pub multiplexer: Option<crate::observation::Multiplexer>,
}

struct Watch {
    session: String,
    pane: crate::observation::ObservationPane,
    pane_id: Option<String>,
}

fn watch_turn(
    watch: &mut Option<Watch>,
    state: &Path,
    turn: &Turn,
    prompt: &str,
    config: crate::observation::ObservationConfig,
    multiplexer: Option<&crate::observation::Multiplexer>,
) -> Result<()> {
    if turn.session_id.is_empty() {
        return Ok(());
    }
    let dir = state.join("grok").join(&turn.session_id);
    if watch
        .as_ref()
        .is_none_or(|item| item.session != turn.session_id)
    {
        *watch = Some(open_watch(&dir, &turn.session_id, config, multiplexer)?);
    }
    let recorded = publish_layout(
        state,
        turn,
        prompt,
        watch.as_ref().and_then(|item| item.pane_id.as_deref()),
    );
    if let Some(item) = watch.as_ref() {
        let _ = item.pane.status();
    }
    recorded
}

fn open_watch(
    dir: &Path,
    session: &str,
    config: crate::observation::ObservationConfig,
    multiplexer: Option<&crate::observation::Multiplexer>,
) -> Result<Watch> {
    fs::create_dir_all(dir)?;
    let opened = crate::observation::ObservationPane::open(config, dir, "grok", || {
        multiplexer
            .cloned()
            .ok_or_else(|| PortError::invalid("observation multiplexer missing"))
    });
    let (pane, pane_id) = match opened {
        Ok(pair) => pair,
        Err(_) => crate::observation::ObservationPane::open(
            crate::observation::ObservationConfig::disabled(),
            dir,
            "grok",
            || Err(PortError::invalid("panes are off")),
        )?,
    };
    Ok(Watch {
        session: session.to_owned(),
        pane,
        pane_id,
    })
}

/// Allow `~/.grok` and the selection state directory. A write turn also allows
/// its worktree. The person's home, `gh`, ssh, and keychains are not listed.
pub(crate) fn confine_allow(grok_home: &Path, state: &Path, cwd: &Path) -> String {
    let mut lines = vec![grok_home.display().to_string(), state.display().to_string()];
    if cwd != state {
        lines.push(cwd.display().to_string());
    }
    lines.join("\n")
}

fn publish_layout(state: &Path, turn: &Turn, prompt: &str, pane_id: Option<&str>) -> Result<()> {
    if turn.session_id.is_empty() {
        return Ok(());
    }
    let dir = state.join("grok").join(&turn.session_id);
    fs::create_dir_all(&dir)?;
    fs::write(
        dir.join("state.json"),
        serde_json::to_vec(&json!({
            "session_id": turn.session_id,
            "opened": turn.opened,
            "pane_id": pane_id,
        }))?,
    )?;
    let mut events = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("events.jsonl"))?;
    let event = json!({
        "method": "session/prompt",
        "text": prompt,
        "stop_reason": turn.stop_reason,
    });
    writeln!(events, "{event}")?;
    let summary: String = prompt.chars().take(80).collect();
    crate::observation::append_view_line(
        &dir,
        crate::observation::ViewEntry::TurnStart { turn: 1 },
    )?;
    crate::observation::append_view_line(
        &dir,
        crate::observation::ViewEntry::PromptSummary { summary: &summary },
    )?;
    crate::observation::append_view_line(
        &dir,
        crate::observation::ViewEntry::Answer { text: &turn.text },
    )?;
    crate::observation::append_view_line(
        &dir,
        crate::observation::ViewEntry::TurnEnd {
            turn: 1,
            status: &turn.stop_reason,
        },
    )?;
    Ok(())
}

fn deadline(budget: Duration) -> Instant {
    Instant::now() + budget
}

/// How long to wait for `session/cancel` to finish the turn before the process is closed.
const CANCEL_GRACE: Duration = Duration::from_millis(400);

struct Link<W: Write> {
    child: Child,
    dialogue: Dialogue<W>,
    opened: String,
    cwd: PathBuf,
    closed: bool,
}

impl Link<std::process::ChildStdin> {
    fn spawn(
        bin: &Path,
        state: &Path,
        credential_root: &Path,
        cwd: &Path,
        deadline: Instant,
    ) -> Result<Self> {
        let socket = state.join("leader.sock");
        reap_previous(state, &socket);
        let args = argv(&socket);
        if args
            .iter()
            .any(|arg| arg == "--always-approve" || arg.contains("bypassPermissions"))
        {
            return Err(PortError::invalid("grok client must not auto-approve"));
        }
        let mut command = crate::confine::command(bin, &args, state, credential_root)?;
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| PortError::invalid("HOME missing"))?;
        let grok_home = home.join(".grok");
        let credential = credential_root.canonicalize().map_err(|_| {
            PortError::new(
                "CREDENTIAL_ROOT_UNRESOLVED",
                "credential root cannot be canonicalized",
                "choose_credential_root",
            )
        })?;
        for path in [&grok_home, state, cwd] {
            if crate::confine::allowed_tree_contains_credential(path, &credential) {
                return Err(PortError::new(
                    "CREDENTIAL_ROOT_COVERED",
                    "grok allowance overlaps credentials",
                    "move_credential_root",
                ));
            }
        }
        let temp = state.join("tmp");
        crate::storage::private_dir(&temp)?;
        let parent = bin
            .parent()
            .ok_or_else(|| PortError::invalid("grok has no parent"))?;
        command
            .env_clear()
            .env("HOME", &home)
            .env("PATH", "/usr/bin:/bin")
            .env("LANG", "C.UTF-8")
            .env("TMPDIR", &temp)
            .env("HCTL2_CONFINE_ALLOW", confine_allow(&grok_home, state, cwd))
            .env("HCTL2_CONFINE_READ", parent)
            .current_dir(cwd)
            .process_group(0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut child = command
            .spawn()
            .map_err(|error| PortError::invalid(format!("spawn grok: {error}")))?;
        fs::write(state.join("pid"), child.id().to_string())?;
        let stdin = child.stdin.take().expect("stdin");
        let stdout = child.stdout.take().expect("stdout");
        let dialogue = Dialogue::new(BufReader::new(stdout), stdin);
        let mut link = Self {
            child,
            dialogue,
            opened: "new".into(),
            cwd: cwd.to_path_buf(),
            closed: false,
        };
        link.dialogue.handshake(deadline)?;
        Ok(link)
    }
}

impl<W: Write> Link<W> {
    fn close(&mut self) {
        if self.closed {
            return;
        }
        self.closed = true;
        let _ = self.dialogue.cancel();
        let pid = self.child.id();
        let _ = Command::new("kill")
            .args(["-TERM", &format!("-{pid}")])
            .status();
        let grace = Instant::now() + Duration::from_millis(400);
        while Instant::now() < grace {
            if self.child.try_wait().ok().flatten().is_some() {
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let _ = Command::new("kill")
            .args(["-KILL", &format!("-{pid}")])
            .status();
        let _ = self.child.wait();
    }
}

impl<W: Write> Drop for Link<W> {
    fn drop(&mut self) {
        self.close();
    }
}

fn reap_previous(state: &Path, socket: &Path) {
    let Ok(text) = fs::read_to_string(state.join("pid")) else {
        return;
    };
    let Ok(pid) = text.trim().parse::<u32>() else {
        return;
    };
    if pid == 0 || !same_leader(pid, socket) {
        return;
    }
    let _ = Command::new("kill")
        .args(["-TERM", &format!("-{pid}")])
        .status();
    std::thread::sleep(Duration::from_millis(200));
    if same_leader(pid, socket) {
        let _ = Command::new("kill")
            .args(["-KILL", &format!("-{pid}")])
            .status();
    }
}

fn same_leader(pid: u32, socket: &Path) -> bool {
    let needle = socket.display().to_string();
    if let Ok(bytes) = fs::read(format!("/proc/{pid}/cmdline")) {
        let text = String::from_utf8_lossy(&bytes);
        return text.contains(&needle);
    }
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "command="])
        .output();
    output
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).contains(&needle))
        .unwrap_or(false)
}

struct Dialogue<W> {
    lines: mpsc::Receiver<std::io::Result<String>>,
    writer: W,
    next_id: i64,
    session_id: Option<String>,
}

impl<W: Write> Dialogue<W> {
    fn new<R: BufRead + Send + 'static>(reader: R, writer: W) -> Self {
        let (tx, lines) = mpsc::channel();
        std::thread::spawn(move || {
            let mut reader = reader;
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        if tx.send(Ok(line)).is_err() {
                            break;
                        }
                    }
                    Err(error) => {
                        let _ = tx.send(Err(error));
                        break;
                    }
                }
            }
        });
        Self {
            lines,
            writer,
            next_id: 1,
            session_id: None,
        }
    }

    fn handshake(&mut self, deadline: Instant) -> Result<()> {
        let init = self.request(
            "initialize",
            json!({"protocolVersion": 1, "clientCapabilities": {}}),
            deadline,
            &mut String::new(),
            &AtomicBool::new(false),
        )?;
        let methods = init["authMethods"].as_array().cloned().unwrap_or_default();
        if methods.iter().any(|method| method["id"] == "cached_token") {
            self.request(
                "authenticate",
                json!({"methodId": "cached_token", "_meta": {"headless": true}}),
                deadline,
                &mut String::new(),
                &AtomicBool::new(false),
            )?;
        }
        Ok(())
    }

    /// `new`, `loaded`, or `load_failed_new` when the saved id no longer loads.
    fn ensure(&mut self, saved: Option<&str>, deadline: Instant) -> Result<String> {
        if let Some(saved) = saved {
            if self
                .request(
                    "session/load",
                    json!({"sessionId": saved, "cwd": ".", "mcpServers": []}),
                    deadline,
                    &mut String::new(),
                    &AtomicBool::new(false),
                )
                .is_ok()
            {
                self.session_id = Some(saved.to_owned());
                return Ok("loaded".into());
            }
            let created = self.request(
                "session/new",
                json!({"cwd": ".", "mcpServers": []}),
                deadline,
                &mut String::new(),
                &AtomicBool::new(false),
            )?;
            self.session_id = Some(session_id(&created)?);
            return Ok("load_failed_new".into());
        }
        let created = self.request(
            "session/new",
            json!({"cwd": ".", "mcpServers": []}),
            deadline,
            &mut String::new(),
            &AtomicBool::new(false),
        )?;
        self.session_id = Some(session_id(&created)?);
        Ok("new".into())
    }

    fn prompt(&mut self, text: &str, deadline: Instant, cancel: &AtomicBool) -> Result<Turn> {
        let session_id = self
            .session_id
            .clone()
            .ok_or_else(|| PortError::invalid("grok session is not open"))?;
        let mut answer = String::new();
        let result = self.request(
            "session/prompt",
            json!({"sessionId": session_id, "prompt": [{"type": "text", "text": text}]}),
            deadline,
            &mut answer,
            cancel,
        )?;
        if answer.is_empty() && result["stopReason"] != "cancelled" {
            return Err(PortError::invalid(
                "turn completed without an agent message",
            ));
        }
        Ok(Turn {
            text: answer,
            stop_reason: result["stopReason"]
                .as_str()
                .unwrap_or("unknown")
                .to_owned(),
            opened: String::new(),
            session_id: session_id.clone(),
        })
    }

    fn cancel(&mut self) -> Result<()> {
        let Some(session_id) = &self.session_id else {
            return Ok(());
        };
        let note =
            json!({"jsonrpc":"2.0","method":"session/cancel","params":{"sessionId": session_id}});
        writeln!(self.writer, "{note}")?;
        self.writer.flush()?;
        Ok(())
    }

    fn request(
        &mut self,
        method: &str,
        params: Value,
        deadline: Instant,
        answer: &mut String,
        cancel: &AtomicBool,
    ) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;
        let body = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        if Instant::now() >= deadline {
            return Err(Self::budget_error(method));
        }
        writeln!(self.writer, "{body}")?;
        self.writer.flush()?;
        let mut notified = false;
        loop {
            if Instant::now() >= deadline {
                if !notified {
                    let _ = self.cancel();
                }
                if let Some(result) = self.read_until(Instant::now() + CANCEL_GRACE, id, answer)? {
                    return Ok(result);
                }
                return Err(Self::budget_error(method));
            }
            if cancel.load(Ordering::SeqCst) && !notified {
                self.cancel()?;
                notified = true;
            }
            let line = match self.lines.recv_timeout(Duration::from_millis(20)) {
                Ok(line) => line?,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(PortError::invalid(format!(
                        "{method} ended before a response"
                    )));
                }
            };
            let message: Value = serde_json::from_str(line.trim())
                .map_err(|error| PortError::invalid(format!("grok acp is not json: {error}")))?;
            if message.get("method").is_some()
                && message.get("id").is_some()
                && message["id"] != json!(id)
            {
                return Err(PortError::new(
                    "HARNESS_TURN_ERROR",
                    format!(
                        "grok asked for {} and Agency will not answer it",
                        message["method"].as_str().unwrap_or("a request")
                    ),
                    "owner_must_confirm",
                ));
            }
            if message["method"] == "session/update" {
                let update = &message["params"]["update"];
                if update["sessionUpdate"] == "agent_message_chunk"
                    && let Some(text) = update["content"]["text"].as_str()
                {
                    answer.push_str(text);
                }
                continue;
            }
            if message.get("id") == Some(&json!(id)) {
                return Self::parse_result(method, &message);
            }
        }
    }

    /// After `session/cancel`, wait out the grace for this request's result.
    /// A line that belongs to another id is ignored. No result means the
    /// interrupt did not land, and the caller closes the process.
    fn read_until(
        &mut self,
        grace: Instant,
        id: i64,
        answer: &mut String,
    ) -> Result<Option<Value>> {
        while Instant::now() < grace {
            let line = match self.lines.recv_timeout(Duration::from_millis(20)) {
                Ok(line) => line?,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(None),
            };
            let message: Value = serde_json::from_str(line.trim())
                .map_err(|error| PortError::invalid(format!("grok acp is not json: {error}")))?;
            if message.get("method").is_some()
                && message.get("id").is_some()
                && message["id"] != json!(id)
            {
                return Err(PortError::new(
                    "HARNESS_TURN_ERROR",
                    format!(
                        "grok asked for {} and Agency will not answer it",
                        message["method"].as_str().unwrap_or("a request")
                    ),
                    "owner_must_confirm",
                ));
            }
            if message["method"] == "session/update" {
                let update = &message["params"]["update"];
                if update["sessionUpdate"] == "agent_message_chunk"
                    && let Some(text) = update["content"]["text"].as_str()
                {
                    answer.push_str(text);
                }
                continue;
            }
            if message.get("id") == Some(&json!(id)) {
                return Ok(Some(Self::parse_result("session/prompt", &message)?));
            }
        }
        Ok(None)
    }

    fn parse_result(method: &str, message: &Value) -> Result<Value> {
        if let Some(error) = message.get("error") {
            return Err(PortError::invalid(format!(
                "{method} error {}: {}",
                error["code"], error["message"]
            )));
        }
        Ok(message["result"].clone())
    }

    fn budget_error(method: &str) -> PortError {
        PortError::new(
            "HARNESS_TURN_ERROR",
            format!("{method} exceeded the turn budget"),
            "retry_dispatch",
        )
    }
}

fn session_id(created: &Value) -> Result<String> {
    created["sessionId"]
        .as_str()
        .filter(|id| !id.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| PortError::invalid("session/new returned no session id"))
}

#[derive(Debug)]
struct Turn {
    text: String,
    stop_reason: String,
    opened: String,
    session_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, Write};
    use std::os::unix::process::CommandExt;

    #[test]
    fn layout_uses_the_shared_turn_schema_and_three_files() {
        assert_eq!(crate::observation::TURN_OUTPUT_SCHEMA, "agency.turn.v1");
        let root = std::env::temp_dir().join(format!("hctl2-grok-layout-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let turn = Turn {
            text: "pong".into(),
            stop_reason: "end_turn".into(),
            opened: "new".into(),
            session_id: "sess-1".into(),
        };
        publish_layout(&root, &turn, "one\n!/two", None).unwrap();
        let dir = root.join("grok").join("sess-1");
        let state: Value =
            serde_json::from_slice(&fs::read(dir.join("state.json")).unwrap()).unwrap();
        assert_eq!(state["session_id"], "sess-1");
        assert!(state["pane_id"].is_null());
        let events = fs::read_to_string(dir.join("events.jsonl")).unwrap();
        assert!(
            events.contains("one\\n!/two") || events.contains("one\n!/two"),
            "{events}"
        );
        let view = fs::read_to_string(dir.join("view.log")).unwrap();
        assert!(view.contains("pong"), "{view}");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn the_task_body_is_not_a_command_argument() {
        let body = "one\n!/two";
        let args = argv(Path::new("/tmp/leader.sock"));
        assert!(args.iter().all(|arg| !arg.contains(body) && arg != body));
        assert_eq!(
            args,
            vec![
                "agent".to_owned(),
                "--no-leader".to_owned(),
                "--leader-socket".to_owned(),
                "/tmp/leader.sock".to_owned(),
                "stdio".to_owned(),
            ]
        );
    }

    #[test]
    fn a_multi_kilobyte_prompt_is_delivered_as_acp_text() {
        let mut body = "head\n!/keep\n".to_owned();
        body.push_str(&"x".repeat(4096));
        body.push_str("\n/tail");
        let seen = Arc::new(Mutex::new(Vec::new()));
        let mut dialogue = dialogue_with(Arc::clone(&seen), false);
        let deadline = Instant::now() + Duration::from_secs(2);
        dialogue.handshake(deadline).unwrap();
        dialogue.ensure(None, deadline).unwrap();
        let turn = dialogue
            .prompt(&body, deadline, &AtomicBool::new(false))
            .unwrap();
        assert_eq!(turn.text, body);
        assert!(
            seen.lock()
                .unwrap()
                .iter()
                .any(|(method, text)| method == "session/prompt" && text == &body)
        );
    }

    #[test]
    fn confine_allow_leaves_out_personal_credential_directories() {
        let home = Path::new("/home/person");
        let state = Path::new("/tmp/hctl2-state");
        let read_only = confine_allow(&home.join(".grok"), state, state);
        assert_eq!(
            read_only.lines().collect::<Vec<_>>(),
            vec!["/home/person/.grok", "/tmp/hctl2-state"]
        );
        let writing = confine_allow(&home.join(".grok"), state, Path::new("/tmp/hctl2-work"));
        assert_eq!(
            writing.lines().collect::<Vec<_>>(),
            vec!["/home/person/.grok", "/tmp/hctl2-state", "/tmp/hctl2-work"]
        );
        for line in writing.lines() {
            assert!(!line.ends_with("/.ssh"), "{line}");
            assert!(!line.contains(".config/gh"), "{line}");
            assert!(!line.contains("Keychains"), "{line}");
            assert_ne!(line, "/home/person");
        }
    }

    #[test]
    fn a_write_spec_reaches_the_boundary_check() {
        let pool = Pool::new(PathBuf::from("/bin/true"), Duration::from_secs(30));
        let root = std::env::temp_dir().join(format!("hctl2-grok-write-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("exec")).unwrap();
        fs::create_dir_all(root.join("tenant")).unwrap();
        fs::create_dir_all(root.join("cred")).unwrap();
        let spec = sealed_write_spec();
        let bundle = sealed_task_bundle("edit the tree");
        let error = match pool.submit(
            &spec,
            &bundle,
            &root.join("exec"),
            &root.join("tenant"),
            &root.join("cred"),
            View {
                observation: crate::observation::ObservationConfig::disabled(),
                multiplexer: None,
            },
        ) {
            Err(error) => error,
            Ok(_) => panic!("an incomplete write spec was accepted"),
        };
        assert_eq!(error.code, "WRITE_BOUNDARY_MISMATCH");
        assert_ne!(error.message, "grok-build write dispatch is not wired yet");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    #[ignore = "UNVERIFIED: logged-in grok required; HCTL2_GROK_REAL=1 and HCTL2_GROK"]
    fn live_read_only_research_returns_a_proposal() {
        assert_eq!(
            std::env::var("HCTL2_GROK_REAL").as_deref(),
            Ok("1"),
            "UNVERIFIED: local grok login required"
        );
        let bin = PathBuf::from(std::env::var("HCTL2_GROK").expect("HCTL2_GROK"));
        let root = std::env::temp_dir().join(format!("hctl2-grok-live-{}", std::process::id()));
        let cred = root.join("cred");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&cred).unwrap();
        fs::create_dir_all(&root).unwrap();
        let mut link = Link::spawn(
            &bin,
            &root,
            &cred,
            &root,
            Instant::now() + Duration::from_secs(30),
        )
        .unwrap_or_else(|error| panic!("{}: {}", error.code, error.message));
        let turn = link
            .dialogue
            .prompt(
                "Reply with exactly HCTL2_GROK_REAL_OK and nothing else.",
                Instant::now() + Duration::from_secs(90),
                &AtomicBool::new(false),
            )
            .unwrap_or_else(|error| panic!("{}: {}", error.code, error.message));
        assert!(turn.text.contains("HCTL2_GROK_REAL_OK"), "{}", turn.text);
        link.close();
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    #[ignore = "UNVERIFIED: logged-in grok and a materialized ChangeSet required; HCTL2_GROK_REAL=1"]
    fn live_write_dispatch_seals_a_changeset() {
        assert_eq!(
            std::env::var("HCTL2_GROK_REAL").as_deref(),
            Ok("1"),
            "UNVERIFIED: local grok login and a write lease required"
        );
        panic!("UNVERIFIED: live write needs a frozen ChangeSet worktree and was not run here");
    }

    #[test]
    fn argv_does_not_auto_approve() {
        let args = argv(Path::new("/tmp/leader.sock"));
        assert!(args.iter().all(|arg| arg != "--always-approve"));
        assert!(args.iter().all(|arg| !arg.contains("bypassPermissions")));
        assert!(args.contains(&"stdio".to_owned()));
    }

    #[test]
    fn one_session_keeps_exact_text_across_two_prompts() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let mut dialogue = dialogue_with(Arc::clone(&seen), false);
        let deadline = Instant::now() + Duration::from_secs(2);
        dialogue.handshake(deadline).unwrap();
        assert_eq!(dialogue.ensure(None, deadline).unwrap(), "new");
        let first = dialogue
            .prompt("one\n!/two", deadline, &AtomicBool::new(false))
            .unwrap();
        let second = dialogue
            .prompt("/cmd", deadline, &AtomicBool::new(false))
            .unwrap();
        assert_eq!(first.text, "one\n!/two");
        assert_eq!(first.stop_reason, "end_turn");
        assert_eq!(second.text, "/cmd");
        let methods: Vec<_> = seen
            .lock()
            .unwrap()
            .iter()
            .map(|(method, _)| method.clone())
            .collect();
        assert_eq!(
            methods,
            vec![
                "initialize",
                "authenticate",
                "session/new",
                "session/prompt",
                "session/prompt",
            ]
        );
    }

    #[test]
    fn a_lost_session_is_marked_and_a_permission_request_is_refused() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let mut dialogue = dialogue_with(Arc::clone(&seen), true);
        let deadline = Instant::now() + Duration::from_secs(2);
        dialogue.handshake(deadline).unwrap();
        assert_eq!(
            dialogue.ensure(Some("missing"), deadline).unwrap(),
            "load_failed_new"
        );
        let error = Dialogue::new(
            std::io::Cursor::new(
                b"{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"session/request_permission\",\"params\":{}}\n"
                    .to_vec(),
            ),
            Vec::<u8>::new(),
        )
        .handshake(deadline)
        .unwrap_err();
        assert!(error.message.contains("will not answer"));
    }

    #[test]
    fn cancel_is_a_notification() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let mut dialogue = dialogue_with(Arc::clone(&seen), false);
        let deadline = Instant::now() + Duration::from_secs(2);
        dialogue.handshake(deadline).unwrap();
        dialogue.ensure(None, deadline).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&cancel);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(40));
            flag.store(true, Ordering::SeqCst);
        });
        let turn = dialogue.prompt("wait", deadline, &cancel).unwrap();
        assert_eq!(turn.stop_reason, "cancelled");
        assert!(
            seen.lock()
                .unwrap()
                .iter()
                .any(|(method, _)| method == "session/cancel")
        );
    }

    #[test]
    fn a_deadline_that_cancel_finishes_is_a_cancelled_turn() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let mut dialogue = hanging_dialogue(Arc::clone(&seen), true);
        let deadline = Instant::now() + Duration::from_secs(2);
        dialogue.handshake(deadline).unwrap();
        dialogue.ensure(None, deadline).unwrap();
        let turn = dialogue
            .prompt(
                "hang",
                Instant::now() + Duration::from_millis(40),
                &AtomicBool::new(false),
            )
            .unwrap();
        assert_eq!(turn.stop_reason, "cancelled");
        assert!(turn.text.is_empty());
        assert!(
            seen.lock()
                .unwrap()
                .iter()
                .any(|(method, _)| method == "session/cancel")
        );
    }

    #[test]
    fn a_deadline_cancel_that_is_not_answered_is_an_error() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let mut dialogue = hanging_dialogue(Arc::clone(&seen), false);
        let deadline = Instant::now() + Duration::from_secs(2);
        dialogue.handshake(deadline).unwrap();
        dialogue.ensure(None, deadline).unwrap();
        let started = Instant::now();
        let error = dialogue
            .prompt(
                "hang",
                Instant::now() + Duration::from_millis(40),
                &AtomicBool::new(false),
            )
            .unwrap_err();
        assert_eq!(error.code, "HARNESS_TURN_ERROR");
        assert!(
            error
                .message
                .contains("session/prompt exceeded the turn budget")
        );
        assert!(
            started.elapsed() < Duration::from_millis(900),
            "{:?}",
            started.elapsed()
        );
        assert!(
            seen.lock()
                .unwrap()
                .iter()
                .any(|(method, _)| method == "session/cancel")
        );
    }

    #[test]
    fn dropping_the_link_kills_the_process_group() {
        let child = Command::new("sleep")
            .arg("30")
            .process_group(0)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let pid = child.id();
        let (agent_read, client_write) = std::io::pipe().unwrap();
        let (client_read, _agent_write) = std::io::pipe().unwrap();
        drop(agent_read);
        let dialogue = Dialogue::new(BufReader::new(client_read), client_write);
        drop(Link {
            child,
            dialogue,
            opened: String::new(),
            cwd: PathBuf::new(),
            closed: false,
        });
        assert!(
            fs::metadata(format!("/proc/{pid}")).is_err(),
            "pid {pid} still alive"
        );
    }

    fn hanging_dialogue(
        seen: Arc<Mutex<Vec<(String, String)>>>,
        answer_cancel: bool,
    ) -> Dialogue<std::io::PipeWriter> {
        let (agent_read, client_write) = std::io::pipe().unwrap();
        let (client_read, mut agent_write) = std::io::pipe().unwrap();
        std::thread::spawn(move || {
            let mut incoming = BufReader::new(agent_read);
            let mut pending = None;
            loop {
                let mut line = String::new();
                if incoming.read_line(&mut line).unwrap() == 0 {
                    break;
                }
                let message: Value = serde_json::from_str(line.trim()).unwrap();
                let method = message["method"].as_str().unwrap_or("").to_owned();
                seen.lock().unwrap().push((method.clone(), String::new()));
                if method == "session/prompt" {
                    pending = Some(message["id"].clone());
                    continue;
                }
                if method == "session/cancel" {
                    if answer_cancel && let Some(id) = pending.take() {
                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": {"stopReason": "cancelled"}
                        });
                        writeln!(agent_write, "{response}").unwrap();
                    }
                    continue;
                }
                let id = message["id"].clone();
                let result = match method.as_str() {
                    "initialize" => {
                        json!({"protocolVersion": 1, "authMethods": [{"id": "cached_token"}]})
                    }
                    "session/new" => json!({"sessionId": "sess-1"}),
                    _ => json!({}),
                };
                let response = json!({"jsonrpc": "2.0", "id": id, "result": result});
                writeln!(agent_write, "{response}").unwrap();
            }
        });
        Dialogue::new(BufReader::new(client_read), client_write)
    }

    fn dialogue_with(
        seen: Arc<Mutex<Vec<(String, String)>>>,
        fail_load: bool,
    ) -> Dialogue<std::io::PipeWriter> {
        let (agent_read, client_write) = std::io::pipe().unwrap();
        let (client_read, mut agent_write) = std::io::pipe().unwrap();
        std::thread::spawn(move || {
            let mut incoming = BufReader::new(agent_read);
            let mut pending = None;
            loop {
                let mut line = String::new();
                if incoming.read_line(&mut line).unwrap() == 0 {
                    break;
                }
                let message: Value = serde_json::from_str(line.trim()).unwrap();
                let method = message["method"].as_str().unwrap_or("").to_owned();
                let text = message["params"]["prompt"][0]["text"]
                    .as_str()
                    .unwrap_or("")
                    .to_owned();
                seen.lock().unwrap().push((method.clone(), text.clone()));
                if method == "session/prompt" && text == "wait" {
                    pending = Some(message["id"].clone());
                    continue;
                }
                if method == "session/cancel" {
                    if let Some(id) = pending.take() {
                        let update = json!({
                            "jsonrpc": "2.0",
                            "method": "session/update",
                            "params": {"update": {
                                "sessionUpdate": "agent_message_chunk",
                                "content": {"type": "text", "text": "cancelled"}
                            }}
                        });
                        writeln!(agent_write, "{update}").unwrap();
                        let response = json!({"jsonrpc":"2.0","id": id, "result": {"stopReason": "cancelled"}});
                        writeln!(agent_write, "{response}").unwrap();
                    }
                    continue;
                }
                if method == "session/prompt" {
                    let update = json!({
                        "jsonrpc": "2.0",
                        "method": "session/update",
                        "params": {"update": {
                            "sessionUpdate": "agent_message_chunk",
                            "content": {"type": "text", "text": text}
                        }}
                    });
                    writeln!(agent_write, "{update}").unwrap();
                }
                let id = message["id"].clone();
                if method == "session/load" && fail_load {
                    let response = json!({"jsonrpc":"2.0","id": id, "error": {"code": -32602, "message": "missing"}});
                    writeln!(agent_write, "{response}").unwrap();
                    continue;
                }
                let result = match method.as_str() {
                    "initialize" => {
                        json!({"protocolVersion": 1, "authMethods": [{"id": "cached_token"}]})
                    }
                    "session/new" => json!({"sessionId": "sess-1"}),
                    "session/prompt" => json!({"stopReason": "end_turn"}),
                    _ => json!({}),
                };
                let response = json!({"jsonrpc": "2.0", "id": id, "result": result});
                writeln!(agent_write, "{response}").unwrap();
            }
        });
        Dialogue::new(BufReader::new(client_read), client_write)
    }

    fn sealed_write_spec() -> Sealed<ExecutionSpec> {
        use agency_proto::{Capabilities, FrozenRef, InputPolicy, Owner, OwnerKind, Profession};
        let frozen = |id: &str| FrozenRef {
            id: id.into(),
            revision: "1".into(),
            digest: agency_proto::hash(id.as_bytes()),
        };
        Sealed::new(ExecutionSpec {
            owner: Owner {
                project: "project".into(),
                kind: OwnerKind::RoomInvocation,
                id: "invocation".into(),
                generation: 1,
            },
            project: frozen("project"),
            selection: frozen("selection"),
            selection_policy_digest: agency_proto::hash(b"policy"),
            profession: Profession {
                reference: frozen("grok-build"),
                harness: frozen("grok"),
                model: "grok/default".into(),
                persona: "test".into(),
                terms: "test".into(),
                default_role: "worker".into(),
                skills: vec![],
                capabilities: Capabilities {
                    stop: true,
                    ..Capabilities::default()
                },
            },
            profile: frozen("profile"),
            manifest: frozen("manifest"),
            bundle: frozen("bundle"),
            binding: frozen("binding"),
            required_capabilities: Capabilities {
                stop: true,
                ..Capabilities::default()
            },
            input_policy: InputPolicy::NoInput,
            permission_digest: agency_proto::hash(b"permissions"),
            permissions: vec!["context.read".into(), "git.write".into()],
            budget: 1,
            deadline_ms: crate::storage::now_ms() + 60_000,
            repo: None,
            base: None,
            delivery_scope: vec![],
            write_lease: Some(frozen("lease")),
            review_publish_policy: None,
            idempotency_key: "grok-write".into(),
        })
        .unwrap()
    }

    fn sealed_task_bundle(task: &str) -> Sealed<Bundle> {
        use agency_proto::context::{Bundle, Delivery, Entry};
        use agency_proto::{FrozenRef, Owner, OwnerKind};
        let frozen = |id: &str| FrozenRef {
            id: id.into(),
            revision: "1".into(),
            digest: agency_proto::hash(id.as_bytes()),
        };
        Sealed::new(Bundle {
            id: "bundle".into(),
            manifest: frozen("manifest"),
            consumer: Owner {
                project: "project".into(),
                kind: OwnerKind::RoomInvocation,
                id: "invocation".into(),
                generation: 1,
            },
            entries: vec![Entry {
                source: frozen("task"),
                description: "task".into(),
                required: true,
                offline_required: false,
                delivery: Delivery::Inline {
                    bytes: task.as_bytes().to_vec(),
                },
                bytes_digest: agency_proto::hash(task.as_bytes()),
            }],
            renderer: frozen("renderer"),
            tokenizer: frozen("tokenizer"),
            redaction: frozen("redaction"),
            compression: vec![],
            candidate_tokens: None,
            selected_tokens: None,
            delivered_tokens: None,
            permission_digest: agency_proto::hash(b"permissions"),
            budget: 1,
            retention: "test".into(),
        })
        .unwrap()
    }
}
