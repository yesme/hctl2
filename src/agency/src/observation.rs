//! Read-only observation pane primitive and unified turn answer schema (`agency.turn.v1`).
//!
//! Control flows through each harness's native structured protocol; the pane only renders
//! `<session_dir>/view.log` via `tail -f` for human observation and is never used as an
//! evidence source or control channel.
use crate::herdr::{Client, Server};
use crate::launch::ExitFeed;
use agency_proto::{PortError, Result};
use serde_json::{Value, json};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

/// Unified read-only turn answer schema shared across all harnesses (`07` §二 第 45 条 ②).
pub const TURN_OUTPUT_SCHEMA: &str = "agency.turn.v1";

/// Environment toggle controlling whether Agency opens an observation pane for dispatches.
pub const OBSERVATION_PANE_ENV: &str = "HCTL2_AGENCY_OBSERVATION_PANE";

/// Configuration for read-only observation panes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservationConfig {
    pub enabled: bool,
}

impl Default for ObservationConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl ObservationConfig {
    pub const fn enabled() -> Self {
        Self { enabled: true }
    }

    pub const fn disabled() -> Self {
        Self { enabled: false }
    }

    pub fn from_env() -> Self {
        match std::env::var(OBSERVATION_PANE_ENV) {
            Ok(val) => {
                let normalized = val.trim().to_ascii_lowercase();
                let disabled = matches!(normalized.as_str(), "0" | "false" | "off" | "no");
                Self { enabled: !disabled }
            }
            Err(_) => Self { enabled: true },
        }
    }
}

/// Opaque handle to the terminal multiplexer server so callers do not depend on Herdr types.
#[derive(Clone)]
pub struct Multiplexer {
    server: Arc<Server>,
}

impl Multiplexer {
    pub(crate) fn from_server(server: Arc<Server>) -> Self {
        Self { server }
    }

    pub fn state_dir(&self) -> &Path {
        &self.server.state
    }

    /// Close a pane by ID directly on the multiplexer (e.g. simulating external close).
    pub fn close_pane(&self, pane_id: &str) -> Result<()> {
        let client = Client::connect(&self.server.socket)?;
        match client.call("pane.close", json!({"pane_id": pane_id})) {
            Ok(_) => Ok(()),
            Err(error) if error.message.contains("pane_not_found") => Ok(()),
            Err(error) => Err(error),
        }
    }
}

/// Liveness state of an observation pane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneStatus {
    /// Observation panes are disabled by configuration; no pane was opened.
    Disabled,
    /// The `tail -f view.log` pane is running.
    Alive,
    /// The pane process exited or the pane was closed.
    Exited,
}

/// Structured entries rendered into `<session_dir>/view.log`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewEntry<'a> {
    TurnStart {
        turn: u64,
    },
    PromptSummary {
        summary: &'a str,
    },
    ToolCall {
        name: &'a str,
        detail: Option<&'a str>,
    },
    Answer {
        text: &'a str,
    },
    TurnEnd {
        turn: u64,
        status: &'a str,
    },
}

impl ViewEntry<'_> {
    /// Render the entry into its canonical `view.log` line format.
    pub fn render(&self) -> String {
        match self {
            Self::TurnStart { turn } => format!("=== turn {turn} start ===\n"),
            Self::PromptSummary { summary } => {
                format!("[prompt] {}\n", summarize_single_line(summary, 240))
            }
            Self::ToolCall { name, detail } => {
                let clean_name = sanitize_single_line(name);
                match detail.map(sanitize_single_line) {
                    Some(clean_detail) if !clean_detail.is_empty() => {
                        format!("[tool] {clean_name}: {clean_detail}\n")
                    }
                    _ => format!("[tool] {clean_name}\n"),
                }
            }
            Self::Answer { text } => {
                let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
                let body = normalized.strip_suffix('\n').unwrap_or(&normalized);
                format!("[answer]\n{body}\n")
            }
            Self::TurnEnd { turn, status } => {
                let clean_status = sanitize_single_line(status);
                format!("=== turn {turn} end ({clean_status}) ===\n")
            }
        }
    }
}

