//! One Antigravity CLI (`agy`) stream-json process per selection record.
//! Prompts are sent as `{"event":"user","message":{"content":...}}` frames on
//! stdin; `result` frames on stdout end a turn after verifying the exact prompt
//! in `agy`'s own `transcript.jsonl`.
use crate::runtime::{Running, RuntimeEvent, Session};
use agency_proto::context::Bundle;
use agency_proto::{EvidenceLevel, ExecutionSpec, PortError, Result, Sealed, hash};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader, Read, Write},
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

const LIMIT: u64 = 16 * 1024 * 1024;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct ProcessIdentity {
    pid: u32,
    started: String,
    command_digest: String,
}

fn process_identity(pid: u32) -> Result<Option<ProcessIdentity>> {
    if pid == 0 {
        return Ok(None);
    }
    let output = Command::new("/bin/ps")
        .args([
            "-ww",
            "-p",
            &pid.to_string(),
            "-o",
            "lstart=,pgid=,stat=,command=",
        ])
        .env("LC_ALL", "C")
        .env("TZ", "UTC")
        .output()?;
    if !output.status.success() {
        if output.status.code() == Some(1) && output.stdout.is_empty() && output.stderr.is_empty() {
            return Ok(None);
        }
        return Err(PortError::new(
            "AGY_IDENTITY_UNAVAILABLE",
            "ps could not read the agy process identity",
            "inspect_agency_processes",
        ));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut fields = text.split_whitespace();
    let started = fields.by_ref().take(5).collect::<Vec<_>>().join(" ");
    let group = fields.next().and_then(|value| value.parse::<u32>().ok());
    let status = fields.next().unwrap_or("");
    let command = fields.collect::<Vec<_>>().join(" ");
    if group != Some(pid) || status.is_empty() || status.starts_with('Z') {
        return Ok(None);
    }
    Ok(Some(ProcessIdentity {
        pid,
        started,
        // Both sandbox-exec on macOS and agency --confine on Linux exec agy in
        // place; the stream-json flags tail is identical before and after exec.
        command_digest: hash(
            command
                .split_once(" --output-format stream-json")
                .map_or(command.as_str(), |(_, args)| args)
                .as_bytes(),
        ),
    }))
}

/// Reap any leftover `agy` processes recorded under this locked Agency root.
pub(crate) fn reap_previous(root: &Path) -> Result<()> {
    let dir = root.join("agy-processes");
    if !dir.exists() {
        return Ok(());
    }
    crate::storage::private_dir(&dir)?;
    for entry in fs::read_dir(&dir)? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let saved: ProcessIdentity = serde_json::from_slice(&fs::read(&path)?)?;
        if process_identity(saved.pid)?.as_ref() == Some(&saved) {
            signal_group(saved.pid, "TERM");
            let deadline = Instant::now() + Duration::from_millis(400);
            while Instant::now() < deadline && process_identity(saved.pid)?.as_ref() == Some(&saved)
            {
                thread::sleep(Duration::from_millis(20));
            }
            if process_identity(saved.pid)?.as_ref() == Some(&saved) {
                signal_group(saved.pid, "KILL");
                let deadline = Instant::now() + Duration::from_secs(2);
                while Instant::now() < deadline
                    && process_identity(saved.pid)?.as_ref() == Some(&saved)
                {
                    thread::sleep(Duration::from_millis(20));
                }
                if process_identity(saved.pid)?.as_ref() == Some(&saved) {
                    return Err(PortError::new(
                        "AGY_REAP_FAILED",
                        "previous agy stream-json process did not stop",
                        "inspect_agency_processes",
                    ));
                }
            }
        }
        fs::remove_file(path)?;
    }
    Ok(())
}

struct AgyProcess {
    child: Child,
    record: Option<PathBuf>,
}

impl Drop for AgyProcess {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(None)) {
            signal_group(self.child.id(), "KILL");
        }
        let _ = self.child.wait();
        if let Some(path) = &self.record {
            let _ = fs::remove_file(path);
        }
    }
}

