//! B1 uses the same native-package fixture, with no seeded Project or Room records.
use super::*;
use std::os::unix::fs::MetadataExt;

fn accepted(f: &Fixture, namespace: &str, kind: &str, key: &str, input: Value) -> Value {
    let (ok, result) = f.command_ns(namespace, kind, key, &input);
    assert!(ok, "{namespace}.{kind}: {result}");
    result
}
fn project_definition(name: &str) -> Value {
    json!({"name":name,"goal":"deliver B1","scope":"registered repo","roles":[],"role_members":{},"defaults":{},"settings":{"selection_policy":{},"publish_review_requires_confirmation":true}})
}

fn ready_timeline(f: &Fixture, project: &str, room: &str) -> Value {
    for _ in 0..100 {
        let (ok, result) = f.run(&["room", "timeline", project, room]);
        if ok {
            return result;
        }
        assert_eq!(result["error"]["code"], "CHAT_UNAVAILABLE", "{result}");
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("chat server did not become ready");
}

struct AgencyChild(std::process::Child);
impl Drop for AgencyChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
/// A control plane, a paired real Agency and one selected roster, all reached
/// through the CLI. `delayed_seconds` is how long the script execution body waits
/// once `delay` exists; touching that file is what keeps a dispatch observable.
struct Paired {
    agency: AgencyChild,
    repo: String,
    project: String,
    room: String,
    profession: Value,
    /// The exact `FrozenRef` file the catalog produced and `accept` consumed.
    profession_reference: PathBuf,
    profile: Value,
    delay: PathBuf,
}
fn paired(name: &str, delayed_seconds: u64) -> (Fixture, Paired) {
    let (f, _) = Fixture::packaged(name);
    assert!(f.run(&["start", "--secret-backend", "user-file"]).0);
    let registered = accepted(
        &f,
        "repo",
        "register",
        "register-dispatch",
        json!({"name":"dispatch","origin":"local","platform":"local","platform_path":"dispatch","default_source":"gitea_issues"}),
    );
    let registration = &registered["registration"];
    let repo = registration["repo_id"].as_str().unwrap();
    let version = registration["version"].to_string();
    let args = [
        "repo",
        "register",
        "--confirm",
        repo,
        "--version",
        &version,
        "--platform-repo-id",
        registration["observed"]["stable_id"].as_str().unwrap(),
        "--key",
        "confirm-dispatch",
    ];
    let (ok, plan) = f.run(&args);
    assert!(ok, "{plan}");
    let mut confirm = args.to_vec();
    confirm.extend(["--preview-token", plan["preview_token"].as_str().unwrap()]);
    assert!(f.run(&confirm).0);
    let created = accepted(
        &f,
        "project",
        "create",
        "project-dispatch",
        json!({"repo_id":repo,"definition":project_definition("Dispatch")}),
    );
    let project = created["project_id"].as_str().unwrap().to_owned();
    let room = created["main_room_id"].as_str().unwrap().to_owned();
    let agency_root = f.root.join("independent-agency");
    let config = f.root.join("script.json");
    let delay = f.root.join("delay-script");
    let script = format!(
        "read -r initial; if test -f \"$1\"; then sleep {delayed_seconds}; fi; printf '%s\\n' '{{\"type\":\"result\",\"schema\":\"adapter.stdout.v1\",\"output\":\"DISPATCH_CHAIN_OK\"}}'"
    );
    std::fs::write(
        &config,
        json!({"program":"/bin/sh","arguments":["-c",script,"dispatch-fixture",delay]}).to_string(),
    )
    .unwrap();
    let binary = std::env::var_os("CARGO_BIN_EXE_agency").unwrap();
    let agency = AgencyChild(
        Command::new(&binary)
            .arg("--root")
            .arg(&agency_root)
            .arg("serve")
            .arg("--script-config")
            .arg(&config)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    for _ in 0..100 {
        if Command::new(&binary)
            .arg("--root")
            .arg(&agency_root)
            .arg("status")
            .output()
            .unwrap()
            .status
            .success()
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let (ok, binding) = f.run(&[
        "agency",
        "pair",
        "--binding-id",
        "local",
        "--agency-root",
        agency_root.to_str().unwrap(),
        "--key",
        "pair",
    ]);
    assert!(ok, "{binding}");
    let (ok, catalog) = f.run(&["agency", "catalog", "local"]);
    assert!(ok, "{catalog}");
    let profession = catalog["professions"][0].clone();
    let profession_reference = f.root.join("profession.json");
    std::fs::write(&profession_reference, profession["reference"].to_string()).unwrap();
    let (ok, accepted_profession) = f.run(&[
        "agency",
        "accept",
        "--binding-id",
        "local",
        "--reference",
        profession_reference.to_str().unwrap(),
        "--key",
        "accept",
    ]);
    assert!(ok, "{accepted_profession}");
    let profile = accepted(
        &f,
        "profile",
        "create",
        "profile",
        json!({"id":"research","profile":{"harness":profession["harness"],"model":profession["model"],"mode":"read_only","permissions":["context.read"],"environment":[],"required_capabilities":agency_proto::Capabilities::default(),"max_context_bytes":65536}}),
    );
    let binding_ref =
        json!({"key":binding["binding"]["key"],"version":{"state":binding["binding"]["version"]}});
    let profession_ref = json!({"key":accepted_profession["profession"]["key"],"version":{"state":accepted_profession["profession"]["version"]}});
    let selection = json!({"room_id":room,"selected_item":profession_ref,"profession":profession_ref,"profession_digest":profession["reference"]["digest"],"agency":binding_ref,"required_skills":[],"optional_skills":[],"worker_profiles":[profile["revision"]],"responsibility":"research","permission":{"allow":["context.read"]},"budget":{"max_bytes":65536},"display_name":"Research","persona_tags":[]});
    accepted(
        &f,
        "project",
        "select",
        "select",
        json!({"project_id":project,"project_version":1,"room_id":room,"topic_command_key":null,"roster_version":null,"selections":[selection]}),
    );
    (
        f,
        Paired {
            agency,
            repo: repo.to_owned(),
            project,
            room,
            profession,
            profession_reference,
            profile,
            delay,
        },
    )
}

#[test]
fn dispatch_from_real_cli_pairing_to_room_answer_and_restart_keeps_one_invocation() {
    let (f, mut setup) = paired("dispatch-chain", 2);
    let p = setup.project.as_str();
    let room = setup.room.as_str();
    let delay = &setup.delay;
    let agency = &mut setup.agency;
    let profile = &setup.profile;
    let show = f.show(p, room);
    accepted(
        &f,
        "room",
        "send",
        "context",
        json!({"project_id":p,"room_id":room,"version":show["binding"]["version"],"body":"READ_ONLY_CONTEXT_MARKER"}),
    );
    let path = f.root.join("invocation.json");
    let deadline = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
        + 60000;
    std::fs::write(&path, json!({"project_id":p,"room_id":room,"target":"research","profile":profile["revision"],"request":"Return the exact answer without modifying files","budget":65536,"deadline_ms":deadline,"retry_of":null}).to_string()).unwrap();
    let (ok, plan) = f.run(&[
        "invocation",
        "preview",
        "--input",
        path.to_str().unwrap(),
        "--key",
        "invoke",
    ]);
    assert!(ok, "{plan}");
    let frozen = &plan["effect_summary"]["assembly"]["bundle"]["document"];
    assert_eq!(frozen["consumer"]["project"], p);
    assert_eq!(
        frozen["entries"].as_array().unwrap().len(),
        2,
        "request and this Room's exact online window"
    );
    // Starting without a real preview must not create an invocation.
    let (ok, refusal) = f.run(&[
        "invocation",
        "start",
        "--input",
        path.to_str().unwrap(),
        "--key",
        "invoke",
        "--preview-token",
        "invented",
    ]);
    assert!(!ok);
    assert_eq!(refusal["error"]["code"], "PREVIEW_REQUIRED");
    let (ok, started) = f.run(&[
        "invocation",
        "start",
        "--input",
        path.to_str().unwrap(),
        "--key",
        "invoke",
        "--preview-token",
        plan["preview_token"].as_str().unwrap(),
    ]);
    assert!(ok, "{started}");
    let id = started["invocation_id"].as_str().unwrap();
    let mut completed = Value::Null;
    for _ in 0..150 {
        let (ok, value) = f.run(&["invocation", "show", p, id]);
        assert!(ok, "{value}");
        if value["state"] == "completed" {
            completed = value;
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert_eq!(
        completed["state"], "completed",
        "answer admission: {completed}"
    );
    assert_eq!(completed["results"][0]["output"], "DISPATCH_CHAIN_OK");
    for _ in 0..100 {
        let (ok, timeline) = f.run(&["room", "timeline", p, room]);
        assert!(ok, "{timeline}");
        let messages = timeline["events"].as_array().unwrap();
        let count = messages
            .iter()
            .filter(|e| e["content"]["body"] == "DISPATCH_CHAIN_OK")
            .count();
        if count > 0 {
            assert_eq!(count, 1);
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(f.run(&["stop"]).0);
    std::thread::sleep(Duration::from_millis(100));
    assert!(f.run(&["start"]).0);
    let (ok, again) = f.run(&["invocation", "show", p, id]);
    assert!(ok, "{again}");
    assert_eq!(again["results"], completed["results"]);
    let (ok, replay_plan) = f.run(&[
        "invocation",
        "preview",
        "--input",
        path.to_str().unwrap(),
        "--key",
        "invoke",
    ]);
    assert!(ok, "{replay_plan}");
    assert_eq!(
        replay_plan["effect_summary"], plan["effect_summary"],
        "replay uses the frozen Context, including the original window"
    );
    let (ok, replay) = f.run(&[
        "invocation",
        "start",
        "--input",
        path.to_str().unwrap(),
        "--key",
        "invoke",
        "--preview-token",
        replay_plan["preview_token"].as_str().unwrap(),
    ]);
    assert!(ok, "{replay}");
    assert_eq!(replay, started);
    let timeline = ready_timeline(&f, p, room);
    assert_eq!(
        timeline["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["content"]["body"] == "DISPATCH_CHAIN_OK")
            .count(),
        1
    );

    // The answer can be admitted while Matrix is unavailable. Recovery of
    // that outbox must not require the Agency that already returned it.
    std::fs::write(delay, b"delay").unwrap();
    let mut next_input: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    next_input["deadline_ms"] = json!(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
            + 60_000
    );
    std::fs::write(&path, next_input.to_string()).unwrap();
    let (ok, pending_plan) = f.run(&[
        "invocation",
        "preview",
        "--input",
        path.to_str().unwrap(),
        "--key",
        "pending-projection",
    ]);
    assert!(ok, "{pending_plan}");
    let stop_chat = || {
        assert!(
            Command::new(f.payload.join("bin/hctl2-services"))
                .env("HCTL2_STATE_ROOT", f.root.join("services"))
                .args(["stop", "tuwunel"])
                .output()
                .unwrap()
                .status
                .success()
        );
    };
    stop_chat();
    let (ok, unavailable) = f.run(&[
        "invocation",
        "start",
        "--input",
        path.to_str().unwrap(),
        "--key",
        "pending-projection",
        "--preview-token",
        pending_plan["preview_token"].as_str().unwrap(),
    ]);
    assert!(!ok, "a live Room is required for a new authorization");
    assert_eq!(unavailable["error"]["code"], "CHAT_UNAVAILABLE");
    assert!(
        Command::new(f.payload.join("bin/hctl2-services"))
            .env("HCTL2_STATE_ROOT", f.root.join("services"))
            .args(["start", "tuwunel"])
            .output()
            .unwrap()
            .status
            .success()
    );
    ready_timeline(&f, p, room);
    let (ok, pending_call) = f.run(&[
        "invocation",
        "start",
        "--input",
        path.to_str().unwrap(),
        "--key",
        "pending-projection",
        "--preview-token",
        pending_plan["preview_token"].as_str().unwrap(),
    ]);
    assert!(ok, "{pending_call}");
    stop_chat();
    let pending_id = pending_call["invocation_id"].as_str().unwrap();
    let mut saved = Value::Null;
    for _ in 0..150 {
        let (ok, value) = f.run(&["invocation", "show", p, pending_id]);
        assert!(ok, "{value}");
        if value["state"] == "completed" {
            saved = value;
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert_eq!(saved["state"], "completed", "{saved}");
    assert!(
        saved["pending_effects"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["operation"] == "invocation.project")
    );
    assert!(f.run(&["stop"]).0);
    agency.0.kill().unwrap();
    agency.0.wait().unwrap();
    assert!(f.run(&["start"]).0);
    let mut delivered = false;
    for _ in 0..100 {
        let timeline = ready_timeline(&f, p, room);
        let count = timeline["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["content"]["body"] == "DISPATCH_CHAIN_OK")
            .count();
        if count == 2 {
            delivered = true;
            break;
        }
        assert!(count < 2);
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(delivered, "pending projection recovered without the Agency");
}

/// The second-half surface walks the real CLI to its handler. Each refusal below
/// is one this surface adds, so removing the check turns the case red.
#[test]
fn dispatch_rest_surface_lists_cancels_retries_and_reads_terminal_from_real_cli() {
    let (f, mut setup) = paired("dispatch-rest", 120);
    let p = setup.project.as_str();
    let room = setup.room.as_str();
    let state_version = |value: &Value| value["state_version"].as_i64().unwrap();
    let deadline = || {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
            + 60_000
    };
    // The script body only waits once this file exists, so the dispatch stays
    // observable instead of returning its answer at once.
    std::fs::write(&setup.delay, b"delay").unwrap();

    // `profession accept` is the same command as `agency accept`, so the pairing's
    // key replays the record instead of accepting a second time.
    let (ok, listed) = f.run(&["profession", "list"]);
    assert!(ok, "{listed}");
    let professions = listed["records"].as_array().unwrap();
    assert_eq!(professions.len(), 1, "{listed}");
    let (ok, alias) = f.run(&[
        "profession",
        "accept",
        "--binding-id",
        "local",
        "--reference",
        setup.profession_reference.to_str().unwrap(),
        "--key",
        "accept",
    ]);
    assert!(ok, "{alias}");
    assert_eq!(alias["profession"]["key"], professions[0]["key"]);

    // Keeping a candidate means resending the whole roster at its exact version.
    let (ok, roster) = f.run(&["room", "roster", "show", p, room]);
    assert!(ok, "{roster}");
    assert_eq!(
        roster["selections"].as_array().unwrap().len(),
        1,
        "{roster}"
    );
    // A roster read returns the stored records; a select resends the selections.
    let selections: Value = json!(
        roster["selections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|record| record["data"]["value"].clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(selections[0]["responsibility"], "research", "{selections}");
    let select = |roster_version: Value, key: &str| {
        let path = f.root.join("roster-select.json");
        std::fs::write(
            &path,
            json!({"project_id":p,"project_version":1,"room_id":room,"topic_command_key":null,"roster_version":roster_version,"selections":selections})
                .to_string(),
        )
        .unwrap();
        let input = path.to_str().unwrap();
        let (ok, preview) = f.run(&["room", "roster", "select", "--key", key, "--input", input]);
        if !ok {
            return (ok, preview);
        }
        f.run(&[
            "room",
            "roster",
            "select",
            "--key",
            key,
            "--input",
            input,
            "--preview-token",
            preview["preview_token"].as_str().unwrap(),
        ])
    };
    let (ok, absent) = select(json!(null), "select-absent");
    assert!(!ok, "an existing roster is not an absent one");
    assert_eq!(absent["error"]["code"], "VERSION_CONFLICT");
    let (ok, reselected) = select(json!(1), "select-again");
    assert!(ok, "{reselected}");
    assert_eq!(reselected["roster_version"], 2);

    // Reading a Profile shows the exact revision its pointer names, and grants nothing.
    let (ok, shown) = f.run(&["profile", "show", "research"]);
    assert!(ok, "{shown}");
    assert_eq!(shown["pointer"]["version"], json!({"state":1}));
    assert_eq!(shown["profile"]["model"], setup.profession["model"]);
    let (ok, missing) = f.run(&["profile", "show", "no-such-profile"]);
    assert!(!ok, "{missing}");
    assert_eq!(missing["error"]["code"], "PROFILE_NOT_FOUND");

    // One Invocation that stays observable, so cancellation has something to revoke.
    let path = f.root.join("invocation.json");
    std::fs::write(
        &path,
        json!({"project_id":p,"room_id":room,"target":"research","profile":setup.profile["revision"],"request":"Return the exact answer without modifying files","budget":65536,"deadline_ms":deadline(),"retry_of":null})
            .to_string(),
    )
    .unwrap();
    let input = path.to_str().unwrap();
    let (ok, plan) = f.run(&["invocation", "preview", "--input", input, "--key", "rest"]);
    assert!(ok, "{plan}");
    let bundle = plan["effect_summary"]["assembly"]["bundle"]["document"]["id"].clone();
    let (ok, started) = f.run(&[
        "invocation",
        "start",
        "--input",
        input,
        "--key",
        "rest",
        "--preview-token",
        plan["preview_token"].as_str().unwrap(),
    ]);
    assert!(ok, "{started}");
    let id = started["invocation_id"].as_str().unwrap().to_owned();
    let mut running = Value::Null;
    for _ in 0..150 {
        let (ok, value) = f.run(&["invocation", "show", p, &id]);
        assert!(ok, "{value}");
        if value["state"] == "running" {
            running = value;
            break;
        }
        assert_ne!(
            value["state"], "completed",
            "the delayed script answered before it could be observed: {value}"
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    assert_eq!(running["state"], "running", "{running}");

    // Listing projects this Project's own records; a second Project in the same
    // control plane proves it does not reach another one's.
    let (ok, list) = f.run(&["invocation", "list", p]);
    assert!(ok, "{list}");
    assert_eq!(list["project_id"], p);
    let invocations = list["invocations"].as_array().unwrap();
    assert_eq!(invocations.len(), 1, "{list}");
    assert_eq!(invocations[0]["invocation_id"], id);
    assert_eq!(invocations[0]["owner"], running["owner"]);
    assert_eq!(invocations[0]["state_version"], running["state_version"]);
    let other = accepted(
        &f,
        "project",
        "create",
        "project-other",
        json!({"repo_id":setup.repo,"definition":project_definition("Other")}),
    );
    let elsewhere = other["project_id"].as_str().unwrap();
    let (ok, other_list) = f.run(&["invocation", "list", elsewhere]);
    assert!(ok, "{other_list}");
    assert!(
        other_list["invocations"].as_array().unwrap().is_empty(),
        "{other_list}"
    );
    for args in [
        vec!["invocation", "show", elsewhere, &id],
        vec!["terminal", "inspect", elsewhere, &id],
        vec!["terminal", "replay", elsewhere, &id],
    ] {
        let (ok, crossed) = f.run(&args);
        assert!(!ok, "{args:?}: {crossed}");
        assert_eq!(crossed["error"]["code"], "NOT_FOUND", "{args:?}");
    }

    // Inspection goes through the Agency port on an observe-only ticket.
    for after in ["0", "999999"] {
        let (ok, inspected) = f.run(&["terminal", "inspect", p, &id, "--after", after]);
        assert!(ok, "{inspected}");
        assert_eq!(inspected["trace"]["dispatch"]["owner"]["id"], id);
    }
    // A cursor past the last recorded event is a gap, not an empty completion.
    let (ok, ahead) = f.run(&["terminal", "inspect", p, &id, "--after", "999999"]);
    assert!(ok, "{ahead}");
    assert!(ahead["trace"]["gap"].as_bool().unwrap(), "{ahead}");
    assert!(!ahead["trace"]["complete"].as_bool().unwrap(), "{ahead}");

    // What the Agency reported reaches the governance records through the reconcile
    // loop alone, so wait for its own observation before asking what a human look
    // adds to it.
    let stored = || -> Vec<Value> {
        let (ok, replayed) = f.run(&["terminal", "replay", p, &id]);
        assert!(ok, "{replayed}");
        replayed["observations"].as_array().unwrap().clone()
    };
    let mut observed = Vec::new();
    for _ in 0..150 {
        observed = stored();
        if !observed.is_empty() {
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(!observed.is_empty(), "the reconcile loop observed nothing");
    for _ in 0..3 {
        let (ok, inspected) = f.run(&["terminal", "inspect", p, &id]);
        assert!(ok, "{inspected}");
    }
    assert_eq!(
        stored(),
        observed,
        "a human inspect is a read: it records no observation"
    );

    // Stored replay reads the control plane only: the Agency is already gone.
    setup.agency.0.kill().unwrap();
    setup.agency.0.wait().unwrap();
    let observations = stored();
    assert_eq!(observations, observed, "replay needs no Agency");
    assert_eq!(observations[0]["dispatch"]["owner"]["id"], id);
    // Record keys order by digest, so replay restores the Agency's cursor order.
    let cursors: Vec<u64> = observations
        .iter()
        .map(|o| o["cursor"].as_u64().unwrap())
        .collect();
    let mut ordered = cursors.clone();
    ordered.sort_unstable();
    assert_eq!(cursors, ordered, "replay is in observation order");
    // An unreachable Agency stays the answer, however often a human asks: the read
    // writes no contact record, so there is no command identity to collide with.
    for _ in 0..5 {
        let (ok, unreachable) = f.run(&["terminal", "inspect", p, &id]);
        assert!(!ok, "{unreachable}");
        assert_eq!(unreachable["error"]["code"], "AGENCY_UNREACHABLE");
    }
    assert_eq!(stored(), observed, "a human inspect records nothing");
    // Attach has no public entry: the internal signer never grants managed input.
    let (ok, attach) = f.run(&["terminal", "attach", p, &id]);
    assert!(!ok, "{attach}");
    assert_eq!(attach["error"]["code"], "INPUT_NOT_IMPLEMENTED");

    // Cancellation is the domain's `End`. Its preview reports the real consequence
    // and never a confirmed isolation.
    let cancel_path = f.root.join("cancel.json");
    let cancel_input = cancel_path.to_str().unwrap();
    let write_cancel = |version: i64| {
        std::fs::write(
            &cancel_path,
            json!({"project_id":p,"invocation_id":id,"state_version":version,"reason":"operator ended the demonstration"})
                .to_string(),
        )
        .unwrap();
    };
    write_cancel(state_version(&running));
    let (ok, invented) = f.run(&[
        "invocation",
        "cancel",
        "--key",
        "cancel-rest",
        "--input",
        cancel_input,
        "--preview-token",
        "invented",
    ]);
    assert!(!ok, "{invented}");
    assert_eq!(invented["error"]["code"], "PREVIEW_REQUIRED");
    write_cancel(state_version(&running) + 100);
    let (ok, stale) = f.run(&[
        "invocation",
        "cancel",
        "--key",
        "cancel-stale",
        "--input",
        cancel_input,
    ]);
    assert!(!ok, "{stale}");
    assert_eq!(stale["error"]["code"], "VERSION_CONFLICT");
    assert_eq!(
        stale["error"]["message"],
        "cancellation names a stale state version"
    );
    let mut failed: Value = serde_json::from_slice(&std::fs::read(&cancel_path).unwrap()).unwrap();
    failed["outcome"] = json!("failed");
    let failed_path = f.root.join("cancel-failed.json");
    std::fs::write(&failed_path, failed.to_string()).unwrap();
    let (ok, refused) = f.run(&[
        "invocation",
        "cancel",
        "--key",
        "cancel-failed",
        "--input",
        failed_path.to_str().unwrap(),
    ]);
    assert!(!ok, "{refused}");
    assert_eq!(refused["error"]["code"], "INVOCATION_COMMAND_FAILED");
    assert_eq!(
        refused["error"]["message"],
        "only a cancelled outcome is a client command"
    );
    write_cancel(state_version(&running));
    let (ok, cancel_plan) = f.run(&[
        "invocation",
        "cancel",
        "--key",
        "cancel-rest",
        "--input",
        cancel_input,
    ]);
    assert!(ok, "{cancel_plan}");
    let effect = &cancel_plan["effect_summary"];
    assert_eq!(effect["state"], "running");
    assert_eq!(effect["outcome"], "cancelled");
    assert_eq!(effect["state_version"], running["state_version"]);
    assert!(
        effect["cleanup_pending"].as_bool().unwrap(),
        "{cancel_plan}"
    );
    assert_eq!(
        effect["isolation_confirmed"],
        json!(false),
        "a queued stop is not a confirmed isolation"
    );
    let (ok, cancelled) = f.run(&[
        "invocation",
        "cancel",
        "--key",
        "cancel-rest",
        "--input",
        cancel_input,
        "--preview-token",
        cancel_plan["preview_token"].as_str().unwrap(),
    ]);
    assert!(ok, "{cancelled}");
    assert_eq!(cancelled["state"], "cancelled");
    assert_eq!(cancelled["authorization_revoked"], json!(true));
    assert_eq!(
        cancelled["state_version"],
        json!(state_version(&running) + 1)
    );
    assert!(
        cancelled["cleanup_pending"].as_bool().unwrap(),
        "{cancelled}"
    );
    // A second cancellation under a new key finds a terminal Invocation and writes
    // nothing, so it cannot enqueue a second stop for the same dispatch.
    write_cancel(state_version(&running) + 1);
    let (ok, twice) = f.run(&[
        "invocation",
        "cancel",
        "--key",
        "cancel-twice",
        "--input",
        cancel_input,
    ]);
    assert!(!ok, "{twice}");
    assert_eq!(twice["error"]["code"], "INVALID_TRANSITION");

    let (ok, after) = f.run(&["invocation", "list", p]);
    assert!(ok, "{after}");
    let revoked = &after["invocations"][0];
    assert_eq!(revoked["state"], "cancelled");
    assert_eq!(revoked["state_version"], json!(state_version(&running) + 1));
    assert_ne!(
        revoked["owner"], running["owner"],
        "revocation advances the authorization root"
    );

    // Retry names the exact revoked authorization and freezes its own Bundle.
    let retry_of = f.root.join("retry-of.json");
    std::fs::write(&retry_of, revoked["owner"].to_string()).unwrap();
    let stale_owner = f.root.join("stale-owner.json");
    std::fs::write(&stale_owner, running["owner"].to_string()).unwrap();
    let retry_path = f.root.join("retry.json");
    std::fs::write(
        &retry_path,
        json!({"project_id":p,"room_id":room,"target":"research","profile":setup.profile["revision"],"request":"Return the exact answer without modifying files","budget":65536,"deadline_ms":deadline(),"retry_of":null})
            .to_string(),
    )
    .unwrap();
    let retry_input = retry_path.to_str().unwrap();
    let retry_of_input = retry_of.to_str().unwrap();
    let (ok, stale_retry) = f.run(&[
        "invocation",
        "retry",
        "--key",
        "retry-stale",
        "--input",
        retry_input,
        "--retry-of",
        stale_owner.to_str().unwrap(),
    ]);
    assert!(!ok, "{stale_retry}");
    assert_eq!(stale_retry["error"]["code"], "RETRY_NOT_ALLOWED");
    // An input file that names a different original is refused before any request.
    let mut claimed: Value = serde_json::from_slice(&std::fs::read(&retry_path).unwrap()).unwrap();
    claimed["retry_of"] = running["owner"].clone();
    let claimed_path = f.root.join("claimed-retry.json");
    std::fs::write(&claimed_path, claimed.to_string()).unwrap();
    let (ok, differs) = f.run(&[
        "invocation",
        "retry",
        "--key",
        "retry-claimed",
        "--input",
        claimed_path.to_str().unwrap(),
        "--retry-of",
        retry_of_input,
    ]);
    assert!(!ok, "{differs}");
    assert_eq!(differs["error"]["code"], "INVOCATION_COMMAND_FAILED");
    assert_eq!(
        differs["error"]["message"],
        "input retry_of differs from --retry-of"
    );
    let (ok, retry_plan) = f.run(&[
        "invocation",
        "retry",
        "--key",
        "retry-1",
        "--input",
        retry_input,
        "--retry-of",
        retry_of_input,
    ]);
    assert!(ok, "{retry_plan}");
    assert_ne!(
        retry_plan["effect_summary"]["assembly"]["bundle"]["document"]["id"], bundle,
        "a retry freezes its own Bundle"
    );
    let (ok, retried) = f.run(&[
        "invocation",
        "retry",
        "--key",
        "retry-1",
        "--input",
        retry_input,
        "--retry-of",
        retry_of_input,
        "--preview-token",
        retry_plan["preview_token"].as_str().unwrap(),
    ]);
    assert!(ok, "{retried}");
    let retry_id = retried["invocation_id"].as_str().unwrap().to_owned();
    assert_ne!(retry_id, id);
    assert_eq!(retried["state"], "pending");
    assert_eq!(retried["state_version"], json!(1));
    // The Agency is gone, so the retry can never commit a dispatch to observe.
    let (ok, unmapped) = f.run(&["terminal", "inspect", p, &retry_id]);
    assert!(!ok, "{unmapped}");
    assert_eq!(unmapped["error"]["code"], "INVALID_INPUT");
    assert_eq!(unmapped["error"]["message"], "dispatch mapping required");
    let (ok, both) = f.run(&["invocation", "list", p]);
    assert!(ok, "{both}");
    assert_eq!(both["invocations"].as_array().unwrap().len(), 2, "{both}");

    // Updating the pointer freezes a new immutable revision; reading shows exactly it.
    let update_path = f.root.join("profile-update.json");
    let mut definition = shown["profile"].clone();
    definition["max_context_bytes"] = json!(32768);
    std::fs::write(
        &update_path,
        json!({"id":"research","version":1,"profile":definition}).to_string(),
    )
    .unwrap();
    let update_input = update_path.to_str().unwrap();
    let (ok, update_plan) = f.run(&[
        "profile",
        "update",
        "--key",
        "profile-2",
        "--input",
        update_input,
    ]);
    assert!(ok, "{update_plan}");
    let (ok, updated) = f.run(&[
        "profile",
        "update",
        "--key",
        "profile-2",
        "--input",
        update_input,
        "--preview-token",
        update_plan["preview_token"].as_str().unwrap(),
    ]);
    assert!(ok, "{updated}");
    assert_eq!(updated["version"], json!(2));
    let (ok, current) = f.run(&["profile", "show", "research"]);
    assert!(ok, "{current}");
    assert_eq!(current["pointer"]["version"], json!({"state":2}));
    assert_eq!(current["profile"]["max_context_bytes"], json!(32768));
    assert_ne!(
        current["revision"], shown["revision"],
        "revisions are immutable"
    );
    // The old pointer version is spent; a stale update cannot land on the new one.
    let (ok, spent) = f.run(&[
        "profile",
        "update",
        "--key",
        "profile-3",
        "--input",
        update_input,
    ]);
    assert!(!ok, "{spent}");
    assert_eq!(spent["error"]["code"], "VERSION_CONFLICT");
}
#[test]
fn b1_register_two_projects_native_rooms_same_card_contract_request_and_restart() {
    let (f, port) = Fixture::packaged("project-b1");
    assert!(f.run(&["start", "--secret-backend", "user-file"]).0);
    let input = json!({"name":"b1","origin":"local","platform":"local","platform_path":"b1","default_source":"gitea_issues"});
    let registered = accepted(&f, "repo", "register", "register-b1", input);
    let registration = &registered["registration"];
    let repo = registration["repo_id"].as_str().unwrap().to_owned();
    let version = registration["version"].as_i64().unwrap().to_string();
    let platform_id = registration["observed"]["stable_id"].as_str().unwrap();
    let args = [
        "repo",
        "register",
        "--confirm",
        &repo,
        "--version",
        &version,
        "--platform-repo-id",
        platform_id,
        "--key",
        "confirm-b1",
    ];
    let (ok, preview) = f.run(&args);
    assert!(ok, "{preview}");
    let mut args = args.to_vec();
    args.extend([
        "--preview-token",
        preview["preview_token"].as_str().unwrap(),
    ]);
    let (ok, result) = f.run(&args);
    assert!(ok, "{result}");
    assert_eq!(result["lifecycle"], "active");
    let mut projects = vec![];
    for name in ["A", "B"] {
        let create = json!({"repo_id":repo,"definition":project_definition(name)});
        let result = accepted(
            &f,
            "project",
            "create",
            &format!("project-{name}"),
            create.clone(),
        );
        assert_eq!(
            accepted(&f, "project", "create", &format!("project-{name}"), create)["project_id"],
            result["project_id"]
        );
        projects.push((
            result["project_id"].as_str().unwrap().to_owned(),
            result["main_room_id"].as_str().unwrap().to_owned(),
        ));
    }
    assert_ne!(projects[0].0, projects[1].0);
    assert_ne!(projects[0].1, projects[1].1);
    let source = accepted(
        &f,
        "task",
        "connect",
        "source-b1",
        json!({"repo_id":repo,"candidate_id":"gitea_issues","consent":true,"make_default":false}),
    );
    let source = source["source_id"].as_str().unwrap();
    let mut topics = vec![];
    for (p, main) in &projects {
        let show = f.show(p, main);
        assert_eq!(show["room"]["participants"], json!([]));
        let sent = accepted(
            &f,
            "room",
            "send",
            &format!("send-{p}"),
            json!({"project_id":p,"room_id":main,"version":show["binding"]["version"],"body":"中文前情"}),
        );
        let event = sent["receipt"]["event_id"].as_str().unwrap();
        let draft=f.query("draft",json!({"project_id":p,"project_version":1,"origin":{"kind":"room","room_id":main,"binding_version":show["binding"]["version"]},"selection":{"kind":"events","event_ids":[event]}}));
        let sources: Vec<Value> = draft["fragments"]
            .as_array()
            .unwrap_or_else(|| panic!("draft has no fragments: {draft}"))
            .iter()
            .map(|t| t["source"].clone())
            .collect();
        let topic = accepted(
            &f,
            "room",
            "create-topic",
            &format!("topic-{p}"),
            json!({"project_id":p,"project_version":1,"name":"topic","origin":{"kind":"room","room_id":main,"binding_version":show["binding"]["version"]},"brief":{"context_and_goal":"confirmed topic","settled_facts_and_reasons":[],"disagreements_and_questions":[],"constraints_and_materials":[],"sources":sources},"participants":[],"roster_confirmed":true}),
        );
        let room = topic["room_id"].as_str().unwrap().to_owned();
        f.query("save-view-state",json!({"project_id":p,"room_id":room,"binding_version":f.show(p,&room)["binding"]["version"],"client_id":"b1","draft":"未发送草稿","read_cursor":null}));
        topics.push(room);
        accepted(
            &f,
            "task",
            "attach",
            &format!("attach-{p}"),
            json!({"project_id":p,"project_version":1,"source_id":source,"approved_scope":platform_id,"consent":true}),
        );
    }
    // Patch 1a, from a human account's own view: a Topic created after the
    // human joined the main Room invites them by default, carries the
    // confirmed brief as its opening message, and the carrier Space follows
    // the main Room's membership.
    let registration_token =
        std::fs::read_to_string(f.root.join("services/config/tuwunel-registration-token")).unwrap();
    let registration =
        std::fs::read_to_string(f.root.join("services/config/appservices/hctl2.yaml")).unwrap();
    let as_token = serde_json::from_str::<Value>(&registration).unwrap()["as_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let server = chat::Server {
        binding: store::Reference {
            key: chat::key(store::Scope::Control, "chat_server", "packaged"),
            version: store::Version::State(1),
        },
        url: format!("http://127.0.0.1:{port}"),
        server_name: "hctl2.localhost".into(),
        sender: "@hctl2_control:hctl2.localhost".into(),
    };
    let matrix = control::MatrixClient::new(server, as_token).unwrap();
    let (human_id, human_token) = matrix
        .human_register("b1human", "b1-password", registration_token.trim())
        .unwrap();
    let (main_project, main_room) = &projects[0];
    let main_binding = f.show(main_project, main_room)["binding"].clone();
    accepted(
        &f,
        "project",
        "members",
        "members-human",
        json!({"project_id":main_project,"project_version":1,"rooms":[{"key":main_binding["key"],"version":{"state":main_binding["version"]}}],"users":[human_id],"invite":true}),
    );
    let main_external =
        f.show(main_project, main_room)["binding"]["data"]["value"]["matrix_room_id"]
            .as_str()
            .unwrap()
            .to_owned();
    matrix.human_join(&human_token, &main_external).unwrap();
    // One human-visible message in the main Room is the Topic's source.
    let human_send = accepted(
        &f,
        "room",
        "send",
        "send-human-view",
        json!({"project_id":main_project,"room_id":main_room,"version":f.show(main_project, main_room)["binding"]["version"],"body":"人的视角来源消息"}),
    );
    let human_event = human_send["receipt"]["event_id"].as_str().unwrap();
    let human_digest = {
        let content = json!({"body":"人的视角来源消息","msgtype":"m.text"});
        let canonical = foundation::canonical_json(&content).unwrap();
        foundation::bytes_sha256(&canonical)
    };
    let source_json = json!({"kind":"message","binding":{"key":main_binding["key"],"version":{"state":main_binding["version"]}},"event_id":human_event,"content_digest":human_digest});
    let human_brief = json!({"context_and_goal":"人的视角","settled_facts_and_reasons":[],"disagreements_and_questions":[],"constraints_and_materials":[],"sources":[source_json]});
    let human_topic = accepted(
        &f,
        "room",
        "create-topic",
        "topic-human-view",
        json!({"project_id":main_project,"project_version":1,"name":"人的视角","origin":{"kind":"room","room_id":main_room,"binding_version":f.show(main_project, main_room)["binding"]["version"]},"brief":human_brief,"participants":[],"roster_confirmed":true}),
    );
    assert_eq!(
        human_topic["invites"],
        json!([human_id]),
        "the confirmed list keeps naming users: {}",
        human_topic["invites"]
    );
    let invited: Vec<&str> = human_topic["invite_results"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|entry| {
            entry["receipt"]["members"]
                .as_array()
                .unwrap_or_else(|| panic!("invite entry without receipt: {entry}"))
                .iter()
        })
        .map(|member| member["user_id"].as_str().unwrap())
        .collect();
    assert_eq!(
        invited,
        vec![human_id.as_str()],
        "the joined human is the default invite list"
    );
    assert_eq!(human_topic["opening"]["delivery"], json!("confirmed"));
    assert_eq!(
        human_topic["invite_results"][0]["user_id"],
        json!(human_id),
        "per-target outcomes name their user"
    );
    assert_eq!(
        human_topic["invite_results"][0]["delivery"],
        json!("confirmed")
    );
    let human_topic_room = human_topic["room_id"].as_str().unwrap().to_owned();
    let topic_external =
        f.show(main_project, &human_topic_room)["binding"]["data"]["value"]["matrix_room_id"]
            .as_str()
            .unwrap()
            .to_owned();
    matrix.human_join(&human_token, &topic_external).unwrap();
    let messages = matrix
        .human_messages(&human_token, &topic_external)
        .unwrap();
    let bodies: Vec<&Value> = messages["chunk"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["type"] == "m.room.message")
        .map(|e| &e["content"]["body"])
        .collect();
    let brief_struct: chat::Brief = serde_json::from_value(human_brief.clone()).unwrap();
    assert_eq!(
        bodies.last(),
        Some(&&json!(chat::opening_body(&brief_struct))),
        "the human sees the confirmed brief as the opening message"
    );
    let hierarchy = f.run(&["room", "hierarchy", main_project, main_room]);
    assert!(hierarchy.0, "{:?}", hierarchy.1);
    let space = hierarchy.1["carrier_space_id"].as_str().unwrap();
    matrix.human_join(&human_token, space).unwrap();
    let space_membership = matrix
        .human_state(&human_token, space, "m.room.member", &human_id)
        .unwrap();
    assert_eq!(
        space_membership["membership"],
        json!("join"),
        "the carrier Space membership follows the main Room"
    );
    let created = accepted(
        &f,
        "task",
        "create",
        "task-b1",
        json!({"project_id":projects[0].0,"project_version":1,"source_id":source,"title":"shared native card","body":"work"}),
    );
    let task_a = created["task_id"].as_str().unwrap().to_owned();
    let (ok, task) = f.run(&["task", "show", &projects[0].0, &task_a]);
    assert!(ok, "{task}");
    let entity = task["data"]["entity"]["immutable_external_entity_id"]
        .as_str()
        .unwrap();
    accepted(
        &f,
        "task",
        "refresh",
        "refresh-b1",
        json!({"repo_id":repo,"source_id":source}),
    );
    let other = accepted(
        &f,
        "task",
        "claim",
        "claim-b1",
        json!({"project_id":projects[1].0,"project_version":1,"source_id":source,"entity_id":entity}),
    );
    let task_b = other["task_id"].as_str().unwrap().to_owned();
    assert_ne!(task_a, task_b);
    for ((p, _), task) in projects.iter().zip([&task_a, &task_b]) {
        let (ok, before) = f.run(&["task", "show", p, task]);
        assert!(ok, "{before}");
        let contract = json!({"scope":"B1","expected_outcome":"complete independently","acceptance":[{"text":"human approves","grade":"human"}],"roles":[],"capabilities":[]});
        let digest = foundation::canonical_json_sha256(&contract).unwrap();
        let adoption = json!({"contract":contract,"origin":{"kind":"local","reference":{"key":{"scope":{"kind":"project","id":p},"kind":"project","id":p},"version":{"state":1}},"proposal_digest":digest}});
        if p == &projects[0].0 {
            let request = accepted(
                &f,
                "request",
                "create",
                "request-b1",
                json!({"project_id":p,"project_version":1,"request":{
                    "question":"approve contract?","target":{"kind":"human","principal":format!("local-owner:{}",std::fs::metadata(&f.root).unwrap().uid())},
                    "owner":{"key":{"scope":{"kind":"project","id":p},"kind":"task_state","id":task},"version":{"state":before["version"]}},
                    "affected_revision":null,"blocking_scope":"contract_adoption","owner_state_version":before["data"]["state_version"],"dedup_root":"b1-contract",
                    "permissions":{"action":"task.adopt"},"deadline":null,"deadline_action":"fail_waiting","action":"task_adopt","input_schema":"hctl2.task.Adoption.v1"
                }}),
            );
            let (ok, pending) = f.run(&["project", "pending", p]);
            assert!(ok);
            assert_eq!(pending["items"].as_array().unwrap().len(), 1);
            let answer = json!({"project_id":p,"request_id":request["request_id"],"version":1,"adoption":adoption});
            let result = accepted(&f, "request", "resolve", "answer-b1", answer.clone());
            assert_eq!(result["delivery"], "confirmed");
            assert_eq!(
                accepted(&f, "request", "resolve", "answer-b1", answer)["request_id"],
                result["request_id"]
            );
            assert!(
                f.run(&["project", "pending", p]).1["items"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            );
        } else {
            accepted(
                &f,
                "task",
                "adopt",
                &format!("adopt-{p}"),
                json!({"project_id":p,"project_version":1,"task_id":task,"version":before["version"],"adoption":adoption}),
            );
        }
    }
    let mut saved = vec![];
    for (index, task) in [&task_a, &task_b].into_iter().enumerate() {
        let p = &projects[index].0;
        let (ok, saved_task) = f.run(&["task", "show", p, task]);
        assert!(ok);
        let saved_room = f.show(p, &topics[index]);
        let (ok, saved_view) = f.run(&["room", "view-state", p, &topics[index], "b1"]);
        assert!(ok, "{saved_view}");
        saved.push((saved_task, saved_room, saved_view));
    }
    // Kill rather than graceful stop: restart must recover both authoritative and native state.
    let pid = std::fs::read_to_string(f.root.join("control.pid")).unwrap();
    assert!(
        Command::new("kill")
            .args(["-KILL", pid.trim()])
            .status()
            .unwrap()
            .success()
    );
    let (ok, services) = f.run(&["services", "status"]);
    assert!(!ok, "dead control should not answer: {services}");
    assert!(
        Command::new(f.payload.join("bin/hctl2-services"))
            .env("HCTL2_STATE_ROOT", f.root.join("services"))
            .arg("stop")
            .status()
            .unwrap()
            .success()
    );
    assert!(f.run(&["start", "--secret-backend", "user-file"]).0);
    let mut ready = false;
    for _ in 0..100 {
        let (ok, view) = f.run(&["room", "view-state", &projects[0].0, &topics[0], "b1"]);
        if ok {
            assert_eq!(view, saved[0].2);
            ready = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(ready, "native services did not recover");
    for (index, task) in [&task_a, &task_b].into_iter().enumerate() {
        let p = &projects[index].0;
        assert_eq!(f.show(p, &topics[index])["room"], saved[index].1["room"]);
        assert_eq!(f.run(&["task", "show", p, task]).1, saved[index].0);
        assert_eq!(
            f.run(&["room", "view-state", p, &topics[index], "b1"]).1,
            saved[index].2
        );
        assert_eq!(
            f.show(p, &projects[index].1)["room"]["id"],
            projects[index].1
        );
    }
}

#[test]
fn human_output_renders_dispatch_preview_sections_and_invocation_table() {
    let (f, setup) = paired("human-dispatch", 60);
    let p = setup.project.as_str();
    let room = setup.room.as_str();
    let path = f.root.join("invocation.json");
    let deadline = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
        + 60000;
    std::fs::write(
        &path,
        json!({"project_id":p,"room_id":room,"target":"research","profile":setup.profile["revision"],
            "request":"Return the exact answer without modifying files","budget":65536,
            "deadline_ms":deadline,"retry_of":null})
        .to_string(),
    )
    .unwrap();
    let input = path.to_str().unwrap();
    let (ok, human, stderr) = f.run_raw(
        false,
        &[
            "invocation",
            "preview",
            "--input",
            input,
            "--key",
            "human-dispatch",
        ],
    );
    assert!(ok, "{}", String::from_utf8_lossy(&stderr));
    let human = String::from_utf8(human).unwrap();
    for heading in [
        "Dispatch preview",
        "Object",
        "Why this needs confirmation",
        "What happens after you confirm",
        "Confirm with",
    ] {
        assert!(
            human.contains(heading),
            "missing section {heading}:\n{human}"
        );
    }
    // The object section names the real dispatch targets, not placeholders.
    for value in [p, room, "research"] {
        assert!(
            human.contains(value),
            "object section must name {value}:\n{human}"
        );
    }
    assert!(
        human.contains("invocation cancel"),
        "the undo path must be named:\n{human}"
    );
    assert!(!human.contains('\u{1b}'), "no ANSI escapes:\n{human}");
    // The printed confirm command is executable as printed: take the line under
    // "Confirm with", strip the binary, run it.
    let confirm_line = human
        .lines()
        .skip_while(|line| !line.contains("Confirm with"))
        .nth(1)
        .expect("a confirm command line")
        .trim();
    assert!(confirm_line.contains("--preview-token"), "{confirm_line}");
    let words = confirm_line.split_whitespace().collect::<Vec<_>>();
    let confirmed = Command::new(words[0])
        .env_remove("HCTL2_PROCESS_COMPOSE_BIN")
        .env("HCTL2_INSTALL_ROOT", &f.payload)
        .env(
            "HCTL2_CONTROL_BIN",
            std::env::var("CARGO_BIN_EXE_hctl2-control").unwrap(),
        )
        .args(&words[1..])
        .output()
        .unwrap();
    assert!(
        confirmed.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&confirmed.stdout),
        String::from_utf8_lossy(&confirmed.stderr)
    );
    // The default invocation listing is a table with a header and this row.
    let (ok, listed, _) = f.run_raw(false, &["invocation", "list", p]);
    assert!(ok);
    let listed = String::from_utf8(listed).unwrap();
    let mut lines = listed.lines();
    let header = lines.next().expect("a header row");
    assert_eq!(
        header.split_whitespace().collect::<Vec<_>>(),
        ["invocation", "state", "version", "reason"]
    );
    let separator = lines.next().expect("a separator row");
    assert!(separator.starts_with("-----------"), "{separator}");
    let row = lines
        .next()
        .expect("one invocation row")
        .split_whitespace()
        .collect::<Vec<_>>();
    assert!(row[0].starts_with("invocation-"), "{row:?}");
    assert!(
        row[1] == "pending" || row[1] == "running",
        "the dispatched invocation is listed with its state: {row:?}"
    );
    assert!(!listed.contains('\u{1b}'));
    // The machine interface stays byte-stable where the content allows it: the
    // catalog is fixed by the paired fixture, so two runs must agree byte for
    // byte and match the saved golden sample — except the digest bytes, which
    // are the fingerprint of the program file the fixture runs. Acceptance 1's
    // "` + "`--json`" + ` byte-identical" compares the same environment before
    // and after a change; a program file differs across platforms and build
    // presets, so the golden holds a placeholder where the digest sits, and the
    // live bytes are compared with the digest computed from the fixture's own
    // program file at run time. Everything else stays byte for byte.
    let (ok, first, _) = f.run_raw(true, &["agency", "catalog", "local"]);
    assert!(ok);
    let (_, second, _) = f.run_raw(true, &["agency", "catalog", "local"]);
    assert_eq!(
        first, second,
        "agency catalog --json must be deterministic in one fixture"
    );
    let catalog = String::from_utf8(first).unwrap();
    let program =
        serde_json::from_slice::<Value>(&std::fs::read(f.root.join("script.json")).unwrap())
            .unwrap()["program"]
            .as_str()
            .expect("the fixture names its program file")
            .to_owned();
    let program_digest = agency_proto::hash(&std::fs::read(&program).unwrap());
    // The fingerprint appears as the harness digest, the profession's harness
    // digest and its reference digest; all three cover the same program file.
    assert_eq!(
        catalog.matches(&program_digest).count(),
        3,
        "the catalog carries the fixture program's digest:\n{catalog}"
    );
    let normalized = catalog.replace(&program_digest, "FIXTURE-PROGRAM-DIGEST");
    assert_eq!(
        normalized.trim_end(),
        AGENCY_CATALOG_JSON.trim_end(),
        "agency catalog --json drifted from the saved sample"
    );
    let (ok, human, _) = f.run_raw(false, &["agency", "catalog", "local"]);
    assert!(ok);
    let human = String::from_utf8(human).unwrap();
    assert!(human.contains("Professions"), "{human}");
    let header = human
        .lines()
        .find(|line| line.starts_with("profession"))
        .expect("a profession header row");
    assert_eq!(
        header.split_whitespace().collect::<Vec<_>>(),
        ["profession", "revision", "harness", "persona"],
        "the catalog is a table with headers:\n{human}"
    );
    assert!(human.contains("script-worker"), "{human}");
    assert!(!human.contains('\u{1b}'));
}

/// Saved ` + "`--json`" + ` sample of ` + "`agency catalog`" + `, which needs a live Agency. The digest
/// positions hold the ` + "`FIXTURE-PROGRAM-DIGEST`" + ` placeholder: those bytes are the fingerprint of
/// the program file the fixture runs (` + "`ScriptRuntime`" + ` hashes ` + "`config.program`" + `), which is not
/// stable across platforms or build presets. The test substitutes the digest
/// computed from the fixture's own program file before comparing, so every
/// other byte is still pinned.
const AGENCY_CATALOG_JSON: &str = r#"{"harnesses":[{"digest":"FIXTURE-PROGRAM-DIGEST","id":"script-protocol-fixture","revision":"1"}],"professions":[{"capabilities":{"event_cursor":true,"exact_attach":false,"input":true,"input_provenance":true,"isolation_effects":[],"managed_single_writer":true,"secure_input":false,"stop":true,"tool_execution_unmediated":false},"default_role":"fixture","harness":{"digest":"FIXTURE-PROGRAM-DIGEST","id":"script-protocol-fixture","revision":"1"},"model":"none","persona":"protocol test executor","reference":{"digest":"FIXTURE-PROGRAM-DIGEST","id":"script-worker","revision":"1"},"skills":[],"terms":"not a coding harness; no PTY, tool provenance or OS hardening"}],"skills":[]}"#;
