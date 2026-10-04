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
#[test]
fn b1_register_two_projects_native_rooms_same_card_contract_request_and_restart() {
    let (f, _) = Fixture::packaged("project-b1");
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
