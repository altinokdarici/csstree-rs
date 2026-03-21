# Core Beliefs

These define the agent-first operating principles for csstree-rs.

## 1. The JS source is the specification

`external/csstree/` is ground truth. Every behavior, edge case, and error message comes from reading that code. When in doubt, read the JS. When the JS is ambiguous, write a test in the JS repo and observe the behavior.

## 2. Correctness before performance

Get every test passing first. Optimize later with benchmarks as proof. Premature optimization in a parser leads to subtle bugs that compound.

## 3. Enforce invariants mechanically, not with prose

If a rule matters, encode it as:
- A `cargo clippy` lint
- A compile-time type constraint
- A `#[test]` that fails when violated

Documentation alone rots. Code enforces.

## 4. Every module must be self-verifiable

Each module must have:
- Unit tests alongside the code (`#[cfg(test)]`)
- Integration tests that load the JS fixtures directly
- `cargo test --lib <module>` must pass in isolation

An agent should be able to verify its own work without human QA.

## 5. The repository is the only context

All architectural decisions, design rationale, and implementation notes live in `docs/`. Nothing lives in Slack, email, or someone's head. If it's not in the repo, it doesn't exist for the agent.

## 6. Strict module boundaries

Dependencies flow in one direction only:

```
tokenizer (no deps)
    ↓
ast (no deps)
    ↓
parser (tokenizer + ast)
    ↓
generator (ast)
walker (ast)
definition_syntax (no deps)
    ↓
lexer (tokenizer + ast + definition_syntax)
    ↓
utils (may use any above)
```

Circular dependencies are a compile error. Cross-module coupling is a design error.

## 7. Prefer Rust idioms over JS translation

Don't transliterate JS patterns into Rust. Use:
- Enums over stringly-typed values
- `Result<T, E>` over exceptions
- Iterators over manual loops
- `Option<T>` over null/undefined checks
- Ownership and borrowing over GC patterns

The output should feel like idiomatic Rust, not JS wearing a Rust costume.

## 8. Agent struggles are environment failures

When the agent can't make progress, the fix is never "try harder." The fix is:
- Missing abstraction? Build it.
- Missing test fixture? Port it.
- Unclear behavior? Document it in `docs/`.
- Ambiguous architecture? Add a structural test.

## 9. Zero-copy where the hot path demands it

The tokenizer and parser operate on `&str` slices into the source. AST nodes reference the source string rather than copying. Heap allocations on the hot path require justification in a comment.

## 10. Entropy is managed continuously

Bad patterns spread if left alone. Every module completion includes a quality check:
- Run `cargo clippy -- -D warnings`
- Verify no `#[allow(unused)]` on public items
- Check that test count matches or exceeds JS test count for that module

## 11. The system improves itself

Every iteration of the build loop ends with a self-improvement reflection. The agent asks: what friction did I hit? What's missing? What could be automated? Then it fixes the infrastructure — adds missing docs, creates new skills/commands, updates golden principles, improves error messages. The build system, docs, and tooling get better with every iteration, not just the code.

This means:
- New patterns discovered during implementation get promoted into golden principles
- Repeated manual steps get automated into scripts or commands
- Stale docs get fixed immediately, not deferred
- Missing test utilities get created when the gap is felt
- The CLAUDE.md, build-next protocol, and conventions evolve as the project matures
