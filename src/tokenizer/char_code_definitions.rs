//! Character classification functions per CSS Syntax Module Level 3 §4.2.
//!
//! Provides fast ASCII category lookup via a 128-byte table, plus individual
//! predicate functions matching the W3C spec definitions exactly.

/// Fast-dispatch categories for ASCII code points (0..128).
///
/// Non-ASCII (>= 0x80) are always `NAME_START_CATEGORY`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CharCategory {
    /// The code point itself (for single-char tokens like `:`, `;`, `{`).
    Itself = 0,
    /// EOF sentinel.
    Eof = 0x80,
    /// Whitespace (space, tab, newline, CR, FF).
    WhiteSpace = 0x82,
    /// Digit 0-9.
    Digit = 0x83,
    /// Name-start code point (letter, `_`, or non-ASCII).
    NameStart = 0x84,
    /// Non-printable code point.
    NonPrintable = 0x85,
}

/// Precomputed category table for ASCII code points 0..128.
///
/// Built at compile time to match the JS `CATEGORY` array.
static CATEGORY: [u8; 128] = {
    let mut table = [0u8; 128];
    let mut i = 0u8;
    loop {
        if i >= 128 {
            break;
        }
        table[i as usize] = if is_whitespace_const(i) {
            CharCategory::WhiteSpace as u8
        } else if is_digit_const(i) {
            CharCategory::Digit as u8
        } else if is_name_start_const(i) {
            CharCategory::NameStart as u8
        } else if is_non_printable_const(i) {
            CharCategory::NonPrintable as u8
        } else if i == 0 {
            CharCategory::Eof as u8
        } else {
            i // the code point itself
        };
        i += 1;
    }
    table
};

// ── Const helper functions for building the lookup table ──

const fn is_digit_const(code: u8) -> bool {
    code >= 0x30 && code <= 0x39
}

const fn is_uppercase_letter_const(code: u8) -> bool {
    code >= 0x41 && code <= 0x5A
}

const fn is_lowercase_letter_const(code: u8) -> bool {
    code >= 0x61 && code <= 0x7A
}

const fn is_letter_const(code: u8) -> bool {
    is_uppercase_letter_const(code) || is_lowercase_letter_const(code)
}

const fn is_name_start_const(code: u8) -> bool {
    is_letter_const(code) || code == 0x5F // letter or _
}

const fn is_newline_const(code: u8) -> bool {
    code == 0x0A || code == 0x0D || code == 0x0C
}

const fn is_whitespace_const(code: u8) -> bool {
    is_newline_const(code) || code == 0x20 || code == 0x09
}

const fn is_non_printable_const(code: u8) -> bool {
    // Note: 0x00 is excluded — it maps to EofCategory in the table,
    // matching JS where `i || EofCategory` makes 0 → Eof, not NonPrintable.
    (code >= 0x01 && code <= 0x08) || (code == 0x0B) || (code >= 0x0E && code <= 0x1F) || (code == 0x7F)
}

// ── Public predicate functions ──

/// A digit (U+0030..U+0039).
#[inline]
pub fn is_digit(code: u8) -> bool {
    (0x30..=0x39).contains(&code)
}

/// A hex digit (0-9, A-F, a-f).
#[inline]
pub fn is_hex_digit(code: u8) -> bool {
    is_digit(code) || (0x41..=0x46).contains(&code) || (0x61..=0x66).contains(&code)
}

/// An uppercase letter (A-Z).
#[inline]
pub fn is_uppercase_letter(code: u8) -> bool {
    (0x41..=0x5A).contains(&code)
}

/// A lowercase letter (a-z).
#[inline]
pub fn is_lowercase_letter(code: u8) -> bool {
    (0x61..=0x7A).contains(&code)
}

/// A letter (a-z or A-Z).
#[inline]
pub fn is_letter(code: u8) -> bool {
    is_uppercase_letter(code) || is_lowercase_letter(code)
}

/// A non-ASCII code point (>= U+0080).
#[inline]
pub fn is_non_ascii(code: u32) -> bool {
    code >= 0x0080
}

