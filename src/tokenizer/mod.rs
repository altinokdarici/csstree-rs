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
