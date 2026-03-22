//! Consume functions and string comparison helpers for the tokenizer.
//!
//! All consume functions take `(source, offset)` and return the new offset.
//! They operate on byte slices for zero-copy performance.

use super::char_code_definitions::{
    is_digit, is_hex_digit, is_name, is_uppercase_letter, is_valid_escape,
    is_whitespace,
};

/// Get a byte from source at offset, returning 0 (EOF) if out of bounds.
#[inline]
pub fn get_char_code(source: &[u8], offset: usize) -> u8 {
    if offset < source.len() {
        source[offset]
    } else {
        0
    }
}

/// Returns the length of a newline sequence at offset.
///
/// CR+LF counts as 2, everything else as 1.
#[inline]
pub fn get_newline_length(source: &[u8], offset: usize, code: u8) -> usize {
    if code == 0x0D && get_char_code(source, offset + 1) == 0x0A {
        2
    } else {
        1
    }
}

/// Case-insensitive single character comparison.
#[inline]
pub fn cmp_char(source: &[u8], offset: usize, reference_code: u8) -> bool {
    if offset >= source.len() {
        return false;
    }
    let mut code = source[offset];
    if is_uppercase_letter(code) {
        code |= 32; // to lowercase
    }
    code == reference_code
}

/// Case-insensitive string comparison of a source slice against a reference.
pub fn cmp_str(source: &[u8], start: usize, end: usize, reference_str: &[u8]) -> bool {
    if end - start != reference_str.len() {
        return false;
    }
    if end > source.len() {
        return false;
    }
    for i in 0..reference_str.len() {
        let reference_code = reference_str[i];
        let mut test_code = source[start + i];
        if is_uppercase_letter(test_code) {
            test_code |= 32;
        }
        if test_code != reference_code {
            return false;
        }
    }
    true
}

/// Scan backwards to find where whitespace starts.
pub fn find_whitespace_start(source: &[u8], mut offset: usize) -> usize {
    loop {
        if offset == 0 || !is_whitespace(source[offset - 1]) {
            break;
        }
        offset -= 1;
    }
    offset
}

/// Scan forward to skip past whitespace.
pub fn find_whitespace_end(source: &[u8], mut offset: usize) -> usize {
    while offset < source.len() && is_whitespace(source[offset]) {
        offset += 1;
    }
    offset
}

/// Scan forward past decimal digits.
pub fn find_decimal_number_end(source: &[u8], mut offset: usize) -> usize {
    while offset < source.len() && is_digit(source[offset]) {
        offset += 1;
    }
    offset
}

/// Consume an escaped code point (§4.3.7).
///
/// Assumes the `\` has already been consumed and the next code point
/// is verified to be part of a valid escape.
pub fn consume_escaped(source: &[u8], offset: usize) -> usize {
    // Skip past the backslash and the next char
    let mut pos = offset + 2;

    // If the char after backslash is a hex digit, consume up to 6 hex digits total
    if pos >= 2 && is_hex_digit(get_char_code(source, pos - 1)) {
        let max_offset = source.len().min(pos + 5);
        while pos < max_offset && is_hex_digit(get_char_code(source, pos)) {
            pos += 1;
        }
        // If the next code point is whitespace, consume it too
        let code = get_char_code(source, pos);
        if is_whitespace(code) {
            pos += get_newline_length(source, pos, code);
        }
    }

    pos
}

/// Consume a name (§4.3.11).
///
/// Does NOT verify that the stream starts with an identifier — the caller
/// must ensure that via `is_identifier_start`.
pub fn consume_name(source: &[u8], mut offset: usize) -> usize {
    while offset < source.len() {
        let code = source[offset];
        // Non-ASCII bytes are name code points (§4.2: non-ASCII → name code point).
        // Advance over the entire UTF-8 sequence.
        if code >= 0x80 {
            offset += utf8_byte_len(code);
            continue;
        }
        if is_name(code) {
            offset += 1;
            continue;
        }
        if is_valid_escape(code, get_char_code(source, offset + 1)) {
            offset = consume_escaped(source, offset);
            continue;
        }
        break;
    }
    offset
}

/// Determine the length of a UTF-8 sequence from its leading byte.
///
/// For ASCII (< 0x80) or invalid continuation bytes (0x80..0xBF), returns 1.
#[inline]
fn utf8_byte_len(leading: u8) -> usize {
    match leading {
        0x00..=0xBF => 1, // ASCII or continuation byte (advance 1 to recover)
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xFF => 4,
    }
}

/// Consume a number (§4.3.12).
pub fn consume_number(source: &[u8], mut offset: usize) -> usize {
    let mut code = get_char_code(source, offset);

    // Optional sign
    if code == 0x2B || code == 0x2D {
        offset += 1;
        code = get_char_code(source, offset);
    }

    // Integer part
    if is_digit(code) {
        offset = find_decimal_number_end(source, offset + 1);
        code = get_char_code(source, offset);
    }

    // Decimal part: . followed by digit
    if code == 0x2E && is_digit(get_char_code(source, offset + 1)) {
        offset = find_decimal_number_end(source, offset + 2);
    }

    // Exponent part: e/E [+-] digit
    if cmp_char(source, offset, b'e') {
        let mut sign_offset = 0;
        code = get_char_code(source, offset + 1);

        if code == 0x2D || code == 0x2B {
            sign_offset = 1;
            code = get_char_code(source, offset + 2);
        }

        if is_digit(code) {
            offset = find_decimal_number_end(source, offset + 1 + sign_offset + 1);
        }
    }

    offset
}

