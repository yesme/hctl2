use foundation::{bytes_sha256, canonical_json_sha256};
use serde_json::{Value, json};
use store::{
    ActorSource, Command, EffectIntent, ObjectKey, Record, RecordData, Reference, Scope, Store,
    TrustedActor, Version,
};

use crate::*;

pub fn owner(
    actor: &TrustedActor,
    scopes: impl IntoIterator<Item = Scope>,
) -> Result<TrustedActor> {
    if actor.0.source != ActorSource::DirectClient
        || !actor.0.permission_scope.contains(&Scope::Control)
    {
        return Err(reject(
            "PERMISSION_DENIED",
            "Task commands require the authenticated control owner",
            "request_authorization",
        ));
    }
    let mut actor = actor.0.clone();
    for scope in scopes {
        if !actor.permission_scope.contains(&scope) {
            actor.permission_scope.push(scope);
        }
    }
    Ok(TrustedActor(actor))
}

pub fn command_key(store: &Store, idem: &str) -> ObjectKey {
    key(
        Scope::Control,
        "task_command",
        &bytes_sha256(format!("{}:{idem}", store.control_id()).as_bytes()),
    )
}

pub fn replay(store: &Store, input: &Input) -> Result<Option<Plan>> {
    let Some(record) = store.get(&command_key(store, &input.key))? else {
        return Ok(None);
    };
    let plan: Plan = decode(&record)?;
    if canonical_json_sha256(&serde_json::to_value(input)?)?
        != canonical_json_sha256(&serde_json::to_value(&plan.input)?)?
    {
        return Err(reject(
            "IDEMPOTENCY_CONFLICT",
            "key has different Task input",
            "use_original_command",
        ));
    }
    Ok(Some(plan))
}

impl Plan {
    fn check(&mut self, record: &Record) {
        if !self.checks.iter().any(|c| c.key == record.key) {
            self.checks.push(Check {
                key: record.key.clone(),
                version: Some(record.version),
            });
        }
    }
    fn put<T: serde::Serialize>(&mut self, store: &Store, key: ObjectKey, data: &T) -> Result<()> {
        let old = store.get(&key)?;
        self.checks.push(Check {
            key: key.clone(),
            version: old.as_ref().map(|r| r.version),
        });
        self.records
            .push(value_record(key, old.map_or(1, |r| r.version + 1), data)?);
        Ok(())
    }
    pub(crate) fn put_task(
        &mut self,
        store: &Store,
        task: &Task,
        source_ref: &Reference,
    ) -> Result<()> {
        let scope = Scope::Project(task.project_id.clone());
        self.put(store, key(scope.clone(), "task_state", &task.id), task)?;
        let identity = key(scope, "task", &task.id);
        let old = store.get(&identity)?;
        self.checks.push(Check {
            key: identity.clone(),
            version: old.as_ref().map(|r| r.version),
        });
        let data = RecordData::Task {
            entity: task.entity.clone(),
            source: source_ref.clone(),
        };
        self.records.push(Record {
            key: identity,
            version: old.map_or(1, |r| r.version + 1),
            revision_digest: canonical_json_sha256(&serde_json::to_value(&data)?)?,
            data,
            sources: vec![],
            materials: vec![],
        });
        self.task_key = Some(key(
            Scope::Project(task.project_id.clone()),
            "task_state",
            &task.id,
        ));
        self.result = json!({"project_id":task.project_id,"task_id":task.id});
        Ok(())
    }
}

fn project(store: &Store, plan: &mut Plan, id: &str, expected: Option<i64>) -> Result<String> {
    let r = required(store, &key(Scope::Project(id.into()), "project", id))?;
    if expected.is_some_and(|v| v != r.version) {
        return Err(stale());
    }
    let RecordData::Project {
        repo_id, archived, ..
    } = &r.data
    else {
        return Err(stale());
    };
    if *archived {
        return Err(reject(
            "PROJECT_ARCHIVED",
            "Project is read-only",
            "restore_project",
        ));
    }
    repo::require_active(store, repo_id)?;
    plan.check(&r);
    Ok(repo_id.clone())
}

pub fn stale() -> StoreError {
    reject(
        "VERSION_CONFLICT",
        "preview input changed",
        "preview_current_version",
    )
}

fn connected(
    store: &Store,
    plan: &mut Plan,
    project_id: &str,
    repo_id: &str,
    id: &str,
) -> Result<(Record, Source)> {
    let pair = approved_source(store, plan, project_id, repo_id, id)?;
    if !pair.1.active {
        return Err(reject(
            "SOURCE_DISABLED",
            "Task home is disabled",
            "reconnect_source",
        ));
    }
    Ok(pair)
}

fn approved_source(
    store: &Store,
    plan: &mut Plan,
    project_id: &str,
    repo_id: &str,
    id: &str,
) -> Result<(Record, Source)> {
    let r = required(
        store,
        &key(
            Scope::Project(project_id.into()),
            "task_source_reference",
            id,
        ),
    )?;
    let approved: SourceReference = decode(&r)?;
    let (sr, source) = source(store, repo_id, id)?;
    if approved.approved_scope != source.board_scope_stable_id || approved.source.key != sr.key {
        return Err(reject(
            "SOURCE_SCOPE",
            "source reference does not authorize this board",
            "attach_source",
        ));
    }
    plan.check(&r);
    plan.check(&sr);
    Ok((sr, source))
}

