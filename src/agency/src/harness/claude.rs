//! One Claude Code print session. Completion is the JSONL `result` event, not the screen.
use agency_proto::{PortError, Result};
use serde_json::Value;

pub struct Session {
    pub session_id: String,
    pub result: String,
    pub is_error: bool,
}

pub fn result_from_jsonl(text: &str) -> Result<Session> {
    let mut session_id = String::new();
    let mut found = None;
    for line in text.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if value.get("type").and_then(Value::as_str) == Some("system")
            && value.get("subtype").and_then(Value::as_str) == Some("init")
        {
            session_id = value
                .get("session_id")
                .and_then(Value::as_str)
                .filter(|id| !id.is_empty())
                .ok_or_else(|| protocol("init has no session id"))?
                .to_owned();
        }
        if value.get("type").and_then(Value::as_str) != Some("result") {
            continue;
        }
        let event_session = value
            .get("session_id")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .ok_or_else(|| protocol("result has no session id"))?
            .to_owned();
        if session_id.is_empty() || event_session != session_id || found.is_some() {
            return Err(protocol(
                "result does not belong to the one initialized session",
            ));
        }
        let subtype = value
            .get("subtype")
            .and_then(Value::as_str)
            .ok_or_else(|| protocol("result has no subtype"))?;
        let is_error = value
            .get("is_error")
            .and_then(Value::as_bool)
            .ok_or_else(|| protocol("result has no error flag"))?;
        if subtype != "success" && !is_error {
            return Err(protocol("non-success result cannot claim success"));
        }
        found = Some(Session {
            session_id: event_session,
            result: value
                .get("result")
                .and_then(Value::as_str)
                .or_else(|| {
                    value
                        .get("errors")
                        .and_then(Value::as_array)
                        .and_then(|a| a.first())
                        .and_then(Value::as_str)
                })
                .ok_or_else(|| protocol("result has no text or error reason"))?
                .to_owned(),
            is_error,
        });
    }
    found.ok_or_else(|| {
        PortError::new(
            "HARNESS_RESULT_MISSING",
            "claude output has no result event",
            "read_claude_jsonl",
        )
    })
}

fn protocol(message: &str) -> PortError {
    PortError::new("HARNESS_RESULT_INVALID", message, "read_claude_jsonl")
}
