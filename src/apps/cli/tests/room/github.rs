//! 第 9 包第 2 条: real CLI -> native Codex -> protected GitHub main -> Task receipt.
//! No seeded governance records, admission seams, copied credentials or direct merges.
use super::*;
use agency_proto::context::{Bundle, Delivery};
use std::{os::unix::fs::MetadataExt, time::Instant};

const CANARY: &str = "yesme/hctl2-canary";

fn test_command(code_dir: &str) -> String {
    format!("/usr/bin/python3 -B -m unittest discover -s {code_dir} -p test_calculator.py -v")
}

fn last_native_task_test<'a>(
    records: &'a [Value],
    cwd: &Path,
    code_dir: &str,
) -> Option<&'a Value> {
    let cwd = format!("file://{}", cwd.display());
    records.iter().rev().find_map(|record| {
        let item = &record["payload"]["item"];
        let command = item["command"].as_array()?;
        let argv: Option<Vec<_>> = command.iter().map(Value::as_str).collect();
        let argv = argv?;
        // Native CommandExecution stores argv, including the shell's script argument.
        // Read whole argument words so an old directory or a directory suffix cannot match.
        let shell = argv.len() == 3
            && matches!(
                Path::new(argv[0]).file_name().and_then(|s| s.to_str()),
                Some("sh" | "bash" | "zsh")
            )
            && matches!(argv[1], "-c" | "-lc");
        let words = if shell {
            argv[2]
                .split_whitespace()
                .map(|word| word.trim_matches(['\'', '"']))
                .collect()
        } else {
            argv
        };
        let requested = words
            .windows(3)
            .any(|w| w == ["-m", "unittest", "discover"])
            && words.windows(2).any(|w| w == ["-s", code_dir])
            && words.windows(2).any(|w| w == ["-p", "test_calculator.py"]);
        (record["type"] == "event_msg"
            && record["payload"]["type"] == "item_completed"
            && item["type"] == "CommandExecution"
            && item["cwd"] == cwd
            && requested)
            .then_some(item)
    })
}

fn native_test_passed(item: &Value) -> bool {
    item["status"] == "completed"
        && item["exit_code"] == 0
        && item["aggregated_output"].as_str().is_some_and(|output| {
            output.lines().any(|line| {
                line.strip_prefix("Ran ")
                    .and_then(|rest| rest.split_whitespace().next())
                    .and_then(|count| count.parse::<u64>().ok())
                    .is_some_and(|count| count > 0)
            }) && output.trim_end().lines().last() == Some("OK")
        })
}

#[test]
fn native_test_evidence_is_bound_to_this_task_and_the_last_attempt() {
    let cwd = Path::new("/tmp/agency/current-change-set");
    let code_dir = "canary_cases/current_task";
    let record = |dir: &str, task: &str, exit: i32| {
        json!({"type":"event_msg", "payload":{"type":"item_completed", "item":{
            "type":"CommandExecution", "command":["/usr/bin/zsh", "-lc", test_command(task)],
            "cwd":format!("file://{dir}"), "status":if exit == 0 { "completed" } else { "failed" },
            "exit_code":exit, "aggregated_output":"Ran 6 tests in 0.01s\n\nOK\n"
        }}})
    };
    let mut records = vec![
        record(cwd.to_str().unwrap(), "canary_cases/old_task", 0),
        record(cwd.to_str().unwrap(), "canary_cases/current_task_old", 0),
        record("/tmp/agency/other-change-set", code_dir, 0),
    ];
    assert!(last_native_task_test(&records, cwd, code_dir).is_none());
    records.push(record(cwd.to_str().unwrap(), code_dir, 0));
    assert!(native_test_passed(
        last_native_task_test(&records, cwd, code_dir).unwrap()
    ));
    // A later failed attempt must supersede the earlier success, even if output contains OK.
    records.push(record(cwd.to_str().unwrap(), code_dir, 1));
    records.last_mut().unwrap()["payload"]["item"]["command"][2] = json!(format!(
        "/usr/bin/python3 -m unittest discover -v -p test_calculator.py -s '{code_dir}'"
    ));
    assert!(!native_test_passed(
        last_native_task_test(&records, cwd, code_dir).unwrap()
    ));
}

