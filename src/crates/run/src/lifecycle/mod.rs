//! Run command ownership. Engines execute committed effects, never govern results.
mod admission;
pub mod engine;
mod freeze;
mod model;
pub use admission::*;
pub use freeze::delivered_content_digest;
pub use model::*;

/// Shared startup/voting content contract; copying consumer metadata or a digest
/// from another consumer never authorizes delivery of different bytes.
pub fn validate_delivery_contract(
    manifest: &Manifest,
    selected: &Selected,
    bundle: &agency_proto::Sealed<agency_proto::context::Bundle>,
) -> store::Result<()> {
    bundle.verify().map_err(|e| chat::invalid(e.to_string()))?;
    bundle
        .document
        .validate_delivery()
        .map_err(|e| chat::invalid(e.to_string()))?;
    if bundle.document.consumer.project != manifest.project.key.id
        || bundle.document.consumer.kind != agency_proto::OwnerKind::RunAttempt
        || bundle.document.manifest.id != manifest.context.reference.key.id
        || bundle.document.manifest.digest != manifest.context.digest
        || bundle.document.budget > selected.max_context_bytes
    {
        return Err(chat::invalid(
            "Attempt Bundle differs from frozen Run context or budget",
        ));
    }
    if let Some(expected) = &selected.frozen_content_digest {
        let skills: Vec<_> = selected
            .required_skills
            .iter()
            .map(|s| s.reference.clone())
            .collect();
        if delivered_content_digest(&bundle.document, &skills)?.as_ref() != Some(expected) {
            return Err(chat::reject(
                "BUNDLE_CONTRACT_CHANGED",
                "Attempt delivery differs from frozen Seat content",
                "replace_run",
            ));
        }
    }
    Ok(())
}
