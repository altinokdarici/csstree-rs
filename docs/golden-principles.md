# Golden Principles

Opinionated, mechanical rules enforced by code (lints, tests, type constraints). When documentation falls short, promote the rule into code. Full Rust conventions in [docs/references/rust-conventions.md](references/rust-conventions.md).

## Code Structure

1. **One module = one responsibility.** No module should both tokenize and parse.
2. **Public API surface is explicit.** Every `pub` item in `lib.rs` is intentional. No `pub use foo::*`.
3. **Tests live next to code.** `#[cfg(test)] mod tests { ... }` at the bottom of each file. Integration tests in `tests/`.
4. **No `unwrap()` in library code.** Use `?` or explicit error handling. `unwrap()` is allowed only in tests.
5. **Use `#[expect(lint, reason = "...")]` not `#[allow]`** when overriding lints. `#[allow]` silently rots.
6. **Essential functionality is inherent.** Core methods in `impl Foo`, traits only for polymorphism.

## Naming

7. **Rust naming conventions.** `snake_case` functions, `PascalCase` types, `SCREAMING_SNAKE_CASE` constants.
8. **Match JS names where it aids traceability.** `TokenStream` → `TokenStream`, `AnPlusB` → `AnPlusB`.
9. **No weasel words.** No "Manager", "Service", "Handler", "Factory" in type names. Name what it *does*.
10. **Conversion naming:** `as_` (cheap borrow), `to_` (expensive/owned), `into_` (consuming).

## Types

11. **Derive eagerly.** `#[derive(Debug, Clone, PartialEq)]` on all types. Add `Eq, Hash, Default` where applicable.
12. **All public types implement `Debug`.** Enforced by lint `missing_debug_implementations`.
13. **Enums over strings.** Token types, node types, combinators — always enums, never `&str` comparisons.
14. **`Option<T>` over sentinel values.** No `-1` for "not found", no empty string for "not set".
15. **Newtypes for domain values.** `Offset(usize)`, `Line(u32)`, `Column(u32)` — not bare primitives.
16. **Strongest type available.** Use `PathBuf` for paths, enums for variants, `Result` for fallibility.

## Error Handling

17. **Panics are for bugs only.** Invariant violations, programming errors. Never for recoverable errors.
18. **Errors are structs, not catch-all enums.** `CssSyntaxError` with message + location + cause.
19. **Error paths return `Result<T, E>`.** Parse errors → `Result` or error recovery into `Raw` nodes.

## Performance

20. **Zero-copy tokenizer.** Tokens reference `&str` slices into source. No `String::from()` on the hot path.
21. **No heap allocation in tight loops.** Per-token and per-character functions must not allocate.
22. **Benchmark before claiming "fast".** Use `criterion` in `benches/`. No performance claims without numbers.
23. **Profile the hot path.** Tokenizer inner loop and parser recursive descent are the hot paths.

## Documentation

24. **First doc sentence: one line, ~15 words.** Becomes the summary in `cargo doc` listings.
25. **Public modules need `//!` docs.** What it contains, when to use it.
26. **Magic values need named constants with doc comments.** Explain *why* this value.

## Testing

27. **Fixture tests load from `tests/fixtures/`** (our owned copy). Use `serde_json` to deserialize.
28. **Test count must match or exceed JS.** Every `it()` in JS → a `#[test]` in Rust.
29. **Edge cases from JS error tests are mandatory.** Error recovery, malformed input, boundary conditions.

## Verification

30. **Every change runs:** `cargo check && cargo test && cargo clippy -- -D warnings`
31. **A module is not "done" until all three pass.**
32. **Lints are configured in `Cargo.toml [lints]`.** Clippy pedantic + perf + correctness all enabled.
