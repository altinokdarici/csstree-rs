//! Definition syntax error type.

use std::fmt;

/// Error from parsing CSS Value Definition Syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionSyntaxError {
    /// Human-readable error message.
    pub message: String,
    /// The input string that caused the error.
    pub input: String,
    /// Byte offset into the input where the error occurred.
    pub offset: usize,
}

impl DefinitionSyntaxError {
    /// Create a new definition syntax error.
    pub fn new(raw_message: &str, input: &str, offset: usize) -> Self {
        let pointer = " ".repeat(offset) + "^";
        let message = format!("{raw_message}\n  {input}\n  {pointer}");
        Self {
            message,
            input: input.to_string(),
            offset,
        }
    }
}

impl fmt::Display for DefinitionSyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for DefinitionSyntaxError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = DefinitionSyntaxError::new("Unexpected token", "<length> |", 9);
        assert!(err.message.contains("Unexpected token"));
        assert!(err.message.contains("<length> |"));
        assert!(err.message.contains("^"));
        assert_eq!(err.offset, 9);
    }

    #[test]
    fn error_at_start() {
        let err = DefinitionSyntaxError::new("Unexpected", "bad", 0);
        assert!(err.to_string().contains("^"));
    }
}