pub fn latest(store: &Store, source: &Source) -> Result<(Record, Snapshot)> {
    let (binding, _) = crate::source(store, &source.repo_id, &source.id)?;
    let r = required(
        store,
        &key(
            Scope::Repo(source.repo_id.clone()),
            "task_snapshot",
            &source.id,
        ),
    )?;
    let snap: Snapshot = decode(&r)?;
    if !source.active
        || snap.source != reference(&binding)
        || !snap.complete
        || snap.error.is_some()
    {
        return Err(reject(
            "READBACK_REQUIRED",
            "source unavailable or incomplete; the transport must read before admission",
            "refresh_source",
        ));
    }
    Ok((r, snap))
}

pub(crate) fn snapshot_at(store: &Store, reference: &Reference) -> Result<Snapshot> {
    let current = required(store, &reference.key)?;
    if crate::reference(&current) == *reference {
        return decode(&current);
    }
    let record = store
        .versions(&reference.key)?
        .into_iter()
        .find(|r| crate::reference(r) == *reference)
        .ok_or_else(stale)?;
    decode(&record)
}

pub(crate) fn card<'a>(snapshot: &'a Snapshot, entity_id: &str) -> Result<&'a Card> {
    snapshot
        .cards
        .iter()
        .find(|c| c.entity.immutable_external_entity_id == entity_id && !c.tombstone)
        .ok_or_else(|| {
            reject(
                "CARD_UNAVAILABLE",
                "original card not observed",
                "refresh_source",
            )
        })
}

fn check_contract(
    store: &Store,
    plan: &mut Plan,
    project_id: &str,
    task: Option<&Task>,
    adoption: &Adoption,
) -> Result<()> {
    let c = &adoption.contract;
    if c.scope.trim().is_empty()
        || c.expected_outcome.trim().is_empty()
        || c.acceptance.is_empty()
        || c.acceptance.iter().any(|a| a.text.trim().is_empty())
    {
        return Err(reject(
            "INVALID_CONTRACT",
            "scope, outcome and graded acceptance items required",
            "edit_proposal",
        ));
    }
    match &adoption.origin {
        ContractOrigin::Local {
            reference: origin,
            proposal_digest,
        } => {
            if origin.key.scope != Scope::Project(project_id.into())
                || !["project", "room", "memo", "artifact", "request"]
                    .contains(&origin.key.kind.as_str())
            {
                return Err(reject(
                    "PROJECT_MISMATCH",
                    "proposal source belongs to another Project or is not a supported source",
                    "choose_same_project",
                ));
            }
            let r = required(store, &origin.key)?;
            if origin.version != Version::State(r.version)
                && origin.version != Version::Revision(r.revision_digest.clone())
            {
                return Err(stale());
            }
            if *proposal_digest != canonical_json_sha256(&serde_json::to_value(c)?)? {
                return Err(reject(
                    "PROPOSAL_DIGEST",
                    "proposal digest does not match contract",
                    "edit_proposal",
                ));
            }
            plan.check(&r);
        }
        ContractOrigin::Backend {
            snapshot,
            state_version,
        } => {
            let task = task.ok_or_else(|| {
                reject(
                    "INVALID_CONTRACT",
                    "new card has no backend Snapshot yet",
                    "use_local_proposal",
                )
            })?;
            if task.snapshot.as_ref() != Some(snapshot) || task.state_version != *state_version {
                return Err(stale());
            }
            let (_, src) = source(store, &task.repo_id, &task.source_id)?;
            let (r, snap) = latest(store, &src)?;
            if r.key != snapshot.key {
                return Err(stale());
            }
            let frozen = snapshot_at(store, snapshot)?;
            let entity = &task
                .entity
                .as_ref()
                .ok_or_else(stale)?
                .immutable_external_entity_id;
            if frozen.source != snap.source
                || !frozen.complete
                || frozen.error.is_some()
                || serde_json::to_value(card(&frozen, entity)?)?
                    != serde_json::to_value(card(&snap, entity)?)?
            {
                return Err(stale());
            }
            plan.check(&r);
        }
    }
    plan.adoption = Some(adoption.clone());
    Ok(())
}

/// 完成要读契约正文：材料读取需要可信身份，所以调用方必须带上 actor。
fn contract_of(store: &Store, actor: &TrustedActor, t: &Task) -> Result<(Contract, Revision)> {
    let revision = t.revision.clone().ok_or_else(|| {
        reject(
            "CONTRACT_REQUIRED",
            "adopt a contract before completing this Task",
            "adopt_contract",
        )
    })?;
    let bytes = store.read_material(actor, &revision.material)?;
    let contract: Contract = serde_json::from_slice(&bytes).map_err(|_| {
        reject(
            "INVALID_RECORD",
            "Task Revision material is not a contract",
            "inspect_storage",
        )
    })?;
    Ok((contract, revision))
}

/// 证据通道的强弱：`narrated` 最弱，`adapter_event` 次之，`unmediated` 最强。
fn channel_rank(channel: &str) -> Option<u8> {
    match channel {
        "narrated" => Some(0),
        "adapter_event" => Some(1),
        "unmediated" => Some(2),
        _ => None,
    }
}

/// 机械项的集成证据：契约事先接受的 Integration Receipt，逐字段核过才算数。
fn integration_evidence(
    store: &Store,
    t: &Task,
    reference: &Reference,
) -> Result<Option<(String, Reference)>> {
    let Some(record) = store.get(&reference.key)? else {
        return Ok(None);
    };
    if record.key.scope != Scope::Repo(t.repo_id.clone())
        || record.key.kind != repo::integration::RECEIPT_KIND
        || reference.version != Version::State(record.version)
    {
        return Ok(None);
    }
    let receipt = repo::integration::receipt(store, &t.repo_id, record.key.id.as_str())?;
    let channel = if receipt.evidence_level == "hctl2-tool" {
        "unmediated"
    } else {
        "adapter_event"
    };
    // 源版本、目标头与「指回意图」的来源都要在，缺一项不算核过。
    if receipt.source.change_set_revision_id.trim().is_empty()
        || receipt.target_head_after.trim().is_empty()
        || record.sources.is_empty()
    {
        return Ok(None);
    }
    Ok(Some((channel.into(), reference.clone())))
}

