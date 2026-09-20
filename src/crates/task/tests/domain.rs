use serde_json::{Value, json};
use std::sync::atomic::{AtomicU64, Ordering};
use store::{
    Actor, ActorSource, Command, Expected, ExternalEntity, ProjectSettings, Record, RecordData,
    Reference, Scope, Store, TrustedActor, Version,
};
use task::*;
static NEXT: AtomicU64 = AtomicU64::new(0);

#[test]
fn observed_pending_creation_cannot_be_claimed_as_second_task() {
    let mut e = Env::new();
    e.observe("seed", false);
    let group = Group {
        kind: GroupKind::Label,
        anchor_stable_id: "9".into(),
    };
    e.attach("A", Some(group.clone()));
    let created = apply(
        &mut e.store,
        "create",
        Action::Create {
            project_id: "A".into(),
            project_version: 1,
            source_id: e.sid.clone(),
            title: "new".into(),
            body: "body".into(),
            adoption: None,
        },
    )
    .unwrap();
    let id = created["effect_id"].as_str().unwrap();
    let (effect, send) = begin(&mut e.store, &actor(), id).unwrap();
    assert!(send);
    let (_, src) = source(&e.store, &e.rid, &e.sid).unwrap();
    let (_, mut snap) = latest(&e.store, &src).unwrap();
    snap.cards[0].body = effect.input["write"]["body"].as_str().unwrap().into();
    snap.cards[0].title = "new".into();
    snap.cards[0].groups = vec![group];
    let card = snap.cards[0].clone();
    observe(&mut e.store, &src, snap).unwrap();
    assert_eq!(tasks(&e.store).unwrap().len(), 1);
    assert_eq!(
        apply(
            &mut e.store,
            "claim",
            Action::Claim {
                project_id: "A".into(),
                project_version: 1,
                source_id: e.sid.clone(),
                entity_id: "node1".into(),
            }
        )
        .unwrap_err()
        .code,
        "CREATION_PENDING"
    );
    confirm(&mut e.store, id, &card).unwrap();
    assert_eq!(e.claim("A"), created["task_id"].as_str().unwrap());
    assert_eq!(tasks(&e.store).unwrap().len(), 1);
}

#[test]
fn pending_delete_rechecks_consequences_before_dispatch() {
    let mut e = Env::new();
    e.attach("A", None);
    e.attach("B", None);
    e.observe("card", false);
    let a = e.claim("A");
    let (r, _) = task(&e.store, "A", &a).unwrap();
    let result = apply(
        &mut e.store,
        "delete",
        Action::DeleteCard {
            project_id: "A".into(),
            task_id: a,
            version: r.version,
            confirm_irreversible: true,
            active_run_choices: vec![],
        },
    )
    .unwrap();
    e.claim("B");
    let effect = result["effect_id"].as_str().unwrap();
    assert_eq!(
        begin(&mut e.store, &actor(), effect).unwrap_err().code,
        "VERSION_CONFLICT"
    );
    assert_eq!(
        e.store.effect(effect).unwrap().1,
        store::EffectState::Pending
    );
}