#[test]
fn native_test_evidence_requires_a_nonempty_successful_unittest_summary() {
    let mut item = json!({"status":"completed", "exit_code":0,
        "aggregated_output":"Ran 8 tests in 0.01s\n\nOK\n"});
    assert!(native_test_passed(&item));
    for output in [
        "the tests Ran and look OK",
        "Ran 0 tests\n\nOK\n",
        "Ran 6 tests\n\nFAILED\n",
    ] {
        item["aggregated_output"] = json!(output);
        assert!(!native_test_passed(&item), "{output}");
    }
}

fn run(f: &Fixture, args: &[&str]) -> Value {
    let (ok, value) = f.run(args);
    let safe_args: Vec<_> = args
        .iter()
        .enumerate()
        .map(|(i, arg)| {
            if i > 0 && args[i - 1] == "--preview-token" {
                "<preview-token>"
            } else {
                *arg
            }
        })
        .collect();
    assert!(ok, "hctl2 {safe_args:?}: {value}");
    value
}

fn accept(f: &Fixture, namespace: &str, kind: &str, key: &str, input: Value) -> Value {
    let by = Instant::now() + Duration::from_secs(90);
    loop {
        let (ok, value) = f.command_ns(namespace, kind, key, &input);
        if ok {
            println!("CLI {namespace} {kind} --key {key} --input JSON; input={input}; accepted");
            return value;
        }
        // A just-created GitHub issue may not yet be visible on the list endpoint.
        // Replay the same authorized command: control reads back the original effect,
        // never issues another POST. Do not treat unknown as success or switch keys.
        assert!(
            namespace == "task"
                && value["error"]["code"] == "RESULT_UNKNOWN"
                && Instant::now() < by,
            "hctl2 {namespace} {kind}: {value}"
        );
        println!(
            "CLI task {kind} key={key}: RESULT_UNKNOWN; replaying original command for readback"
        );
        std::thread::sleep(Duration::from_secs(2));
    }
}

