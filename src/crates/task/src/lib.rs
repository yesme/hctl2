//! P2.2 Task shadows. Native providers perform I/O outside this crate's transactions.
#![forbid(unsafe_code)]

mod commands;
mod model;
mod observations;
mod planning;
pub use commands::*;
pub use model::*;
pub use observations::*;
pub use planning::*;
pub use repo::reject;
pub use store::{Result, StoreError};

use foundation::canonical_json_sha256;
use serde::{Serialize, de::DeserializeOwned};
use store::{ObjectKey, Record, RecordData, Reference, Scope, Store, Version};

pub fn key(scope: Scope, kind: &str, id: &str) -> ObjectKey {
    ObjectKey {
        scope,
        kind: kind.into(),
        id: id.into(),
    }
}
pub fn reference(record: &Record) -> Reference {
    Reference {
        key: record.key.clone(),
        version: Version::State(record.version),
    }
}
pub fn value_record<T: Serialize>(key: ObjectKey, version: i64, value: &T) -> Result<Record> {
    let value = serde_json::to_value(value)?;
    Ok(Record {
        key,
        version,
        revision_digest: canonical_json_sha256(&value)?,
        data: RecordData::Value { value },
        sources: vec![],
        materials: vec![],
    })
}
pub fn decode<T: DeserializeOwned>(record: &Record) -> Result<T> {
    match &record.data {
        RecordData::Value { value } => Ok(serde_json::from_value(value.clone())?),
        _ => Err(reject(
            "INVALID_RECORD",
            "unexpected record type",
            "inspect_storage",
        )),
    }
}
pub fn required(store: &Store, key: &ObjectKey) -> Result<Record> {
    store.get(key)?.ok_or_else(|| {
        reject(
            "NOT_FOUND",
            format!("{} not found", key.kind),
            "inspect_object",
        )
    })
}
pub fn source(store: &Store, repo_id: &str, id: &str) -> Result<(Record, Source)> {
    let record = required(store, &key(Scope::Repo(repo_id.into()), "task_source", id))?;
    let source = decode(&record)?;
    Ok((record, source))
}
pub fn task(store: &Store, project: &str, id: &str) -> Result<(Record, Task)> {
    let record = required(
        store,
        &key(Scope::Project(project.into()), "task_state", id),
    )?;
    let task = decode(&record)?;
    Ok((record, task))
}
pub fn tasks(store: &Store) -> Result<Vec<(Record, Task)>> {
    store
        .list("task_state")?
        .into_iter()
        .map(|r| Ok((r.clone(), decode(&r)?)))
        .collect()
}
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
