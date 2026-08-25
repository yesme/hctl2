//! Standalone harness and runtime process.
//!
//! Agentd owns physical process, PTY, stream, and host observations. It is not a governance
//! command entry and does not decide Project, Task, Run, or Agent domain outcomes.

#![forbid(unsafe_code)]

use std::ffi::{OsStr, OsString};

use hctl2_protocol::ErrorEnvelope;

/// Installed executable name.
pub const PROGRAM_NAME: &str = "hctl2-agentd";

const UNSUPPORTED_ARGUMENT: &str = "HCTL2_AGENTD_UNSUPPORTED_ARGUMENT";

/// Runs the current P1 command surface.
///
/// Only process introspection is exposed by the scaffold. Runtime and harness operations arrive
/// with their real contracts rather than as placeholder handlers.
///
/// # Errors
///
/// Returns a stable [`ErrorEnvelope`] when the argument count or argument value is unsupported.
pub fn run(arguments: impl IntoIterator<Item = OsString>) -> Result<String, ErrorEnvelope> {
    let mut arguments = arguments.into_iter();
    let first = arguments.next();

    if arguments.next().is_some() {
        return Err(ErrorEnvelope::new(
            UNSUPPORTED_ARGUMENT,
            "expected at most one argument; use --help",
        ));
    }

    match first.as_deref() {
        None => Ok(help()),
        Some(value) if value == OsStr::new("--help") || value == OsStr::new("-h") => Ok(help()),
        Some(value) if value == OsStr::new("--version") || value == OsStr::new("-V") => {
            Ok(version())
        }
        Some(value) => Err(ErrorEnvelope::new(
            UNSUPPORTED_ARGUMENT,
            format!(
                "unsupported argument {:?}; use --help",
                value.to_string_lossy()
            ),
        )),
    }
}

fn version() -> String {
    format!("{PROGRAM_NAME} {}", env!("CARGO_PKG_VERSION"))
}

fn help() -> String {
    format!(
        "{PROGRAM_NAME} {}\n\nHarness and runtime process (P1 scaffold).\n\nUsage:\n  {PROGRAM_NAME} --help\n  {PROGRAM_NAME} --version",
        env!("CARGO_PKG_VERSION")
    )
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::run;

    #[test]
    fn reports_version() {
        let output = run([OsString::from("--version")]);

        assert_eq!(
            output,
            Ok(format!("hctl2-agentd {}", env!("CARGO_PKG_VERSION")))
        );
    }

    #[test]
    fn rejects_unknown_arguments_with_a_stable_code() {
        let error = run([OsString::from("not-a-command")]).expect_err("argument must be rejected");

        assert_eq!(error.code(), "HCTL2_AGENTD_UNSUPPORTED_ARGUMENT");
    }
}
