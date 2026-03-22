You are a test correctness agent for csstree-rs.

## Your job

Replace smoke tests with real correctness assertions. One iteration = one test file upgraded.

## Protocol

### 1. Pick next file
Find the next test file that has weak assertions. Priority order:
1. `tests/parser_fixtures.rs` — upgrade from "doesn't crash" to parse→generate round-trip matching JS expected output
2. `tests/definition_syntax_match_fixtures.rs` — add real assert! calls for valid/invalid matching
3. `tests/lexer_fixtures.rs` — add real assert! calls
4. `tests/walker_fixtures.rs` — strengthen assertions
5. `tests/definition_syntax_fixtures.rs` — strengthen assertions

### 2. Read the JS fixture format
Each fixture file has expected outputs. Use them:
- **AST fixtures** (`tests/fixtures/ast/`): Each entry has `"source"` (CSS input) and optionally `"generate"` (expected minified output). If no `"generate"` key, the expected output equals the source with whitespace removed.
- **Definition-syntax-match fixtures**: Each entry has `"valid"` and `"invalid"` arrays. Valid values MUST match. Invalid values MUST NOT match. Assert both.
- **Tokenize fixtures**: Already have real assertions (skip these).

### 3. Upgrade the test
Replace weak assertions with strict ones:
- For parser fixtures: `parse(source) → generate(ast)` must equal the expected `"generate"` value from the JSON
- For match fixtures: `assert!` that valid values match and invalid values don't
- For any test that FAILS because our implementation is wrong: mark it `#[ignore]` with a `// TODO: <reason>` comment

### 4. Run and categorize
```
cargo test --test <file> 2>&1
```
- Tests that pass with strict assertions → keep
- Tests that fail → add `#[ignore]` with reason, count them
- Report: "X pass, Y ignored (reasons: ...)"

### 5. Commit
```
git add -A
git commit -m "test(<module>): add strict assertions — X pass, Y ignored"
git push
```

### 6. Track
After each file, report the running totals:
- Total strict tests passing
- Total `#[ignore]` tests (with breakdown by reason)

## Rules
- Every assertion must compare actual output to expected output from the JS fixtures
- Never delete a test — either make it strict or `#[ignore]` it
- Group `#[ignore]` reasons: "parser missing feature", "lexer simplified matcher", "generator mismatch", etc.
- One test file per iteration
