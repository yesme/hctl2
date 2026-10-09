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
fn explicit_content_deletion_after_cancel_keeps_task_cancelled() {
    let mut e = Env::new();
    e.attach("A", None);
    e.observe("card", false);
    let id = e.claim("A");
    let (r, _) = task(&e.store, "A", &id).unwrap();
    apply(
        &mut e.store,
        "cancel",
        Action::Cancel {
            project_id: "A".into(),
            task_id: id.clone(),
            version: r.version,
        },
    )
    .unwrap();
    assert!(e.store.pending_effects().unwrap().is_empty());
    let (r, _) = task(&e.store, "A", &id).unwrap();
    let result = apply(
        &mut e.store,
        "delete",
        Action::DeleteCard {
            project_id: "A".into(),
            task_id: id.clone(),
            version: r.version,
            confirm_irreversible: true,
            active_run_choices: vec![],
        },
    )
    .unwrap();
    let effect = result["effect_id"].as_str().unwrap();
    let (intent, send) = begin(&mut e.store, &actor(), effect).unwrap();
    assert!(send);
    let mut card: Card = serde_json::from_value(intent.input["write"]["card"].clone()).unwrap();
    card.tombstone = true;
    confirm(&mut e.store, effect, &card).unwrap();
    let (_, task) = task(&e.store, "A", &id).unwrap();
    assert_eq!(task.lifecycle, "cancelled");
    assert!(task.archived);
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
    let (sr, _) = source(&e.store, &e.rid, &e.sid).unwrap();
    apply(
        &mut e.store,
        "enable",
        Action::SetActive {
            repo_id: e.rid.clone(),
            source_id: e.sid.clone(),
            version: sr.version,
            active: true,
        },
    )
    .unwrap();
    let (_, src) = source(&e.store, &e.rid, &e.sid).unwrap();
    assert_eq!(
        latest(&e.store, &src).unwrap_err().code,
        "READBACK_REQUIRED"
    );
    e.observe("card", false);
    assert!(latest(&e.store, &src).is_ok());
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
                human_account: None,
                auto_complete_provider_done: false,
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
                evidence: None,
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

// —— 第 7 包：完成 Task 与重开 ————————————————————————————————

fn apply_as(s: &mut Store, actor: &TrustedActor, k: &str, a: Action) -> Result<Value> {
    let p = prepare_as(
        s,
        actor,
        Input {
            key: k.into(),
            action: a,
        },
    )?;
    admit(s, actor, p)
}

/// 先按当前事实组 action（只借用一次 Env），再在一条命令里 prepare+admit。
fn apply_built(
    e: &mut Env,
    actor: &TrustedActor,
    k: &str,
    build: impl FnOnce(&Env) -> Action,
) -> Result<Value> {
    let action = build(e);
    apply_as(&mut e.store, actor, k, action)
}

/// 完成用的 actor：直连的人，并带上该 Project 的权限范围（读契约材料需要）。
fn completing_actor(project: &str) -> TrustedActor {
    owner(&actor(), [Scope::Project(project.into())]).unwrap()
}

/// 机械项（不点名集成凭证）+ 人判项各一的契约。
fn mixed_contract(p: &str) -> Adoption {
    let contract = Contract {
        scope: "scope".into(),
        expected_outcome: "outcome".into(),
        acceptance: vec![
            Acceptance {
                text: "build is green".into(),
                grade: Grade::Mechanical,
                evidence: Some(EvidenceRequirement {
                    accept: None,
                    min_channel: None,
                }),
            },
            Acceptance {
                text: "person accepts".into(),
                grade: Grade::Human,
                evidence: None,
            },
        ],
        roles: vec![],
        capabilities: vec![],
    };
    adoption_of(p, contract)
}

/// 机械项点名要第 6 包签出的 Integration Receipt 的契约。
fn integration_contract(p: &str) -> Adoption {
    let contract = Contract {
        scope: "scope".into(),
        expected_outcome: "outcome".into(),
        acceptance: vec![Acceptance {
            text: "the change is integrated".into(),
            grade: Grade::Mechanical,
            evidence: Some(EvidenceRequirement {
                accept: Some("integration_receipt".into()),
                min_channel: Some("unmediated".into()),
            }),
        }],
        roles: vec![],
        capabilities: vec![],
    };
    adoption_of(p, contract)
}

/// Gate 项一个的契约（第 9 包接 Gate 席位；此处只核完成时的判定者与引用）。
fn gate_contract(p: &str) -> Adoption {
    let contract = Contract {
        scope: "scope".into(),
        expected_outcome: "outcome".into(),
        acceptance: vec![Acceptance {
            text: "gate passes".into(),
            grade: Grade::Gate,
            evidence: None,
        }],
        roles: vec![],
        capabilities: vec![],
    };
    adoption_of(p, contract)
}

/// 机械项点名直报下限（`unmediated`）的契约：旁路证据不够。
fn min_channel_contract(p: &str) -> Adoption {
    let contract = Contract {
        scope: "scope".into(),
        expected_outcome: "outcome".into(),
        acceptance: vec![Acceptance {
            text: "build is green".into(),
            grade: Grade::Mechanical,
            evidence: Some(EvidenceRequirement {
                accept: None,
                min_channel: Some("unmediated".into()),
            }),
        }],
        roles: vec![],
        capabilities: vec![],
    };
    adoption_of(p, contract)
}

fn adoption_of(p: &str, contract: Contract) -> Adoption {
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

fn facts(e: &Env, project: &str, id: &str) -> (i64, i64, i64) {
    let (r, t) = task(&e.store, project, id).unwrap();
    (
        r.version,
        t.lifecycle_version,
        t.revision.as_ref().map(|rev| rev.number).unwrap_or(0),
    )
}

fn human_judge() -> Judge {
    Judge::Human {
        actor: "owner".into(),
    }
}

fn complete_action(e: &Env, project: &str, id: &str, evidence: Vec<ItemEvidence>) -> Action {
    let (version, lifecycle_version, revision_number) = facts(e, project, id);
    Action::Complete {
        project_id: project.into(),
        task_id: id.into(),
        version,
        lifecycle_version,
        revision_number,
        acceptance: evidence,
    }
}

/// 一份 Task 从「已采纳 mixed 契约」出发，供完成类用例起步。
fn ready(e: &mut Env, contract: Adoption) -> String {
    e.attach("A", None);
    e.observe("card", false);
    let id = e.claim("A");
    let (r, _) = task(&e.store, "A", &id).unwrap();
    apply(
        &mut e.store,
        "adopt-A",
        Action::Adopt {
            project_id: "A".into(),
            project_version: 1,
            task_id: id.clone(),
            version: r.version,
            adoption: contract,
        },
    )
    .unwrap();
    id
}

fn evidence_record(e: &mut Env, id: &str) -> Reference {
    let key = key(Scope::Project("A".into()), "task_snapshot", id);
    seed(
        &mut e.store,
        value_record(key.clone(), 1, &json!({"evidence": id})).unwrap(),
    );
    Reference {
        key,
        version: Version::State(1),
    }
}

fn seed_open_run(e: &mut Env, id: &str) {
    seed(
        &mut e.store,
        value_record(
            key(Scope::Project("A".into()), "run", "run-1"),
            1,
            &json!({"task_id": id, "lifecycle": "running"}),
        )
        .unwrap(),
    );
}

#[test]
fn completion_preview_refuses_without_a_contract() {
    let mut e = Env::new();
    e.attach("A", None);
    e.observe("card", false);
    let id = e.claim("A");
    let err = apply_built(&mut e, &completing_actor("A"), "complete-A", |e| {
        complete_action(e, "A", &id, vec![])
    })
    .unwrap_err();
    assert_eq!(err.code, "CONTRACT_REQUIRED");
    assert_eq!(err.recovery_action, "adopt_contract");
    let (_, t) = task(&e.store, "A", &id).unwrap();
    assert_eq!(t.lifecycle, "open");
}

#[test]
fn completion_refuses_while_a_bound_run_is_open() {
    let mut e = Env::new();
    let id = ready(&mut e, mixed_contract("A"));
    seed_open_run(&mut e, &id);
    let reference = evidence_record(&mut e, "green");
    let err = apply_built(&mut e, &completing_actor("A"), "complete-A", |e| {
        complete_action(
            e,
            "A",
            &id,
            vec![
                ItemEvidence {
                    item: 0,
                    judge: Judge::Hctl2Tool,
                    channel: "unmediated".into(),
                    references: vec![reference],
                    producer: Some("hctl2-tool".into()),
                    generation: Some(1),
                },
                ItemEvidence {
                    item: 1,
                    judge: human_judge(),
                    channel: "unmediated".into(),
                    references: vec![],
                    producer: None,
                    generation: None,
                },
            ],
        )
    })
    .unwrap_err();
    assert_eq!(err.code, "RUN_ACTIVE");
    assert_eq!(err.recovery_action, "finish_run_first");
    let (_, t) = task(&e.store, "A", &id).unwrap();
    assert_eq!(t.lifecycle, "open");
}

#[test]
fn completion_refuses_narrated_or_weak_evidence() {
    let mut e = Env::new();
    let id = ready(&mut e, mixed_contract("A"));
    let reference = evidence_record(&mut e, "green");
    // 机械项只给转述证据：拒。
    let action = complete_action(
        &e,
        "A",
        &id,
        vec![
            ItemEvidence {
                item: 0,
                judge: Judge::Hctl2Tool,
                channel: "narrated".into(),
                references: vec![reference.clone()],
                producer: None,
                generation: None,
            },
            ItemEvidence {
                item: 1,
                judge: human_judge(),
                channel: "unmediated".into(),
                references: vec![],
                producer: None,
                generation: None,
            },
        ],
    );
    assert_eq!(
        apply_as(
            &mut e.store,
            &completing_actor("A"),
            "complete-narrated",
            action.clone()
        )
        .unwrap_err()
        .code,
        "EVIDENCE_TOO_WEAK"
    );
    // 缺一项的证据：拒。
    let missing = complete_action(
        &e,
        "A",
        &id,
        vec![ItemEvidence {
            item: 0,
            judge: Judge::Hctl2Tool,
            channel: "unmediated".into(),
            references: vec![reference.clone()],
            producer: None,
            generation: None,
        }],
    );
    assert_eq!(
        apply_as(
            &mut e.store,
            &completing_actor("A"),
            "complete-missing",
            missing
        )
        .unwrap_err()
        .code,
        "EVIDENCE_REQUIRED"
    );
    // 引用不存在的证据：拒。
    let ghost = complete_action(
        &e,
        "A",
        &id,
        vec![
            ItemEvidence {
                item: 0,
                judge: Judge::Hctl2Tool,
                channel: "unmediated".into(),
                references: vec![Reference {
                    key: key(Scope::Project("A".into()), "task_snapshot", "ghost"),
                    version: Version::State(1),
                }],
                producer: None,
                generation: None,
            },
            ItemEvidence {
                item: 1,
                judge: human_judge(),
                channel: "unmediated".into(),
                references: vec![],
                producer: None,
                generation: None,
            },
        ],
    );
    assert_eq!(
        apply_as(
            &mut e.store,
            &completing_actor("A"),
            "complete-ghost",
            ghost
        )
        .unwrap_err()
        .code,
        "EVIDENCE_MISSING"
    );
    // 别人的判定不算人判：拒。
    let forged = complete_action(
        &e,
        "A",
        &id,
        vec![
            ItemEvidence {
                item: 0,
                judge: Judge::Hctl2Tool,
                channel: "unmediated".into(),
                references: vec![reference.clone()],
                producer: None,
                generation: None,
            },
            ItemEvidence {
                item: 1,
                judge: Judge::Human {
                    actor: "someone-else".into(),
                },
                channel: "unmediated".into(),
                references: vec![],
                producer: None,
                generation: None,
            },
        ],
    );
    assert_eq!(
        apply_as(
            &mut e.store,
            &completing_actor("A"),
            "complete-forged",
            forged
        )
        .unwrap_err()
        .code,
        "HUMAN_JUDGEMENT_REQUIRED"
    );
    // 契约点名要集成凭证而证据不是它：拒。
    let mut e = Env::new();
    let id = ready(&mut e, integration_contract("A"));
    let reference = evidence_record(&mut e, "green");
    let no_receipt = complete_action(
        &e,
        "A",
        &id,
        vec![ItemEvidence {
            item: 0,
            judge: Judge::Hctl2Tool,
            channel: "unmediated".into(),
            references: vec![reference],
            producer: None,
            generation: None,
        }],
    );
    assert_eq!(
        apply_as(
            &mut e.store,
            &completing_actor("A"),
            "complete-no-receipt",
            no_receipt
        )
        .unwrap_err()
        .code,
        "INTEGRATION_RECEIPT_REQUIRED"
    );
    let (_, t) = task(&e.store, "A", &id).unwrap();
    assert_eq!(t.lifecycle, "open");
}

/// Gate 项只认 Gate 席位的判定：工具回读与非 Gate 判定都拒，缺引用也拒。
#[test]
fn completion_refuses_gate_items_without_a_gate_receipt() {
    let mut e = Env::new();
    let id = ready(&mut e, gate_contract("A"));
    let reference = evidence_record(&mut e, "green");
    let tool = complete_action(
        &e,
        "A",
        &id,
        vec![ItemEvidence {
            item: 0,
            judge: Judge::Hctl2Tool,
            channel: "unmediated".into(),
            references: vec![reference],
            producer: None,
            generation: None,
        }],
    );
    assert_eq!(
        apply_as(
            &mut e.store,
            &completing_actor("A"),
            "complete-gate-tool",
            tool
        )
        .unwrap_err()
        .code,
        "GATE_EVIDENCE_REQUIRED"
    );
    let empty = complete_action(
        &e,
        "A",
        &id,
        vec![ItemEvidence {
            item: 0,
            judge: Judge::Gate {
                seat: "gate-1".into(),
            },
            channel: "unmediated".into(),
            references: vec![],
            producer: None,
            generation: None,
        }],
    );
    assert_eq!(
        apply_as(
            &mut e.store,
            &completing_actor("A"),
            "complete-gate-empty",
            empty
        )
        .unwrap_err()
        .code,
        "GATE_EVIDENCE_REQUIRED"
    );
    let (_, t) = task(&e.store, "A", &id).unwrap();
    assert_eq!(t.lifecycle, "open");
}

/// 未知通道与低于项声明的下限的通道都拒，且都不改动 Task。
#[test]
fn completion_refuses_unknown_and_below_minimum_channels() {
    let mut e = Env::new();
    let id = ready(&mut e, min_channel_contract("A"));
    let reference = evidence_record(&mut e, "green");
    let unknown = complete_action(
        &e,
        "A",
        &id,
        vec![ItemEvidence {
            item: 0,
            judge: Judge::Hctl2Tool,
            channel: "guessed".into(),
            references: vec![reference.clone()],
            producer: None,
            generation: None,
        }],
    );
    assert_eq!(
        apply_as(
            &mut e.store,
            &completing_actor("A"),
            "complete-unknown-channel",
            unknown
        )
        .unwrap_err()
        .code,
        "EVIDENCE_CHANNEL"
    );
    // 契约要求直报，旁路证据不够：拒。
    let bypass = complete_action(
        &e,
        "A",
        &id,
        vec![ItemEvidence {
            item: 0,
            judge: Judge::Hctl2Tool,
            channel: "adapter_event".into(),
            references: vec![reference],
            producer: None,
            generation: None,
        }],
    );
    assert_eq!(
        apply_as(
            &mut e.store,
            &completing_actor("A"),
            "complete-below-min",
            bypass
        )
        .unwrap_err()
        .code,
        "EVIDENCE_TOO_WEAK"
    );
    let (_, t) = task(&e.store, "A", &id).unwrap();
    assert_eq!(t.lifecycle, "open");
}

/// 人判项必须由本人判定：工具代判即拒。
#[test]
fn completion_refuses_a_human_item_judged_by_a_tool() {
    let mut e = Env::new();
    let id = ready(&mut e, mixed_contract("A"));
    let reference = evidence_record(&mut e, "green");
    let tool_judged = complete_action(
        &e,
        "A",
        &id,
        vec![
            ItemEvidence {
                item: 0,
                judge: Judge::Hctl2Tool,
                channel: "unmediated".into(),
                references: vec![reference],
                producer: None,
                generation: None,
            },
            ItemEvidence {
                item: 1,
                judge: Judge::Hctl2Tool,
                channel: "unmediated".into(),
                references: vec![],
                producer: None,
                generation: None,
            },
        ],
    );
    assert_eq!(
        apply_as(
            &mut e.store,
            &completing_actor("A"),
            "complete-tool-judged",
            tool_judged
        )
        .unwrap_err()
        .code,
        "HUMAN_JUDGEMENT_REQUIRED"
    );
    let (_, t) = task(&e.store, "A", &id).unwrap();
    assert_eq!(t.lifecycle, "open");
}

#[test]
fn completion_preview_dies_when_the_task_moves_after_preview() {
    let mut e = Env::new();
    let id = ready(&mut e, mixed_contract("A"));
    let reference = evidence_record(&mut e, "green");
    let action = complete_action(
        &e,
        "A",
        &id,
        vec![
            ItemEvidence {
                item: 0,
                judge: Judge::Hctl2Tool,
                channel: "unmediated".into(),
                references: vec![reference],
                producer: None,
                generation: None,
            },
            ItemEvidence {
                item: 1,
                judge: human_judge(),
                channel: "unmediated".into(),
                references: vec![],
                producer: None,
                generation: None,
            },
        ],
    );
    let actor = completing_actor("A");
    let plan = prepare_as(
        &e.store,
        &actor,
        Input {
            key: "complete-stale".into(),
            action,
        },
    )
    .unwrap();
    // 预览之后来了新 Snapshot：Task 记录前进一个版本，冻结的预览必须失效。
    let (r, mut t) = task(&e.store, "A", &id).unwrap();
    t.state_version += 1;
    seed(
        &mut e.store,
        value_record(r.key.clone(), r.version + 1, &t).unwrap(),
    );
    let err = admit(&mut e.store, &actor, plan).unwrap_err();
    assert_eq!(err.code, stale().code);
    let (_, t) = task(&e.store, "A", &id).unwrap();
    assert_eq!(t.lifecycle, "open");
}

#[test]
fn human_completion_is_refused_while_completion_pending() {
    let mut e = Env::new();
    let id = ready(&mut e, mixed_contract("A"));
    let (r, mut t) = task(&e.store, "A", &id).unwrap();
    t.run_occupancy = Some(json!("completion_pending"));
    seed(
        &mut e.store,
        value_record(r.key.clone(), r.version + 1, &t).unwrap(),
    );
    let reference = evidence_record(&mut e, "green");
    let err = apply_built(&mut e, &completing_actor("A"), "complete-pending", |e| {
        complete_action(
            e,
            "A",
            &id,
            vec![
                ItemEvidence {
                    item: 0,
                    judge: Judge::Hctl2Tool,
                    channel: "unmediated".into(),
                    references: vec![reference],
                    producer: None,
                    generation: None,
                },
                ItemEvidence {
                    item: 1,
                    judge: human_judge(),
                    channel: "unmediated".into(),
                    references: vec![],
                    producer: None,
                    generation: None,
                },
            ],
        )
    })
    .unwrap_err();
    assert_eq!(err.code, "RUN_ACTIVE");
    assert_eq!(err.recovery_action, "wait_for_run_reducer");
    let (_, t) = task(&e.store, "A", &id).unwrap();
    assert_eq!(t.lifecycle, "open");
}

#[test]
fn completion_writes_receipt_and_lifecycle_together_and_replays_the_same_key() {
    let mut e = Env::new();
    let id = ready(&mut e, mixed_contract("A"));
    let reference = evidence_record(&mut e, "green");
    let action = complete_action(
        &e,
        "A",
        &id,
        vec![
            ItemEvidence {
                item: 0,
                judge: Judge::Hctl2Tool,
                channel: "unmediated".into(),
                references: vec![reference.clone()],
                producer: Some("hctl2-tool".into()),
                generation: Some(7),
            },
            ItemEvidence {
                item: 1,
                judge: human_judge(),
                channel: "unmediated".into(),
                references: vec![],
                producer: None,
                generation: None,
            },
        ],
    );
    let actor = completing_actor("A");
    let result = apply_as(&mut e.store, &actor, "complete-A", action.clone()).unwrap();
    assert_eq!(result["lifecycle"], "completed");
    let receipt_id = result["receipt_id"].as_str().unwrap();
    let receipt_key = key(
        Scope::Project("A".into()),
        COMPLETION_RECEIPT_KIND,
        receipt_id,
    );
    let receipt: CompletionReceipt = decode(&required(&e.store, &receipt_key).unwrap()).unwrap();
    assert_eq!(receipt.lifecycle_version, 2);
    assert_eq!(receipt.revision_number, 1);
    assert_eq!(receipt.idempotency_key, "complete-A");
    assert_eq!(receipt.items.len(), 2);
    assert_eq!(receipt.items[0].grade, "mechanical");
    assert_eq!(receipt.items[0].validation_level, "unmediated");
    assert_eq!(receipt.items[0].judge, Judge::Hctl2Tool);
    assert_eq!(receipt.items[0].generation, Some(7));
    assert_eq!(receipt.items[1].judge, human_judge());
    assert_eq!(receipt.items[1].grade, "human");
    let (_, t) = task(&e.store, "A", &id).unwrap();
    assert_eq!(t.lifecycle, "completed");
    assert_eq!(t.lifecycle_version, 2);
    // 同一个键重投：返回原结果，不重复完成。
    let again = apply_as(&mut e.store, &actor, "complete-A", action).unwrap();
    assert_eq!(again, result);
    let (_, t) = task(&e.store, "A", &id).unwrap();
    assert_eq!(t.lifecycle_version, 2);
}

#[test]
fn reopen_keeps_the_receipt_and_advances_the_version() {
    let mut e = Env::new();
    let id = ready(&mut e, mixed_contract("A"));
    let reference = evidence_record(&mut e, "green");
    let actor = completing_actor("A");
    let result = apply_built(&mut e, &actor, "reopen-first-complete-A", |e| {
        complete_action(
            e,
            "A",
            &id,
            vec![
                ItemEvidence {
                    item: 0,
                    judge: Judge::Hctl2Tool,
                    channel: "unmediated".into(),
                    references: vec![reference],
                    producer: None,
                    generation: None,
                },
                ItemEvidence {
                    item: 1,
                    judge: human_judge(),
                    channel: "unmediated".into(),
                    references: vec![],
                    producer: None,
                    generation: None,
                },
            ],
        )
    })
    .unwrap();
    let receipt_key = key(
        Scope::Project("A".into()),
        COMPLETION_RECEIPT_KIND,
        result["receipt_id"].as_str().unwrap(),
    );
    // 有活动 Run 时取消被拒绝（既有行为，留一条回归）。
    seed_open_run(&mut e, &id);
    let (r, _) = task(&e.store, "A", &id).unwrap();
    let cancel = apply(
        &mut e.store,
        "cancel-A",
        Action::Cancel {
            project_id: "A".into(),
            task_id: id.clone(),
            version: r.version,
        },
    )
    .unwrap_err();
    assert_eq!(cancel.code, "RUN_ACTIVE");
    // 清理这个 Run 之后重开：旧 Receipt 与历史都在，生命周期版本前进。
    let run_key = key(Scope::Project("A".into()), "run", "run-1");
    seed(
        &mut e.store,
        value_record(
            run_key,
            2,
            &json!({"task_id": id, "lifecycle": "completed"}),
        )
        .unwrap(),
    );
    let (r, t) = task(&e.store, "A", &id).unwrap();
    let reopened = apply_as(
        &mut e.store,
        &actor,
        "reopen-A",
        Action::Reopen {
            project_id: "A".into(),
            task_id: id.clone(),
            version: r.version,
            lifecycle_version: t.lifecycle_version,
            revision_number: None,
        },
    )
    .unwrap();
    assert_eq!(reopened["lifecycle"], "open");
    assert_eq!(reopened["lifecycle_version"], 3);
    let (_, t) = task(&e.store, "A", &id).unwrap();
    assert_eq!(t.lifecycle, "open");
    assert_eq!(t.lifecycle_version, 3);
    assert!(required(&e.store, &receipt_key).is_ok());
}

/// 一次「卡片被某账号关闭」的观测：state=closed，closed_by 是指定账号。
fn observe_closed(e: &mut Env, closer: &str) {
    let (r, src) = source(&e.store, &e.rid, &e.sid).unwrap();
    let (_, previous) = latest(&e.store, &src).unwrap();
    let mut card = previous.cards[0].clone();
    card.stage = "closed".into();
    card.remote_revision = "closed-1".into();
    card.raw["state"] = json!("closed");
    card.raw["closed_by"] = json!({"login": closer});
    observe(
        &mut e.store,
        &src,
        Snapshot {
            source: reference(&r),
            observed_at: now(),
            complete: true,
            error: None,
            cards: vec![card],
            stable_groups: Vec::new(),
        },
    )
    .unwrap();
}

fn completion_requests(e: &Env) -> Vec<CompletionRequest> {
    e.store
        .list(COMPLETION_REQUEST_KIND)
        .unwrap()
        .into_iter()
        .filter_map(|r| decode(&r).ok())
        .collect()
}

#[test]
fn provider_done_archives_one_request_only_for_the_mapped_human() {
    let mut e = Env::new();
    let (sr, mut src) = source(&e.store, &e.rid, &e.sid).unwrap();
    src.human_account = Some("human".into());
    src.auto_complete_provider_done = true;
    seed(
        &mut e.store,
        value_record(sr.key.clone(), sr.version + 1, &src).unwrap(),
    );
    e.attach("A", None);
    e.observe("card", false);
    let _ = e.claim("A");
    // 控制面自己的账号写回关闭：只作观测，不产生请求。
    observe_closed(&mut e, &src.platform.account_id);
    assert_eq!(completion_requests(&e).len(), 0);
    // 未知 actor 的关闭：同样只作观测。
    e.observe("card", false);
    observe_closed(&mut e, "someone-else");
    assert_eq!(completion_requests(&e).len(), 0);
    // 归属 human 把卡片从非终态推进到 Done：归档一条命令草稿。
    e.observe("card", false);
    observe_closed(&mut e, "human");
    let requests = completion_requests(&e);
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].provider_actor, "human");
    assert_eq!(requests[0].stage_before, "open");
    assert_eq!(requests[0].stage_after, "closed");
    assert_eq!(requests[0].state, "pending");
    assert_eq!(requests[0].binding.version, Version::State(sr.version + 1));
    // 重复与迟到的投递落到同一条记录。
    observe_closed(&mut e, "human");
    assert_eq!(completion_requests(&e).len(), 1);
}

