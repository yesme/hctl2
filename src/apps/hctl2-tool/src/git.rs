use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Output;

use crate::ToolError;

use hctl2_foundation::git::{
    parse_version, resolve_executable, sanitized_command, version_supported,
};

#[derive(Clone, Debug)]
pub(crate) struct Git {
    executable: PathBuf,
    version: String,
}

#[derive(Debug)]
pub(crate) struct GitOutput {
    output: Output,
}

impl Git {
    pub(crate) fn discover() -> Result<Self, ToolError> {
        let requested = std::env::var_os("HCTL2_GIT").unwrap_or_else(|| OsString::from("git"));
        let executable = resolve_executable(&requested).ok_or_else(|| {
            ToolError::new(
                "HCTL2_TOOL_GIT_NOT_FOUND",
                format!(
                    "could not resolve Git executable {:?}; set HCTL2_GIT to an executable path",
                    requested
                ),
            )
        })?;
        let output = sanitized_command(&executable)
            .arg("--version")
            .output()
            .map_err(|error| {
                ToolError::new(
                    "HCTL2_TOOL_GIT_NOT_EXECUTABLE",
                    format!("could not execute {}: {error}", executable.display()),
                )
            })?;
        if !output.status.success() {
            return Err(ToolError::new(
                "HCTL2_TOOL_GIT_NOT_EXECUTABLE",
                format!(
                    "{} --version failed: {}",
                    executable.display(),
                    stderr_summary(&output.stderr)
                ),
            ));
        }
        let banner = String::from_utf8(output.stdout).map_err(|_| {
            ToolError::new(
                "HCTL2_TOOL_GIT_OUTPUT_INVALID",
                "git --version did not return UTF-8",
            )
        })?;
        let version = parse_version(&banner).ok_or_else(|| {
            ToolError::new(
                "HCTL2_TOOL_GIT_OUTPUT_INVALID",
                format!("could not parse git --version output: {}", banner.trim()),
            )
        })?;
        if !version_supported(version.0, version.1) {
            return Err(ToolError::new(
                "HCTL2_TOOL_GIT_VERSION_UNSUPPORTED",
                format!(
                    "Git {} is below the required 2.39 minimum ({})",
                    version.2,
                    executable.display()
                ),
            ));
        }
        Ok(Self {
            executable,
            version: version.2,
        })
    }

    pub(crate) fn executable(&self) -> &Path {
        &self.executable
    }

    pub(crate) fn version(&self) -> &str {
        &self.version
    }

    pub(crate) fn invoke(
        &self,
        repository: &Path,
        arguments: &[OsString],
    ) -> Result<GitOutput, ToolError> {
        self.invoke_with_env(repository, arguments, &[])
    }

    pub(crate) fn invoke_with_env(
        &self,
        repository: &Path,
        arguments: &[OsString],
        extra_env: &[(&str, OsString)],
    ) -> Result<GitOutput, ToolError> {
        let mut command = sanitized_command(&self.executable);
        command.arg("-C").arg(repository).args(arguments);
        for (name, value) in extra_env {
            command.env(name, value);
        }
        let output = command.output().map_err(|error| {
            ToolError::new(
                "HCTL2_TOOL_GIT_COMMAND_FAILED",
                format!("could not run {}: {error}", self.executable.display()),
            )
        })?;
        Ok(GitOutput { output })
    }

    pub(crate) fn checked(
        &self,
        repository: &Path,
        arguments: &[OsString],
        code: &'static str,
        operation: &str,
    ) -> Result<GitOutput, ToolError> {
        let output = self.invoke(repository, arguments)?;
        if output.success() {
            Ok(output)
        } else {
            Err(ToolError::new(
                code,
                format!("{operation} failed: {}", output.stderr()),
            ))
        }
    }

    pub(crate) fn checked_with_env(
        &self,
        repository: &Path,
        arguments: &[OsString],
        extra_env: &[(&str, OsString)],
        code: &'static str,
        operation: &str,
    ) -> Result<GitOutput, ToolError> {
        let output = self.invoke_with_env(repository, arguments, extra_env)?;
        if output.success() {
            Ok(output)
        } else {
            Err(ToolError::new(
                code,
                format!("{operation} failed: {}", output.stderr()),
            ))
        }
    }
}

impl GitOutput {
    pub(crate) fn success(&self) -> bool {
        self.output.status.success()
    }

    pub(crate) fn code(&self) -> Option<i32> {
        self.output.status.code()
    }

    pub(crate) fn stdout(&self) -> &[u8] {
        &self.output.stdout
    }

    pub(crate) fn stdout_text(&self) -> Result<String, ToolError> {
        String::from_utf8(self.output.stdout.clone())
            .map(|value| value.trim_end_matches(['\r', '\n']).to_owned())
            .map_err(|_| {
                ToolError::new(
                    "HCTL2_TOOL_GIT_OUTPUT_INVALID",
                    "git command returned non-UTF-8 text",
                )
            })
    }

    pub(crate) fn stderr(&self) -> String {
        stderr_summary(&self.output.stderr)
    }
}

fn stderr_summary(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        "git returned no diagnostic".to_owned()
    } else {
        trimmed.chars().take(2_000).collect()
    }
}

pub(crate) fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}

#[cfg(test)]
mod tests {
    use super::{parse_version, version_supported};

    #[test]
    fn parses_upstream_and_apple_versions() {
        assert_eq!(
            parse_version("git version 2.50.1 (Apple Git-155)\n"),
            Some((2, 50, "2.50.1".to_owned()))
        );
        assert_eq!(
            parse_version("git version 2.39.5\n"),
            Some((2, 39, "2.39.5".to_owned()))
        );
    }

    #[test]
    fn enforces_the_git_2_39_floor() {
        assert!(!version_supported(2, 38));
        assert!(version_supported(2, 39));
        assert!(version_supported(3, 0));
    }
}
