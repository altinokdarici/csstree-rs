//! CSS string literal encoding and decoding.
//!
//! Handles CSS escape sequences in string values, including hex escapes,
//! line continuations, and special character escaping.

use std::fmt::Write;

use crate::tokenizer::char_code_definitions::{is_hex_digit, is_valid_escape, is_whitespace};
use crate::tokenizer::utils::{consume_escaped, decode_escaped};

/// Decode a CSS string value, resolving escape sequences.
///
/// Input may be a quoted CSS string (e.g., `"hello"` or `'hello'`), or an
/// unquoted value. Strips opening/closing quotes and processes escape sequences.
pub fn decode_string(value: &str) -> String {
    let bytes = value.as_bytes();
    let len = bytes.len();
    if len == 0 {
        return String::new();
    }

    let first_char = bytes[0];
    let is_quoted = first_char == b'"' || first_char == b'\'';
    let start = usize::from(is_quoted);
    let end = if start == 1 && len > 1 && bytes[len - 1] == first_char {
        len - 2
    } else {
        len - 1
    };

    let mut decoded = String::with_capacity(len);
    let mut i = start;

    while i <= end {
        let code = bytes[i];

        if code == b'\\' {
            // Special case at the ending
            if i == end {
                // If the next input code point is EOF, do nothing.
                // Otherwise include last quote as escaped.
                if i != len - 1 {
                    decoded.push_str(&value[i + 1..]);
                }
                break;
            }

            i += 1;
            let next = bytes[i];

            // Consume escaped
            if is_valid_escape(b'\\', next) {
                let escape_start = i - 1;
                let escape_end = consume_escaped(bytes, escape_start);

                i = escape_end;
                decoded.push(decode_escaped(&value[escape_start + 1..escape_end]));
            } else {
                // Line continuation: \r\n, \r, \n, \f
                if next == 0x0D && i + 1 < bytes.len() && bytes[i + 1] == 0x0A {
                    i += 1;
                }
                i += 1;
            }
        } else {
            decoded.push(code as char);
            i += 1;
        }
    }

    decoded
}

/// Encode a string as a CSS string literal.
///
/// Wraps the value in double quotes (or single quotes if `use_apostrophe` is true)
/// and escapes special characters according to the CSSOM serialization spec.
///
/// <https://drafts.csswg.org/cssom/#serialize-a-string>
pub fn encode_string(value: &str, use_apostrophe: bool) -> String {
    let quote = if use_apostrophe { '\'' } else { '"' };
    let quote_code = if use_apostrophe { b'\'' } else { b'"' };

    let mut encoded = String::with_capacity(value.len() + 2);
    encoded.push(quote);

    let mut ws_before_hex_needed = false;

    for &byte in value.as_bytes() {
        // NULL → U+FFFD
        if byte == 0x00 {
            encoded.push('\u{FFFD}');
            continue;
        }

        // Control chars U+0001..U+001F and U+007F → hex escape
        if byte <= 0x1F || byte == 0x7F {
            encoded.push('\\');
            let _ = write!(encoded, "{byte:x}");
            ws_before_hex_needed = true;
            continue;
        }

        // Quote char or backslash → escaped character
        if byte == quote_code || byte == b'\\' {
            encoded.push('\\');
            encoded.push(byte as char);
            ws_before_hex_needed = false;
        } else {
            if ws_before_hex_needed && (is_hex_digit(byte) || is_whitespace(byte)) {
                encoded.push(' ');
            }
            encoded.push(byte as char);
            ws_before_hex_needed = false;
        }
    }

    encoded.push(quote);
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_empty() {
        assert_eq!(decode_string(""), "");
    }

    #[test]
    fn encode_empty() {
        assert_eq!(encode_string("", false), "\"\"");
    }

    #[test]
    fn round_trip_simple() {
        let original = "hello world";
        let encoded = encode_string(original, false);
        assert_eq!(decode_string(&encoded), original);
    }
}