fn sanitize_single_line(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if matches!(c, '\r' | '\n' | '\t') {
                ' '
            } else {
                c
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn summarize_single_line(input: &str, max_chars: usize) -> String {
    let single = sanitize_single_line(input);
    if single.chars().count() <= max_chars {
        return single;
    }
    let truncated: String = single.chars().take(max_chars).collect();
    format!("{truncated}...")
}

struct LivePane {
    pane_id: String,
    client: Client,
    events: Mutex<ExitFeed>,
    closed: Mutex<bool>,
    _server: Arc<Server>,
}

/// Read-only observation pane backed by `<session_dir>/view.log`.
pub struct ObservationPane {
    view_log: PathBuf,
    inner: Option<LivePane>,
}

impl ObservationPane {
    /// Open an observation pane running `tail -f <session_dir>/view.log` when enabled,
    /// or prepare `<session_dir>/view.log` without starting the multiplexer when disabled.
    pub fn open(
        config: ObservationConfig,
        session_dir: &Path,
        label: &str,
        multiplexer: impl FnOnce() -> Result<Multiplexer>,
    ) -> Result<(Self, Option<String>)> {
        let view_log = ensure_view_log(session_dir)?;
        if !config.enabled {
            return Ok((
                Self {
                    view_log,
                    inner: None,
                },
                None,
            ));
        }
        let mux = multiplexer()?;
        let events = ExitFeed::subscribe(&mux.server.socket)?;
        let client = Client::connect(&mux.server.socket)?;
        let created = client.call(
            "workspace.create",
            json!({
                "cwd": session_dir,
                "label": label,
                "focus": false,
            }),
        )?;
        let tab = created
            .pointer("/tab/tab_id")
            .and_then(Value::as_str)
            .ok_or_else(|| PortError::invalid("workspace has no tab"))?;
        let applied = client.call(
            "layout.apply",
            json!({
                "tab_id": tab,
                "focus": false,
                "root": {
                    "type": "pane",
                    "cwd": session_dir,
                    "command": ["/usr/bin/tail", "-n", "+1", "-f", &view_log],
                }
            }),
        )?;
        let pane_id: String = applied
            .pointer("/layout/root/pane_id")
            .and_then(Value::as_str)
            .ok_or_else(|| PortError::invalid("layout has no pane"))?
            .into();
        Ok((
            Self {
                view_log,
                inner: Some(LivePane {
                    pane_id: pane_id.clone(),
                    client,
                    events: Mutex::new(events),
                    closed: Mutex::new(false),
                    _server: mux.server,
                }),
            },
            Some(pane_id),
        ))
    }

    pub fn pane_id(&self) -> Option<&str> {
        self.inner.as_ref().map(|live| live.pane_id.as_str())
    }

    pub fn view_log(&self) -> &Path {
        &self.view_log
    }

    /// Append a rendered entry to `<session_dir>/view.log`.
    /// Writing to `view.log` always succeeds even if the pane is disabled or was closed by a human.
    pub fn append(&self, entry: ViewEntry<'_>) -> Result<()> {
        append_rendered(&self.view_log, &entry.render())
    }

    pub fn append_turn_start(&self, turn: u64) -> Result<()> {
        self.append(ViewEntry::TurnStart { turn })
    }

    pub fn append_prompt_summary(&self, summary: &str) -> Result<()> {
        self.append(ViewEntry::PromptSummary { summary })
    }

    pub fn append_tool_call(&self, name: &str, detail: Option<&str>) -> Result<()> {
        self.append(ViewEntry::ToolCall { name, detail })
    }

    pub fn append_answer(&self, text: &str) -> Result<()> {
        self.append(ViewEntry::Answer { text })
    }

    pub fn append_turn_end(&self, turn: u64, status: &str) -> Result<()> {
        self.append(ViewEntry::TurnEnd { turn, status })
    }

    /// Query whether the pane is disabled, alive, or exited.
    pub fn status(&self) -> Result<PaneStatus> {
        let Some(live) = &self.inner else {
            return Ok(PaneStatus::Disabled);
        };
        if *live.closed.lock().expect("pane close") {
            return Ok(PaneStatus::Exited);
        }
        if live.events.lock().expect("exit feed").poll(&live.pane_id)? {
            *live.closed.lock().expect("pane close") = true;
            return Ok(PaneStatus::Exited);
        }
        match live.client.call(
            "pane.read",
            json!({
                "pane_id": live.pane_id,
                "source": "visible",
                "format": "text",
                "strip_ansi": true,
            }),
        ) {
            Ok(_) => Ok(PaneStatus::Alive),
            Err(error) if error.message.contains("pane_not_found") => {
                *live.closed.lock().expect("pane close") = true;
                Ok(PaneStatus::Exited)
            }
            Err(error) => Err(error),
        }
    }

    pub fn is_alive(&self) -> Result<bool> {
        Ok(self.status()? == PaneStatus::Alive)
    }

    pub fn exited(&self) -> Result<bool> {
        Ok(self.status()? == PaneStatus::Exited)
    }

    /// Read the visible text of the observation pane if enabled and alive.
    pub fn read_visible(&self) -> Result<Option<String>> {
        let Some(live) = &self.inner else {
            return Ok(None);
        };
        if *live.closed.lock().expect("pane close") {
            return Ok(None);
        }
        match live.client.call(
            "pane.read",
            json!({
                "pane_id": live.pane_id,
                "source": "visible",
                "format": "text",
                "strip_ansi": true,
            }),
        ) {
            Ok(screen) => Ok(Some(
                screen
                    .pointer("/read/text")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned(),
            )),
            Err(error) if error.message.contains("pane_not_found") => {
                *live.closed.lock().expect("pane close") = true;
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }

    /// Close the observation pane idempotently. No-op when disabled or already closed.
    pub fn close(&self) -> Result<()> {
        let Some(live) = &self.inner else {
            return Ok(());
        };
        let mut closed = live.closed.lock().expect("pane close");
        if *closed {
            return Ok(());
        }
        if live.events.lock().expect("exit feed").poll(&live.pane_id)? {
            *closed = true;
            return Ok(());
        }
        if let Err(error) = live
            .client
            .call("pane.close", json!({"pane_id": live.pane_id}))
            && !error.message.contains("pane_not_found")
            && !live.events.lock().expect("exit feed").poll(&live.pane_id)?
        {
            return Err(error);
        }
        *closed = true;
        Ok(())
    }
}

impl Drop for ObservationPane {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

/// Convenience function to open an observation pane for `session_dir`.
pub fn open_pane(
    config: ObservationConfig,
    session_dir: &Path,
    label: &str,
    multiplexer: impl FnOnce() -> Result<Multiplexer>,
) -> Result<(ObservationPane, Option<String>)> {
    ObservationPane::open(config, session_dir, label, multiplexer)
}

/// Convenience function to append a rendered entry directly to `<session_dir>/view.log`.
pub fn append_view_line(session_dir: &Path, entry: ViewEntry<'_>) -> Result<()> {
    let view_log = ensure_view_log(session_dir)?;
    append_rendered(&view_log, &entry.render())
}

fn ensure_view_log(session_dir: &Path) -> Result<PathBuf> {
    crate::storage::private_dir(session_dir)?;
    let view_log = session_dir.join("view.log");
    if !view_log.exists() {
        fs::write(&view_log, [])?;
    }
    Ok(view_log)
}

fn append_rendered(view_log: &Path, rendered: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(view_log)?;
    file.write_all(rendered.as_bytes())?;
    file.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_entry_renders_golden_line_format() {
        let mut rendered = String::new();
        rendered.push_str(&ViewEntry::TurnStart { turn: 1 }.render());
        rendered.push_str(
            &ViewEntry::PromptSummary {
                summary: "Inspect repository\nand summarize status",
            }
            .render(),
        );
        rendered.push_str(
            &ViewEntry::ToolCall {
                name: "Bash",
                detail: Some("git status\n--short"),
            }
            .render(),
        );
        rendered.push_str(
            &ViewEntry::ToolCall {
                name: "Read",
                detail: None,
            }
            .render(),
        );
        rendered.push_str(
            &ViewEntry::Answer {
                text: " line 1 \nline 2",
            }
            .render(),
        );
        rendered.push_str(
            &ViewEntry::TurnEnd {
                turn: 1,
                status: "completed",
            }
            .render(),
        );

        let golden = "\
=== turn 1 start ===
[prompt] Inspect repository and summarize status
[tool] Bash: git status --short
[tool] Read
[answer]
 line 1 
line 2
=== turn 1 end (completed) ===
";
        assert_eq!(rendered, golden);
    }
}
