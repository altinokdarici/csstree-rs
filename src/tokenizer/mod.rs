//! CSS tokenizer implementing W3C CSS Syntax Module Level 3.
//!
//! Reference: `external/csstree/lib/tokenizer/`
//!
//! ## Architecture (from JS source analysis)
//!
//! The tokenizer has 6 submodules:
//!
//! - **types** — 26 token types (EOF + 25 CSS tokens) as integer constants 0..25.
//! - **`char_code_definitions`** — Character classification functions (`is_digit`, `is_name_start`,
//!   `is_identifier_start`, `is_number_start`, `char_code_category`) plus fast-lookup category
//!   table for ASCII (0..127). Non-ASCII chars are all `NameStartCategory`.
//! - **names** — Token type to CSS spec name mapping (e.g. `1 => "ident-token"`).
//! - **utils** — Consume functions that advance an offset through the source string:
//!   `consume_escaped` (hex escape + optional trailing whitespace), `consume_name` (ident chars),
//!   `consume_number` (sign, digits, dot, exponent), `consume_bad_url_remnants`, plus helpers
//!   `cmp_char`, `cmp_str`, `find_whitespace_start/end`, `find_decimal_number_end`,
//!   `decode_escaped`.
//! - **`token_stream`** — `TokenStream`: tokenizes source into a packed `Uint32Array` where each
//!   entry stores `(type << 24) | end_offset`. Also builds a `balance` array that maps block
//!   openers to their closers (and vice versa). Provides indexed access: `next()`, `skip()`,
//!   `lookup_type()`, `lookup_value()`, `skip_until_balanced()`, `for_each_token()`, etc.
//!   The parser calls `TokenStream` methods to navigate the token sequence.
//! - **`offset_to_location`** — `OffsetToLocation`: lazily computes line/column arrays from source,
//!   then maps byte offsets to `{source, offset, line, column}` locations. Used for AST position
//!   tracking.
//!
//! ## Tokenize algorithm (§4.3.1)
//!
//! The main `tokenize(source, on_token)` function is a single `while` loop over the source.
//! It uses `char_code_category(code)` for fast dispatch (switch on category), then calls nested
//! consume functions (`consume_numeric_token`, `consume_ident_like_token`, `consume_string_token`,
//! `consume_url_token`) which mutate an `offset` and `type` variable. After each token,
//! `on_token(type, start, end)` is called.
//!
//! Key behaviors:
//! - BOM at position 0 is skipped.
//! - `url(` followed by whitespace+quote → `Function` token (quoted URL), otherwise → `Url` token.
//! - Comments `/* ... */` are emitted as `Comment` tokens (not skipped).
//! - Numbers: sign → digits → `.` digits → `e`/`E` [±] digits → check for Dimension/Percentage.
//! - Strings: newline inside string → `BadString`. Escape handling inside strings.
//! - `<!--` → CDO, `-->` → CDC.
//!
//! ## Rust design notes
//!
//! - Token types → `#[repr(u8)]` enum for compact storage.
//! - `tokenize()` operates on `&str`, uses `source.as_bytes()` for byte-level access.
//! - `TokenStream` stores tokens in a `Vec<u32>` (packed type+offset) + `Vec<u32>` (balance).
//! - `OffsetToLocation` lazily computes `Vec<u32>` for lines and columns.
//! - All consume functions take `(source: &[u8], offset: usize)` and return new offset.
//! - Zero-copy: no String allocations during tokenization.

pub mod types;
pub mod char_code_definitions;
pub mod names;
pub mod utils;
pub mod token_stream;
pub mod offset_to_location;

use char_code_definitions::{
    char_code_category, is_bom, is_identifier_start, is_name_u32, is_newline, is_number_start,
    is_valid_escape, CharCategory,
};
use types::TokenType;
use utils::{
    cmp_str, consume_bad_url_remnants, consume_escaped, consume_name, consume_number,
    find_whitespace_end, get_char_code, get_newline_length,
};

