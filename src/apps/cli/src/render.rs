//! Human-readable default output. `--json` is the machine interface and never
//! passes through here; every renderer below only runs when the flag is absent.

use serde_json::Value;

/// One rendered table: a header row, a dashed separator, then aligned rows.
pub(crate) struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    pub(crate) fn new(headers: &[&str]) -> Self {
        Table {
            headers: headers.iter().map(|h| (*h).to_owned()).collect(),
            rows: Vec::new(),
        }
    }

    pub(crate) fn row(&mut self, cells: Vec<String>) {
        self.rows.push(cells);
    }

    /// Plain aligned text: two spaces between columns, width by widest cell.
    /// No color escapes anywhere, terminal or not.
    pub(crate) fn render(&self, empty: &str) -> String {
        if self.rows.is_empty() {
            return empty.to_owned();
        }
        let columns = self.headers.len();
        let mut widths = self.headers.iter().map(String::len).collect::<Vec<_>>();
        for row in &self.rows {
            for (index, cell) in row.iter().enumerate().take(columns) {
                widths[index] = widths[index].max(cell.len());
            }
        }
        let mut lines = Vec::with_capacity(self.rows.len() + 2);
        let pad = |cells: &[String]| -> String {
            cells
                .iter()
                .enumerate()
                .take(columns)
                .map(|(index, cell)| {
                    let mut out = cell.clone();
                    if let Some(fill) = widths[index].checked_sub(cell.len()) {
                        out.extend(std::iter::repeat_n(' ', fill));
                    }
                    out
                })
                .collect::<Vec<_>>()
                .join("  ")
                .trim_end()
                .to_owned()
        };
        lines.push(pad(&self.headers));
        lines.push(
            widths
                .iter()
                .map(|width| "-".repeat(*width))
                .collect::<Vec<_>>()
                .join("  "),
        );
        for row in &self.rows {
            lines.push(pad(row));
        }
        lines.join("\n")
    }
}

fn cell(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number.to_string(),
        other => other.to_string(),
    }
}

fn rows_of<'a>(value: &'a Value, field: &str) -> Option<&'a Vec<Value>> {
    value.get(field).and_then(Value::as_array)
}

/// A human preview in three sections plus the exact command that confirms it.
pub(crate) struct PreviewSections {
    pub(crate) title: String,
    /// What this operation is about: labeled identity lines.
    pub(crate) object: Vec<(String, String)>,
    /// Why the command stops for a confirmation instead of acting.
    pub(crate) why: Vec<String>,
    /// What confirmation does: where it writes, and whether it can be undone.
    pub(crate) after: Vec<String>,
}

impl PreviewSections {
    pub(crate) fn render(&self, confirm: &str) -> String {
        let mut out = Vec::new();
        out.push(self.title.clone());
        out.push(String::new());
        out.push("Object".to_owned());
        for (label, value) in &self.object {
            out.push(format!("  {label}: {value}"));
        }
        out.push(String::new());
        out.push("Why this needs confirmation".to_owned());
        for line in &self.why {
            out.push(format!("  {line}"));
        }
        out.push(String::new());
        out.push("What happens after you confirm".to_owned());
        for line in &self.after {
            out.push(format!("  - {line}"));
        }
        out.push(String::new());
        out.push("Confirm with".to_owned());
        out.push(format!("  {confirm}"));
        out.join("\n")
    }
}

/// Rebuild the confirming command from the invocation the human just ran by
/// appending the preview token, so the printed line can be pasted as-is.
pub(crate) fn confirm_command(token: &str) -> String {
    finish_command(std::env::args(), None, token)
}

/// The dispatch preview's confirming subcommand is `start`, not `preview`, so
/// one word is renamed before the token is appended.
pub(crate) fn confirm_command_renaming(from: &str, to: &str, token: &str) -> String {
    finish_command(std::env::args(), Some((from, to)), token)
}

fn finish_command(
    arguments: impl IntoIterator<Item = String>,
    rename: Option<(&str, &str)>,
    token: &str,
) -> String {
    let mut out = String::new();
    let mut renamed = false;
    for argument in arguments {
        if !out.is_empty() {
            out.push(' ');
        }
        if let (Some((from, to)), false) = (rename, renamed)
            && argument == from
        {
            out.push_str(to);
            renamed = true;
        } else {
            out.push_str(&shell_word(&argument));
        }
    }
    out.push_str(" --preview-token ");
    out.push_str(&shell_word(token));
    out
}

fn shell_word(word: &str) -> String {
    if word.is_empty() {
        return "''".to_owned();
    }
    let plain = word
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '=' | '/' | '.' | ':'));
    if plain {
        word.to_owned()
    } else {
        format!("'{}'", word.replace('\'', "'\\''"))
    }
}