impl AgyProcess {
    fn record(&mut self, root: &Path) -> Result<()> {
        let identity = process_identity(self.child.id())?
            .ok_or_else(|| PortError::invalid("agy process identity unavailable"))?;
        let dir = root.join("agy-processes");
        crate::storage::private_dir(&dir)?;
        let path = dir.join(format!("{}.json", self.child.id()));
        let mut file = tempfile::NamedTempFile::new_in(&dir)?;
        file.write_all(&serde_json::to_vec(&identity)?)?;
        file.as_file().sync_all()?;
        file.persist(&path).map_err(|error| error.error)?;
        self.record = Some(path);
        Ok(())
    }
}

struct Token {
    writing: bool,
    cancelled: AtomicBool,
    finished: AtomicBool,
    notify: Mutex<Option<mpsc::Sender<Message>>>,
}

impl Token {
    fn request_stop(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        if self.writing
            && let Some(tx) = self.notify.lock().expect("agy notify").as_ref()
        {
            let _ = tx.send(Message::CloseSession);
        }
    }
}

struct Handle {
    token: Arc<Token>,
}

impl Drop for Handle {
    fn drop(&mut self) {
        if !self.token.finished.load(Ordering::SeqCst) {
            self.token.request_stop();
        }
    }
}

impl Session for Handle {
    fn input(&mut self, _: &[u8]) -> Result<()> {
        Err(PortError::invalid(
            "this session does not accept terminal input",
        ))
    }

    fn stop(&mut self) -> Result<()> {
        self.token.request_stop();
        Ok(())
    }
}

struct Job {
    spec: Sealed<ExecutionSpec>,
    model: String,
    text: String,
    copy: Option<crate::write::WorkCopy>,
    credential_root: PathBuf,
    token: Arc<Token>,
    tx: mpsc::SyncSender<RuntimeEvent>,
}

enum Message {
    Job(Box<Job>),
    CloseSession,
    Shutdown(mpsc::SyncSender<Result<()>>),
}

struct Sender {
    tx: mpsc::Sender<Message>,
    stopped: Arc<AtomicBool>,
}

pub(crate) struct Pool {
    program: PathBuf,
    idle: Duration,
    slots: Mutex<HashMap<String, Sender>>,
}

