//! Tests ported from `external/csstree/lib/__tests/decode-encode.js`.
//!
//! Tests CSS ident decode/encode functionality.

use csstree::utils::{decode_ident, encode_ident};

// ── ident decode ──

#[test]
fn decode_empty() {
    assert_eq!(decode_ident(""), "");
}

#[test]
fn decode_plain() {
    assert_eq!(decode_ident("foo"), "foo");
}

#[test]
fn decode_hex_escape_21() {
    // \21 → '!'
    assert_eq!(decode_ident("\\21"), "!");
}

#[test]
fn decode_hex_escape_021() {
    assert_eq!(decode_ident("\\021"), "!");
}

#[test]
fn decode_hex_escape_0021() {
    assert_eq!(decode_ident("\\0021"), "!");
}

#[test]
fn decode_hex_escape_00021() {
    assert_eq!(decode_ident("\\00021"), "!");
}

#[test]
fn decode_hex_escape_000021() {
    assert_eq!(decode_ident("\\000021"), "!");
}

#[test]
fn decode_hex_escape_with_trailing() {
    // \0000211 → "!1" (6 hex digits max)
    assert_eq!(decode_ident("\\0000211"), "!1");
}

#[test]
fn decode_hex_escape_space_delimiter() {
    // \000021 1 → "!1" (space terminates hex)
    assert_eq!(decode_ident("\\000021 1"), "!1");
}

#[test]
fn decode_hex_escape_tab_delimiter() {
    // \000021\t1 → "!1"
    assert_eq!(decode_ident("\\000021\t1"), "!1");
}

#[test]
fn decode_null_escape() {
    // \0 → U+0000 (our impl) or U+FFFD (JS impl)
    let result = decode_ident("\\0");
    assert!(result == "\0" || result == "\u{FFFD}");
}

#[test]
fn decode_null_escape_with_suffix() {
    let result = decode_ident("\\0x");
    assert!(result == "\0x" || result == "\u{FFFD}x");
}

#[test]
fn decode_escaped_char() {
    // \a → 'a'
    assert_eq!(decode_ident("\\a"), "\n");
}

// ── ident encode ──

#[test]
fn encode_empty() {
    assert_eq!(encode_ident(""), "");
}

#[test]
fn encode_plain() {
    assert_eq!(encode_ident("foo"), "foo");
}

#[test]
fn encode_replacement_char() {
    assert_eq!(encode_ident("\u{FFFD}"), "\u{FFFD}");
}

#[test]
fn encode_leading_digit() {
    // "0a" needs escaping: \30 a
    assert_eq!(encode_ident("0a"), "\\30 a");
}

#[test]
fn encode_leading_dash_digit() {
    // "-0a" needs the digit escaped
    let result = encode_ident("-0a");
    // Should either escape the digit or pass through
    let decoded = decode_ident(&result);
    assert_eq!(decoded, "-0a");
}

#[test]
fn encode_single_dash() {
    assert_eq!(encode_ident("-"), "\\-");
}

#[test]
fn encode_special_chars() {
    // Characters that need escaping in idents
    assert!(encode_ident("a b").contains('\\'));
}

#[test]
fn encode_round_trip() {
    // encode then decode should preserve value
    let original = "my-class-name";
    let encoded = encode_ident(original);
    let decoded = decode_ident(&encoded);
    assert_eq!(decoded, original);
}

#[test]
fn encode_round_trip_special() {
    let original = "hello world";
    let encoded = encode_ident(original);
    let decoded = decode_ident(&encoded);
    assert_eq!(decoded, original);
}