#[test]
fn local_adoption_survives_disabled_source_without_faking_backend_authority() {
    let mut e = Env::new();
    e.attach("A", None);
    e.observe("card", false);
    let id = e.claim("A");
    let (sr, _) = source(&e.store, &e.rid, &e.sid).unwrap();
    apply(
        &mut e.store,
        "disable",
        Action::SetActive {
            repo_id: e.rid.clone(),
            source_id: e.sid.clone(),
            version: sr.version,
            active: false,
        },
    )
    .unwrap();
    let (r, _) = task(&e.store, "A", &id).unwrap();
    let adoption = e.local_contract("A");
    apply(
        &mut e.store,
        "adopt",
        Action::Adopt {
            project_id: "A".into(),
            project_version: 1,
            task_id: id.clone(),
            version: r.version,
            adoption,
        },
    )
    .unwrap();
    let (_, task) = task(&e.store, "A", &id).unwrap();
    assert!(task.revision.unwrap().binding.is_none());
    assert!(task.needs_attention);
}
struct Env {
    store: Store,
    root: std::path::PathBuf,
    rid: String,
    sid: String,
}
impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
fn actor() -> TrustedActor {
    TrustedActor(Actor {
        principal: "owner".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![Scope::Control],
        authority: None,
    })
}
fn apply(s: &mut Store, k: &str, a: Action) -> Result<Value> {
    let p = prepare(
        s,
        Input {
            key: k.into(),
            action: a,
        },
    )?;
    admit(s, &actor(), p)
}
fn seed(s: &mut Store, record: Record) {
    let a = owner(&actor(), [record.key.scope.clone()]).unwrap();
    let input = serde_json::to_value(&record).unwrap();
    let cmd = Command {
        command_id: format!("seed:{}:{}", record.key.id, record.version),
        idempotency_key: format!("seed:{}:{}", record.key.id, record.version),
        actor: a.0.clone(),
        target: record.key.clone(),
        expected: if record.version == 1 {
            Expected::Absent
        } else {
            Expected::Exact(Version::State(record.version - 1))
        },
        binding: reference(&record),
        input_digest: Command::digest_input("seed", &input).unwrap(),
        operation: "seed".into(),
        input,
    };
    s.submit(s.generation(), &a, &cmd, None, |tx| {
        tx.put(&record)?;
        Ok(json!({}))
    })
    .unwrap();
}
impl Env {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "hctl-task-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let mut s = Store::open(&root).unwrap();
        let request = repo::Register {
            name: "fixture".into(),
            origin: repo::Origin::External,
            platform: Some(repo::Platform::Github),
            instance: Some("github.com".into()),
            platform_repo_id: Some("77".into()),
            platform_path: Some("owner/fixture".into()),
            local: None,
            remote_evidence: None,
            default_source: Some("github_issues".into()),
        };
        let reg = repo::admit(
            &mut s,
            &actor(),
            "repo",
            "repo",
            repo::prepare(request, None).unwrap(),
        )
        .unwrap();
        repo::begin_step(&mut s, &actor(), &reg.repo_id, "platform").unwrap();
        let reg = repo::confirm_platform(
            &mut s,
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
        assert_eq!(reg.lifecycle, repo::Lifecycle::Active);
        for id in ["A", "B"] {
            let data = RecordData::Project {
                repo_id: reg.repo_id.clone(),
                settings: ProjectSettings {
                    publish_review_requires_confirmation: true,
                    selection_policy: json!({}),
                },
                archived: false,
            };
            seed(
                &mut s,
                Record {
                    key: key(Scope::Project(id.into()), "project", id),
                    version: 1,
                    revision_digest: foundation::canonical_json_sha256(
                        &serde_json::to_value(&data).unwrap(),
                    )
                    .unwrap(),
                    data,
                    sources: vec![],
                    materials: vec![],
                },
            );
        }
        let result = apply(
            &mut s,
            "connect",
            Action::Connect {
                repo_id: reg.repo_id.clone(),
                candidate_id: "github_issues".into(),
                consent: true,
                make_default: false,
            },
        )
        .unwrap();
        let sid = result["source_id"].as_str().unwrap().into();
        Self {
            store: s,
            root,
            rid: reg.repo_id,
            sid,
        }
    }
    fn attach(&mut self, p: &str, group: Option<Group>) {
        apply(
            &mut self.store,
            &format!("attach-{p}"),
            Action::Attach {
                project_id: p.into(),
                project_version: 1,
                source_id: self.sid.clone(),
                approved_scope: "77".into(),
                group,
                consent: true,
            },
        )
        .unwrap();
    }
    fn observe(&mut self, title: &str, group: bool) {
        let (r, src) = source(&self.store, &self.rid, &self.sid).unwrap();
        let g = Group {
            kind: GroupKind::Label,
            anchor_stable_id: "9".into(),
        };
        let card = Card {
            entity: ExternalEntity {
                provider: "github_issues@github.com".into(),
                account_stable_id: "5".into(),
                external_entity_kind: "issue".into(),
                immutable_external_entity_id: "node1".into(),
            },
            number: 1,
            title: title.into(),
            body: "body".into(),
            stage: "open".into(),
            remote_revision: title.into(),
            content_version: None,
            groups: if group { vec![g.clone()] } else { vec![] },
            dependencies: Dependencies {
                parent: Some(json!({"id":2})),
                children: vec![json!({"id":3})],
                blocked_by: vec![json!({"id":4})],
                blocking: vec![json!({"id":5})],
            },
            raw: json!({"title":title}),
            comments: vec![],
            tombstone: false,
        };
        observe(
            &mut self.store,
            &src,
            Snapshot {
                source: reference(&r),
                observed_at: now(),
                complete: true,
                error: None,
                cards: vec![card],
                stable_groups: vec![g],
            },
        )
        .unwrap();
    }
    fn claim(&mut self, p: &str) -> String {
        apply(
            &mut self.store,
            &format!("claim-{p}"),
            Action::Claim {
                project_id: p.into(),
                project_version: 1,
                source_id: self.sid.clone(),
                entity_id: "node1".into(),
            },
        )
        .unwrap()["task_id"]
            .as_str()
            .unwrap()
            .into()
    }
    fn local_contract(&self, p: &str) -> Adoption {
        let contract = Contract {
            scope: "scope".into(),
            expected_outcome: "outcome".into(),
            acceptance: vec![Acceptance {
                text: "verified".into(),
                grade: Grade::Human,
            }],
            roles: vec![],
            capabilities: vec![],
        };
        Adoption {
            origin: ContractOrigin::Local {
                reference: Reference {
                    key: key(Scope::Project(p.into()), "project", p),
                    version: Version::State(1),
                },
                proposal_digest: foundation::canonical_json_sha256(
                    &serde_json::to_value(&contract).unwrap(),
                )
                .unwrap(),
            },
            contract,
        }
    }
}

