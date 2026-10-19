//! One omp RPC process per selection. The transport is newline-delimited JSON
//! over stdio (`omp --mode rpc --no-ui`); the pane is H0's read-only observation
//! pane over `view.log`, never a control channel. The process runs with a managed
//! `HOME` so omp's auth, sessions and caches stay out of the person's `~/.omp`.
use crate::{
    observation::{Multiplexer, ObservationConfig, ObservationPane, TURN_OUTPUT_SCHEMA},
    runtime::{Running, RuntimeEvent, Session},
    standby::selection_key,
};
use agency_proto::{EvidenceLevel, ExecutionSpec, PortError, Result, Sealed, context::Bundle};
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
    spec: Sealed<ExecutionSpec>,
    text: String,
    model: String,
    copy: Option<crate::write::WorkCopy>,
    credential_root: PathBuf,
    token: Arc<Token>,
    tx: mpsc::SyncSender<RuntimeEvent>,
}
struct Worker {
    tx: mpsc::Sender<Job>,
    join: JoinHandle<Result<()>>,
}

/// Observation inputs for one selection: state root, pane config and the lazily
/// started multiplexer handle (None when panes are disabled).
pub(crate) struct PaneEnv {
    pub(crate) state: PathBuf,
    pub(crate) config: ObservationConfig,
    pub(crate) multiplexer: Option<Multiplexer>,
}

pub(crate) struct Pool {
    omp: PathBuf,
    idle: Duration,
    stopped: Arc<AtomicBool>,
    workers: Mutex<HashMap<String, Worker>>,
}

