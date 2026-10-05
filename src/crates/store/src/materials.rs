use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use foundation::command::{CommandEnd, run_bounded};
use foundation::{canonical_json, canonical_json_sha256, git};
use serde::{Deserialize, Serialize};

use crate::model::{digest, nonempty};
use crate::{Result, Scope, StoreError};

/// Local git plumbing for the material repository, including mirror copy.
const MATERIAL_COMMAND_TIMEOUT: Duration = Duration::from_secs(90);

fn bounded(
    mut command: std::process::Command,
    input: Option<&[u8]>,
    time_limit: Duration,
) -> Result<std::process::Output> {
    match run_bounded(&mut command, input, time_limit)? {
        CommandEnd::Finished(output) => Ok(output),
        CommandEnd::TimedOut => Err(missing("material command timed out")),
        CommandEnd::OutputLimit => Err(missing("material command output exceeded limit")),
    }
}

/// Stable control/scope/content reference. No backend path or Git object ID crosses this boundary.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MaterialRef {
    pub control_id: String,
    pub scope: Scope,
    pub material_id: String,
    /// SHA-256 of the exact bytes, distinct from a domain Revision's JCS digest.
    pub byte_digest: String,
}

/// A trusted reducer's exact material grant, not general access to the materials repository.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeliveryGrant {
    pub recipient: String,
    pub purpose: String,
    pub materials: Vec<MaterialRef>,
}

#[derive(Serialize, Deserialize)]
struct Candidate {
    control_id: String,
    scope: Scope,
    command_key: String,
    slot: String,
    byte_digest: String,
}

pub(crate) struct Materials {
    path: PathBuf,
    executable: PathBuf,
}

impl Materials {
    pub(crate) fn open(path: &Path, create: bool) -> Result<Self> {
        let requested = std::env::var_os("HCTL2_GIT").unwrap_or_else(|| OsString::from("git"));
        let executable = git::resolve_executable(&requested).ok_or_else(|| {
            StoreError::new("GIT_UNAVAILABLE", "host Git not found", "install_git")
        })?;
        let mut version_command = git::sanitized_command(&executable);
        version_command.arg("--version");
        let output = bounded(version_command, None, MATERIAL_COMMAND_TIMEOUT)?;
        let version = git::parse_version(&String::from_utf8_lossy(&output.stdout));
        if !output.status.success()
            || !version.is_some_and(|(major, minor, _)| git::version_supported(major, minor))
        {
            return Err(StoreError::new(
                "GIT_VERSION",
                "host Git 2.39 or newer required",
                "install_git",
            ));
        }
        if !path.exists() {
            if !create {
                return Err(missing("material repository is missing"));
            }
            private_dir(path)?;
            let mut init = git::sanitized_command(&executable);
            init.args(["init", "--bare", "--template=", "--object-format=sha1"])
                .arg(path);
            let output = bounded(init, None, MATERIAL_COMMAND_TIMEOUT)?;
            if !output.status.success() {
                return Err(missing("cannot initialize material repository"));
            }
        }
        let result = Self {
            path: path.to_path_buf(),
            executable,
        };
        if result.text(&["rev-parse", "--is-bare-repository"], None)? != "true" {
            return Err(missing("material backend is not a bare repository"));
        }
        Ok(result)
    }

    /// Each immutable candidate has a durable ref, including its original command and scope.
    /// Keeping all refs is conservative retention; no automatic application-level GC is enabled.
    pub(crate) fn save(
        &self,
        control_id: &str,
        scope: &Scope,
        command_key: &str,
        slot: &str,
        bytes: &[u8],
    ) -> Result<MaterialRef> {
        scope.validate()?;
        nonempty(command_key, "candidate command key")?;
        nonempty(slot, "candidate slot")?;
        let metadata = Candidate {
            control_id: control_id.into(),
            scope: scope.clone(),
            command_key: command_key.into(),
            slot: slot.into(),
            byte_digest: hash(bytes),
        };
        let value = serde_json::to_value(&metadata)?;
        let id = canonical_json_sha256(&value)?;
        let reference = MaterialRef {
            control_id: control_id.into(),
            scope: scope.clone(),
            material_id: id.clone(),
            byte_digest: metadata.byte_digest.clone(),
        };
        let body = self.text(&["hash-object", "-w", "--stdin"], Some(bytes))?;
        let header = self.text(
            &["hash-object", "-w", "--stdin"],
            Some(&canonical_json(&value)?),
        )?;
        let tree = self.text(
            &["mktree"],
            Some(format!("100644 blob {body}\tbody\n100644 blob {header}\tmetadata\n").as_bytes()),
        )?;
        let commit = self.text(&["commit-tree", &tree], Some(b"HCTL control material\n"))?;
        let name = material_ref_name(&id)?;
        // A deterministic immutable ref; a replay can only reaffirm the same value.
        let old = self.text(&["rev-parse", "--verify", &name], None).ok();
        if old.as_deref().is_some_and(|old| old != commit) {
            return Err(missing("immutable candidate ref was changed"));
        }
        self.run(
            &[
                "update-ref",
                &name,
                &commit,
                old.as_deref()
                    .unwrap_or("0000000000000000000000000000000000000000"),
            ],
            None,
        )?;
        self.read(&reference)?;
        Ok(reference)
    }

