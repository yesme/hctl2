use project::*;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use store::{
    Actor, ActorSource, Command, Expected, Record, Reference, RoomState, Scope, Store,
    TrustedActor, Version,
};
static NEXT: AtomicU64 = AtomicU64::new(0);

fn actor() -> TrustedActor {
    TrustedActor(Actor {
        principal: "owner".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![Scope::Control],
        authority: None,
    })
}
fn server() -> chat::Server {
    chat::Server {
        binding: Reference {
            key: key(Scope::Control, "chat_server", "local"),
            version: Version::State(1),
        },
        url: "http://127.0.0.1:8008".into(),
        server_name: "hctl2.localhost".into(),
        sender: "@hctl2_control:hctl2.localhost".into(),
    }
}
fn def(name: &str) -> Definition {
    Definition {
        name: name.into(),
        goal: "outcome".into(),
        scope: "repo".into(),
        roles: vec!["reviewer".into()],
        role_members: BTreeMap::from([("reviewer".into(), vec!["owner".into()])]),
        defaults: json!({}),
        settings: store::ProjectSettings {
            publish_review_requires_confirmation: true,
            selection_policy: json!({}),
        },
    }
}
struct Env {
    store: Store,
    root: PathBuf,
    rid: String,
    sid: String,
    a: String,
    b: String,
}
impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
impl Env {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "hctl-project-domain-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let mut store = Store::open(&root).unwrap();
        let request=serde_json::from_value(json!({"name":"fixture","origin":"external","platform":"github","instance":"github.com","platform_repo_id":"77","platform_path":"owner/fixture","default_source":"github_issues"})).unwrap();
        let reg = repo::admit(
            &mut store,
            &actor(),
            "repo",
            "repo",
            repo::prepare(request, None).unwrap(),
        )
        .unwrap();
        repo::begin_step(&mut store, &actor(), &reg.repo_id, "platform").unwrap();
        repo::confirm_platform(
            &mut store,
            &reg.repo_id,
            repo::PlatformObservation {
                instance: "github.com".into(),
                stable_id: "77".into(),
                full_name: "owner/fixture".into(),
                clone_url: "https://github.com/owner/fixture.git".into(),
                account_id: "5".into(),
                has_issues: true,
                can_write_issues: true,
                credential_ref: String::new(),
            },
        )
        .unwrap();
        let mut e = Self {
            store,
            root,
            rid: reg.repo_id,
            sid: String::new(),
            a: String::new(),
            b: String::new(),
        };
        for name in ["A", "B"] {
            let result = e
                .apply(
                    name,
                    Action::Create {
                        repo_id: e.rid.clone(),
                        definition: def(name),
                    },
                )
                .unwrap();
            let id = result["project_id"].as_str().unwrap().to_owned();
            let effect = result["effect_id"].as_str().unwrap();
            e.store.begin_effect(e.store.generation(), effect).unwrap();
            chat::confirm(
                &mut e.store,
                &actor(),
                effect,
                json!({"matrix_room_id":format!("!{name}:hctl2.localhost")}),
            )
            .unwrap();
            if name == "A" {
                e.a = id;
            } else {
                e.b = id;
            }
        }
        let input = task::Input {
            key: "connect".into(),
            action: task::Action::Connect {
                repo_id: e.rid.clone(),
                candidate_id: "github_issues".into(),
                consent: true,
                make_default: false,
            },
        };
        let plan = task::prepare(&e.store, input).unwrap();
        e.sid = task::admit(&mut e.store, &actor(), plan).unwrap()["source_id"]
            .as_str()
            .unwrap()
            .into();
        e
    }
    fn apply(&mut self, k: &str, a: Action) -> Result<Value> {
        let p = prepare(
            &self.store,
            Input {
                key: k.into(),
                action: a,
            },
            Some(server()),
            &actor(),
            task::now(),
        )?;
        admit(&mut self.store, &actor(), p)
    }
    fn seed(&mut self, mut r: Record) {
        if matches!(r.data, store::RecordData::Room { .. }) {
            r.revision_digest =
                foundation::canonical_json_sha256(&serde_json::to_value(&r.data).unwrap()).unwrap();
        }
        let a = task::owner(&actor(), [r.key.scope.clone()]).unwrap();
        let input = serde_json::to_value(&r).unwrap();
        let cmd = Command {
            command_id: format!("seed:{}:{}", r.key.id, r.version),
            idempotency_key: format!("seed:{}:{}", r.key.id, r.version),
            actor: a.0.clone(),
            target: r.key.clone(),
            expected: if r.version == 1 {
                Expected::Absent
            } else {
                Expected::Exact(Version::State(r.version - 1))
            },
            binding: reference(&r),
            operation: "fixture".into(),
            input_digest: Command::digest_input("fixture", &input).unwrap(),
            input,
        };
        self.store
            .submit(self.store.generation(), &a, &cmd, None, |tx| {
                tx.put(&r)?;
                Ok(json!({}))
            })
            .unwrap();
    }
    fn claim(&mut self, project: &str) -> String {
        let input = task::Input {
            key: format!("attach-{project}"),
            action: task::Action::Attach {
                project_id: project.into(),
                project_version: 1,
                source_id: self.sid.clone(),
                approved_scope: "77".into(),
                group: None,
                consent: true,
            },
        };
        let plan = task::prepare(&self.store, input).unwrap();
        task::admit(&mut self.store, &actor(), plan).unwrap();
        let (_, source) = task::source(&self.store, &self.rid, &self.sid).unwrap();
        task::observe(
            &mut self.store,
            &source,
            task::Snapshot {
                source: Reference {
                    key: task::key(Scope::Repo(self.rid.clone()), "task_source", &self.sid),
                    version: Version::State(1),
                },
                observed_at: task::now(),
                complete: true,
                error: None,
                cards: vec![task::Card {
                    entity: store::ExternalEntity {
                        provider: "github".into(),
                        account_stable_id: "5".into(),
                        external_entity_kind: "issue".into(),
                        immutable_external_entity_id: "node1".into(),
                    },
                    number: 1,
                    title: "card".into(),
                    body: "contract".into(),
                    stage: "open".into(),
                    remote_revision: "t0".into(),
                    content_version: None,
                    groups: vec![],
                    dependencies: task::Dependencies::default(),
                    comments: vec![],
                    raw: json!({}),
                    tombstone: false,
                }],
                stable_groups: vec![],
            },
        )
        .unwrap();
        let input = task::Input {
            key: format!("claim-{project}"),
            action: task::Action::Claim {
                project_id: project.into(),
                project_version: 1,
                source_id: self.sid.clone(),
                entity_id: "node1".into(),
            },
        };
        let plan = task::prepare(&self.store, input).unwrap();
        task::admit(&mut self.store, &actor(), plan).unwrap()["task_id"]
            .as_str()
            .unwrap()
            .into()
    }
    fn spec(&self, task_id: &str) -> RequestSpec {
        let (r, t) = task::task(&self.store, &self.a, task_id).unwrap();
        RequestSpec {
            question: "approve contract?".into(),
            target: Target::Human {
                principal: "owner".into(),
            },
            owner: reference(&r),
            affected_revision: None,
            blocking_scope: "contract_adoption".into(),
            owner_state_version: t.state_version,
            dedup_root: format!("task:{task_id}:contract"),
            permissions: json!({"action":"task.adopt"}),
            deadline: None,
            deadline_action: DeadlineAction::FailWaiting,
            action: RequiredAction::TaskAdopt,
            input_schema: "hctl2.task.Adoption.v1".into(),
        }
    }
    fn adoption(&self) -> task::Adoption {
        let contract = task::Contract {
            scope: "scope".into(),
            expected_outcome: "outcome".into(),
            acceptance: vec![task::Acceptance {
                text: "approved".into(),
                grade: task::Grade::Human,
            }],
            roles: vec![],
            capabilities: vec![],
        };
        task::Adoption {
            origin: task::ContractOrigin::Local {
                reference: reference(&project(&self.store, &self.a).unwrap()),
                proposal_digest: foundation::canonical_json_sha256(
                    &serde_json::to_value(&contract).unwrap(),
                )
                .unwrap(),
            },
            contract,
        }
    }
    fn create_request(&mut self, k: &str, spec: RequestSpec) -> Value {
        self.apply(
            k,
            Action::CreateRequest {
                project_id: self.a.clone(),
                project_version: project(&self.store, &self.a).unwrap().version,
                request: spec,
            },
        )
        .unwrap()
    }
}

