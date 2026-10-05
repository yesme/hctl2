//! Independent Agency. Its only first-party dependency is the public port contract.
#![forbid(unsafe_code)]
pub mod catalog;
pub mod confine;
pub mod harness;
pub mod herdr;
pub mod launch;
pub mod runtime;
pub mod service;
mod standby;
mod storage;
mod tenant;
pub use service::{Agency, serve};
