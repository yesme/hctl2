use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::ToolError;

const MINIMUM_GIT_MAJOR: u64 = 2;
const MINIMUM_GIT_MINOR: u64 = 39;

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
        let mut command = sanitized_command(&self.executable);
        command.arg("-C").arg(repository).args(arguments);
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

fn sanitized_command(executable: &Path) -> Command {
    let mut command = Command::new(executable);
    for name in [
        "GIT_DIR",
        "GIT_COMMON_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_NAMESPACE",
        "GIT_CONFIG_COUNT",
        "GIT_CONFIG_PARAMETERS",
    ] {
        command.env_remove(name);
    }
    for (name, _) in std::env::vars_os() {
        let name_text = name.to_string_lossy();
        if name_text.starts_with("GIT_CONFIG_KEY_") || name_text.starts_with("GIT_CONFIG_VALUE_") {
            command.env_remove(name);
        }
    }
    command
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("LC_ALL", "C");
    command
}

fn resolve_executable(requested: &OsStr) -> Option<PathBuf> {
    let requested_path = Path::new(requested);
    if requested_path.is_absolute() || requested_path.components().count() > 1 {
        return requested_path
            .canonicalize()
            .ok()
            .filter(|path| path.is_file());
    }

    let search_path = std::env::var_os("PATH")?;
    for directory in std::env::split_paths(&search_path) {
        let candidate = directory.join(requested_path);
        if let Ok(canonical) = candidate.canonicalize()
            && canonical.is_file()
        {
            return Some(canonical);
        }
        #[cfg(windows)]
        {
            let candidate = directory.join(format!("{}.exe", requested.to_string_lossy()));
            if let Ok(canonical) = candidate.canonicalize()
                && canonical.is_file()
            {
                return Some(canonical);
            }
        }
    }
    None
}

fn parse_version(banner: &str) -> Option<(u64, u64, String)> {
    let version = banner
        .trim()
        .strip_prefix("git version ")?
        .split_whitespace()
        .next()?;
    let mut components = version.split('.');
    let major = components.next()?.parse().ok()?;
    let minor = components.next()?.parse().ok()?;
    Some((major, minor, version.to_owned()))
}

const fn version_supported(major: u64, minor: u64) -> bool {
    major > MINIMUM_GIT_MAJOR || (major == MINIMUM_GIT_MAJOR && minor >= MINIMUM_GIT_MINOR)
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
