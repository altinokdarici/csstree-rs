//! Comprehensive tests for CSS string and URL decode/encode functions.
//!
//! Test cases are ported from `external/csstree/lib/__tests/decode-encode.js`.

use csstree::utils::{decode_string, decode_url, encode_string, encode_url};

// ============================================================================
// String decode tests (35 cases from JS)
// ============================================================================

#[test]
fn string_decode_00_empty() {
    assert_eq!(decode_string(""), "");
}

#[test]
fn string_decode_01_double_quote_only() {
    assert_eq!(decode_string("\""), "");
}

#[test]
fn string_decode_02_single_quote_only() {
    assert_eq!(decode_string("'"), "");
}

#[test]
fn string_decode_03_empty_double_quoted() {
    assert_eq!(decode_string("\"\""), "");
}

#[test]
fn string_decode_04_empty_single_quoted() {
    assert_eq!(decode_string("''"), "");
}

#[test]
fn string_decode_05_double_containing_single() {
    // "'" → '
    assert_eq!(decode_string("\"'"), "'");
}

#[test]
fn string_decode_06_single_containing_double() {
    // '" → "
    assert_eq!(decode_string("'\""), "\"");
}

#[test]
fn string_decode_07_escaped_double_in_double() {
    // "\"" → "
    assert_eq!(decode_string("\"\\\"\""), "\"");
}

#[test]
fn string_decode_08_single_in_double() {
    // "'" → '
    assert_eq!(decode_string("\"'\""), "'");
}

#[test]
fn string_decode_09_escaped_double_at_end() {
    // "\" → "
    assert_eq!(decode_string("\"\\\""), "\"");
}

#[test]
fn string_decode_10_escaped_single_at_end() {
    // '\' → '
    assert_eq!(decode_string("'\\\'"), "'");
}

#[test]
fn string_decode_11_backslash_at_end_double() {
    // "\ → ""
    assert_eq!(decode_string("\"\\"), "");
}

#[test]
fn string_decode_12_backslash_at_end_single() {
    // '\ → ""
    assert_eq!(decode_string("'\\"), "");
}

#[test]
fn string_decode_13_line_continuation_lf() {
    // "a\<LF>b" → "ab"
    assert_eq!(decode_string("\"a\\\nb\""), "ab");
}

#[test]
fn string_decode_14_line_continuation_cr() {
    // "a\<CR>b" → "ab"
    assert_eq!(decode_string("\"a\\\rb\""), "ab");
}

#[test]
fn string_decode_15_line_continuation_ff() {
    // "a\<FF>b" → "ab"
    assert_eq!(decode_string("\"a\\\x0Cb\""), "ab");
}

#[test]
fn string_decode_16_hex_escape_2_digit() {
    // "\21" → "!"
    assert_eq!(decode_string("\"\\21\""), "!");
}

#[test]
fn string_decode_17_hex_escape_3_digit() {
    // "\021" → "!"
    assert_eq!(decode_string("\"\\021\""), "!");
}

#[test]
fn string_decode_18_hex_escape_4_digit() {
    // "\0021" → "!"
    assert_eq!(decode_string("\"\\0021\""), "!");
}

#[test]
fn string_decode_19_hex_escape_5_digit() {
    // "\00021" → "!"
    assert_eq!(decode_string("\"\\00021\""), "!");
}

#[test]
fn string_decode_20_hex_escape_6_digit() {
    // "\000021" → "!"
    assert_eq!(decode_string("\"\\000021\""), "!");
}

#[test]
fn string_decode_21_hex_escape_6_digit_overflow() {
    // "\0000211" → "!1" (6 hex digits max, then literal)
    assert_eq!(decode_string("\"\\0000211\""), "!1");
}

#[test]
fn string_decode_22_hex_escape_6_digit_space_sep() {
    // "\000021 1" → "!1" (space consumed as separator)
    assert_eq!(decode_string("\"\\000021 1\""), "!1");
}

#[test]
fn string_decode_23_hex_escape_6_digit_tab_sep() {
    // "\000021\t1" → "!1"
    assert_eq!(decode_string("\"\\000021\t1\""), "!1");
}

#[test]
fn string_decode_24_null_escape() {
    // "\0" → U+FFFD
    assert_eq!(decode_string("\"\\0\""), "\u{FFFD}");
}

#[test]
fn string_decode_25_null_escape_followed() {
    // "\0x" → U+FFFD + "x"
    assert_eq!(decode_string("\"\\0x\""), "\u{FFFD}x");
}

#[test]
fn string_decode_26_code_point_too_large() {
    // "\abcdefa" → U+FFFD + "a" (abcdef > max code point)
    assert_eq!(decode_string("\"\\abcdefa\""), "\u{FFFD}a");
}

#[test]
fn string_decode_27_surrogate() {
    // "\def0" → U+FFFD (surrogate range)
    assert_eq!(decode_string("\"\\def0\""), "\u{FFFD}");
}

