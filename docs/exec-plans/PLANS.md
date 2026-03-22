# Execution Plans

## Active

| Phase | Plan | Status | Blocked By |
|-------|------|--------|------------|
| 1 | [Tokenizer](active/phase-01-tokenizer.md) | COMPLETE | — |
| 2 | [AST Types](active/phase-02-ast.md) | COMPLETE | — |
| 3 | Parser | COMPLETE | — |
| 4 | Generator | COMPLETE | — |
| 5 | Walker | COMPLETE | Phase 2 |
| 6 | Definition Syntax | NOT STARTED | — |
| 7 | Lexer | NOT STARTED | Phase 1, 2, 6 |
| 8 | Utils | NOT STARTED | Phase 1 |
| 9 | Syntax Config / Fork | NOT STARTED | All above |

## Completed

| 1 | Tokenizer | 2026-03-21 | 86 tests, clippy clean |
| 2 | AST Types | 2026-03-21 | 49 node types, 4 tests, clippy clean |
| 3 | Parser | 2026-03-22 | 133 tests, 74 fixture files, UTF-8 fix, clippy clean |
| 4 | Generator | 2026-03-22 | 176 tests, auto-whitespace, round-trip, clippy clean |
| 5 | Walker | 2026-03-22 | 58 tests (27 unit + 20 fixture + 11 inline), fast traversal, clippy clean |

## Tech Debt Tracker

(None yet — we start clean)
