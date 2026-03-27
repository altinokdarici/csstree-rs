//! CSS `url()` value encoding and decoding.
//!
//! Handles CSS escape sequences in `url()` function values, including hex escapes,
//! line continuations, and special character escaping.
//!
//! Reference: `external/csstree/lib/utils/url.js`

use crate::tokenizer::char_code_definitions::{is_hex_digit, is_valid_escape, is_whitespace};
use crate::tokenizer::utils::{consume_escaped, decode_escaped};

/// Decode a CSS `url()` value, resolving escape sequences.
///
/// Input should be a CSS `url()` function (e.g., `url(foo)`, `url( bar )`).
/// Strips the `url(` prefix and `)` suffix, trims whitespace, and processes
/// escape sequences.
pub fn decode_url(value: &str) -> String {
    let bytes = value.as_bytes();
    let len = bytes.len();

    // Determine content boundaries: skip "url(" prefix and optional ")" suffix.
    // Use saturating_sub to avoid underflow when len is small.
    let mut start = 4; // length of "url("
    let content_end = if len > 0 && bytes[len - 1] == b')' {
        len.saturating_sub(2)
    } else {
        len.saturating_sub(1)
    };

    // If start is already past content_end, there's no content
    if start > content_end {
        return String::new();
    }

    let mut end = content_end;

    // Trim leading whitespace
    while start < end && is_whitespace(bytes[start]) {
        start += 1;
    }

    // Trim trailing whitespace
    while start < end && is_whitespace(bytes[end]) {
        end -= 1;
    }

    let mut decoded = String::with_capacity(len);
    let mut i = start;

    while i <= end {
        let code = bytes[i];

        if code == b'\\' {
            // Special case at the ending
            if i == end {
                // If the next input code point is EOF, do nothing.
                // Otherwise include last right parenthesis as escaped.
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
                // Line continuation: \r\n
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

/// Encode a string as a CSS `url()` value.
///
/// Wraps the value in `url(...)` and escapes special characters including
/// spaces, quotes, parentheses, and backslashes.
pub fn encode_url(value: &str) -> String {
    use std::fmt::Write;

    let mut encoded = String::with_capacity(value.len() + 5);
    let mut ws_before_hex_needed = false;

    for &byte in value.as_bytes() {
        // NULL -> U+FFFD
        if byte == 0x00 {
            encoded.push('\u{FFFD}');
            continue;
        }

        // Control chars U+0001..U+001F and U+007F -> hex escape
        if byte <= 0x1F || byte == 0x7F {
            encoded.push('\\');
            let _ = write!(encoded, "{byte:x}");
            ws_before_hex_needed = true;
            continue;
        }

        // Space, backslash, quotes, parens -> escaped character
        if byte == b' '
            || byte == b'\\'
            || byte == b'"'
            || byte == b'\''
            || byte == b'('
            || byte == b')'
        {
            encoded.push('\\');
            encoded.push(byte as char);
            ws_before_hex_needed = false;
        } else {
            if ws_before_hex_needed && is_hex_digit(byte) {
                encoded.push(' ');
            }
            encoded.push(byte as char);
            ws_before_hex_needed = false;
        }
    }

    format!("url({encoded})")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_empty_url() {
        assert_eq!(decode_url("url("), "");
    }

    #[test]
    fn encode_empty() {
        assert_eq!(encode_url(""), "url()");
    }

    #[test]
    fn round_trip_simple() {
        let original = "foo.jpg";
        let encoded = encode_url(original);
        assert_eq!(decode_url(&encoded), original);
    }
}