/// Tokenize a CSS source string, calling `on_token` for each token found.
///
/// The callback receives `(token_type, start_offset, end_offset)` for each token.
/// This is a direct port of `tokenize()` from `external/csstree/lib/tokenizer/index.js`.
#[expect(clippy::too_many_lines, reason = "single dispatch loop matching JS structure — splitting would obscure the W3C algorithm")]
pub fn tokenize(source: &str, mut on_token: impl FnMut(TokenType, usize, usize)) {
    let bytes = source.as_bytes();
    let source_length = bytes.len();

    // Skip BOM if present
    let first_char = if source_length > 0 {
        u32::from(bytes[0])
    } else {
        0
    };
    let bom_offset = is_bom(first_char);
    let mut start = bom_offset;
    let mut offset = start;

    // §4.3.1 Consume a token
    while offset < source_length {
        let code = bytes[offset];
        let category = char_code_category(u32::from(code));
        let token_type;

        match category {
            // Whitespace
            cat if cat == CharCategory::WhiteSpace as u8 => {
                token_type = TokenType::WhiteSpace;
                offset = find_whitespace_end(bytes, offset + 1);
            }

            // U+0022 QUOTATION MARK (") or U+0027 APOSTROPHE (')
            0x22 | 0x27 => {
                token_type = consume_string_token(bytes, &mut offset, 0);
            }

            // U+0023 NUMBER SIGN (#)
            0x23 => {
                if is_name_u32(u32::from(get_char_code(bytes, offset + 1)))
                    || is_valid_escape(
                        get_char_code(bytes, offset + 1),
                        get_char_code(bytes, offset + 2),
                    )
                {
                    token_type = TokenType::Hash;
                    offset = consume_name(bytes, offset + 1);
                } else {
                    token_type = TokenType::Delim;
                    offset += 1;
                }
            }

            // U+0028 LEFT PARENTHESIS (()
            0x28 => {
                token_type = TokenType::LeftParenthesis;
                offset += 1;
            }

            // U+0029 RIGHT PARENTHESIS ())
            0x29 => {
                token_type = TokenType::RightParenthesis;
                offset += 1;
            }

            // U+002B PLUS SIGN (+) or U+002E FULL STOP (.)
            0x2B | 0x2E => {
                if is_number_start(
                    code,
                    get_char_code(bytes, offset + 1),
                    get_char_code(bytes, offset + 2),
                ) != 0
                {
                    token_type = consume_numeric_token(bytes, &mut offset);
                } else {
                    token_type = TokenType::Delim;
                    offset += 1;
                }
            }

            // U+002C COMMA (,)
            0x2C => {
                token_type = TokenType::Comma;
                offset += 1;
            }

            // U+002D HYPHEN-MINUS (-)
            0x2D => {
                if is_number_start(
                    code,
                    get_char_code(bytes, offset + 1),
                    get_char_code(bytes, offset + 2),
                ) != 0
                {
                    token_type = consume_numeric_token(bytes, &mut offset);
                } else if get_char_code(bytes, offset + 1) == 0x2D
                    && get_char_code(bytes, offset + 2) == 0x3E
                {
                    // -->
                    token_type = TokenType::Cdc;
                    offset += 3;
                } else if is_identifier_start(
                    code,
                    get_char_code(bytes, offset + 1),
                    get_char_code(bytes, offset + 2),
                ) {
                    token_type = consume_ident_like_token(bytes, &mut offset);
                } else {
                    token_type = TokenType::Delim;
                    offset += 1;
                }
            }

            // U+002F SOLIDUS (/)
            0x2F => {
                if get_char_code(bytes, offset + 1) == 0x2A {
                    // /* comment */
                    token_type = TokenType::Comment;
                    // Find closing */
                    let close = find_comment_end(bytes, offset + 2);
                    offset = close;
                } else {
                    token_type = TokenType::Delim;
                    offset += 1;
                }
            }

            // U+003A COLON (:)
            0x3A => {
                token_type = TokenType::Colon;
                offset += 1;
            }

            // U+003B SEMICOLON (;)
            0x3B => {
                token_type = TokenType::Semicolon;
                offset += 1;
            }

            // U+003C LESS-THAN SIGN (<)
            0x3C => {
                if get_char_code(bytes, offset + 1) == 0x21
                    && get_char_code(bytes, offset + 2) == 0x2D
                    && get_char_code(bytes, offset + 3) == 0x2D
                {
                    // <!--
                    token_type = TokenType::Cdo;
                    offset += 4;
                } else {
                    token_type = TokenType::Delim;
                    offset += 1;
                }
            }

            // U+0040 COMMERCIAL AT (@)
            0x40 => {
                if is_identifier_start(
                    get_char_code(bytes, offset + 1),
                    get_char_code(bytes, offset + 2),
                    get_char_code(bytes, offset + 3),
                ) {
                    token_type = TokenType::AtKeyword;
                    offset = consume_name(bytes, offset + 1);
                } else {
                    token_type = TokenType::Delim;
                    offset += 1;
                }
            }

            // U+005B LEFT SQUARE BRACKET ([)
            0x5B => {
                token_type = TokenType::LeftSquareBracket;
                offset += 1;
            }

            // U+005C REVERSE SOLIDUS (\)
            0x5C => {
                if is_valid_escape(code, get_char_code(bytes, offset + 1)) {
                    token_type = consume_ident_like_token(bytes, &mut offset);
                } else {
                    token_type = TokenType::Delim;
                    offset += 1;
                }
            }

            // U+005D RIGHT SQUARE BRACKET (])
            0x5D => {
                token_type = TokenType::RightSquareBracket;
                offset += 1;
            }

            // U+007B LEFT CURLY BRACKET ({)
            0x7B => {
                token_type = TokenType::LeftCurlyBracket;
                offset += 1;
            }

            // U+007D RIGHT CURLY BRACKET (})
            0x7D => {
                token_type = TokenType::RightCurlyBracket;
                offset += 1;
            }

            // digit
            cat if cat == CharCategory::Digit as u8 => {
                token_type = consume_numeric_token(bytes, &mut offset);
            }

            // name-start code point
            cat if cat == CharCategory::NameStart as u8 => {
                token_type = consume_ident_like_token(bytes, &mut offset);
            }

            // anything else → delim
            _ => {
                token_type = TokenType::Delim;
                offset += 1;
            }
        }

        on_token(token_type, start, offset);
        start = offset;
    }
}

