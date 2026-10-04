//! Exact harness and skill references. A declaration is not a verification.
use agency_proto::{
    Capabilities, Catalog, FrozenRef, PortError, Profession, Result, SkillClaim, hash,
};
use std::{fs, path::Path, process::Command};

pub const HERDR_PROTOCOL: u32 = 20;
pub const HERDR_VERSION: &str = "0.8.2";
pub const CODEX_VERSION: &str = "0.153.4";
pub const CLAUDE_VERSION: &str = "2.1.263";

pub fn file_digest(path: &Path) -> Result<String> {
    Ok(hash(&fs::read(path)?))
}

pub fn skill_claims(dir: &Path) -> Result<Vec<SkillClaim>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut claims = Vec::new();
    let mut names = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect::<Vec<_>>();
    names.sort();
    for skill in names {
        let text = skill.join("SKILL.md");
        if !text.is_file() {
            continue;
        }
        let id = skill
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("skill")
            .to_owned();
        claims.push(SkillClaim {
            reference: FrozenRef {
                id,
                revision: "file".into(),
                digest: file_digest(&text)?,
            },
            required: false,
            verification: None,
        });
    }
    Ok(claims)
}

#[derive(Clone, Debug)]
pub struct Binary {
    pub path: std::path::PathBuf,
    pub version: String,
    pub digest: String,
    pub locked: bool,
}

pub fn describe(path: &Path, locked_version: &str) -> Result<Binary> {
    let digest = file_digest(path)?;
    let output = Command::new(path).arg("--version").output()?;
    let version = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_owned();
    if version.is_empty() {
        return Err(PortError::new(
            "HARNESS_VERSION_UNREAD",
            "harness version command produced no version",
            "install_locked_harness",
        ));
    }
    Ok(Binary {
        locked: version.split_whitespace().any(|w| w == locked_version)
            || version.contains(locked_version),
        path: path.to_path_buf(),
        version,
        digest,
    })
}

pub fn reference(id: &str, revision: &str, digest: String) -> FrozenRef {
    FrozenRef {
        id: id.into(),
        revision: revision.into(),
        digest,
    }
}

/// Native interactive only. Per-item approval is not offered.
pub fn native_capabilities() -> Capabilities {
    Capabilities {
        input: true,
        stop: true,
        ..Capabilities::default()
    }
}

pub fn script_capabilities() -> Capabilities {
    Capabilities {
        input: true,
        stop: true,
        event_cursor: true,
        input_provenance: true,
        managed_single_writer: true,
        ..Capabilities::default()
    }
}

pub fn profession(id: &str, binary: &Binary, persona: &str, terms: &str) -> Profession {
    let harness = reference(id, &binary.version, binary.digest.clone());
    Profession {
        reference: harness.clone(),
        harness,
        model: "provider-default".into(),
        persona: persona.into(),
        terms: terms.into(),
        default_role: "worker".into(),
        skills: vec![],
        capabilities: native_capabilities(),
    }
}

pub fn catalog(professions: Vec<Profession>, skills: Vec<SkillClaim>) -> Catalog {
    let harnesses = professions.iter().map(|p| p.harness.clone()).collect();
    Catalog {
        professions,
        harnesses,
        skills,
    }
}
