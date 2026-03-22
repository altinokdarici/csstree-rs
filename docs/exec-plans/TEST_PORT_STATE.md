# Test Porting State

Tracks progress of porting JS test cases to Rust.

## Current

```yaml
status: IN_PROGRESS
target: 2957
current: 245
coverage_pct: 8.3
remaining_fixture_files: 107
remaining_fixture_cases: 2538
remaining_inline_cases: ~174
```

## Strategy

Port in this priority order (highest ROI first):
1. **tokenize/** fixtures (7 files, 112 cases) — straightforward token verification
2. **ast/** fixtures (71 files, ~722 cases) — parse→generate round trips
3. **definition-syntax-match/** fixtures (12 files, ~1046 cases) — lexer validation
4. **definition-syntax/** fixtures (17 files, ~718 cases) — parse→generate round trips
5. **Inline tests** (~174 cases) — JS it() blocks

## Progress Log

| Date | Fixture | Cases | Total Coverage |
|------|---------|-------|---------------|
| (start) | — | 245/2957 | 8.3% |
