# Loop State

Machine-readable state for the autonomous build loop. Updated after every iteration.

## Current

```yaml
phase: 1
phase_name: tokenizer
step: 2
step_name: implement_types
status: IN_PROGRESS
blocked: false
blocker: null
last_completed_phase: 0
last_commit: null
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

- **2026-03-21 P1S1** read_js_source — Read all 7 JS tokenizer files (types, char-code-definitions, names, utils, index, TokenStream, OffsetToLocation). Wrote architecture summary to src/tokenizer/mod.rs.
