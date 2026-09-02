//! Standalone mechanical Git and SCM toolbox.
//!
//! P1 does not grant this process governance authority. When control arrives in P2, it will ask
//! the tool to execute already-persisted intents and will independently verify readback.

#![forbid(unsafe_code)]

use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Display, Formatter};

/// Stable tool-local error code plus a human-readable explanation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolError {
    code: &'static str,
    message: String,
}

impl ToolError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Returns the stable machine-readable error code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    /// Returns the human-readable explanation.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for ToolError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl Error for ToolError {}

/// Installed executable name.
pub const PROGRAM_NAME: &str = "hctl2-tool";

const UNSUPPORTED_ARGUMENT: &str = "HCTL2_TOOL_UNSUPPORTED_ARGUMENT";

/// Runs the current P1 command surface.
///
/// Only process introspection is exposed by the scaffold. Git mutation commands arrive with their
/// real standalone contracts rather than as placeholder handlers.
///
/// # Errors
///
/// Returns a stable [`ToolError`] when the argument count or argument value is unsupported.
pub fn run(arguments: impl IntoIterator<Item = OsString>) -> Result<String, ToolError> {
    let mut arguments = arguments.into_iter();
    let first = arguments.next();

    if arguments.next().is_some() {
        return Err(ToolError::new(
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
        Some(value) => Err(ToolError::new(
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
        "{PROGRAM_NAME} {}\n\nMechanical Git/SCM toolbox (P1 scaffold).\n\nUsage:\n  {PROGRAM_NAME} --help\n  {PROGRAM_NAME} --version",
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
            Ok(format!("hctl2-tool {}", env!("CARGO_PKG_VERSION")))
        );
    }

    #[test]
    fn rejects_unknown_arguments_with_a_stable_code() {
        let error = run([OsString::from("not-a-command")]).expect_err("argument must be rejected");

        assert_eq!(error.code(), "HCTL2_TOOL_UNSUPPORTED_ARGUMENT");
    }
}