#[test]
fn namespace_claim_observation_and_contracts_are_independent() {
    let mut e = Env::new();
    e.attach("A", None);
    e.attach("B", None);
    e.observe("old", false);
    assert!(tasks(&e.store).unwrap().is_empty());
    assert_eq!(
        board(&e.store, "A", &e.sid).unwrap()["cards"][0]["claimed"],
        false
    );
    let a = e.claim("A");
    let b = e.claim("B");
    assert_ne!(a, b);
    assert_eq!(a, e.claim("A"));
    assert_eq!(tasks(&e.store).unwrap().len(), 2);
    let (r, _) = task(&e.store, "A", &a).unwrap();
    let adoption = e.local_contract("A");
    let input = Input {
        key: "adopt".into(),
        action: Action::Adopt {
            project_id: "A".into(),
            project_version: 1,
            task_id: a.clone(),
            version: r.version,
            adoption,
        },
    };
    let p = prepare(&e.store, input.clone()).unwrap();
    admit(&mut e.store, &actor(), p).unwrap();
    let before = task(&e.store, "A", &a).unwrap().1.revision.unwrap();
    e.observe("changed", false);
    for p in ["A", "B"] {
        assert_eq!(
            board(&e.store, p, &e.sid).unwrap()["cards"][0]["card"]["title"],
            "changed"
        );
    }
    assert_eq!(
        task(&e.store, "A", &a)
            .unwrap()
            .1
            .revision
            .unwrap()
            .material,
        before.material
    );
    assert!(task(&e.store, "B", &b).unwrap().1.revision.is_none());
    let p = prepare(&e.store, input).unwrap();
    admit(&mut e.store, &actor(), p).unwrap();
    assert_eq!(e.store.list("task_revision").unwrap().len(), 1);
    assert_eq!(task(&e.store, "B", &b).unwrap().1.lifecycle, "open");
}

