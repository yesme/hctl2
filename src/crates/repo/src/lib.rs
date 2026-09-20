//! Repo owns registration. Platform calls and Git writes run outside Store transactions.
#![forbid(unsafe_code)]

pub mod git;
mod model;
mod registry;

pub use model::*;
pub use registry::*;
pub use store::{Result, StoreError};

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
