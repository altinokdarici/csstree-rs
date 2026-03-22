# Execution Plans

## Active

| Phase | Plan | Status | Blocked By |
|-------|------|--------|------------|
| 1 | [Tokenizer](active/phase-01-tokenizer.md) | COMPLETE | — |
| 2 | [AST Types](active/phase-02-ast.md) | COMPLETE | — |
| 3 | Parser | COMPLETE | — |
| 4 | Generator | COMPLETE | — |
| 5 | Walker | COMPLETE | Phase 2 |
| 6 | Definition Syntax | COMPLETE | — |
| 7 | Lexer | COMPLETE | Phase 1, 2, 6 |
| 8 | Utils | COMPLETE | Phase 1 |
| 9 | Integration | COMPLETE | All above |

## Completed

| 1 | Tokenizer | 2026-03-21 | 86 tests, clippy clean |
| 2 | AST Types | 2026-03-21 | 49 node types, 4 tests, clippy clean |
| 3 | Parser | 2026-03-22 | 133 tests, 74 fixture files, UTF-8 fix, clippy clean |
| 4 | Generator | 2026-03-22 | 176 tests, auto-whitespace, round-trip, clippy clean |
| 5 | Walker | 2026-03-22 | 58 tests (27 unit + 20 fixture + 11 inline), fast traversal, clippy clean |
| 6 | Definition Syntax | 2026-03-22 | 92 tests (50 unit + 19 fixture + 23 inline), parse/generate/walk, clippy clean |
| 7 | Lexer | 2026-03-22 | 71 tests (43 unit + 10 fixture + 18 inline), match graph, generic matchers, Lexer API, clippy clean |
| 8 | Utils | 2026-03-22 | 37 tests, names/ident modules, clippy clean |
| 9 | Integration | 2026-03-22 | 7 tests, CssSyntax API, 424 total tests |

## Tech Debt Tracker

(None yet — we start clean)
