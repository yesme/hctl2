use crate::catalog::{self, Binary, CLAUDE_VERSION};
use agency_proto::Profession;
use std::path::Path;

pub fn profession(binary: &Binary) -> Profession {
    catalog::profession(
        "claude-code",
        binary,
        "Claude Code",
        &format!(
            "native interactive via Herdr; locked Claude Code {CLAUDE_VERSION}; per-item approval is not activated; measured {}",
            binary.version
        ),
    )
}

pub fn arguments(_exec_root: &Path) -> Vec<String> {
    // Interactive CLI in the Herdr pane. Print mode and permission-prompt tools stay off.
    Vec::new()
}
