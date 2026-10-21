use super::*;
struct Cli {
    temp: Temp,
}
impl Cli {
    fn new() -> (Self, Start) {
        let cli = Self { temp: Temp::new() };
        cli.ok(&["init", "--secret-backend", "user-file"]);
        let mut store = Store::open(&cli.temp.0).unwrap();
        let spec = setup(&mut store);
        drop(store);
        cli.ok(&["start"]);
        (cli, spec)
    }
    fn run(&self, args: &[&str]) -> (bool, Value) {
        let output = std::process::Command::new(std::env::var("CARGO_BIN_EXE_hctl2").unwrap())
            .env(
                "HCTL2_CONTROL_BIN",
                std::env::var("CARGO_BIN_EXE_hctl2-control").unwrap(),
            )
            .args(["--root", self.temp.0.to_str().unwrap(), "--json"])
            .args(args)
            .output()
            .unwrap();
        let value=serde_json::from_slice(&output.stdout).unwrap_or_else(|_|json!({"stdout":String::from_utf8_lossy(&output.stdout),"stderr":String::from_utf8_lossy(&output.stderr)}));
        (output.status.success(), value)
    }
    fn ok(&self, args: &[&str]) -> Value {
        let (ok, value) = self.run(args);
        assert!(ok, "{args:?}: {value}");
        value
    }
    fn path(&self, input: &Input) -> PathBuf {
        let path = self.temp.0.join(format!("{}.json", input.key));
        std::fs::write(&path, serde_json::to_vec(&input.action).unwrap()).unwrap();
        path
    }
    fn preview(&self, input: &Input) -> Value {
        let path = self.path(input);
        self.ok(&[
            "run",
            input.action.kind(),
            "--input",
            path.to_str().unwrap(),
            "--key",
            &input.key,
        ])
    }
    fn write(&self, input: &Input) -> Value {
        let path = self.path(input);
        let preview = self.preview(input);
        self.ok(&[
            "run",
            input.action.kind(),
            "--input",
            path.to_str().unwrap(),
            "--key",
            &input.key,
            "--preview-token",
            preview["preview_token"].as_str().unwrap(),
        ])
    }
    fn rejected(&self, input: &Input, code: &str) {
        let path = self.path(input);
        let (ok, result) = self.run(&[
            "run",
            input.action.kind(),
            "--input",
            path.to_str().unwrap(),
            "--key",
            &input.key,
        ]);
        assert!(!ok, "{result}");
        assert_eq!(result["error"]["code"], code, "{result}");
    }
    fn show(&self, id: &str) -> Value {
        self.ok(&["run", "show", "P", id])
    }
    fn version(&self, id: &str) -> i64 {
        self.show(id)["reference"]["version"]["state"]
            .as_i64()
            .unwrap()
    }
    fn stop(&self) {
        let _ = self.run(&["stop"]);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            match foundation::ExclusiveFileLock::try_acquire(&self.temp.0.join("control.lock")) {
                Ok(lock) => {
                    drop(lock);
                    break;
                }
                Err(e) if e.is_lock_contended() => {}
                Err(e) => panic!("{e}"),
            };
            assert!(std::time::Instant::now() < deadline, "control did not stop");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    fn engine(&self, id: &str, op: &str, fake: &Fake) {
        self.stop();
        let mut store = Store::open(&self.temp.0).unwrap();
        drive(&mut store, id, op, fake);
        drop(store);
        self.ok(&["start"]);
    }
}

#[test]
fn real_cli_guards_timeout_quorum_and_placement_preserve_frozen_authorization() {
    let (cli, mut spec) = Cli::new();
    let initial = spec.clone();
    let mut invalid = spec.clone();
    invalid.task_revisions = vec![p("task_revision", "T:1"), p("task_revision", "T:1")];
    cli.rejected(&start_input(invalid, "two-tasks"), "INVALID_INPUT");
    spec.policy.transition_timeout_ms = 1;
    cli.write(&start_input(spec.clone(), "timeout-start"));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let shown = cli.show("first");
        assert_eq!(shown["run"]["lifecycle"], "starting");
        if !shown["run"]["attention"].is_null() {
            assert_eq!(shown["needs_attention"], true);
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "timeout worker did not mark attention"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let original = cli.show("first")["manifest"].clone();
    cli.write(&Input {
        key: "cli-place".into(),
        action: Action::Place {
            project_id: "P".into(),
            run_id: "first".into(),
            version: cli.version("first"),
            destination: p("room_binding", "delivery"),
        },
    });
    assert_eq!(cli.show("first")["manifest"], original);
    cli.stop();
    let mut store = Store::open(&cli.temp.0).unwrap();
    assert_eq!(store.list("run_placement").unwrap().len(), 1);
    seed(
        &mut store,
        &[value_record(
            p("room_binding", "delivery").key,
            2,
            &json!({"closed":true}),
        )
        .unwrap()],
        "close-display",
    );
    approve(&mut store, "gate", workflow(true));
    gate_picks(&store, &mut spec);
    spec.run_id = "gate-run".into();
    spec.policy.transition_timeout_ms = 900_000;
    spec.placement = life::Placement::default();
    let known = bundle(&mut store, "known");
    let mut same = spec.clone();
    for pick in &mut same.picks {
        pick.frozen_bundle = Some(known.clone());
    }
    drop(store);
    cli.ok(&["start"]);
    assert_eq!(cli.show("first")["run"]["lifecycle"], "starting");
    assert_eq!(cli.show("first")["manifest"], original);
    let preview = cli.preview(&start_input(spec, "unknown-material"));
    assert!(
        !preview["effect_summary"]["result"]["warnings"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    cli.rejected(&start_input(same, "unreachable"), "QUORUM_UNREACHABLE");
    cli.stop();
    let mut store = Store::open(&cli.temp.0).unwrap();
    let mut project = store.get(&p("project", "P").key).unwrap().unwrap();
    if let RecordData::Project { archived, .. } = &mut project.data {
        *archived = true;
    }
    project.version += 1;
    seed(&mut store, &[project], "archive");
    drop(store);
    cli.ok(&["start"]);
    let mut archived = initial;
    archived.project_version = 2;
    archived.run_id = "archived".into();
    cli.rejected(&start_input(archived, "archived"), "PROJECT_READ_ONLY");
    println!(
        "real CLI: cardinality/archived guards, wall-clock attention without cancellation, unknown/known quorum and audited placement passed"
    );
}
impl Drop for Cli {
    fn drop(&mut self) {
        self.stop();
    }
}
#[test]
fn real_cli_previews_starts_pauses_resumes_replaces_and_cancels() {
    let (cli, mut spec) = Cli::new();
    let input = start_input(spec.clone(), "cli-start");
    let path = cli.path(&input);
    let preview = cli.ok(&[
        "run",
        "preview",
        "--input",
        path.to_str().unwrap(),
        "--key",
        &input.key,
    ]);
    assert_eq!(
        preview["effect_summary"]["result"]["integration"]["mode"],
        "automatic"
    );
    assert_eq!(cli.ok(&["run", "list", "P"])["runs"], json!([]));
    let (ok, error) = cli.run(&[
        "run",
        "start",
        "--input",
        path.to_str().unwrap(),
        "--key",
        &input.key,
        "--preview-token",
        "not-the-preview",
    ]);
    assert!(!ok);
    assert_eq!(error["error"]["code"], "PREVIEW_REQUIRED");
    cli.ok(&[
        "run",
        "start",
        "--input",
        path.to_str().unwrap(),
        "--key",
        &input.key,
        "--preview-token",
        preview["preview_token"].as_str().unwrap(),
    ]);
    assert_eq!(cli.show("first")["run"]["lifecycle"], "starting");
    assert_eq!(cli.write(&input)["run_id"], "first");
    assert_eq!(
        cli.ok(&["run", "list", "P"])["runs"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let bad = command("resume", "first", cli.version("first"));
    let path = cli.path(&bad);
    let (ok, error) = cli.run(&[
        "run",
        "resume",
        "--input",
        path.to_str().unwrap(),
        "--key",
        &bad.key,
    ]);
    assert!(!ok);
    assert_eq!(error["error"]["code"], "INVALID_TRANSITION");
    let fake = Fake::default();
    cli.engine("first", "start", &fake);
    assert_eq!(cli.show("first")["run"]["lifecycle"], "running");
    cli.write(&command("pause", "first", cli.version("first")));
    assert_eq!(cli.show("first")["run"]["lifecycle"], "pausing");
    cli.engine("first", "pause", &fake);
    assert_eq!(cli.show("first")["run"]["lifecycle"], "paused");
    cli.write(&command("resume", "first", cli.version("first")));
    cli.engine("first", "resume", &fake);
    assert_eq!(cli.show("first")["run"]["lifecycle"], "running");
    let original = cli.show("first")["manifest"].clone();
    spec.run_id = "second".into();
    spec.integration.mode = life::IntegrationMode::HumanPreview;
    cli.write(&Input {
        key: "cli-replace".into(),
        action: Action::Replace {
            project_id: "P".into(),
            run_id: "first".into(),
            version: cli.version("first"),
            spec: Box::new(spec),
        },
    });
    assert_eq!(cli.show("first")["run"]["lifecycle"], "replaced");
    assert_eq!(cli.show("first")["manifest"], original);
    assert_eq!(
        cli.show("second")["manifest"]["integration"]["mode"],
        "human_preview"
    );
    cli.write(&command("cancel", "second", cli.version("second")));
    assert_eq!(cli.show("second")["run"]["lifecycle"], "cancelled");
    let bad = command("resume", "second", cli.version("second"));
    let path = cli.path(&bad);
    let (ok, error) = cli.run(&[
        "run",
        "resume",
        "--input",
        path.to_str().unwrap(),
        "--key",
        &bad.key,
    ]);
    assert!(!ok);
    assert_eq!(error["error"]["code"], "INVALID_TRANSITION");
    println!(
        "real hctl2 -> authenticated control -> Run: preview/start/pause/resume/replace/cancel passed; phase acknowledgements are an internal fake engine, not Dagu"
    );
}