#[test]
fn string_decode_28_valid_large_code_point() {
    // "\00abcdef" → U+ABCDEF (only first 6 hex digits used)
    assert_eq!(decode_string("\"\\00abcdef\""), "\u{abcd}ef");
}

#[test]
fn string_decode_29_max_hex_then_literal() {
    // "\abcdef1" → U+FFFD + "1"
    assert_eq!(decode_string("\"\\abcdef1\""), "\u{FFFD}1");
}

#[test]
fn string_decode_30_multiple_hex_escapes() {
    // "\a\d\c\9" → \n\r\f\t
    assert_eq!(decode_string("\"\\a\\d\\c\\9\""), "\n\r\x0C\t");
}

#[test]
fn string_decode_31_escaped_parens_and_backslash() {
    // "\(\)\\" → ()\
    assert_eq!(decode_string("\"\\(\\)\\\\\""), "()\\");
}

#[test]
fn string_decode_32_multiple_line_continuations() {
    // "\r\n\r\n" (line continuations)
    assert_eq!(decode_string("\"\\\r\\\n\\\r\n\""), "");
}

#[test]
fn string_decode_33_backslash_double_quote_at_end() {
    // "\" → "
    assert_eq!(decode_string("\"\\\""), "\"");
}

#[test]
fn string_decode_34_backslash_at_eof() {
    // "\ (no closing quote) → ""
    assert_eq!(decode_string("\"\\"), "");
}

#[test]
fn string_decode_35_unquoted_with_escape() {
    // \31  b → "1 b" (unquoted, hex escape + space separator + space + b)
    assert_eq!(decode_string("\\31  b"), "1 b");
}

// ============================================================================
// String encode tests (13 cases from JS)
// ============================================================================

#[test]
fn string_encode_00_empty() {
    assert_eq!(encode_string("", false), "\"\"");
}

#[test]
fn string_encode_01_double_quote() {
    assert_eq!(encode_string("\"", false), "\"\\\"\"");
}

#[test]
fn string_encode_02_single_quote() {
    assert_eq!(encode_string("'", false), "\"'\"");
}

#[test]
fn string_encode_03_newline_before_b() {
    // a + LF + b → "a\a b" (hex escape for LF, space separator before hex-like 'b')
    assert_eq!(encode_string("a\nb", false), "\"a\\a b\"");
}

#[test]
fn string_encode_04_newline_before_z() {
    // a + LF + z → "a\az" (no space needed, 'z' is not hex)
    assert_eq!(encode_string("a\nz", false), "\"a\\az\"");
}

#[test]
fn string_encode_05_cr_before_b() {
    assert_eq!(encode_string("a\rb", false), "\"a\\d b\"");
}

#[test]
fn string_encode_06_ff_before_b() {
    assert_eq!(encode_string("a\x0Cb", false), "\"a\\c b\"");
}

#[test]
fn string_encode_07_tab_before_b() {
    assert_eq!(encode_string("a\tb", false), "\"a\\9 b\"");
}

#[test]
fn string_encode_08_mixed_escapes() {
    // a\nbc\n"b\tx
    assert_eq!(
        encode_string("a\nbc\n\"b\tx", false),
        "\"a\\a bc\\a\\\"b\\9x\""
    );
}

#[test]
fn string_encode_09_backslash_before_hex() {
    // a\26b → "a\\26b" (backslash escaped, no space needed after)
    assert_eq!(encode_string("a\\26b", false), "\"a\\\\26b\"");
}

#[test]
fn string_encode_10_ampersand_b() {
    assert_eq!(encode_string("a&b", false), "\"a&b\"");
}

#[test]
fn string_encode_11_ampersand_z() {
    assert_eq!(encode_string("a&z", false), "\"a&z\"");
}

#[test]
fn string_encode_12_newline_space_b() {
    // \n + " " + b → "\a  b" (hex escape for LF, space separator, then space + b)
    assert_eq!(encode_string("\n b", false), "\"\\a  b\"");
}

// ============================================================================
// URL decode tests (23 cases from JS)
// ============================================================================

#[test]
fn url_decode_00_url_open_only() {
    assert_eq!(decode_url("url("), "");
}

#[test]
fn url_decode_01_no_closing_paren() {
    assert_eq!(decode_url("url(foo"), "foo");
}

#[test]
fn url_decode_02_simple() {
    assert_eq!(decode_url("url(foo)"), "foo");
}

#[test]
fn url_decode_03_whitespace_trimmed() {
    assert_eq!(decode_url("url(  foo  )"), "foo");
}

#[test]
fn url_decode_04_escaped_space_and_parens() {
    assert_eq!(decode_url("url(  1\\ \\(2\\).jpg  )"), "1 (2).jpg");
}

