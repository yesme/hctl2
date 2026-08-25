use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// Stable machine-readable error code plus a human-readable explanation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErrorEnvelope {
    code: &'static str,
    message: String,
}

impl ErrorEnvelope {
    /// Creates an error envelope without assigning domain meaning to it.
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
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

impl Display for ErrorEnvelope {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl Error for ErrorEnvelope {}

#[cfg(test)]
mod tests {
    use super::ErrorEnvelope;

    #[test]
    fn preserves_code_and_message() {
        let error = ErrorEnvelope::new("HCTL2_TEST", "test failure");

        assert_eq!(error.code(), "HCTL2_TEST");
        assert_eq!(error.message(), "test failure");
        assert_eq!(error.to_string(), "HCTL2_TEST: test failure");
    }
}
