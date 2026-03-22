# Correctness Tracking

Real-time tracking of parse→generate round-trip correctness vs JS csstree.

## Current Scores

```yaml
parser_pass: 430
parser_total: 662
parser_pct: 65.0
lexer_pass: 657
lexer_total: 1027
lexer_pct: 64.0
```

## Fix Priority Queue

| # | Fix | Est. Impact | Status |
|---|-----|-------------|--------|
| 1 | Parse parenthesized at-rule preludes (@supports, @media features) | ~80 | TODO |
| 2 | Strip comments in raw content during parse | ~40 | TODO |
| 3 | Whitespace normalization in function/paren args | ~35 | TODO |
| 4 | CSS nesting detection in declaration blocks | ~20 | TODO |
| 5 | Custom property value handling (empty, !important) | ~12 | TODO |
| 6 | `progid:` IE filter special case | ~10 | TODO |
| 7 | String quote normalization | ~8 | DONE (+3) |
| 8 | `url()` double-wrapping | ~6 | DONE (+5) |
| 9 | Preserve `/*!` stylesheet comments | ~7 | TODO |
| 10 | Misc (nth-selector, /deep/, @layer dots) | ~15 | TODO |

## History

- 2026-03-22: Baseline — 422/662 parser, 657/1027 lexer
- 2026-03-22: Fix #7 (string quotes) — 425/662
- 2026-03-22: Fix #8 (url double-wrap) — 430/662
