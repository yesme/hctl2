//! Shared public contract; no control storage or runtime implementation.
#![forbid(unsafe_code)]
pub mod client;
pub mod context;
pub mod model;
pub use model::*;
pub mod wire {
    #![allow(clippy::pedantic, clippy::useless_borrows_in_formatting)]
    include!(concat!(env!("HCTL2_AGENCY_PROTO_OUT"), "/mod.rs"));
}