#[test]
fn source_consent_scope_disable_reconnect_and_second_reference() {
    let mut e = Env::new();
    assert!(e.store.list("task_source_reference").unwrap().is_empty());
    assert_eq!(
        apply(
            &mut e.store,
            "no",
            Action::Connect {
                repo_id: e.rid.clone(),
                candidate_id: "github_issues".into(),
                consent: false,
                make_default: true
            }
        )
        .unwrap_err()
        .code,
        "CONSENT_REQUIRED"
    );
    assert_eq!(
        apply(
            &mut e.store,
            "bad",
            Action::Attach {
                project_id: "A".into(),
                project_version: 1,
                source_id: e.sid.clone(),
                approved_scope: "different".into(),
                group: None,
                consent: true
            }
        )
        .unwrap_err()
        .code,
        "SOURCE_SCOPE"
    );
    e.attach("A", None);
    e.observe("card", false);
    let id = e.claim("A");
    let (sr, mut second) = source(&e.store, &e.rid, &e.sid).unwrap();
    second.id = "other-source".into();
    second.board_scope_stable_id = "88".into();
    second.platform.stable_id = "88".into();
    seed(
        &mut e.store,
        value_record(
            key(Scope::Repo(e.rid.clone()), "task_source", "other-source"),
            1,
            &second,
        )
        .unwrap(),
    );
    apply(
        &mut e.store,
        "second",
        Action::Attach {
            project_id: "A".into(),
            project_version: 1,
            source_id: second.id,
            approved_scope: "88".into(),
            group: None,
            consent: true,
        },
    )
    .unwrap();
    assert_eq!(e.store.list("task_source_reference").unwrap().len(), 2);
    apply(
        &mut e.store,
        "disable",
        Action::SetActive {
            repo_id: e.rid.clone(),
            source_id: e.sid.clone(),
            active: false,
            version: sr.version,
        },
    )
    .unwrap();
    assert!(task(&e.store, "A", &id).unwrap().1.needs_attention);
    assert_eq!(board(&e.store, "A", &e.sid).unwrap()["complete"], false);
    apply(
        &mut e.store,
        "enable",
        Action::SetActive {
            repo_id: e.rid.clone(),
            source_id: e.sid.clone(),
            active: true,
            version: sr.version + 1,
        },
    )
    .unwrap();
    e.observe("card", false);
    assert_eq!(task(&e.store, "A", &id).unwrap().1.lifecycle, "open");
    assert_eq!(tasks(&e.store).unwrap().len(), 1);
}

#[test]
fn label_only_auto_claim_two_projects_and_group_drift_does_not_freeze() {
    let mut e = Env::new();
    e.observe("card", true);
    let g = Some(Group {
        kind: GroupKind::Label,
        anchor_stable_id: "9".into(),
    });
    e.attach("A", g.clone());
    e.attach("B", g);
    e.observe("card", true);
    assert_eq!(tasks(&e.store).unwrap().len(), 2);
    e.observe("moved", false);
    assert_eq!(tasks(&e.store).unwrap().len(), 2);
    for (_, t) in tasks(&e.store).unwrap() {
        assert!(!t.needs_attention);
        assert_eq!(t.lifecycle, "open");
    }
    let view = board(&e.store, "A", &e.sid).unwrap();
    for name in ["parent", "children", "blocked_by", "blocking"] {
        assert!(!view["cards"][0]["card"]["dependencies"][name].is_null());
    }
    assert!(e.store.list("dependency").unwrap().is_empty());
}