/// A dispatch preview: one Room Invocation about to start under the Project's
/// authorization, with its frozen context assembly.
pub(crate) fn dispatch_preview(token: &str, effect: &Value) -> String {
    let input = &effect["preview"]["input"];
    let sections = PreviewSections {
        title: "Dispatch preview".to_owned(),
        object: vec![
            (
                "invocation".to_owned(),
                dash(&effect["preview"]["consumer"]["id"]),
            ),
            ("project".to_owned(), dash(&input["project_id"])),
            ("room".to_owned(), dash(&input["room_id"])),
            ("worker target".to_owned(), dash(&input["target"])),
            (
                "worker profile".to_owned(),
                dash(&input["profile"]["key"]["id"]),
            ),
            (
                "bundle".to_owned(),
                dash(&effect["assembly"]["bundle"]["document"]["id"]),
            ),
            (
                "budget".to_owned(),
                format!("{} bytes", cell(&input["budget"])),
            ),
        ],
        why: vec![
            "confirming starts a Room Invocation: the Project authorizes this dispatch and freezes the context bundle listed above".to_owned(),
            "the dispatched worker acts under the selection's permissions for this room".to_owned(),
        ],
        after: vec![
            "writes the Invocation, its authorization and the dispatch intent to the control store; the roster is not changed".to_owned(),
            "the reconcile loop hands the dispatch to the paired Agency, which runs the worker".to_owned(),
            "undo: `hctl2 invocation cancel` revokes the authorization and queues the worker's stop; completed results stay recorded".to_owned(),
        ],
    };
    sections.render(&confirm_command_renaming("preview", "start", token))
}

/// An integration preview: one admitted ChangeSet Revision into one target ref.
pub(crate) fn integration_preview(token: &str, effect: &Value) -> String {
    let input = &effect["input"];
    let mut object = vec![
        (
            "ChangeSet Revision".to_owned(),
            dash(&input["change_set_revision_id"]),
        ),
        ("repo".to_owned(), dash(&input["repo_id"])),
        ("target ref".to_owned(), dash(&input["target_ref"])),
        ("authorization form".to_owned(), dash(&input["form"])),
        ("strategy".to_owned(), dash(&input["strategy"])),
        (
            "target head at preview".to_owned(),
            dash(&effect["observed_head"]),
        ),
    ];
    if let Some(expected) = effect["expected_head"].as_str() {
        object.push(("expected head".to_owned(), expected.to_owned()));
    }
    let sections = PreviewSections {
        title: "Integration preview".to_owned(),
        object,
        why: vec![
            "confirming freezes this intent and executes the integration".to_owned(),
            "the exact revision and target recorded here are what will be merged".to_owned(),
        ],
        after: vec![
            "persists the integration intent, then executes and reads back the merge in the background".to_owned(),
            "the target ref advances in the registered repository; this command cannot undo a merge".to_owned(),
            "an Integration Receipt is written only after readback confirms the target head".to_owned(),
        ],
    };
    sections.render(&confirm_command(token))
}

/// The three error fields, verbatim, for the default output.
pub(crate) fn error_block(code: &str, message: &str, recovery: &str) -> String {
    [
        "error".to_owned(),
        format!("  code: {code}"),
        format!("  message: {message}"),
        format!("  recovery_action: {recovery}"),
    ]
    .join("\n")
}

fn dash(value: &Value) -> String {
    let text = cell(value);
    if text.is_empty() {
        "-".to_owned()
    } else {
        text
    }
}

