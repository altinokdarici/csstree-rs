# Correctness Tracking

Real-time tracking of parse→generate round-trip correctness vs JS csstree.

## Current Scores

```yaml
parser_pass: 563
parser_total: 662
parser_pct: 85.0
lexer_pass: 657
lexer_total: 1027
lexer_pct: 64.0
```

## Fix Priority Queue

| # | Fix | Est. Impact | Status |
|---|-----|-------------|--------|
| 1 | Parse parenthesized at-rule preludes (@supports, @media features) | ~80 | PARTIAL (+17) |
| 2 | Strip comments in raw content during parse | ~40 | TODO |
| 3 | Whitespace normalization in function/paren args | ~35 | TODO |
| 4 | CSS nesting detection in declaration blocks | ~20 | DONE (+23) |
| 5 | Custom property value handling (empty, !important) | ~12 | DONE (+5) |
| 6 | `progid:` IE filter special case | ~10 | DONE (+14) |
| 7 | String quote normalization | ~8 | DONE (+3) |
| 8 | `url()` double-wrapping | ~6 | DONE (+5) |
| 9 | Preserve `/*!` stylesheet comments | ~7 | DONE (+5) |
| 10 | Misc (nth-selector, /deep/, @layer dots) | ~15 | TODO |

## History

- 2026-03-22: Baseline — 422/662 parser, 657/1027 lexer
- 2026-03-22: Fix #7 (string quotes) — 425/662
- 2026-03-22: Fix #8 (url double-wrap) — 430/662
- 2026-03-22: Fix #9 (comments) — 435/662
- 2026-03-22: Fix #5 (custom property values) + consume_raw stop fix — 440/662
- 2026-03-22: Attr selector whitespace/comment skip — 449/662
- 2026-03-22: More value delimiters + atrule prelude colon — 466/662
- 2026-03-22: Value context wrapping fix — 486/662
- 2026-03-22: Parenthesized colon handling — 502/662
- 2026-03-22: Namespace attr selectors + value delimiters — 508/662
- 2026-03-22: Progid IE filter — 522/662
- 2026-03-22: Pseudo-class/element args + pseudo-element — 531/662
- 2026-03-22: CSS nesting detection heuristic — 554/662
- 2026-03-22: Keyframe selectors + custom property whitespace + function colons — 563/662
