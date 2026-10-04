//! Local, model-free assembly: selection ordering, permission and budget
//! gates, three-tier delivery and honest (unimplemented) metering.

use crate::{SourceKind, Sources};
use agency_proto::context::{Bundle, Delivery, Entry, Manifest};
use agency_proto::{FrozenRef, Owner, PortError, Result, Sealed, hash};
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct AssemblyRequest {
    pub manifest: Manifest,
    pub consumer: Owner,
}

#[derive(Debug)]
pub struct Assembly {
    pub manifest: Sealed<Manifest>,
    pub bundle: Sealed<Bundle>,
}

pub trait Assembler {
    fn assemble(&self, sources: &dyn Sources, request: AssemblyRequest) -> Result<Assembly>;
}

/// The default assembler. `permitted` is the permission policy point's answer
/// for this consumer (package 5 wires real policy; the placeholder returns the
/// project's admitted sources). `budget` bounds inline bytes.
pub struct LocalAssembler {
    pub permitted: BTreeSet<String>,
    pub budget: u64,
}

impl LocalAssembler {
    /// No tokenizer implementation exists in this package: token counts are
    /// reported as un-metered (None), never invented from byte counts.
    fn meter(&self) -> Option<u64> {
        None
    }
}

impl Assembler for LocalAssembler {
    fn assemble(&self, sources: &dyn Sources, request: AssemblyRequest) -> Result<Assembly> {
        let AssemblyRequest { manifest, consumer } = request;
        consumer.validate()?;
        validate_manifest(&manifest)?;

        let permission_digest =
            crate::permission_digest(&self.permitted.iter().cloned().collect::<Vec<_>>());
        if manifest.permission_digest != permission_digest {
            return Err(PortError::new(
                "PERMISSION_CHANGED",
                "manifest permission set differs from the policy point",
                "preview_again",
            ));
        }
        if manifest.budget != self.budget {
            return Err(PortError::new(
                "BUDGET_CHANGED",
                "manifest budget differs from the assembler's budget",
                "preview_again",
            ));
        }

        // Every manifest source must be in the permission set.
        for reference in &manifest.sources {
            if !self.permitted.contains(&reference.id) {
                return Err(PortError::new(
                    "PERMISSION_DENIED",
                    format!(
                        "source {} is not in this consumer's permission set",
                        reference.id
                    ),
                    "request_authorization",
                ));
            }
        }

        // Fetch every source; adapters verify the frozen version themselves.
        // Order: stable content first (room lines), high-churn later.
        let mut fetched = Vec::new();
        for reference in &manifest.sources {
            let kind = kind_of(&reference.id)?;
            let content = sources.exact(kind.clone(), reference)?;
            fetched.push((reference.clone(), content.bytes, kind));
        }
        fetched.sort_by_key(|(reference, _, kind)| order_key(kind, reference));

        let mut inline_used = 0u64;
        let mut entries = Vec::new();
        for (reference, bytes, _kind) in fetched {
            let entry_digest = hash(&bytes);
            let over_budget = inline_used + bytes.len() as u64 > self.budget;
            if over_budget {
                // Required material over budget degrades to a pointer that
                // carries the exact bytes plus a shard suggestion; it stays
                // required and is never silently dropped.
                let name = pointer_name(&reference);
                entries.push(Entry {
                    source: reference,
                    description: format!(
                        "over budget: {} bytes; shard suggestion: split or read the byte copy on demand",
                        bytes.len()
                    ),
                    required: true,
                    offline_required: true,
                    delivery: Delivery::Pointer {
                        bytes,
                        relative_name: name,
                    },
                    bytes_digest: entry_digest,
                });
            } else {
                inline_used += bytes.len() as u64;
                entries.push(Entry {
                    source: reference,
                    description: "exact source bytes".into(),
                    required: true,
                    offline_required: false,
                    delivery: Delivery::Inline { bytes },
                    bytes_digest: entry_digest,
                });
            }
        }

        let metered = self.meter();
        let manifest_digest = Sealed::new(&manifest)?.digest;
        let bundle = Bundle {
            id: bundle_id(&manifest_digest, &consumer),
            manifest: FrozenRef {
                id: manifest.id.clone(),
                revision: manifest_digest.clone(),
                digest: manifest_digest,
            },
            consumer,
            entries,
            renderer: renderer_ref(),
            tokenizer: untokenizer_ref(),
            redaction: manifest.redaction.clone(),
            compression: Vec::new(),
            candidate_tokens: metered,
            selected_tokens: metered,
            delivered_tokens: metered,
            permission_digest,
            budget: self.budget,
            retention: "until-owner-terminal-and-admission-window-closed".into(),
        };
        bundle.validate_delivery()?;
        let assembly = Assembly {
            manifest: Sealed::new(manifest)?,
            bundle: Sealed::new(bundle)?,
        };
        assembly.bundle.verify()?;
        assembly.manifest.verify()?;
        Ok(assembly)
    }
}