// ── Internal consume functions ──

/// Consume a numeric token (§4.3.3).
fn consume_numeric_token(source: &[u8], offset: &mut usize) -> TokenType {
    *offset = consume_number(source, *offset);

    // If next 3 code points would start an identifier → Dimension
    if is_identifier_start(
        get_char_code(source, *offset),
        get_char_code(source, *offset + 1),
        get_char_code(source, *offset + 2),
    ) {
        *offset = consume_name(source, *offset);
        return TokenType::Dimension;
    }

    // If next is % → Percentage
    if get_char_code(source, *offset) == 0x25 {
        *offset += 1;
        return TokenType::Percentage;
    }

    TokenType::Number
}

/// Consume an ident-like token (§4.3.4).
fn consume_ident_like_token(source: &[u8], offset: &mut usize) -> TokenType {
    let name_start_offset = *offset;
    *offset = consume_name(source, *offset);

    // Check for url(
    if cmp_str(source, name_start_offset, *offset, b"url")
        && get_char_code(source, *offset) == 0x28
    {
        *offset = find_whitespace_end(source, *offset + 1);

        // If next is quote → Function token (quoted URL)
        let next = get_char_code(source, *offset);
        if next == 0x22 || next == 0x27 {
            *offset = name_start_offset + 4; // "url("
            return TokenType::Function;
        }

        // Otherwise → consume unquoted URL
        return consume_url_token(source, offset);
    }

    // If next is ( → Function
    if get_char_code(source, *offset) == 0x28 {
        *offset += 1;
        return TokenType::Function;
    }

    TokenType::Ident
}

