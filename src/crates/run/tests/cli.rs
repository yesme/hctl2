//! Real hctl2 -> authenticated control socket -> Run handlers, with pinned Dagu.
use super::*;
struct Cli {
    temp: Temp,
    dagu: PathBuf,
}
impl Cli {
    fn new() -> Self {
        let cli = Self {
            temp: Temp::new(),
            dagu: find_binary(Path::new(&std::env::var("HCTL2_TEST_DAGU_CACHE").unwrap())),
        };
        cli.ok(&["init", "--secret-backend", "user-file"]);
        let mut store = Store::open(&cli.temp.0).unwrap();
        let mut p = value_record(target("project", "P").key, 1, &json!({})).unwrap();
        p.data = RecordData::Project {
            repo_id: "repo".into(),
            settings: store::ProjectSettings {
                publish_review_requires_confirmation: true,
                selection_policy: json!({}),
            },
            archived: false,
        };
        let mut q = p.clone();
        q.key = key(Scope::Project("Q".into()), "project", "Q");
        seed(
            &mut store,
            &[
                p,
                q,
                task_record("P", "T"),
                task_record("P", "U"),
                task_record("Q", "T"),
            ],
            "cli-setup",
        );
        // The package-7 test double writes internal evidence, never command input.
        let p = proof();
        let scoped = chat::owner(&actor(), "P").unwrap();
        let material = store
            .save_material(
                store.generation(),
                &scoped,
                &Scope::Project("P".into()),
                "readback-fixture",
                "body",
                b"readback body",
            )
            .unwrap();
        let mut read = value_record(target("workflow_readback", "r").key, 1, &p.read).unwrap();
        read.materials.push(material.clone());
        let evidence = value_record(
            target("workflow_invocation_evidence", "read").key,
            1,
            &p.actual,
        )
        .unwrap();
        let room=value_record(target("room_binding","source").key,1,&json!({"project_id":"P","id":"source","name":"source","server":{"binding":target("chat_server","local"),"url":"http://127.0.0.1:1","server_name":"fixture","sender":"fixture"},"matrix_room_id":null,"participants":[],"brief":null,"origin":null})).unwrap();
        let source_material = store
            .save_material(
                store.generation(),
                &scoped,
                &Scope::Project("P".into()),
                "readback-fixture",
                "source-body",
                b"discussion",
            )
            .unwrap();
        let mut source = value_record(
            target("chat_source_reference", "source-event").key,
            1,
            &chat::Source::Message {
                binding: target("room_binding", "source"),
                event_id: "$event".into(),
                content_digest: sha("discussion"),
            },
        )
        .unwrap();
        source.materials.push(source_material.clone());
        let mut producer_spec = p.actual.spec.document.clone();
        producer_spec.owner.id = "producer".into();
        producer_spec.profile.digest = sha("producer-profile");
        let producer=value_record(target("room_invocation","producer").key,1,&json!({"spec":Sealed::new(producer_spec).unwrap(),"preview":{"input":{"room_id":"source"}}})).unwrap();
        let command = Command {
            command_id: "readback-fixture".into(),
            idempotency_key: "readback-fixture".into(),
            actor: scoped.0.clone(),
            target: key(Scope::Project("P".into()), "fixture", "readback"),
            expected: Expected::Absent,
            binding: target("module", "run"),
            input_digest: Command::digest_input("fixture", &json!({})).unwrap(),
            operation: "fixture".into(),
            input: json!({}),
        };
        store
            .submit(store.generation(), &scoped, &command, None, |tx| {
                tx.admit_material(&material)?;
                tx.admit_material(&source_material)?;
                tx.put(&room)?;
                tx.put(&source)?;
                tx.put(&producer)?;
                tx.put(&read)?;
                tx.put(&evidence)?;
                Ok(json!({}))
            })
            .unwrap();
        drop(store);
        cli.ok(&["start"]);
        cli
    }
    fn stop(&self) -> bool {
        let _ = self.run(&["stop"]);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            match foundation::ExclusiveFileLock::try_acquire(&self.temp.0.join("control.lock")) {
                Ok(lock) => {
                    drop(lock);
                    return true;
                }
                Err(error) if error.is_lock_contended() => {}
                Err(error) => panic!("stop fixture: {error}"),
            }
            if std::time::Instant::now() >= deadline {
                return false;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    fn run(&self, args: &[&str]) -> (bool, Value) {
        let output = std::process::Command::new(std::env::var("CARGO_BIN_EXE_hctl2").unwrap())
            .env(
                "HCTL2_CONTROL_BIN",
                std::env::var("CARGO_BIN_EXE_hctl2-control").unwrap(),
            )
            .env("HCTL2_DAGU_BIN", &self.dagu)
            .args(["--root", self.temp.0.to_str().unwrap(), "--json"])
            .args(args)
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&output.stdout);
        let value = serde_json::from_str(&text).unwrap_or_else(
            |_| json!({"stdout":text,"stderr":String::from_utf8_lossy(&output.stderr)}),
        );
        (output.status.success(), value)
    }
    fn ok(&self, args: &[&str]) -> Value {
        let (ok, value) = self.run(args);
        assert!(ok, "{args:?}: {value}");
        value
    }
    fn preview(&self, input: &Input) -> (PathBuf, Value) {
        let path = self.temp.0.join(format!("{}.json", input.key));
        std::fs::write(&path, serde_json::to_vec(&input.action).unwrap()).unwrap();
        let result = self.ok(&[
            "workflow",
            input.action.kind(),
            "--input",
            path.to_str().unwrap(),
            "--key",
            &input.key,
        ]);
        (path, result)
    }
    fn write(&self, input: &Input) -> Value {
        let (path, preview) = self.preview(input);
        assert!(preview["preview_token"].is_string());
        self.ok(&[
            "workflow",
            input.action.kind(),
            "--input",
            path.to_str().unwrap(),
            "--key",
            &input.key,
            "--preview-token",
            preview["preview_token"].as_str().unwrap(),
        ])
    }
}
impl Drop for Cli {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
#[test]
fn workflow_real_cli_recovers_task_versions_and_rejects_foreign_or_unknown_input() {
    let cli = Cli::new();
    for kind in ["fork", "join", "noop"] {
        let mut input = serde_json::to_value(register(kind, workflow()).action).unwrap();
        input["workflow"]["graph"]["nodes"][0]["operation"] = json!({"kind":kind,"seconds":10});
        let path = cli.temp.0.join(format!("bad-{kind}.json"));
        std::fs::write(&path, serde_json::to_vec(&input).unwrap()).unwrap();
        let (ok, value) = cli.run(&[
            "workflow",
            "register",
            "--input",
            path.to_str().unwrap(),
            "--key",
            kind,
        ]);
        assert!(!ok, "unknown {kind} field accepted: {value}");
        assert!(value["error"].is_object(), "{value}");
    }
    let mut foreign = register("foreign-task", workflow());
    if let Action::Register { task, .. } = &mut foreign.action {
        *task = Some(chat::reference(&task_record("Q", "T")));
    }
    let path = cli.temp.0.join("foreign.json");
    std::fs::write(&path, serde_json::to_vec(&foreign.action).unwrap()).unwrap();
    let (ok, error) = cli.run(&[
        "workflow",
        "register",
        "--input",
        path.to_str().unwrap(),
        "--key",
        &foreign.key,
    ]);
    assert!(!ok, "foreign Task accepted: {error}");
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("reference belongs to another Project")
    );
    assert_eq!(cli.ok(&["workflow", "list", "P"])["workflows"], json!([]));