impl Pool {
    pub(crate) fn new(program: PathBuf, idle: Duration) -> Self {
        Self {
            program,
            idle,
            slots: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn submit(
        &self,
        state_root: &Path,
        spec: &Sealed<ExecutionSpec>,
        bundle: &Sealed<Bundle>,
        exec_root: &Path,
        tenant: &Path,
        credential_root: &Path,
    ) -> Result<Running> {
        crate::confine::reject_unenforceable_isolation(
            &spec.document.required_capabilities.isolation_effects,
        )?;
        let copy = crate::write::prepare(
            &spec.document,
            &bundle.document,
            exec_root,
            tenant,
            credential_root,
        )?;
        let text = crate::write::task_text(&bundle.document, copy.as_ref())?;
        let key = hash(
            json!({
                "tenant": tenant,
                "binding": spec.document.binding.id,
                "project": spec.document.project.id,
                "selection": spec.document.selection.id,
            })
            .to_string()
            .as_bytes(),
        );
        let token = Arc::new(Token {
            writing: copy.is_some(),
            cancelled: AtomicBool::new(false),
            finished: AtomicBool::new(false),
            notify: Mutex::new(None),
        });
        let (tx, rx) = mpsc::sync_channel(64);
        let job = Box::new(Job {
            spec: spec.clone(),
            model: spec.document.profession.model.clone(),
            text,
            copy,
            credential_root: credential_root.to_path_buf(),
            token: Arc::clone(&token),
            tx,
        });
        let mut slots = self.slots.lock().expect("agy slots");
        if !slots.contains_key(&key) {
            let sender = self.spawn_slot(state_root, &key, exec_root)?;
            slots.insert(key.clone(), sender);
        }
        let sender = slots.get(&key).expect("inserted agy worker");
        *token.notify.lock().expect("agy notify") = Some(sender.tx.clone());
        sender
            .tx
            .send(Message::Job(job))
            .map_err(|_| PortError::new("STANDBY_UNAVAILABLE", "agy queue closed", "retry"))?;
        Ok(Running {
            session: Arc::new(Mutex::new(Box::new(Handle { token }))),
            events: rx,
        })
    }

    pub(crate) fn shutdown(&self) -> Result<()> {
        let slots = std::mem::take(&mut *self.slots.lock().expect("agy slots"));
        let mut first = Ok(());
        for (_, sender) in slots {
            sender.stopped.store(true, Ordering::SeqCst);
            let (ack_tx, ack_rx) = mpsc::sync_channel(1);
            if sender.tx.send(Message::Shutdown(ack_tx)).is_ok()
                && let Ok(Err(error)) = ack_rx.recv_timeout(Duration::from_secs(10))
                && first.is_ok()
            {
                first = Err(error);
            }
        }
        first
    }

    fn spawn_slot(&self, state_root: &Path, key: &str, exec_root: &Path) -> Result<Sender> {
        let dir = state_root.join(format!("agy-{key}"));
        let profile = state_root.join("agy-home");
        let cwd = state_root.join(format!("agy-cwd-{key}"));
        crate::storage::private_dir(&dir)?;
        crate::storage::private_dir(&profile)?;
        crate::storage::private_dir(&cwd)?;
        let _ = exec_root;
        let program = self.program.clone();
        let idle = self.idle;
        let stopped = Arc::new(AtomicBool::new(false));
        let worker_stopped = Arc::clone(&stopped);
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = worker(program, dir, profile, cwd, idle, worker_stopped, rx);
        });
        Ok(Sender { tx, stopped })
    }
}

fn worker(
    agy: PathBuf,
    dir: PathBuf,
    profile: PathBuf,
    cwd: PathBuf,
    idle: Duration,
    stopped: Arc<AtomicBool>,
    receiver: mpsc::Receiver<Message>,
) -> Result<()> {
    let mut live: Option<Live> = None;
    let mut returned: Option<Job> = None;
    let mut last = Instant::now();
    loop {
        let wait = if live.is_some() {
            idle.saturating_sub(last.elapsed())
        } else {
            Duration::from_secs(3600)
        };
        let message = match receiver.recv_timeout(wait) {
            Ok(message) => message,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Some(mut session) = live.take() {
                    session.close()?;
                }
                release_returned(&mut returned, false);
                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
        let job = match message {
            Message::Job(job) => {
                if returned.is_some() {
                    if let Some(mut session) = live.take() {
                        session.close()?;
                    }
                    release_returned(&mut returned, false);
                }
                job
            }
            Message::CloseSession => {
                if let Some(mut session) = live.take() {
                    session.close()?;
                }
                release_returned(&mut returned, true);
                continue;
            }
            Message::Shutdown(ack) => {
                let result = if let Some(mut session) = live.take() {
                    session.close()
                } else {
                    Ok(())
                };
                release_returned(&mut returned, true);
                let _ = ack.send(result);
                return Ok(());
            }
        };
        let result = run_job(
            &TurnEnv {
                agy: &agy,
                dir: &dir,
                profile: &profile,
                cwd: &cwd,
            },
            &stopped,
            &mut live,
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
            if let Some(mut session) = live.take() {
                session.close()?;
                if job.copy.is_some() {
                    let _ = job.tx.send(RuntimeEvent::TurnStopped {
                        requested_stop: job.token.cancelled.load(Ordering::SeqCst)
                            || stopped.load(Ordering::SeqCst),
                        session_closed: true,
                    });
                }
            }
        }
        if sealed {
            returned = Some(*job);
        } else {
            if job.copy.is_some()
                && let Some(mut session) = live.take()
            {
                session.close()?;
                let _ = job.tx.send(RuntimeEvent::TurnStopped {
                    requested_stop: job.token.cancelled.load(Ordering::SeqCst)
                        || stopped.load(Ordering::SeqCst),
                    session_closed: true,
                });
            }
            let _ = job.tx.send(RuntimeEvent::DispatchReleased);
        }
        last = Instant::now();
    }
    if let Some(mut session) = live.take() {
        session.close()?;
    }
    release_returned(&mut returned, true);
    for message in receiver.try_iter() {
        if let Message::Job(job) = message {
            job.token.finished.store(true, Ordering::SeqCst);
            let _ = job.tx.send(RuntimeEvent::TurnStopped {
                requested_stop: true,
                session_closed: false,
            });
            let _ = job.tx.send(RuntimeEvent::DispatchReleased);
        }
    }
    Ok(())
}

fn release_returned(returned: &mut Option<Job>, requested_stop: bool) {
    if let Some(job) = returned.take() {
        let _ = job.tx.send(RuntimeEvent::TurnStopped {
            requested_stop,
            session_closed: true,
        });
        let _ = job.tx.send(RuntimeEvent::DispatchReleased);
    }
}

struct TurnEnv<'a> {
    agy: &'a Path,
    dir: &'a Path,
    profile: &'a Path,
    cwd: &'a Path,
}

