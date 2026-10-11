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
/// harness is on it, the evidence so far, and the one command that moves it on. Everything
/// is about the newest dispatch and the newest admitted version, in the Store's event order.
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
    let last = |field: &str| {
        progress[field]
            .as_array()
            .and_then(|items| items.last())
            .cloned()
            .unwrap_or(Value::Null)
    };
    let latest = last("invocations");
    let revision = last("revisions");
    let publication = &revision["publication"];
    let integration = &revision["integration"];
    let completion = &progress["completion"]["current"];
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
    let mut evidence = Vec::new();
    if !revision.is_null() {
        evidence.push(format!(
            "version {}",
            short_id(&cell(&revision["change_set_revision_id"]))
        ));
        evidence.push(match publication["review_request"].as_u64() {
            Some(index) => format!("review request #{index} ({})", dash(&publication["state"])),
            None => format!("review {}", dash(&publication["state"])),
        });
        if !integration.is_null() {
            evidence.push(match integration["receipt_id"].as_str() {
                Some(receipt) => format!("Integration Receipt {}", short_id(receipt)),
                None => format!("integration {}", dash(&integration["state"])),
            });
        }
    }
    if let Some(receipt) = completion["receipt_id"].as_str() {
        evidence.push(format!("Completion Receipt {}", short_id(receipt)));
    } else if let Some(count) = progress["completion"]["history"]
        .as_array()
        .map(Vec::len)
        .filter(|n| *n > 0)
    {
        evidence.push(format!(
            "{count} earlier Completion Receipt(s), reopened since"
        ));
    }
    let attention = [&publication["attention"], &integration["attention"]]
        .into_iter()
        .filter_map(|a| a.as_str())
        .collect::<Vec<_>>();
    let blocking = if !blockers.is_empty() {
        format!("{} pending request(s)", blockers.len())
    } else if data["revision"].is_null() {
        "completion needs an adopted contract".to_owned()
    } else if !attention.is_empty() && data["lifecycle"] != "completed" {
        attention.join(", ")
    } else {
        "nothing".to_owned()
    };
    let next = next_step(&project, &task, data, &blockers, &latest, &revision);
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

