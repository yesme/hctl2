//! One Antigravity CLI (`agy`) stream-json process per selection record.
//! Prompts are sent as `{"event":"user","message":{"content":...}}` frames on
//! stdin; `result` frames on stdout end a turn after verifying the exact prompt
//! in `agy`'s own `transcript.jsonl`.
use crate::{
    observation::{Multiplexer, ObservationConfig, ObservationPane, TURN_OUTPUT_SCHEMA},
    runtime::{Running, RuntimeEvent, Session},
};
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
    observation: ObservationConfig,
    mux: Option<Multiplexer>,
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

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn submit(
        &self,
        state_root: &Path,
        spec: &Sealed<ExecutionSpec>,
        bundle: &Sealed<Bundle>,
        exec_root: &Path,
        tenant: &Path,
        credential_root: &Path,
        observation: ObservationConfig,
        mux: Option<Multiplexer>,
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
            observation,
            mux,
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
        let harness_dir = state_root.join("antigravity");
        let dir = harness_dir.join(key);
        let profile = state_root.join("agy-home");
        let cwd = state_root.join(format!("agy-cwd-{key}"));
        crate::storage::private_dir(&harness_dir)?;
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
    if live.as_mut().is_some_and(|s| {
        s.cwd != cwd || s.writing != job.copy.is_some() || s.model != model || !s.is_alive()
    }) && let Some(mut prev) = live.take()
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
            job.observation,
            job.mux.clone(),
        )?;
        let pane_val = session
            .pane
            .as_ref()
            .and_then(|p| p.pane_id())
            .map_or(Value::Null, |id| Value::String(id.to_owned()));
        let _ = job.tx.send(RuntimeEvent::Observation {
            kind: "session_opened".into(),
            payload: json!({
                "resumed": resumed,
                "resume_failed": resume_failed,
                "conversation": session.conversation,
                "file": session.transcript.display().to_string(),
                "session_dir": env.dir.display().to_string(),
                "pane": pane_val,
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
    session.turn += 1;
    session.sync_state()?;
    if let Some(pane) = &session.pane {
        pane.append_turn_start(session.turn)?;
        pane.append_prompt_summary(&job.text)?;
    }
    let transcript_offset = match fs::metadata(&session.transcript) {
        Ok(meta) => meta.len(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
        Err(error) => return Err(error.into()),
    };
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
                if msg["event"] == "step_update"
                    && let Some(calls) = msg
                        .pointer("/step_update/tool_calls")
                        .and_then(Value::as_array)
                    && let Some(pane) = &session.pane
                {
                    for call in calls {
                        if let Some(name) = call["name"].as_str() {
                            let detail = call.get("arguments").map(Value::to_string);
                            pane.append_tool_call(name, detail.as_deref())?;
                        }
                    }
                }
                // Subagent steps emit step_update frames with subagent_info; only
                // top-level result frames for this conversation end the turn.
                if msg["event"] == "result"
                    && msg["result"]["conversation_id"].as_str()
                        == Some(session.conversation.as_str())
                {
                    if let Some((_, requested_stop)) = interrupt {
                        if let Some(pane) = &session.pane {
                            let _ = pane.append_turn_end(session.turn, "stopped");
                        }
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
                            transcript_matches_since(
                                &session.transcript,
                                transcript_offset,
                                &job.text,
                            )?;
                            if let Some(pane) = &session.pane {
                                pane.append_answer(&text)?;
                                pane.append_turn_end(session.turn, "completed")?;
                            }
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
                                (TURN_OUTPUT_SCHEMA.into(), text.into_bytes())
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
                            if let Some(pane) = &session.pane {
                                let _ = pane.append_turn_end(session.turn, "error");
                            }
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
                    if let Some(pane) = &session.pane {
                        let _ = pane.append_turn_end(session.turn, "stopped");
                    }
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
            if let Some(pane) = &session.pane {
                let _ = pane.append_turn_end(session.turn, "stopped");
            }
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

pub(crate) fn slot_home(dir: &Path) -> PathBuf {
    dir.join("home")
}

fn read_managed_oauth_token(auth_profile: &Path) -> Result<String> {
    let token_path = auth_profile
        .join(".gemini/antigravity-cli")
        .join("antigravity-oauth-token");
    let meta = match fs::symlink_metadata(&token_path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(login_required_error(auth_profile));
        }
        Err(error) => return Err(error.into()),
    };
    if meta.file_type().is_symlink() || !meta.file_type().is_file() {
        return Err(PortError::invalid(
            "managed agy oauth token must be a regular file, not a symlink",
        ));
    }
    let token = fs::read_to_string(&token_path)?;
    if token.trim().is_empty() || token.trim() == "login_required" {
        return Err(login_required_error(auth_profile));
    }
    Ok(token)
}

fn prepare_profile(dir: &Path, auth_profile: &Path, writing: bool) -> Result<PathBuf> {
    let token = read_managed_oauth_token(auth_profile)?;
    let home = slot_home(dir);
    let gemini = home.join(".gemini");
    let agy_dir = gemini.join("antigravity-cli");
    let config_dir = gemini.join("config");
    crate::storage::private_dir(dir)?;
    crate::storage::private_dir(&home)?;
    crate::storage::private_dir(&gemini)?;
    crate::storage::private_dir(&agy_dir)?;
    crate::storage::private_dir(&config_dir)?;
    write_private_bytes(&agy_dir.join("antigravity-oauth-token"), token.as_bytes())?;
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
    Ok(home)
}

fn write_private_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = crate::storage::private_file(path)?;
    file.set_len(0)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn write_private_json(path: &Path, value: &Value) -> Result<()> {
    write_private_bytes(path, &serde_json::to_vec(value)?)
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
    eof: Arc<AtomicBool>,
    reader: Option<thread::JoinHandle<()>>,
    dir: PathBuf,
    pane: Option<ObservationPane>,
    resumed: bool,
    resume_failed: bool,
    turn: u64,
    conversation: String,
    transcript: PathBuf,
    closed: bool,
    writing: bool,
    model: Option<String>,
    cwd: PathBuf,
}

impl Live {
    #[allow(clippy::too_many_arguments)]
    fn open(
        agy: &Path,
        cwd: &Path,
        dir: &Path,
        auth_profile: &Path,
        credential_root: &Path,
        writing: bool,
        model: &Option<String>,
        observation: ObservationConfig,
        mux: Option<Multiplexer>,
    ) -> Result<(Self, bool, bool)> {
        let home = prepare_profile(dir, auth_profile, writing)?;
        let saved = read_saved_state(dir)?;
        let (mut live, resumed, resume_failed) = if let Some((saved_id, saved_turn)) = &saved {
            match Self::spawn_one(
                agy,
                cwd,
                dir,
                &home,
                credential_root,
                writing,
                model,
                Some(saved_id),
            ) {
                Ok(mut live) if live.conversation == *saved_id => {
                    live.turn = *saved_turn;
                    (live, true, false)
                }
                Ok(mut live) => {
                    live.close()?;
                    let _ = fs::remove_file(dir.join("state.json"));
                    let fresh = Self::spawn_one(
                        agy,
                        cwd,
                        dir,
                        &home,
                        credential_root,
                        writing,
                        model,
                        None,
                    )?;
                    (fresh, false, true)
                }
                Err(_) => {
                    let _ = fs::remove_file(dir.join("state.json"));
                    let fresh = Self::spawn_one(
                        agy,
                        cwd,
                        dir,
                        &home,
                        credential_root,
                        writing,
                        model,
                        None,
                    )?;
                    (fresh, false, true)
                }
            }
        } else {
            let fresh =
                Self::spawn_one(agy, cwd, dir, &home, credential_root, writing, model, None)?;
            (fresh, false, false)
        };
        live.resumed = resumed;
        live.resume_failed = resume_failed;
        let (pane, _) = ObservationPane::open(observation, dir, &live.conversation, || {
            mux.ok_or_else(|| PortError::invalid("observation multiplexer unavailable"))
        })?;
        if resume_failed {
            pane.append_tool_call("session", Some("resume_failed: started new conversation"))?;
        }
        live.pane = Some(pane);
        live.sync_state()?;
        Ok((live, resumed, resume_failed))
    }

    fn is_alive(&mut self) -> bool {
        !self.eof.load(Ordering::SeqCst) && matches!(self.process.child.try_wait(), Ok(None))
    }

    fn sync_state(&self) -> Result<()> {
        write_private_json(
            &self.dir.join("state.json"),
            &json!({
                "conversation_id": self.conversation,
                "pane_id": self.pane.as_ref().and_then(ObservationPane::pane_id),
                "resumed": self.resumed,
                "resume_failed": self.resume_failed,
                "turn": self.turn,
                "schema": TURN_OUTPUT_SCHEMA,
            }),
        )
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
        let eof = Arc::new(AtomicBool::new(false));
        let reader_eof = Arc::clone(&eof);
        let reader = thread::spawn(move || read_frames(stdout, tx, reader_eof));
        let mut live = Self {
            process,
            stdin: Some(stdin),
            frames: rx,
            eof,
            reader: Some(reader),
            dir: dir.to_path_buf(),
            pane: None,
            resumed: false,
            resume_failed: false,
            turn: 0,
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
        append_event(&self.dir, &frame)?;
        Ok(())
    }

    fn interrupt(&self) {
        signal_group(self.process.child.id(), "INT");
    }

    fn poll(&mut self, wait: Duration) -> Result<Poll> {
        match self.frames.recv_timeout(wait) {
            Ok(Ok(value)) => {
                append_event(&self.dir, &value)?;
                Ok(Poll::Frame(value))
            }
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
        if let Some(pane) = self.pane.take() {
            let _ = pane.close();
        }
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

fn read_frames(
    stdout: std::process::ChildStdout,
    tx: mpsc::Sender<Result<Value>>,
    eof: Arc<AtomicBool>,
) {
    struct EofGuard(Arc<AtomicBool>);
    impl Drop for EofGuard {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }
    let _guard = EofGuard(eof);
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

fn read_saved_state(dir: &Path) -> Result<Option<(String, u64)>> {
    match fs::read(dir.join("state.json")) {
        Ok(bytes) => {
            let value: Value = serde_json::from_slice(&bytes)?;
            let turn = value["turn"].as_u64().unwrap_or(0);
            Ok(value["conversation_id"]
                .as_str()
                .map(str::trim)
                .filter(|id| !id.is_empty())
                .map(|id| (id.to_owned(), turn)))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn append_event(dir: &Path, frame: &Value) -> Result<()> {
    let path = dir.join("events.jsonl");
    let _ = crate::storage::private_file(&path)?;
    let mut file = fs::OpenOptions::new().append(true).open(&path)?;
    serde_json::to_writer(&mut file, frame)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}

fn signal_group(pid: u32, signal: &str) {
    let _ = Command::new("/bin/kill")
        .args(["-s", signal, "--", &format!("-{pid}")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

pub(crate) fn transcript_matches_since(file: &Path, offset: u64, text: &str) -> Result<()> {
    if !file.is_file() {
        return Err(PortError::new(
            "AGY_TRANSCRIPT_MISSING",
            "agy recorded no transcript.jsonl for this conversation",
            "inspect_agy_session",
        ));
    }
    let matched = if offset == 0 {
        transcript_file_has_input(file, text)?
    } else {
        transcript_file_has_input_since(file, offset, text)?
    };
    if matched {
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
    transcript_file_has_input_since(file, 0, text)
}

pub(crate) fn transcript_file_has_input_since(
    file: &Path,
    offset: u64,
    text: &str,
) -> Result<bool> {
    use std::io::{Seek, SeekFrom};
    let mut handle = fs::File::open(file)?;
    let len = handle.metadata()?.len();
    if len < offset {
        return Ok(false);
    }
    handle.seek(SeekFrom::Start(offset))?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut handle)
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > LIMIT {
        return Err(PortError::invalid(
            "agy transcript turn slice exceeds read limit",
        ));
    }
    let slice = String::from_utf8_lossy(&bytes);
    let mut last_user_input: Option<Option<String>> = None;
    for line in slice.lines() {
        if !line.contains("USER_INPUT") {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if value["type"] == "USER_INPUT" {
            last_user_input = Some(value["content"].as_str().map(str::to_owned));
        }
    }
    Ok(matches!(
        last_user_input,
        Some(Some(ref content)) if user_input_matches(content, text)
    ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn write_transcript_lines(path: &Path, inputs: &[&str]) {
        let mut file = fs::File::create(path).unwrap();
        for (idx, text) in inputs.iter().enumerate() {
            let entry = json!({
                "step_index": idx + 1,
                "source": "USER_EXPLICIT",
                "type": "USER_INPUT",
                "status": "DONE",
                "content": format!("<USER_REQUEST>\n{text}\n</USER_REQUEST>\n<ADDITIONAL_METADATA>\ntest\n</ADDITIONAL_METADATA>")
            });
            writeln!(file, "{entry}").unwrap();
        }
        file.sync_all().unwrap();
    }

    #[test]
    fn old_equal_input_cannot_certify_a_mismatched_current_turn_and_matching_repeat_passes() {
        let dir = tempfile::tempdir().unwrap();
        let mismatched = dir.path().join("mismatched.jsonl");
        write_transcript_lines(
            &mismatched,
            &["repeat this task", "repeat this task WITH EXTRA INPUT"],
        );
        assert!(!transcript_file_has_input(&mismatched, "repeat this task").unwrap());

        let matched = dir.path().join("matched.jsonl");
        write_transcript_lines(&matched, &["repeat this task"]);
        let offset = fs::metadata(&matched).unwrap().len();
        // Without a new USER_INPUT appended in the current turn after `offset`,
        // the historical turn alone cannot certify the new turn.
        assert!(!transcript_file_has_input_since(&matched, offset, "repeat this task").unwrap());

        // Appending a mismatched turn after `offset` is rejected.
        write_transcript_lines(
            &matched,
            &["repeat this task", "repeat this task WITH EXTRA INPUT"],
        );
        assert!(!transcript_file_has_input_since(&matched, offset, "repeat this task").unwrap());

        // Appending an identical USER_INPUT in the current turn after `offset` passes.
        write_transcript_lines(&matched, &["repeat this task", "repeat this task"]);
        assert!(transcript_file_has_input(&matched, "repeat this task").unwrap());
        assert!(transcript_file_has_input_since(&matched, offset, "repeat this task").unwrap());
    }

    #[test]
    fn transcript_verification_since_offset_handles_history_exceeding_16_mib() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("large-history.jsonl");
        {
            let mut file = fs::File::create(&path).unwrap();
            let filler = "h".repeat(1024 * 1024);
            for idx in 0..17 {
                let entry = json!({
                    "step_index": idx,
                    "source": "USER_EXPLICIT",
                    "type": "USER_INPUT",
                    "status": "DONE",
                    "content": filler,
                });
                writeln!(file, "{entry}").unwrap();
            }
            file.sync_all().unwrap();
        }
        let offset = fs::metadata(&path).unwrap().len();
        assert!(offset > LIMIT);
        {
            let mut file = fs::OpenOptions::new().append(true).open(&path).unwrap();
            let entry = json!({
                "step_index": 18,
                "source": "USER_EXPLICIT",
                "type": "USER_INPUT",
                "status": "DONE",
                "content": "<USER_REQUEST>\npost-limit turn\n</USER_REQUEST>",
            });
            writeln!(file, "{entry}").unwrap();
            file.sync_all().unwrap();
        }
        assert!(transcript_matches_since(&path, offset, "post-limit turn").is_ok());
        assert_eq!(
            transcript_matches_since(&path, offset, "wrong turn")
                .unwrap_err()
                .code,
            "STANDBY_PROMPT_MISMATCH"
        );
    }

    #[test]
    fn interleaved_start_isolates_read_only_and_write_settings_and_hooks_per_slot() {
        let temp = tempfile::tempdir().unwrap();
        let cred = temp.path().join("cred");
        let state = temp.path().join("state");
        let auth_profile = state.join("agy-home");
        let auth_dir = auth_profile.join(".gemini/antigravity-cli");
        fs::create_dir_all(&cred).unwrap();
        fs::create_dir_all(&auth_dir).unwrap();
        let token_path = auth_dir.join("antigravity-oauth-token");
        fs::write(&token_path, b"managed-token\n").unwrap();
        fs::set_permissions(&token_path, fs::Permissions::from_mode(0o600)).unwrap();

        let dir_read = state.join("antigravity/slot-read");
        let dir_write = state.join("antigravity/slot-write");
        let cwd_read = state.join("cwd-read");
        let cwd_write = state.join("cwd-write");
        fs::create_dir_all(&cwd_read).unwrap();
        fs::create_dir_all(&cwd_write).unwrap();
        // Create minimal git metadata for write worktree denial on macOS.
        Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&cwd_write)
            .status()
            .unwrap();

        // Interleave profile preparation before spawning the read-only child:
        // read-only A prepares -> write B prepares -> read-only A spawns -> write B spawns.
        let home_read = prepare_profile(&dir_read, &auth_profile, false).unwrap();
        let home_write = prepare_profile(&dir_write, &auth_profile, true).unwrap();

        let probe_bin = state.join("agy-probe.sh");
        fs::write(
            &probe_bin,
            "#!/bin/sh\ncp \"$HOME/.gemini/antigravity-cli/settings.json\" \"$PWD/observed-settings.json\"\ncp \"$HOME/.gemini/config/hooks.json\" \"$PWD/observed-hooks.json\"\nprintf '%s\\n' '{\"event\":\"init\",\"conversation_id\":\"conv-probe\"}'\nexec sleep 5\n",
        )
        .unwrap();
        fs::set_permissions(&probe_bin, fs::Permissions::from_mode(0o755)).unwrap();

        let mut live_read = Live::spawn_one(
            &probe_bin, &cwd_read, &dir_read, &home_read, &cred, false, &None, None,
        )
        .unwrap();
        let mut live_write = Live::spawn_one(
            &probe_bin,
            &cwd_write,
            &dir_write,
            &home_write,
            &cred,
            true,
            &None,
            None,
        )
        .unwrap();
        live_read.close().unwrap();
        live_write.close().unwrap();

        let read_settings: Value =
            serde_json::from_slice(&fs::read(cwd_read.join("observed-settings.json")).unwrap())
                .unwrap();
        let write_settings: Value =
            serde_json::from_slice(&fs::read(cwd_write.join("observed-settings.json")).unwrap())
                .unwrap();
        assert_eq!(read_settings["toolPermission"], "request-review");
        assert_eq!(write_settings["toolPermission"], "always-proceed");

        let read_hooks: Value =
            serde_json::from_slice(&fs::read(cwd_read.join("observed-hooks.json")).unwrap())
                .unwrap();
        let write_hooks: Value =
            serde_json::from_slice(&fs::read(cwd_write.join("observed-hooks.json")).unwrap())
                .unwrap();
        assert_eq!(
            read_hooks["hooks"]["PreToolUse"][0]["matcher"],
            "write_to_file|replace_file_content|multi_replace_file_content|sed_file|notebook_edit|run_command"
        );
        assert_eq!(
            write_hooks["hooks"]["PreToolUse"][0]["matcher"],
            "run_command"
        );
    }

    #[test]
    fn managed_oauth_token_rejects_symlinks_and_missing_files() {
        let temp = tempfile::tempdir().unwrap();
        let auth_profile = temp.path().join("agy-home");
        let agy_dir = auth_profile.join(".gemini/antigravity-cli");
        fs::create_dir_all(&agy_dir).unwrap();
        assert_eq!(
            read_managed_oauth_token(&auth_profile).unwrap_err().code,
            "HARNESS_LOGIN_REQUIRED"
        );

        let target = temp.path().join("outside-token");
        fs::write(&target, b"secret-token\n").unwrap();
        std::os::unix::fs::symlink(&target, agy_dir.join("antigravity-oauth-token")).unwrap();
        assert_eq!(
            read_managed_oauth_token(&auth_profile).unwrap_err().code,
            "INVALID_INPUT"
        );
    }
}