fn model_arg(model: &str) -> Option<String> {
    let trimmed = model.trim();
    if trimmed.is_empty() || trimmed == "none" {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

fn run_job(
    env: &TurnEnv<'_>,
    stopped: &AtomicBool,
    live: &mut Option<Live>,
    job: &Job,
) -> Result<()> {
    crate::confine::reject_unenforceable_isolation(
        &job.spec.document.required_capabilities.isolation_effects,
    )?;
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
    let cwd = job.copy.as_ref().map_or(env.cwd, |copy| copy.cwd.as_path());
    let model = model_arg(&job.model);
    if live
        .as_ref()
        .is_some_and(|s| s.cwd != cwd || s.writing != job.copy.is_some() || s.model != model)
        && let Some(mut prev) = live.take()
    {
        prev.close()?;
    }
    if live.is_none() {
        let (session, resumed, resume_failed) = Live::open(
            env.agy,
            cwd,
            env.dir,
            env.profile,
            &job.credential_root,
            job.copy.is_some(),
            &model,
        )?;
        let _ = job.tx.send(RuntimeEvent::Observation {
            kind: "session_opened".into(),
            payload: json!({
                "resumed": resumed,
                "resume_failed": resume_failed,
                "conversation": session.conversation,
                "file": session.transcript.display().to_string(),
                "pane": Value::Null,
            }),
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
    let session = live.as_mut().expect("agy session");
    session.send_user(&job.text)?;
    let mut interrupt: Option<(Instant, bool)> = None;
    loop {
        let requested =
            job.token.cancelled.load(Ordering::SeqCst) || stopped.load(Ordering::SeqCst);
        let expired = crate::storage::now_ms() >= job.spec.document.deadline_ms;
        if interrupt.is_none() && (requested || expired) {
            if expired {
                let _ = job.tx.send(RuntimeEvent::DeadlineReached);
            }
            session.interrupt();
            interrupt = Some((Instant::now(), requested));
        }
        match session.poll(Duration::from_millis(200))? {
            Poll::Frame(msg) => {
                // Subagent steps emit step_update frames with subagent_info; only
                // top-level result frames for this conversation end the turn.
                if msg["event"] == "result"
                    && msg["result"]["conversation_id"].as_str()
                        == Some(session.conversation.as_str())
                {
                    if let Some((_, requested_stop)) = interrupt {
                        let session_closed = job.copy.is_some();
                        if let Some(mut closed_session) = live.take() {
                            closed_session.close()?;
                        }
                        let _ = job.tx.send(RuntimeEvent::TurnStopped {
                            requested_stop,
                            session_closed,
                        });
                        return Ok(());
                    }
                    match msg["result"]["status"].as_str() {
                        Some("SUCCESS") => {
                            let text = msg["result"]["response"]
                                .as_str()
                                .filter(|s| !s.is_empty())
                                .ok_or_else(|| {
                                    PortError::invalid("agy turn completed without a response")
                                })?
                                .to_owned();
                            transcript_matches(&session.transcript, &job.text)?;
                            let (schema, bytes) = if let Some(copy) = &job.copy {
                                let output = crate::write::seal_return(
                                    copy,
                                    text.as_bytes(),
                                    job.spec.document.deadline_ms,
                                    &job.tx,
                                    || {
                                        job.token.cancelled.load(Ordering::SeqCst)
                                            || stopped.load(Ordering::SeqCst)
                                    },
                                )?;
                                let Some(output) = output else {
                                    if let Some(mut closed_session) = live.take() {
                                        closed_session.close()?;
                                    }
                                    let _ = job.tx.send(RuntimeEvent::TurnStopped {
                                        requested_stop: job.token.cancelled.load(Ordering::SeqCst)
                                            || stopped.load(Ordering::SeqCst),
                                        session_closed: true,
                                    });
                                    return Ok(());
                                };
                                output
                            } else {
                                ("agy.turn.v1".into(), text.into_bytes())
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
                        Some("ERROR") => {
                            let err_text = msg["result"]["error"]
                                .as_str()
                                .or_else(|| msg["result"]["response"].as_str())
                                .unwrap_or("agy ended this turn with an error");
                            if is_login_error(err_text) {
                                return Err(login_required_error(env.profile));
                            }
                            return Err(PortError::new(
                                "HARNESS_TURN_ERROR",
                                err_text,
                                "inspect_dispatch",
                            ));
                        }
                        other => {
                            return Err(PortError::new(
                                "AGY_PROTOCOL",
                                format!("unknown agy result status {other:?}"),
                                "inspect_agy_session",
                            ));
                        }
                    }
                }
            }
            Poll::Timeout => {}
            Poll::Closed => {
                if let Some((_, requested_stop)) = interrupt {
                    let session_closed = job.copy.is_some();
                    if let Some(mut closed_session) = live.take() {
                        closed_session.close()?;
                    }
                    let _ = job.tx.send(RuntimeEvent::TurnStopped {
                        requested_stop,
                        session_closed,
                    });
                    return Ok(());
                }
                return Err(PortError::new(
                    "AGY_STREAM_EXITED",
                    "agy closed the stream-json output before a result frame",
                    "inspect_agy_session",
                ));
            }
        }
        if let Some((sent, requested_stop)) = interrupt
            && sent.elapsed() >= Duration::from_secs(3)
        {
            if let Some(mut closed_session) = live.take() {
                closed_session.close()?;
            }
            let _ = job.tx.send(RuntimeEvent::TurnStopped {
                requested_stop,
                session_closed: true,
            });
            return Ok(());
        }
    }
}

fn is_login_error(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("authentication required")
        || lower.contains("agy login")
        || lower.contains("login_required")
        || lower.contains("unauthenticated")
}

fn login_required_error(profile: &Path) -> PortError {
    PortError::new(
        "HARNESS_LOGIN_REQUIRED",
        format!(
            "agy requires login inside the managed profile; run `HOME={} agy login` for this tenant",
            profile.display()
        ),
        "sign_in_agy",
    )
}

fn prepare_profile(profile: &Path, writing: bool) -> Result<()> {
    let gemini = profile.join(".gemini");
    let agy_dir = gemini.join("antigravity-cli");
    let config_dir = gemini.join("config");
    crate::storage::private_dir(profile)?;
    crate::storage::private_dir(&gemini)?;
    crate::storage::private_dir(&agy_dir)?;
    crate::storage::private_dir(&config_dir)?;
    let token_path = agy_dir.join("antigravity-oauth-token");
    let token = fs::read_to_string(&token_path).unwrap_or_default();
    if token.trim().is_empty() || token.trim() == "login_required" {
        return Err(login_required_error(profile));
    }
    let settings = if writing {
        json!({
            "toolPermission": "always-proceed",
            "allowNonWorkspaceAccess": false,
            "permissions": {
                "deny": [
                    "Bash(git push*)",
                    "Bash(gh*)",
                    "Bash(git credential*)"
                ]
            }
        })
    } else {
        json!({
            "toolPermission": "request-review",
            "allowNonWorkspaceAccess": false
        })
    };
    let hooks = if writing {
        json!({
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "run_command",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "/bin/sh -c 'input=$(cat); case \"$input\" in *\"git push\"*|*\"git credential\"*|*\"\\\"gh \"*|*\"/gh \"*) printf \"%s\\n\" \"{\\\"decision\\\":\\\"deny\\\",\\\"reason\\\":\\\"publication and credential commands are denied\\\"}\" ;; *) printf \"%s\\n\" \"{\\\"decision\\\":\\\"allow\\\"}\" ;; esac'"
                            }
                        ]
                    }
                ]
            }
        })
    } else {
        json!({
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "write_to_file|replace_file_content|multi_replace_file_content|sed_file|notebook_edit|run_command",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "/bin/sh -c 'printf \"%s\\n\" \"{\\\"decision\\\":\\\"deny\\\",\\\"reason\\\":\\\"read-only dispatch\\\"}\"'"
                            }
                        ]
                    }
                ]
            }
        })
    };
    write_private_json(&agy_dir.join("settings.json"), &settings)?;
    write_private_json(&config_dir.join("hooks.json"), &hooks)?;
    Ok(())
}

