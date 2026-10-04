//! Independent Agency. Its only first-party dependency is the public port contract.
#![forbid(unsafe_code)]
pub mod catalog;
pub mod confine;
pub mod herdr;
pub mod runtime;
pub mod service;
mod storage;
mod tenant;
pub use service::{Agency, serve};
