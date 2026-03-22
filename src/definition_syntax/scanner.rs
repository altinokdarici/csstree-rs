//! Scanner for CSS Value Definition Syntax strings.

use super::error::DefinitionSyntaxError;

type Result<T> = std::result::Result<T, DefinitionSyntaxError>;

/// Name character lookup table: `[a-zA-Z0-9-]` are valid.
fn is_name_char(code: u8) -> bool {
    code.is_ascii_alphanumeric() || code == b'-'
}

/// Scanner for definition syntax strings.
#[derive(Debug)]
pub struct Scanner {
    /// The input bytes.
    bytes: Vec<u8>,
    /// Current position in the input.
    pub pos: usize,
    /// Original source string (for error messages).
    source: String,
}

impl Scanner {
    /// Create a new scanner for the given source string.
    pub fn new(source: &str) -> Self {
        Self {
            bytes: source.as_bytes().to_vec(),
            pos: 0,
            source: source.to_string(),
        }
    }

    /// Returns the char code at the given position, or 0 if past end.
    pub fn char_code_at(&self, pos: usize) -> u8 {
        if pos < self.bytes.len() {
            self.bytes[pos]
        } else {
            0
        }
    }

    /// Returns the char code at the current position.
    pub fn char_code(&self) -> u8 {
        self.char_code_at(self.pos)
    }

    /// Returns true if the given code (or current) is a name character.
    pub fn is_name_char_code(&self, code: u8) -> bool {
        code < 128 && is_name_char(code)
    }

    /// Returns the char code at position + 1.
    pub fn next_char_code(&self) -> u8 {
        self.char_code_at(self.pos + 1)
    }

    /// Returns the char code at the next non-whitespace position from pos.
    #[allow(dead_code)]
    pub fn next_non_ws_code(&self, pos: usize) -> u8 {
        self.char_code_at(self.find_ws_end(pos))
    }

    /// Skip whitespace at the current position.
    pub fn skip_ws(&mut self) {
        self.pos = self.find_ws_end(self.pos);
    }

    /// Find the end of whitespace starting from pos.
    pub fn find_ws_end(&self, mut pos: usize) -> usize {
        while pos < self.bytes.len() {
            match self.bytes[pos] {
                b'\t' | b'\n' | 12 | b'\r' | b' ' => pos += 1,
                _ => break,
            }
        }
        pos
    }

    /// Return the substring from current pos to end, advancing pos to end.
    pub fn substring_to_pos(&mut self, end: usize) -> String {
        let start = self.pos;
        self.pos = end;
        String::from_utf8_lossy(&self.bytes[start..end]).to_string()
    }

    /// Expect the current char code to match, then advance. Error if not.
    pub fn eat(&mut self, code: u8) -> Result<()> {
        if self.char_code() != code {
            return Err(self.error(&format!("Expect `{}`", code as char)));
        }
        self.pos += 1;
        Ok(())
    }

    /// Read and return the current character, advancing position.
    pub fn peek_char(&mut self) -> Option<u8> {
        self.bytes.get(self.pos).map(|&ch| {
            self.pos += 1;
            ch
        })
    }

    /// Create a syntax error at the current position.
    pub fn error(&self, message: &str) -> DefinitionSyntaxError {
        DefinitionSyntaxError::new(message, &self.source, self.pos)
    }

    /// Scan and return whitespace from current position.
    #[allow(dead_code)]
    pub fn scan_spaces(&mut self) -> String {
        let end = self.find_ws_end(self.pos);
        self.substring_to_pos(end)
    }

    /// Scan a word (name characters: `[a-zA-Z0-9-]`).
    pub fn scan_word(&mut self) -> Result<String> {
        let mut end = self.pos;
        while end < self.bytes.len() {
            let code = self.bytes[end];
            if code >= 128 || !is_name_char(code) {
                break;
            }
            end += 1;
        }
        if self.pos == end {
            return Err(self.error("Expect a keyword"));
        }
        Ok(self.substring_to_pos(end))
    }

    /// Scan a decimal number.
    pub fn scan_number(&mut self) -> Result<String> {
        let mut end = self.pos;
        while end < self.bytes.len() {
            let code = self.bytes[end];
            if !code.is_ascii_digit() {
                break;
            }
            end += 1;
        }
        if self.pos == end {
            return Err(self.error("Expect a number"));
        }
        Ok(self.substring_to_pos(end))
    }

    /// Scan a single-quoted string.
    pub fn scan_string(&mut self) -> Result<String> {
        let start = self.pos + 1;
        if let Some(idx) = self.bytes[start..].iter().position(|&b| b == b'\'') {
            let end = start + idx + 1;
            Ok(self.substring_to_pos(end))
        } else {
            self.pos = self.bytes.len();
            Err(self.error("Expect an apostrophe"))
        }
    }

    /// Whether scanner has reached end of input.
    pub fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    /// Whether the input is empty.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Total length of the input.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_word_basic() {
        let mut s = Scanner::new("length");
        assert_eq!(s.scan_word().unwrap(), "length");
        assert!(s.is_eof());
    }

    #[test]
    fn scan_word_with_hyphens() {
        let mut s = Scanner::new("font-size");
        assert_eq!(s.scan_word().unwrap(), "font-size");
    }

    #[test]
    fn scan_number_basic() {
        let mut s = Scanner::new("42");
        assert_eq!(s.scan_number().unwrap(), "42");
    }

    #[test]
    fn scan_string_basic() {
        let mut s = Scanner::new("'hello'");
        assert_eq!(s.scan_string().unwrap(), "'hello'");
    }

    #[test]
    fn skip_ws() {
        let mut s = Scanner::new("  ab");
        s.skip_ws();
        assert_eq!(s.pos, 2);
    }

    #[test]
    fn eat_success() {
        let mut s = Scanner::new("<");
        assert!(s.eat(b'<').is_ok());
        assert_eq!(s.pos, 1);
    }

    #[test]
    fn eat_failure() {
        let mut s = Scanner::new("x");
        assert!(s.eat(b'<').is_err());
    }
}