/// A name-start code point: letter, non-ASCII, or `_`.
#[inline]
pub fn is_name_start(code: u8) -> bool {
    is_letter(code) || code == 0x5F // _ underscore
}

/// A name-start code point including non-ASCII.
#[inline]
pub fn is_name_start_u32(code: u32) -> bool {
    if code < 0x80 {
        #[expect(clippy::cast_possible_truncation, reason = "guarded by code < 0x80")]
        is_name_start(code as u8)
    } else {
        true // non-ASCII
    }
}

/// A name code point: name-start, digit, or `-`.
#[inline]
pub fn is_name(code: u8) -> bool {
    is_name_start(code) || is_digit(code) || code == 0x2D // -
}

/// A name code point including non-ASCII.
#[inline]
pub fn is_name_u32(code: u32) -> bool {
    if code < 0x80 {
        #[expect(clippy::cast_possible_truncation, reason = "guarded by code < 0x80")]
        is_name(code as u8)
    } else {
        true // non-ASCII
    }
}

/// A non-printable code point.
#[inline]
pub fn is_non_printable(code: u8) -> bool {
    code <= 0x08 || code == 0x0B || (0x0E..=0x1F).contains(&code) || code == 0x7F
}

/// A newline (LF, CR, or FF).
#[inline]
pub fn is_newline(code: u8) -> bool {
    code == 0x0A || code == 0x0D || code == 0x0C
}

/// A whitespace character (newline, tab, or space).
#[inline]
pub fn is_whitespace(code: u8) -> bool {
    is_newline(code) || code == 0x20 || code == 0x09
}

/// Check if two code points form a valid escape (§4.3.8).
#[inline]
pub fn is_valid_escape(first: u8, second: u8) -> bool {
    first == 0x5C && !is_newline(second) && second != 0
}

/// Check if three code points would start an identifier (§4.3.9).
pub fn is_identifier_start(first: u8, second: u8, third: u8) -> bool {
    match first {
        // U+002D HYPHEN-MINUS
        0x2D => is_name_start(second) || second == 0x2D || is_valid_escape(second, third),
        // U+005C REVERSE SOLIDUS
        0x5C => is_valid_escape(first, second),
        // name-start code point
        _ => is_name_start(first),
    }
}

/// Check if three code points would start a number (§4.3.10).
///
/// Returns number of consumed chars (0 = not a number start, 1-3 = number start).
pub fn is_number_start(first: u8, second: u8, third: u8) -> u8 {
    match first {
        // + or -
        0x2B | 0x2D => {
            if is_digit(second) {
                2
            } else if second == 0x2E && is_digit(third) {
                3
            } else {
                0
            }
        }
        // .
        0x2E => {
            if is_digit(second) { 2 } else { 0 }
        }
        // digit
        _ if is_digit(first) => 1,
        _ => 0,
    }
}

/// Detect BOM at the start of source.
///
/// Returns 1 if BOM found, 0 otherwise.
pub fn is_bom(code: u32) -> usize {
    usize::from(code == 0xFEFF || code == 0xFFFE)
}