/// Consume a string token (§4.3.5).
fn consume_string_token(source: &[u8], offset: &mut usize, ending_code_point: u8) -> TokenType {
    let ending = if ending_code_point == 0 {
        let ch = source[*offset];
        *offset += 1;
        ch
    } else {
        ending_code_point
    };

    while *offset < source.len() {
        let code = source[*offset];
        let category = char_code_category(u32::from(code));

        // Ending code point
        if category == ending {
            *offset += 1;
            return TokenType::String;
        }

        match category {
            // Whitespace — check for newline (→ BadString)
            cat if cat == CharCategory::WhiteSpace as u8 => {
                if is_newline(code) {
                    *offset += get_newline_length(source, *offset, code);
                    return TokenType::BadString;
                }
                *offset += 1;
            }

            // Backslash
            0x5C => {
                if *offset == source.len() - 1 {
                    *offset += 1;
                } else {
                    let next_code = get_char_code(source, *offset + 1);
                    if is_newline(next_code) {
                        *offset += 1 + get_newline_length(source, *offset + 1, next_code);
                    } else if is_valid_escape(code, next_code) {
                        *offset = consume_escaped(source, *offset);
                    } else {
                        *offset += 1;
                    }
                }
            }

            // Anything else
            _ => {
                *offset += 1;
            }
        }
    }

    // EOF before closing quote — still a string token (parse error in spec)
    TokenType::String
}

/// Consume a URL token (§4.3.6).
fn consume_url_token(source: &[u8], offset: &mut usize) -> TokenType {
    *offset = find_whitespace_end(source, *offset);

    while *offset < source.len() {
        let code = source[*offset];
        let category = char_code_category(u32::from(code));

        match category {
            // )
            0x29 => {
                *offset += 1;
                return TokenType::Url;
            }

            // Whitespace
            cat if cat == CharCategory::WhiteSpace as u8 => {
                *offset = find_whitespace_end(source, *offset);
                if get_char_code(source, *offset) == 0x29 || *offset >= source.len() {
                    if *offset < source.len() {
                        *offset += 1;
                    }
                    return TokenType::Url;
                }
                *offset = consume_bad_url_remnants(source, *offset);
                return TokenType::BadUrl;
            }

            // Quote, (, or non-printable → bad URL
            0x22 | 0x27 | 0x28 => {
                *offset = consume_bad_url_remnants(source, *offset);
                return TokenType::BadUrl;
            }
            cat if cat == CharCategory::NonPrintable as u8 => {
                *offset = consume_bad_url_remnants(source, *offset);
                return TokenType::BadUrl;
            }

            // Backslash
            0x5C => {
                if is_valid_escape(code, get_char_code(source, *offset + 1)) {
                    *offset = consume_escaped(source, *offset);
                } else {
                    *offset = consume_bad_url_remnants(source, *offset);
                    return TokenType::BadUrl;
                }
            }

            // Anything else
            _ => {
                *offset += 1;
            }
        }
    }

    // EOF before closing ) — still a URL token (parse error in spec)
    TokenType::Url
}