    pub(crate) fn read(&self, reference: &MaterialRef) -> Result<Vec<u8>> {
        digest(&reference.byte_digest)?;
        reference.scope.validate()?;
        let name = material_ref_name(&reference.material_id)?;
        let header = self.run(&["cat-file", "blob", &format!("{name}:metadata")], None)?;
        let candidate: Candidate = serde_json::from_slice(&header)?;
        if canonical_json_sha256(&serde_json::to_value(&candidate)?)? != reference.material_id
            || candidate.control_id != reference.control_id
            || candidate.scope != reference.scope
            || candidate.byte_digest != reference.byte_digest
        {
            return Err(StoreError::new(
                "MATERIAL_DIGEST_MISMATCH",
                "material locator and frozen reference disagree",
                "restore_material",
            ));
        }
        let bytes = self.run(&["cat-file", "blob", &format!("{name}:body")], None)?;
        if hash(&bytes) != reference.byte_digest {
            return Err(StoreError::new(
                "MATERIAL_DIGEST_MISMATCH",
                "material bytes do not match",
                "restore_material",
            ));
        }
        Ok(bytes)
    }

    pub(crate) fn command_key(&self, reference: &MaterialRef) -> Result<String> {
        let name = material_ref_name(&reference.material_id)?;
        let bytes = self.run(&["cat-file", "blob", &format!("{name}:metadata")], None)?;
        Ok(serde_json::from_slice::<Candidate>(&bytes)?.command_key)
    }

    pub(crate) fn export(&self, destination: &Path) -> Result<()> {
        // --no-hardlinks makes this a separate backup, never dependent on the live object files.
        let mut clone = self.command();
        clone
            .args(["clone", "--mirror", "--no-hardlinks", "--no-local"])
            .arg(&self.path)
            .arg(destination);
        let output = bounded(clone, None, MATERIAL_COMMAND_TIMEOUT)?;
        if !output.status.success() {
            return Err(missing("material backup copy failed"));
        }
        Self::open(destination, false)?.run(&["fsck", "--full", "--no-reflogs"], None)?;
        sync_tree(destination)?;
        Ok(())
    }

    pub(crate) fn import(&self, source: &Path) -> Result<()> {
        // Append-only refs: preserve candidates/newer local byte copies even when restoring old records.
        let source = Self::open(source, false)?;
        let refs = source.text(
            &[
                "for-each-ref",
                "--format=%(objectname) %(refname)",
                "refs/hctl2/materials/",
            ],
            None,
        )?;
        for line in refs.lines() {
            let (oid, name) = line
                .split_once(' ')
                .ok_or_else(|| missing("invalid material ref"))?;
            let id = name
                .strip_prefix("refs/hctl2/materials/")
                .ok_or_else(|| missing("unexpected material ref"))?;
            digest(id)?;
            if let Ok(existing) = self.text(&["rev-parse", "--verify", name], None)
                && existing != oid
            {
                return Err(missing("backup conflicts with immutable material"));
            }
        }
        let mut fetch = self.command();
        fetch
            .arg("-C")
            .arg(&self.path)
            .args(["fetch", "--no-tags", "--no-write-fetch-head"])
            .arg(&source.path)
            .arg("refs/hctl2/materials/*:refs/hctl2/materials/*");
        let output = bounded(fetch, None, MATERIAL_COMMAND_TIMEOUT)?;
        if !output.status.success() {
            return Err(missing("material restore failed"));
        }
        Ok(())
    }

