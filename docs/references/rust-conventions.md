# Rust Coding Conventions

Distilled from [Microsoft Pragmatic Rust Guidelines](../../../external/rust-guidelines/) and Rust API Guidelines. These are the rules the agent MUST follow when writing code.

## Error Handling

- **Panics are for bugs only.** Never use panic to communicate errors upstream. Valid panic reasons: programming errors, invariant violations, const contexts.
- **Return `Result<T, E>` for fallible operations.** Use `?` operator for propagation.
- **No `unwrap()` in library code.** Use `expect("reason")` only when the condition is truly impossible (and document why). `unwrap()` is allowed in tests.
- **Error types are structs, not enums.** For this project: a `CssSyntaxError` struct with message, source location, and optional cause. Use `thiserror` if needed.

## Naming

- **No weasel words:** avoid "Manager", "Service", "Factory", "Handler" in type names. Name what it *does*.
- **Conversions follow `as_`/`to_`/`into_` convention:**
  - `as_` — cheap, borrowed → borrowed
  - `to_` — expensive or borrowed → owned
  - `into_` — owned → owned (consumes self)
- **Constructors:** `Foo::new()` for primary constructor. Use `Foo::builder()` if 4+ optional params.
- **Getters:** `fn name(&self) -> &str` not `fn get_name()`.

## Types

- **Derive eagerly:** `#[derive(Debug, Clone, PartialEq)]` on all types. Add `Eq, Hash, Default` where sensible.
- **All public types must implement `Debug`.** No exceptions.
- **Use strongest type available.** `PathBuf` not `String` for paths. Enums not strings for variants.
- **`Option<T>` over sentinel values.** No `-1`, no empty string, no null pattern.
- **Prefer concrete types over generics over `dyn Trait`.**
- **Avoid smart pointers in public APIs.** No `Arc<T>`, `Box<T>`, `Rc<T>` in signatures unless fundamental to purpose.
- **Accept `impl AsRef<str>` where feasible** for functions that take string-like input.

## Documentation

- **First doc sentence: one line, ~15 words.** This becomes the summary in module listings.
- **Public modules need `//!` docs.** What it contains, when to use it.
- **Canonical sections:** `# Examples`, `# Errors` (if Result), `# Panics` (if may panic), `# Safety` (if unsafe).
- **No parameter tables.** Explain params in prose, not bulleted lists.

## Code Quality

- **Static verification is mandatory.** All lints configured in `Cargo.toml [lints]` must pass.
- **Use `#[expect(lint, reason = "...")]` not `#[allow(lint)]`** when overriding lints.
- **Magic values need constants with doc comments** explaining why that value.
- **Prefer regular functions over associated functions** except for constructors and trait impls.
- **Essential functionality is inherent.** Core methods live in `impl Foo`, not behind a trait.

## Performance

- **Identify hot paths early.** Tokenizer inner loop and parser are hot paths for this project.
- **Optimize for throughput.** Items per CPU cycle, not individual item latency.
- **Avoid allocations in tight loops.** No `String::from()`, no `Vec` growth in per-token code.
- **Profile before claiming fast.** Use criterion benchmarks in `benches/`.
- **Common wins:** avoid frequent reallocations, cloned strings, repeated re-hashing, wrong hasher.

## Safety

- **No `unsafe` without justification.** Valid reasons: FFI, performance (with benchmarks), novel abstractions.
- **All code must be sound.** Safe API must never allow UB regardless of input.
- **`unsafe` blocks need `// SAFETY:` comments** explaining why the invariants hold.

## Crate Organization

- **Features are additive.** Adding a feature must not disable or modify existing public items.
- **No glob re-exports.** Use `pub use foo::{A, B, C}` not `pub use foo::*`.
- **Library works out of the box.** `cargo build` must succeed without extra tools or system deps.

## Testing

- **Tests adjacent to code.** `#[cfg(test)] mod tests` at bottom of each file.
- **Test observable behavior, not internals.** Enables refactoring without test rewrites.
- **Feature-gate test utilities.** If we add test helpers to the public API, gate behind `test-util` feature.
