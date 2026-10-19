//! Pinned native validator, bounded and isolated; no server or workflow execution.
use crate::{Compiled, compiler::DAGU_VERSION};
use chat::reject;
use foundation::command::{CommandEnd, run_bounded};
use std::{
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use store::Result;
static NEXT: AtomicU64 = AtomicU64::new(0);
pub trait Validator {
    fn validate(&self, compiled: &Compiled) -> Result<()>;
}
pub struct NativeDagu {
    pub binary: PathBuf,
    pub scratch: PathBuf,
}
struct Temp(PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
impl Validator for NativeDagu {
    fn validate(&self, compiled: &Compiled) -> Result<()> {
        let binary = std::fs::canonicalize(&self.binary).map_err(|error| {
            reject(
                "WORKFLOW_ENGINE_UNAVAILABLE",
                format!("packaged Dagu unavailable: {error}"),
                "install_packaged_dagu",
            )
        })?;
        std::fs::create_dir_all(&self.scratch)?;
        let scratch = std::fs::canonicalize(&self.scratch)?;
        let path = scratch.join(format!(
            "workflow-validate-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path)?;
        let temp = Temp(path);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&temp.0, std::fs::Permissions::from_mode(0o700))?;
        }
        let file = temp.0.join("workflow.yaml");
        std::fs::write(&file, &compiled.yaml)?;
        for args in [
            vec!["version".to_string()],
            vec!["validate".to_string(), file.to_string_lossy().into_owned()],
        ] {
            let mut command = Command::new(&binary);
            command
                .env_clear()
                .env("HOME", &temp.0)
                .env("DAGU_HOME", temp.0.join("dagu"))
                .env("TMPDIR", &temp.0)
                .env("LANG", "C")
                .env("PATH", "/usr/bin:/bin")
                .current_dir(&temp.0)
                .args(&args);
            match run_bounded(&mut command, None, Duration::from_secs(20))? {
                CommandEnd::Finished(output) if output.status.success() => {
                    if args[0] == "version"
                        && String::from_utf8_lossy(&output.stdout).trim() != DAGU_VERSION
                    {
                        return Err(reject(
                            "WORKFLOW_ENGINE_VERSION",
                            "expected pinned Dagu 2.15.1",
                            "install_packaged_dagu",
                        ));
                    }
                }
                CommandEnd::Finished(output) => {
                    return Err(reject(
                        "WORKFLOW_ENGINE_VALIDATE",
                        format!(
                            "Dagu validation failed: {}",
                            String::from_utf8_lossy(&output.stderr)
                        ),
                        "edit_workflow",
                    ));
                }
                _ => {
                    return Err(reject(
                        "WORKFLOW_ENGINE_VALIDATE",
                        "Dagu validation exceeded time/output limit",
                        "inspect_dagu",
                    ));
                }
            }
        }
        Ok(())
    }
}
