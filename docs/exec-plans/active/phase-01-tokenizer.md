# Phase 1: Tokenizer

**Status:** NOT STARTED
**Depends on:** Nothing
**Unlocks:** Parser (Phase 3)

## Goal

Port `external/csstree/lib/tokenizer/` to `src/tokenizer/` with 100% test parity.

## JS Source Files to Port

| JS File | Rust Target | Description |
|---------|-------------|-------------|
| `lib/tokenizer/types.js` | `src/tokenizer/types.rs` | 25 token type constants |
| `lib/tokenizer/char-code-definitions.js` | `src/tokenizer/char_code_definitions.rs` | `isDigit`, `isHexDigit`, `isLetter`, `isNewline`, etc. |
| `lib/tokenizer/names.js` | `src/tokenizer/names.rs` | Token type → name string |
| `lib/tokenizer/utils.js` | `src/tokenizer/utils.rs` | `consumeNumber`, `consumeEscaped`, `consumeName`, etc. |
| `lib/tokenizer/index.js` | `src/tokenizer/mod.rs` | `tokenize()` entry point |
| `lib/tokenizer/TokenStream.js` | `src/tokenizer/token_stream.rs` | Token stream with indexed access |
| `lib/tokenizer/OffsetToLocation.js` | `src/tokenizer/offset_to_location.rs` | Byte offset to line/column |

## Test Sources

| JS Test | Description |
|---------|-------------|
| `lib/__tests/tokenizer.js` | Main tokenizer tests |
| `lib/__tests/fixture/tokenize.js` | Fixture loader |
| `fixtures/tokenize/` | Tokenization fixture data |

## Steps

1. Read all JS source files listed above
2. Implement `types.rs` — token type enum with `#[repr(u8)]`
3. Implement `char_code_definitions.rs` — character classification functions
4. Implement `names.rs` — token type to string name mapping
5. Implement `utils.rs` — consume functions (number, escaped, name, bad-url-remnants, etc.)
6. Implement `mod.rs` — main `tokenize()` function
7. Implement `token_stream.rs` — `TokenStream` struct
8. Implement `offset_to_location.rs` — `OffsetToLocation`
9. Port all test fixtures from `fixtures/tokenize/`
10. Port all test cases from `lib/__tests/tokenizer.js`
11. Run `cargo test --lib tokenizer` — all must pass
12. Run `cargo clippy -- -D warnings` — clean

## Acceptance Criteria

- [ ] All 25 token types defined
- [ ] `tokenize()` produces identical token sequences to JS for all fixture inputs
- [ ] All JS tokenizer test cases have Rust equivalents
- [ ] Zero-copy: tokens reference source slice, no string allocation
- [ ] `cargo clippy` clean

## Decision Log

(Decisions made during implementation go here)