#[test]
fn stale_snapshot_and_illegal_moves_fail_and_offline_is_not_empty() {
    let mut e = Env::new();
    e.attach("A", None);
    e.observe("card", false);
    let id = e.claim("A");
    let (r, t) = task(&e.store, "A", &id).unwrap();
    let action = Action::Move {
        project_id: "A".into(),
        task_id: id.clone(),
        version: r.version,
        state_version: t.state_version,
        stage: "closed".into(),
        rank: None,
        relative_source_id: None,
    };
    let plan = prepare(
        &e.store,
        Input {
            key: "move".into(),
            action: action.clone(),
        },
    )
    .unwrap();
    e.observe("changed", false);
    assert_eq!(
        admit(&mut e.store, &actor(), plan).unwrap_err().code,
        "VERSION_CONFLICT"
    );
    let (r, t) = task(&e.store, "A", &id).unwrap();
    assert_eq!(
        apply(
            &mut e.store,
            "cross",
            Action::Move {
                project_id: "A".into(),
                task_id: id.clone(),
                version: r.version,
                state_version: t.state_version,
                stage: "closed".into(),
                rank: None,
                relative_source_id: Some("elsewhere".into())
            }
        )
        .unwrap_err()
        .code,
        "CROSS_SOURCE_MOVE"
    );
    assert!(serde_json::from_value::<Action>(json!({"kind":"move","project_id":"A","task_id":id,"version":r.version,"state_version":t.state_version,"stage":"closed","lifecycle":"completed"})).is_err());
    let (r, src) = source(&e.store, &e.rid, &e.sid).unwrap();
    observe(
        &mut e.store,
        &src,
        Snapshot {
            source: reference(&r),
            observed_at: now(),
            complete: false,
            error: Some("offline".into()),
            cards: vec![],
            stable_groups: vec![],
        },
    )
    .unwrap();
    let board = board(&e.store, "A", &e.sid).unwrap();
    assert_eq!(board["complete"], false);
    assert_eq!(board["cards"].as_array().unwrap().len(), 1);
    assert_eq!(
        latest(&e.store, &src).unwrap_err().code,
        "READBACK_REQUIRED"
    );
}

#[test]
fn adoption_grades_project_scope_and_backend_version_are_checked() {
    let mut e = Env::new();
    e.attach("A", None);
    e.observe("card", false);
    let id = e.claim("A");
    let (r, t) = task(&e.store, "A", &id).unwrap();
    let bad = e.local_contract("B");
    assert_eq!(
        apply(
            &mut e.store,
            "bad",
            Action::Adopt {
                project_id: "A".into(),
                project_version: 1,
                task_id: id.clone(),
                version: r.version,
                adoption: bad
            }
        )
        .unwrap_err()
        .code,
        "PROJECT_MISMATCH"
    );
    let mut a = e.local_contract("A");
    let mut json = serde_json::to_value(&a).unwrap();
    json["contract"]["acceptance"][0]
        .as_object_mut()
        .unwrap()
        .remove("grade");
    assert!(serde_json::from_value::<Adoption>(json).is_err());
    a.origin = ContractOrigin::Backend {
        snapshot: t.snapshot.unwrap(),
        state_version: t.state_version + 1,
    };
    assert_eq!(
        apply(
            &mut e.store,
            "bad-snapshot",
            Action::Adopt {
                project_id: "A".into(),
                project_version: 1,
                task_id: id,
                version: r.version,
                adoption: a
            }
        )
        .unwrap_err()
        .code,
        "VERSION_CONFLICT"
    );
}

#[test]
fn cancellation_is_local_and_delete_preview_includes_other_project_run() {
    let mut e = Env::new();
    e.attach("A", None);
    e.attach("B", None);
    e.observe("card", false);
    let a = e.claim("A");
    let b = e.claim("B");
    let run = value_record(
        key(Scope::Project("B".into()), "run", "run-b"),
        1,
        &json!({"task_id":b,"lifecycle":"active"}),
    )
    .unwrap();
    seed(&mut e.store, run.clone());
    let (r, _) = task(&e.store, "A", &a).unwrap();
    let plan = prepare(
        &e.store,
        Input {
            key: "delete".into(),
            action: Action::DeleteCard {
                project_id: "A".into(),
                task_id: a.clone(),
                version: r.version,
                confirm_irreversible: false,
                active_run_choices: vec![],
            },
        },
    )
    .unwrap();
    assert_eq!(plan.result["affected"].as_array().unwrap().len(), 2);
    assert!(plan.result.to_string().contains("run-b"));
    assert_eq!(
        admit(&mut e.store, &actor(), plan).unwrap_err().code,
        "DELETE_CONFIRMATION_REQUIRED"
    );
    assert_eq!(
        apply(
            &mut e.store,
            "delete-yes",
            Action::DeleteCard {
                project_id: "A".into(),
                task_id: a.clone(),
                version: r.version,
                confirm_irreversible: true,
                active_run_choices: vec![]
            }
        )
        .unwrap_err()
        .code,
        "RUN_CHOICE_REQUIRED"
    );
    apply(
        &mut e.store,
        "cancel",
        Action::Cancel {
            project_id: "A".into(),
            task_id: a.clone(),
            version: r.version,
        },
    )
    .unwrap();
    assert_eq!(task(&e.store, "A", &a).unwrap().1.lifecycle, "cancelled");
    assert_eq!(task(&e.store, "B", &b).unwrap().1.lifecycle, "open");
    assert!(e.store.pending_effects().unwrap().is_empty());
    let (r, _) = task(&e.store, "B", &b).unwrap();
    assert_eq!(
        apply(
            &mut e.store,
            "cancel-b",
            Action::Cancel {
                project_id: "B".into(),
                task_id: b,
                version: r.version
            }
        )
        .unwrap_err()
        .code,
        "RUN_ACTIVE"
    );
}