/// 供应端 Done 请求能提供的证据：只覆盖契约里的机械项，用这次观测作旁路证据；
/// `gate` 与 `human` 项一律不给证据——准入会按各自的规则拒，Task 保持开放。
pub fn provider_evidence(
    store: &Store,
    actor: &TrustedActor,
    t: &Task,
    snapshot: &Reference,
    port: &str,
) -> Result<Vec<ItemEvidence>> {
    let (contract, _) = contract_of(store, actor, t)?;
    Ok(contract
        .acceptance
        .iter()
        .enumerate()
        .filter(|(_, item)| matches!(item.grade, Grade::Mechanical))
        .map(|(index, _)| ItemEvidence {
            item: index,
            judge: Judge::Adapter { port: port.into() },
            channel: "adapter_event".into(),
            references: vec![snapshot.clone()],
            producer: Some(port.into()),
            generation: None,
        })
        .collect())
}

/// 逐项核对：判定者、校验等级、证据通道与集成凭证都要对得上，缺一项就拒绝。
fn completion_items(
    store: &Store,
    actor: &TrustedActor,
    t: &Task,
    contract: &Contract,
    submitted: &[ItemEvidence],
) -> Result<Vec<ItemOutcome>> {
    if contract.acceptance.is_empty() {
        return Err(reject(
            "INVALID_CONTRACT",
            "contract has no acceptance items",
            "adopt_contract",
        ));
    }
    let mut outcomes = Vec::new();
    for (index, item) in contract.acceptance.iter().enumerate() {
        let candidate = submitted.iter().find(|e| e.item == index).ok_or_else(|| {
            reject(
                "EVIDENCE_REQUIRED",
                format!("acceptance item {index} has no evidence"),
                "supply_evidence",
            )
        })?;
        let channel = channel_rank(&candidate.channel).ok_or_else(|| {
            reject(
                "EVIDENCE_CHANNEL",
                "unknown evidence channel",
                "correct_input",
            )
        })?;
        match item.grade {
            Grade::Mechanical => {
                if channel < 1 {
                    return Err(reject(
                        "EVIDENCE_TOO_WEAK",
                        "mechanical items accept only unmediated or adapter_event evidence",
                        "supply_evidence",
                    ));
                }
            }
            Grade::Gate => {
                if !matches!(candidate.judge, Judge::Gate { .. }) || candidate.references.is_empty()
                {
                    return Err(reject(
                        "GATE_EVIDENCE_REQUIRED",
                        "gate items accept only a Gate Receipt verdict",
                        "supply_verdict",
                    ));
                }
            }
            Grade::Human => {
                let Judge::Human { actor: who } = &candidate.judge else {
                    return Err(reject(
                        "HUMAN_JUDGEMENT_REQUIRED",
                        "human items accept only an explicit human verdict",
                        "supply_judgement",
                    ));
                };
                if who != &actor.0.principal {
                    return Err(reject(
                        "HUMAN_JUDGEMENT_REQUIRED",
                        "the judging human is not the submitting actor",
                        "supply_judgement",
                    ));
                }
            }
        }
        if let Some(minimum) = item
            .evidence
            .as_ref()
            .and_then(|r| r.min_channel.as_deref())
            .and_then(channel_rank)
            && channel < minimum
        {
            return Err(reject(
                "EVIDENCE_TOO_WEAK",
                format!("acceptance item {index} requires a higher evidence channel"),
                "supply_evidence",
            ));
        }
        let mut validation_level = candidate.channel.clone();
        let mut references = candidate.references.clone();
        for reference in &candidate.references {
            let record = store.get(&reference.key)?.ok_or_else(|| {
                reject(
                    "EVIDENCE_MISSING",
                    "referenced evidence does not exist",
                    "supply_evidence",
                )
            })?;
            if reference.version != Version::State(record.version) {
                return Err(stale());
            }
        }
        if item.evidence.as_ref().and_then(|r| r.accept.as_deref()) == Some("integration_receipt") {
            let mut verified = None;
            for reference in &candidate.references {
                if let Some((channel, reference)) = integration_evidence(store, t, reference)? {
                    verified = Some((channel, reference));
                    break;
                }
            }
            let Some((channel, reference)) = verified else {
                return Err(reject(
                    "INTEGRATION_RECEIPT_REQUIRED",
                    "this item needs the Repo module's Integration Receipt",
                    "integrate_first",
                ));
            };
            validation_level = channel;
            references = vec![reference];
        }
        outcomes.push(ItemOutcome {
            item: index,
            text: item.text.clone(),
            text_digest: canonical_json_sha256(&serde_json::to_value(item)?)?,
            grade: serde_json::to_value(&item.grade)?
                .as_str()
                .unwrap_or("unknown")
                .to_owned(),
            outcome: "passed".into(),
            validation_level,
            judge: candidate.judge.clone(),
            references,
            producer: candidate.producer.clone(),
            generation: candidate.generation,
            source_snapshot: t.snapshot.clone(),
            source_head: None,
        });
    }
    Ok(outcomes)
}

