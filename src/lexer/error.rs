//! Lexer error types.

use std::fmt;

/// Error when referencing an unknown type, property, or at-rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxReferenceError {
    /// Error description.
    pub message: String,
    /// The name that was not found.
    pub reference: String,
}

impl fmt::Display for SyntaxReferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SyntaxReferenceError {}

/// Error when a CSS value does not match its definition syntax.
#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxMatchError {
    /// Brief error description.
    pub raw_message: String,
    /// The expected syntax as a string.
    pub syntax: String,
    /// The CSS value that failed to match.
    pub css: String,
    /// Byte offset of the mismatch in the CSS string.
    pub mismatch_offset: usize,
    /// Length of the mismatched portion.
    pub mismatch_length: usize,
    /// Formatted error message with location indicator.
    pub message: String,
}

impl SyntaxMatchError {
    /// Create a new syntax match error.
    pub fn new(raw_message: &str, syntax: &str, css: &str, offset: usize, length: usize) -> Self {
        let pointer = " ".repeat(offset) + &"^".repeat(length.max(1));
        let message = format!(
            "{raw_message}\n  syntax: {syntax}\n    value: {css}\n           {pointer}"
        );
        Self {
            raw_message: raw_message.to_string(),
            syntax: syntax.to_string(),
            css: css.to_string(),
            mismatch_offset: offset,
            mismatch_length: length,
            message,
        }
    }
}

impl fmt::Display for SyntaxMatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SyntaxMatchError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_reference_error_display() {
        let err = SyntaxReferenceError {
            message: "Unknown property `foo`".into(),
            reference: "foo".into(),
        };
        assert!(err.to_string().contains("Unknown property"));
    }

    #[test]
    fn syntax_match_error_display() {
        let err = SyntaxMatchError::new(
            "Mismatch",
            "<length> | auto",
            "red",
            0,
            3,
        );
        assert!(err.to_string().contains("Mismatch"));
        assert!(err.to_string().contains("<length> | auto"));
        assert!(err.to_string().contains("^^^"));
    }
}
