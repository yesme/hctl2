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
    credential_root: PathBuf,
    state: PathBuf,
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
    ) -> Result<Running> {
        crate::confine::reject_unenforceable_isolation(
            &spec.document.required_capabilities.isolation_effects,
        )?;
        if spec.document.profession.reference.id != PROFESSION {
            return Err(PortError::invalid("not a grok-build dispatch"));
        }
        if spec.document.write_lease.is_some() {
            return Err(PortError::new(
                "HARNESS_NOT_STARTED",
                "grok-build write dispatch is not wired yet",
                "use_read_only_until_write_slice",
            ));
        }
        let text = crate::launch::task_text(&bundle.document)?;
        let key = crate::standby::selection_key(&spec.document, tenant)?;
        let parent = exec_root
            .parent()
            .ok_or_else(|| PortError::invalid("execution parent missing"))?;
        let state = parent.join(format!("grok-session-{key}"));
        crate::storage::private_dir(&state)?;
        let token = Arc::new(Token {
            cancelled: AtomicBool::new(false),
        });
        let (tx, rx) = mpsc::sync_channel(8);
        let job = Job {
            text,
            budget: budget(spec.document.deadline_ms),
            credential_root: credential_root.to_path_buf(),
            state,
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
    let mut live: Option<Link> = None;
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
            Ok(turn) => {
                let _ = job.tx.send(RuntimeEvent::Observation {
                    kind: "grok.stop".into(),
                    payload: json!({
                        "stop_reason": turn.stop_reason,
                        "session": turn.opened,
                    }),
                    source: EvidenceLevel::AdapterEvent,
                });
                let _ = publish_layout(&job.state, &turn, &job.text);
                let _ = job.tx.send(RuntimeEvent::Proposal {
                    schema: crate::observation::TURN_OUTPUT_SCHEMA.into(),
                    bytes: turn.text.into_bytes(),
                    source: EvidenceLevel::AdapterEvent,
                });
                let _ = job.tx.send(RuntimeEvent::TurnReturned);
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

fn run_job(bin: &Path, live: &mut Option<Link>, job: &Job) -> Result<Turn> {
    if live.is_none() {
        *live = Some(Link::spawn(bin, &job.state, &job.credential_root)?);
    }
    let link = live.as_mut().expect("grok link");
    let saved = fs::read_to_string(job.state.join("session-id")).ok();
    let saved = saved.as_deref().map(str::trim).filter(|id| !id.is_empty());
    if link.dialogue.session_id.is_none() {
        let opened = link.dialogue.ensure(saved, deadline(job.budget))?;
        if let Some(id) = &link.dialogue.session_id {
            fs::write(job.state.join("session-id"), id)?;
        }
        link.opened = opened;
    }
    link.dialogue
        .prompt(&job.text, deadline(job.budget), &job.token.cancelled)
        .map(|mut turn| {
            turn.opened = link.opened.clone();
            turn.session_id = link.dialogue.session_id.clone().unwrap_or_default();
            turn
        })
}

fn publish_layout(state: &Path, turn: &Turn, prompt: &str) -> Result<()> {
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
            "pane_id": Value::Null,
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

struct Link {
    child: Child,
    dialogue: Dialogue<std::process::ChildStdin>,
    opened: String,
}

impl Link {
    fn spawn(bin: &Path, state: &Path, credential_root: &Path) -> Result<Self> {
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
            .env(
                "HCTL2_CONFINE_ALLOW",
                format!("{}\n{}", grok_home.display(), state.display()),
            )
            .env("HCTL2_CONFINE_READ", parent)
            .current_dir(state)
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
        };
        link.dialogue
            .handshake(Instant::now() + Duration::from_secs(30))?;
        Ok(link)
    }

    fn close(&mut self) {
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
        if answer.is_empty() {
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
        writeln!(self.writer, "{body}")?;
        self.writer.flush()?;
        let mut notified = false;
        loop {
            if Instant::now() >= deadline {
                return Err(PortError::new(
                    "HARNESS_TURN_ERROR",
                    format!("{method} exceeded the turn budget"),
                    "retry_dispatch",
                ));
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
                if let Some(error) = message.get("error") {
                    return Err(PortError::invalid(format!(
                        "{method} error {}: {}",
                        error["code"], error["message"]
                    )));
                }
                return Ok(message["result"].clone());
            }
        }
    }
}

fn session_id(created: &Value) -> Result<String> {
    created["sessionId"]
        .as_str()
        .filter(|id| !id.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| PortError::invalid("session/new returned no session id"))
}

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
        publish_layout(&root, &turn, "one\n!/two").unwrap();
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
}