fn write_private_json(path: &Path, value: &Value) -> Result<()> {
    let mut file = crate::storage::private_file(path)?;
    file.set_len(0)?;
    file.write_all(&serde_json::to_vec(value)?)?;
    file.sync_all()?;
    Ok(())
}

enum Poll {
    Frame(Value),
    Timeout,
    Closed,
}

struct Live {
    process: AgyProcess,
    stdin: Option<ChildStdin>,
    frames: mpsc::Receiver<Result<Value>>,
    reader: Option<thread::JoinHandle<()>>,
    conversation: String,
    transcript: PathBuf,
    closed: bool,
    writing: bool,
    model: Option<String>,
    cwd: PathBuf,
}

impl Live {
    fn open(
        agy: &Path,
        cwd: &Path,
        dir: &Path,
        profile: &Path,
        credential_root: &Path,
        writing: bool,
        model: &Option<String>,
    ) -> Result<(Self, bool, bool)> {
        prepare_profile(profile, writing)?;
        let saved = read_conversation(dir)?;
        if let Some(saved_id) = &saved {
            match Self::spawn_one(
                agy,
                cwd,
                dir,
                profile,
                credential_root,
                writing,
                model,
                Some(saved_id),
            ) {
                Ok(live) if live.conversation == *saved_id => {
                    write_conversation(dir, &live.conversation)?;
                    return Ok((live, true, false));
                }
                Ok(mut live) => {
                    live.close()?;
                }
                Err(_) => {}
            }
            let _ = fs::remove_file(dir.join("conversation.id"));
            let live = Self::spawn_one(
                agy,
                cwd,
                dir,
                profile,
                credential_root,
                writing,
                model,
                None,
            )?;
            write_conversation(dir, &live.conversation)?;
            return Ok((live, false, true));
        }
        let live = Self::spawn_one(
            agy,
            cwd,
            dir,
            profile,
            credential_root,
            writing,
            model,
            None,
        )?;
        write_conversation(dir, &live.conversation)?;
        Ok((live, false, false))
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn_one(
        agy: &Path,
        cwd: &Path,
        dir: &Path,
        profile: &Path,
        credential_root: &Path,
        writing: bool,
        model: &Option<String>,
        resume: Option<&str>,
    ) -> Result<Self> {
        let log = crate::storage::private_file(&dir.join("agy.err"))?;
        let mut args = vec![
            "--output-format".to_owned(),
            "stream-json".to_owned(),
            "--input-format".to_owned(),
            "stream-json".to_owned(),
            "--disable-slash-commands".to_owned(),
            "--print=".to_owned(),
        ];
        if let Some(model) = model {
            args.push("--model".into());
            args.push(model.clone());
        }
        if let Some(conversation) = resume {
            args.push("--conversation".into());
            args.push(conversation.to_owned());
        }
        let mut cmd =
            crate::confine::agy_command(agy, &args, cwd, dir, profile, credential_root, writing)?;
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::from(log))
            .current_dir(cwd)
            .process_group(0);
        let mut child = cmd.spawn()?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| PortError::invalid("agy stdin unavailable"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| PortError::invalid("agy stdout unavailable"))?;
        let mut process = AgyProcess {
            child,
            record: None,
        };
        process.record(credential_root)?;
        let (tx, rx) = mpsc::channel();
        let reader = thread::spawn(move || read_frames(stdout, tx));
        let mut live = Self {
            process,
            stdin: Some(stdin),
            frames: rx,
            reader: Some(reader),
            conversation: String::new(),
            transcript: PathBuf::new(),
            closed: false,
            writing,
            model: model.clone(),
            cwd: cwd.to_path_buf(),
        };
        if let Err(error) = live.wait_init(profile) {
            let _ = live.close();
            return Err(error);
        }
        Ok(live)
    }