#[test]
fn create_material_outbox_retry_is_one_revision_and_unknown_is_read_only() {
    let mut e = Env::new();
    e.attach("A", None);
    let action = Action::Create {
        project_id: "A".into(),
        project_version: 1,
        source_id: e.sid.clone(),
        title: "new".into(),
        body: "body".into(),
        adoption: Some(e.local_contract("A")),
    };
    let r = apply(&mut e.store, "create", action.clone()).unwrap();
    let effect = r["effect_id"].as_str().unwrap();
    assert_eq!(e.store.list("task_revision").unwrap().len(), 1);
    assert!(begin(&mut e.store, &actor(), effect).unwrap().1);
    assert!(!begin(&mut e.store, &actor(), effect).unwrap().1);
    let r2 = apply(&mut e.store, "create", action).unwrap();
    assert_eq!(r, r2);
    assert_eq!(tasks(&e.store).unwrap().len(), 1);
    assert_eq!(e.store.list("task_revision").unwrap().len(), 1);
    let mut altered = serde_json::to_value(
        &prepare(
            &e.store,
            Input {
                key: "create".into(),
                action: Action::Resume {
                    effect_id: effect.into(),
                },
            },
        )
        .unwrap_err()
        .code,
    )
    .unwrap();
    assert_eq!(altered.take(), "IDEMPOTENCY_CONFLICT");
}

#[test]
fn source_contract_drift_is_pending_but_stage_change_is_projection_only() {
    let mut e = Env::new();
    e.attach("A", None);
    e.observe("title", false);
    let id = e.claim("A");
    let (r, t) = task(&e.store, "A", &id).unwrap();
    let mut adoption = e.local_contract("A");
    adoption.origin = ContractOrigin::Backend {
        snapshot: t.snapshot.unwrap(),
        state_version: t.state_version,
    };
    apply(
        &mut e.store,
        "adopt-external",
        Action::Adopt {
            project_id: "A".into(),
            project_version: 1,
            task_id: id.clone(),
            version: r.version,
            adoption,
        },
    )
    .unwrap();
    let (sr, src) = source(&e.store, &e.rid, &e.sid).unwrap();
    let (_, mut snap) = latest(&e.store, &src).unwrap();
    snap.cards[0].stage = "closed".into();
    snap.source = reference(&sr);
    observe(&mut e.store, &src, snap).unwrap();
    let (_, t) = task(&e.store, "A", &id).unwrap();
    assert!(t.pending_contract.is_none());
    assert_eq!(t.lifecycle, "open");
    e.observe("new title", false);
    let (_, t) = task(&e.store, "A", &id).unwrap();
    assert!(t.pending_contract.is_some());
    assert_eq!(t.revision.unwrap().number, 1);
    assert_eq!(t.lifecycle, "open");
}

#[test]
fn new_binding_after_delete_preview_invalidates_shared_consequence_list() {
    let mut e = Env::new();
    e.attach("A", None);
    e.attach("B", None);
    e.observe("card", false);
    let a = e.claim("A");
    let (r, _) = task(&e.store, "A", &a).unwrap();
    let plan = prepare(
        &e.store,
        Input {
            key: "delete".into(),
            action: Action::DeleteCard {
                project_id: "A".into(),
                task_id: a,
                version: r.version,
                confirm_irreversible: true,
                active_run_choices: vec![],
            },
        },
    )
    .unwrap();
    e.claim("B");
    assert_eq!(
        admit(&mut e.store, &actor(), plan).unwrap_err().code,
        "VERSION_CONFLICT"
    );
    assert!(e.store.pending_effects().unwrap().is_empty());
}

