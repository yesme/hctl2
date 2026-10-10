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

/// A review-publish preview: one frozen intent about to push a branch and open the review
/// request for it. The policy that froze `requires_human_confirmation` is why this stops.
pub(crate) fn review_preview(token: &str, effect: &Value) -> String {
    let updates = if effect["allow_update"] == Value::Bool(true) {
        "allowed: a later round may update the request this round opens".to_owned()
    } else {
        "no: the first round creates the request, later revisions are refused".to_owned()
    };
    let sections = PreviewSections {
        title: "Review publish preview".to_owned(),
        object: vec![
            ("repo".to_owned(), dash(&effect["repo_id"])),
            ("publish intent".to_owned(), dash(&effect["intent_id"])),
            ("round".to_owned(), cell(&effect["round"])),
            ("ChangeSet".to_owned(), dash(&effect["change_set_id"])),
            (
                "ChangeSet Revision".to_owned(),
                dash(&effect["change_set_revision_id"]),
            ),
            ("push to branch".to_owned(), dash(&effect["branch"])),
            (
                "review request target".to_owned(),
                dash(&effect["target_branch"]),
            ),
            ("updates an existing request".to_owned(), updates),
        ],
        why: vec![
            "this intent was frozen to wait for a human; confirming is the release".to_owned(),
            "it authorizes a platform write: push the branch, then create or update the review request — not a merge".to_owned(),
        ],
        after: vec![
            "persists the release, then runs both stages in the background: push the commit, then create or update the review request, each confirmed by reading the platform back".to_owned(),
            "records the pushed commit, the review request index and this ChangeSet's revision mapping in the control store".to_owned(),
            "undo: the push cannot be taken back; a review request that was created is closed on the platform, not by this CLI".to_owned(),
        ],
    };
    sections.render(&confirm_command(token))
}

/// The judge of one acceptance item as one label.
fn judge_label(judge: &Value) -> String {
    match judge["kind"].as_str() {
        Some("hctl2_tool") => "hctl2-tool".to_owned(),
        Some("adapter") => format!("adapter {}", dash(&judge["port"])),
        Some("gate") => format!("gate seat {}", dash(&judge["seat"])),
        Some("human") => format!("human {}", dash(&judge["actor"])),
        other => other.unwrap_or("unknown").to_owned(),
    }
}

