//! 第 7 包：供应端 Done 归档出的请求，经 control 走同一条预览与准入。
//! 这里用真 Store + 真领域命令，只是把「观测那次」换成本地播种的请求记录。
use super::*;
use std::sync::Arc;
use store::{ProjectSettings, Record, RecordData, Reference, Version};

fn actor() -> TrustedActor {
    TrustedActor(store::Actor {
        principal: "owner".into(),
        source: store::ActorSource::DirectClient,
        permission_scope: vec![Scope::Control],
        authority: None,
    })
}

/// 播种一条记录（内部 actor；新建用 `Absent`，更新按前一个版本 CAS）。
fn seed(s: &mut Store, record: Record) {
    let a = task::owner(&actor(), [record.key.scope.clone()]).unwrap();
    let input = serde_json::to_value(&record).unwrap();
    let expected = if record.version > 1 {
        store::Expected::Exact(Version::State(record.version - 1))
    } else {
        store::Expected::Absent
    };
    let cmd = store::Command {
        command_id: format!("seed:{}:{}", record.key.id, record.version),
        idempotency_key: format!("seed:{}:{}", record.key.id, record.version),
        actor: a.0.clone(),
        target: record.key.clone(),
        expected,
        binding: task::reference(&record),
        input_digest: store::Command::digest_input("fixture", &input).unwrap(),
        operation: "fixture".into(),
        input,
    };
    s.submit(s.generation(), &a, &cmd, None, |tx| {
        tx.put(&record)?;
        Ok(json!({}))
    })
    .unwrap();
}

fn project(s: &mut Store, repo_id: &str, id: &str) {
    let data = RecordData::Project {
        repo_id: repo_id.into(),
        settings: ProjectSettings {
            publish_review_requires_confirmation: true,
            selection_policy: json!({}),
        },
        archived: false,
    };
    let record = Record {
        key: task::key(Scope::Project(id.into()), "project", id),
        version: 1,
        revision_digest: foundation::canonical_json_sha256(&serde_json::to_value(&data).unwrap())
            .unwrap(),
        data,
        sources: vec![],
        materials: vec![],
    };
    seed(s, record);
}

/// 注册一个外部仓库（与 CLI 测试同一套真命令）。
fn register_repo(s: &mut Store) -> String {
    let a = actor();
    let request = serde_json::from_value(json!({
        "name": "repo",
        "origin": "external",
        "platform": "github",
        "instance": "github.com",
        "platform_repo_id": "77",
        "platform_path": "owner/repo",
        "default_source": "github_issues",
    }))
    .unwrap();
    let reg = repo::admit(s, &a, "repo", "repo", repo::prepare(request, None).unwrap()).unwrap();
    repo::begin_step(s, &a, &reg.repo_id, "platform").unwrap();
    repo::confirm_platform(
        s,
        &reg.repo_id,
        repo::PlatformObservation {
            instance: "github.com".into(),
            stable_id: "77".into(),
            full_name: "owner/repo".into(),
            clone_url: "https://github.com/owner/repo.git".into(),
            account_id: "1".into(),
            has_issues: true,
            can_write_issues: true,
            credential_ref: String::new(),
        },
    )
    .unwrap();
    reg.repo_id
}

