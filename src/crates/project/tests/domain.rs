use agency_proto::{
    Capabilities, Catalog, EvidenceLevel, FrozenRef, Profession, SkillClaim, SkillVerification,
    hash,
};
mod invocation;
mod memo;
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

fn accepted_selection(
    e: &mut Env,
    room: &str,
    name: &str,
    available: bool,
    verification: Option<SkillVerification>,
) -> Selection {
    let harness = FrozenRef {
        id: "script".into(),
        revision: "1".into(),
        digest: hash(b"script"),
    };
    let skill = SkillClaim {
        reference: FrozenRef {
            id: "method".into(),
            revision: "1".into(),
            digest: hash(b"skill"),
        },
        required: true,
        verification,
    };
    let profession = Profession {
        reference: FrozenRef {
            id: name.into(),
            revision: "1".into(),
            digest: hash(name.as_bytes()),
        },
        harness: harness.clone(),
        model: "fixture".into(),
        persona: "researcher".into(),
        terms: "read only".into(),
        default_role: "planner".into(),
        skills: vec![skill.clone()],
        capabilities: Capabilities::default(),
    };
    let binding = participant::accept_binding(
        &mut e.store,
        &actor(),
        &format!("pair-{name}"),
        participant::Binding {
            id: name.into(),
            protocol: agency_proto::PROTOCOL.into(),
            catalog: Catalog {
                professions: vec![profession.clone()],
                harnesses: vec![harness.clone()],
                skills: if available {
                    vec![skill.clone()]
                } else {
                    vec![]
                },
            },
        },
    )
    .unwrap();
    let accepted = participant::accept_profession(
        &mut e.store,
        &actor(),
        &format!("accept-{name}"),
        name,
        &profession,
    )
    .unwrap();
    let profile = participant::profiles::WorkerProfile {
        harness,
        model: "fixture".into(),
        mode: "read_only".into(),
        permissions: vec!["context.read".into()],
        environment: vec![],
        required_capabilities: Capabilities::default(),
        max_context_bytes: 65536,
    };
    let plan = participant::profiles::prepare_profile(
        &e.store,
        participant::profiles::ProfileInput {
            key: format!("profile-{name}"),
            action: participant::profiles::ProfileAction::Create {
                id: name.into(),
                profile,
            },
        },
        &actor(),
    )
    .unwrap();
    let result = participant::profiles::admit_profile(&mut e.store, &actor(), plan).unwrap();
    Selection {
        room_id: room.into(),
        selected_item: reference(&accepted),
        profession: reference(&accepted),
        profession_digest: profession.reference.digest,
        agency: reference(&binding),
        required_skills: vec![Skill {
            reference: Reference {
                key: key(Scope::Control, "skill", &skill.reference.id),
                version: Version::Revision(skill.reference.digest),
            },
            digest: None,
        }],
        optional_skills: vec![],
        worker_profiles: vec![serde_json::from_value(result["revision"].clone()).unwrap()],
        responsibility: "research".into(),
        permission: json!({"allow":["context.read"]}),
        budget: json!({"max_bytes":65536}),
        display_name: "researcher".into(),
        persona_tags: vec![],
    }
}
fn select_action(project: &str, room: &str, selections: Vec<Selection>) -> Action {
    Action::Select {
        project_id: project.into(),
        project_version: 1,
        room_id: room.into(),
        topic_command_key: None,
        roster_version: None,
        selections,
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
    fn create_topic(
        &mut self,
        k: &str,
        origin: chat::Origin,
        texts: Vec<chat::SourceText>,
    ) -> String {
        let input = chat::Input {
            key: k.into(),
            action: chat::Action::CreateTopic {
                project_id: self.a.clone(),
                project_version: project(&self.store, &self.a).unwrap().version,
                name: k.into(),
                origin: Box::new(origin),
                brief: Box::new(chat::Brief {
                    context_and_goal: "clarify".into(),
                    settled_facts_and_reasons: vec![],
                    disagreements_and_questions: vec![],
                    constraints_and_materials: vec![],
                    sources: texts.iter().map(|t| t.source.clone()).collect(),
                }),
                participants: vec![],
                roster_confirmed: true,
                invites: None,
            },
        };
        let plan = chat::prepare(&self.store, input, texts, vec![]).unwrap();
        let result = chat::admit(&mut self.store, &actor(), plan).unwrap();
        let effect = result["effect_id"].as_str().unwrap();
        self.store
            .begin_effect(self.store.generation(), effect)
            .unwrap();
        chat::confirm(
            &mut self.store,
            &actor(),
            effect,
            json!({"matrix_room_id":format!("!{k}:hctl2.localhost")}),
        )
        .unwrap();
        result["room_id"].as_str().unwrap().into()
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
fn roster_rejects_unaccepted_candidates_instead_of_storing_placeholder_refs() {
    let mut e = Env::new();
    let a = e.a.clone();
    let (_, room) = chat::main_binding(&e.store, &a).unwrap();
    let r = |kind: &str| Reference {
        key: key(Scope::Control, kind, kind),
        version: Version::State(1),
    };
    let result = e.apply(
        "unaccepted-candidate",
        Action::Select {
            project_id: a,
            project_version: 1,
            room_id: room.id.clone(),
            topic_command_key: None,
            roster_version: None,
            selections: vec![Selection {
                room_id: room.id,
                selected_item: r("profession"),
                profession: r("profession"),
                profession_digest: "a".repeat(64),
                agency: r("agency_binding"),
                required_skills: vec![],
                optional_skills: vec![],
                worker_profiles: vec![r("worker_profile_revision")],
                responsibility: "research".into(),
                permission: json!({"allow": ["context.read"]}),
                budget: json!({"max_bytes": 65536}),
                display_name: "researcher".into(),
                persona_tags: vec![],
            }],
        },
    );
    assert_eq!(result.unwrap_err().code, "CANDIDATE_NOT_ACCEPTED");
    assert!(e.store.list("room_roster").unwrap().is_empty());
}

#[test]
fn roster_rejects_profession_accepted_from_another_agency_even_with_identical_catalogs() {
    let mut e = Env::new();
    let project = e.a.clone();
    let room = chat::main_binding(&e.store, &project).unwrap().1.id;
    let original = accepted_selection(&mut e, &room, "agency-a", true, None);
    let accepted_a = e.store.get(&original.profession.key).unwrap().unwrap();
    let profession: Profession = participant::decode(&accepted_a).unwrap();
    let binding_a = e.store.get(&original.agency.key).unwrap().unwrap();
    let mut binding_b: participant::Binding = participant::decode(&binding_a).unwrap();
    // Keep every catalog field identical so only the acceptance source differs.
    binding_b.id = "agency-b".into();
    let binding_b =
        participant::accept_binding(&mut e.store, &actor(), "pair-b", binding_b).unwrap();
    let accepted_b =
        participant::accept_profession(&mut e.store, &actor(), "accept-b", "agency-b", &profession)
            .unwrap();
    assert_ne!(reference(&accepted_a), reference(&accepted_b));
    assert!(!accepted_a.sources.contains(&reference(&binding_b)));
    let before = serde_json::to_value(chat::room(&e.store, &project, &room).unwrap().0).unwrap();
    prepare(
        &e.store,
        Input {
            key: "matching-a-preview".into(),
            action: select_action(&project, &room, vec![original.clone()]),
        },
        None,
        &actor(),
        task::now(),
    )
    .unwrap();

    let mut mismatched = original;
    mismatched.agency = reference(&binding_b);
    assert_eq!(
        e.apply(
            "wrong-acceptance-source",
            select_action(&project, &room, vec![mismatched.clone()]),
        )
        .unwrap_err()
        .code,
        "PROFESSION_CHANGED"
    );
    assert!(e.store.list("room_roster").unwrap().is_empty());
    assert!(e.store.list("room_selection").unwrap().is_empty());
    assert_eq!(
        serde_json::to_value(chat::room(&e.store, &project, &room).unwrap().0).unwrap(),
        before
    );

    // The same candidate succeeds when it names B's own acceptance record.
    mismatched.profession = reference(&accepted_b);
    mismatched.selected_item = mismatched.profession.clone();
    e.apply(
        "matching-b",
        select_action(&project, &room, vec![mismatched]),
    )
    .unwrap();
    assert_eq!(e.store.list("room_roster").unwrap().len(), 1);
    assert_eq!(e.store.list("room_selection").unwrap().len(), 1);
}

#[test]
fn roster_is_independent_immutable_and_does_not_revise_external_binding() {
    let mut e = Env::new();
    let a = e.a.clone();
    let (binding, room) = chat::main_binding(&e.store, &a).unwrap();
    let selection = accepted_selection(&mut e, &room.id, "planner", true, None);
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
            invites: None,
        },
    };
    let plan = chat::prepare(&e.store, input, texts, vec![]).unwrap();
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
fn required_skill_missing_or_mismatched_readback_is_not_accepted() {
    let mut e = Env::new();
    let a = e.a.clone();
    let room = chat::main_binding(&e.store, &a).unwrap().1.id;
    let missing = accepted_selection(&mut e, &room, "missing-skill", false, None);
    assert_eq!(
        e.apply("missing", select_action(&a, &room, vec![missing]))
            .unwrap_err()
            .code,
        "SKILL_MISSING"
    );
    let report = SkillVerification {
        source: EvidenceLevel::Unmediated,
        report: FrozenRef {
            id: "verify".into(),
            revision: "1".into(),
            digest: hash(b"report"),
        },
        readback_digest: hash(b"other skill"),
    };
    let mismatch = accepted_selection(&mut e, &room, "bad-readback", true, Some(report));
    // Even digest=None (unknown) cannot hide a verification mismatch.
    assert_eq!(
        e.apply("mismatch", select_action(&a, &room, vec![mismatch]))
            .unwrap_err()
            .code,
        "SKILL_DIGEST_MISMATCH"
    );
    assert!(e.store.list("room_roster").unwrap().is_empty());
}

#[test]
fn unknown_skill_cannot_be_upgraded_by_user_input_or_narrated_report() {
    let mut e = Env::new();
    let a = e.a.clone();
    let room = chat::main_binding(&e.store, &a).unwrap().1.id;
    let mut unknown = accepted_selection(&mut e, &room, "unknown", true, None);
    unknown.required_skills[0].digest = Some(hash(b"skill"));
    assert_eq!(
        e.apply("invent-known", select_action(&a, &room, vec![unknown]))
            .unwrap_err()
            .code,
        "SKILL_UNKNOWN"
    );
    let report = SkillVerification {
        source: EvidenceLevel::Narrated,
        report: FrozenRef {
            id: "verify".into(),
            revision: "1".into(),
            digest: hash(b"report"),
        },
        readback_digest: hash(b"skill"),
    };
    let mut narrated = accepted_selection(&mut e, &room, "narrated", true, Some(report));
    narrated.required_skills[0].digest = Some(hash(b"skill"));
    assert_eq!(
        e.apply("narrated-known", select_action(&a, &room, vec![narrated]))
            .unwrap_err()
            .code,
        "SKILL_UNKNOWN"
    );
    assert!(e.store.list("room_roster").unwrap().is_empty());
}

#[test]
fn verified_skill_and_missing_optional_skill_can_be_selected_without_inventing_proof() {
    let mut e = Env::new();
    let a = e.a.clone();
    let room = chat::main_binding(&e.store, &a).unwrap().1.id;
    let report = SkillVerification {
        source: EvidenceLevel::Unmediated,
        report: FrozenRef {
            id: "verify".into(),
            revision: "1".into(),
            digest: hash(b"report"),
        },
        readback_digest: hash(b"skill"),
    };
    let mut selected = accepted_selection(&mut e, &room, "verified", true, Some(report));
    selected.required_skills[0].digest = Some(hash(b"skill"));
    selected.optional_skills.push(Skill {
        reference: Reference {
            key: key(Scope::Control, "skill", "optional"),
            version: Version::Revision(hash(b"optional")),
        },
        digest: None,
    });
    let result = e
        .apply("select-verified", select_action(&a, &room, vec![selected]))
        .unwrap();
    assert_eq!(
        result["optional_skill_degradations"][0]["selection_index"],
        0
    );
    assert_eq!(
        result["optional_skill_degradations"][0]["reference"]["key"]["id"],
        "optional"
    );
    let record =
        participant::selection::resolve_room_candidate(&e.store, &a, &room, "research").unwrap();
    let selected: Selection = participant::decode(&record).unwrap();
    assert_eq!(selected.required_skills[0].digest, Some(hash(b"skill")));
    assert_eq!(selected.optional_skills[0].digest, None);
}

#[test]
fn changed_profession_wrong_profile_and_capability_shortfall_reject_before_roster_write() {
    let mut e = Env::new();
    let a = e.a.clone();
    let room = chat::main_binding(&e.store, &a).unwrap().1.id;
    let original = accepted_selection(&mut e, &room, "profile-check", true, None);
    let mut changed = original.clone();
    changed.profession_digest = hash(b"other terms");
    assert_eq!(
        e.apply("wrong-terms", select_action(&a, &room, vec![changed]))
            .unwrap_err()
            .code,
        "PROFESSION_CHANGED"
    );
    for (name, model, required, code) in [
        (
            "model",
            "other",
            Capabilities::default(),
            "PROFILE_MISMATCH",
        ),
        (
            "capability",
            "fixture",
            Capabilities {
                exact_attach: true,
                ..Capabilities::default()
            },
            "CAPABILITY_MISSING",
        ),
    ] {
        let mut selected = original.clone();
        let mut profile = participant::profiles::profile_at(&e.store, &selected.worker_profiles[0])
            .unwrap()
            .1;
        profile.model = model.into();
        profile.required_capabilities = required;
        let p = participant::profiles::prepare_profile(
            &e.store,
            participant::profiles::ProfileInput {
                key: name.into(),
                action: participant::profiles::ProfileAction::Create {
                    id: name.into(),
                    profile,
                },
            },
            &actor(),
        )
        .unwrap();
        let result = participant::profiles::admit_profile(&mut e.store, &actor(), p).unwrap();
        selected.worker_profiles =
            vec![serde_json::from_value(result["revision"].clone()).unwrap()];
        assert_eq!(
            e.apply(name, select_action(&a, &room, vec![selected]))
                .unwrap_err()
                .code,
            code
        );
    }
    assert!(e.store.list("room_roster").unwrap().is_empty());
}

#[test]
fn project_policy_checks_agency_profession_permissions_and_budget() {
    for (name, policy, code) in [
        (
            "agency",
            json!({"allowed_agencies":[]}),
            "SELECTION_POLICY_DENIED",
        ),
        (
            "profession",
            json!({"allowed_professions":[]}),
            "SELECTION_POLICY_DENIED",
        ),
        (
            "permission",
            json!({"permissions":[]}),
            "SELECTION_POLICY_DENIED",
        ),
        (
            "budget",
            json!({"max_context_bytes":1}),
            "SELECTION_POLICY_DENIED",
        ),
    ] {
        let mut e = Env::new();
        let a = e.a.clone();
        let room = chat::main_binding(&e.store, &a).unwrap().1.id;
        let selected = accepted_selection(&mut e, &room, name, true, None);
        let mut project =
            required(&e.store, &key(Scope::Project(a.clone()), "project", &a)).unwrap();
        if let store::RecordData::Project { settings, .. } = &mut project.data {
            settings.selection_policy = policy;
        }
        project.version += 1;
        project.revision_digest =
            foundation::canonical_json_sha256(&serde_json::to_value(&project.data).unwrap())
                .unwrap();
        e.seed(project);
        let mut action = select_action(&a, &room, vec![selected]);
        if let Action::Select {
            project_version, ..
        } = &mut action
        {
            *project_version = 2;
        }
        assert_eq!(e.apply(name, action).unwrap_err().code, code, "{name}");
        assert!(e.store.list("room_roster").unwrap().is_empty());
    }
}

#[test]
fn unknown_selection_policy_is_rejected_at_project_create_and_update() {
    let mut e = Env::new();
    let mut definition = def("invalid-policy");
    definition.settings.selection_policy = json!({"allow_all_typo":true});
    for (name, action) in [
        (
            "bad-create-policy",
            Action::Create {
                repo_id: e.rid.clone(),
                definition: definition.clone(),
            },
        ),
        (
            "bad-update-policy",
            Action::Update {
                project_id: e.a.clone(),
                version: 1,
                definition,
            },
        ),
    ] {
        assert_eq!(e.apply(name, action).unwrap_err().code, "INVALID_INPUT");
    }
    assert_eq!(e.store.list("project").unwrap().len(), 2);
    assert_eq!(project(&e.store, &e.a).unwrap().version, 1);
}

#[test]
fn shared_roster_validation_rejects_empty_profiles_and_non_state_candidate_refs() {
    let mut e = Env::new();
    let a = e.a.clone();
    let room = chat::main_binding(&e.store, &a).unwrap().1.id;
    let original = accepted_selection(&mut e, &room, "typed-reference", true, None);
    let project = project(&e.store, &a).unwrap();
    let mut missing = original.clone();
    missing.worker_profiles.clear();
    let error = participant::selection::validate_roster(&e.store, &project, &[missing])
        .err()
        .expect("empty Profile candidates rejected by shared validator");
    assert_eq!(error.code, "INVALID_INPUT");
    for field in ["agency", "profession"] {
        let mut bad = original.clone();
        let target = if field == "agency" {
            &mut bad.agency
        } else {
            &mut bad.profession
        };
        target.version = Version::Revision(hash(b"non-state-reference"));
        if field == "profession" {
            bad.selected_item = bad.profession.clone();
        }
        let error = participant::selection::validate_roster(&e.store, &project, &[bad])
            .err()
            .expect("candidate references require State versions");
        assert_eq!(error.code, "INVALID_INPUT", "{field}");
    }
}

#[test]
fn malformed_selection_is_rejected_before_catalog_lookup() {
    let mut e = Env::new();
    let a = e.a.clone();
    let room = chat::main_binding(&e.store, &a).unwrap().1.id;
    let mut selected = accepted_selection(&mut e, &room, "structural-first", true, None);
    selected.room_id = "another-room".into();
    selected.agency.key.id = "never-accepted".into();
    assert_eq!(
        e.apply(
            "malformed-selection",
            select_action(&a, &room, vec![selected])
        )
        .unwrap_err()
        .code,
        "INVALID_INPUT"
    );
}

#[test]
fn new_selection_sources_include_exact_profile_revision() {
    let mut e = Env::new();
    let a = e.a.clone();
    let room = chat::main_binding(&e.store, &a).unwrap().1.id;
    let selected = accepted_selection(&mut e, &room, "source-profile", true, None);
    let frozen_profile = selected.worker_profiles[0].clone();
    let result = e
        .apply(
            "select-with-source",
            select_action(&a, &room, vec![selected]),
        )
        .unwrap();
    let refs: Vec<Reference> = serde_json::from_value(result["selections"].clone()).unwrap();
    let record = required(&e.store, &refs[0].key).unwrap();
    assert!(record.sources.contains(&frozen_profile));
}

#[test]
fn mention_resolution_uses_exact_room_records_not_names_and_rejects_ambiguous_tags() {
    let mut e = Env::new();
    let a = e.a.clone();
    let room = chat::main_binding(&e.store, &a).unwrap().1.id;
    let one = accepted_selection(&mut e, &room, "one", true, None);
    let two = accepted_selection(&mut e, &room, "two", true, None);
    let result = e
        .apply("two-candidates", select_action(&a, &room, vec![one, two]))
        .unwrap();
    let refs: Vec<Reference> = serde_json::from_value(result["selections"].clone()).unwrap();
    assert_eq!(
        participant::selection::resolve_room_candidate(&e.store, &a, &room, "research")
            .unwrap_err()
            .code,
        "CANDIDATE_AMBIGUOUS"
    );
    assert_eq!(
        participant::selection::resolve_room_candidate(&e.store, &a, &room, "researcher")
            .unwrap_err()
            .code,
        "CANDIDATE_NOT_FOUND"
    );
    assert_eq!(
        participant::selection::resolve_room_candidate(&e.store, &a, &room, &refs[0].key.id)
            .unwrap()
            .key,
        refs[0].key
    );
    let other = chat::main_binding(&e.store, &e.b).unwrap().1.id;
    assert_eq!(
        participant::selection::resolve_room_candidate(&e.store, &e.b, &other, &refs[0].key.id)
            .unwrap_err()
            .code,
        "CANDIDATE_NOT_FOUND"
    );
}

#[test]
fn profile_pointer_update_does_not_change_a_selection_preview_or_its_frozen_revision() {
    let mut e = Env::new();
    let a = e.a.clone();
    let room = chat::main_binding(&e.store, &a).unwrap().1.id;
    let selected = accepted_selection(&mut e, &room, "profile-update", true, None);
    let old = selected.worker_profiles[0].clone();
    let prepared = prepare(
        &e.store,
        Input {
            key: "select-old-profile".into(),
            action: select_action(&a, &room, vec![selected]),
        },
        None,
        &actor(),
        task::now(),
    )
    .unwrap();
    let mut profile = participant::profiles::profile_at(&e.store, &old).unwrap().1;
    profile.max_context_bytes = 1024;
    let p = participant::profiles::prepare_profile(
        &e.store,
        participant::profiles::ProfileInput {
            key: "update-profile".into(),
            action: participant::profiles::ProfileAction::Update {
                id: "profile-update".into(),
                version: 1,
                profile,
            },
        },
        &actor(),
    )
    .unwrap();
    participant::profiles::admit_profile(&mut e.store, &actor(), p).unwrap();
    admit(&mut e.store, &actor(), prepared).unwrap();
    let record =
        participant::selection::resolve_room_candidate(&e.store, &a, &room, "research").unwrap();
    let selected: Selection = participant::decode(&record).unwrap();
    assert_eq!(selected.worker_profiles, vec![old]);
}

#[test]
fn selection_preview_rejects_a_changed_project_policy_without_writing_a_roster() {
    let mut e = Env::new();
    let a = e.a.clone();
    let room = chat::main_binding(&e.store, &a).unwrap().1.id;
    let selected = accepted_selection(&mut e, &room, "policy-change", true, None);
    let plan = prepare(
        &e.store,
        Input {
            key: "before-policy-change".into(),
            action: select_action(&a, &room, vec![selected]),
        },
        None,
        &actor(),
        task::now(),
    )
    .unwrap();
    let mut definition = def("A");
    definition.settings.selection_policy = json!({"allowed_agencies":[]});
    e.apply(
        "change-policy",
        Action::Update {
            project_id: a,
            version: 1,
            definition,
        },
    )
    .unwrap();
    assert_eq!(
        admit(&mut e.store, &actor(), plan).unwrap_err().code,
        "VERSION_CONFLICT"
    );
    assert!(e.store.list("room_roster").unwrap().is_empty());
}

#[test]
fn selection_does_not_expand_profile_permissions_or_context_budget() {
    let mut e = Env::new();
    let a = e.a.clone();
    let room = chat::main_binding(&e.store, &a).unwrap().1.id;
    let original = accepted_selection(&mut e, &room, "scope", true, None);
    for (name, permission, budget) in [
        (
            "permissions",
            json!({"allow":[]}),
            json!({"max_bytes":65536}),
        ),
        (
            "budget",
            json!({"allow":["context.read"]}),
            json!({"max_bytes":65535}),
        ),
    ] {
        let mut selected = original.clone();
        selected.permission = permission;
        selected.budget = budget;
        assert_eq!(
            e.apply(name, select_action(&a, &room, vec![selected]))
                .unwrap_err()
                .code,
            "PROFILE_SCOPE_EXCEEDED"
        );
    }
    assert!(e.store.list("room_roster").unwrap().is_empty());
}

#[test]
fn same_harness_can_be_selected_twice_but_duplicate_profile_candidates_are_rejected() {
    let mut e = Env::new();
    let a = e.a.clone();
    let room = chat::main_binding(&e.store, &a).unwrap().1.id;
    let one = accepted_selection(&mut e, &room, "same-harness-1", true, None);
    let two = accepted_selection(&mut e, &room, "same-harness-2", true, None);
    let mut duplicate = one.clone();
    duplicate
        .worker_profiles
        .push(duplicate.worker_profiles[0].clone());
    assert_eq!(
        e.apply(
            "duplicate-profile",
            select_action(&a, &room, vec![duplicate])
        )
        .unwrap_err()
        .code,
        "INVALID_INPUT"
    );
    let result = e
        .apply("same-harness", select_action(&a, &room, vec![one, two]))
        .unwrap();
    let refs: Vec<Reference> = serde_json::from_value(result["selections"].clone()).unwrap();
    assert_eq!(refs.len(), 2);
    assert_ne!(refs[0], refs[1]);
}

#[test]
fn ordinary_topic_does_not_trigger_idle_attention() {
    let mut e = Env::new();
    let a = e.a.clone();
    let task = e.claim(&a);
    let req = e.create_request("ask", e.spec(&task));
    let request_id = req["request_id"].as_str().unwrap();
    let r = required(
        &e.store,
        &key(Scope::Project(a.clone()), "request", request_id),
    )
    .unwrap();
    let request: Request = decode(&r).unwrap();
    let texts = chat::request_texts(&e.store, &reference(&r), &request.blockers).unwrap();
    let request_topic = e.create_topic(
        "request-topic",
        chat::Origin::Request {
            request: reference(&r),
            blockers: request.blockers,
        },
        texts,
    );
    let (binding, main) = chat::main_binding(&e.store, &a).unwrap();
    let body = json!({"body":"ordinary discussion","msgtype":"m.text"}).to_string();
    let ordinary_topic = e.create_topic(
        "ordinary-topic",
        chat::Origin::Room {
            room_id: main.id,
            binding_version: binding.version,
        },
        vec![chat::SourceText {
            source: chat::Source::Message {
                binding: reference(&binding),
                event_id: "$ordinary".into(),
                content_digest: foundation::bytes_sha256(body.as_bytes()),
            },
            body,
            excerpt: "ordinary discussion".into(),
        }],
    );
    let now = task::now();
    for room_id in [&request_topic, &ordinary_topic] {
        let identity =
            required(&e.store, &key(Scope::Project(a.clone()), "room", room_id)).unwrap();
        assert!(matches!(
            identity.data,
            store::RecordData::Room {
                room_kind: store::RoomKind::Topic,
                state: RoomState::Active,
            }
        ));
    }
    let activity = BTreeMap::from([
        (request_topic.clone(), Some(now)),
        (ordinary_topic, Some(now)),
    ]);
    // Both Topics have the same readable activity time; only the Request Topic qualifies.
    let items = attention(&e.store, &a, now + 15 * 86400, &activity).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["room_id"], request_topic);
}

#[test]
fn resolved_request_topic_does_not_trigger_idle_attention() {
    let mut e = Env::new();
    let a = e.a.clone();
    let task = e.claim(&a);
    let req = e.create_request("ask", e.spec(&task));
    let request_id = req["request_id"].as_str().unwrap().to_owned();
    let r = required(
        &e.store,
        &key(Scope::Project(a.clone()), "request", &request_id),
    )
    .unwrap();
    let request: Request = decode(&r).unwrap();
    let texts = chat::request_texts(&e.store, &reference(&r), &request.blockers).unwrap();
    let topic = e.create_topic(
        "request-topic",
        chat::Origin::Request {
            request: reference(&r),
            blockers: request.blockers,
        },
        texts,
    );
    let now = task::now();
    let activity = BTreeMap::from([(topic.clone(), Some(now))]);
    let idle_at = now + 15 * 86400;
    let before = attention(&e.store, &a, idle_at, &activity).unwrap();
    assert_eq!(before.len(), 1);
    assert_eq!(before[0]["room_id"], topic);
    e.apply(
        "answer",
        Action::ResolveRequest {
            project_id: a.clone(),
            request_id: request_id.clone(),
            version: r.version,
            adoption: e.adoption(),
        },
    )
    .unwrap();
    assert_eq!(
        requests(&e.store, &a).unwrap()[0].1.state,
        RequestState::Resolved
    );
    let identity = required(&e.store, &key(Scope::Project(a.clone()), "room", &topic)).unwrap();
    assert!(matches!(
        identity.data,
        store::RecordData::Room {
            state: RoomState::Active,
            ..
        }
    ));
    // The Room stays active and equally idle; resolving its Request alone removes the reminder.
    assert!(
        attention(&e.store, &a, idle_at, &activity)
            .unwrap()
            .is_empty()
    );
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