    let mut input = register("task-first", workflow());
    if let Action::Register { task, .. } = &mut input.action {
        *task = Some(target("task", "T"));
    }
    let (path, preview) = cli.preview(&input);
    assert_eq!(
        cli.ok(&["workflow", "list", "P", "--task", "T"])["workflows"],
        json!([])
    );
    cli.ok(&[
        "workflow",
        "register",
        "--input",
        path.to_str().unwrap(),
        "--key",
        &input.key,
        "--preview-token",
        preview["preview_token"].as_str().unwrap(),
    ]);
    // The Task may advance; saved versions remain reachable by its stable identity.
    assert!(cli.stop());
    let mut store = Store::open(&cli.temp.0).unwrap();
    let mut task = task_record("P", "T");
    task.version = 2;
    seed(&mut store, &[task.clone()], "task-advanced");
    drop(store);
    cli.ok(&["start"]);
    input.key = "task-second".into();
    if let Action::Register {
        supersedes,
        task: target,
        workflow,
        ..
    } = &mut input.action
    {
        *supersedes = Some(1);
        *target = Some(chat::reference(&task));
        workflow.context = "second saved version".into();
    }
    cli.write(&input);
    input.key = "unlinked-third".into();
    if let Action::Register {
        supersedes, task, ..
    } = &mut input.action
    {
        *supersedes = Some(2);
        *task = None;
    }
    cli.write(&input);
    let mut unrelated = register("another-task", workflow());
    if let Action::Register {
        workflow_id, task, ..
    } = &mut unrelated.action
    {
        *workflow_id = "unrelated".into();
        *task = Some(target("task", "U"));
    }
    cli.write(&unrelated);
    if let Action::Register {
        project_id,
        workflow_id,
        task,
        ..
    } = &mut unrelated.action
    {
        *project_id = "Q".into();
        *workflow_id = "foreign".into();
        *task = Some(chat::reference(&task_record("Q", "T")));
    }
    unrelated.key = "other-project".into();
    cli.write(&unrelated);
    let listed = cli.ok(&["workflow", "list", "P", "--task", "T"]);
    let versions = listed["workflows"].as_array().unwrap();
    assert_eq!(versions.len(), 2, "{listed}");
    for (entry, revision) in versions.iter().zip([1, 2]) {
        assert_eq!(entry["workflow_id"], "plan");
        assert_eq!(entry["revision"], revision);
        assert_eq!(
            entry["task"]["version"],
            serde_json::to_value(Version::State(revision)).unwrap()
        );
        let shown = cli.ok(&[
            "workflow",
            "show",
            "P",
            entry["workflow_id"].as_str().unwrap(),
            "--revision",
            &revision.to_string(),
        ]);
        assert_eq!(shown["reference"], entry["revision_reference"]);
        assert_eq!(shown["revision"]["revision"], revision);
    }
    assert_eq!(
        cli.ok(&["workflow", "show", "P", "plan"])["revision"]["revision"],
        3
    );
    let (ok, _) = cli.run(&["workflow", "list", "P", "--task", "absent"]);
    assert!(!ok, "missing Task silently accepted");
    println!(
        "real CLI rejects unknown structure fields and existing foreign Task; Task query retrieves exact historical versions after Task and workflow advance"
    );
}
#[test]
fn workflow_real_cli_previews_then_registers_compiles_and_approves() {
    let cli = Cli::new();
    let input = register("cli-first", workflow());
    let (path, preview) = cli.preview(&input);
    assert_eq!(cli.ok(&["workflow", "list", "P"])["workflows"], json!([]));
    let registered = cli.ok(&[
        "workflow",
        "register",
        "--input",
        path.to_str().unwrap(),
        "--key",
        &input.key,
        "--preview-token",
        preview["preview_token"].as_str().unwrap(),
    ]);
    assert_eq!(registered["revision"], 1);
    assert_eq!(registered["started"], false);
    let replay = cli.write(&input);
    assert_eq!(registered, replay);
    let shown = cli.ok(&["workflow", "show", "P", "plan"]);
    assert_eq!(shown["startable"], false);
    assert_eq!(
        shown["revision"]["workflow"]["graph"]["nodes"][0]["operation"]["criterion"]["kind"],
        "mechanical"
    );
    let compiled = cli.write(&action("compile", 1));
    assert_eq!(compiled["validated"], true);
    let mut approval = action("approve", 1);
    if let Action::Approve { readback, .. } = &mut approval.action {
        *readback = Some(target("workflow_readback", "r"));
    }
    let approved = cli.write(&approval);
    assert_eq!(approved["proof"]["source_room_roster"], "not_applicable");
    assert_eq!(
        cli.ok(&["workflow", "show", "P", "plan"])["startable"],
        true
    );
    assert_eq!(
        cli.ok(&["workflow", "list", "P"])["workflows"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    // Confirmation is tied to exact action bytes, not just a reused token.
    let mut next = register("cli-next", workflow());
    if let Action::Register { supersedes, .. } = &mut next.action {
        *supersedes = Some(1);
    }
    let (path, preview) = cli.preview(&next);
    let mut changed = serde_json::to_value(&next.action).unwrap();
    changed["workflow"]["context"] = json!("changed after preview");
    std::fs::write(&path, serde_json::to_vec(&changed).unwrap()).unwrap();
    let (ok, _) = cli.run(&[
        "workflow",
        "register",
        "--input",
        path.to_str().unwrap(),
        "--key",
        &next.key,
        "--preview-token",
        preview["preview_token"].as_str().unwrap(),
    ]);
    assert!(!ok);
    println!(
        "real hctl2 workflow list/show/register/compile/approve: preview + confirm passed; pinned native validate passed; isolated readback test double admitted"
    );
}
#[test]
fn workflow_real_cli_rejects_cycle_dependency_and_old_delivered_snapshot() {
    let cli = Cli::new();
    for (name, w) in [
        (
            "cycle",
            with_graph(
                json!({"nodes":[noop("a"),noop("b")],"edges":[edge("a","b"),edge("b","a")]}),
            ),
        ),
        (
            "dependency",
            with_graph(
                json!({"nodes":[external("a"),external("b")],"edges":[{"from":"a","to":"b","artifact":{"output":"absent","kind":"receipt"}}]}),
            ),
        ),
    ] {
        let input = register(name, w);
        let path = cli.temp.0.join(format!("{name}.json"));
        std::fs::write(&path, serde_json::to_vec(&input.action).unwrap()).unwrap();
        let (ok, value) = cli.run(&[
            "workflow",
            "register",
            "--input",
            path.to_str().unwrap(),
            "--key",
            name,
        ]);
        assert!(!ok, "{name}: {value}");
    }
    let mut changed = register("new-snapshot", workflow());
    if let Action::Register { snapshot, .. } = &mut changed.action {
        snapshot
            .as_mut()
            .unwrap()
            .unresolved
            .push("new snapshot".into());
    }
    cli.write(&changed);
    cli.write(&action("compile", 1));
    let mut approval = action("approve", 1);
    if let Action::Approve { readback, .. } = &mut approval.action {
        *readback = Some(target("workflow_readback", "r"));
    }
    let path = cli.temp.0.join("approve.json");
    std::fs::write(&path, serde_json::to_vec(&approval.action).unwrap()).unwrap();
    let (ok, value) = cli.run(&[
        "workflow",
        "approve",
        "--input",
        path.to_str().unwrap(),
        "--key",
        &approval.key,
    ]);
    assert!(!ok, "{value}");
    println!("real CLI rejected static cycle, unlanded artifact and stale delivered snapshot");
}

#[test]
fn approval_real_cli_resolves_actual_producers_and_current_source_roster() {
    let cli = Cli::new();
    let mut input = register("source-plan", workflow());
    if let Action::Register { provenance, .. } = &mut input.action {
        let source = provenance.as_mut().unwrap();
        source.source_chain.push(SourceMessage {
            room_id: "source".into(),
            event: target("chat_source_reference", "source-event"),
        });
        source.production = Production::Invocations {
            references: vec![target("room_invocation", "producer")],
        };
    }
    cli.write(&input);
    cli.write(&action("compile", 1));
    let mut approve = action("approve", 1);
    if let Action::Approve { readback, .. } = &mut approve.action {
        *readback = Some(target("workflow_readback", "r"));
    }
    assert_eq!(
        cli.write(&approve)["proof"]["profile_comparison"],
        "different_from_all_producers"
    );
    // Change the real current roster after the first approval, then re-preview.
    assert!(cli.stop(), "control did not release writer lock");
    let mut store = Store::open(&cli.temp.0).unwrap();
    let mut member = proof().actual.selection;
    member.version = Version::State(9);
    seed(
        &mut store,
        &[value_record(target("room_roster", "source").key, 1, &vec![member]).unwrap()],
        "roster-change",
    );
    drop(store);
    cli.ok(&["start"]);
    approve.key = "recheck-roster".into();
    let path = cli.temp.0.join("roster-approve.json");
    std::fs::write(&path, serde_json::to_vec(&approve.action).unwrap()).unwrap();
    let (ok, value) = cli.run(&[
        "workflow",
        "approve",
        "--input",
        path.to_str().unwrap(),
        "--key",
        &approve.key,
    ]);
    assert!(!ok, "{value}");
    assert_eq!(value["error"]["code"], "WORKFLOW_READBACK");
}