/// Consume the remnants of a bad URL (§4.3.14).
pub fn consume_bad_url_remnants(source: &[u8], mut offset: usize) -> usize {
    while offset < source.len() {
        let code = source[offset];
        if code == 0x29 {
            // )
            offset += 1;
            break;
        }
        if is_valid_escape(code, get_char_code(source, offset + 1)) {
            offset = consume_escaped(source, offset);
        } else {
            offset += 1;
        }
    }
    offset
}

/// Decode an escaped code point (§4.3.7).
///
/// Takes the escaped content (without leading `\`) and returns the decoded char.
pub fn decode_escaped(escaped: &str) -> char {
    let bytes = escaped.as_bytes();
    // Single non-hex-digit char
    if bytes.len() == 1 && !is_hex_digit(bytes[0]) {
        return bytes[0] as char;
    }

    // Parse hex
    let code = u32::from_str_radix(escaped.trim(), 16).unwrap_or(0);
    if code == 0 || (0xD800..=0xDFFF).contains(&code) || code > 0x10_FFFF {
        '\u{FFFD}'
    } else {
        char::from_u32(code).unwrap_or('\u{FFFD}')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_char_code_in_bounds() {
        assert_eq!(get_char_code(b"abc", 0), b'a');
        assert_eq!(get_char_code(b"abc", 2), b'c');
    }

    #[test]
    fn get_char_code_out_of_bounds() {
        assert_eq!(get_char_code(b"abc", 3), 0);
        assert_eq!(get_char_code(b"", 0), 0);
    }

    #[test]
    fn newline_length_crlf() {
        assert_eq!(get_newline_length(b"\r\n", 0, b'\r'), 2);
        assert_eq!(get_newline_length(b"\r", 0, b'\r'), 1);
        assert_eq!(get_newline_length(b"\n", 0, b'\n'), 1);
    }

    #[test]
    fn case_insensitive_cmp_char() {
        assert!(cmp_char(b"E", 0, b'e'));
        assert!(cmp_char(b"e", 0, b'e'));
        assert!(!cmp_char(b"x", 0, b'e'));
    }

    #[test]
    fn case_insensitive_cmp_str() {
        assert!(cmp_str(b"URL", 0, 3, b"url"));
        assert!(cmp_str(b"Url", 0, 3, b"url"));
        assert!(!cmp_str(b"UR", 0, 2, b"url"));
    }

    #[test]
    fn whitespace_scanning() {
        assert_eq!(find_whitespace_end(b"   abc", 0), 3);
        assert_eq!(find_whitespace_end(b"abc", 0), 0);
        assert_eq!(find_whitespace_start(b"abc   ", 6), 3);
    }

    #[test]
    fn decimal_number_end() {
        assert_eq!(find_decimal_number_end(b"123abc", 0), 3);
        assert_eq!(find_decimal_number_end(b"abc", 0), 0);
    }

    #[test]
    fn consume_name_basic() {
        assert_eq!(consume_name(b"foo-bar ", 0), 7);
        assert_eq!(consume_name(b"_test123 ", 0), 8);
    }

    #[test]
    fn consume_number_integer() {
        assert_eq!(consume_number(b"123px", 0), 3);
    }

    #[test]
    fn consume_number_decimal() {
        assert_eq!(consume_number(b"3.14 ", 0), 4);
    }

    #[test]
    fn consume_number_exponent() {
        assert_eq!(consume_number(b"1e10 ", 0), 4);
        assert_eq!(consume_number(b"1E10 ", 0), 4);
        assert_eq!(consume_number(b"1e+10 ", 0), 5);
        assert_eq!(consume_number(b"1e-10 ", 0), 5);
    }

    #[test]
    fn consume_number_signed() {
        assert_eq!(consume_number(b"+42 ", 0), 3);
        assert_eq!(consume_number(b"-3.14 ", 0), 5);
    }

    #[test]
    fn consume_escaped_hex() {
        // \41 → 'A', consumes backslash + 2 hex digits
        assert_eq!(consume_escaped(b"\\41 rest", 0), 4); // \41 + space
    }

    #[test]
    fn consume_escaped_nonhex() {
        // \n → just the two chars
        assert_eq!(consume_escaped(b"\\n rest", 0), 2);
    }

    #[test]
    fn consume_bad_url_remnants_basic() {
        assert_eq!(consume_bad_url_remnants(b"bad stuff) rest", 0), 10);
    }

    #[test]
    fn decode_escaped_hex() {
        assert_eq!(decode_escaped("41"), 'A');
        assert_eq!(decode_escaped("0"), '\u{FFFD}');
        assert_eq!(decode_escaped("D800"), '\u{FFFD}');
        assert_eq!(decode_escaped("110000"), '\u{FFFD}');
    }

    #[test]
    fn decode_escaped_single_char() {
        assert_eq!(decode_escaped("n"), 'n');
        assert_eq!(decode_escaped("@"), '@');
    }
}
