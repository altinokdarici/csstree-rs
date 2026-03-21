# csstree-rs

Self-maintaining CSS toolkit in Rust. Agent-built, agent-shipped, agent-maintained.

## Reference

- **JS source (ground truth):** `external/csstree/`
- **JS tests:** `external/csstree/lib/__tests/`
- **JS fixtures:** `external/csstree/fixtures/`

Always read the JS source before writing the Rust equivalent.

## Knowledge Base (start here, then follow pointers)

- [docs/design-docs/project-goals.md](docs/design-docs/project-goals.md) — Vision, goal tiers, success metrics
- [docs/design-docs/core-beliefs.md](docs/design-docs/core-beliefs.md) — Operating principles
- [docs/design-docs/architecture.md](docs/design-docs/architecture.md) — Module map, dependency direction, design decisions
- [docs/golden-principles.md](docs/golden-principles.md) — Mechanical rules for code consistency (32 rules)
- [docs/references/rust-conventions.md](docs/references/rust-conventions.md) — Rust coding conventions (from MS guidelines)
- [docs/exec-plans/PLANS.md](docs/exec-plans/PLANS.md) — All phases, status, dependencies
- [docs/QUALITY.md](docs/QUALITY.md) — Per-module quality grades

## Agent Commands

| Command | Tier | Purpose |
|---------|------|---------|
| `/build-next` | 1: Build | One iteration of the build loop — implements next step |
| `/sync-upstream` | 3: Maintain | Detect + port changes from upstream JS csstree |
| `/quality-sweep` | 3: Maintain | Scan for code quality issues, fix, update grades |
| `/ship` | 2: Ship | Prepare and execute a crates.io release |

## Module Layout

```
src/tokenizer/       → external/csstree/lib/tokenizer/
src/ast/             → external/csstree/lib/syntax/node/
src/parser/          → external/csstree/lib/parser/
src/generator/       → external/csstree/lib/generator/
src/walker/          → external/csstree/lib/walker/
src/definition_syntax/ → external/csstree/lib/definition-syntax/
src/lexer/           → external/csstree/lib/lexer/
src/utils/           → external/csstree/lib/utils/
```

## Commands

```sh
cargo check                      # type check
cargo test                       # all tests
cargo test --lib tokenizer       # module tests
cargo clippy -- -D warnings      # lint (must pass)
```

## Verification Loop

Every change must pass all three:
```sh
cargo check && cargo test && cargo clippy -- -D warnings
```

## Test Infrastructure

Test cases are owned copies synced from `external/csstree/`, not live references.

```sh
node scripts/extract_test_manifest.mjs  # sync fixtures + extract inline tests
cargo run --bin test-coverage            # compare manifest vs Rust #[test] functions
```

- `tests/fixtures/` — our copy of all fixture JSON files (tokenize, ast, definition-syntax, etc.)
- `tests/fixtures/inline/` — inline test cases extracted from JS `it()` blocks as JSON
- `tests/test_manifest.json` — canonical list of all 2,957 test cases with IDs, CSS inputs, expected outputs
- Re-run the sync script anytime `external/csstree/` is updated to pick up changes

Rust tests load from `tests/fixtures/` (our copy), never from `external/` directly.

## Autonomous Build Loop

The project is built by a Ralph Wiggum loop running `/build-next` on repeat.

- **State:** [docs/exec-plans/LOOP_STATE.md](docs/exec-plans/LOOP_STATE.md) — machine-readable YAML, updated each iteration
- **Command:** `.claude/commands/build-next.md` — full protocol for each iteration
- **Start:** `/ralph-wiggum:ralph-loop` then enter `/build-next`

Each iteration = one step. 10 steps per phase. 9 phases total. ~90 iterations to completion.

### Step sequence per phase:
1. read_js_source → 2. implement_types → 3. implement_core → 4. implement_remaining →
5. port_fixture_tests → 6. port_inline_tests → 7. verify → 8. coverage_check →
9. update_status → 10. advance

## Current Status

See [docs/exec-plans/LOOP_STATE.md](docs/exec-plans/LOOP_STATE.md)