    fn wait_init(&mut self, profile: &Path) -> Result<()> {
        let deadline = Instant::now() + Duration::from_secs(20);
        while Instant::now() < deadline {
            if self.process.child.try_wait()?.is_some() {
                return Err(PortError::new(
                    "AGY_STREAM_EXITED",
                    "agy exited before emitting the init frame",
                    "inspect_agy_session",
                ));
            }
            match self.poll(Duration::from_millis(100))? {
                Poll::Frame(value) if value["event"] == "init" => {
                    let id = value["conversation_id"]
                        .as_str()
                        .filter(|id| !id.is_empty())
                        .ok_or_else(|| {
                            PortError::new(
                                "AGY_PROTOCOL",
                                "agy init frame had no conversation_id",
                                "upgrade_agy",
                            )
                        })?;
                    self.conversation = id.to_owned();
                    self.transcript = profile
                        .join(".gemini/antigravity-cli/brain")
                        .join(id)
                        .join(".system_generated/logs/transcript.jsonl");
                    return Ok(());
                }
                Poll::Frame(_) | Poll::Timeout => {}
                Poll::Closed => {
                    return Err(PortError::new(
                        "AGY_STREAM_EXITED",
                        "agy closed stdout before emitting the init frame",
                        "inspect_agy_session",
                    ));
                }
            }
        }
        Err(PortError::new(
            "AGY_STREAM_EXITED",
            "agy did not emit an init frame before timeout",
            "inspect_agy_session",
        ))
    }