/// The bundle identity includes the consumer's generation so the next
/// generation of the same owner never collides with a frozen bundle.
pub fn bundle_id(manifest_digest: &str, consumer: &Owner) -> String {
    format!(
        "bundle-{}",
        hash(
            format!(
                "{manifest_digest}:{}:{:?}:{}:{}",
                consumer.project, consumer.kind, consumer.id, consumer.generation
            )
            .as_bytes()
        )
    )
}

fn renderer_ref() -> FrozenRef {
    FrozenRef {
        id: "renderer/mechanical-v1".into(),
        revision: "1".into(),
        digest: hash(b"hctl2.context.renderer.mechanical.v1"),
    }
}

fn untokenizer_ref() -> FrozenRef {
    FrozenRef {
        id: "tokenizer/none".into(),
        revision: "1".into(),
        digest: hash(b"hctl2.context.tokenizer.none.v1"),
    }
}

fn pointer_name(reference: &FrozenRef) -> String {
    let safe: String = reference
        .id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    // A short id-hash suffix keeps two sources that sanitize to the same
    // text from colliding into one file name.
    let suffix = &hash(reference.id.as_bytes())[..12];
    format!("{safe}-{suffix}.bin")
}

fn kind_of(packed: &str) -> Result<SourceKind> {
    let (kind, _) = packed
        .split_once('/')
        .ok_or_else(|| PortError::invalid("reference must be kind/id"))?;
    match kind {
        crate::sources::ROOM_LINE_KIND | crate::sources::ROOM_BRIEF_KIND => Ok(SourceKind::Room),
        crate::sources::TASK_LINE_KIND => Ok(SourceKind::TaskComments),
        "review_comments" => Ok(SourceKind::ReviewComments),
        other => Err(PortError::invalid(format!("unknown source kind {other}"))),
    }
}

fn order_key(kind: &SourceKind, reference: &FrozenRef) -> (u8, String) {
    let rank = match kind {
        SourceKind::Room if reference.id.starts_with("room_binding/") => 0,
        SourceKind::Room => 1,
        SourceKind::TaskComments => 2,
        SourceKind::ReviewComments => 3,
    };
    (rank, reference.id.clone())
}

/// Manifest completeness: every CT-named mandatory field must be present.
pub(crate) fn validate_manifest(manifest: &Manifest) -> Result<()> {
    agency_proto::nonempty(&manifest.id)?;
    agency_proto::nonempty(&manifest.purpose)?;
    agency_proto::nonempty(&manifest.scope)?;
    agency_proto::nonempty(&manifest.freshness)?;
    agency_proto::nonempty(&manifest.coverage)?;
    manifest.selection_policy.validate()?;
    manifest.redaction.validate()?;
    if let Some(parent) = &manifest.parent {
        parent.validate()?;
    }
    agency_proto::digest(&manifest.permission_digest)?;
    if manifest.budget == 0 {
        return Err(PortError::invalid("budget must be positive"));
    }
    for source in &manifest.sources {
        source.validate()?;
    }
    for skill in &manifest.required_skills {
        skill.validate()?;
    }
    if manifest.sources.is_empty() {
        return Err(PortError::invalid("manifest needs at least one source"));
    }
    Ok(())
}