/// The one command that moves the Task on, from its current lifecycle and the newest
/// dispatch and version. Failures and unknown outcomes point at the record to read, never at
/// a step that would assume success.
fn next_step(
    project: &str,
    task: &str,
    data: &Value,
    blockers: &[Value],
    latest: &Value,
    revision: &Value,
) -> String {
    if data["lifecycle"] == "completed" {
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
    let invocation = cell(&latest["invocation_id"]);
    if matches!(
        latest["state"].as_str(),
        Some("pending" | "running" | "waiting_input")
    ) {
        return format!(
            "wait for {} to return (invocation {})",
            dash(&latest["harness"]),
            dash(&latest["state"])
        );
    }
    if revision.is_null() || latest["write"].is_null() && latest["state"] != "completed" {
        return format!(
            "the last invocation is {}: hctl2 invocation show {project} {invocation}",
            dash(&latest["state"])
        );
    }
    let publication = &revision["publication"];
    let repo = cell(&publication["repo_id"]);
    let intent = cell(&publication["intent_id"]);
    let show_review = format!("hctl2 review show {repo} {intent}");
    match publication["state"].as_str() {
        Some("published") => {}
        Some("pending_human") => {
            return format!("release publishing for review: hctl2 review publish {repo} {intent}");
        }
        Some("publishing" | "queued") => {
            return format!(
                "wait for publishing (review {})",
                dash(&publication["state"])
            );
        }
        Some("unknown" | "failed") => {
            return format!(
                "publishing is {} ({}): {show_review}",
                dash(&publication["state"]),
                dash(&publication["attention"])
            );
        }
        Some("update_not_allowed") => {
            return format!(
                "version {} is not published — the frozen policy only creates the review request: {show_review}",
                short_id(&cell(&revision["change_set_revision_id"]))
            );
        }
        _ => {
            return format!(
                "inspect the version: hctl2 changeset show {} {}",
                cell(&revision["repo_id"]),
                cell(&revision["change_set_id"])
            );
        }
    }
    let integration = &revision["integration"];
    match integration["state"].as_str() {
        None => format!(
            "integrate the published version into {}: hctl2 integration preview --input <integration.json> --key <key>",
            dash(&publication["target_branch"])
        ),
        Some("succeeded") => "complete the Task citing the Integration Receipt: hctl2 task complete --input <completion.json> --key <key>".to_owned(),
        Some("pending") => "wait for integration (pending)".to_owned(),
        Some(state) => format!(
            "integration is {state} ({}): hctl2 integration show {} {}",
            dash(&integration["attention"]),
            cell(&revision["repo_id"]),
            cell(&integration["intent_id"])
        ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn show(data: Value, progress: Value) -> (String, String) {
        let text = task_show(&json!({"data": data, "progress": progress, "request_blockers": []}));
        let next = text
            .lines()
            .find_map(|l| l.trim_start().strip_prefix("next"))
            .unwrap()
            .trim()
            .to_owned();
        (text, next)
    }

    fn open_task() -> Value {
        json!({"id": "t-1", "project_id": "P", "title": "change it", "lifecycle": "open",
            "revision": {"number": 1}})
    }

    fn write_call(state: &str) -> Value {
        json!({"invocation_id": "invocation-a", "state": state, "harness": "claude-code",
            "model": "m", "write": {"repo_id": "R", "change_set_id": "cs-1"}})
    }

    fn version(publication: Value, integration: Value) -> Value {
        json!({"repo_id": "R", "change_set_id": "cs-1", "change_set_revision_id": "csr-1",
            "publication": publication, "integration": integration})
    }

    fn published() -> Value {
        json!({"state": "published", "intent_id": "pub", "repo_id": "R", "branch": "hctl2/cs-1",
            "target_branch": "main", "review_request": 7})
    }

    /// A reopened Task keeps its old Completion Receipt as history; it is not completed.
    #[test]
    fn a_reopened_task_is_not_reported_completed_by_a_historical_receipt() {
        let (text, next) = show(
            open_task(),
            json!({"invocations": [write_call("completed")],
                "revisions": [version(published(), json!({"state": "succeeded", "receipt_id": "receipt-1", "intent_id": "i"}))],
                "completion": {"current": null, "history": ["completion-1"]}}),
        );
        assert!(next.starts_with("complete the Task"), "{text}");
        assert!(text.contains("earlier Completion Receipt"), "{text}");
        assert!(text.contains("state     open"), "{text}");
    }

    /// A failed or unknown publication is never followed by advice to integrate.
    #[test]
    fn failed_or_unknown_publication_points_at_the_review_not_at_integration() {
        for state in ["failed", "unknown"] {
            let publication = json!({"state": state, "intent_id": "pub", "repo_id": "R",
                "attention": "PUSH_FAILED", "target_branch": "main"});
            let (text, next) = show(
                open_task(),
                json!({"invocations": [write_call("completed")],
                    "revisions": [version(publication, Value::Null)],
                    "completion": {"current": null, "history": []}}),
            );
            assert_eq!(
                next,
                format!("publishing is {state} (PUSH_FAILED): hctl2 review show R pub"),
                "{text}"
            );
            assert!(text.contains("blocking  PUSH_FAILED"), "{text}");
        }
    }

    /// A failed integration names its record; it is not something to wait for.
    #[test]
    fn a_failed_integration_names_the_intent_to_read() {
        let (text, next) = show(
            open_task(),
            json!({"invocations": [write_call("completed")],
                "revisions": [version(published(), json!({"state": "failed", "intent_id": "merge-1", "attention": "SOURCE_HEAD_MISMATCH"}))],
                "completion": {"current": null, "history": []}}),
        );
        assert_eq!(
            next, "integration is failed (SOURCE_HEAD_MISMATCH): hctl2 integration show R merge-1",
            "{text}"
        );
    }

    /// A later read-only dispatch does not erase the Repo in the release command.
    #[test]
    fn a_later_read_only_dispatch_keeps_the_release_command_whole() {
        let pending = json!({"state": "pending_human", "intent_id": "pub", "repo_id": "R",
            "branch": "hctl2/cs-1", "target_branch": "main"});
        let read_only = json!({"invocation_id": "invocation-b", "state": "completed",
            "harness": "codex-cli", "model": "m", "write": null});
        let (text, next) = show(
            open_task(),
            json!({"invocations": [write_call("completed"), read_only],
                "revisions": [version(pending, Value::Null)],
                "completion": {"current": null, "history": []}}),
        );
        assert_eq!(
            next, "release publishing for review: hctl2 review publish R pub",
            "{text}"
        );
    }

    /// A version a create-only policy refused to publish is said so, not offered for merge.
    #[test]
    fn a_version_refused_by_a_create_only_policy_is_not_offered_for_integration() {
        let refused = json!({"state": "update_not_allowed", "intent_id": "pub", "repo_id": "R",
            "attention": "UPDATE_NOT_ALLOWED", "target_branch": "main"});
        let (text, next) = show(
            open_task(),
            json!({"invocations": [write_call("completed")],
                "revisions": [version(refused, Value::Null)],
                "completion": {"current": null, "history": []}}),
        );
        assert!(next.starts_with("version csr-1 is not published"), "{text}");
    }

    /// The newest dispatch decides the harness line and waiting, whatever its id.
    #[test]
    fn the_newest_dispatch_decides_the_harness_line() {
        let older = json!({"invocation_id": "invocation-zzz", "state": "completed",
            "harness": "script", "model": "m", "write": {"repo_id": "R", "change_set_id": "cs-1"}});
        let newer = json!({"invocation_id": "invocation-aaa", "state": "running",
            "harness": "claude-code", "model": "m", "write": {"repo_id": "R", "change_set_id": "cs-1"}});
        let (text, next) = show(
            open_task(),
            json!({"invocations": [older, newer],
                "revisions": [version(published(), Value::Null)],
                "completion": {"current": null, "history": []}}),
        );
        assert!(text.contains("harness   claude-code"), "{text}");
        assert_eq!(next, "wait for claude-code to return (invocation running)");
    }
}