pub(crate) fn new_task(
    store: &Store,
    project: &str,
    source: &Source,
    identity: &str,
    title: &str,
) -> Task {
    Task {
        id: bytes_sha256(format!("{}:{project}:{identity}", store.control_id()).as_bytes()),
        project_id: project.into(),
        source_id: source.id.clone(),
        repo_id: source.repo_id.clone(),
        entity: None,
        number: None,
        title: title.into(),
        lifecycle: "open".into(),
        lifecycle_version: 1,
        archived: false,
        revision: None,
        snapshot: None,
        state_version: 0,
        needs_attention: false,
        pending_contract: None,
        run_occupancy: None,
    }
}

fn add_effect(
    store: &Store,
    plan: &mut Plan,
    task: &Task,
    src: &Record,
    operation: &str,
    input: Value,
) -> Result<()> {
    let id = format!("task:{}", command_key(store, &plan.input.key).id);
    let input = json!({"project_id":task.project_id,"task_id":task.id,"source_id":task.source_id,"repo_id":task.repo_id,"write":input});
    let resource = if let Some(entity) = &task.entity {
        canonical_json_sha256(&serde_json::to_value(entity)?)?
    } else {
        id.clone()
    };
    plan.effects.push(EffectIntent {
        intent_id: id.clone(),
        owner: Reference {
            key: key(Scope::Project(task.project_id.clone()), "task", &task.id),
            version: Version::State(
                store
                    .get(&key(
                        Scope::Project(task.project_id.clone()),
                        "task",
                        &task.id,
                    ))?
                    .map_or(1, |r| r.version),
            ),
        },
        binding: reference(src),
        operation: operation.into(),
        target: resource.clone(),
        conflict_scope: format!("issue:{resource}"),
        permission_scope: Scope::Project(task.project_id.clone()),
        input_digest: Command::digest_input(operation, &input)?,
        input,
        idempotency_key: id.clone(),
    });
    plan.result["effect_id"] = json!(id);
    Ok(())
}

pub fn prepare(store: &Store, input: Input) -> Result<Plan> {
    prepare_inner(store, input, None, None)
}

/// 与 [`prepare`] 相同，但带上提交者的可信身份：完成要读契约正文并核对 human 判定者。
pub fn prepare_as(store: &Store, actor: &TrustedActor, input: Input) -> Result<Plan> {
    prepare_inner(store, input, None, Some(actor))
}

pub fn request_blockers(
    store: &Store,
    project: &str,
    task: &str,
) -> Result<Vec<(Record, RequestBlocker)>> {
    store
        .list("task_request_blocker")?
        .into_iter()
        .filter(|r| r.key.scope == Scope::Project(project.into()))
        .map(|r| Ok((r.clone(), decode::<RequestBlocker>(&r)?)))
        .collect::<Result<Vec<_>>>()
        .map(|items| {
            items
                .into_iter()
                .filter(|(_, b)| b.owner.key.id == task)
                .collect()
        })
}

/// Project resolves a Task-owned blocker through the existing adoption reducer.
/// It supplies the exact waiting record, not an arbitrary bypass flag from a client.
pub fn prepare_request_adoption(store: &Store, input: Input, blocker: &Record) -> Result<Plan> {
    let waiting: RequestBlocker = decode(blocker)?;
    let Action::Adopt {
        project_id,
        task_id,
        version,
        ..
    } = &input.action
    else {
        return Err(reject(
            "INVALID_INPUT",
            "Request action is not contract adoption",
            "correct_input",
        ));
    };
    if blocker.key.kind != "task_request_blocker"
        || blocker.key.scope != Scope::Project(project_id.clone())
        || store.get(&blocker.key)?.as_ref().map(|r| r.version) != Some(blocker.version)
        || waiting.owner.key != key(blocker.key.scope.clone(), "task_state", task_id)
        || waiting.owner.version != Version::State(*version)
        || !waiting.waiting
        || waiting.delivery.is_some()
        || waiting.outcome.is_some()
    {
        return Err(stale());
    }
    prepare_inner(store, input, Some(&waiting.request_id), None)
}