/// Fast category lookup for a code point.
///
/// For ASCII (< 0x80), returns the precomputed category.
/// For non-ASCII (>= 0x80), returns `NameStart`.
#[inline]
pub fn char_code_category(code: u32) -> u8 {
    if code < 0x80 {
        CATEGORY[code as usize]
    } else {
        CharCategory::NameStart as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digit_range() {
        assert!(is_digit(b'0'));
        assert!(is_digit(b'9'));
        assert!(!is_digit(b'a'));
        assert!(!is_digit(b'/'));
    }

    #[test]
    fn hex_digit_range() {
        assert!(is_hex_digit(b'0'));
        assert!(is_hex_digit(b'9'));
        assert!(is_hex_digit(b'A'));
        assert!(is_hex_digit(b'F'));
        assert!(is_hex_digit(b'a'));
        assert!(is_hex_digit(b'f'));
        assert!(!is_hex_digit(b'G'));
        assert!(!is_hex_digit(b'g'));
    }

    #[test]
    fn name_start_chars() {
        assert!(is_name_start(b'a'));
        assert!(is_name_start(b'Z'));
        assert!(is_name_start(b'_'));
        assert!(!is_name_start(b'0'));
        assert!(!is_name_start(b'-'));
    }

    #[test]
    fn name_chars() {
        assert!(is_name(b'a'));
        assert!(is_name(b'0'));
        assert!(is_name(b'-'));
        assert!(is_name(b'_'));
        assert!(!is_name(b' '));
        assert!(!is_name(b'('));
    }

    #[test]
    fn whitespace_chars() {
        assert!(is_whitespace(b' '));
        assert!(is_whitespace(b'\t'));
        assert!(is_whitespace(b'\n'));
        assert!(is_whitespace(b'\r'));
        assert!(is_whitespace(0x0C)); // form feed
        assert!(!is_whitespace(b'a'));
    }

    #[test]
    fn valid_escape() {
        assert!(is_valid_escape(b'\\', b'a'));
        assert!(is_valid_escape(b'\\', b'0'));
        assert!(!is_valid_escape(b'\\', b'\n'));
        assert!(!is_valid_escape(b'\\', 0)); // EOF
        assert!(!is_valid_escape(b'a', b'b'));
    }

    #[test]
    fn identifier_start_detection() {
        assert!(is_identifier_start(b'a', 0, 0));
        assert!(is_identifier_start(b'-', b'a', 0));
        assert!(is_identifier_start(b'-', b'-', 0));
        assert!(is_identifier_start(b'\\', b'a', 0));
        assert!(!is_identifier_start(b'0', 0, 0));
        assert!(!is_identifier_start(b'-', b'0', 0));
    }

    #[test]
    fn number_start_detection() {
        assert_eq!(is_number_start(b'1', 0, 0), 1);
        assert_eq!(is_number_start(b'+', b'1', 0), 2);
        assert_eq!(is_number_start(b'-', b'1', 0), 2);
        assert_eq!(is_number_start(b'.', b'1', 0), 2);
        assert_eq!(is_number_start(b'+', b'.', b'1'), 3);
        assert_eq!(is_number_start(b'a', 0, 0), 0);
        assert_eq!(is_number_start(b'+', b'a', 0), 0);
    }

    #[test]
    fn category_table_matches_js() {
        // whitespace
        assert_eq!(char_code_category(0x20), CharCategory::WhiteSpace as u8);
        assert_eq!(char_code_category(0x09), CharCategory::WhiteSpace as u8);
        assert_eq!(char_code_category(0x0A), CharCategory::WhiteSpace as u8);

        // digits
        assert_eq!(char_code_category(u32::from(b'0')), CharCategory::Digit as u8);
        assert_eq!(char_code_category(u32::from(b'9')), CharCategory::Digit as u8);

        // name-start
        assert_eq!(char_code_category(u32::from(b'a')), CharCategory::NameStart as u8);
        assert_eq!(char_code_category(u32::from(b'_')), CharCategory::NameStart as u8);

        // non-printable
        assert_eq!(char_code_category(0x01), CharCategory::NonPrintable as u8);
        assert_eq!(char_code_category(0x7F), CharCategory::NonPrintable as u8);

        // self-representing
        assert_eq!(char_code_category(u32::from(b'#')), u32::from(b'#') as u8);
        assert_eq!(char_code_category(u32::from(b'(')), u32::from(b'(') as u8);

        // EOF
        assert_eq!(char_code_category(0), CharCategory::Eof as u8);

        // non-ASCII
        assert_eq!(char_code_category(0x80), CharCategory::NameStart as u8);
        assert_eq!(char_code_category(0xFFFF), CharCategory::NameStart as u8);
    }

    #[test]
    fn bom_detection() {
        assert_eq!(is_bom(0xFEFF), 1);
        assert_eq!(is_bom(0xFFFE), 1);
        assert_eq!(is_bom(0x0041), 0);
    }
}