fn git(path: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(["-C", path.to_str().unwrap()])
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn gh(f: &Fixture, path: &str) -> Value {
    // Same bundled native client and login as control. Read only, no token extraction.
    let output = Command::new(f.payload.join("libexec/hctl2/gh"))
        .env_remove("GH_DEBUG")
        .env("GH_PROMPT_DISABLED", "1")
        .args(["api", "--hostname", "github.com", "--method", "GET", path])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "gh api {path} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn wait(f: &Fixture, args: &[&str], seconds: u64, done: impl Fn(&Value) -> bool) -> Value {
    let by = Instant::now() + Duration::from_secs(seconds);
    loop {
        let value = run(f, args);
        if done(&value) {
            println!("CLI {} -> {value}", args.join(" "));
            return value;
        }
        assert!(
            Instant::now() < by,
            "hctl2 {args:?} did not settle: {value}"
        );
        assert!(
            !matches!(
                value["state"].as_str(),
                Some("cannot_fulfill" | "cancelled" | "lost")
            ),
            "{value}"
        );
        assert_ne!(value["intent"]["state"], "failed", "{value}");
        std::thread::sleep(Duration::from_millis(500));
    }
}

struct NativeAgency {
    child: std::process::Child,
    root: PathBuf,
}
impl NativeAgency {
    fn start(f: &Fixture) -> Self {
        let root = f.root.join("native-agency");
        std::fs::create_dir_all(&root).unwrap();
        let log = root.join("serve.err");
        let mut child = Command::new(std::env::var_os("CARGO_BIN_EXE_agency").unwrap())
            .env("HCTL2_INSTALL_ROOT", &f.payload)
            .env_remove("GH_DEBUG")
            .arg("--root")
            .arg(&root)
            .arg("serve")
            .stdout(Stdio::null())
            .stderr(File::create(&log).unwrap())
            .spawn()
            .unwrap();
        let by = Instant::now() + Duration::from_secs(90);
        loop {
            let status = Command::new(std::env::var_os("CARGO_BIN_EXE_agency").unwrap())
                .arg("--root")
                .arg(&root)
                .arg("status")
                .output()
                .unwrap();
            if status.status.success() {
                break;
            }
            assert!(
                child.try_wait().unwrap().is_none() && Instant::now() < by,
                "native Agency unavailable: {}",
                std::fs::read_to_string(&log).unwrap()
            );
            std::thread::sleep(Duration::from_millis(100));
        }
        Self { child, root }
    }
}
impl Drop for NativeAgency {
    fn drop(&mut self) {
        let _ = Command::new(std::env::var_os("CARGO_BIN_EXE_agency").unwrap())
            .arg("--root")
            .arg(&self.root)
            .arg("stop")
            .output();
        let by = Instant::now() + Duration::from_secs(5);
        while self.child.try_wait().ok().flatten().is_none() && Instant::now() < by {
            std::thread::sleep(Duration::from_millis(25));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
#[ignore = "UNVERIFIED: native Codex/Claude installations, Codex login and gh push/admin login to yesme/hctl2-canary; HCTL2_HARNESS_LIVE=1 HCTL2_GITHUB_LIVE=1"]
fn demo3_github_codex_real_cli_reaches_protected_main_and_task_completion() {
    for flag in ["HCTL2_HARNESS_LIVE", "HCTL2_GITHUB_LIVE"] {
        assert_eq!(
            std::env::var(flag).as_deref(),
            Ok("1"),
            "UNVERIFIED: {flag}=1 required"
        );
    }
    let (f, _) = Fixture::packaged("demo3-github-codex");
    // Assert the target before starting anything that can publish or merge. Do not repair or
    // weaken the target's protection as part of a test.
    let metadata = gh(&f, &format!("repos/{CANARY}"));
    let protection = gh(&f, &format!("repos/{CANARY}/branches/main/protection"));
    assert_eq!(
        protection["enforce_admins"]["enabled"], true,
        "{protection}"
    );
    assert_eq!(
        protection["required_pull_request_reviews"]["required_approving_review_count"],
        0
    );
    assert_eq!(
        protection["required_status_checks"]["contexts"],
        json!(["canary"])
    );
    println!("PLATFORM gh api repos/{CANARY}/branches/main/protection -> {protection}");
    let source_repo = f.root.join("canary-source");
    let output = Command::new("git")
        .args([
            "clone",
            "--branch",
            "main",
            "--single-branch",
            "https://github.com/yesme/hctl2-canary.git",
        ])
        .arg(&source_repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    git(
        &source_repo,
        &["config", "user.name", "Codex Agency Canary"],
    );
    git(
        &source_repo,
        &[
            "config",
            "user.email",
            "281847692+a-chatgpt-codex-bot[bot]@users.noreply.github.com",
        ],
    );
    let baseline = git(&source_repo, &["rev-parse", "HEAD"]);
    run(&f, &["start", "--secret-backend", "user-file"]);
    let registered = accept(
        &f,
        "repo",
        "register",
        "canary-register",
        json!({
            "name":"GitHub Codex canary", "origin":"external", "platform":"github",
            "instance":"github.com", "platform_repo_id":metadata["id"].to_string(),
            "platform_path":CANARY, "default_source":"github_issues",
            "local":{"machine":"control","path":source_repo}
        }),
    );
    let registration = &registered["registration"];
    let repo = registration["repo_id"].as_str().unwrap().to_owned();
    let stable = registration["observed"]["stable_id"].as_str().unwrap();
    // External bindings are already active after the native identity readback;
    // the explicit confirmation command belongs to newly created local repos.
    assert_eq!(registration["lifecycle"], "active");
    let current_repo = run(&f, &["repo", "show", &repo]);
    assert_eq!(current_repo["observed"]["full_name"], CANARY);
    println!("CLI repo show {repo} -> {current_repo}");
    let created = accept(
        &f,
        "project",
        "create",
        "canary-project",
        json!({"repo_id":repo,
        "definition":{"name":"Codex protected main chain", "goal":"deliver tested code through a protected PR",
        "scope":CANARY, "roles":[], "role_members":{}, "defaults":{},
        "settings":{"selection_policy":{}, "publish_review_requires_confirmation":false}}}),
    );
    let project = created["project_id"].as_str().unwrap().to_owned();
    let room = created["main_room_id"].as_str().unwrap().to_owned();
    let agency = NativeAgency::start(&f);
    let pairing = run(
        &f,
        &[
            "agency",
            "pair",
            "--binding-id",
            "native",
            "--agency-root",
            agency.root.to_str().unwrap(),
            "--key",
            "canary-pair",
        ],
    );
    let catalog = run(&f, &["agency", "catalog", "native"]);
    let profession = catalog["professions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["reference"]["id"] == "codex-cli")
        .unwrap_or_else(|| {
            panic!(
                "UNVERIFIED: native Codex not cataloged: {catalog}; {}",
                std::fs::read_to_string(agency.root.join("serve.err")).unwrap()
            )
        });
    let profession_input = f.root.join("codex-profession.json");
    std::fs::write(&profession_input, profession["reference"].to_string()).unwrap();
    let accepted_profession = run(
        &f,
        &[
            "agency",
            "accept",
            "--binding-id",
            "native",
            "--reference",
            profession_input.to_str().unwrap(),
            "--key",
            "canary-profession",
        ],
    );
    let permissions = json!(["context.read", "git.read", "git.write"]);
    let profile = accept(
        &f,
        "profile",
        "create",
        "canary-profile",
        json!({"id":"canary-codex",
        "profile":{"harness":profession["harness"], "model":profession["model"], "mode":"write",
        "permissions":permissions, "environment":[], "required_capabilities":agency_proto::Capabilities::default(),
        "max_context_bytes":65536}}),
    );
    let profession_ref = json!({"key":accepted_profession["profession"]["key"],
        "version":{"state":accepted_profession["profession"]["version"]}});
    let binding_ref =
        json!({"key":pairing["binding"]["key"], "version":{"state":pairing["binding"]["version"]}});
    accept(
        &f,
        "project",
        "select",
        "canary-selection",
        json!({"project_id":project, "project_version":1,
        "room_id":room, "topic_command_key":null, "roster_version":null, "selections":[{
        "room_id":room, "selected_item":profession_ref, "profession":profession_ref,
        "profession_digest":profession["reference"]["digest"], "agency":binding_ref,
        "required_skills":[], "optional_skills":[], "worker_profiles":[profile["revision"]],
        "responsibility":"coding", "permission":{"allow":permissions}, "budget":{"max_bytes":65536},
        "display_name":"Codex Canary", "persona_tags":[]}]}),
    );
    let source = accept(
        &f,
        "task",
        "connect",
        "canary-task-source",
        json!({"repo_id":repo,
        "candidate_id":"github_issues", "consent":true, "make_default":false}),
    );
    let source = source["source_id"].as_str().unwrap();
    accept(
        &f,
        "task",
        "attach",
        "canary-task-attach",
        json!({"project_id":project, "project_version":1,
        "source_id":source, "approved_scope":stable, "consent":true}),
    );
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let code_dir = format!("canary_cases/codex_{nonce}");
    let requested_test = test_command(&code_dir);
    let request = format!(
        "In {code_dir}, add calculator.py implementing ceil_div(numerator, denominator) for signed integers using integer arithmetic, raising ValueError on zero denominator. Add test_calculator.py with unittest covering positive, negative, exact and zero cases. This is real non-documentation code for the adopted Task. Run {requested_test} and report the command and output. Modify only these two files, leave changes uncommitted for Agency sealing, and do not push, obtain credentials, publish, or change global harness configuration."
    );
    let task = accept(
        &f,
        "task",
        "create",
        "canary-task",
        json!({"project_id":project, "project_version":1,
        "source_id":source, "title":format!("Codex ceil_div chain {nonce}"), "body":request}),
    );
    let task = task["task_id"].as_str().unwrap().to_owned();
    let before = run(&f, &["task", "show", &project, &task]);
    let contract = json!({"scope":code_dir, "expected_outcome":"tested integer ceil_div merged through protected main",
        "acceptance":[{"text":"the Task change is integrated", "grade":"mechanical",
        "evidence":{"accept":"integration_receipt", "min_channel":"unmediated"}},
        {"text":"a human verified the code and tests", "grade":"human"}], "roles":[], "capabilities":[]});
    let adopted = accept(
        &f,
        "task",
        "adopt",
        "canary-adopt",
        json!({"project_id":project, "project_version":1,
        "task_id":task, "version":before["version"], "adoption":{"contract":contract,
        "origin":{"kind":"local", "reference":{"key":{"scope":{"kind":"project","id":project},
        "kind":"project","id":project}, "version":{"state":1}},
        "proposal_digest":foundation::canonical_json_sha256(&contract).unwrap()}}}),
    );
    let input = f.root.join("codex-invocation.json");
    std::fs::write(&input, json!({"project_id":project, "room_id":room, "task_id":task, "target":"coding",
        "profile":profile["revision"], "request":request, "budget":65536,
        "deadline_ms":nonce as u64 + 600_000, "retry_of":null,
        "write":{"change_set_id":null,"baseline_commit":baseline,"target_branch":"main","allow_update":true}}).to_string()).unwrap();
    let args = [
        "invocation",
        "preview",
        "--input",
        input.to_str().unwrap(),
        "--key",
        "canary-write",
    ];
    let preview = run(&f, &args);
    let write = &preview["effect_summary"]["preview"]["write"];
    assert_eq!(write["authorization"], "publish_for_review_not_integration");
    assert_eq!(
        write["publication_target"]["requires_human_confirmation"],
        false
    );
    assert_eq!(write["publication_target"]["target_branch"], "main");
    let pending = &write["lease"]["pending"];
    let change_set = pending["change_set_id"].as_str().unwrap().to_owned();
    println!("CLI invocation preview -> {}", preview["effect_summary"]);
    let mut start = args.to_vec();
    start[1] = "start";
    start.extend([
        "--preview-token",
        preview["preview_token"].as_str().unwrap(),
    ]);
    let started = run(&f, &start);
    let invocation = started["invocation_id"].as_str().unwrap().to_owned();
    let finished = wait(
        &f,
        &["invocation", "show", &project, &invocation],
        600,
        |v| v["state"] == "completed",
    );
    assert_eq!(finished["results"].as_array().unwrap().len(), 1);
    let admitted = run(&f, &["changeset", "show", &repo, &change_set]);
    assert_eq!(admitted["revisions"].as_array().unwrap().len(), 1);
    let revision = &admitted["revisions"][0];
    assert_eq!(revision["producer_ref"]["invocation_id"], invocation);
    assert_eq!(revision["base_commit_sha"], baseline);
    let proposal: Value =
        serde_json::from_str(finished["results"][0]["output"].as_str().unwrap()).unwrap();
    assert_eq!(proposal["change_set_id"], change_set);
    assert_eq!(proposal["base_commit_sha"], baseline);
    assert_eq!(proposal["location"]["kind"], "commit");
    let proposal_commit = proposal["location"]["commit_sha"].as_str().unwrap();
    assert_eq!(
        git(
            &source_repo,
            &["rev-parse", &format!("{proposal_commit}^{{tree}}")]
        ),
        revision["result_tree_sha"]
    );
    let revision_id = revision["change_set_revision_id"].as_str().unwrap();
    println!("CLI changeset show -> {admitted}");
    let trace = run(&f, &["terminal", "inspect", &project, &invocation]);
    let events = trace["trace"]["events"].as_array().unwrap();
    let thread = events
        .iter()
        .find_map(|e| e["payload"]["thread"].as_str())
        .unwrap();
    let home = std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap()).join(".codex"));
    let rollout = find(&home.join("sessions"), |p| {
        p.is_file()
            && p.file_name().unwrap().to_str().unwrap().contains(thread)
            && p.extension().is_some_and(|e| e == "jsonl")
    })
    .expect("Codex native rollout");
    let records: Vec<Value> = std::fs::read_to_string(&rollout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let session_cwd = records
        .iter()
        .find(|r| r["type"] == "session_meta")
        .unwrap()["payload"]["cwd"]
        .as_str()
        .map(PathBuf::from)
        .unwrap();
    let cwd = session_cwd.canonicalize().unwrap();
    assert_eq!(cwd.file_name().unwrap().to_str().unwrap(), change_set);
    assert!(!cwd.starts_with(agency.root.canonicalize().unwrap()));
    assert_eq!(
        git(
            &cwd,
            &["rev-parse", "--path-format=absolute", "--git-common-dir"]
        ),
        source_repo
            .join(".git")
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
    );
    assert_eq!(git(&source_repo, &["rev-parse", "HEAD"]), baseline);
    let bundle: Bundle =
        serde_json::from_value(preview["effect_summary"]["assembly"]["bundle"]["document"].clone())
            .unwrap();
    let mut expected_body = String::new();
    for entry in &bundle.entries {
        if let Delivery::Inline { bytes } | Delivery::Pointer { bytes, .. } = &entry.delivery {
            expected_body.push_str(std::str::from_utf8(bytes).unwrap());
            expected_body.push('\n');
        }
    }
    expected_body.push_str(&format!("\nAgency ChangeSet: {change_set}\nBaseline: {baseline}\nWrite lease: {} generation {}\nAgency execution directory: {}\nEdit and test this ChangeSet in this materialized worktree. Leave changes uncommitted for Agency sealing. Do not push, publish reviews, obtain Git credentials, or change harness global configuration.\n",
        pending["lease"]["lease_id"].as_str().unwrap(), pending["lease"]["generation"], cwd.display()));
    assert!(
        records.iter().any(|r| r["payload"]["role"] == "user"
            && r["payload"]["content"]
                .as_array()
                .is_some_and(|blocks| blocks
                    .iter()
                    .any(|b| b["type"] == "input_text" && b["text"] == expected_body))),
        "Codex rollout does not contain the exact frozen Bundle + Agency boundary"
    );
    let native_test = last_native_task_test(&records, &session_cwd, &code_dir)
        .expect("native Codex test execution for this Task in this ChangeSet worktree");
    assert!(
        native_test_passed(native_test),
        "last native test: {native_test}"
    );
    println!(
        "CODEX rollout={} thread={thread}; exact input_text verified\nUSER BODY:\n{expected_body}\nNATIVE TEST: {native_test}",
        rollout.display()
    );
    let test = Command::new("/usr/bin/python3")
        .args([
            "-B",
            "-m",
            "unittest",
            "discover",
            "-s",
            &code_dir,
            "-p",
            "test_calculator.py",
            "-v",
        ])
        .current_dir(&cwd)
        .output()
        .unwrap();
    assert!(
        test.status.success(),
        "{}",
        String::from_utf8_lossy(&test.stderr)
    );
    // Independent examples, not just the test suite authored by the harness.
    let examples = format!(
        "import sys; sys.path.insert(0, {code_dir:?}); from calculator import ceil_div; cases=[(7,3,3),(-7,3,-2),(7,-3,-2),(-7,-3,3),(6,3,2),(0,3,0)]; assert all(ceil_div(a,b)==c for a,b,c in cases);\ntry: ceil_div(1,0)\nexcept ValueError: pass\nelse: raise AssertionError('zero denominator must raise ValueError')\nprint('7 independent ceil_div cases OK')"
    );
    let examples_output = Command::new("/usr/bin/python3")
        .args(["-B", "-c", &examples])
        .current_dir(&cwd)
        .output()
        .unwrap();
    assert!(
        examples_output.status.success(),
        "{}",
        String::from_utf8_lossy(&examples_output.stderr)
    );
    println!(
        "INDEPENDENT python3 unittest:\n{}\n{}",
        String::from_utf8_lossy(&test.stderr),
        String::from_utf8_lossy(&examples_output.stdout)
    );
    let seal = Command::new(std::env::var_os("HCTL2_TOOL_BIN").unwrap())
        .args([
            "repo",
            "seal",
            "--path",
            cwd.to_str().unwrap(),
            "--change-set-ref",
            &change_set,
            "--baseline",
            &baseline,
            "--key",
            "demo3-independent-seal",
        ])
        .output()
        .unwrap();
    assert!(
        seal.status.success(),
        "{}",
        String::from_utf8_lossy(&seal.stderr)
    );
    let seal: Value = serde_json::from_slice(&seal.stdout).unwrap();
    assert_eq!(seal["result_tree_sha"], revision["result_tree_sha"]);
    assert_eq!(seal["base_commit_sha"], baseline);
    println!("TOOL independent repo seal -> {seal}");
    let diff = run(&f, &["changeset", "diff", &repo, &change_set, revision_id]);
    assert!(diff.to_string().contains("calculator.py"), "{diff}");
    println!("CLI changeset diff -> {diff}");
    let listed = run(&f, &["review", "list", &repo]);
    assert_eq!(listed["items"].as_array().unwrap().len(), 1);
    let publish_id = listed["items"][0]["intent_id"].as_str().unwrap();
    let published = wait(&f, &["review", "show", &repo, publish_id], 180, |v| {
        v["intent"]["state"] == "published"
    });
    assert_eq!(published["mappings"].as_array().unwrap().len(), 1);
    assert_eq!(published["stages"]["push"]["confirmed"], true);
    assert_eq!(published["stages"]["review_request"]["confirmed"], true);
    let mapping = &published["mappings"][0];
    let head = mapping["platform_commit_sha"].as_str().unwrap();
    assert_eq!(
        git(&source_repo, &["rev-parse", &format!("{head}^{{tree}}")]),
        revision["result_tree_sha"]
    );
    let number = mapping["review_request"]["index"].as_u64().unwrap();
    let pr_path = format!("repos/{CANARY}/pulls/{number}");
    let pr = gh(&f, &pr_path);
    assert_eq!(pr["head"]["sha"], head);
    assert_eq!(pr["base"]["ref"], "main");
    assert_eq!(pr["merged"], false);
    println!("PLATFORM published PR {} head={head}", pr["html_url"]);
    let by = Instant::now() + Duration::from_secs(300);
    let checks = loop {
        let checks = gh(&f, &format!("repos/{CANARY}/commits/{head}/check-runs"));
        if checks["check_runs"].as_array().unwrap().iter().any(|c| {
            c["name"] == "canary" && c["status"] == "completed" && c["conclusion"] == "success"
        }) {
            break checks;
        }
        assert!(Instant::now() < by, "canary did not pass: {checks}");
        std::thread::sleep(Duration::from_secs(5));
    };
    println!("PLATFORM gh api commits/{head}/check-runs -> {checks}");
    // Only the platform-supported authorization form may enter the real merge path.
    let integration_input = f.root.join("canary-integration.json");
    let mut integration = json!({"repo_id":repo,"change_set_revision_id":revision_id,"target_kind":"platform",
        "target_ref":"refs/heads/main","form":"expected_head","strategy":"merge_commit"});
    std::fs::write(&integration_input, integration.to_string()).unwrap();
    let args = [
        "integration",
        "preview",
        "--input",
        integration_input.to_str().unwrap(),
        "--key",
        "canary-merge",
    ];
    let (ok, rejected) = f.run(&args);
    assert!(
        !ok,
        "unsupported exact-head authorization was accepted: {rejected}"
    );
    assert_eq!(rejected["error"]["code"], "EXPECTED_HEAD_UNSUPPORTED");
    assert_eq!(
        rejected["error"]["recovery_action"],
        "choose_accept_advance_form"
    );
    println!("CLI integration preview form=expected_head -> {rejected}");
    integration["form"] = json!("accept_advance");
    std::fs::write(&integration_input, integration.to_string()).unwrap();
    let merge_preview = run(&f, &args);
    let protection_snapshot = &merge_preview["effect_summary"]["protection"];
    assert_eq!(
        protection_snapshot["requires_review_request"], true,
        "{merge_preview}"
    );
    assert_eq!(protection_snapshot["required_checks"], json!(["canary"]));
    assert_eq!(
        protection_snapshot["other"]["enforce_admins"]["enabled"],
        true
    );
    println!(
        "CLI integration preview form=accept_advance -> {}",
        merge_preview["effect_summary"]
    );
    let mut submit = args.to_vec();
    submit[1] = "submit";
    submit.extend([
        "--preview-token",
        merge_preview["preview_token"].as_str().unwrap(),
    ]);
    let submitted = run(&f, &submit);
    let intent = submitted["intent_id"].as_str().unwrap();
    let merged = wait(&f, &["integration", "show", &repo, intent], 180, |v| {
        v["intent"]["state"] == "succeeded"
    });
    let receipt = &merged["receipt"];
    assert_eq!(receipt["source"]["change_set_revision_id"], revision_id);
    assert_eq!(receipt["evidence_level"], "hctl2-tool");
    assert_eq!(receipt["readback"]["git"]["contains"], true);
    assert!(
        receipt["readback"]["git"]["commit_parents"]
            .as_array()
            .unwrap()
            .contains(&json!(head))
    );
    let pr = gh(&f, &pr_path);
    let main = gh(&f, &format!("repos/{CANARY}/branches/main"));
    assert_eq!(pr["merged"], true);
    assert_eq!(pr["merge_commit_sha"], receipt["integrated_commit"]);
    assert_eq!(main["commit"]["sha"], receipt["target_head_after"]);
    assert_ne!(
        receipt["integrated_commit"], head,
        "merge commit is not the candidate"
    );
    println!(
        "PLATFORM PR {} merged as {}; main={}",
        pr["html_url"], pr["merge_commit_sha"], main["commit"]["sha"]
    );
    let principal = format!("local-owner:{}", std::fs::metadata(&f.root).unwrap().uid());
    let adopted_task = &adopted["task"];
    let complete = json!({"project_id":project,"task_id":task,"version":adopted_task["version"],
        "lifecycle_version":adopted_task["data"]["lifecycle_version"],
        "revision_number":adopted_task["data"]["revision"]["number"],"acceptance":[
        {"item":0,"judge":{"kind":"hctl2_tool"},"channel":"unmediated","references":[{
        "key":{"scope":{"kind":"repo","id":repo},"kind":"integration_receipt","id":receipt["receipt_id"]},
        "version":{"state":1}}],"producer":"hctl2-tool","generation":1},
        {"item":1,"judge":{"kind":"human","actor":principal},"channel":"narrated","references":[],
        "producer":null,"generation":null}]});
    let completed = accept(&f, "task", "complete", "canary-complete", complete);
    let task_after = run(&f, &["task", "show", &project, &task]);
    println!("CLI task complete -> {completed}\nCLI task show -> {task_after}");
    assert_eq!(task_after["data"]["lifecycle"], "completed");
    assert_eq!(completed["lifecycle"], "completed");
    assert_eq!(completed["items"][0]["validation_level"], "unmediated");
    assert_eq!(
        completed["items"][0]["references"][0]["key"]["id"],
        receipt["receipt_id"]
    );
    assert!(completed["receipt_id"].as_str().is_some());
    println!(
        "LIVE CLI demo3 GitHub: Codex {thread} -> invocation {invocation} -> revision {revision_id} -> PR #{number} at {head} -> merge {} -> Integration Receipt {} -> Task {task} completed; human delivery previews: integration, completion",
        receipt["integrated_commit"], receipt["receipt_id"]
    );
}
