use crate::{
    Compiled, Provenance, Snapshot, SnapshotSource, Validator, Workflow, compile,
    readback::{self, InvocationEvidence, Readback, Readbacks},
};
use chat::{decode, invalid, key, reference, reject, required, stale, value_record};
use foundation::{bytes_sha256, canonical_json, canonical_json_sha256};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{cell::RefCell, collections::BTreeSet};
use store::{
    Command, Expected, ObjectKey, Record, Reference, Result, Scope, Store, TrustedActor, Version,
};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub key: String,
    pub action: Action,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Register {
        project_id: String,
        project_version: i64,
        workflow_id: String,
        supersedes: Option<i64>,
        task: Option<Reference>,
        workflow: Box<Workflow>,
        snapshot: Option<Snapshot>,
        provenance: Option<Box<Provenance>>,
    },
    Compile {
        project_id: String,
        project_version: i64,
        workflow_id: String,
        revision: i64,
    },
    Approve {
        project_id: String,
        project_version: i64,
        workflow_id: String,
        revision: i64,
        readback: Option<Reference>,
    },
}
impl Action {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Register { .. } => "register",
            Self::Compile { .. } => "compile",
            Self::Approve { .. } => "approve",
        }
    }
    pub fn project_id(&self) -> &str {
        match self {
            Self::Register { project_id, .. }
            | Self::Compile { project_id, .. }
            | Self::Approve { project_id, .. } => project_id,
        }
    }
    fn parts(&self) -> (&str, i64, &str) {
        match self {
            Self::Register {
                project_id,
                project_version,
                workflow_id,
                ..
            }
            | Self::Compile {
                project_id,
                project_version,
                workflow_id,
                ..
            }
            | Self::Approve {
                project_id,
                project_version,
                workflow_id,
                ..
            } => (project_id, *project_version, workflow_id),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Revision {
    pub workflow_id: String,
    pub revision: i64,
    pub author: String,
    pub workflow_digest: String,
    pub snapshot_digest: Option<String>,
    pub workflow: Workflow,
    pub snapshot: Option<Snapshot>,
    pub provenance: Option<Provenance>,
    pub task: Option<Reference>,
    pub warnings: Vec<crate::compiler::Warning>,
}
/// Engine definitions are a separate immutable revision family, not the Workflow's version counter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Deployment {
    pub workflow: Reference,
    pub workflow_digest: String,
    pub profile: String,
    pub adapter: String,
    pub adapter_version: u64,
    pub compiled: Compiled,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub input: Input,
    pub checks: Vec<chat::Check>,
    pub records: Vec<Record>,
    pub materials: Vec<(ObjectKey, String, Vec<u8>)>,
    pub result: Value,
}
fn scope(project: &str) -> Scope {
    Scope::Project(project.into())
}
fn command_key(input: &Input) -> ObjectKey {
    key(
        scope(input.action.project_id()),
        "workflow_command",
        &bytes_sha256(input.key.as_bytes()),
    )
}
fn track(checks: &mut Vec<chat::Check>, record: &Record) {
    checks.push(chat::Check {
        key: record.key.clone(),
        version: Some(record.version),
    });
}
fn check_absent(
    store: &Store,
    checks: &mut Vec<chat::Check>,
    key: &ObjectKey,
) -> Result<Option<Record>> {
    let current = store.get(key)?;
    checks.push(chat::Check {
        key: key.clone(),
        version: current.as_ref().map(|r| r.version),
    });
    Ok(current)
}
fn exact(store: &Store, project: &str, target: &Reference) -> Result<Record> {
    if target.key.scope != scope(project) {
        return Err(invalid("reference belongs to another Project"));
    }
    let record = required(store, &target.key)?;
    if reference(&record) != *target {
        return Err(stale());
    }
    Ok(record)
}
fn replay(store: &Store, input: &Input, actor: &TrustedActor) -> Result<Option<Plan>> {
    let Some(record) = store.get(&command_key(input))? else {
        return Ok(None);
    };
    let value: Value = decode(&record)?;
    let plan: Plan = serde_json::from_value(value["plan"].clone())?;
    if value["actor"] != serde_json::to_value(&actor.0)? || plan.input != *input {
        return Err(reject(
            "IDEMPOTENCY_CONFLICT",
            "workflow key belongs to different input or actor",
            "use_original_command",
        ));
    }
    Ok(Some(plan))
}
pub fn at(
    store: &Store,
    actor: &TrustedActor,
    project: &str,
    workflow: &str,
    revision: i64,
) -> Result<(Record, Revision)> {
    let scoped = chat::owner(actor, project)?;
    let record = required(
        store,
        &key(
            scope(project),
            "workflow_revision",
            &format!("{workflow}:{revision}"),
        ),
    )?;
    let data: Revision = decode(&record)?;
    let body = canonical_json(&serde_json::to_value(&data.workflow)?)?;
    if record.materials.is_empty()
        || store.read_material(&scoped, &record.materials[0])? != body
        || canonical_json_sha256(&serde_json::to_value(&data.workflow)?)? != data.workflow_digest
        || data.workflow_id != workflow
        || data.revision != revision
    {
        return Err(reject(
            "DIGEST_MISMATCH",
            "immutable workflow body differs",
            "inspect_workflow",
        ));
    }
    if let Some(snapshot) = &data.snapshot {
        let value = serde_json::to_value(snapshot)?;
        if record.materials.len() != 2
            || store.read_material(&scoped, &record.materials[1])? != canonical_json(&value)?
            || data.snapshot_digest.as_deref() != Some(canonical_json_sha256(&value)?.as_str())
        {
            return Err(invalid("immutable snapshot differs"));
        }
    }
    Ok((record, data))
}
pub fn prepare(
    store: &Store,
    input: Input,
    actor: &TrustedActor,
    validator: &dyn Validator,
) -> Result<Plan> {
    chat::owner(actor, input.action.project_id())?;
    if input.key.trim().is_empty() || input.key != input.key.trim() {
        return Err(invalid("workflow command key required"));
    }
    if let Some(plan) = replay(store, &input, actor)? {
        return Ok(plan);
    }
    let (project, version, workflow_id) = input.action.parts();
    if workflow_id.is_empty() || workflow_id.contains(':') || workflow_id.trim() != workflow_id {
        return Err(invalid(
            "workflow id required, without colon or surrounding spaces",
        ));
    }
    let active = chat::active_project(store, project, version)?;
    let mut checks = vec![];
    track(&mut checks, &active);
    let mut records = vec![];
    let mut materials = vec![];
    let result = match &input.action {
        Action::Register {
            supersedes,
            task,
            workflow,
            snapshot,
            provenance,
            ..
        } => {
            let compiled = compile(workflow)?;
            if workflow.readback.required && (snapshot.is_none() || provenance.is_none()) {
                return Err(invalid(
                    "readback requires a frozen three-list snapshot and explicit source chain",
                ));
            }
            if let Some(provenance) = provenance {
                provenance_checks(store, actor, project, provenance, &mut checks)?;
            }
            if let (Some(snapshot), Some(provenance)) = (snapshot, provenance)
                && let SnapshotSource::Message { event, .. } = &provenance.snapshot_source
            {
                let source = exact(store, project, event)?;
                let material = source
                    .materials
                    .first()
                    .ok_or_else(|| invalid("snapshot source body missing"))?;
                let bytes = store.read_material(&chat::owner(actor, project)?, material)?;
                let source_value: Value = serde_json::from_slice(&bytes)?;
                if canonical_json_sha256(&source_value)?
                    != canonical_json_sha256(&serde_json::to_value(snapshot)?)?
                {
                    return Err(invalid(
                        "snapshot differs from its frozen source message; use direct text with source chain for an extracted snapshot",
                    ));
                }
            }
            let pointer_key = key(scope(project), "workflow", workflow_id);
            let current = check_absent(store, &mut checks, &pointer_key)?;
            let revision = match (current.as_ref(), supersedes) {
                (None, None) => 1,
                (Some(current), Some(previous)) if *previous == current.version => previous
                    .checked_add(1)
                    .ok_or_else(|| invalid("workflow revision exhausted"))?,
                _ => return Err(stale()),
            };
            if let Some(target) = task {
                if target.key.kind != "task" {
                    return Err(invalid("Task reference required"));
                }
                let task = exact(store, project, target)?;
                track(&mut checks, &task);
            }
            let workflow_value = serde_json::to_value(workflow)?;
            let snapshot_value = snapshot.as_ref().map(serde_json::to_value).transpose()?;
            let revision_data = Revision {
                workflow_id: workflow_id.into(),
                revision,
                author: actor.0.principal.clone(),
                workflow_digest: canonical_json_sha256(&workflow_value)?,
                snapshot_digest: snapshot_value
                    .as_ref()
                    .map(canonical_json_sha256)
                    .transpose()?,
                workflow: (**workflow).clone(),
                snapshot: snapshot.clone(),
                provenance: provenance.as_ref().map(|p| (**p).clone()),
                task: task.clone(),
                warnings: compiled.warnings,
            };
            let record = value_record(
                key(
                    scope(project),
                    "workflow_revision",
                    &format!("{workflow_id}:{revision}"),
                ),
                1,
                &revision_data,
            )?;
            materials.push((
                record.key.clone(),
                "workflow".into(),
                canonical_json(&workflow_value)?,
            ));
            if let Some(value) = snapshot_value {
                materials.push((
                    record.key.clone(),
                    "snapshot".into(),
                    canonical_json(&value)?,
                ));
            }
            records.push(value_record(pointer_key, revision, &reference(&record))?);
            if let Some(task) = task {
                records.push(value_record(
                    key(
                        scope(project),
                        "workflow_task_link",
                        &format!("{workflow_id}:{revision}"),
                    ),
                    1,
                    &json!({"workflow":reference(&record),"task":task}),
                )?);
            }
            let result = json!({"workflow_id":workflow_id,"revision":revision,"revision_reference":reference(&record),"workflow_digest":revision_data.workflow_digest,"snapshot_digest":revision_data.snapshot_digest,"warnings":revision_data.warnings,"started":false});
            records.push(record);
            result
        }
        Action::Compile { revision, .. } => {
            let (record, data) = at(store, actor, project, workflow_id, *revision)?;
            track(&mut checks, &record);
            let compiled = compile(&data.workflow)?;
            let document = Deployment {
                workflow: reference(&record),
                workflow_digest: data.workflow_digest,
                profile: data.workflow.profile,
                adapter: "dagu.passive".into(),
                adapter_version: 1,
                compiled: compiled.clone(),
            };
            let deployment_id = canonical_json_sha256(&serde_json::to_value(&document)?)?;
            let deployment_key = key(scope(project), "engine_deployment", &deployment_id);
            if let Some(existing) = check_absent(store, &mut checks, &deployment_key)? {
                let current: Deployment = decode(&existing)?;
                if current != document {
                    return Err(invalid("immutable engine deployment differs"));
                }
            }
            validator.validate(&compiled)?;
            let deployment = value_record(deployment_key, 1, &document)?;
            materials.push((
                deployment.key.clone(),
                "dagu_yaml".into(),
                compiled.yaml.as_bytes().to_vec(),
            ));
            let pointer_key = key(
                scope(project),
                "workflow_deployment",
                &format!("{workflow_id}:{revision}"),
            );
            let current = check_absent(store, &mut checks, &pointer_key)?;
            let unchanged = current
                .as_ref()
                .map(decode::<Reference>)
                .transpose()?
                .is_some_and(|r| r == reference(&deployment));
            if !unchanged {
                records.push(value_record(
                    pointer_key,
                    current.map_or(1, |r| r.version + 1),
                    &reference(&deployment),
                )?);
            }
            let result = json!({"workflow_id":workflow_id,"revision":revision,"deployment":reference(&deployment),"compiled":compiled,"validated":true,"started":false});
            records.push(deployment);
            result
        }
        Action::Approve {
            revision, readback, ..
        } => {
            let (record, data) = at(store, actor, project, workflow_id, *revision)?;
            track(&mut checks, &record);
            let deployment_pointer = required(
                store,
                &key(
                    scope(project),
                    "workflow_deployment",
                    &format!("{workflow_id}:{revision}"),
                ),
            )?;
            track(&mut checks, &deployment_pointer);
            let deployment_ref: Reference = decode(&deployment_pointer)?;
            if deployment_ref.key.kind != "engine_deployment" {
                return Err(invalid("immutable Engine Deployment required"));
            }
            let deployment = exact(store, project, &deployment_ref)?;
            let document: Deployment = decode(&deployment)?;
            if document.workflow != reference(&record)
                || document.workflow_digest != data.workflow_digest
            {
                return Err(invalid(
                    "deployment belongs to a different Workflow Revision",
                ));
            }
            track(&mut checks, &deployment);
            let proof = if data.workflow.readback.required {
                let target = readback.as_ref().ok_or_else(|| {
                    reject(
                        "WORKFLOW_READBACK_REQUIRED",
                        "isolated readback reference required",
                        "produce_readback",
                    )
                })?;
                let port = StoreReadbacks {
                    store,
                    actor,
                    project,
                    checks: RefCell::new(vec![]),
                };
                let proof = readback::verify(
                    &port,
                    target,
                    &data.workflow_digest,
                    data.snapshot_digest
                        .as_deref()
                        .ok_or_else(|| invalid("snapshot missing"))?,
                    data.provenance
                        .as_ref()
                        .ok_or_else(|| invalid("provenance missing"))?,
                    data.workflow.readback.different_worker_profile,
                )?;
                checks.extend(port.checks.into_inner());
                proof
            } else {
                json!({"readback":"explicitly_disabled"})
            };
            let approval_key = key(
                scope(project),
                "workflow_approval",
                &format!("{workflow_id}:{revision}"),
            );
            let current = check_absent(store, &mut checks, &approval_key)?;
            let approval = value_record(
                approval_key,
                current.map_or(1, |r| r.version + 1),
                &json!({"revision":reference(&record),"deployment":reference(&deployment),"proof":proof,"approved_by":actor.0.principal}),
            )?;
            let result = json!({"workflow_id":workflow_id,"revision":revision,"approval":reference(&approval),"proof":proof,"started":false});
            records.push(approval);
            result
        }
    };
    Ok(Plan {
        input,
        checks,
        records,
        materials,
        result,
    })
}
fn provenance_checks(
    store: &Store,
    actor: &TrustedActor,
    project: &str,
    provenance: &Provenance,
    checks: &mut Vec<chat::Check>,
) -> Result<()> {
    let mut sources: Vec<_> = provenance
        .source_chain
        .iter()
        .map(|s| (&s.room_id, &s.event))
        .collect();
    if let SnapshotSource::Message { room_id, event } = &provenance.snapshot_source {
        sources.push((room_id, event));
    }
    for (room, event) in sources {
        if event.key.kind != "chat_source_reference" {
            return Err(invalid("frozen Room message event reference required"));
        }
        let record = exact(store, project, event)?;
        let source: chat::Source = decode(&record)?;
        let chat::Source::Message {
            binding,
            event_id,
            content_digest,
        } = source
        else {
            return Err(invalid("source chain must name real Room messages"));
        };
        if binding.key.id != *room || event_id.is_empty() {
            return Err(invalid("source Room and frozen message disagree"));
        }
        let [material] = record.materials.as_slice() else {
            return Err(invalid("source message body must be admitted"));
        };
        let bytes = store.read_material(&chat::owner(actor, project)?, material)?;
        if bytes_sha256(&bytes) != content_digest || material.byte_digest != content_digest {
            return Err(invalid("source message digest differs"));
        }
        track(checks, &record);
    }
    if let crate::Production::Invocations { references } = &provenance.production {
        if references.is_empty() {
            return Err(invalid(
                "empty production set is not explicit no-model production",
            ));
        }
        let unique: BTreeSet<_> = references
            .iter()
            .map(serde_json::to_string)
            .collect::<std::result::Result<_, _>>()?;
        if unique.len() != references.len() {
            return Err(invalid("duplicate producer reference"));
        }
        // Missing/unresolvable origin is not silently satisfied at approval.
    }
    Ok(())
}
pub fn admit(
    store: &mut Store,
    actor: &TrustedActor,
    plan: Plan,
    validator: &dyn Validator,
) -> Result<Value> {
    let scoped = chat::owner(actor, plan.input.action.project_id())?;
    if let Some(previous) = replay(store, &plan.input, actor)? {
        return Ok(previous.result);
    }
    if serde_json::to_value(prepare(store, plan.input.clone(), actor, validator)?)?
        != serde_json::to_value(&plan)?
    {
        return Err(stale());
    }
    let mut records = plan.records.clone();
    let mut saved = vec![];
    for (target, role, bytes) in &plan.materials {
        let material = store.save_material(
            store.generation(),
            &scoped,
            &scope(plan.input.action.project_id()),
            &plan.input.key,
            role,
            bytes,
        )?;
        records
            .iter_mut()
            .find(|r| r.key == *target)
            .ok_or_else(|| invalid("material target missing"))?
            .materials
            .push(material.clone());
        saved.push(material);
    }
    let input = serde_json::to_value(&plan.input)?;
    let operation = format!("workflow.{}", plan.input.action.kind());
    let command = Command {
        command_id: format!("workflow:{}", plan.input.key),
        idempotency_key: plan.input.key.clone(),
        actor: scoped.0.clone(),
        target: command_key(&plan.input),
        expected: Expected::Absent,
        binding: Reference {
            key: key(Scope::Control, "module", "run"),
            version: Version::State(1),
        },
        input_digest: Command::digest_input(&operation, &input)?,
        operation,
        input,
    };
    store.submit(store.generation(), &scoped, &command, None, |tx| {
        for check in &plan.checks {
            if tx.get(&check.key)?.as_ref().map(|r| r.version) != check.version {
                return Err(stale());
            }
        }
        for material in &saved {
            tx.admit_material(material)?;
        }
        for record in &records {
            if record.key.kind == "engine_deployment" && tx.get(&record.key)?.is_some() {
                continue;
            }
            tx.put(record)?;
        }
        tx.put(&value_record(
            command.target.clone(),
            1,
            &json!({"actor":actor.0,"plan":plan}),
        )?)?;
        Ok(plan.result.clone())
    })
}
pub fn show(
    store: &Store,
    actor: &TrustedActor,
    project: &str,
    workflow: &str,
    revision: Option<i64>,
) -> Result<Value> {
    chat::owner(actor, project)?;
    let pointer = required(store, &key(scope(project), "workflow", workflow))?;
    let (record, data) = at(
        store,
        actor,
        project,
        workflow,
        revision.unwrap_or(pointer.version),
    )?;
    let deployment_pointer = store.get(&key(
        scope(project),
        "workflow_deployment",
        &format!("{workflow}:{}", data.revision),
    ))?;
    let deployment = deployment_pointer
        .as_ref()
        .map(|p| -> Result<Record> {
            let target: Reference = decode(p)?;
            exact(store, project, &target)
        })
        .transpose()?;
    let approval = store.get(&key(
        scope(project),
        "workflow_approval",
        &format!("{workflow}:{}", data.revision),
    ))?;
    Ok(
        json!({"reference":reference(&record),"revision":data,"deployment":deployment,"approval":approval,"startable":approval.is_some()}),
    )
}
pub fn list(store: &Store, actor: &TrustedActor, project: &str) -> Result<Value> {
    chat::owner(actor, project)?;
    let workflows = store
        .list("workflow")?
        .into_iter()
        .filter(|r| r.key.scope == scope(project))
        .map(|r| json!({"workflow_id":r.key.id,"revision":r.version}))
        .collect::<Vec<_>>();
    Ok(json!({"workflows":workflows}))
}
/// Downstream Run start must call this, not infer approval from a saved plan.
pub fn require_approved(
    store: &Store,
    project: &str,
    workflow: &str,
    revision: i64,
) -> Result<Record> {
    store
        .get(&key(
            scope(project),
            "workflow_approval",
            &format!("{workflow}:{revision}"),
        ))?
        .ok_or_else(|| {
            reject(
                "WORKFLOW_NOT_APPROVED",
                "saved Workflow Revision is not approved",
                "approve_workflow",
            )
        })
}
struct StoreReadbacks<'a> {
    store: &'a Store,
    actor: &'a TrustedActor,
    project: &'a str,
    checks: RefCell<Vec<chat::Check>>,
}
impl StoreReadbacks<'_> {
    fn load<T: serde::de::DeserializeOwned>(&self, target: &Reference, kind: &str) -> Result<T> {
        if target.key.kind != kind {
            return Err(invalid(format!("{kind} reference required")));
        }
        let record = exact(self.store, self.project, target)?;
        track(&mut self.checks.borrow_mut(), &record);
        decode(&record)
    }
}
impl Readbacks for StoreReadbacks<'_> {
    fn readback(&self, target: &Reference) -> Result<Readback> {
        let read: Readback = self.load(target, "workflow_readback")?;
        let record = exact(self.store, self.project, target)?;
        let [body] = record.materials.as_slice() else {
            return Err(invalid("readback body must be admitted"));
        };
        if body.byte_digest != read.body_digest
            || bytes_sha256(
                &self
                    .store
                    .read_material(&chat::owner(self.actor, self.project)?, body)?,
            ) != read.body_digest
        {
            return Err(invalid("readback body digest differs"));
        }
        Ok(read)
    }
    fn invocation(&self, target: &Reference) -> Result<InvocationEvidence> {
        self.load(target, "workflow_invocation_evidence")
    }
    fn producer(&self, target: &Reference) -> Result<readback::Producer> {
        let value: Value = self.load(target, "room_invocation")?;
        let spec: agency_proto::Sealed<agency_proto::ExecutionSpec> =
            serde_json::from_value(value["spec"].clone())?;
        spec.verify().map_err(|e| invalid(e.to_string()))?;
        spec.document
            .validate()
            .map_err(|e| invalid(e.to_string()))?;
        let source_room = value["preview"]["input"]["room_id"]
            .as_str()
            .ok_or_else(|| invalid("producer has no source Room"))?;
        if spec.document.owner.id != target.key.id
            || spec.document.owner.kind != agency_proto::OwnerKind::RoomInvocation
        {
            return Err(invalid("actual Room Invocation producer required"));
        }
        Ok(readback::Producer {
            source_room: source_room.into(),
            profile_digest: spec.document.profile.digest,
        })
    }
    fn roster(&self, room: &str) -> Result<Vec<Reference>> {
        let (binding, room) = chat::room(self.store, self.project, room)?;
        track(&mut self.checks.borrow_mut(), &binding);
        check_absent(
            self.store,
            &mut self.checks.borrow_mut(),
            &key(scope(self.project), "room_roster", &room.id),
        )?;
        Ok(room.participants)
    }
}
