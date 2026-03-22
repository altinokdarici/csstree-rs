//! Token-before whitespace insertion logic for CSS serialization.
//!
//! Implements the CSS Syntax §9 requirement that serialization must round-trip
//! with parsing. Certain token pairs require an intervening whitespace to avoid
//! being re-parsed as a different token.

use std::collections::HashSet;

use crate::tokenizer::types::TokenType;
use super::GenerateMode;

/// Encode a token type and optional delimiter value into a token code.
///
/// The encoding packs the token type (shifted left by 1 bit) into the upper
/// bits, with the first char code (or delimiter code point) shifted left by 7.
/// The lowest bit is reserved for the "emit whitespace" flag.
///
/// Matches the JS `code()` function in `token-before.js`.
pub fn encode_token(token_type: TokenType, value: &str) -> u32 {
    let type_val = if token_type == TokenType::Delim {
        // For delimiters, use the char code instead of the token type
        let ch = value.as_bytes().first().copied().unwrap_or(0);
        u32::from(ch.min(0x80)) << 6
    } else {
        u32::from(token_type as u8) << 6
    };
    type_val << 1
}

/// Encode a delimiter character as a token code.
fn delim_code(ch: u8) -> u32 {
    (u32::from(ch.min(0x80)) << 6) << 1
}

/// Encode a token type as a token code.
fn type_code(tt: TokenType) -> u32 {
    (u32::from(tt as u8) << 6) << 1
}

/// Build the whitespace-required pair set for the given mode.
pub fn build_pairs(mode: GenerateMode) -> HashSet<u32> {
    let mut pairs = HashSet::new();

    // Add W3C spec pairs (always included)
    add_spec_pairs(&mut pairs);

    // Safe mode adds extra pairs for browser compatibility
    if mode == GenerateMode::Safe {
        add_safe_pairs(&mut pairs);
    }

    pairs
}

/// Check if whitespace is needed between `prev_code` and the next token.
///
/// Returns the new token code (with the emit-ws flag in bit 0 if needed).
#[allow(clippy::implicit_hasher)]
pub fn token_before(
    prev_code: u32,
    token_type: TokenType,
    value: &str,
    ws_pairs: &HashSet<u32>,
) -> u32 {
    let next_code = encode_token(token_type, value);
    let next_char = value.as_bytes().first().copied().unwrap_or(0);

    let emit_ws = if (next_char == b'-'
        && token_type != TokenType::Ident
        && token_type != TokenType::Function
        && token_type != TokenType::Cdc)
        || next_char == b'+'
    {
        // For - and + that aren't ident/function/CDC, check against the char code
        let char_code = u32::from(next_char) << 7;
        ws_pairs.contains(&((prev_code & 0xFFFE) << 16 | char_code))
    } else {
        ws_pairs.contains(&((prev_code & 0xFFFE) << 16 | next_code))
    };

    if emit_ws { next_code | 1 } else { next_code }
}

/// Combine a prev and next code into a pair key.
fn pair(prev: u32, next: u32) -> u32 {
    prev << 16 | next
}