#[test]
fn create_is_atomic_idempotent_and_same_repo_projects_are_independent() {
    let mut e = Env::new();
    assert_ne!(e.a, e.b);
    assert_ne!(
        chat::main_binding(&e.store, &e.a).unwrap().1.id,
        chat::main_binding(&e.store, &e.b).unwrap().1.id
    );
    let same = e
        .apply(
            "A",
            Action::Create {
                repo_id: e.rid.clone(),
                definition: def("A"),
            },
        )
        .unwrap();
    assert_eq!(same["project_id"], e.a);
    assert_eq!(e.store.list("project").unwrap().len(), 2);
    assert_eq!(e.store.list("room").unwrap().len(), 2);
    assert_eq!(
        e.apply(
            "A",
            Action::Create {
                repo_id: e.rid.clone(),
                definition: def("different")
            }
        )
        .unwrap_err()
        .code,
        "IDEMPOTENCY_CONFLICT"
    );
    let pending=repo::admit(&mut e.store,&actor(),"pending","pending",repo::prepare(serde_json::from_value(json!({"name":"pending","origin":"local","platform":"local","platform_path":"pending"})).unwrap(),None).unwrap()).unwrap();
    assert_eq!(
        e.apply(
            "no-project",
            Action::Create {
                repo_id: pending.repo_id,
                definition: def("pending")
            }
        )
        .unwrap_err()
        .code,
        "REPO_PENDING"
    );
    let before = e.store.list("project").unwrap().len();
    let mut plan = prepare(
        &e.store,
        Input {
            key: "rollback".into(),
            action: Action::Create {
                repo_id: e.rid.clone(),
                definition: def("rollback"),
            },
        },
        Some(server()),
        &actor(),
        task::now(),
    )
    .unwrap();
    plan.effects.push(plan.effects[0].clone()); // outbox conflict must roll back Project and Room too
    assert!(admit(&mut e.store, &actor(), plan).is_err());
    assert_eq!(e.store.list("project").unwrap().len(), before);
}