#[test]
fn cancelling_unsent_creation_withdraws_only_pending_effect() {
    let mut e = Env::new();
    e.attach("A", None);
    let created = apply(
        &mut e.store,
        "create",
        Action::Create {
            project_id: "A".into(),
            project_version: 1,
            source_id: e.sid.clone(),
            title: "new".into(),
            body: "body".into(),
            adoption: None,
        },
    )
    .unwrap();
    let id = created["task_id"].as_str().unwrap();
    let (r, _) = task(&e.store, "A", id).unwrap();
    apply(
        &mut e.store,
        "cancel",
        Action::Cancel {
            project_id: "A".into(),
            task_id: id.into(),
            version: r.version,
        },
    )
    .unwrap();
    assert_eq!(
        e.store
            .effect(created["effect_id"].as_str().unwrap())
            .unwrap()
            .1,
        store::EffectState::Cancelled
    );
    assert_eq!(task(&e.store, "A", id).unwrap().1.lifecycle, "cancelled");
}

#[test]
fn no_current_source_read_can_replace_contract_or_write_while_project_archived() {
    let mut e = Env::new();
    e.attach("A", None);
    e.observe("card", false);
    let id = e.claim("A");
    let project_key = key(Scope::Project("A".into()), "project", "A");
    let mut r = required(&e.store, &project_key).unwrap();
    if let RecordData::Project { archived, .. } = &mut r.data {
        *archived = true;
    }
    r.version += 1;
    r.revision_digest =
        foundation::canonical_json_sha256(&serde_json::to_value(&r.data).unwrap()).unwrap();
    seed(&mut e.store, r);
    let (r, _) = task(&e.store, "A", &id).unwrap();
    assert_eq!(
        apply(
            &mut e.store,
            "cancel-archived",
            Action::Cancel {
                project_id: "A".into(),
                task_id: id.clone(),
                version: r.version
            }
        )
        .unwrap_err()
        .code,
        "PROJECT_ARCHIVED"
    );
    let mut actor = actor();
    actor.0.source = ActorSource::ProviderEvent;
    let input = Input {
        key: "spoof".into(),
        action: Action::Refresh {
            repo_id: e.rid.clone(),
            source_id: e.sid.clone(),
        },
    };
    let plan = prepare(&e.store, input).unwrap();
    assert_eq!(
        admit(&mut e.store, &actor, plan).unwrap_err().code,
        "PERMISSION_DENIED"
    );
}

#[test]
fn saved_material_before_admission_recovers_once_after_reopen() {
    let mut e = Env::new();
    e.attach("A", None);
    let adoption = e.local_contract("A");
    let owner = owner(&actor(), [Scope::Project("A".into())]).unwrap();
    let bytes =
        foundation::canonical_json(&serde_json::to_value(&adoption.contract).unwrap()).unwrap();
    e.store
        .save_material(
            e.store.generation(),
            &owner,
            &Scope::Project("A".into()),
            "create-crash",
            "contract",
            &bytes,
        )
        .unwrap();
    assert!(e.store.list("task_revision").unwrap().is_empty());
    let scratch = Store::open(&e.root.join("scratch")).unwrap();
    let original = std::mem::replace(&mut e.store, scratch);
    drop(original);
    e.store = Store::open(&e.root).unwrap();
    let action = Action::Create {
        project_id: "A".into(),
        project_version: 1,
        source_id: e.sid.clone(),
        title: "new".into(),
        body: "body".into(),
        adoption: Some(adoption),
    };
    let first = apply(&mut e.store, "create-crash", action.clone()).unwrap();
    let second = apply(&mut e.store, "create-crash", action).unwrap();
    assert_eq!(first, second);
    assert_eq!(e.store.list("task_revision").unwrap().len(), 1);
}