#[test]
fn provider_evidence_covers_mechanical_items_only() {
    let mut e = Env::new();
    let id = ready(&mut e, mixed_contract("A"));
    let (_, t) = task(&e.store, "A", &id).unwrap();
    let snapshot = evidence_record(&mut e, "observed");
    let actor = completing_actor("A");
    let evidence = provider_evidence(&e.store, &actor, &t, &snapshot, "task_source").unwrap();
    // 只有机械项拿到证据；人判项没有，准入会以 EVIDENCE_REQUIRED 拒。
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].item, 0);
    assert_eq!(evidence[0].channel, "adapter_event");
    assert_eq!(
        evidence[0].judge,
        Judge::Adapter {
            port: "task_source".into()
        }
    );
    assert_eq!(evidence[0].references, vec![snapshot.clone()]);
    let refused = apply_built(&mut e, &actor, "complete-by-provider", |e| {
        Action::Complete {
            project_id: "A".into(),
            task_id: id.clone(),
            version: facts(e, "A", &id).0,
            lifecycle_version: facts(e, "A", &id).1,
            revision_number: facts(e, "A", &id).2,
            acceptance: evidence.clone(),
        }
    })
    .unwrap_err();
    assert_eq!(refused.code, "EVIDENCE_REQUIRED");
    let (_, t) = task(&e.store, "A", &id).unwrap();
    assert_eq!(t.lifecycle, "open");
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
                make_default: true,
                human_account: None,
                auto_complete_provider_done: false,
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
fn identical_observation_preserves_versions_and_an_existing_preview() {
    let mut e = Env::new();
    e.attach("A", None);
    e.observe("card", false);
    let id = e.claim("A");
    let (r, t) = task(&e.store, "A", &id).unwrap();
    let plan = prepare(
        &e.store,
        Input {
            key: "move-stable".into(),
            action: Action::Move {
                project_id: "A".into(),
                task_id: id.clone(),
                version: r.version,
                state_version: t.state_version,
                stage: "closed".into(),
                rank: None,
                relative_source_id: None,
            },
        },
    )
    .unwrap();
    let (_, src) = source(&e.store, &e.rid, &e.sid).unwrap();
    let (snapshot_record, mut snapshot) = latest(&e.store, &src).unwrap();
    // A heartbeat is not a new backend revision, even after the old 60-second TTL.
    snapshot.observed_at += 120;
    let stamp = e.store.read_stamp();
    observe(&mut e.store, &src, snapshot).unwrap();
    assert_eq!(e.store.read_stamp(), stamp);
    assert_eq!(
        latest(&e.store, &src).unwrap().0.version,
        snapshot_record.version
    );
    assert_eq!(task(&e.store, "A", &id).unwrap().0.version, r.version);
    admit(&mut e.store, &actor(), plan).unwrap();
}