/// 一个带 Task 与已采纳契约的项目；契约的验收项由调用方给。
fn task_with_contract(s: &mut Store, repo_id: &str, contract: task::Contract) -> String {
    let a = task::owner(&actor(), [Scope::Project("A".into())]).unwrap();
    seed(
        s,
        task::value_record(
            task::key(Scope::Repo(repo_id.into()), "task_source", "source"),
            1,
            &json!({
                "id": "source",
                "repo_id": repo_id,
                "port_kind": "task_source",
                "candidate": {
                    "id": "gitea_issues", "provider": "gitea_issues",
                    "actual_source_and_scope": "owner/repo", "recommended": true,
                    "create": true, "field_writeback": true, "can_claim": true, "available": true,
                },
                "platform": {
                    "instance": "http://127.0.0.1:3000", "stable_id": "1",
                    "full_name": "owner/repo", "clone_url": "http://127.0.0.1:3000/owner/repo.git",
                    "account_id": "1", "has_issues": true, "can_write_issues": true,
                    "credential_ref": "",
                },
                "capabilities": {
                    "create": true, "field_writeback": true, "conditional_write": false,
                    "conditional_fields": [], "placement": false, "delete": false,
                },
                "board_scope_stable_id": "1",
                "binding_revision": 1,
                "active": true,
                "human_account": "human",
                "auto_complete_provider_done": true,
            }),
        )
        .unwrap(),
    );
    // `Claim` 需要一个已被观测到的实体：先给出该实体的快照，再认领。
    // 项目必须先把这条源映射批准进来，`Claim` 才认。
    let plan = task::prepare(
        s,
        task::Input {
            key: "attach".into(),
            action: task::Action::Attach {
                project_id: "A".into(),
                project_version: 1,
                source_id: "source".into(),
                approved_scope: "1".into(),
                group: None,
                consent: true,
            },
        },
    )
    .unwrap();
    task::admit(s, &a, plan).unwrap();
    let card = json!({
        "entity": {
            "provider": "gitea_issues@127.0.0.1:3000", "account_stable_id": "1",
            "external_entity_kind": "issue", "immutable_external_entity_id": "node1",
        },
        "number": 1, "title": "task", "body": "body", "stage": "open",
        "remote_revision": "r1", "content_version": null, "groups": [],
        "dependencies": {"parent": null, "children": [], "blocked_by": [], "blocking": []},
        "raw": {"title": "task"}, "comments": [], "tombstone": false,
    });
    let (sr, src) = task::source(s, repo_id, "source").unwrap();
    let snapshot = json!({
        "source": task::reference(&sr),
        "observed_at": task::now(),
        "complete": true,
        "error": null,
        "cards": [card],
        "stable_groups": [],
    });
    task::observe(s, &src, serde_json::from_value(snapshot).unwrap()).unwrap();
    let plan = task::prepare(
        s,
        task::Input {
            key: "claim".into(),
            action: task::Action::Claim {
                project_id: "A".into(),
                project_version: 1,
                source_id: "source".into(),
                entity_id: "node1".into(),
            },
        },
    )
    .unwrap();
    let claimed = task::admit(s, &a, plan).unwrap();
    let id = claimed["task_id"].as_str().unwrap().to_owned();
    let (record, _) = task::task(s, "A", &id).unwrap();
    let adoption = task::Adoption {
        origin: task::ContractOrigin::Local {
            reference: Reference {
                key: task::key(Scope::Project("A".into()), "project", "A"),
                version: Version::State(1),
            },
            proposal_digest: foundation::canonical_json_sha256(
                &serde_json::to_value(&contract).unwrap(),
            )
            .unwrap(),
        },
        contract,
    };
    let plan = task::prepare(
        s,
        task::Input {
            key: "adopt".into(),
            action: task::Action::Adopt {
                project_id: "A".into(),
                project_version: 1,
                task_id: id.clone(),
                version: record.version,
                adoption,
            },
        },
    )
    .unwrap();
    task::admit(s, &a, plan).unwrap();
    id
}

/// 播种「供应端 Done」归档出的请求与它引用的观测记录。
fn seed_request(
    s: &mut Store,
    repo_id: &str,
    id: &str,
    snapshot_kind: &str,
) -> task::CompletionRequest {
    let snapshot = task::key(Scope::Project("A".into()), snapshot_kind, "observed");
    seed(
        s,
        task::value_record(snapshot.clone(), 1, &json!({"observed": true})).unwrap(),
    );
    let request = task::CompletionRequest {
        request_id: "request".into(),
        task_id: id.into(),
        project_id: "A".into(),
        repo_id: repo_id.into(),
        source_id: "source".into(),
        binding: Reference {
            key: task::key(Scope::Repo(repo_id.into()), "task_source", "source"),
            version: Version::State(1),
        },
        entity: "node1".into(),
        provider_actor: "human".into(),
        stage_before: "open".into(),
        stage_after: "closed".into(),
        remote_revision: "r2".into(),
        idempotency_key: "tuple".into(),
        snapshot: Reference {
            key: snapshot,
            version: Version::State(1),
        },
        observed_at: task::now(),
        state: "pending".into(),
        last_error: None,
    };
    seed(
        s,
        task::value_record(
            task::key(
                Scope::Project("A".into()),
                task::COMPLETION_REQUEST_KIND,
                "request",
            ),
            1,
            &request,
        )
        .unwrap(),
    );
    request
}

