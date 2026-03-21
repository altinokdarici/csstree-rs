//! CSS syntax error type with source location and formatted display.

use std::fmt;
use std::fmt::Write as _;

/// A CSS syntax error with source location information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssSyntaxError {
    /// Error message.
    pub message: String,
    /// Source CSS string (for error display).
    pub source: String,
    /// Byte offset of the error.
    pub offset: usize,
    /// 1-based line number.
    pub line: u32,
    /// 1-based column number.
    pub column: u32,
}

impl CssSyntaxError {
    /// Create a new syntax error.
    pub fn new(
        message: impl Into<String>,
        source: impl Into<String>,
        offset: usize,
        line: u32,
        column: u32,
    ) -> Self {
        Self {
            message: message.into(),
            source: source.into(),
            offset,
            line,
            column,
        }
    }

    /// Extract a source fragment around the error for display.
    pub fn source_fragment(&self, extra_lines: usize) -> String {
        let lines: Vec<&str> = self.source.split('\n').collect();
        let error_line_idx = (self.line as usize).saturating_sub(1);
        let start = error_line_idx.saturating_sub(extra_lines);
        let end = (error_line_idx + extra_lines + 1).min(lines.len());

        let mut result = String::new();
        let max_line_num = end;
        let num_width = max_line_num.to_string().len();

        for (i, line) in lines.iter().enumerate().take(end).skip(start) {
            let line_num = i + 1;
            let line_content = truncate_line(line);
            let _ = writeln!(result, "{line_num:>num_width$} |{line_content}");

            if i == error_line_idx {
                let pointer_col = (self.column as usize).saturating_sub(1);
                let padding = " ".repeat(num_width + 1 + pointer_col);
                let _ = writeln!(result, "{padding}^");
            }
        }

        result
    }

    /// Full formatted message with source context.
    pub fn formatted_message(&self) -> String {
        format!(
            "{}:{}: {}\n{}",
            self.line,
            self.column,
            self.message,
            self.source_fragment(2)
        )
    }
}

impl fmt::Display for CssSyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for CssSyntaxError {}

/// Truncate a line for display (max 100 chars).
fn truncate_line(line: &str) -> String {
    let s = line.replace('\t', "    ");
    if s.len() > 100 {
        format!("{}…", &s[..99])
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = CssSyntaxError::new("Unexpected token", "a { color: }", 11, 1, 12);
        assert_eq!(err.to_string(), "1:12: Unexpected token");
    }

    #[test]
    fn source_fragment() {
        let err = CssSyntaxError::new("test", "line1\nline2\nline3", 6, 2, 1);
        let frag = err.source_fragment(0);
        assert!(frag.contains("2 |line2"));
        assert!(frag.contains("^"));
    }

    #[test]
    fn formatted_message() {
        let err = CssSyntaxError::new("Unexpected", ".foo\n{\n  }", 8, 3, 3);
        let msg = err.formatted_message();
        assert!(msg.starts_with("3:3: Unexpected"));
        assert!(msg.contains("^"));
    }
}