/// Find the end of a `/* ... */` comment.
fn find_comment_end(source: &[u8], start: usize) -> usize {
    let mut i = start;
    while i + 1 < source.len() {
        if source[i] == b'*' && source[i + 1] == b'/' {
            return i + 2;
        }
        i += 1;
    }
    // Unterminated comment → consume to end
    source.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect_tokens(source: &str) -> Vec<(TokenType, String)> {
        let mut tokens = Vec::new();
        tokenize(source, |tt, start, end| {
            tokens.push((tt, source[start..end].to_string()));
        });
        tokens
    }

    #[test]
    fn empty_source() {
        let tokens = collect_tokens("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn whitespace_token() {
        let tokens = collect_tokens("   ");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].0, TokenType::WhiteSpace);
    }

    #[test]
    fn ident_token() {
        let tokens = collect_tokens("color");
        assert_eq!(tokens, vec![(TokenType::Ident, "color".to_string())]);
    }

    #[test]
    fn simple_rule() {
        let tokens = collect_tokens("a { }");
        let types: Vec<_> = tokens.iter().map(|(t, _)| *t).collect();
        assert_eq!(
            types,
            vec![
                TokenType::Ident,
                TokenType::WhiteSpace,
                TokenType::LeftCurlyBracket,
                TokenType::WhiteSpace,
                TokenType::RightCurlyBracket,
            ]
        );
    }

    #[test]
    fn number_and_dimension() {
        let tokens = collect_tokens("10px");
        assert_eq!(tokens, vec![(TokenType::Dimension, "10px".to_string())]);
    }

    #[test]
    fn percentage() {
        let tokens = collect_tokens("50%");
        assert_eq!(tokens, vec![(TokenType::Percentage, "50%".to_string())]);
    }

    #[test]
    fn function_token() {
        let tokens = collect_tokens("rgb(");
        assert_eq!(tokens, vec![(TokenType::Function, "rgb(".to_string())]);
    }

    #[test]
    fn at_keyword() {
        let tokens = collect_tokens("@media");
        assert_eq!(tokens, vec![(TokenType::AtKeyword, "@media".to_string())]);
    }

    #[test]
    fn hash_token() {
        let tokens = collect_tokens("#fff");
        assert_eq!(tokens, vec![(TokenType::Hash, "#fff".to_string())]);
    }

    #[test]
    fn string_double_quotes() {
        let tokens = collect_tokens("\"hello\"");
        assert_eq!(tokens, vec![(TokenType::String, "\"hello\"".to_string())]);
    }

    #[test]
    fn string_single_quotes() {
        let tokens = collect_tokens("'world'");
        assert_eq!(tokens, vec![(TokenType::String, "'world'".to_string())]);
    }

    #[test]
    fn comment() {
        let tokens = collect_tokens("/* test */");
        assert_eq!(tokens, vec![(TokenType::Comment, "/* test */".to_string())]);
    }

    #[test]
    fn cdo_cdc() {
        let tokens = collect_tokens("<!--");
        assert_eq!(tokens, vec![(TokenType::Cdo, "<!--".to_string())]);

        let tokens = collect_tokens("-->");
        assert_eq!(tokens, vec![(TokenType::Cdc, "-->".to_string())]);
    }

    #[test]
    fn url_unquoted() {
        let tokens = collect_tokens("url(foo.png)");
        assert_eq!(tokens, vec![(TokenType::Url, "url(foo.png)".to_string())]);
    }

    #[test]
    fn url_quoted() {
        let tokens = collect_tokens("url(\"foo.png\")");
        let types: Vec<_> = tokens.iter().map(|(t, _)| *t).collect();
        // url( → Function, "foo.png" → String, ) → RightParenthesis
        assert_eq!(
            types,
            vec![
                TokenType::Function,
                TokenType::String,
                TokenType::RightParenthesis,
            ]
        );
    }

    #[test]
    fn delimiters() {
        let tokens = collect_tokens(".:;,()[]{}");
        let types: Vec<_> = tokens.iter().map(|(t, _)| *t).collect();
        assert_eq!(
            types,
            vec![
                TokenType::Delim,         // .
                TokenType::Colon,         // :
                TokenType::Semicolon,     // ;
                TokenType::Comma,         // ,
                TokenType::LeftParenthesis,
                TokenType::RightParenthesis,
                TokenType::LeftSquareBracket,
                TokenType::RightSquareBracket,
                TokenType::LeftCurlyBracket,
                TokenType::RightCurlyBracket,
            ]
        );
    }

    #[test]
    fn declaration() {
        let tokens = collect_tokens("color: red");
        let types: Vec<_> = tokens.iter().map(|(t, _)| *t).collect();
        assert_eq!(
            types,
            vec![
                TokenType::Ident,      // color
                TokenType::Colon,      // :
                TokenType::WhiteSpace, // space
                TokenType::Ident,      // red
            ]
        );
    }
}