#[test]
fn archive_rechecks_repo_owned_blockers_and_restores_only_previously_active_rooms() {
    let mut e = Env::new();
    let open = value_record(
        key(Scope::Project(e.a.clone()), "room", "open-topic"),
        1,
        &json!({}),
    )
    .unwrap();
    let mut open = open;
    open.data = store::RecordData::Room {
        room_kind: store::RoomKind::Topic,
        state: RoomState::Active,
    };
    e.seed(open);
    let mut closed = e
        .store
        .get(&key(Scope::Project(e.a.clone()), "room", "open-topic"))
        .unwrap()
        .unwrap();
    closed.key.id = "closed-topic".into();
    closed.data = store::RecordData::Room {
        room_kind: store::RoomKind::Topic,
        state: RoomState::Archived,
    };
    e.seed(closed);
    let input = Input {
        key: "archive".into(),
        action: Action::Archive {
            project_id: e.a.clone(),
            version: 1,
        },
    };
    let plan = prepare(&e.store, input, None, &actor(), task::now()).unwrap();
    let mut lease = value_record(
        key(Scope::Repo(e.rid.clone()), "write_lease", "lease"),
        1,
        &json!({"state":"active","project_id":e.a}),
    )
    .unwrap();
    lease
        .sources
        .push(reference(&project(&e.store, &e.a).unwrap()));
    e.seed(lease.clone());
    assert_eq!(
        archive_blockers(&e.store, &e.a).unwrap()[0]
            .reference
            .key
            .kind,
        "write_lease"
    );
    assert_eq!(
        admit(&mut e.store, &actor(), plan).unwrap_err().code,
        "PROJECT_BUSY"
    );
    lease.version = 2;
    lease.data = store::RecordData::Value {
        value: json!({"state":"released","project_id":e.a}),
    };
    e.seed(lease);
    e.apply(
        "archive",
        Action::Archive {
            project_id: e.a.clone(),
            version: 1,
        },
    )
    .unwrap();
    assert!(readonly(&project(&e.store, &e.a).unwrap()));
    assert!(
        e.apply(
            "bad-update",
            Action::Update {
                project_id: e.a.clone(),
                version: 2,
                definition: def("bad")
            }
        )
        .is_err()
    );
    e.apply(
        "restore",
        Action::Restore {
            project_id: e.a.clone(),
            version: 2,
        },
    )
    .unwrap();
    assert!(matches!(
        e.store
            .get(&key(Scope::Project(e.a.clone()), "room", "open-topic"))
            .unwrap()
            .unwrap()
            .data,
        store::RecordData::Room {
            state: RoomState::Active,
            ..
        }
    ));
    assert!(matches!(
        e.store
            .get(&key(Scope::Project(e.a.clone()), "room", "closed-topic"))
            .unwrap()
            .unwrap()
            .data,
        store::RecordData::Room {
            state: RoomState::Archived,
            ..
        }
    ));
    assert_eq!(project(&e.store, &e.b).unwrap().version, 1);
}

