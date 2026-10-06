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

#[test]
fn dispatch_from_real_cli_pairing_to_room_answer_and_restart_keeps_one_invocation() {
    let (f, _) = Fixture::packaged("dispatch-chain");
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
    let p = created["project_id"].as_str().unwrap();
    let room = created["main_room_id"].as_str().unwrap();
    let agency_root = f.root.join("independent-agency");
    let config = f.root.join("script.json");
    let delay = f.root.join("delay-script");
    std::fs::write(&config, json!({"program":"/bin/sh","arguments":["-c","read -r initial; if test -f \"$1\"; then sleep 2; fi; printf '%s\\n' '{\"type\":\"result\",\"schema\":\"adapter.stdout.v1\",\"output\":\"DISPATCH_CHAIN_OK\"}'","dispatch-fixture",delay]}).to_string()).unwrap();
    struct AgencyChild(std::process::Child);
    impl Drop for AgencyChild {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let binary = std::env::var_os("CARGO_BIN_EXE_agency").unwrap();
    let mut agency = AgencyChild(
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
    let profession = &catalog["professions"][0];
    let reference_path = f.root.join("profession.json");
    std::fs::write(&reference_path, profession["reference"].to_string()).unwrap();
    let (ok, accepted_profession) = f.run(&[
        "agency",
        "accept",
        "--binding-id",
        "local",
        "--reference",
        reference_path.to_str().unwrap(),
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
        json!({"project_id":p,"project_version":1,"room_id":room,"topic_command_key":null,"roster_version":null,"selections":[selection]}),
    );
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
    std::fs::write(&delay, b"delay").unwrap();
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