#[test]
fn withdraw_unsent_intent_releases_card_without_cancelling_task_but_unknown_stays_reserved() {
    let mut e = Env::new();
    e.attach("A", None);
    e.attach("B", None);
    e.observe("card", false);
    let a = e.claim("A");
    let b = e.claim("B");
    let movement = |s: &Store, project: &str, id: &str| {
        let (r, t) = task(s, project, id).unwrap();
        Action::Move {
            project_id: project.into(),
            task_id: id.into(),
            version: r.version,
            state_version: t.state_version,
            stage: "closed".into(),
            rank: None,
            relative_source_id: None,
        }
    };
    let action = movement(&e.store, "A", &a);
    let result = apply(&mut e.store, "first", action).unwrap();
    let effect = result["effect_id"].as_str().unwrap().to_owned();
    let action = movement(&e.store, "B", &b);
    assert_eq!(
        apply(&mut e.store, "second-blocked", action)
            .unwrap_err()
            .code,
        "EFFECT_CONFLICT"
    );
    apply(
        &mut e.store,
        "withdraw",
        Action::Withdraw {
            effect_id: effect.clone(),
        },
    )
    .unwrap();
    assert_eq!(
        e.store.effect(&effect).unwrap().1,
        store::EffectState::Cancelled
    );
    assert_eq!(task(&e.store, "A", &a).unwrap().1.lifecycle, "open");
    let action = movement(&e.store, "B", &b);
    let result = apply(&mut e.store, "second", action).unwrap();
    let effect = result["effect_id"].as_str().unwrap();
    let withdrawal = prepare(
        &e.store,
        Input {
            key: "withdraw-race".into(),
            action: Action::Withdraw {
                effect_id: effect.into(),
            },
        },
    )
    .unwrap();
    begin(&mut e.store, &actor(), effect).unwrap();
    assert_eq!(
        admit(&mut e.store, &actor(), withdrawal).unwrap_err().code,
        "READBACK_REQUIRED"
    );
    assert_eq!(
        apply(
            &mut e.store,
            "withdraw-unknown",
            Action::Withdraw {
                effect_id: effect.into()
            }
        )
        .unwrap_err()
        .code,
        "READBACK_REQUIRED"
    );
    let action = movement(&e.store, "A", &a);
    assert_eq!(
        apply(&mut e.store, "still-blocked", action)
            .unwrap_err()
            .code,
        "EFFECT_CONFLICT"
    );
}

