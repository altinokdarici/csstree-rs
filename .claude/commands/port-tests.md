You are an autonomous test-porting agent for csstree-rs.

## Your job

Port JS test cases to Rust `#[test]` functions until 100% coverage. One iteration = one fixture file fully ported.

## Execution protocol

### 1. Read state
Run `cargo run --bin test-coverage 2>&1 | grep "TODO" | sed 's/│//g' | head -1` to find the next unported fixture file.

### 2. Read the fixture file
Read the fixture JSON from `tests/fixtures/<path>`. Understand the structure:
- For `ast/` fixtures: `{ "test_name": { "source": "css", "ast": {...}, "generate": "css" } }`
- For `tokenize/` fixtures: `{ "valid": [...], "invalid": [...] }` with token type/value expectations
- For `definition-syntax/` fixtures: `{ "test_name": { "syntax": "...", "valid": [...], "invalid": [...] } }`
- For `definition-syntax-match/` fixtures: `{ "test_name": { "syntax": "...", "valid": [...], "invalid": [...] } }`

### 3. Write tests
- For `ast/` fixtures: Write parse→AST verification tests OR parse→generate round-trip tests
  - Put tests in the appropriate existing integration test file (e.g., `tests/parser_fixtures.rs`)
  - OR create a new targeted test file if needed
- For `tokenize/` fixtures: Write tokenization tests verifying token types and values
  - Put in `tests/tokenizer_fixtures.rs`
- For `definition-syntax/` fixtures: Write parse→generate round-trip tests
  - Put in `tests/definition_syntax_fixtures.rs`
- For `definition-syntax-match/` fixtures: Write Lexer matching tests
  - Put in `tests/lexer_fixtures.rs`

### 4. Verify
Run `cargo test --test <test_file>` to verify. Fix failures.

### 5. Commit
```sh
git add -A
git commit -m "test(<module>): port <fixture_file> (<N> cases)"
git push
```

### 6. Track progress
After each commit, run `cargo run --bin test-coverage 2>&1 | tail -5` and report progress.

## Rules
- Port ALL test cases from a fixture file in one iteration
- If a test case fails because our implementation doesn't support a feature, mark it `#[ignore]` with a comment explaining why
- Never skip fixture files — port them all, even if some tests need `#[ignore]`
- Use the existing test infrastructure patterns from the codebase
- Batch related fixture files when they're small (e.g., all `ast/selector/functional-pseudo/*.json`)
- For very large fixture files (100+ cases), it's OK to do it in chunks
