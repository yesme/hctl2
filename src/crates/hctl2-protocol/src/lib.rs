//! Transport-level values shared by HCTL2 processes.
//!
//! This crate deliberately contains no governance commands or domain rules. In P1, `hctl2-tool`
//! and `hctl2-agentd` are standalone mechanical components rather than command-service entries.

#![forbid(unsafe_code)]

mod error;

pub use error::ErrorEnvelope;