fn prepare_inner(
    store: &Store,
    input: Input,
    answering: Option<&str>,
    actor: Option<&TrustedActor>,
) -> Result<Plan> {
    if input.key.trim().is_empty() {
        return Err(reject(
            "INVALID_INPUT",
            "command key required",
            "correct_input",
        ));
    }
    if let Some(old) = replay(store, &input)? {
        return Ok(old);
    }
    let mut p = Plan {
        input: input.clone(),
        checks: vec![],
        records: vec![],
        effects: vec![],
        cancel_effects: vec![],
        result: json!({}),
        adoption: None,
        task_key: None,
    };
    match &input.action {
        Action::Connect {
            repo_id,
            candidate_id,
            consent,
            make_default,
            human_account,
            auto_complete_provider_done,
        } => {
            if !consent {
                return Err(reject(
                    "CONSENT_REQUIRED",
                    "connecting source needs explicit consent",
                    "confirm_source",
                ));
            }
            let reg = repo::require_active(store, repo_id)?;
            let candidate = reg
                .sources
                .iter()
                .find(|c| c.id == *candidate_id && c.available && c.can_claim)
                .ok_or_else(|| {
                    reject(
                        "SOURCE_UNAVAILABLE",
                        "candidate is not available",
                        "inspect_repo_candidates",
                    )
                })?
                .clone();
            if *make_default && !candidate.can_default() {
                return Err(reject(
                    "SOURCE_READ_ONLY",
                    "default needs create and writeback",
                    "choose_capable_source",
                ));
            }
            let platform = reg.observed.ok_or_else(|| {
                reject(
                    "SOURCE_UNAVAILABLE",
                    "platform not confirmed",
                    "finish_registration",
                )
            })?;
            p.check(&required(store, &repo::key(repo_id))?);
            let id = bytes_sha256(
                format!(
                    "{repo_id}:{}:{}:{}:{}",
                    candidate.provider, platform.instance, platform.account_id, platform.stable_id
                )
                .as_bytes(),
            );
            let skey = key(Scope::Repo(repo_id.clone()), "task_source", &id);
            if let Some(existing) = store.get(&skey)? {
                let mut src: Source = decode(&existing)?;
                if !src.active {
                    return Err(reject(
                        "SOURCE_DISABLED",
                        "existing source disabled",
                        "reconnect_source",
                    ));
                }
                if src.human_account != *human_account
                    || src.auto_complete_provider_done != *auto_complete_provider_done
                {
                    src.human_account = human_account.clone();
                    src.auto_complete_provider_done = *auto_complete_provider_done;
                    p.put(store, skey.clone(), &src)?;
                } else {
                    p.check(&existing);
                }
            } else {
                let src = Source {
                    id: id.clone(),
                    repo_id: repo_id.clone(),
                    port_kind: "task_source".into(),
                    capabilities: Capabilities {
                        create: candidate.create,
                        field_writeback: candidate.field_writeback,
                        conditional_write: candidate.provider == "gitea_issues",
                        conditional_fields: if candidate.provider == "gitea_issues" {
                            vec!["body".into()]
                        } else {
                            vec![]
                        },
                        placement: false,
                        delete: candidate.field_writeback,
                    },
                    board_scope_stable_id: platform.stable_id.clone(),
                    platform,
                    candidate,
                    binding_revision: 1,
                    active: true,
                    human_account: human_account.clone(),
                    auto_complete_provider_done: *auto_complete_provider_done,
                };
                p.put(store, skey, &src)?;
            }
            let default_key = key(Scope::Repo(repo_id.clone()), "task_default_source", repo_id);
            if *make_default
                || (reg.prepared.request.default_source.as_ref() == Some(candidate_id)
                    && store.get(&default_key)?.is_none())
            {
                if !reg
                    .sources
                    .iter()
                    .any(|c| c.id == *candidate_id && c.can_default())
                {
                    return Err(reject(
                        "SOURCE_READ_ONLY",
                        "confirmed default lost write capability",
                        "choose_capable_source",
                    ));
                }
                p.put(
                    store,
                    default_key,
                    &json!({"source_id":id,"candidate_id":candidate_id}),
                )?;
            }
            p.result = json!({"source_id":id,"repo_id":repo_id});
        }
        Action::SetActive {
            repo_id,
            source_id,
            active,
            version,
        } => {
            let (r, mut src) = source(store, repo_id, source_id)?;
            if r.version != *version {
                return Err(stale());
            }
            src.active = *active;
            p.put(store, r.key, &src)?;
            for (r, mut t) in tasks(store)?
                .into_iter()
                .filter(|(_, t)| t.source_id == *source_id)
            {
                t.needs_attention = true;
                p.put(store, r.key, &t)?;
            }
            p.result = json!({"source_id":source_id,"active":active});
        }
        Action::Attach {
            project_id,
            project_version,
            source_id,
            approved_scope,
            group,
            consent,
        } => {
            if !consent {
                return Err(reject(
                    "CONSENT_REQUIRED",
                    "Project source selection needs consent",
                    "confirm_source",
                ));
            }
            let rid = project(store, &mut p, project_id, Some(*project_version))?;
            let (sr, src) = source(store, &rid, source_id)?;
            if !src.active || *approved_scope != src.board_scope_stable_id {
                return Err(reject(
                    "SOURCE_SCOPE",
                    "unavailable or unauthorized board",
                    "choose_source_scope",
                ));
            }
            p.check(&sr);
            if let Some(group) = group {
                let (r, snap) = latest(store, &src)?;
                if !snap.stable_groups.contains(group) {
                    return Err(reject(
                        "GROUP_UNSTABLE",
                        "group anchor cannot be stably read",
                        "choose_native_group",
                    ));
                }
                p.check(&r);
            }
            let k = key(
                Scope::Project(project_id.clone()),
                "task_source_reference",
                source_id,
            );
            if let Some(old) = store.get(&k)? {
                let old: SourceReference = decode(&old)?;
                if old.group != *group {
                    return Err(reject(
                        "GROUP_IMMUTABLE",
                        "reconnecting does not replace approved group",
                        "use_original_group",
                    ));
                }
            }
            p.put(
                store,
                k,
                &SourceReference {
                    project_id: project_id.clone(),
                    source: reference(&sr),
                    approved_scope: approved_scope.clone(),
                    group: group.clone(),
                },
            )?;
            p.result = json!({"project_id":project_id,"source_id":source_id});
        }
        Action::Claim {
            project_id,
            project_version,
            source_id,
            entity_id,
        } => {
            let rid = project(store, &mut p, project_id, Some(*project_version))?;
            let (sr, src) = connected(store, &mut p, project_id, &rid, source_id)?;
            let (r, snap) = latest(store, &src)?;
            let c = card(&snap, entity_id)?;
            p.check(&r);
            if let Some((_, t)) = tasks(store)?
                .into_iter()
                .find(|(_, t)| t.project_id == *project_id && t.entity.as_ref() == Some(&c.entity))
            {
                p.result = json!({"project_id":project_id,"task_id":t.id});
            } else {
                if pending_creation(store, project_id, &src.id, c)?.is_some() {
                    return Err(reject(
                        "CREATION_PENDING",
                        "card belongs to an unresolved Task creation",
                        "resume_original_creation",
                    ));
                }
                let mut t = new_task(
                    store,
                    project_id,
                    &src,
                    &canonical_json_sha256(&serde_json::to_value(&c.entity)?)?,
                    &c.title,
                );
                t.entity = Some(c.entity.clone());
                t.number = Some(c.number);
                t.snapshot = Some(reference(&r));
                t.state_version = 1;
                p.put_task(store, &t, &reference(&sr))?;
            }
        }
        Action::Create {
            project_id,
            project_version,
            source_id,
            title,
            body,
            adoption,
        } => {
            let rid = project(store, &mut p, project_id, Some(*project_version))?;
            let (sr, src) = connected(store, &mut p, project_id, &rid, source_id)?;
            if !src.capabilities.create || title.trim().is_empty() {
                return Err(reject(
                    "CREATE_UNAVAILABLE",
                    "explicit writable source and title required",
                    "choose_capable_source",
                ));
            }
            if let Some(a) = adoption {
                check_contract(store, &mut p, project_id, None, a)?;
            }
            let mut t = new_task(
                store,
                project_id,
                &src,
                &format!("create:{}", input.key),
                title,
            );
            t.needs_attention = true;
            p.put_task(store, &t, &reference(&sr))?;
            let marker = format!(
                "<!-- hctl2:{}:{}:{} -->",
                store.control_id(),
                t.id,
                canonical_json_sha256(&json!({"title":title,"body":body}))?
            );
            add_effect(
                store,
                &mut p,
                &t,
                &sr,
                "task.create",
                json!({"title":title,"body":format!("{body}\n\n{marker}"),"marker":marker}),
            )?;
        }
        Action::Adopt {
            project_id,
            project_version,
            task_id,
            version,
            adoption,
        } => {
            project(store, &mut p, project_id, Some(*project_version))?;
            let (r, t) = task(store, project_id, task_id)?;
            if *version != r.version {
                return Err(stale());
            }
            for (_, blocker) in request_blockers(store, project_id, task_id)? {
                if blocker.owner == reference(&r)
                    && blocker.waiting
                    && blocker.outcome.is_none()
                    && answering != Some(blocker.request_id.as_str())
                {
                    return Err(reject(
                        "REQUEST_REQUIRED",
                        "answer the existing contract Request",
                        "resolve_request",
                    ));
                }
            }
            check_contract(store, &mut p, project_id, Some(&t), adoption)?;
            let (_, src) = source(store, &t.repo_id, &t.source_id)?;
            let (sr, _) = approved_source(store, &mut p, project_id, &src.repo_id, &src.id)?;
            p.put_task(store, &t, &reference(&sr))?;
        }
        Action::Update {
            project_id,
            task_id,
            version,
            state_version,
            ..
        }
        | Action::Move {
            project_id,
            task_id,
            version,
            state_version,
            ..
        } => {
            let rid = project(store, &mut p, project_id, None)?;
            let (r, t) = task(store, project_id, task_id)?;
            if r.version != *version || t.state_version != *state_version {
                return Err(stale());
            }
            let (sr, src) = connected(store, &mut p, project_id, &rid, &t.source_id)?;
            if !src.capabilities.field_writeback {
                return Err(reject(
                    "SOURCE_READ_ONLY",
                    "source cannot write fields",
                    "choose_writable_binding",
                ));
            }
            let (snap_r, snap) = latest(store, &src)?;
            let c = card(
                &snap,
                &t.entity
                    .as_ref()
                    .ok_or_else(stale)?
                    .immutable_external_entity_id,
            )?;
            p.check(&r);
            p.check(&snap_r);
            let write = match &input.action {
                Action::Update { fields, .. } => {
                    if fields.title.as_ref().is_some_and(|s| s.trim().is_empty())
                        || (fields.title.is_none()
                            && fields.body.is_none()
                            && fields.comment.is_none())
                    {
                        return Err(reject(
                            "INVALID_INPUT",
                            "nonempty update required",
                            "correct_input",
                        ));
                    }
                    if fields.comment.is_some() && (fields.title.is_some() || fields.body.is_some())
                    {
                        return Err(reject(
                            "INVALID_INPUT",
                            "append comment and edit fields are separate effects",
                            "split_commands",
                        ));
                    }
                    json!({"title":fields.title,"body":fields.body,"comment":fields.comment.as_ref().map(|s|format!("{s}\n\n<!-- hctl2:{}:{}:{} -->",store.control_id(),t.id,command_key(store,&input.key).id))})
                }
                Action::Move {
                    stage,
                    rank,
                    relative_source_id,
                    ..
                } => {
                    if relative_source_id
                        .as_ref()
                        .is_some_and(|s| s != &t.source_id)
                    {
                        return Err(reject(
                            "CROSS_SOURCE_MOVE",
                            "relative move across sources is unsupported",
                            "move_within_source",
                        ));
                    }
                    if rank.is_some() || !["open", "closed"].contains(&stage.as_str()) {
                        return Err(reject(
                            "PLACEMENT_UNSUPPORTED",
                            "issues binding exposes state only, not native board ordering",
                            "use_provider_client",
                        ));
                    }
                    json!({"state":stage})
                }
                _ => unreachable!(),
            };
            p.result = json!({"project_id":project_id,"task_id":task_id});
            add_effect(
                store,
                &mut p,
                &t,
                &sr,
                "task.update",
                json!({"fields":write,"card":c}),
            )?;
        }
        Action::Cancel {
            project_id,
            task_id,
            version,
        } => {
            project(store, &mut p, project_id, None)?;
            let (r, mut t) = task(store, project_id, task_id)?;
            if *version != r.version {
                return Err(stale());
            }
            if t.run_occupancy.is_some() || !active_runs(store, &t)?.is_empty() {
                return Err(reject(
                    "RUN_ACTIVE",
                    "cancel does not stop Run",
                    "finish_run_first",
                ));
            }
            if t.lifecycle != "open" {
                return Err(reject("TASK_TERMINAL", "Task is not open", "inspect_task"));
            }
            for id in store.pending_effects()? {
                let (effect, state) = store.effect(&id)?;
                if effect.owner.key == key(Scope::Project(project_id.clone()), "task", task_id) {
                    if state == store::EffectState::Pending {
                        p.cancel_effects.push(id);
                    } else {
                        t.needs_attention = true;
                    }
                }
            }
            t.lifecycle = "cancelled".into();
            t.lifecycle_version += 1;
            t.archived = true;
            let (sr, _) = source(store, &t.repo_id, &t.source_id)?;
            p.put_task(store, &t, &reference(&sr))?;
        }
        Action::Complete {
            project_id,
            task_id,
            version,
            lifecycle_version,
            revision_number,
            acceptance,
        } => {
            let actor = actor.ok_or_else(|| {
                reject(
                    "PERMISSION_DENIED",
                    "completion needs the submitting human actor",
                    "retry_with_identity",
                )
            })?;
            project(store, &mut p, project_id, None)?;
            let (r, mut t) = task(store, project_id, task_id)?;
            if *version != r.version || *lifecycle_version != t.lifecycle_version {
                return Err(stale());
            }
            if t.run_occupancy.is_some() {
                // `completion_pending` 期间只接受匹配 Run 归约器的内部完成命令（P2.3 接）。
                return Err(reject(
                    "RUN_ACTIVE",
                    "completion_pending 期间不接受人的完成命令",
                    "wait_for_run_reducer",
                ));
            }
            if !active_runs(store, &t)?.is_empty() {
                return Err(reject(
                    "RUN_ACTIVE",
                    "a bound Run is not terminal",
                    "finish_run_first",
                ));
            }
            if t.lifecycle != "open" {
                return Err(reject("TASK_TERMINAL", "Task is not open", "inspect_task"));
            }
            let (contract, revision) = contract_of(store, actor, &t)?;
            if *revision_number != revision.number {
                return Err(stale());
            }
            let items = completion_items(store, actor, &t, &contract, acceptance)?;
            let receipt_id = format!("{}:{}", t.id, t.lifecycle_version + 1);
            let receipt = CompletionReceipt {
                receipt_id: receipt_id.clone(),
                task_id: t.id.clone(),
                project_id: project_id.clone(),
                command_id: format!("task:{}", input.key),
                idempotency_key: input.key.clone(),
                lifecycle_version: t.lifecycle_version + 1,
                revision_number: revision.number,
                revision_digest: revision.proposal_digest.clone(),
                policy_digest: revision.policy_digest.clone(),
                items,
                completed_at: now(),
            };
            p.put(
                store,
                key(
                    Scope::Project(project_id.clone()),
                    COMPLETION_RECEIPT_KIND,
                    &receipt_id,
                ),
                &receipt,
            )?;
            t.lifecycle = "completed".into();
            t.lifecycle_version += 1;
            let (sr, _) = source(store, &t.repo_id, &t.source_id)?;
            p.put_task(store, &t, &reference(&sr))?;
            p.result = json!({
                "project_id": project_id,
                "task_id": task_id,
                "receipt_id": receipt_id,
                "lifecycle": "completed",
                "lifecycle_version": t.lifecycle_version,
                "items": &receipt.items,
            });
        }
        Action::Reopen {
            project_id,
            task_id,
            version,
            lifecycle_version,
            revision_number,
        } => {
            project(store, &mut p, project_id, None)?;
            let (r, mut t) = task(store, project_id, task_id)?;
            if *version != r.version || *lifecycle_version != t.lifecycle_version {
                return Err(stale());
            }
            if t.lifecycle == "open" {
                return Err(reject("TASK_OPEN", "Task is already open", "inspect_task"));
            }
            if t.run_occupancy.is_some() || !active_runs(store, &t)?.is_empty() {
                return Err(reject(
                    "RUN_ACTIVE",
                    "reopen does not stop Run",
                    "finish_run_first",
                ));
            }
            match (revision_number, t.revision.as_ref()) {
                // 未处理 drift：必须显式冻结继续使用的当前 Revision。
                (None, _) if t.pending_contract.is_some() => {
                    return Err(reject(
                        "CONTRACT_DRIFT",
                        "adopt the new contract or freeze the current Revision explicitly",
                        "adopt_contract",
                    ));
                }
                (Some(expected), Some(current)) if expected != &current.number => {
                    return Err(stale());
                }
                _ => {}
            }
            t.lifecycle = "open".into();
            t.lifecycle_version += 1;
            t.archived = false;
            let (sr, _) = source(store, &t.repo_id, &t.source_id)?;
            p.put_task(store, &t, &reference(&sr))?;
            p.result = json!({
                "project_id": project_id,
                "task_id": task_id,
                "lifecycle": "open",
                "lifecycle_version": t.lifecycle_version,
            });
        }
        Action::DeleteCard {
            project_id,
            task_id,
            version,
            confirm_irreversible,
            active_run_choices,
        } => {
            let rid = project(store, &mut p, project_id, None)?;
            let (r, t) = task(store, project_id, task_id)?;
            if r.version != *version {
                return Err(stale());
            }
            let (sr, src) = connected(store, &mut p, project_id, &rid, &t.source_id)?;
            if !src.capabilities.delete {
                return Err(reject(
                    "DELETE_UNSUPPORTED",
                    "binding cannot delete",
                    "use_provider_client",
                ));
            }
            let (snap_r, snap) = latest(store, &src)?;
            let c = card(
                &snap,
                &t.entity
                    .as_ref()
                    .ok_or_else(stale)?
                    .immutable_external_entity_id,
            )?;
            p.check(&snap_r);
            let mut affected = vec![];
            for (r, other) in tasks(store)?
                .into_iter()
                .filter(|(_, o)| o.entity == t.entity)
            {
                p.check(&r);
                let runs = active_runs(store, &other)?;
                for run in &runs {
                    p.check(run);
                }
                if *confirm_irreversible
                    && runs
                        .iter()
                        .any(|r| !active_run_choices.contains(&reference(r)))
                {
                    return Err(reject(
                        "RUN_CHOICE_REQUIRED",
                        "explicitly acknowledge each active Run; deletion will not stop it",
                        "review_affected_tasks",
                    ));
                }
                affected.push(json!({"project_id":other.project_id,"task_id":other.id,"lifecycle":other.lifecycle,"runs":runs,"occupancy":other.run_occupancy}));
                if *confirm_irreversible
                    && other.run_occupancy.is_some()
                    && active_runs(store, &other)?.is_empty()
                {
                    return Err(reject(
                        "RUN_CHOICE_REQUIRED",
                        "occupied Task lacks readable Run",
                        "inspect_run",
                    ));
                }
            }
            p.result = json!({"project_id":project_id,"task_id":task_id,"affected":affected,"irreversible":true,"other_controls":"not_enumerated","does_not_stop_runs":true});
            if *confirm_irreversible {
                add_effect(
                    store,
                    &mut p,
                    &t,
                    &sr,
                    "task.delete",
                    json!({"card":c,"affected":affected}),
                )?;
            }
        }
        Action::Refresh { repo_id, source_id } => {
            source(store, repo_id, source_id)?;
            p.result = json!({"repo_id":repo_id,"source_id":source_id});
        }
        Action::Resume { effect_id } => {
            let (e, state) = store.effect(effect_id)?;
            if !e.operation.starts_with("task.") {
                return Err(stale());
            }
            p.result = json!({"effect_id":effect_id,"state":state});
        }
        Action::Withdraw { effect_id } => {
            let (e, state) = store.effect(effect_id)?;
            if !e.operation.starts_with("task.") {
                return Err(stale());
            }
            if state != store::EffectState::Pending {
                return Err(reject(
                    "READBACK_REQUIRED",
                    "only an unsent intent can be withdrawn",
                    "resume_effect",
                ));
            }
            p.cancel_effects.push(effect_id.clone());
            p.result = json!({"withdrawn_effect_id":effect_id,"state":"cancelled"});
        }
    }
    // Exact Project authorization survives subsequent default/policy updates.
    // Claim has no contract yet, but already creates a Project-owned Task.
    for record in p
        .records
        .iter_mut()
        .filter(|r| r.key.kind == "task" && r.version == 1)
    {
        let project_id = match &record.key.scope {
            Scope::Project(id) => id,
            _ => continue,
        };
        let accepted = required(store, &key(record.key.scope.clone(), "project", project_id))?;
        record.sources.push(reference(&accepted));
    }
    Ok(p)
}

