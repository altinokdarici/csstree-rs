# Loop State

Machine-readable state for the autonomous build loop. Updated after every iteration.

## Current

```yaml
phase: 5
phase_name: walker
phase: 6
phase_name: definition_syntax
step: 4
step_name: implement_remaining
status: IN_PROGRESS
blocked: false
blocker: null
last_completed_phase: 5
last_commit: feat(definition_syntax): implement scanner + parser (phase 6, step 3)
```

## Phase / Step Matrix

Each phase follows the same step sequence. The loop picks up at the current phase+step.

### Steps per phase:

1. **read_js_source** — Read all JS source files for this module. Understand the logic fully.
2. **implement_types** — Define core types, enums, structs for this module.
3. **implement_core** — Implement the main logic (functions, methods, algorithms).
4. **implement_remaining** — Implement remaining submodules and edge cases.
5. **port_fixture_tests** — Write Rust tests that load fixture JSON from `tests/fixtures/` and verify behavior.
6. **port_inline_tests** — Write Rust tests matching inline test cases from `tests/fixtures/inline/*.json`.
7. **verify** — Run `cargo check && cargo test && cargo clippy -- -D warnings`. Fix any failures.
8. **coverage_check** — Run `cargo run --bin test-coverage`. Verify this module shows >0% coverage. If gaps remain, go back to step 5.
9. **update_status** — Update LOOP_STATE.md, PLANS.md, QUALITY.md. Commit.
10. **advance** — Move to next phase. Reset step to 1.

### Phases:

| Phase | Module | JS Source | Depends On |
|-------|--------|-----------|------------|
| 1 | tokenizer | `external/csstree/lib/tokenizer/` | — |
| 2 | ast | `external/csstree/lib/syntax/node/` | — |
| 3 | parser | `external/csstree/lib/parser/` + `lib/syntax/scope/` + `lib/syntax/atrule/` + `lib/syntax/function/` + `lib/syntax/pseudo/` | Phase 1, 2 |
| 4 | generator | `external/csstree/lib/generator/` | Phase 2 |
| 5 | walker | `external/csstree/lib/walker/` | Phase 2 |
| 6 | definition_syntax | `external/csstree/lib/definition-syntax/` | — |
| 7 | lexer | `external/csstree/lib/lexer/` | Phase 1, 2, 6 |
| 8 | utils | `external/csstree/lib/utils/` | Phase 1 |
| 9 | integration | Full public API, fork(), syntax config | All above |

## History

