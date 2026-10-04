//! Package 4's assembler boundary; records are shared with the Agency port.
#![forbid(unsafe_code)]
pub use agency_proto::context::*;
pub use agency_proto::{FrozenRef, Owner, Result, Sealed};
pub enum SourceKind {
    Room,
    TaskComments,
    ReviewComments,
}
pub struct SourceContent {
    pub reference: FrozenRef,
    pub bytes: Vec<u8>,
}
pub trait Sources {
    fn exact(&self, kind: SourceKind, reference: &FrozenRef) -> Result<SourceContent>;
}
pub struct AssemblyRequest {
    pub manifest: Manifest,
    pub consumer: Owner,
}
pub struct Assembly {
    pub manifest: Sealed<Manifest>,
    pub bundle: Sealed<Bundle>,
}
pub trait Assembler {
    fn assemble(&self, sources: &dyn Sources, request: AssemblyRequest) -> Result<Assembly>;
}
