//! Transport-level values shared by HCTL2 processes.
//!
//! This crate deliberately contains no governance commands or domain rules. In P1, `hctl2-tool`
//! is a standalone mechanical component rather than a command-service entry.

#![forbid(unsafe_code)]

mod error;

pub use error::ErrorEnvelope;