#[test]
fn url_decode_05_escaped_quotes_parens_space_backslash() {
    assert_eq!(
        decode_url("url(  \\\"\\'\\(\\)\\ \\\\ )"),
        "\"'() \\"
    );
}

#[test]
fn url_decode_06_line_continuations() {
    assert_eq!(decode_url("url(  a\\\r\\\n\\\r\nb  )"), "ab");
}

#[test]
fn url_decode_07_hex_2_digit() {
    assert_eq!(decode_url("url(\\21)"), "!");
}

#[test]
fn url_decode_08_hex_3_digit() {
    assert_eq!(decode_url("url(\\021)"), "!");
}

#[test]
fn url_decode_09_hex_4_digit() {
    assert_eq!(decode_url("url(\\0021)"), "!");
}

#[test]
fn url_decode_10_hex_5_digit() {
    assert_eq!(decode_url("url(\\00021)"), "!");
}

#[test]
fn url_decode_11_hex_6_digit() {
    assert_eq!(decode_url("url(\\000021)"), "!");
}

#[test]
fn url_decode_12_hex_6_digit_overflow() {
    assert_eq!(decode_url("url(\\0000211)"), "!1");
}

#[test]
fn url_decode_13_hex_6_digit_space_sep() {
    assert_eq!(decode_url("url(\\000021 1)"), "!1");
}

#[test]
fn url_decode_14_hex_6_digit_tab_sep() {
    assert_eq!(decode_url("url(\\000021\t1)"), "!1");
}

#[test]
fn url_decode_15_null_escape() {
    assert_eq!(decode_url("url(\\0)"), "\u{FFFD}");
}

#[test]
fn url_decode_16_null_escape_followed() {
    assert_eq!(decode_url("url(\\0x)"), "\u{FFFD}x");
}

#[test]
fn url_decode_17_code_point_too_large() {
    assert_eq!(decode_url("url(\\abcdefa)"), "\u{FFFD}a");
}

#[test]
fn url_decode_18_surrogate() {
    assert_eq!(decode_url("url(\\def0)"), "\u{FFFD}");
}

#[test]
fn url_decode_19_valid_large_code_point() {
    assert_eq!(decode_url("url(\\00abcdef)"), "\u{abcd}ef");
}

#[test]
fn url_decode_20_max_hex_then_literal() {
    assert_eq!(decode_url("url(\\abcdef1)"), "\u{FFFD}1");
}

#[test]
fn url_decode_21_escaped_closing_paren() {
    // url(\) → ")" (backslash at end position, next char is ')')
    assert_eq!(decode_url("url(\\)"), ")");
}

#[test]
fn url_decode_22_backslash_at_eof() {
    assert_eq!(decode_url("url(\\"), "");
}

// ============================================================================
// URL encode tests (15 cases from JS)
// ============================================================================

#[test]
fn url_encode_00_empty() {
    assert_eq!(encode_url(""), "url()");
}

#[test]
fn url_encode_01_double_quote() {
    assert_eq!(encode_url("\""), "url(\\\")");
}

#[test]
fn url_encode_02_single_quote() {
    assert_eq!(encode_url("'"), "url(\\')");
}

#[test]
fn url_encode_03_newline_before_b() {
    assert_eq!(encode_url("a\nb"), "url(a\\a b)");
}

#[test]
fn url_encode_04_newline_before_z() {
    assert_eq!(encode_url("a\nz"), "url(a\\az)");
}

#[test]
fn url_encode_05_cr_before_b() {
    assert_eq!(encode_url("a\rb"), "url(a\\d b)");
}

#[test]
fn url_encode_06_ff_before_b() {
    assert_eq!(encode_url("a\x0Cb"), "url(a\\c b)");
}

#[test]
fn url_encode_07_tab_before_b() {
    assert_eq!(encode_url("a\tb"), "url(a\\9 b)");
}

#[test]
fn url_encode_08_mixed_escapes() {
    assert_eq!(
        encode_url("a\nbc\n\"b\tx"),
        "url(a\\a bc\\a\\\"b\\9x)"
    );
}

#[test]
fn url_encode_09_backslash_before_hex() {
    assert_eq!(encode_url("a\\26b"), "url(a\\\\26b)");
}

#[test]
fn url_encode_10_space_and_parens() {
    assert_eq!(encode_url("1 (2).jpg"), "url(1\\ \\(2\\).jpg)");
}

#[test]
fn url_encode_11_all_special_chars() {
    assert_eq!(
        encode_url("\"'() \\"),
        "url(\\\"\\'\\(\\)\\ \\\\)"
    );
}

#[test]
fn url_encode_12_backspace() {
    assert_eq!(encode_url("\x08"), "url(\\8)");
}

#[test]
fn url_encode_13_space_in_middle() {
    assert_eq!(encode_url("1 b"), "url(1\\ b)");
}

#[test]
fn url_encode_14_newline_then_space() {
    assert_eq!(encode_url("1\n b"), "url(1\\a\\ b)");
}
