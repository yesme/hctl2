//! Published knowledge. A Memo revision is append-only; updates are new revisions.
use crate::{decode, invalid, key, reference, reject, stale, value_record};
use foundation::{bytes_sha256, canonical_json_sha256};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use store::{Command, Expected, Record, Reference, Scope, Store, TrustedActor, Version};

/// The spec names Message and Artifact refs; Artifact registration is a later
/// package, so today only frozen Message references resolve.
const SOURCE_KINDS: [&str; 2] = ["chat_source_reference", "artifact_revision"];

/// One published revision. The exact body locator is the revision record's single
/// admitted material, filled on admission; a caller cannot supply it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Memo {
    pub memo_id: String,
    pub revision: i64,
    pub applicability: String,
    /// Authenticated principal. Never a payload field.
    pub author: String,
    /// Exact frozen sources, resolved to the stored version this command read.
    pub sources: Vec<Reference>,
    pub content_digest: String,
    /// The exact previous revision; `None` only for the first one.
    pub supersedes: Option<Reference>,
    /// Seconds since the epoch. `None` stays valid indefinitely.
    pub expires_at: Option<u64>,
}

impl Memo {
    fn validate(&self) -> store::Result<()> {
        if self.memo_id.trim().is_empty()
            || self.memo_id != self.memo_id.trim()
            || self.revision < 1
            || self.applicability.trim().is_empty()
            || self.author.trim().is_empty()
            || self.sources.is_empty()
            || self.expires_at == Some(0)
        {
            return Err(invalid(
                "Memo requires id, revision, applicability, author and exact sources",
            ));
        }
        if (self.revision == 1) != self.supersedes.is_none() {
            return Err(invalid(
                "the first revision supersedes nothing; a later one names its exact predecessor",
            ));
        }
        for (index, source) in self.sources.iter().enumerate() {
            if !SOURCE_KINDS.contains(&source.key.kind.as_str())
                || self.sources[..index].contains(source)
            {
                return Err(invalid(
                    "unique frozen Message or Artifact sources required",
                ));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub key: String,
    pub action: Action,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Publish {
        project_id: String,
        project_version: i64,
        memo_id: String,
        applicability: String,
        sources: Vec<chat::Source>,
        /// Exact revision replaced by this publication; `None` publishes the first.
        supersedes: Option<i64>,
        expires_at: Option<u64>,
        /// Exact published text. It becomes governance material on admission.
        body: String,
    },
}

impl Action {
    pub fn kind(&self) -> &'static str {
        "publish"
    }
    pub fn project_id(&self) -> &str {
        match self {
            Self::Publish { project_id, .. } => project_id,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub input: Input,
    pub project_id: String,
    pub checks: Vec<chat::Check>,
    /// Immutable revision at version 1; its body material is added on admission.
    pub revision: Record,
    /// Current pointer, keyed by Memo id and versioned by the revision number.
    pub pointer: Record,
    pub result: Value,
}

fn command_key(input: &Input) -> store::ObjectKey {
    key(
        Scope::Project(input.action.project_id().into()),
        "memo_command",
        &bytes_sha256(input.key.as_bytes()),
    )
}

fn missing() -> store::StoreError {
    reject(
        "MEMO_NOT_FOUND",
        "no such published Memo revision",
        "publish_memo",
    )
}

/// Replay is bound to the exact input and the original actor, so a reused key can
/// neither republish different text nor answer for somebody else.
fn replay(store: &Store, input: &Input, actor: &TrustedActor) -> store::Result<Option<Plan>> {
    let Some(record) = store.get(&command_key(input))? else {
        return Ok(None);
    };
    let stored: Value = decode(&record)?;
    if stored["actor"] != serde_json::to_value(&actor.0)? {
        return Err(reject(
            "ACTOR_MISMATCH",
            "Memo command belongs to another actor",
            "use_original_actor",
        ));
    }
    let plan: Plan = serde_json::from_value(stored["plan"].clone())?;
    if plan.input != *input {
        return Err(reject(
            "IDEMPOTENCY_CONFLICT",
            "Memo key has different input",
            "use_original_command",
        ));
    }
    Ok(Some(plan))
}

/// Pure read: the confirmed text becomes material only when the command is admitted.
pub fn prepare(store: &Store, input: Input, actor: &TrustedActor) -> store::Result<Plan> {
    let scoped = chat::owner(actor, input.action.project_id())?;
    if input.key.trim().is_empty() || input.key != input.key.trim() {
        return Err(invalid("command key required"));
    }
    if let Some(plan) = replay(store, &input, actor)? {
        return Ok(plan);
    }
    let Action::Publish {
        project_id,
        project_version,
        memo_id,
        applicability,
        sources,
        supersedes,
        expires_at,
        body,
    } = &input.action;
    if body.trim().is_empty() {
        return Err(invalid("published Memo text required"));
    }
    let p = chat::active_project(store, project_id, *project_version)?;
    let mut checks = vec![chat::Check {
        key: p.key.clone(),
        version: Some(p.version),
    }];
    let mut resolved = vec![];
    for source in sources {
        let record = frozen_source(store, &scoped, project_id, source)?;
        checks.push(chat::Check {
            key: record.key.clone(),
            version: Some(record.version),
        });
        resolved.push(reference(&record));
    }
    let pointer_key = key(Scope::Project(project_id.clone()), "memo", memo_id);
    let published = store.get(&pointer_key)?;
    let (revision, supersedes) = match (supersedes, &published) {
        (None, None) => (1, None),
        (None, Some(_)) => {
            return Err(reject(
                "SUPERSEDES_REQUIRED",
                "a published Memo is updated by a new revision naming its exact predecessor",
                "name_superseded_revision",
            ));
        }
        (Some(_), None) => return Err(missing()),
        (Some(previous), Some(pointer)) => {
            let target: Reference = decode(pointer)?;
            let (record, memo) = at(store, &target)?;
            if *previous < 1 || memo.revision != *previous {
                return Err(reject(
                    "VERSION_CONFLICT",
                    "superseded revision is not the published one",
                    "preview_current_revision",
                ));
            }
            (
                previous
                    .checked_add(1)
                    .ok_or_else(|| invalid("Memo revision exhausted"))?,
                Some(reference(&record)),
            )
        }
    };
    checks.push(chat::Check {
        key: pointer_key.clone(),
        version: published.as_ref().map(|r| r.version),
    });
    let memo = Memo {
        memo_id: memo_id.clone(),
        revision,
        applicability: applicability.clone(),
        author: actor.0.principal.clone(),
        sources: resolved,
        content_digest: bytes_sha256(body.as_bytes()),
        supersedes,
        expires_at: *expires_at,
    };
    memo.validate()?;
    let mut revision_record = value_record(
        key(
            Scope::Project(project_id.clone()),
            "memo_revision",
            &format!("{memo_id}:{revision}"),
        ),
        1,
        &memo,
    )?;
    revision_record.sources = std::iter::once(reference(&p))
        .chain(memo.sources.iter().cloned())
        .collect();
    let mut pointer = value_record(pointer_key, revision, &reference(&revision_record))?;
    pointer.sources = vec![reference(&revision_record)];
    Ok(Plan {
        input: input.clone(),
        project_id: project_id.clone(),
        checks,
        result: json!({
            "memo_id": memo_id,
            "revision": revision,
            "revision_reference": reference(&revision_record),
            "memo": memo,
            "body_bytes": body.len(),
        }),
        revision: revision_record,
        pointer,
    })
}

/// Only text a Room command already froze has a source reference; anything else has
/// no locator here, so it cannot be cited and cannot become a Memo.
fn frozen_source(
    store: &Store,
    scoped: &TrustedActor,
    project: &str,
    source: &chat::Source,
) -> store::Result<Record> {
    let (origin, declared) = match source {
        chat::Source::Message {
            binding,
            event_id,
            content_digest,
        } => {
            if event_id.trim().is_empty() {
                return Err(invalid("frozen message requires its exact event ID"));
            }
            (&binding.key.scope, content_digest)
        }
        chat::Source::Object {
            reference: target,
            content_digest,
        } => (&target.key.scope, content_digest),
    };
    if *origin != Scope::Project(project.into()) {
        return Err(invalid("Memo source belongs to another Project"));
    }
    let id = canonical_json_sha256(&serde_json::to_value(source)?)?;
    let record = store
        .get(&key(
            Scope::Project(project.into()),
            "chat_source_reference",
            &id,
        ))?
        .ok_or_else(|| {
            reject(
                "SOURCE_UNAVAILABLE",
                "exact source was never frozen by a Room command",
                "freeze_source",
            )
        })?;
    let stored: chat::Source = decode(&record)?;
    let [material] = record.materials.as_slice() else {
        return Err(reject(
            "SOURCE_UNAVAILABLE",
            "frozen source has no admitted body",
            "freeze_source",
        ));
    };
    let bytes = store.read_material(scoped, material)?;
    if stored != *source || bytes_sha256(&bytes) != *declared || material.byte_digest != *declared {
        return Err(reject(
            "DIGEST_MISMATCH",
            "frozen source bytes differ from the declared digest",
            "inspect_source",
        ));
    }
    Ok(record)
}

pub fn admit(store: &mut Store, actor: &TrustedActor, plan: Plan) -> store::Result<Value> {
    let scoped = chat::owner(actor, &plan.project_id)?;
    if let Some(previous) = replay(store, &plan.input, actor)? {
        return Ok(previous.result);
    }
    if serde_json::to_value(&prepare(store, plan.input.clone(), actor)?)?
        != serde_json::to_value(&plan)?
    {
        return Err(reject(
            "VERSION_CONFLICT",
            "Memo preview differs",
            "preview_again",
        ));
    }
    let Action::Publish { body, .. } = &plan.input.action;
    let material = store.save_material(
        store.generation(),
        &scoped,
        &Scope::Project(plan.project_id.clone()),
        &plan.input.key,
        "body",
        body.as_bytes(),
    )?;
    let mut revision = plan.revision.clone();
    revision.materials.push(material.clone());
    let input = serde_json::to_value(&plan.input)?;
    let command = Command {
        command_id: format!("memo:{}", plan.input.key),
        idempotency_key: plan.input.key.clone(),
        actor: scoped.0.clone(),
        target: command_key(&plan.input),
        expected: Expected::Absent,
        binding: Reference {
            key: key(Scope::Control, "module", "project"),
            version: Version::State(1),
        },
        input_digest: Command::digest_input("memo.publish", &input)?,
        operation: "memo.publish".into(),
        input,
    };
    store.submit(store.generation(), &scoped, &command, None, |tx| {
        for check in &plan.checks {
            if tx.get(&check.key)?.as_ref().map(|r| r.version) != check.version {
                return Err(stale());
            }
        }
        tx.admit_material(&material)?;
        if let Some(existing) = tx.get(&revision.key)? {
            if serde_json::to_value(&existing.data)? != serde_json::to_value(&revision.data)? {
                return Err(invalid("immutable Memo revision differs"));
            }
        } else {
            tx.put(&revision)?;
        }
        tx.put(&plan.pointer)?;
        tx.put(&value_record(
            command.target.clone(),
            1,
            &json!({"actor":actor.0,"plan":plan}),
        )?)?;
        Ok(plan.result.clone())
    })
}

/// Read one exact revision. The current pointer and any search index are projections;
/// neither can stand in for the referenced revision.
pub fn at(store: &Store, target: &Reference) -> store::Result<(Record, Memo)> {
    if target.key.kind != "memo_revision" {
        return Err(invalid("immutable Memo revision required"));
    }
    let record = store.get(&target.key)?.ok_or_else(missing)?;
    if reference(&record) != *target {
        return Err(reject(
            "VERSION_CONFLICT",
            "Memo revision differs",
            "preview_again",
        ));
    }
    let memo: Memo = decode(&record)?;
    memo.validate()?;
    if canonical_json_sha256(&serde_json::to_value(&memo)?)? != record.revision_digest
        || record.key.id != format!("{}:{}", memo.memo_id, memo.revision)
    {
        return Err(reject(
            "DIGEST_MISMATCH",
            "Memo content differs from its revision identity",
            "inspect_memo",
        ));
    }
    if memo
        .supersedes
        .as_ref()
        .is_some_and(|s| s.key.scope != record.key.scope)
    {
        return Err(invalid("superseded Memo belongs to another Project"));
    }
    let [body] = record.materials.as_slice() else {
        return Err(reject(
            "MATERIAL_NOT_ADMITTED",
            "Memo revision has no admitted body",
            "admit_material",
        ));
    };
    if body.byte_digest != memo.content_digest {
        return Err(reject(
            "DIGEST_MISMATCH",
            "Memo body locator differs from the published digest",
            "inspect_memo",
        ));
    }
    Ok((record, memo))
}

pub fn show(
    store: &Store,
    actor: &TrustedActor,
    project: &str,
    memo_id: &str,
    revision: Option<i64>,
) -> store::Result<Value> {
    let scoped = chat::owner(actor, project)?;
    if memo_id.trim().is_empty() {
        return Err(invalid("Memo id required"));
    }
    let scope = Scope::Project(project.into());
    let pointer = store
        .get(&key(scope.clone(), "memo", memo_id))?
        .ok_or_else(missing)?;
    let current: Reference = decode(&pointer)?;
    let target = match revision {
        None => current.clone(),
        Some(number) => {
            if number < 1 {
                return Err(invalid("Memo revision starts at one"));
            }
            Reference {
                key: key(scope, "memo_revision", &format!("{memo_id}:{number}")),
                version: Version::State(1),
            }
        }
    };
    let (record, memo) = at(store, &target)?;
    if memo.memo_id != memo_id {
        return Err(invalid("Memo revision belongs to another Memo"));
    }
    let body = String::from_utf8(store.read_material(&scoped, &record.materials[0])?)
        .map_err(|_| invalid("Memo body is not UTF-8"))?;
    Ok(json!({
        "project_id": project,
        "memo_id": memo_id,
        "current": target == current,
        "pointer": reference(&pointer),
        "revision": reference(&record),
        "memo": memo,
        "body": body,
    }))
}

/// Pointer manifest for assembly: superseded revisions are not pointers, and expired
/// ones are filtered out mechanically. An explicit reference bypasses this list.
pub fn list(store: &Store, actor: &TrustedActor, project: &str, now: u64) -> store::Result<Value> {
    chat::owner(actor, project)?;
    let scope = Scope::Project(project.into());
    let mut items = vec![];
    let mut manifest = vec![];
    for pointer in store.list("memo")? {
        if pointer.key.scope != scope {
            continue;
        }
        let target: Reference = decode(&pointer)?;
        let (record, memo) = at(store, &target)?;
        let expired = memo.expires_at.is_some_and(|expires| expires <= now);
        if !expired {
            manifest.push(reference(&record));
        }
        items.push(json!({
            "memo_id": memo.memo_id,
            "revision": memo.revision,
            "reference": reference(&record),
            "applicability": memo.applicability,
            "author": memo.author,
            "expires_at": memo.expires_at,
            "expired": expired,
        }));
    }
    Ok(json!({
        "project_id": project,
        "now": now,
        "items": items,
        "manifest": manifest,
    }))
}
