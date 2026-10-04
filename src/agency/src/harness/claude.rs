//! One Claude Code print session. Completion is the JSONL `result` event, not the screen.
use agency_proto::{PortError, Result};
use serde_json::Value;
use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
    time::Duration,
};

pub struct Session {
    pub session_id: String,
    pub result: String,
    pub is_error: bool,
}

pub fn print_session(prompt: &str, timeout: Duration) -> Result<Session> {
    let mut child = Command::new("claude")
        .args([
            "-p",
            "--output-format",
            "stream-json",
            "--verbose",
            "--permission-mode",
            "dontAsk",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            PortError::new(
                "HARNESS_NOT_STARTED",
                format!("claude did not start: {error}"),
                "install_claude_code",
            )
        })?;
    child
        .stdin
        .take()
        .ok_or_else(|| PortError::invalid("claude stdin is missing"))?
        .write_all(prompt.as_bytes())?;
    let stdout = child.stdout.take().expect("claude stdout");
    let started = std::time::Instant::now();
    let mut session_id = String::new();
    let mut found: Option<Session> = None;
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        if started.elapsed() > timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(PortError::new(
                "HARNESS_TIMEOUT",
                "claude result event did not arrive before the caller timeout",
                "raise_timeout",
            ));
        }
        let line = line?;
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if let Some(id) = value.get("session_id").and_then(Value::as_str)
            && session_id.is_empty()
        {
            session_id = id.to_owned();
        }
        if value.get("type").and_then(Value::as_str) == Some("result") {
            let event_session = value
                .get("session_id")
                .and_then(Value::as_str)
                .unwrap_or(&session_id)
                .to_owned();
            if !session_id.is_empty() && event_session != session_id {
                continue;
            }
            let result = value
                .get("result")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned();
            let is_error = value
                .get("is_error")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            found = Some(Session {
                session_id: event_session,
                result,
                is_error,
            });
            break;
        }
    }
    let _ = child.wait();
    found.ok_or_else(|| {
        PortError::new(
            "HARNESS_RESULT_MISSING",
            "claude exited without a result event",
            "read_claude_jsonl",
        )
    })
}
