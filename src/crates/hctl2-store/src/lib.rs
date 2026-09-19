//! Storage primitives for trusted control reducers, not a public command transport.
//!
//! Domain admission belongs to the consuming module. This crate supplies one transaction
//! for records, links, command results, inbox and outbox, and a separate durable Git byte store.

#![forbid(unsafe_code)]

mod backup;
mod command;
mod error;
mod materials;
mod model;
mod schema;
mod store;

pub use backup::BackupReport;
pub use command::{CommandTransaction, EffectIntent, EffectState, InboxEntry, Readback};
pub use error::{Result, StoreError};
pub use materials::{DeliveryGrant, MaterialRef};
pub use model::*;
pub use store::{StartupStatus, Store, WriterGeneration};