/// A completion preview: the acceptance items about to be bound to a Task Completion
/// Receipt, each with the evidence channel and the judge that passed it.
pub(crate) fn completion_preview(token: &str, effect: &Value) -> String {
    let result = &effect["result"];
    let action = &effect["input"]["action"];
    let mut object = vec![
        ("project".to_owned(), dash(&result["project_id"])),
        ("task".to_owned(), dash(&result["task_id"])),
        ("Task Revision".to_owned(), cell(&action["revision_number"])),
        (
            "lifecycle version".to_owned(),
            format!(
                "{} -> {}",
                cell(&action["lifecycle_version"]),
                cell(&result["lifecycle_version"]),
            ),
        ),
        (
            "Task Completion Receipt".to_owned(),
            dash(&result["receipt_id"]),
        ),
    ];
    if let Some(items) = result["items"].as_array() {
        for (index, item) in items.iter().enumerate() {
            object.push((
                format!("item {}", index + 1),
                format!(
                    "{} · {} · {} — {}",
                    dash(&item["grade"]),
                    dash(&item["validation_level"]),
                    judge_label(&item["judge"]),
                    dash(&item["text"]),
                ),
            ));
        }
    }
    let mut after = vec![
        "writes the Task Completion Receipt and advances the Task to completed in the same control-store transaction".to_owned(),
        "the receipt binds exactly the Task Revision and the items above; nothing is rewritten later".to_owned(),
    ];
    if effect["effects"]
        .as_array()
        .is_some_and(|effects| !effects.is_empty())
    {
        after.push(
            "queues the listed external writeback and reads it back in the background; a failed writeback only marks attention".to_owned(),
        );
    }
    after.push(
        "undo: `hctl2 task reopen` returns the Task to open under a new lifecycle version; the Completion Receipt stays recorded".to_owned(),
    );
    let sections = PreviewSections {
        title: "Completion preview".to_owned(),
        object,
        why: vec![
            "completing is terminal for this lifecycle version: the receipt below is what the Task is judged by".to_owned(),
            "each item's judge and evidence channel are as listed; a human item is the submitting person's own verdict".to_owned(),
        ],
        after,
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
        "task.show" => Some(task_show(value)),
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

/// `task show` for a person: what the Task is for, where it stands, what blocks it, which
/// harness is on it, the evidence so far, and the one command that moves it on.
fn task_show(value: &Value) -> String {
    let data = &value["data"];
    let progress = &value["progress"];
    let project = cell(&data["project_id"]);
    let task = cell(&data["id"]);
    let contract = if data["revision"].is_null() {
        "no contract adopted".to_owned()
    } else {
        format!("contract revision {}", cell(&data["revision"]["number"]))
    };
    let blockers = value["request_blockers"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let latest = progress["invocations"]
        .as_array()
        .and_then(|items| items.last())
        .cloned()
        .unwrap_or(Value::Null);
    let harness = if latest.is_null() {
        "none dispatched yet".to_owned()
    } else {
        format!(
            "{} ({}) — invocation {} {}",
            dash(&latest["harness"]),
            dash(&latest["model"]),
            short_id(&cell(&latest["invocation_id"])),
            dash(&latest["state"]),
        )
    };
    let revision = &progress["revision"];
    let review = &progress["review"];
    let integration = &progress["integration"];
    let completion = &progress["completion"];
    let mut evidence = Vec::new();
    if !revision.is_null() {
        evidence.push(format!(
            "version {}",
            short_id(&cell(&revision["change_set_revision_id"]))
        ));
    }
    if !review.is_null() {
        evidence.push(match review["review_request"].as_u64() {
            Some(index) => format!("review request #{index} ({})", dash(&review["state"])),
            None => format!("review {}", dash(&review["state"])),
        });
    }
    if !integration.is_null() {
        evidence.push(match integration["receipt_id"].as_str() {
            Some(receipt) => format!("Integration Receipt {}", short_id(receipt)),
            None => format!("integration {}", dash(&integration["state"])),
        });
    }
    if !completion.is_null() {
        evidence.push(format!(
            "Completion Receipt {}",
            short_id(&cell(&completion["receipt_id"]))
        ));
    }
    let blocking = if !blockers.is_empty() {
        format!("{} pending request(s)", blockers.len())
    } else if data["revision"].is_null() {
        "completion needs an adopted contract".to_owned()
    } else {
        "nothing".to_owned()
    };
    let next = next_step(
        &project,
        &task,
        data,
        &blockers,
        &latest,
        review,
        integration,
        completion,
    );
    [
        format!("Task {} — {}", short_id(&task), dash(&data["title"])),
        format!("  goal      {}", dash(&data["title"])),
        format!("  state     {}; {contract}", dash(&data["lifecycle"])),
        format!("  blocking  {blocking}"),
        format!("  harness   {harness}"),
        format!(
            "  evidence  {}",
            if evidence.is_empty() {
                "none yet".to_owned()
            } else {
                evidence.join(", ")
            }
        ),
        format!("  next      {next}"),
    ]
    .join("\n")
}

#[allow(clippy::too_many_arguments)]
fn next_step(
    project: &str,
    task: &str,
    data: &Value,
    blockers: &[Value],
    latest: &Value,
    review: &Value,
    integration: &Value,
    completion: &Value,
) -> String {
    let running = |state: &Value| {
        matches!(
            state.as_str(),
            Some("pending" | "running" | "waiting_input")
        )
    };
    if data["lifecycle"] == "completed" || !completion.is_null() {
        return "nothing — the Task is completed".into();
    }
    if !blockers.is_empty() {
        return format!("answer the pending request: hctl2 project pending {project}");
    }
    if data["revision"].is_null() {
        return format!(
            "adopt a contract for Task {}: hctl2 task adopt --key <key> --input <adoption.json>",
            short_id(task)
        );
    }
    if latest.is_null() {
        return "dispatch from the Room with this task_id: hctl2 invocation preview --input <invocation.json> --key <key>".into();
    }
    if running(&latest["state"]) {
        return format!(
            "wait for {} to return (invocation {})",
            dash(&latest["harness"]),
            dash(&latest["state"])
        );
    }
    if review.is_null() && integration.is_null() {
        return format!(
            "inspect the invocation: hctl2 invocation show {project} {}",
            cell(&latest["invocation_id"])
        );
    }
    match review["state"].as_str() {
        Some("pending_human") => {
            return format!(
                "release publishing for review: hctl2 review publish {} {}",
                cell(&latest["write"]["repo_id"]),
                cell(&review["intent_id"])
            );
        }
        Some("pending" | "unknown") => {
            return format!("wait for publishing (review {})", dash(&review["state"]));
        }
        _ => {}
    }
    match integration["state"].as_str() {
        None => format!(
            "integrate the published version into {}: hctl2 integration preview --input <integration.json> --key <key>",
            dash(&review["target_branch"])
        ),
        Some("succeeded") => "complete the Task citing the Integration Receipt: hctl2 task complete --input <completion.json> --key <key>".to_owned(),
        Some(state) => format!("wait for integration ({state})"),
    }
}

/// Long ids shortened for a person: the kind prefix plus twelve characters.
fn short_id(id: &str) -> String {
    match id.split_once('-') {
        Some((kind, rest)) if kind.len() <= 11 && rest.len() > 12 => {
            format!("{kind}-{}", rest.chars().take(12).collect::<String>())
        }
        _ if id.len() > 16 => id.chars().take(16).collect(),
        _ => id.to_owned(),
    }
}
