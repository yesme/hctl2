use std::fmt::{self, Display, Formatter};

/// Stable rejection plus recovery guidance for the public boundary to present unchanged.
#[derive(Debug)]
pub struct StoreError {
    pub code: &'static str,
    pub message: String,
    pub recovery_action: &'static str,
}

pub type Result<T> = std::result::Result<T, StoreError>;

impl StoreError {
    pub(crate) fn new(
        code: &'static str,
        message: impl Into<String>,
        recovery: &'static str,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            recovery_action: recovery,
        }
    }

    pub(crate) fn invalid(message: impl Into<String>) -> Self {
        Self::new("INVALID_INPUT", message, "correct_input")
    }
}

impl Display for StoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for StoreError {}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        Self::new("STORAGE_IO", e.to_string(), "inspect_storage")
    }
}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        Self::new("STORAGE_SQLITE", e.to_string(), "inspect_storage")
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        Self::new("INVALID_JSON", e.to_string(), "correct_input")
    }
}

impl From<foundation::FoundationError> for StoreError {
    fn from(e: foundation::FoundationError) -> Self {
        if e.is_lock_contended() {
            Self::new(
                "WRITER_BUSY",
                "another control writer owns the storage",
                "stop_previous_writer",
            )
        } else {
            Self::new("FOUNDATION", e.to_string(), "inspect_storage")
        }
    }
}
