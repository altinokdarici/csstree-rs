# Project Goals

## Vision

csstree-rs is not just a Rust port of csstree. It is a **self-maintaining software system** where an AI agent is responsible for the full lifecycle: building, shipping, maintaining, and iterating a production-quality CSS toolkit in Rust.

The human role is to set direction, define constraints, and validate outcomes. The agent handles everything else.

## Goal Tiers

### Tier 1: Build (current)

Port csstree to Rust with 100% feature parity and all test cases.

**Done when:**
- All 9 modules implemented (tokenizer → integration)
- `cargo run --bin test-coverage` shows 2957/2957
- `cargo test` passes, `cargo clippy -- -D warnings` clean
- Public API matches JS surface: parse, generate, walk, find, findAll, clone, tokenize, lexer, fork

### Tier 2: Ship

Package and publish as a usable Rust crate.

**Done when:**
- Published to crates.io as `csstree`
- Complete rustdoc documentation on all public types and functions
- README with usage examples, feature comparison to JS version
- Benchmarks published: tokenizer, parser, generator throughput vs cssparser/lightningcss
- CI/CD pipeline: test, lint, bench, publish on tag
- MSRV (minimum supported Rust version) documented and tested
- `no_std` support evaluated and documented (feasibility)

### Tier 3: Maintain

Keep the Rust implementation in sync with upstream JS changes and healthy over time.

**Done when the agent can autonomously:**
- Detect upstream changes: monitor `external/csstree/` for new commits
- Re-run `node scripts/extract_test_manifest.mjs` to detect added/changed/removed test cases
- Run `cargo run --bin test-coverage` to identify gaps
- Implement missing features and tests to close gaps
- Fix regressions caught by CI
- Update dependencies (`cargo update`, review changelogs)
- Triage and fix issues filed on the repo
- Open PRs for each change, self-review, merge when CI passes

**Maintenance loops:**
- **Upstream sync** — periodic check for JS changes, port delta
- **Dependency update** — periodic `cargo update` + test
- **Quality sweep** — scan for code smells, update QUALITY.md grades
- **Doc gardening** — verify docs match code, fix stale references

### Tier 4: Iterate

Improve the implementation beyond the JS reference: better performance, Rust-native features, ecosystem integration.

**Done when the agent can autonomously:**
- Profile and optimize hot paths (tokenizer, parser) with criterion benchmarks
- Implement Rust-specific features not in JS: `serde` serialization, `Display`/`FromStr` traits, `no_std` mode
- Add WASM target support (`wasm32-unknown-unknown`)
- Integrate with Rust CSS ecosystem: provide adapters for `cssparser`, `style` crate patterns
- Respond to user feature requests from GitHub issues
- Write and publish blog posts about performance characteristics
- Evaluate and adopt new Rust language features as they stabilize

## Non-Goals

- We do NOT aim to deviate from csstree's behavior. The JS implementation defines correctness.
- We do NOT build a CSS engine (layout, rendering). This is parsing, validation, and transformation only.
- We do NOT maintain backward compatibility with pre-1.0 API changes. Move fast until 1.0.

## Success Metrics

| Metric | Tier 1 | Tier 2 | Tier 3 | Tier 4 |
|--------|--------|--------|--------|--------|
| Test coverage | 2957/2957 | Same | Tracks upstream | Same + Rust-specific |
| CI status | Passes | Green + publish | Green + auto-fix | Green + bench regression |
| Human intervention | Per-session steering | Release approval | Issue triage only | Direction setting only |
| Agent autonomy | Build loop | Build + publish | Build + maintain | Build + maintain + iterate |

## Agent Capability Roadmap

### Phase A: Builder (Tier 1)
The agent follows explicit instructions step-by-step. Each iteration is one step. Human reviews commits.

### Phase B: Shipper (Tier 2)
The agent handles packaging, documentation, and CI setup. Human approves releases.

### Phase C: Maintainer (Tier 3)
The agent runs maintenance loops autonomously. It detects upstream changes, implements fixes, and opens PRs. Human approves merges for non-trivial changes.

### Phase D: Iterator (Tier 4)
The agent identifies improvement opportunities, implements them, and validates with benchmarks. Human sets priorities and approves architectural changes.

## Current State

**Phase A: Builder — Tier 1 in progress.**

See [LOOP_STATE.md](../exec-plans/LOOP_STATE.md) for current build progress.