/// Add the W3C CSS Syntax §9 spec pairs.
fn add_spec_pairs(pairs: &mut HashSet<u32>) {
    let ident = type_code(TokenType::Ident);
    let func = type_code(TokenType::Function);
    let url = type_code(TokenType::Url);
    let bad_url = type_code(TokenType::BadUrl);
    let at_keyword = type_code(TokenType::AtKeyword);
    let hash = type_code(TokenType::Hash);
    let dimension = type_code(TokenType::Dimension);
    let number = type_code(TokenType::Number);
    let percentage = type_code(TokenType::Percentage);
    let cdc = type_code(TokenType::Cdc);
    let left_paren = type_code(TokenType::LeftParenthesis);

    let dash = delim_code(b'-');
    let hash_delim = delim_code(b'#');
    let at_delim = delim_code(b'@');
    let dot = delim_code(b'.');
    let plus = delim_code(b'+');
    let slash = delim_code(b'/');
    let star = delim_code(b'*');
    let percent = delim_code(b'%');

    // Ident pairs
    for next in [ident, func, url, bad_url, dash, number, percentage, dimension, cdc, left_paren] {
        pairs.insert(pair(ident, next));
    }

    // AtKeyword pairs
    for next in [ident, func, url, bad_url, dash, number, percentage, dimension, cdc] {
        pairs.insert(pair(at_keyword, next));
    }

    // Hash pairs
    for next in [ident, func, url, bad_url, dash, number, percentage, dimension, cdc] {
        pairs.insert(pair(hash, next));
    }

    // Dimension pairs
    for next in [ident, func, url, bad_url, dash, number, percentage, dimension, cdc] {
        pairs.insert(pair(dimension, next));
    }

    // '#' delim pairs
    for next in [ident, func, url, bad_url, dash, number, percentage, dimension, cdc] {
        pairs.insert(pair(hash_delim, next));
    }

    // '-' delim pairs
    for next in [ident, func, url, bad_url, dash, number, percentage, dimension, cdc] {
        pairs.insert(pair(dash, next));
    }

    // Number pairs
    for next in [ident, func, url, bad_url, number, percentage, dimension, percent, cdc] {
        pairs.insert(pair(number, next));
    }

    // '@' delim pairs
    for next in [ident, func, url, bad_url, dash, cdc] {
        pairs.insert(pair(at_delim, next));
    }

    // '.' delim pairs
    for next in [number, percentage, dimension] {
        pairs.insert(pair(dot, next));
    }

    // '+' delim pairs
    for next in [number, percentage, dimension] {
        pairs.insert(pair(plus, next));
    }

    // '/' + '*'
    pairs.insert(pair(slash, star));
}

/// Add extra safe-mode pairs for browser compatibility.
fn add_safe_pairs(pairs: &mut HashSet<u32>) {
    let ident = type_code(TokenType::Ident);
    let func = type_code(TokenType::Function);
    let hash = type_code(TokenType::Hash);
    let at_keyword = type_code(TokenType::AtKeyword);
    let dimension = type_code(TokenType::Dimension);
    let percentage = type_code(TokenType::Percentage);
    let right_paren = type_code(TokenType::RightParenthesis);
    let string = type_code(TokenType::String);
    let colon = type_code(TokenType::Colon);

    let dash = delim_code(b'-');

    // Extra safe pairs
    pairs.insert(pair(ident, hash));
    pairs.insert(pair(dimension, hash));
    pairs.insert(pair(hash, hash));

    pairs.insert(pair(at_keyword, type_code(TokenType::LeftParenthesis)));
    pairs.insert(pair(at_keyword, string));
    pairs.insert(pair(at_keyword, colon));

    pairs.insert(pair(percentage, percentage));
    pairs.insert(pair(percentage, dimension));
    pairs.insert(pair(percentage, func));
    pairs.insert(pair(percentage, dash));

    pairs.insert(pair(right_paren, ident));
    pairs.insert(pair(right_paren, func));
    pairs.insert(pair(right_paren, percentage));
    pairs.insert(pair(right_paren, dimension));
    pairs.insert(pair(right_paren, hash));
    pairs.insert(pair(right_paren, dash));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_spec_pairs_not_empty() {
        let pairs = build_pairs(GenerateMode::Spec);
        assert!(!pairs.is_empty());
    }

    #[test]
    fn build_safe_pairs_larger_than_spec() {
        let spec = build_pairs(GenerateMode::Spec);
        let safe = build_pairs(GenerateMode::Safe);
        assert!(safe.len() > spec.len());
    }

    #[test]
    fn ident_before_ident_needs_space() {
        let pairs = build_pairs(GenerateMode::Safe);
        let prev = encode_token(TokenType::Ident, "foo");
        let result = token_before(prev, TokenType::Ident, "bar", &pairs);
        assert_eq!(result & 1, 1, "should need whitespace");
    }

    #[test]
    fn ident_before_colon_no_space() {
        let pairs = build_pairs(GenerateMode::Safe);
        let prev = encode_token(TokenType::Ident, "color");
        let result = token_before(prev, TokenType::Colon, ":", &pairs);
        assert_eq!(result & 1, 0, "should not need whitespace");
    }
}