#[test]
fn another_card_changes_without_reversioning_this_task_or_breaking_its_contract_origin() {
    let mut e = Env::new();
    e.attach("A", None);
    e.observe("card", false);
    let id = e.claim("A");
    let (r, t) = task(&e.store, "A", &id).unwrap();
    let (_, src) = source(&e.store, &e.rid, &e.sid).unwrap();
    let (_, mut snap) = latest(&e.store, &src).unwrap();
    let mut other = snap.cards[0].clone();
    other.entity.immutable_external_entity_id = "other".into();
    other.number = 2;
    snap.cards.push(other);
    observe(&mut e.store, &src, snap).unwrap();
    assert_eq!(task(&e.store, "A", &id).unwrap().0.version, r.version);
    let mut adoption = e.local_contract("A");
    adoption.origin = ContractOrigin::Backend {
        snapshot: t.snapshot.unwrap(),
        state_version: t.state_version,
    };
    apply(
        &mut e.store,
        "adopt-still-current",
        Action::Adopt {
            project_id: "A".into(),
            project_version: 1,
            task_id: id,
            version: r.version,
            adoption,
        },
    )
    .unwrap();
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
        prepare(
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
    let (r, t) = task(&e.store, "A", &id).unwrap();
    let result = apply(
        &mut e.store,
        "comment",
        Action::Update {
            project_id: "A".into(),
            task_id: id.clone(),
            version: r.version,
            state_version: t.state_version,
            fields: Fields {
                title: None,
                body: None,
                comment: Some("status update".into()),
            },
        },
    )
    .unwrap();
    let effect_id = result["effect_id"].as_str().unwrap();
    let (intent, _) = begin(&mut e.store, &actor(), effect_id).unwrap();
    let (_, mut snapshot) = latest(&e.store, &src).unwrap();
    let mut observed = snapshot.cards[0].clone();
    observed.comments = vec![json!({"id":8,"body":intent.input["write"]["fields"]["comment"]})];
    snapshot.cards[0] = observed.clone();
    // Observation may win the race against delivery confirmation.
    observe(&mut e.store, &src, snapshot.clone()).unwrap();
    assert!(
        task(&e.store, "A", &id)
            .unwrap()
            .1
            .pending_contract
            .is_some()
    );
    confirm(&mut e.store, effect_id, &observed).unwrap();
    observe(&mut e.store, &src, snapshot.clone()).unwrap();
    assert!(
        task(&e.store, "A", &id)
            .unwrap()
            .1
            .pending_contract
            .is_none()
    );
    // A copied marker is not proof of an HCTL write. Keep this distinct native comment.
    let mut copy = snapshot.cards[0].comments[0].clone();
    copy["id"] = json!(9);
    snapshot.cards[0].comments.push(copy);
    observe(&mut e.store, &src, snapshot).unwrap();
    assert!(
        task(&e.store, "A", &id)
            .unwrap()
            .1
            .pending_contract
            .is_some()
    );
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