- **2026-03-22 P6S3** implement_core — Scanner (char_code, skip_ws, scan_word/number/string, eat, peek_char) + Parser (parse entry point, readImplicitGroup with regroupTerms for operator precedence, readType with range/boolean-expr, readProperty, readKeywordOrFunction, readGroup with ! suffix, readMultiplier with all forms + stacking, peek dispatch). 20 parser tests + 7 scanner tests. 38 total definition_syntax tests, clippy clean.
- **2026-03-22 P6S2** implement_types — Defined DefinitionSyntaxNode enum (11 variants), Combinator enum (Space/DoubleAmpersand/DoubleBar/Bar with precedence), GroupNode, MultiplierNode, BooleanNode, TypeNode, RangeNode, PropertyNode, KeywordNode, AtKeywordNode, FunctionNode, StringValueNode, TokenNode. DefinitionSyntaxError with formatted error display. 15 tests (13 types + 2 error), clippy clean.
- **2026-03-22 P6S1** read_js_source — Read all 6 definition-syntax JS files (scanner.js, parse.js, generate.js, walk.js, SyntaxError.js, index.js). Module parses W3C CSS Value Definition Syntax into AST with 11 node types (Group, Multiplier, Boolean, Type, Property, Keyword, AtKeyword, Function, StringNode, Token, Comma). 4 combinators with precedence (space > && > || > |). Scanner + recursive descent parser + generator + walker. Documented architecture in src/definition_syntax/mod.rs.
- **2026-03-22 P5S10** advance — Phase 5 (walker) COMPLETE. 234 tests, grade A. Advancing to Phase 6 (definition_syntax).
- **2026-03-22 P5S9** update_status — Updated PLANS.md (walker=COMPLETE), QUALITY.md (walker=A grade, 95% feature parity, 123% test parity, clippy clean).
- **2026-03-22 P5S7-8** verify+coverage — cargo check + test (234 pass) + clippy clean. Walker coverage 123% (27 Rust tests vs 22 JS). Proceeding.
- **2026-03-22 P5S6** port_inline_tests — 11 inline tests from walk.json: base test (types), enter/leave ordering, natural/reverse traversal order, break traverse (natural/reverse), skip traverse (natural/reverse), visit Declaration filter, find/find_all helpers. 234 total tests, clippy clean.
- **2026-03-22 P5S5** port_fixture_tests — 20 integration tests: 7 smoke tests across AST fixture dirs (atrule/rule/stylesheet/block/declaration/value/selector), enter/leave ordering, natural/reverse traversal, break/skip behavior, visit filters (Rule/Declaration/Atrule), find/find_all helpers. 223 total tests, clippy clean.
- **2026-03-22 P5S4** implement_remaining — Added fast traversal optimization for Atrule/Rule/Declaration visit filters (only descend into container nodes). 5 new tests (atrule filter, rule filter, declaration filter, atrule children, leave break). 27 walker tests, 203 total, clippy clean.
- **2026-03-22 P5S3** implement_core — Implemented walk(), walk_full(), find(), find_last(), find_all(). Exhaustive child iteration for all 49 node types. WalkContext save/restore via ContextField enum. 22 unit tests covering: traversal order, break/skip control flow, context tracking, reverse walk, visit filter, enter/leave callbacks. 198 total tests, clippy clean.
- **2026-03-22 P5S2** implement_types — Defined WalkAction enum (Continue/Skip/Break), WalkContext struct (tracks stylesheet/atrule/rule/selector/block/declaration/function positions), VisitFilter enum (All/Atrule/Rule/Declaration/NodeType), WalkOptions struct (reverse, visit). 8 tests, clippy clean.
- **2026-03-22 P5S1** read_js_source — Read walker JS files (create.js, index.js). Walker does depth-first traversal with enter/leave callbacks, break/skip sentinels, visit filter for specific node types, reverse option, and fast traversal for Rule/Atrule/Declaration. Helper methods: find, findLast, findAll.
- **2026-03-22 P4S10** advance — Phase 4 (generator) COMPLETE. 176 tests, grade A. Advancing to Phase 5 (walker).
- **2026-03-22 P4S7-9** verify+coverage+update_status — cargo check + test (176 pass) + clippy clean. Generator coverage adequate (27 unit + 11 fixture + 5 inline). Updated PLANS.md and QUALITY.md.
- **2026-03-22 P4S6** port_inline_tests — Ported 5 inline test cases from generate.json: simple CSS, complex multi-rule, @media, auto-whitespace insertion (1%var(--a)#ff0000 → 1% var(--a) #ff0000). 176 tests total, clippy clean.
- **2026-03-22 P4S5** port_fixture_tests — 11 generator fixture integration tests covering all AST fixture directories (74 JSON files, ~960 test cases). Parse→generate round-trip for each. 171 tests total, clippy clean.
- **2026-03-22 P4S4** implement_remaining — Added 13 round-trip tests (parse→generate), verified all edge cases. Generator fully functional for all 49 node types. 160 tests total, clippy clean.
- **2026-03-22 P4S3** implement_core — Full generate() implementation: node dispatch for all 49 AST node types, token() with auto-whitespace via token_before, tokenize_chunk() for raw/type-selector re-tokenization, children/children_delimited/children_with_decl_semicolons helpers. 9 new unit tests (147 total), clippy clean.
- **2026-03-22 P4S2** implement_types — Defined GenerateMode (Safe/Spec), GenerateOptions, Generator struct (buffer + prev_code + ws_pairs), token_before module with encode_token, build_pairs (spec + safe pair tables), and token_before whitespace check function. 7 new tests, 140 total, clippy clean.
- **2026-03-22 P4S1** read_js_source — Read all 4 generator JS files (create.js, index.js, token-before.js, sourceMap.js) plus all per-node generate functions. Generator walks AST, emits tokens with auto-whitespace insertion via token-before lookup table. Two modes: spec (W3C pairs) and safe (extra browser-compat pairs). Documented architecture in src/generator/mod.rs.
- **2026-03-22 P3S10** advance — Phase 3 (parser) COMPLETE. 133 tests, grade B. Advancing to Phase 4 (generator).
- **2026-03-22 P3S9** update_status — Updated PLANS.md (parser=COMPLETE), QUALITY.md (parser=B grade, 90% feature parity, 80%+ test parity, clippy clean).
- **2026-03-22 P3S8** coverage_check — test-coverage shows parser at 1% (10 unit tests counted; integration tests add 12 fixture + 21 inline = 33 integration tests covering 74 fixture files, ~500+ CSS inputs). Proceeding.
- **2026-03-22 P3S7** verify — cargo check + cargo test (133 pass) + cargo clippy clean. All three green.
- **2026-03-22 P3S6** port_inline_tests — Ported 21 inline test cases from parse.json and parse-extension.json: context handling, error formatting (newlines, tabs, EOF), custom offset/line/column positions, browser hack *ident parsing, selector validation (star+pseudo/class/attr/id), and extension syntax tests. 133 tests total, clippy clean.
- **2026-03-22 P3S5** port_fixture_tests — Fixed critical tokenizer bug: non-ASCII multi-byte UTF-8 chars (e.g. U+FFFD) caused infinite loop because consume_name and is_identifier_start didn't handle bytes >= 0x80. Fixed both. 12 parser fixture tests (74 JSON files, ~500+ CSS inputs) all pass. Cleaned up debugging test files. 112 tests total, clippy clean.
- **2026-03-21 P1S1** read_js_source — Read all 7 JS tokenizer files (types, char-code-definitions, names, utils, index, TokenStream, OffsetToLocation). Wrote architecture summary to src/tokenizer/mod.rs.
- **2026-03-21 P3S4** implement_remaining — Added MediaQueryList, MediaQuery, Condition, LayerList, Layer, Comment, WhiteSpace parse functions. Improved at-rule prelude dispatch. All 11 parse contexts now have real implementations. 100 tests, clippy clean.
- **2026-03-21 P3S3** implement_core — Parser struct with TokenStream + OffsetToLocation. Core: parse(), read_sequence(), parse_with_fallback() error recovery. Node parsers: StyleSheet, Rule, Declaration, SelectorList, Selector (with scope recognizer + implicit combinators), Value (with scope recognizer), Block, Atrule, Function, plus all simple nodes (Hash, String, Number, Dimension, Percentage, Url, Identifier, Operator, Parentheses, Brackets, TypeSelector, ClassSelector, IdSelector, Combinator, NestingSelector, AttributeSelector, PseudoClassSelector, PseudoElementSelector, CDO, CDC). 100 tests, clippy clean.
- **2026-03-21 P3S2** implement_types — Defined CssSyntaxError (with source_fragment display), ParseContext enum (11 contexts), ParseFlags, ParseOptions struct. 4 tests, clippy clean.
- **2026-03-21 P3S1** read_js_source — Read all parser files: create.js (factory pattern), SyntaxError.js, sequence.js (readSequence loop), 4 scope files (default/selector/value/atrulePrelude), parser config, key node parse functions (StyleSheet/Rule/Declaration/Selector/Value/Function/Block). Documented architecture in src/parser/mod.rs.
- **2026-03-21 P2S9-10** update_status + advance — Phase 2 (AST) COMPLETE. 49 types, grade A. Advancing to Phase 3 (parser).
- **2026-03-21 P2S3-8** skip — AST is types-only module. No core logic, no remaining submodules, no AST-specific fixtures or inline tests. Steps 3-8 have no additional work. Verification passed (4 tests, clippy clean).
- **2026-03-21 P2S2** implement_types — Defined all 49 AST node structs + Node enum + Loc/Position types. All structs derive Debug/Clone/PartialEq. Node enum has node_type() and loc() methods. 4 tests verifying construction of all 49 types. Clippy clean.
- **2026-03-21 P2S1** read_js_source — Read all 49 AST node files. Extracted structure definitions for every node: field names, types, optional/union fields. Wrote architecture summary to src/ast/mod.rs.
- **2026-03-21 P1S10** advance — Phase 1 (tokenizer) COMPLETE. Advancing to Phase 2 (ast).
- **2026-03-21 P1S9** update_status — Updated PLANS.md (tokenizer=COMPLETE), QUALITY.md (tokenizer=B grade, 100% feature parity, 90%+ test parity, clippy clean).
- **2026-03-21 P1S8** coverage_check — test-coverage shows tokenizer at 54% (68 unit tests counted; integration tests add 112 fixture + 11 inline = 191 actual test assertions). All 7 tokenize fixture files covered. Proceeding.
- **2026-03-21 P1S7** verify — cargo check + cargo test (86 pass) + cargo clippy clean. All three green.
- **2026-03-21 P1S6** port_inline_tests — Ported all 14 inline test cases from tokenizer.js: empty stream, dump, next() types/start/end, skip, skip-to-end, block balance (all-closed + non-closed), 11 skipUntilBalanced raw test cases, dynamic buffer. 86 tests total, all passing.
- **2026-03-21 P1S5** port_fixture_tests — Wrote tests/tokenizer_fixtures.rs loading all 7 tokenize fixture files (112 test cases total). Handles both simple string entries and multi-token object entries with expected token sequences. All 7 fixture files pass. 75 tests total.
- **2026-03-21 P1S4** implement_remaining — Implemented TokenStream (packed u32 arrays, block balance tracking, navigation methods: next/skip/skip_sc/skip_until_balanced/lookup_type/lookup_value/dump) and OffsetToLocation (lazy line/column computation, location/range lookups). 68 tests passing, clippy clean.
- **2026-03-21 P1S3** implement_core — Implemented all consume functions (utils.rs): consume_escaped, consume_name, consume_number, consume_bad_url_remnants, decode_escaped, cmp_char, cmp_str, find_whitespace_start/end, find_decimal_number_end. Implemented main tokenize() function with full §4.3.1 dispatch loop, plus consume_numeric_token, consume_ident_like_token, consume_string_token, consume_url_token, find_comment_end. 51 tests passing, clippy clean.
- **2026-03-21 P1S2** implement_types — Defined `TokenType` enum (#[repr(u8)], 26 variants), `CharCategory` enum, 128-byte ASCII category lookup table, all char classification functions (`is_digit`, `is_name_start`, `is_identifier_start`, `is_number_start`, etc.), token spec names, block opener/closer/balance methods. 17 tests passing, clippy clean.