#[test]
fn requests_deduplicate_supersede_and_reject_stale_or_unauthorized_answers() {
    let mut e = Env::new();
    let a = e.a.clone();
    let task = e.claim(&a);
    let spec = e.spec(&task);
    let first = e.create_request("ask", spec.clone());
    assert_eq!(
        e.create_request("ask-again", spec.clone())["request_id"],
        first["request_id"]
    );
    assert_eq!(requests(&e.store, &a).unwrap().len(), 1);
    let mut changed = spec;
    changed.blocking_scope = "new-contract".into();
    let newer = e.create_request("ask-new", changed.clone());
    assert_ne!(first["request_id"], newer["request_id"]);
    assert_eq!(
        requests(&e.store, &a)
            .unwrap()
            .iter()
            .filter(|(_, q)| q.state == RequestState::Superseded)
            .count(),
        1
    );
    assert_eq!(
        pending(&e.store, &a, &actor()).unwrap()["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let old = Action::ResolveRequest {
        project_id: a.clone(),
        request_id: first["request_id"].as_str().unwrap().into(),
        version: 1,
        adoption: e.adoption(),
    };
    assert!(e.apply("old-answer", old).is_err());
    let other = TrustedActor(Actor {
        principal: "other".into(),
        ..actor().0
    });
    assert!(
        pending(&e.store, &a, &other).unwrap()["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        prepare(
            &e.store,
            Input {
                key: "unauthorized".into(),
                action: Action::ResolveRequest {
                    project_id: a,
                    request_id: newer["request_id"].as_str().unwrap().into(),
                    version: 1,
                    adoption: e.adoption()
                }
            },
            None,
            &other,
            task::now()
        )
        .unwrap_err()
        .code,
        "PERMISSION_DENIED"
    );
}

#[test]
fn resolution_delivery_recovers_exactly_once_and_project_updates_do_not_rewrite_contracts() {
    let mut e = Env::new();
    let a = e.a.clone();
    let task = e.claim(&a);
    let spec = e.spec(&task);
    let req = e.create_request("ask", spec);
    let request_id = req["request_id"].as_str().unwrap().to_owned();
    let input = Input {
        key: "answer".into(),
        action: Action::ResolveRequest {
            project_id: a.clone(),
            request_id: request_id.clone(),
            version: 1,
            adoption: e.adoption(),
        },
    };
    let plan = prepare(&e.store, input.clone(), None, &actor(), task::now()).unwrap();
    let result = admit(&mut e.store, &actor(), plan).unwrap();
    assert!(
        task::task(&e.store, &a, &task)
            .unwrap()
            .1
            .revision
            .is_none()
    );
    let effect = result["effect_id"].as_str().unwrap();
    e.store.begin_effect(e.store.generation(), effect).unwrap();
    // Keep the same directory and durable signal; a new writer generation resumes it.
    let replacement = Store::open(&e.root.join("reopen-placeholder")).unwrap();
    let old = std::mem::replace(&mut e.store, replacement);
    drop(old);
    e.store = Store::open(&e.root).unwrap();
    let received = deliver(&mut e.store, &actor(), effect).unwrap();
    assert_eq!(received["delivery"], "confirmed");
    assert_eq!(deliver(&mut e.store, &actor(), effect).unwrap(), received);
    assert_eq!(e.store.list("task_revision").unwrap().len(), 1);
    assert_eq!(e.store.list("request_delivery_receipt").unwrap().len(), 1);
    assert!(
        pending(&e.store, &a, &actor()).unwrap()["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let (_, before) = task::task(&e.store, &a, &task).unwrap();
    let mut updated = def("new");
    updated.settings.publish_review_requires_confirmation = false;
    e.apply(
        "update",
        Action::Update {
            project_id: a.clone(),
            version: 1,
            definition: updated,
        },
    )
    .unwrap();
    let (_, after) = task::task(&e.store, &a, &task).unwrap();
    assert_eq!(
        serde_json::to_value(before.revision).unwrap(),
        serde_json::to_value(after.revision).unwrap()
    );
    assert!(
        definition(&e.store, &e.b)
            .unwrap()
            .settings
            .publish_review_requires_confirmation
    );
    assert_eq!(replay(&e.store, &input).unwrap().unwrap().result, result);
}

#[test]
fn deadline_and_reading_never_answer_or_complete_task_and_archived_requests_stay_open() {
    let mut e = Env::new();
    let a = e.a.clone();
    let task = e.claim(&a);
    let mut spec = e.spec(&task);
    spec.deadline = Some(task::now() + 100);
    let req = e.create_request("deadline", spec.clone());
    let snapshot = pending(&e.store, &a, &actor()).unwrap();
    assert_eq!(pending(&e.store, &a, &actor()).unwrap(), snapshot);
    assert_eq!(
        requests(&e.store, &a).unwrap()[0].1.state,
        RequestState::Open
    );
    e.apply(
        "archive",
        Action::Archive {
            project_id: a.clone(),
            version: 1,
        },
    )
    .unwrap();
    assert!(
        pending(&e.store, &a, &actor()).unwrap()["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        expire(&mut e.store, &actor(), &a, task::now() + 200).unwrap(),
        0
    );
    e.apply(
        "restore",
        Action::Restore {
            project_id: a.clone(),
            version: 2,
        },
    )
    .unwrap();
    assert_eq!(
        requests(&e.store, &a).unwrap()[0].1.state,
        RequestState::Open
    );
    assert_eq!(
        expire(&mut e.store, &actor(), &a, task::now() + 200).unwrap(),
        1
    );
    assert_eq!(
        expire(&mut e.store, &actor(), &a, task::now() + 200).unwrap(),
        0
    );
    assert_eq!(task::task(&e.store, &a, &task).unwrap().1.lifecycle, "open");
    assert!(
        e.apply(
            "late-answer",
            Action::ResolveRequest {
                project_id: a,
                request_id: req["request_id"].as_str().unwrap().into(),
                version: 1,
                adoption: e.adoption()
            }
        )
        .is_err()
    );
}

#[test]
fn archive_preview_catches_new_rooms_and_pending_external_effects() {
    let mut e = Env::new();
    let a = e.a.clone();
    let plan = prepare(
        &e.store,
        Input {
            key: "archive".into(),
            action: Action::Archive {
                project_id: a.clone(),
                version: 1,
            },
        },
        None,
        &actor(),
        task::now(),
    )
    .unwrap();
    let mut room = e
        .store
        .list("room")
        .unwrap()
        .into_iter()
        .find(|r| r.key.scope == Scope::Project(a.clone()))
        .unwrap();
    room.key.id = "new-topic".into();
    room.data = store::RecordData::Room {
        room_kind: store::RoomKind::Topic,
        state: RoomState::Active,
    };
    e.seed(room);
    assert_eq!(
        admit(&mut e.store, &actor(), plan).unwrap_err().code,
        "VERSION_CONFLICT"
    );
    for kind in [
        "run",
        "room_invocation",
        "input_lease",
        "integration_intent",
        "review_publish_intent",
    ] {
        e.seed(
            value_record(
                key(Scope::Project(a.clone()), kind, kind),
                1,
                &json!({"state":"unknown"}),
            )
            .unwrap(),
        );
    }
    assert_eq!(archive_blockers(&e.store, &a).unwrap().len(), 5);
}

#[test]
fn roster_is_independent_immutable_and_does_not_revise_external_binding() {
    let mut e = Env::new();
    let a = e.a.clone();
    let (binding, room) = chat::main_binding(&e.store, &a).unwrap();
    let r = |kind: &str| Reference {
        key: key(Scope::Control, kind, kind),
        version: Version::State(1),
    };
    let selection = Selection {
        room_id: room.id.clone(),
        selected_item: r("profession_instance"),
        profession: r("profession"),
        profession_digest: "a".repeat(64),
        agency: r("agency"),
        required_skills: vec![Skill {
            reference: r("skill"),
            digest: None,
        }],
        optional_skills: vec![],
        worker_profiles: vec![r("worker_profile")],
        responsibility: "reviewer".into(),
        permission: json!({}),
        budget: json!({}),
        display_name: "reviewer".into(),
        persona_tags: vec![],
    };
    let first = e
        .apply(
            "select",
            Action::Select {
                project_id: a.clone(),
                project_version: 1,
                room_id: room.id.clone(),
                topic_command_key: None,
                roster_version: None,
                selections: vec![selection.clone()],
            },
        )
        .unwrap();
    let selected: Vec<Reference> = serde_json::from_value(first["selections"].clone()).unwrap();
    let frozen = chat::at(&e.store, &selected[0]).unwrap();
    assert_eq!(
        chat::room(&e.store, &a, &room.id).unwrap().1.participants,
        selected
    );
    assert!(
        chat::main_binding(&e.store, &e.b)
            .unwrap()
            .1
            .participants
            .is_empty()
    );
    e.apply(
        "remove",
        Action::Select {
            project_id: a.clone(),
            project_version: 1,
            room_id: room.id.clone(),
            topic_command_key: None,
            roster_version: Some(1),
            selections: vec![],
        },
    )
    .unwrap();
    assert!(
        chat::room(&e.store, &a, &room.id)
            .unwrap()
            .1
            .participants
            .is_empty()
    );
    assert_eq!(
        chat::at(&e.store, &selected[0]).unwrap().revision_digest,
        frozen.revision_digest
    );
    assert_eq!(
        chat::main_binding(&e.store, &a).unwrap().0.version,
        binding.version
    );
    assert!(
        e.apply(
            "stale-select",
            Action::Select {
                project_id: a.clone(),
                project_version: 1,
                room_id: room.id.clone(),
                topic_command_key: None,
                roster_version: Some(1),
                selections: vec![selection]
            }
        )
        .is_err()
    );
    assert!(
        e.apply(
            "unscoped",
            Action::Select {
                project_id: a,
                project_version: 1,
                room_id: "arbitrary".into(),
                topic_command_key: Some("topic-key".into()),
                roster_version: None,
                selections: vec![]
            }
        )
        .is_err()
    );
}

#[test]
fn idle_request_topic_attention_is_not_pending_and_discussion_does_not_answer() {
    let mut e = Env::new();
    let a = e.a.clone();
    let task = e.claim(&a);
    let mut spec = e.spec(&task);
    spec.target = Target::Role {
        role: "reviewer".into(),
    };
    let req = e.create_request("ask", spec);
    let request_id = req["request_id"].as_str().unwrap();
    let r = required(
        &e.store,
        &key(Scope::Project(a.clone()), "request", request_id),
    )
    .unwrap();
    let request: Request = decode(&r).unwrap();
    let texts = chat::request_texts(&e.store, &reference(&r), &request.blockers).unwrap();
    let sources = texts.iter().map(|t| t.source.clone()).collect();
    let input = chat::Input {
        key: "request-topic".into(),
        action: chat::Action::CreateTopic {
            project_id: a.clone(),
            project_version: 1,
            name: "discussion".into(),
            origin: Box::new(chat::Origin::Request {
                request: reference(&r),
                blockers: request.blockers.clone(),
            }),
            brief: Box::new(chat::Brief {
                context_and_goal: "clarify".into(),
                settled_facts_and_reasons: vec![],
                disagreements_and_questions: vec![],
                constraints_and_materials: vec![],
                sources,
            }),
            participants: vec![],
            roster_confirmed: true,
        },
    };
    let plan = chat::prepare(&e.store, input, texts).unwrap();
    let topic = chat::admit(&mut e.store, &actor(), plan).unwrap();
    let effect = topic["effect_id"].as_str().unwrap();
    e.store.begin_effect(e.store.generation(), effect).unwrap();
    chat::confirm(
        &mut e.store,
        &actor(),
        effect,
        json!({"matrix_room_id":"!topic:hctl2.localhost"}),
    )
    .unwrap();
    let room = topic["room_id"].as_str().unwrap().to_owned();
    let before = pending(&e.store, &a, &actor()).unwrap();
    let map = BTreeMap::from([(room.clone(), Some(task::now()))]);
    assert_eq!(
        attention(&e.store, &a, task::now() + 15 * 86400, &map)
            .unwrap()
            .len(),
        1
    );
    assert!(
        attention(&e.store, &a, task::now() + 13 * 86400, &map)
            .unwrap()
            .is_empty()
    );
    assert!(
        attention(
            &e.store,
            &a,
            task::now() + 15 * 86400,
            &BTreeMap::from([(room.clone(), None)])
        )
        .unwrap()
        .is_empty()
    );
    assert_eq!(pending(&e.store, &a, &actor()).unwrap(), before);
    let binding = chat::room(&e.store, &a, &room).unwrap().0;
    let plan = chat::prepare(
        &e.store,
        chat::Input {
            key: "close-request-topic".into(),
            action: chat::Action::Close {
                project_id: a.clone(),
                room_id: room,
                version: binding.version,
            },
        },
        vec![],
    )
    .unwrap();
    chat::admit(&mut e.store, &actor(), plan).unwrap();
    assert_eq!(
        requests(&e.store, &a).unwrap()[0].1.state,
        RequestState::Open
    );
    assert_eq!(pending(&e.store, &a, &actor()).unwrap(), before);
}

#[test]
fn source_change_before_submit_or_receipt_never_advances_new_blocker() {
    let mut e = Env::new();
    let a = e.a.clone();
    let task = e.claim(&a);
    let spec = e.spec(&task);
    let req = e.create_request("ask", spec);
    let action = Action::ResolveRequest {
        project_id: a.clone(),
        request_id: req["request_id"].as_str().unwrap().into(),
        version: 1,
        adoption: e.adoption(),
    };
    let plan = prepare(
        &e.store,
        Input {
            key: "answer".into(),
            action,
        },
        None,
        &actor(),
        task::now(),
    )
    .unwrap();
    let (r, mut t) = task::task(&e.store, &a, &task).unwrap();
    t.state_version += 1;
    e.seed(value_record(r.key, r.version + 1, &t).unwrap());
    assert_eq!(
        admit(&mut e.store, &actor(), plan).unwrap_err().code,
        "VERSION_CONFLICT"
    );
    assert_eq!(
        requests(&e.store, &a).unwrap()[0].1.state,
        RequestState::Open
    );
    let new = e.create_request("new-source", e.spec(&task));
    let result = e
        .apply(
            "new-answer",
            Action::ResolveRequest {
                project_id: a.clone(),
                request_id: new["request_id"].as_str().unwrap().into(),
                version: 1,
                adoption: e.adoption(),
            },
        )
        .unwrap();
    let (r, mut t) = task::task(&e.store, &a, &task).unwrap();
    t.state_version += 1;
    e.seed(value_record(r.key, r.version + 1, &t).unwrap());
    assert_eq!(
        deliver(
            &mut e.store,
            &actor(),
            result["effect_id"].as_str().unwrap()
        )
        .unwrap_err()
        .code,
        "VERSION_CONFLICT"
    );
    assert!(
        task::task(&e.store, &a, &task)
            .unwrap()
            .1
            .revision
            .is_none()
    );
    assert!(e.store.list("request_delivery_receipt").unwrap().is_empty());
}

#[test]
fn a_request_can_supply_its_own_frozen_answer_source_and_deliver_after_default_change() {
    let mut e = Env::new();
    let a = e.a.clone();
    let task = e.claim(&a);
    let req = e.create_request("ask", e.spec(&task));
    let id = req["request_id"].as_str().unwrap().to_owned();
    let mut adoption = e.adoption();
    let task::ContractOrigin::Local {
        reference: origin, ..
    } = &mut adoption.origin
    else {
        panic!()
    };
    *origin =
        reference(&required(&e.store, &key(Scope::Project(a.clone()), "request", &id)).unwrap());
    let result = e
        .apply(
            "answer",
            Action::ResolveRequest {
                project_id: a.clone(),
                request_id: id,
                version: 1,
                adoption,
            },
        )
        .unwrap();
    let mut d = def("changed");
    d.settings.publish_review_requires_confirmation = false;
    e.apply(
        "update",
        Action::Update {
            project_id: a.clone(),
            version: 1,
            definition: d,
        },
    )
    .unwrap();
    deliver(
        &mut e.store,
        &actor(),
        result["effect_id"].as_str().unwrap(),
    )
    .unwrap();
    let rev = e.store.list("task_revision").unwrap().pop().unwrap();
    assert_eq!(rev.sources[0].version, Version::State(1));
    assert_eq!(
        task::task(&e.store, &a, &task)
            .unwrap()
            .1
            .revision
            .unwrap()
            .number,
        1
    );
}

#[test]
fn request_blocks_old_task_preview_and_pending_deduplicates_the_source_action() {
    let mut e = Env::new();
    let a = e.a.clone();
    let id = e.claim(&a);
    let (r, mut task) = task::task(&e.store, &a, &id).unwrap();
    task.pending_contract = Some(reference(&project(&e.store, &a).unwrap()));
    e.seed(value_record(r.key, r.version + 1, &task).unwrap());
    let (source, _) = task::task(&e.store, &a, &id).unwrap();
    let input = task::Input {
        key: "old-adoption-preview".into(),
        action: task::Action::Adopt {
            project_id: a.clone(),
            project_version: 1,
            task_id: id.clone(),
            version: source.version,
            adoption: e.adoption(),
        },
    };
    let old_preview = task::prepare(&e.store, input).unwrap();
    let req = e.create_request("ask", e.spec(&id));
    assert_eq!(
        task::admit(&mut e.store, &actor(), old_preview)
            .unwrap_err()
            .code,
        "REQUEST_REQUIRED"
    );
    assert_eq!(
        pending(&e.store, &a, &actor()).unwrap()["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let other = TrustedActor(Actor {
        principal: "other".into(),
        ..actor().0
    });
    assert!(
        pending(&e.store, &a, &other).unwrap()["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let mut duplicate = e.spec(&id);
    duplicate.dedup_root = "different-root-same-input".into();
    assert_eq!(
        e.apply(
            "duplicate",
            Action::CreateRequest {
                project_id: a.clone(),
                project_version: 1,
                request: duplicate
            }
        )
        .unwrap_err()
        .code,
        "REQUEST_BUSY"
    );
    let mut unsupported = e.spec(&id);
    unsupported.permissions = json!({"action":"task.complete"});
    assert!(
        e.apply(
            "wrong-permission",
            Action::CreateRequest {
                project_id: a.clone(),
                project_version: 1,
                request: unsupported
            }
        )
        .is_err()
    );
    let result = e
        .apply(
            "answer",
            Action::ResolveRequest {
                project_id: a.clone(),
                request_id: req["request_id"].as_str().unwrap().into(),
                version: 1,
                adoption: e.adoption(),
            },
        )
        .unwrap();
    assert_eq!(
        task::request_blockers(&e.store, &a, &id).unwrap()[0]
            .1
            .delivery
            .as_deref(),
        result["effect_id"].as_str()
    );
    deliver(
        &mut e.store,
        &actor(),
        result["effect_id"].as_str().unwrap(),
    )
    .unwrap();
    assert!(
        pending(&e.store, &a, &actor()).unwrap()["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        !task::request_blockers(&e.store, &a, &id).unwrap()[0]
            .1
            .waiting
    );
}

#[test]
fn stale_deadline_does_not_starve_a_current_request_and_members_are_explicit() {
    let mut e = Env::new();
    let a = e.a.clone();
    let id = e.claim(&a);
    let mut spec = e.spec(&id);
    spec.deadline = Some(task::now() + 100);
    let old = e.create_request("old-deadline", spec);
    let (r, mut task) = task::task(&e.store, &a, &id).unwrap();
    task.state_version += 1;
    e.seed(value_record(r.key, r.version + 1, &task).unwrap());
    let mut current = e.spec(&id);
    current.dedup_root = "new-source-root".into();
    current.deadline = Some(task::now() + 100);
    let new = e.create_request("current-deadline", current);
    assert_eq!(
        expire(&mut e.store, &actor(), &a, task::now() + 101).unwrap(),
        1
    );
    let qs = requests(&e.store, &a).unwrap();
    assert_eq!(
        qs.iter()
            .find(|(r, _)| r.key.id == old["request_id"])
            .unwrap()
            .1
            .state,
        RequestState::Open
    );
    assert_eq!(
        qs.iter()
            .find(|(r, _)| r.key.id == new["request_id"])
            .unwrap()
            .1
            .state,
        RequestState::Expired
    );
    let rooms = e.store.list("room_binding").unwrap();
    let own = reference(
        rooms
            .iter()
            .find(|r| r.key.scope == Scope::Project(a.clone()))
            .unwrap(),
    );
    let foreign = reference(
        rooms
            .iter()
            .find(|r| r.key.scope == Scope::Project(e.b.clone()))
            .unwrap(),
    );
    assert!(
        e.apply(
            "cross-project-members",
            Action::Members {
                project_id: a.clone(),
                project_version: 1,
                rooms: vec![own.clone(), foreign],
                users: vec!["@user:hctl2.localhost".into()],
                invite: true
            }
        )
        .is_err()
    );
    assert!(
        e.apply(
            "duplicate-members",
            Action::Members {
                project_id: a.clone(),
                project_version: 1,
                rooms: vec![own.clone(), own.clone()],
                users: vec!["@user:hctl2.localhost".into()],
                invite: true
            }
        )
        .is_err()
    );
    let result = e
        .apply(
            "members",
            Action::Members {
                project_id: a,
                project_version: 1,
                rooms: vec![own],
                users: vec!["@user:hctl2.localhost".into()],
                invite: true,
            },
        )
        .unwrap();
    assert_eq!(result["effect_ids"].as_array().unwrap().len(), 1);
    let (intent, _) = e
        .store
        .effect(result["effect_ids"][0].as_str().unwrap())
        .unwrap();
    assert_eq!(intent.operation, "chat.members");
}

#[test]
fn overview_recent_admitted_activity_is_scoped_ordered_and_readonly() {
    let mut e = Env::new();
    let a = e.a.clone();
    e.claim(&a);
    let stamp = e.store.read_stamp();
    let view = overview(&e.store, &a, &actor()).unwrap();
    assert_eq!(view["counts"]["tasks"], 1);
    let items = view["recent_activity"].as_array().unwrap();
    assert!(!items.is_empty());
    for item in items {
        let source: Reference = serde_json::from_value(item["source"].clone()).unwrap();
        assert_eq!(source.key.scope, Scope::Project(a.clone()));
        chat::at(&e.store, &source).unwrap();
    }
    assert!(
        items
            .windows(2)
            .all(|w| w[0]["sequence"].as_i64().unwrap() > w[1]["sequence"].as_i64().unwrap())
    );
    assert_eq!(e.store.read_stamp(), stamp);
    assert_eq!(overview(&e.store, &a, &actor()).unwrap(), view);
}