/// An unresolved creation reserves its own card even if periodic reads see it first.
pub(crate) fn pending_creation(
    store: &Store,
    project: &str,
    source: &str,
    card: &Card,
) -> Result<Option<String>> {
    for id in store.pending_effects()? {
        let (effect, _) = store.effect(&id)?;
        if effect.operation == "task.create"
            && effect.input["project_id"].as_str() == Some(project)
            && effect.binding.key.id == source
            && effect.input["write"]["marker"]
                .as_str()
                .is_some_and(|marker| card.body.contains(marker))
        {
            return Ok(effect.input["task_id"].as_str().map(str::to_owned));
        }
    }
    Ok(None)
}

pub(crate) fn deletion_consequences(store: &Store, task: &Task) -> Result<Value> {
    let mut affected = vec![];
    for (_, other) in tasks(store)?
        .into_iter()
        .filter(|(_, other)| other.entity == task.entity)
    {
        let runs = active_runs(store, &other)?;
        affected.push(json!({"project_id":other.project_id,"task_id":other.id,"lifecycle":other.lifecycle,"runs":runs,"occupancy":other.run_occupancy}));
    }
    Ok(Value::Array(affected))
}

/// P2.3 writes Run records. Until then unknown Run shapes are conservatively nonterminal.
pub fn active_runs(store: &Store, task: &Task) -> Result<Vec<Record>> {
    Ok(store
        .list("run")?
        .into_iter()
        .filter(|r| {
            if r.key.scope != Scope::Project(task.project_id.clone()) {
                return false;
            }
            let RecordData::Value { value } = &r.data else {
                return true;
            };
            let target = value.get("task_id").and_then(Value::as_str);
            target.is_none_or(|s| s == task.id)
                && !["completed", "cancelled", "failed", "replaced"]
                    .contains(&value["lifecycle"].as_str().unwrap_or("unknown"))
        })
        .collect())
}
