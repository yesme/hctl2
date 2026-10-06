//! One Codex app-server per selection. Herdr only shows `codex resume --remote`.
//! Login stays in the process environment. This module does not copy it or edit `~/.codex`.
use crate::{
    herdr::{Client, Server},
    runtime::{Running, RuntimeEvent, Session},
    standby::selection_key,
};
use agency_proto::{
    EvidenceLevel, ExecutionSpec, PortError, Result, Sealed, canonical, context::Bundle, hash,
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    os::unix::{net::UnixStream, process::CommandExt},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

const LIMIT: u64 = 16 * 1024 * 1024;

struct Token {
    cancelled: AtomicBool,
    finished: AtomicBool,
}
struct Handle(Arc<Token>);
impl Session for Handle {
    fn input(&mut self, _: &[u8]) -> Result<()> {
        Err(PortError::invalid("standby input is not advertised"))
    }
    fn stop(&mut self) -> Result<()> {
        if !self.0.finished.load(Ordering::SeqCst) {
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
    spec: Sealed<ExecutionSpec>,
    text: String,
    token: Arc<Token>,
    tx: std::sync::mpsc::SyncSender<RuntimeEvent>,
}
struct Worker {
    tx: std::sync::mpsc::Sender<Job>,
    join: JoinHandle<Result<()>>,
}

pub(crate) struct Pool {
    codex: PathBuf,
    idle: Duration,
    stopped: Arc<AtomicBool>,
    workers: Mutex<HashMap<String, Worker>>,
}

impl Pool {
    pub(crate) fn new(codex: PathBuf, idle: Duration) -> Self {
        Self {
            codex,
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
    ) -> Result<Running> {
        crate::standby::readonly(&spec.document)?;
        let text = crate::launch::task_text(&bundle.document)?;
        let key = selection_key(&spec.document, tenant)?;
        let token = Arc::new(Token {
            cancelled: AtomicBool::new(false),
            finished: AtomicBool::new(false),
        });
        let (tx, rx) = std::sync::mpsc::sync_channel(8);
        let job = Job {
            spec: spec.clone(),
            text,
            token: Arc::clone(&token),
            tx,
        };
        let mut workers = self.workers.lock().expect("codex workers");
        if self.stopped.load(Ordering::SeqCst) {
            return Err(PortError::invalid("runtime is stopping"));
        }
        if !workers.contains_key(&key) {
            let (tx, receiver) = std::sync::mpsc::channel();
            let codex = self.codex.clone();
            let stopped = Arc::clone(&self.stopped);
            let idle = self.idle;
            let parent = exec
                .parent()
                .ok_or_else(|| PortError::invalid("execution parent missing"))?;
            let cwd = parent.join(format!("codex-session-{key}"));
            crate::storage::private_dir(&cwd)?;
            let worker_key = key.clone();
            let join = std::thread::spawn(move || {
                worker(server, codex, cwd, worker_key, idle, stopped, receiver)
            });
            workers.insert(key.clone(), Worker { tx, join });
        }
        workers
            .get(&key)
            .expect("inserted codex worker")
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
        let workers = std::mem::take(&mut *self.workers.lock().expect("codex workers"));
        let mut error = None;
        for (_, Worker { tx, join }) in workers {
            drop(tx);
            match join.join() {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    error.get_or_insert(e);
                }
                Err(_) => {
                    error.get_or_insert_with(|| PortError::invalid("codex worker panicked"));
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

fn worker(
    server: Arc<Server>,
    codex: PathBuf,
    cwd: PathBuf,
    key: String,
    idle: Duration,
    stopped: Arc<AtomicBool>,
    receiver: std::sync::mpsc::Receiver<Job>,
) -> Result<()> {
    let dir = server.state.join(format!("codex-{key}"));
    crate::storage::private_dir(&dir)?;
    let socket = server
        .socket
        .parent()
        .ok_or_else(|| PortError::invalid("Herdr socket directory is missing"))?
        .join(format!("codex-{}.sock", &hash(key.as_bytes())[..8]));
    let mut live: Option<Live> = None;
    let mut last = Instant::now();
    loop {
        if stopped.load(Ordering::SeqCst) {
            break;
        }
        let job = match receiver.recv_timeout(Duration::from_millis(25)) {
            Ok(job) => job,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if live.is_some() && last.elapsed() >= idle {
                    if let Some(session) = live.as_mut() {
                        session.close()?;
                    }
                    live = None;
                }
                continue;
            }
        };
        let result = run_job(
            &TurnEnv {
                server: &server,
                codex: &codex,
                cwd: &cwd,
                dir: &dir,
                socket: &socket,
            },
            &stopped,
            &mut live,
            &job,
        );
        job.token.finished.store(true, Ordering::SeqCst);
        if let Err(error) = result {
            let _ = job.tx.send(RuntimeEvent::Observation {
                kind: "harness_failure".into(),
                payload: json!({"code":error.code,"message":error.message}),
                source: EvidenceLevel::AdapterEvent,
            });
            let _ = job.tx.send(RuntimeEvent::ProtocolError(error.code));
            if let Some(session) = live.as_mut() {
                session.close()?;
            }
            live = None;
        }
        let _ = job.tx.send(RuntimeEvent::DispatchReleased);
        last = Instant::now();
    }
    if let Some(session) = live.as_mut() {
        session.close()?;
    }
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

struct TurnEnv<'a> {
    server: &'a Arc<Server>,
    codex: &'a Path,
    cwd: &'a Path,
    dir: &'a Path,
    socket: &'a Path,
}

fn run_job(
    env: &TurnEnv<'_>,
    stopped: &AtomicBool,
    live: &mut Option<Live>,
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
    if live.is_none() {
        let (session, resumed, resume_failed) =
            Live::open(env.codex, env.cwd, env.dir, env.socket)?;
        let _ = job.tx.send(RuntimeEvent::Observation {
            kind: "session_opened".into(),
            payload: json!({"resumed":resumed,"resume_failed":resume_failed,"thread":session.thread}),
            source: EvidenceLevel::AdapterEvent,
        });
        *live = Some(session);
    }
    let requested = job.token.cancelled.load(Ordering::SeqCst) || stopped.load(Ordering::SeqCst);
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
    let session = live.as_mut().expect("codex session");
    if session.resumed_existing {
        session.ensure_pane(env.server, env.codex, env.cwd, env.dir)?;
    }
    let turn_id = session.turn_start(&job.text)?;
    let mut answer = None;
    let mut interrupt: Option<(Instant, bool)> = None;
    loop {
        let requested =
            job.token.cancelled.load(Ordering::SeqCst) || stopped.load(Ordering::SeqCst);
        let expired = crate::storage::now_ms() >= job.spec.document.deadline_ms;
        if interrupt.is_none() && (requested || expired) {
            if expired {
                let _ = job.tx.send(RuntimeEvent::DeadlineReached);
            }
            match session.interrupt(&turn_id) {
                Ok(()) => interrupt = Some((Instant::now(), requested)),
                Err(_) => {
                    session.close()?;
                    let _ = job.tx.send(RuntimeEvent::TurnStopped {
                        requested_stop: requested,
                        session_closed: true,
                    });
                    *live = None;
                    return Ok(());
                }
            }
        }
        match session.rpc.poll(Duration::from_millis(200))? {
            Some(msg) if msg.get("id").is_some() => {}
            Some(msg) => {
                if msg["method"] == "item/completed"
                    && msg["params"]["turnId"] == turn_id
                    && matches!(
                        msg["params"]["item"]["type"].as_str(),
                        Some("agentMessage" | "agent_message")
                    )
                    && let Some(text) = msg["params"]["item"]["text"].as_str()
                {
                    answer = Some(text.to_owned());
                }
                if msg["method"] == "turn/completed" && msg["params"]["turn"]["id"] == turn_id {
                    if let Some((_, requested_stop)) = interrupt {
                        let _ = job.tx.send(RuntimeEvent::TurnStopped {
                            requested_stop,
                            session_closed: false,
                        });
                        return Ok(());
                    }
                    if msg["params"]["turn"]["error"].is_object() && answer.is_none() {
                        return Err(PortError::new(
                            "HARNESS_TURN_ERROR",
                            "Codex ended this turn with an error",
                            "inspect_dispatch",
                        ));
                    }
                    let text = answer.ok_or_else(|| {
                        PortError::invalid("turn completed without an agent message")
                    })?;
                    rollout_matches(&session.thread, &job.text)?;
                    session.ensure_pane(env.server, env.codex, env.cwd, env.dir)?;
                    job.token.finished.store(true, Ordering::SeqCst);
                    let _ = job.tx.send(RuntimeEvent::Proposal {
                        schema: "codex.turn.v1".into(),
                        bytes: text.into_bytes(),
                        source: EvidenceLevel::AdapterEvent,
                    });
                    let _ = job.tx.send(RuntimeEvent::TurnReturned);
                    return Ok(());
                }
            }
            None => {}
        }
        if let Some((sent, requested_stop)) = interrupt
            && sent.elapsed() >= Duration::from_secs(3)
        {
            session.close()?;
            let _ = job.tx.send(RuntimeEvent::TurnStopped {
                requested_stop,
                session_closed: true,
            });
            *live = None;
            return Ok(());
        }
    }
}

struct Live {
    child: Child,
    rpc: Rpc,
    socket: PathBuf,
    thread: String,
    resumed_existing: bool,
    pane: Option<Pane>,
    closed: bool,
}

struct Pane {
    client: Client,
    id: String,
}

impl Live {
    fn open(codex: &Path, cwd: &Path, dir: &Path, socket: &Path) -> Result<(Self, bool, bool)> {
        let _ = fs::remove_file(socket);
        let log = crate::storage::private_file(&dir.join("app-server.err"))?;
        let mut child = Command::new(codex);
        child
            .args(listen_args(socket))
            .stdin(Stdio::null())
            .stdout(Stdio::from(log.try_clone()?))
            .stderr(Stdio::from(log))
            .current_dir(cwd)
            .process_group(0);
        let mut child = child.spawn()?;
        let rpc = wait_socket(socket, &mut child)?;
        let saved = read_thread(dir)?;
        let mut live = Self {
            child,
            rpc,
            socket: socket.to_path_buf(),
            thread: String::new(),
            resumed_existing: false,
            pane: None,
            closed: false,
        };
        let attached = (|| {
            live.rpc.initialize()?;
            if let Some(id) = &saved
                && live
                    .rpc
                    .request("thread/resume", json!({"threadId": id}))
                    .is_ok()
            {
                live.thread = id.clone();
                live.resumed_existing = true;
                return Ok(false);
            }
            let started = live.rpc.request(
                "thread/start",
                json!({"cwd": cwd, "approvalPolicy": "never", "sandbox": "read-only"}),
            )?;
            let id = started["thread"]["id"]
                .as_str()
                .filter(|id| !id.is_empty())
                .ok_or_else(|| PortError::invalid("thread/start returned no thread id"))?
                .to_owned();
            write_thread(dir, &id)?;
            live.thread = id;
            Ok(saved.is_some())
        })();
        let resumed = live.resumed_existing;
        match attached {
            Ok(resume_failed) => Ok((live, resumed, resume_failed)),
            Err(error) => {
                live.close()?;
                Err(error)
            }
        }
    }

    fn turn_start(&mut self, text: &str) -> Result<String> {
        let turn = self.rpc.request(
            "turn/start",
            json!({
                "threadId": self.thread,
                "approvalPolicy": "never",
                "sandboxPolicy": {"type": "readOnly"},
                "input": [{"type": "text", "text": text}],
            }),
        )?;
        turn["turn"]["id"]
            .as_str()
            .filter(|id| !id.is_empty())
            .map(str::to_owned)
            .ok_or_else(|| PortError::invalid("turn/start returned no turn id"))
    }

    fn interrupt(&mut self, turn: &str) -> Result<()> {
        self.rpc
            .request(
                "turn/interrupt",
                json!({"threadId": self.thread, "turnId": turn}),
            )
            .map(|_| ())
    }

    fn ensure_pane(
        &mut self,
        server: &Arc<Server>,
        codex: &Path,
        cwd: &Path,
        dir: &Path,
    ) -> Result<()> {
        if let Some(pane) = &self.pane
            && pane
                .client
                .call("agent.get", json!({"target": pane.id}))
                .is_ok_and(|v| v["agent"]["agent"] == "codex")
        {
            return Ok(());
        }
        if let Some(pane) = self.pane.as_mut() {
            let _ = pane.client.call("pane.close", json!({"pane_id": pane.id}));
        }
        self.pane = Some(open_pane(
            server,
            codex,
            cwd,
            dir,
            &self.thread,
            &self.socket,
        )?);
        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        if self.closed {
            return Ok(());
        }
        self.closed = true;
        if let Some(pane) = self.pane.as_mut() {
            let _ = pane.client.call("pane.close", json!({"pane_id": pane.id}));
        }
        signal_group(self.child.id(), "TERM");
        let deadline = Instant::now() + Duration::from_millis(400);
        while Instant::now() < deadline {
            if self.child.try_wait()?.is_some() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        signal_group(self.child.id(), "KILL");
        let _ = self.child.wait();
        let _ = fs::remove_file(&self.socket);
        Ok(())
    }
}

impl Drop for Live {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

fn open_pane(
    server: &Arc<Server>,
    codex: &Path,
    cwd: &Path,
    dir: &Path,
    thread: &str,
    socket: &Path,
) -> Result<Pane> {
    let bin = dir.join("bin");
    crate::storage::private_dir(&bin)?;
    let executable = bin.join("codex");
    match fs::read_link(&executable) {
        Ok(existing) if existing == codex => {}
        Ok(_) => return Err(PortError::invalid("private Codex link changed")),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            std::os::unix::fs::symlink(codex, &executable)?
        }
        Err(e) => return Err(e.into()),
    }
    let client = Client::connect(&server.socket)?;
    let home = std::env::var("HOME").unwrap_or_default();
    let user = std::env::var("USER").unwrap_or_default();
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".into())
    );
    let created = client.call(
        "workspace.create",
        json!({"cwd":cwd,"focus":false,"label":"agency-codex","env":{"HOME":home,"USER":user,"PATH":path}}),
    )?;
    let tab = created
        .pointer("/tab/tab_id")
        .and_then(Value::as_str)
        .ok_or_else(|| PortError::invalid("workspace tab missing"))?;
    let applied = client.call(
        "layout.apply",
        json!({"tab_id":tab,"focus":false,"root":{"type":"pane","cwd":cwd,"command":["/usr/bin/env",
            format!("PATH={path}"), format!("HOME={home}"), format!("USER={user}"), "/bin/sh"]}}),
    )?;
    let pane = applied
        .pointer("/layout/root/pane_id")
        .and_then(Value::as_str)
        .ok_or_else(|| PortError::invalid("layout pane missing"))?
        .to_owned();
    let remote = format!("unix://{}", socket.display());
    client.call(
        "agent.start",
        json!({"name":format!("codex-{}", &hash(dir.as_os_str().as_encoded_bytes())[..24]),
            "kind":"codex","pane_id":pane,"args":["resume", thread, "--remote", remote],
            "timeout_ms":30_000}),
    )?;
    let timeout = Instant::now() + Duration::from_secs(30);
    while Instant::now() < timeout {
        if client
            .call("agent.get", json!({"target": pane}))
            .is_ok_and(|v| v["agent"]["agent"] == "codex")
        {
            return Ok(Pane { client, id: pane });
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = client.call("pane.close", json!({"pane_id": pane}));
    Err(PortError::new(
        "STANDBY_START_TIMEOUT",
        "codex --remote did not become ready",
        "inspect_herdr_pane",
    ))
}

fn wait_socket(socket: &Path, child: &mut Child) -> Result<Rpc> {
    let deadline = Instant::now() + Duration::from_secs(20);
    while Instant::now() < deadline {
        if socket.exists() {
            return Rpc::connect(socket);
        }
        if child.try_wait()?.is_some() {
            return Err(PortError::new(
                "CODEX_APP_SERVER_EXITED",
                "app-server exited before its socket appeared",
                "inspect_app_server_log",
            ));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Err(PortError::new(
        "CODEX_APP_SERVER_EXITED",
        "app-server did not open its socket",
        "inspect_app_server_log",
    ))
}

pub(crate) fn listen_args(socket: &Path) -> Vec<String> {
    vec![
        "app-server".into(),
        "--listen".into(),
        format!("unix://{}", socket.display()),
    ]
}

fn read_thread(dir: &Path) -> Result<Option<String>> {
    let mut file = match fs::File::open(dir.join("thread.json")) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let value: Value = serde_json::from_slice(&bytes)?;
    Ok(value["thread"].as_str().map(str::to_owned))
}

fn write_thread(dir: &Path, thread: &str) -> Result<()> {
    let path = dir.join("thread.json");
    let temp = path.with_extension("pending");
    let mut file = crate::storage::private_file(&temp)?;
    file.write_all(&canonical(&json!({"thread": thread}))?)?;
    file.sync_all()?;
    fs::rename(temp, path)?;
    Ok(())
}

pub(crate) fn rollout_matches(thread: &str, text: &str) -> Result<()> {
    let home = std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".codex")
        });
    let sessions = home.join("sessions");
    let mut found = None;
    if sessions.is_dir() {
        let _ = visit_rollouts(&sessions, thread, 0, &mut found);
    }
    let path = found.ok_or_else(|| {
        PortError::new(
            "CODEX_ROLLOUT_MISSING",
            "no rollout file for this thread",
            "inspect_codex_session",
        )
    })?;
    if rollout_file_has_input(&path, text)? {
        Ok(())
    } else {
        Err(PortError::new(
            "STANDBY_PROMPT_MISMATCH",
            "rollout input_text does not match this dispatch",
            "inspect_dispatch",
        ))
    }
}

fn visit_rollouts(dir: &Path, thread: &str, depth: u8, found: &mut Option<PathBuf>) -> Result<()> {
    if depth > 6 || found.is_some() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_rollouts(&path, thread, depth + 1, found)?;
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.contains(thread) && name.ends_with(".jsonl"))
        {
            *found = Some(path);
            return Ok(());
        }
    }
    Ok(())
}

pub(crate) fn rollout_file_has_input(path: &Path, text: &str) -> Result<bool> {
    let mut file = fs::File::open(path)?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > LIMIT {
        return Err(PortError::invalid("rollout exceeds read limit"));
    }
    let body = String::from_utf8_lossy(&bytes);
    for line in body.lines() {
        if !line.contains("input_text") {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if input_text_eq(&value, text) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn input_text_eq(value: &Value, text: &str) -> bool {
    let mut stack = vec![value];
    while let Some(current) = stack.pop() {
        match current {
            Value::Object(map) => {
                if map.get("type").and_then(Value::as_str) == Some("input_text")
                    && map.get("text").and_then(Value::as_str) == Some(text)
                {
                    return true;
                }
                stack.extend(map.values());
            }
            Value::Array(items) => stack.extend(items),
            _ => {}
        }
    }
    false
}

struct Rpc {
    stream: UnixStream,
    buf: Vec<u8>,
    next: u64,
}

impl Rpc {
    fn connect(path: &Path) -> Result<Self> {
        let mut stream = UnixStream::connect(path)?;
        stream.set_read_timeout(Some(Duration::from_secs(20)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;
        let key = "dGhlIHNhbXBsZSBub25jZQ==";
        let req = format!(
            "GET / HTTP/1.1\r\nHost: localhost\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n"
        );
        stream.write_all(req.as_bytes())?;
        let mut buf = Vec::new();
        let mut tmp = [0; 1024];
        while !buf.windows(4).any(|w| w == b"\r\n\r\n") {
            let n = read_some(&mut stream, &mut tmp)?;
            buf.extend_from_slice(&tmp[..n]);
            if buf.len() > 8192 {
                return Err(PortError::invalid("app-server upgrade too large"));
            }
        }
        let status = buf
            .split(|b| *b == b'\n')
            .next()
            .and_then(|line| std::str::from_utf8(line).ok())
            .unwrap_or("");
        if !status.contains("101") {
            return Err(PortError::new(
                "CODEX_APP_SERVER_EXITED",
                "app-server refused the websocket",
                "inspect_app_server_log",
            ));
        }
        let header = buf.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4;
        buf.drain(..header);
        Ok(Self {
            stream,
            buf,
            next: 1,
        })
    }

    fn initialize(&mut self) -> Result<()> {
        self.request(
            "initialize",
            json!({"clientInfo":{"name":"hctl2-agency","title":"hctl2-agency","version":"0"}}),
        )?;
        self.notify("initialized", json!({}))?;
        Ok(())
    }

    fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next;
        self.next += 1;
        self.send(&json!({"id": id, "method": method, "params": params}))?;
        let deadline = Instant::now() + Duration::from_secs(40);
        while Instant::now() < deadline {
            let Some(msg) = self.poll(Duration::from_secs(20))? else {
                continue;
            };
            if msg.get("id").and_then(Value::as_u64) == Some(id) {
                if let Some(error) = msg.get("error") {
                    return Err(PortError::new(
                        "CODEX_RPC",
                        redact(
                            error["message"]
                                .as_str()
                                .unwrap_or("app-server request failed"),
                        ),
                        "inspect_dispatch",
                    ));
                }
                return Ok(msg.get("result").cloned().unwrap_or(Value::Null));
            }
        }
        Err(PortError::new(
            "CODEX_RPC",
            "app-server request timed out",
            "inspect_dispatch",
        ))
    }

    fn notify(&mut self, method: &str, params: Value) -> Result<()> {
        self.send(&json!({"method": method, "params": params}))
    }

    fn poll(&mut self, timeout: Duration) -> Result<Option<Value>> {
        self.stream.set_read_timeout(Some(timeout))?;
        match self.read_frame() {
            Ok(text) => Ok(Some(serde_json::from_str(&text)?)),
            Err(error)
                if error.kind() == std::io::ErrorKind::TimedOut
                    || error.kind() == std::io::ErrorKind::WouldBlock =>
            {
                Ok(None)
            }
            Err(error) => Err(error.into()),
        }
    }

    fn send(&mut self, value: &Value) -> Result<()> {
        let data = serde_json::to_vec(value)?;
        let mask = [rand_byte(), rand_byte(), rand_byte(), rand_byte()];
        let mut frame = Vec::new();
        frame.push(0x81);
        let n = data.len();
        if n < 126 {
            frame.push(0x80 | n as u8);
        } else if n < 65536 {
            frame.push(0x80 | 126);
            frame.extend_from_slice(&(n as u16).to_be_bytes());
        } else {
            frame.push(0x80 | 127);
            frame.extend_from_slice(&(n as u64).to_be_bytes());
        }
        frame.extend_from_slice(&mask);
        frame.extend(data.iter().enumerate().map(|(i, b)| b ^ mask[i % 4]));
        self.stream.write_all(&frame)?;
        Ok(())
    }

    fn read_frame(&mut self) -> std::io::Result<String> {
        loop {
            while self.buf.len() < 2 {
                self.fill()?;
            }
            let opcode = self.buf[0] & 0x0f;
            let masked = self.buf[1] & 0x80 != 0;
            let mut len = (self.buf[1] & 0x7f) as usize;
            let mut offset = 2;
            if len == 126 {
                while self.buf.len() < 4 {
                    self.fill()?;
                }
                len = u16::from_be_bytes([self.buf[2], self.buf[3]]) as usize;
                offset = 4;
            } else if len == 127 {
                while self.buf.len() < 10 {
                    self.fill()?;
                }
                let mut raw = [0; 8];
                raw.copy_from_slice(&self.buf[2..10]);
                len = u64::from_be_bytes(raw) as usize;
                offset = 10;
            }
            if masked {
                offset += 4;
            }
            while self.buf.len() < offset + len {
                self.fill()?;
            }
            let mut payload = self.buf[offset..offset + len].to_vec();
            if masked {
                let mask = self.buf[offset - 4..offset].to_vec();
                for (i, byte) in payload.iter_mut().enumerate() {
                    *byte ^= mask[i % 4];
                }
            }
            self.buf.drain(..offset + len);
            if opcode == 0x9 {
                continue;
            }
            if opcode == 0x8 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    "app-server closed the websocket",
                ));
            }
            return Ok(String::from_utf8_lossy(&payload).into_owned());
        }
    }

    fn fill(&mut self) -> std::io::Result<()> {
        let mut tmp = [0; 8192];
        let n = read_some(&mut self.stream, &mut tmp)?;
        self.buf.extend_from_slice(&tmp[..n]);
        Ok(())
    }
}

fn read_some(stream: &mut UnixStream, buf: &mut [u8]) -> std::io::Result<usize> {
    match stream.read(buf) {
        Ok(0) => Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "app-server closed the socket",
        )),
        other => other,
    }
}

fn signal_group(pid: u32, signal: &str) {
    let _ = Command::new("/bin/kill")
        .args(["-s", signal, "--", &format!("-{pid}")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

fn redact(text: &str) -> String {
    if text.to_ascii_lowercase().contains("bearer ") || text.contains("sk-") {
        "app-server request failed".into()
    } else {
        text.chars().take(300).collect()
    }
}

fn rand_byte() -> u8 {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    (n ^ std::process::id() as u128) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listen_does_not_install_a_daemon_or_name_a_config_file() {
        let args = listen_args(Path::new("/tmp/hctl2-codex-test/app.sock"));
        assert_eq!(
            args,
            [
                "app-server",
                "--listen",
                "unix:///tmp/hctl2-codex-test/app.sock"
            ]
        );
        assert!(
            !args
                .iter()
                .any(|arg| arg == "daemon" || arg.contains("config.toml"))
        );
    }

    #[test]
    fn turn_text_is_stored_unchanged() {
        let samples = [
            "line\n!not-a-shell\n/clear\n",
            &format!("/{}", "x".repeat(4000)),
            "!\n",
            "  keep spaces  \n\t",
        ];
        for text in samples {
            let params = json!({
                "threadId": "thr",
                "approvalPolicy": "never",
                "sandboxPolicy": {"type": "readOnly"},
                "input": [{"type": "text", "text": text}],
            });
            assert_eq!(params["input"][0]["text"], text);
        }
    }

    #[test]
    fn rollout_accepts_only_the_exact_task_text() {
        let dir = std::env::temp_dir().join(format!("hctl2-rollout-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("rollout.jsonl");
        let text = "multi\n!bang\n/slash\n";
        fs::write(
            &path,
            format!(
                "{}\n{}\n",
                json!({"payload":{"type":"message","content":[{"type":"input_text","text":"<environment_context>secret</environment_context>"}]}}),
                json!({"payload":{"type":"message","content":[{"type":"input_text","text":text}]}})
            ),
        )
        .unwrap();
        assert!(rollout_file_has_input(&path, text).unwrap());
        assert!(!rollout_file_has_input(&path, "multi").unwrap());
        assert!(!rollout_file_has_input(&path, &format!("{text} ")).unwrap());
        let _ = fs::remove_dir_all(&dir);
    }
}