    fn command(&self) -> std::process::Command {
        let mut command = git::sanitized_command(&self.executable);
        // The private store has no user worktree configuration. Avoid global hooks, replacements
        // and maintenance; these plumbing calls never perform external delivery.
        command
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_NO_REPLACE_OBJECTS", "1")
            .env("GIT_AUTHOR_NAME", "HCTL material store")
            .env("GIT_AUTHOR_EMAIL", "material@hctl.invalid")
            .env("GIT_COMMITTER_NAME", "HCTL material store")
            .env("GIT_COMMITTER_EMAIL", "material@hctl.invalid")
            .env("GIT_AUTHOR_DATE", "2000-01-01T00:00:00Z")
            .env("GIT_COMMITTER_DATE", "2000-01-01T00:00:00Z")
            .args([
                "-c",
                "core.fsync=loose-object,pack,reference",
                "-c",
                "core.fsyncMethod=fsync",
                "-c",
                "core.hooksPath=/dev/null",
                "-c",
                "gc.auto=0",
                "-c",
                "maintenance.auto=false",
            ]);
        command
    }

    pub(crate) fn run(&self, args: &[&str], input: Option<&[u8]>) -> Result<Vec<u8>> {
        let mut command = self.command();
        command.arg("-C").arg(&self.path).args(args);
        let output = bounded(command, input, MATERIAL_COMMAND_TIMEOUT)?;
        if !output.status.success() {
            return Err(missing(
                "Git could not resolve or persist the precise material",
            ));
        }
        Ok(output.stdout)
    }

    fn text(&self, args: &[&str], input: Option<&[u8]>) -> Result<String> {
        String::from_utf8(self.run(args, input)?)
            .map(|s| s.trim_end().to_owned())
            .map_err(|_| missing("invalid Git response"))
    }
}

fn material_ref_name(id: &str) -> Result<String> {
    digest(id)?;
    Ok(format!("refs/hctl2/materials/{id}"))
}

pub(crate) fn hash(bytes: &[u8]) -> String {
    foundation::bytes_sha256(bytes)
}

fn missing(message: &str) -> StoreError {
    StoreError::new("MATERIAL_UNAVAILABLE", message, "restore_material")
}

pub(crate) fn private_dir(path: &Path) -> Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)?;
    Ok(())
}

pub(crate) fn sync_tree(path: &Path) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            sync_tree(&entry.path())?;
        } else if entry.file_type()?.is_file() {
            fs::File::open(entry.path())?.sync_all()?;
        } else {
            return Err(StoreError::invalid("backup contains a non-regular entry"));
        }
    }
    fs::File::open(path)?.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    use super::bounded;

    fn shell_quote(path: &std::path::Path) -> String {
        format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
    }

    #[test]
    fn a_stuck_material_command_is_unreadable_within_the_limit() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("hctl2-material-stuck-{nanos}"));
        fs::create_dir(&dir).unwrap();
        let program = dir.join("hang.sh");
        let pidfile = dir.join("pid");
        fs::write(
            &program,
            format!(
                "#!/bin/sh\necho $$ > {}\nexec sleep 30\n",
                shell_quote(&pidfile)
            ),
        )
        .unwrap();
        fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
        let started = Instant::now();
        let mut command = Command::new("/bin/sh");
        command.arg(&program);
        let error = bounded(command, None, Duration::from_secs(3))
            .expect_err("a stuck command must not succeed");
        assert!(
            started.elapsed() < Duration::from_secs(7),
            "{:?}",
            started.elapsed()
        );
        assert_eq!(error.code, "MATERIAL_UNAVAILABLE");
        assert!(error.message.contains("timed out"), "{}", error.message);
        let pid: u32 = fs::read_to_string(&pidfile)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        let mut listed = Command::new("ps");
        listed.args(["-p", &pid.to_string(), "-o", "pid="]);
        let listed = bounded(listed, None, Duration::from_secs(5)).expect("ps");
        assert!(
            listed.stdout.iter().all(u8::is_ascii_whitespace),
            "pid {pid}"
        );
        fs::remove_dir_all(dir).unwrap();
    }
}