    fn send_user(&mut self, text: &str) -> Result<()> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| PortError::invalid("agy stdin is closed"))?;
        let frame = json!({
            "event": "user",
            "message": {
                "content": text,
            }
        });
        serde_json::to_writer(&mut *stdin, &frame)?;
        stdin.write_all(b"\n")?;
        stdin.flush()?;
        Ok(())
    }

    fn interrupt(&self) {
        signal_group(self.process.child.id(), "INT");
    }

    fn poll(&mut self, wait: Duration) -> Result<Poll> {
        match self.frames.recv_timeout(wait) {
            Ok(Ok(value)) => Ok(Poll::Frame(value)),
            Ok(Err(error)) => Err(error),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(Poll::Timeout),
            Err(mpsc::RecvTimeoutError::Disconnected) => Ok(Poll::Closed),
        }
    }

    fn close(&mut self) -> Result<()> {
        if self.closed {
            return Ok(());
        }
        self.closed = true;
        drop(self.stdin.take());
        let deadline = Instant::now() + Duration::from_millis(400);
        while Instant::now() < deadline {
            if self.process.child.try_wait()?.is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        if self.process.child.try_wait()?.is_none() {
            signal_group(self.process.child.id(), "TERM");
            let deadline = Instant::now() + Duration::from_millis(400);
            while Instant::now() < deadline {
                if self.process.child.try_wait()?.is_some() {
                    break;
                }
                thread::sleep(Duration::from_millis(20));
            }
        }
        if self.process.child.try_wait()?.is_none() {
            signal_group(self.process.child.id(), "KILL");
            let _ = self.process.child.wait();
        }
        if let Some(path) = self.process.record.take() {
            let _ = fs::remove_file(path);
        }
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
        Ok(())
    }
}