/// Default output for one query kind, or `None` to keep the pretty-JSON
/// fallback. List kinds render as tables with headers; empty lists say so.
pub(crate) fn query(kind: &str, value: &Value) -> Option<String> {
    match kind {
        "task.list" => {
            let items = rows_of(value, "items")?;
            let mut table = Table::new(&["task", "project", "title", "lifecycle", "version"]);
            for item in items {
                table.row(vec![
                    dash(&item["data"]["id"]),
                    dash(&item["data"]["project_id"]),
                    dash(&item["data"]["title"]),
                    dash(&item["data"]["lifecycle"]),
                    cell(&item["version"]),
                ]);
            }
            Some(table.render("No tasks yet."))
        }
        "task.sources" => {
            let items = rows_of(value, "items")?;
            let mut table = Table::new(&["source", "version"]);
            for item in items {
                table.row(vec![dash(&item["key"]["id"]), cell(&item["version"])]);
            }
            Some(table.render("No task sources connected yet."))
        }
        "task.board" => {
            let cards = rows_of(value, "cards")?;
            let mut table = Table::new(&["card", "title", "stage", "claimed"]);
            for entry in cards {
                table.row(vec![
                    format!(
                        "{}#{}",
                        cell(&entry["card"]["entity"]["immutable_external_entity_id"]),
                        cell(&entry["card"]["number"])
                    ),
                    dash(&entry["card"]["title"]),
                    dash(&entry["card"]["stage"]),
                    cell(&entry["claimed"]),
                ]);
            }
            Some(table.render("This board has no cards yet."))
        }
        "room.list" => {
            let rooms = rows_of(value, "rooms")?;
            let mut table = Table::new(&["room", "project", "attention"]);
            for entry in rooms {
                table.row(vec![
                    dash(&entry["room"]["id"]),
                    dash(&entry["binding"]["key"]["scope"]["id"]),
                    cell(&entry["health"]["needs_attention"]),
                ]);
            }
            Some(table.render("No rooms yet."))
        }
        "room.hierarchy" => {
            let mut out = Vec::new();
            out.push(format!(
                "carrier_space: {}",
                dash(&value["carrier_space_id"])
            ));
            out.push(format!(
                "needs_attention: {}",
                dash(&value["needs_attention"])
            ));
            out.push(String::new());
            let mut table = Table::new(&["parents", "children", "external_links"]);
            let parents = value["parents"].as_array();
            let children = value["children"].as_array();
            let links = value["external_links"].as_array();
            let longest = parents.map(Vec::len).unwrap_or_default().max(
                children
                    .map(Vec::len)
                    .unwrap_or_default()
                    .max(links.map(Vec::len).unwrap_or_default()),
            );
            if longest == 0 {
                out.push("No parent or child rooms recorded.".to_owned());
            } else {
                for index in 0..longest {
                    table.row(vec![
                        parents
                            .and_then(|rows| rows.get(index))
                            .map(|row| dash(&row["id"]))
                            .unwrap_or_default(),
                        children
                            .and_then(|rows| rows.get(index))
                            .map(dash)
                            .unwrap_or_default(),
                        links
                            .and_then(|rows| rows.get(index))
                            .map(dash)
                            .unwrap_or_default(),
                    ]);
                }
                out.push(table.render(""));
            }
            Some(out.join("\n"))
        }
        "project.list" => {
            let projects = rows_of(value, "projects")?;
            let mut table = Table::new(&["project", "version", "archived"]);
            for item in projects {
                table.row(vec![
                    dash(&item["key"]["id"]),
                    cell(&item["version"]),
                    cell(&item["data"]["archived"]),
                ]);
            }
            Some(table.render("No projects yet."))
        }
        "agency.bindings" => {
            let records = rows_of(value, "records")?;
            let mut table = Table::new(&["binding", "protocol", "version"]);
            for item in records {
                table.row(vec![
                    dash(&item["key"]["id"]),
                    dash(&item["data"]["value"]["protocol"]),
                    cell(&item["version"]),
                ]);
            }
            Some(table.render("No Agency paired yet."))
        }
        "agency.catalog" => {
            let professions = value["professions"].as_array()?;
            let mut out = Vec::new();
            let mut table = Table::new(&["profession", "revision", "harness", "persona"]);
            for item in professions {
                table.row(vec![
                    dash(&item["reference"]["id"]),
                    dash(&item["reference"]["revision"]),
                    dash(&item["harness"]["id"]),
                    dash(&item["persona"]),
                ]);
            }
            out.push("Professions".to_owned());
            out.push(table.render("This catalog lists no professions."));
            let harnesses = value["harnesses"].as_array()?;
            if !harnesses.is_empty() {
                let mut table = Table::new(&["harness", "revision"]);
                for item in harnesses {
                    table.row(vec![dash(&item["id"]), dash(&item["revision"])]);
                }
                out.push(String::new());
                out.push("Harnesses".to_owned());
                out.push(table.render(""));
            }
            Some(out.join("\n"))
        }
        "profession.list" => {
            let records = rows_of(value, "records")?;
            let mut table = Table::new(&["profession", "revision", "digest", "version"]);
            for item in records {
                let reference = &item["data"]["value"]["reference"];
                table.row(vec![
                    dash(&reference["id"]),
                    dash(&reference["revision"]),
                    dash(&reference["digest"]),
                    cell(&item["version"]),
                ]);
            }
            Some(table.render("No professions accepted yet."))
        }
        "invocation.list" => {
            let invocations = rows_of(value, "invocations")?;
            let mut table = Table::new(&["invocation", "state", "version", "reason"]);
            for item in invocations {
                table.row(vec![
                    dash(&item["invocation_id"]),
                    dash(&item["state"]),
                    cell(&item["state_version"]),
                    dash(&item["reason"]),
                ]);
            }
            Some(table.render("This Project has no Invocations yet."))
        }
        "integration.list" => {
            let items = rows_of(value, "items")?;
            let mut table = Table::new(&["intent", "state"]);
            for item in items {
                table.row(vec![dash(&item["intent_id"]), dash(&item["state"])]);
            }
            Some(table.render("This Repo has no integration intents yet."))
        }
        "repo.list" => {
            let registrations = rows_of(value, "repos")?;
            let mut table = Table::new(&["repo", "name", "platform", "lifecycle", "version"]);
            for item in registrations {
                table.row(vec![
                    dash(&item["repo_id"]),
                    dash(&item["prepared"]["request"]["name"]),
                    dash(&item["prepared"]["request"]["platform"]),
                    dash(&item["lifecycle"]),
                    cell(&item["version"]),
                ]);
            }
            Some(table.render("No Repos registered yet."))
        }
        _ => None,
    }
}
