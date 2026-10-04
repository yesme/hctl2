use crate::catalog::{self, Binary, CODEX_VERSION};
use agency_proto::Profession;
use std::path::Path;

pub fn profession(binary: &Binary) -> Profession {
    catalog::profession(
        "codex",
        binary,
        "Codex CLI",
        &format!(
            "native interactive via Herdr; locked Codex CLI {CODEX_VERSION}; per-item approval is not activated; measured {}",
            binary.version
        ),
    )
}

pub fn arguments(_exec_root: &Path) -> Vec<String> {
    // Interactive CLI in the Herdr pane. `exec` and approval callbacks stay off.
    Vec::new()
}