impl Pool {
    pub(crate) fn new(omp: PathBuf, idle: Duration) -> Self {
        Self {
            omp,
            idle,
            stopped: Arc::new(AtomicBool::new(false)),
            workers: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn submit(
        &self,
        pane: PaneEnv,
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
            spec: spec.clone(),
            text,
            model: spec.document.profession.model.clone(),
            copy,
            credential_root: credential_root.to_path_buf(),
            token: Arc::clone(&token),
            tx,
        };
        let mut workers = self.workers.lock().expect("omp workers");
        if self.stopped.load(Ordering::SeqCst) {
            return Err(PortError::invalid("runtime is stopping"));
        }
        if !workers.contains_key(&key) {
            let (tx, receiver) = mpsc::channel();
            let omp = self.omp.clone();
            let stopped = Arc::clone(&self.stopped);
            let idle = self.idle;
            let parent = exec
                .parent()
                .ok_or_else(|| PortError::invalid("execution parent missing"))?;
            let cwd = parent.join(format!("omp-session-{key}"));
            crate::storage::private_dir(&cwd)?;
            let worker_key = key.clone();
            let join = std::thread::spawn(move || {
                worker(omp, cwd, worker_key, idle, stopped, pane, receiver)
            });
            workers.insert(key.clone(), Worker { tx, join });
        }
        workers
            .get(&key)
            .expect("inserted omp worker")
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
        let workers = std::mem::take(&mut *self.workers.lock().expect("omp workers"));
        let mut error = None;
        for (_, Worker { tx, join }) in workers {
            drop(tx);
            match join.join() {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    error.get_or_insert(e);
                }
                Err(_) => {
                    error.get_or_insert_with(|| PortError::invalid("omp worker panicked"));
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
    omp: PathBuf,
    cwd: PathBuf,
    key: String,
    idle: Duration,
    stopped: Arc<AtomicBool>,
    pane: PaneEnv,
    receiver: mpsc::Receiver<Job>,
) -> Result<()> {
    let PaneEnv {
        state,
        config: observation,
        multiplexer,
    } = pane;
    let dir = state.join(format!("omp-{key}"));
    crate::storage::private_dir(&dir)?;
    let sessions = dir.join("sessions");
    crate::storage::private_dir(&sessions)?;
    let profile = state.join("omp-home");
    crate::storage::private_dir(&profile)?;
    let mut live: Option<Live> = None;
    let mut last = Instant::now();
    let mut returned: Option<Job> = None;
    loop {
        let shutdown = stopped.load(Ordering::SeqCst);
        if returned
            .as_ref()
            .is_some_and(|job| job.token.cancelled.load(Ordering::SeqCst))
            || shutdown
        {
            if let Some(session) = live.as_mut() {
                session.close()?;
            }
            live = None;
            release_returned(&mut returned, true);
        }
        if shutdown {
            break;
        }
        let job = match receiver.recv_timeout(Duration::from_millis(25)) {
            Ok(job) => job,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if live.is_some() && last.elapsed() >= idle {
                    if let Some(session) = live.as_mut() {
                        session.close()?;
                    }
                    live = None;
                    release_returned(&mut returned, false);
                }
                continue;
            }
        };
        // A new frozen Spec must not enter a process retaining the old write lease.
        // The native session is resumed by directory when run_job opens it again.
        if returned.is_some() {
            if let Some(session) = live.as_mut() {
                session.close()?;
            }
            live = None;
            release_returned(&mut returned, false);
        }
        let result = run_job(
            &TurnEnv {
                omp: &omp,
                dir: &dir,
                sessions: &sessions,
                profile: &profile,
                cwd: &cwd,
                state: &state,
                observation,
                multiplexer: multiplexer.clone(),
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
            if let Some(session) = live.as_mut() {
                session.close()?;
                if job.copy.is_some() {
                    let _ = job.tx.send(RuntimeEvent::TurnStopped {
                        requested_stop: job.token.cancelled.load(Ordering::SeqCst)
                            || stopped.load(Ordering::SeqCst),
                        session_closed: true,
                    });
                }
            }
            live = None;
        }
        if sealed {
            returned = Some(job);
        } else {
            if job.copy.is_some() {
                if let Some(session) = live.as_mut() {
                    session.close()?;
                    let _ = job.tx.send(RuntimeEvent::TurnStopped {
                        requested_stop: job.token.cancelled.load(Ordering::SeqCst)
                            || stopped.load(Ordering::SeqCst),
                        session_closed: true,
                    });
                }
                live = None;
            }
            let _ = job.tx.send(RuntimeEvent::DispatchReleased);
        }
        last = Instant::now();
    }
    if let Some(session) = live.as_mut() {
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

struct TurnEnv<'a> {
    omp: &'a Path,
    dir: &'a Path,
    sessions: &'a Path,
    profile: &'a Path,
    cwd: &'a Path,
    state: &'a Path,
    observation: ObservationConfig,
    multiplexer: Option<Multiplexer>,
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
    }) && let Some(mut previous) = live.take()
    {
        previous.close()?;
    }
    if live.is_none() {
        let (session, resumed, resume_failed) = Live::open(
            env.omp,
            cwd,
            env.dir,
            env.sessions,
            env.profile,
            &job.credential_root,
            job.copy.is_some(),
            &model,
            env.state,
            env.observation,
            env.multiplexer.clone(),
        )?;
        if let Some(error) = &session.pane_error {
            let _ = job.tx.send(RuntimeEvent::Observation {
                kind: "observation_failed".into(),
                payload: json!({"message": error}),
                source: EvidenceLevel::AdapterEvent,
            });
        }
        let _ = job.tx.send(RuntimeEvent::Observation {
            kind: "session_opened".into(),
            payload: json!({
                "resumed": resumed,
                "resume_failed": resume_failed,
                "session": session.session,
                "file": session.file.as_ref().map(|path| path.display().to_string()),
                "pane": session.pane_id,
                "session_dir": session.session_dir.display().to_string(),
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
    let session = live.as_mut().expect("omp session");
    let transcript_offset = session
        .file
        .as_ref()
        .and_then(|path| fs::metadata(path).ok())
        .map_or(0, |meta| meta.len());
    let turn = session.start_turn(&job.text)?;
    session.turn += 1;
    let turn_no = session.turn;
    if let Some(pane) = &session.pane {
        let _ = pane.append_turn_start(turn_no);
        let _ = pane.append_prompt_summary(&job.text);
    }
    let mut interrupt: Option<(Instant, bool)> = None;
    loop {
        let requested =
            job.token.cancelled.load(Ordering::SeqCst) || stopped.load(Ordering::SeqCst);
        let expired = crate::storage::now_ms() >= job.spec.document.deadline_ms;
        if interrupt.is_none() && (requested || expired) {
            if expired {
                let _ = job.tx.send(RuntimeEvent::DeadlineReached);
            }
            match session.abort() {
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
        if let Some(msg) = session.poll(Duration::from_millis(200))? {
            {
                if msg["type"] == "extension_ui_request" {
                    // RPC emits only a host-issued login prompt. Agency never
                    // answers a person's sign-in; it reports the command.
                    if let Some(pane) = &session.pane {
                        let _ = pane.append_turn_end(turn_no, "error");
                    }
                    return Err(PortError::new(
                        "HARNESS_LOGIN_REQUIRED",
                        "omp asked for a login inside the managed profile; \
                         run `HOME=<state>/omp-home omp auth-broker login` (the Agency-managed omp HOME)",
                        "sign_in_omp",
                    ));
                }
                if msg["type"] == "response"
                    && msg["id"] == turn.as_str()
                    && msg["success"] == true
                    && msg["data"]["agentInvoked"] == false
                {
                    // A registered slash command finished locally: the body was
                    // not delivered to the agent as data.
                    if let Some(pane) = &session.pane {
                        let _ = pane.append_turn_end(turn_no, "error");
                    }
                    return Err(PortError::new(
                        "STANDBY_PROMPT_MISMATCH",
                        "omp handled the prompt as a local command instead of an agent turn",
                        "inspect_dispatch",
                    ));
                }
                if msg["type"] == "prompt_result" && msg["id"] == turn.as_str() {
                    match msg["status"].as_str() {
                        Some("completed") => {
                            if let Some((_, requested_stop)) = interrupt {
                                if let Some(pane) = &session.pane {
                                    let _ = pane.append_turn_end(turn_no, "stopped");
                                }
                                let _ = job.tx.send(RuntimeEvent::TurnStopped {
                                    requested_stop,
                                    session_closed: false,
                                });
                                return Ok(());
                            }
                            let text = session.answer()?.ok_or_else(|| {
                                PortError::invalid("turn completed without an assistant message")
                            })?;
                            session_matches_since(
                                session.file.as_deref(),
                                transcript_offset,
                                &job.text,
                            )?;
                            if let Some(pane) = &session.pane {
                                let _ = pane.append_answer(&text);
                                let _ = pane.append_turn_end(turn_no, "completed");
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
                                    session.close()?;
                                    *live = None;
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
                        Some("aborted") => {
                            if let Some(pane) = &session.pane {
                                let _ = pane.append_turn_end(turn_no, "aborted");
                            }
                            let requested_stop =
                                interrupt.map(|(_, requested)| requested).unwrap_or(false);
                            let _ = job.tx.send(RuntimeEvent::TurnStopped {
                                requested_stop,
                                session_closed: false,
                            });
                            return Ok(());
                        }
                        Some("error") => {
                            if let Some(pane) = &session.pane {
                                let _ = pane.append_turn_end(turn_no, "error");
                            }
                            let message = msg["error"]["message"]
                                .as_str()
                                .unwrap_or("omp ended this turn with an error");
                            return Err(PortError::new(
                                "HARNESS_TURN_ERROR",
                                message,
                                "inspect_dispatch",
                            ));
                        }
                        other => {
                            if let Some(pane) = &session.pane {
                                let _ = pane.append_turn_end(turn_no, "error");
                            }
                            return Err(PortError::invalid(format!(
                                "omp prompt_result status {other:?}"
                            )));
                        }
                    }
                }
            }
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

fn model_arg(model: &str) -> String {
    let model = model.trim();
    if model.is_empty() || model == "none" {
        String::new()
    } else {
        model.to_owned()
    }
}

struct Live {
    child: Child,
    stdin: Option<ChildStdin>,
    frames: mpsc::Receiver<Value>,
    reader: Option<JoinHandle<()>>,
    eof: Arc<AtomicBool>,
    next: u64,
    turn: u64,
    session: String,
    file: Option<PathBuf>,
    session_dir: PathBuf,
    pane: Option<ObservationPane>,
    pane_id: Option<String>,
    pane_error: Option<String>,
    events: Arc<Mutex<Option<fs::File>>>,
    events_pending: PathBuf,
    resumed_existing: bool,
    closed: bool,
    writing: bool,
    cwd: PathBuf,
    model: String,
}

impl Live {
    #[allow(clippy::too_many_arguments)]
    fn open(
        omp: &Path,
        cwd: &Path,
        dir: &Path,
        sessions: &Path,
        profile: &Path,
        credential_root: &Path,
        writing: bool,
        model: &str,
        state: &Path,
        observation: ObservationConfig,
        multiplexer: Option<Multiplexer>,
    ) -> Result<(Self, bool, bool)> {
        crate::storage::private_dir(sessions)?;
        crate::storage::private_dir(profile)?;
        let log = crate::storage::private_file(&dir.join("rpc.err"))?;
        let mut arguments = vec![
            "--mode".to_owned(),
            "rpc".to_owned(),
            "--no-ui".to_owned(),
            "--session-dir".to_owned(),
            sessions.display().to_string(),
            "--approval-mode".to_owned(),
            "yolo".to_owned(),
        ];
        if !model.is_empty() {
            arguments.push("--model".to_owned());
            arguments.push(model.to_owned());
        }
        let mut child = crate::confine::omp_command(
            omp,
            &arguments,
            cwd,
            dir,
            profile,
            credential_root,
            writing,
        )?;
        child
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::from(log))
            .current_dir(cwd)
            .process_group(0);
        let mut child = child.spawn()?;
        let stdin = child.stdin.take();
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| PortError::invalid("omp stdout missing"))?;
        let pending = dir.join("events-pending");
        crate::storage::private_dir(&pending)?;
        let events_pending = pending.join("events.jsonl");
        let event_file = crate::storage::private_file(&events_pending)?;
        let (tx, frames) = mpsc::channel();
        let events: Arc<Mutex<Option<fs::File>>> = Arc::new(Mutex::new(Some(event_file)));
        let reader_events = Arc::clone(&events);
        let eof = Arc::new(AtomicBool::new(false));
        let reader_eof = Arc::clone(&eof);
        let reader = std::thread::spawn(move || {
            struct EofGuard(Arc<AtomicBool>);
            impl Drop for EofGuard {
                fn drop(&mut self) {
                    self.0.store(true, Ordering::SeqCst);
                }
            }
            let _eof = EofGuard(reader_eof);
            use std::io::Write as _;
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(mut guard) = reader_events.lock()
                    && let Some(file) = guard.as_mut()
                {
                    let _ = file.write_all(line.as_bytes());
                    let _ = file.write_all(b"\n");
                    let _ = file.flush();
                }
                match serde_json::from_str::<Value>(&line) {
                    Ok(value) => {
                        if tx.send(value).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        let mut live = Self {
            child,
            stdin,
            frames,
            reader: Some(reader),
            eof,
            next: 1,
            turn: 0,
            session: String::new(),
            file: None,
            session_dir: PathBuf::new(),
            pane: None,
            pane_id: None,
            pane_error: None,
            events,
            events_pending,
            resumed_existing: false,
            closed: false,
            writing,
            cwd: cwd.to_path_buf(),
            model: model.to_owned(),
        };
        let attached = (|| {
            live.wait_ready()?;
            let opened = live.request(
                "open_session",
                json!({"sessionDir": sessions.display().to_string()}),
            );
            match opened {
                Ok(data) => {
                    live.session = data["sessionId"].as_str().unwrap_or("").to_owned();
                    live.file = data["sessionFile"].as_str().map(PathBuf::from);
                    live.resumed_existing = data["resumed"].as_bool().unwrap_or(false);
                    Ok(false)
                }
                Err(_) => {
                    let fresh = live.request("new_session", json!({}))?;
                    let _ = fresh;
                    let state = live.request("get_state", json!({}))?;
                    live.session = state["sessionId"].as_str().unwrap_or("").to_owned();
                    live.file = state["sessionFile"].as_str().map(PathBuf::from);
                    Ok(true)
                }
            }
        })();
        let resumed = live.resumed_existing;
        match attached {
            Ok(resume_failed) => {
                live.attach_observation(state, observation, multiplexer)?;
                Ok((live, resumed, resume_failed))
            }
            Err(error) => {
                live.close()?;
                Err(error)
            }
        }
    }

    fn attach_observation(
        &mut self,
        state: &Path,
        observation: ObservationConfig,
        multiplexer: Option<Multiplexer>,
    ) -> Result<()> {
        let session_dir = state.join("omp").join(&self.session);
        crate::storage::private_dir(&session_dir)?;
        fs::rename(&self.events_pending, session_dir.join("events.jsonl"))?;
        let _ = fs::remove_dir(self.events_pending.parent().unwrap_or(&session_dir));
        self.session_dir = session_dir;
        self.write_state()?;
        match ObservationPane::open(observation, &self.session_dir, "agency-omp", || {
            multiplexer
                .clone()
                .ok_or_else(|| PortError::invalid("observation multiplexer missing"))
        }) {
            Ok((pane, pane_id)) => {
                self.pane = Some(pane);
                self.pane_id = pane_id;
            }
            Err(error) => {
                // A pane is observational, never a precondition for dispatch.
                self.pane_error = Some(format!("{}: {}", error.code, error.message));
            }
        }
        self.write_state()
    }

    fn write_state(&self) -> Result<()> {
        let path = self.session_dir.join("state.json");
        let temp = path.with_extension("pending");
        let mut file = crate::storage::private_file(&temp)?;
        file.write_all(&serde_json::to_vec(&json!({
            "harness": "omp",
            "session": self.session,
            "file": self.file.as_ref().map(|path| path.display().to_string()),
            "pane_id": self.pane_id,
            "two_way": false,
        }))?)?;
        file.sync_all()?;
        fs::rename(temp, path)?;
        Ok(())
    }

    fn is_alive(&mut self) -> bool {
        !self.eof.load(Ordering::SeqCst) && matches!(self.child.try_wait(), Ok(None))
    }

    fn wait_ready(&mut self) -> Result<()> {
        let deadline = Instant::now() + Duration::from_secs(20);
        while Instant::now() < deadline {
            if self.child.try_wait()?.is_some() {
                return Err(PortError::new(
                    "OMP_RPC_EXITED",
                    "omp exited before the ready frame",
                    "inspect_omp_session",
                ));
            }
            match self.poll(Duration::from_millis(100))? {
                Some(value) if value["type"] == "ready" => {
                    let version = value["protocolVersion"].as_u64().unwrap_or(0);
                    if version < 1 {
                        return Err(PortError::new(
                            "OMP_PROTOCOL",
                            "omp ready frame advertised no protocol v1",
                            "upgrade_omp",
                        ));
                    }
                    return Ok(());
                }
                _ => {}
            }
        }
        Err(PortError::new(
            "OMP_RPC_EXITED",
            "omp did not write a ready frame",
            "inspect_omp_session",
        ))
    }

    fn send(&mut self, value: Value) -> Result<()> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| PortError::invalid("omp stdin is closed"))?;
        serde_json::to_writer(&mut *stdin, &value)?;
        stdin.write_all(b"\n")?;
        stdin.flush()?;
        Ok(())
    }

    fn request_id(&mut self, id: &str, command: &str, mut params: Value) -> Result<Value> {
        params["id"] = json!(id);
        params["type"] = json!(command);
        self.send(params)?;
        let deadline = Instant::now() + Duration::from_secs(40);
        while Instant::now() < deadline {
            let Some(value) = self.poll(Duration::from_millis(200))? else {
                continue;
            };
            if value["type"] == "extension_ui_request" {
                return Err(PortError::new(
                    "HARNESS_LOGIN_REQUIRED",
                    "omp asked for a login inside the managed profile; \
                     run `HOME=<state>/omp-home omp auth-broker login` (the Agency-managed omp HOME)",
                    "sign_in_omp",
                ));
            }
            if value["id"] == id && value["type"] == "response" {
                if value["success"] == true {
                    return Ok(value["data"].clone());
                }
                return Err(PortError::new(
                    "OMP_COMMAND_FAILED",
                    value["error"].as_str().unwrap_or("omp refused the command"),
                    "inspect_omp_session",
                ));
            }
        }
        Err(PortError::invalid("omp did not answer a command"))
    }

    fn request(&mut self, command: &str, params: Value) -> Result<Value> {
        let id = format!("cmd-{}", self.next);
        self.next += 1;
        self.request_id(&id, command, params)
    }

    fn start_turn(&mut self, text: &str) -> Result<String> {
        let id = format!("turn-{}", self.next);
        self.next += 1;
        self.send(json!({"id": id, "type": "prompt", "message": text}))?;
        Ok(id)
    }

    fn answer(&mut self) -> Result<Option<String>> {
        let data = self.request("get_last_assistant_text", json!({}))?;
        Ok(data["text"].as_str().map(str::to_owned))
    }

    fn abort(&mut self) -> Result<()> {
        self.send(json!({"id": format!("abort-{}", self.next), "type": "abort"}))?;
        self.next += 1;
        Ok(())
    }

    fn poll(&mut self, wait: Duration) -> Result<Option<Value>> {
        match self.frames.recv_timeout(wait) {
            Ok(value) => Ok(Some(value)),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(PortError::new(
                "OMP_RPC_EXITED",
                "omp closed the RPC stream",
                "inspect_omp_session",
            )),
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
        *self.events.lock().expect("omp events") = None;
        drop(self.stdin.take());
        let deadline = Instant::now() + Duration::from_millis(400);
        while Instant::now() < deadline {
            if self.child.try_wait()?.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        if self.child.try_wait()?.is_none() {
            signal_group(self.child.id(), "TERM");
            let deadline = Instant::now() + Duration::from_millis(400);
            while Instant::now() < deadline {
                if self.child.try_wait()?.is_some() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        }
        if self.child.try_wait()?.is_none() {
            signal_group(self.child.id(), "KILL");
            let _ = self.child.wait();
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

fn signal_group(pid: u32, signal: &str) {
    let _ = Command::new("/bin/kill")
        .arg(format!("-{signal}"))
        .arg(format!("-{pid}"))
        .status();
}

pub(crate) fn session_matches_since(file: Option<&Path>, offset: u64, text: &str) -> Result<()> {
    let Some(file) = file else {
        return Err(PortError::new(
            "OMP_SESSION_MISSING",
            "omp opened no session file",
            "inspect_omp_session",
        ));
    };
    if session_file_has_input_since(file, offset, text)? {
        Ok(())
    } else {
        Err(PortError::new(
            "STANDBY_PROMPT_MISMATCH",
            "omp session input for this turn does not match the dispatch",
            "inspect_dispatch",
        ))
    }
}

/// Only the slice appended after `offset` may certify this turn. An older equal
/// message must not vouch for a mismatched current one, and a large history
/// must not make every later turn unreadable.
pub(crate) fn session_file_has_input_since(file: &Path, offset: u64, text: &str) -> Result<bool> {
    use std::io::{Seek, SeekFrom};
    let mut file = fs::File::open(file)?;
    let len = file.metadata()?.len();
    if len < offset {
        return Ok(false);
    }
    file.seek(SeekFrom::Start(offset))?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > LIMIT {
        return Err(PortError::invalid(
            "omp session turn slice exceeds read limit",
        ));
    }
    for line in String::from_utf8_lossy(&bytes).lines() {
        if !line.contains("\"role\":\"user\"") && !line.contains("\"role\": \"user\"") {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if value["type"] == "message"
            && value["message"]["role"] == "user"
            && value["message"]["content"].as_array().is_some_and(|parts| {
                parts
                    .iter()
                    .any(|part| part["type"] == "text" && part["text"].as_str() == Some(text))
            })
        {
            return Ok(true);
        }
    }
    Ok(false)
}
