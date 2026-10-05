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
        if let Some(id) = value.get("session_id").and_then(Value::as_str)
            && session_id.is_empty()
        {
            session_id = id.to_owned();
        }
        if value.get("type").and_then(Value::as_str) != Some("result") {
            continue;
        }
        let event_session = value
            .get("session_id")
            .and_then(Value::as_str)
            .unwrap_or(&session_id)
            .to_owned();
        if !session_id.is_empty() && event_session != session_id {
            continue;
        }
        found = Some(Session {
            session_id: event_session,
            result: value
                .get("result")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
            is_error: value
                .get("is_error")
                .and_then(Value::as_bool)
                .unwrap_or(true),
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
