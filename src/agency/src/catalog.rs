//! Digests of files the Agency actually has. A declaration is not a verification.
use agency_proto::{FrozenRef, Result, SkillClaim, hash};
use std::{fs, path::Path, process::Command};

/// In-process SHA-256 of a debug build is too slow for the Codex standalone
/// binary. The server does not listen until this digest returns, and the
/// ready budget is shorter than that hash. The platform tool reads the file
/// itself and returns the same SHA-256.
const PLATFORM_DIGEST_MIN_BYTES: u64 = 32 * 1024 * 1024;

pub fn file_digest(path: &Path) -> Result<String> {
    if fs::metadata(path)?.len() > PLATFORM_DIGEST_MIN_BYTES
        && let Some(digest) = platform_digest(path)
    {
        return Ok(digest);
    }
    Ok(hash(&fs::read(path)?))
}

fn platform_digest(path: &Path) -> Option<String> {
    let output = if cfg!(target_os = "linux") {
        Command::new("/usr/bin/sha256sum").arg(path).output().ok()?
    } else if cfg!(target_os = "macos") {
        Command::new("/usr/bin/shasum")
            .args(["-a", "256"])
            .arg(path)
            .output()
            .ok()?
    } else {
        return None;
    };
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let digest = text.split_whitespace().next()?;
    (digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())).then(|| {
        digest
            .chars()
            .map(|char| char.to_ascii_lowercase())
            .collect()
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn large_file_digest_matches_in_process_sha256() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        let bytes = vec![0u8; usize::try_from(PLATFORM_DIGEST_MIN_BYTES).unwrap() + 1];
        file.write_all(&bytes).unwrap();
        assert_eq!(file_digest(file.path()).unwrap(), hash(&bytes));
    }
}
