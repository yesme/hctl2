//! Room governance; Matrix transport stays in the control adapter.
#![forbid(unsafe_code)]

mod actions;
mod brief;
pub use actions::*;
mod commands;
mod model;
pub use brief::*;
pub use commands::*;
pub use model::*;
pub use store::{Result, StoreError};

use foundation::canonical_json_sha256;
use serde::{Serialize, de::DeserializeOwned};
use store::{ObjectKey, Record, RecordData, Reference, Scope, Store, Version};

pub fn reject(
    code: &'static str,
    message: impl Into<String>,
    recovery: &'static str,
) -> StoreError {
    StoreError {
        code,
        message: message.into(),
        recovery_action: recovery,
    }
}
pub fn invalid(message: impl Into<String>) -> StoreError {
    reject("INVALID_INPUT", message, "correct_input")
}
pub fn stale() -> StoreError {
    reject(
        "VERSION_CONFLICT",
        "Room or source changed after preview",
        "preview_again",
    )
}
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
        _ => Err(invalid("unexpected stored record kind")),
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
pub fn room(store: &Store, project: &str, id: &str) -> Result<(Record, Room)> {
    let record = required(
        store,
        &key(Scope::Project(project.into()), "room_binding", id),
    )?;
    let room = decode(&record)?;
    Ok((record, room))
}
pub fn active_project(store: &Store, project: &str, version: i64) -> Result<Record> {
    let record = required(
        store,
        &key(Scope::Project(project.into()), "project", project),
    )?;
    match &record.data {
        RecordData::Project {
            archived: false, ..
        } if record.version == version => Ok(record),
        RecordData::Project { archived: true, .. } => Err(reject(
            "PROJECT_READ_ONLY",
            "Project is archived",
            "restore_project",
        )),
        _ => Err(stale()),
    }
}
