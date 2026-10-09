//! Project governance over the shared store; provider I/O stays in control.
#![forbid(unsafe_code)]

mod commands;
pub mod invocation;
pub mod memo;
mod model;
mod requests;
mod views;
pub use chat::{decode, invalid, key, reference, reject, required, value_record};
pub use commands::*;
pub use model::*;
pub use requests::*;
pub use store::{Result, StoreError};
pub use views::*;

use store::{Record, RecordData, Scope, Store};

pub fn project(store: &Store, id: &str) -> Result<Record> {
    required(store, &key(Scope::Project(id.into()), "project", id))
}
pub fn definition(store: &Store, id: &str) -> Result<Definition> {
    decode(&required(
        store,
        &key(Scope::Project(id.into()), "project_details", id),
    )?)
}
pub fn readonly(record: &Record) -> bool {
    matches!(record.data, RecordData::Project { archived: true, .. })
}
pub fn stale() -> StoreError {
    reject(
        "VERSION_CONFLICT",
        "Project or source changed after preview",
        "preview_again",
    )
}