fn mechanical_contract() -> task::Contract {
    task::Contract {
        scope: "scope".into(),
        expected_outcome: "outcome".into(),
        acceptance: vec![task::Acceptance {
            text: "build is green".into(),
            grade: task::Grade::Mechanical,
            evidence: Some(task::EvidenceRequirement {
                accept: None,
                min_channel: Some("adapter_event".into()),
            }),
        }],
        roles: vec![],
        capabilities: vec![],
    }
}

fn with_human_item(mut contract: task::Contract) -> task::Contract {
    contract.acceptance.push(task::Acceptance {
        text: "person accepts".into(),
        grade: task::Grade::Human,
        evidence: None,
    });
    contract
}

fn shared(store: Store) -> Shared {
    Arc::new(Mutex::new(Some(store)))
}

#[test]
fn provider_done_auto_submits_when_mechanical_evidence_is_complete() {
    let root = std::env::temp_dir().join(format!("hctl2-provider-done-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut store = Store::open(&root).unwrap();
    let repo_id = register_repo(&mut store);
    project(&mut store, &repo_id, "A");
    let id = task_with_contract(&mut store, &repo_id, mechanical_contract());
    seed_request(&mut store, &repo_id, &id, "task_snapshot");
    let shared = shared(store);
    let driven = drive_completion_requests(&shared, "A", &actor()).unwrap();
    assert_eq!(driven[0]["state"], "accepted", "{driven}");
    let mut slot = shared.blocking_lock();
    let store = slot.as_mut().unwrap();
    let (_, t) = task::task(store, "A", &id).unwrap();
    assert_eq!(t.lifecycle, "completed");
    assert_eq!(t.lifecycle_version, 2);
    let request: task::CompletionRequest = task::decode(
        &store
            .get(&task::key(
                Scope::Project("A".into()),
                task::COMPLETION_REQUEST_KIND,
                "request",
            ))
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(request.state, "accepted");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn provider_done_is_declined_when_evidence_is_incomplete() {
    let root = std::env::temp_dir().join(format!("hctl2-provider-decline-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut store = Store::open(&root).unwrap();
    let repo_id = register_repo(&mut store);
    project(&mut store, &repo_id, "A");
    let id = task_with_contract(&mut store, &repo_id, with_human_item(mechanical_contract()));
    seed_request(&mut store, &repo_id, &id, "task_snapshot");
    let shared = shared(store);
    let driven = drive_completion_requests(&shared, "A", &actor()).unwrap();
    assert_eq!(driven[0]["state"], "declined", "{driven}");
    assert_eq!(driven[0]["error"]["code"], "EVIDENCE_REQUIRED");
    let mut slot = shared.blocking_lock();
    let store = slot.as_mut().unwrap();
    let (_, t) = task::task(store, "A", &id).unwrap();
    assert_eq!(t.lifecycle, "open");
    let request: task::CompletionRequest = task::decode(
        &store
            .get(&task::key(
                Scope::Project("A".into()),
                task::COMPLETION_REQUEST_KIND,
                "request",
            ))
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(request.state, "declined");
    assert!(request.last_error.is_some());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn provider_done_waits_when_the_binding_does_not_auto_submit() {
    let root = std::env::temp_dir().join(format!("hctl2-provider-wait-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut store = Store::open(&root).unwrap();
    let repo_id = register_repo(&mut store);
    project(&mut store, &repo_id, "A");
    let id = task_with_contract(&mut store, &repo_id, mechanical_contract());
    // 绑定把自动提交关掉：请求保持待处理，不动 Task。
    let (sr, mut src) = task::source(&store, &repo_id, "source").unwrap();
    src.auto_complete_provider_done = false;
    seed(
        &mut store,
        task::value_record(sr.key.clone(), sr.version + 1, &src).unwrap(),
    );
    seed_request(&mut store, &repo_id, &id, "task_snapshot");
    let shared = shared(store);
    let driven = drive_completion_requests(&shared, "A", &actor()).unwrap();
    assert_eq!(driven[0]["state"], "pending", "{driven}");
    assert_eq!(driven[0]["reason"], "binding_does_not_auto_submit");
    let mut slot = shared.blocking_lock();
    let store = slot.as_mut().unwrap();
    let (_, t) = task::task(store, "A", &id).unwrap();
    assert_eq!(t.lifecycle, "open");
    let _ = std::fs::remove_dir_all(&root);
}