impl Drop for Live {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

fn read_frames(stdout: std::process::ChildStdout, tx: mpsc::Sender<Result<Value>>) {
    let mut reader = BufReader::new(stdout);
    loop {
        let mut line = Vec::new();
        match reader.by_ref().take(LIMIT + 1).read_until(b'\n', &mut line) {
            Ok(0) => return,
            Ok(_) if line.len() as u64 > LIMIT => {
                let _ = tx.send(Err(PortError::new(
                    "AGY_PROTOCOL",
                    "agy stream-json frame exceeds 16 MiB",
                    "inspect_agy_session",
                )));
                return;
            }
            Ok(_) => {
                let trimmed = String::from_utf8_lossy(&line);
                let trimmed = trimmed.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match serde_json::from_str::<Value>(trimmed) {
                    Ok(value) => {
                        if tx.send(Ok(value)).is_err() {
                            return;
                        }
                    }
                    Err(error) => {
                        let _ = tx.send(Err(PortError::new(
                            "AGY_PROTOCOL",
                            format!("invalid agy stream-json frame: {error}"),
                            "inspect_agy_session",
                        )));
                        return;
                    }
                }
            }
            Err(error) => {
                let _ = tx.send(Err(error.into()));
                return;
            }
        }
    }
}

fn read_conversation(dir: &Path) -> Result<Option<String>> {
    match fs::read_to_string(dir.join("conversation.id")) {
        Ok(id) => Ok(Some(id.trim().to_owned()).filter(|id| !id.is_empty())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn write_conversation(dir: &Path, conversation: &str) -> Result<()> {
    let mut file = crate::storage::private_file(&dir.join("conversation.id"))?;
    file.set_len(0)?;
    file.write_all(conversation.as_bytes())?;
    file.sync_all()?;
    Ok(())
}

fn signal_group(pid: u32, signal: &str) {
    let _ = Command::new("/bin/kill")
        .arg(format!("-{signal}"))
        .arg(format!("-{pid}"))
        .status();
}

pub(crate) fn transcript_matches(file: &Path, text: &str) -> Result<()> {
    if !file.is_file() {
        return Err(PortError::new(
            "AGY_TRANSCRIPT_MISSING",
            "agy recorded no transcript.jsonl for this conversation",
            "inspect_agy_session",
        ));
    }
    if transcript_file_has_input(file, text)? {
        Ok(())
    } else {
        Err(PortError::new(
            "STANDBY_PROMPT_MISMATCH",
            "agy transcript user input does not match this dispatch",
            "inspect_dispatch",
        ))
    }
}

pub(crate) fn transcript_file_has_input(file: &Path, text: &str) -> Result<bool> {
    let mut handle = fs::File::open(file)?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut handle)
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > LIMIT {
        return Err(PortError::invalid("agy transcript exceeds read limit"));
    }
    for line in String::from_utf8_lossy(&bytes).lines() {
        if !line.contains("USER_INPUT") {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if value["type"] == "USER_INPUT"
            && let Some(content) = value["content"].as_str()
            && user_input_matches(content, text)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn user_input_matches(content: &str, expected: &str) -> bool {
    if content == expected {
        return true;
    }
    content
        .strip_prefix("<USER_REQUEST>\n")
        .and_then(|rest| rest.strip_prefix(expected))
        .is_some_and(|tail| tail == "\n</USER_REQUEST>" || tail.starts_with("\n</USER_REQUEST>\n"))
}
