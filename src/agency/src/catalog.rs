//! Digests of files the Agency actually has. A declaration is not a verification.
use agency_proto::{FrozenRef, Result, SkillClaim, hash};
use std::{fs, path::Path};

pub fn file_digest(path: &Path) -> Result<String> {
    Ok(hash(&fs::read(path)?))
}

pub fn skill_claims(dir: &Path) -> Result<Vec<SkillClaim>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut claims = Vec::new();
    let mut names = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    names.sort();
    for skill in names {
        let text = skill.join("SKILL.md");
        if !text.is_file() {
            continue;
        }
        let id = skill
            .file_name()
            .and_then(|name| name.to_str())
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
