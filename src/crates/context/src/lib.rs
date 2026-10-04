//! Package 4's assembler boundary; records are shared with the Agency port.
//!
//! Assembly is local and mechanical: no model calls, no summaries, no
//! compression. Small-brain integration is a later engine concern; here a
//! missing tokenizer is reported as un-metered, never invented.
#![forbid(unsafe_code)]

mod assembler;
mod records;
mod sources;

pub use agency_proto::context::*;
pub use agency_proto::{FrozenRef, Owner, Result, Sealed};

pub use assembler::{Assembler, Assembly, AssemblyRequest, LocalAssembler};
pub use records::{bundle_record, manifest_record, read_bundle, read_manifest, save_assembly};
pub use sources::{
    MemorySources, ReviewComments, StoreSources, frozen_from_record, pack_reference,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceKind {
    Room,
    TaskComments,
    ReviewComments,
}

/// One chosen source: the exact frozen reference plus delivery intent.
#[derive(Clone, Debug)]
pub struct Selection {
    pub reference: FrozenRef,
    pub description: String,
    /// Every-call material the consumer cannot fetch itself.
    pub required: bool,
    /// Must be readable offline: an actual byte copy is delivered and verified.
    pub offline_required: bool,
    /// Prefer inline bytes over a pointer when the budget allows.
    pub prefer_inline: bool,
}

#[derive(Clone, Debug)]
pub struct SourceContent {
    pub reference: FrozenRef,
    pub bytes: Vec<u8>,
}

pub trait Sources {
    /// Exact version and bytes for a frozen reference. A reference whose
    /// current version differs is an error, never a silent older preview.
    fn exact(&self, kind: SourceKind, reference: &FrozenRef) -> Result<SourceContent>;
}
